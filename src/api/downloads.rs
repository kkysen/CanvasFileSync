use std::path::{PathBuf, Path};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use crate::api::data::{Directory, File, FileTree};
use crate::api::download::{Download, GetFileBaseExt};
use crate::api::diff::Diff;
use std::{io, fs};
use std::error::Error;
use crate::util;
use crate::util::future::FutureIterator;
use std::io::Write;

// need to separate into immut and mut parts
pub struct Downloads {
    immut: DownloadsImmut,
    r#mut: DownloadsMut,
}

pub struct DownloadsImmut {
    root: PathBuf,
    ignore: Gitignore,
    file_tree_file: std::fs::File,
    current_file_tree: FileTree,
}

pub struct DownloadsMut {
    directories: Vec<Download>,
    files: Vec<Download>,
}


impl DownloadsImmut {
    fn new(root: PathBuf) -> Result<Self, Box<dyn Error>> {
        let ignore = GitignoreBuilder::new(root.as_path()).build()?;
        let file_tree_path = {
            let mut path = root.clone();
            path.push("file_tree.json");
            path
        };
        let mut file_tree_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .create(true)
            .open(file_tree_path)?;
        let file_tree_bytes = util::fs::read_all(&mut file_tree_file)?;
        let current_file_tree = serde_json::from_slice(file_tree_bytes.as_ref())?;
        Ok(Self {
            root,
            ignore,
            file_tree_file,
            current_file_tree,
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
    
    fn save_current_file_tree(&mut self) -> Result<(), Box<dyn Error>> {
        let bytes = serde_json::to_vec_pretty(&self.current_file_tree)?;
        self.file_tree_file.write_all(bytes.as_ref())?;
        Ok(())
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

impl Downloads {
    pub fn new(root: PathBuf) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            immut: DownloadsImmut::new(root)?,
            r#mut: DownloadsMut::new(),
        })
    }
    
    fn add_download(
        self_immut: &DownloadsImmut, self_mut: &mut DownloadsMut,
        download: Download, is_dir: bool,
    ) -> bool {
        let added = self_immut.can_add(&download, is_dir);
        if added {
            self_mut.add(download, is_dir)
        }
        added
    }
    
    fn add_directory(
        self_immut: &DownloadsImmut, self_mut: &mut DownloadsMut,
        dir: Directory, path: &Path,
    ) {
        let download = dir.base.into_download(path);
        let path = download.path.clone();
        let path = path.as_path();
        if !Self::add_download(
            self_immut, self_mut,
            download, true,
        ) {
            return;
        }
        for file in dir.files {
            match file {
                File::Directory(dir) => {
                    Self::add_directory(
                        self_immut, self_mut,
                        dir, path,
                    );
                }
                File::RegularFile(file) => {
                    Self::add_download(
                        self_immut, self_mut,
                        file.into_download(path), false,
                    );
                }
            }
        }
    }
    
    pub fn add_file_tree(&mut self, file_tree: FileTree) -> Result<(), Box<dyn Error>> {
        let diff = match file_tree.diff(&self.immut.current_file_tree) {
            None => return Ok(()),
            Some(it) => it,
        };
        Self::add_directory(
            &self.immut, &mut self.r#mut,
            diff.root, self.immut.root(),
        );
        // TODO merge diff and current
        // set current to merged
        self.immut.save_current_file_tree()?;
        Ok(())
    }
    
    pub fn create_directories(&mut self) -> io::Result<()> {
        for dir in self
            .r#mut.directories
            .drain(0..) {
            dir.download_as_directory()?;
        }
        Ok(())
    }
    
    pub async fn download_files(&mut self) -> Result<(), Box<dyn Error>> {
        let domain = &self.immut.current_file_tree.domain;
        for result in self
            .r#mut.files
            .drain(0..)
            .map(|file| file.download_as_file_into(domain))
            .join_all()
            .await {
            result?;
        }
        Ok(())
    }
    
    pub async fn download(&mut self) -> Result<(), Box<dyn Error>> {
        self.create_directories()?;
        self.download_files().await?;
        Ok(())
    }
}

// TODO remove
impl FileTree {
    fn default_path(dir: &Path) -> PathBuf {
        let mut dir = dir.to_owned();
        dir.push("file_tree.json");
        dir
    }
    
    pub fn from_path(path: &Path) -> Result<Self, Box<dyn Error>> {
        let bytes = fs::read(path)?;
        let dir = serde_json::from_slice(bytes.as_ref())?;
        Ok(dir)
    }
    
    pub fn from_dir(dir: &Path) -> Result<Self, Box<dyn Error>> {
        let dir = Self::default_path(dir);
        Self::from_path(dir.as_ref())
    }
    
    pub fn to_path(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let bytes = serde_json::to_vec_pretty(self)?;
        fs::write(path, bytes)?;
        Ok(())
    }
    
    pub fn to_dir(&self, dir: &Path) -> Result<(), Box<dyn Error>> {
        let dir = Self::default_path(dir);
        self.to_path(dir.as_ref())
    }
}
