use std::path::{PathBuf, Path};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use crate::api::data::{Directory, File};
use crate::api::download::{Download, GetFileBaseExt};
use crate::api::diff::Diff;
use std::{io, fs};
use std::error::Error;

// need to separate into immut and mut parts
pub struct Downloads {
    immut: DownloadsImmut,
    r#mut: DownloadsMut,
}

pub struct DownloadsImmut {
    root: PathBuf,
    ignore: Gitignore,
    current: Directory,
}

pub struct DownloadsMut {
    directories: Vec<Download>,
    files: Vec<Download>,
}

impl Directory {
    fn add_to_downloads_internal(
        self,
        path: &Path,
        downloads_immut: &DownloadsImmut,
        downloads_mut: &mut DownloadsMut,
    ) {
        let download = self.base.into_download(path);
        let path = download.path.clone();
        let path = path.as_path();
        if !Downloads::add(downloads_immut, downloads_mut, download, true) {
            return;
        }
        for file in self.files {
            match file {
                File::Directory(dir) => {
                    dir.add_to_downloads_internal(path, downloads_immut, downloads_mut);
                }
                File::RegularFile(file) => {
                    Downloads::add(
                        downloads_immut, downloads_mut,
                        file.into_download(path), false,
                    );
                }
            }
        }
    }
    
    pub fn add_to_downloads(self, downloads: &mut Downloads) {
        let downloads_immut = &downloads.immut;
        let downloads_mut = &mut downloads.r#mut;
        self.add_to_downloads_internal(downloads_immut.root(), downloads_immut, downloads_mut);
    }
}

impl Downloads {
    pub fn add_file_tree(&mut self, dir: Directory) {
        if let Some(dir) = dir.diff(&self.immut.current) {
            dir.add_to_downloads(self);
        }
    }
}

impl Downloads {
    pub fn create_directories(&self) -> io::Result<()> {
        for dir in &self.r#mut.directories {
            dir.download_as_directory()?;
        }
        Ok(())
    }
    
    pub async fn download_files(&self) -> io::Result<()> {
        let download_futures = self
            .r#mut.files
            .iter()
            .map(|file| file.download_as_file());
        futures::future::join_all(download_futures).await;
        Ok(())
    }
    
    pub async fn download(&self) -> io::Result<()> {
        self.create_directories()?;
        self.download_files().await?;
        Ok(())
    }
}

impl Directory {
    fn default_path(dir: &Path) -> PathBuf {
        let mut dir = dir.to_owned();
        dir.push("file_tree.json");
        dir
    }
    
    pub fn from_path(path: &Path) -> Result<Directory, Box<dyn Error>> {
        let bytes = fs::read(path)?;
        let dir = serde_json::from_slice(bytes.as_ref())?;
        Ok(dir)
    }
    
    pub fn from_dir(dir: &Path) -> Result<Directory, Box<dyn Error>> {
        let dir = Directory::default_path(dir);
        Directory::from_path(dir.as_ref())
    }
    
    pub fn to_path(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let bytes = serde_json::to_vec_pretty(self)?;
        fs::write(path, bytes)?;
        Ok(())
    }
    
    pub fn to_dir(&self, dir: &Path) -> Result<(), Box<dyn Error>> {
        let dir = Directory::default_path(dir);
        self.to_path(dir.as_ref())
    }
}

impl DownloadsImmut {
    fn new(root: PathBuf) -> Result<Self, Box<dyn Error>> {
        let root_ref = root.as_ref();
        let ignore = GitignoreBuilder::new(root_ref).build()?;
        let current = Directory::from_dir(root_ref)?;
        Ok(Self {
            root,
            ignore,
            current,
        })
    }
    
    pub fn root(&self) -> &Path {
        self.root.as_ref()
    }
    
    fn can_add(&self, download: &Download, is_dir: bool) -> bool {
        !self.ignore
            .matched(download.path.as_path(), is_dir)
            .is_ignore()
    }
}

impl DownloadsMut {
    fn new() -> Self {
        Self {
            directories: Vec::new(),
            files: Vec::new(),
        }
    }
    
    fn downloads(&mut self, is_dir: bool) -> &mut Vec<Download> {
        match is_dir {
            true => &mut self.directories,
            false => &mut self.files,
        }
    }
    
    fn add(&mut self, download: Download, is_dir: bool) {
        self.downloads(is_dir).push(download)
    }
}

impl<'a> Downloads {
    pub fn new(root: PathBuf) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            immut: DownloadsImmut::new(root)?,
            r#mut: DownloadsMut::new(),
        })
    }
    
    fn add(
        self_immut: &DownloadsImmut, self_mut: &mut DownloadsMut,
        download: Download, is_dir: bool,
    ) -> bool {
        let added = self_immut.can_add(&download, is_dir);
        if added {
            self_mut.add(download, is_dir)
        }
        added
    }
}
