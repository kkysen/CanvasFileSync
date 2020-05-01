use chrono::{DateTime, Local};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use optional::Optioned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::{fs, io};
use crate::api::diff::Diff;

pub type Id = u64;

#[derive(Serialize, Deserialize)]
pub struct IdName {
    id: Id,
    name: String,
}

pub struct Canvas {
    id: IdName,
    domain: String,
    users: Vec<User>,
}

pub struct User {
    id: IdName,
    created_at: DateTime<Local>,
    courses: Vec<Course>,
}

pub struct Course {
    id: IdName,
    created_at: DateTime<Local>,
    modules: Vec<Module>,
    folder: Directory,
}

pub struct Module {
    id: IdName,
    completed_at: DateTime<Local>,
    files: Vec<RegularFile>,
}

#[derive(Serialize, Deserialize)]
pub struct FileBase {
    id: IdName,
    time: FileTime,
    size: Optioned<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct FileTime {
    created_at: DateTime<Local>,
    updated_at: Option<DateTime<Local>>,
    modified_at: Option<DateTime<Local>>,
}

#[derive(Serialize, Deserialize)]
pub struct Directory {
    pub(crate) base: FileBase,
    pub(crate) files: Vec<File>,
}

#[derive(Serialize, Deserialize)]
pub struct RegularFile {
    base: FileBase,
}

#[derive(Serialize, Deserialize)]
pub enum File {
    Directory(Directory),
    RegularFile(RegularFile),
}

fn to_directories<T: Into<Directory>>(vec: Vec<T>) -> impl Iterator<Item = File> {
    vec.into_iter()
        .map(|it| it.into())
        .map(File::Directory)
}

impl FileTime {
    pub fn created_at(created_at: DateTime<Local>) -> FileTime {
        FileTime {
            created_at,
            updated_at: None,
            modified_at: None,
        }
    }
    
    pub fn modified(&self) -> DateTime<Local> {
        self.modified_at
            .or(self.updated_at)
            .unwrap_or(self.created_at)
    }
    
    pub fn convert(time: DateTime<Local>) -> filetime::FileTime {
        let seconds = time.timestamp();
        let nanoseconds = time.timestamp_subsec_nanos();
        filetime::FileTime::from_unix_time(seconds, nanoseconds)
    }
}

impl Default for FileTime {
    fn default() -> Self {
        FileTime::created_at(Local::now())
    }
}

impl FileBase {
    pub fn directory(id: IdName, time: DateTime<Local>) -> FileBase {
        FileBase {
            id,
            time: FileTime::created_at(time),
            size: Optioned::none(),
        }
    }
    
    pub fn into_file(self) -> RegularFile {
        RegularFile {
            base: self,
        }
    }
    
    pub fn into_directory(self, files: Vec<File>) -> Directory {
        Directory {
            base: self,
            files,
        }
    }
}

impl From<Canvas> for Directory {
    fn from(canvas: Canvas) -> Self {
        let Canvas {
            id,
            domain: _,
            users,
        } = canvas;
        Directory {
            base: FileBase::directory(id, Local::now()),
            files: to_directories(users).collect(),
        }
    }
}

impl From<User> for Directory {
    fn from(user: User) -> Self {
        let User {
            id,
            created_at,
            courses,
        } = user;
        Directory {
            base: FileBase::directory(id, created_at),
            files: to_directories(courses).collect(),
        }
    }
}

impl From<Course> for Directory {
    fn from(course: Course) -> Self {
        let Course {
            id,
            created_at,
            modules,
            folder,
        } = course;
        let mut files = Vec::with_capacity(1 + modules.len());
        files.push(File::Directory(folder));
        files.extend(to_directories(modules));
        Directory {
            base: FileBase::directory(id, created_at),
            files,
        }
    }
}

impl From<Module> for Directory {
    fn from(module: Module) -> Self {
        let Module {
            id,
            completed_at,
            files,
        } = module;
        Directory {
            base: FileBase::directory(id, completed_at),
            files: files
                .into_iter()
                .map(File::RegularFile)
                .collect(),
        }
    }
}

pub struct Download {
    file: FileBase,
    path: PathBuf,
}

impl FileBase {
    pub fn to_path(&self, path: &Path) -> PathBuf {
        let mut path = path.to_owned();
        path.push(&self.id.name);
        path
    }
    
    pub fn into_download(self, path: &Path) -> Download {
        let path = self.to_path(path);
        Download {
            file: self,
            path,
        }
    }
}

pub trait GetFileBase {
    fn base(&self) -> &FileBase;
    
    fn id(&self) -> u64 {
        self.base().id.id
    }
    
    fn is_newer_than(&self, other: &impl GetFileBase) -> bool {
        self.base().time.modified() > other.base().time.modified()
    }
}

impl GetFileBase for Directory {
    fn base(&self) -> &FileBase {
        &self.base
    }
}

impl GetFileBase for RegularFile {
    fn base(&self) -> &FileBase {
        &self.base
    }
}

impl GetFileBase for File {
    fn base(&self) -> &FileBase {
        match &self {
            File::Directory(dir) => dir.base(),
            File::RegularFile(file) => file.base(),
        }
    }
}

impl Directory {
    pub(crate) fn id_to_file_map(&self) -> HashMap<Id, &File> {
        self.files
            .iter()
            .map(|file| (file.id(), file))
            .collect()
    }
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
                    let download = file.base.into_download(path);
                    Downloads::add(downloads_immut, downloads_mut, download, false);
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

impl Download {
    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }
    
    pub fn set_time(&self) -> io::Result<()> {
        let mtime = self.file.time.modified();
        let mtime = FileTime::convert(mtime);
        filetime::set_file_times(self.path(), mtime, mtime)?;
        Ok(())
    }
    
    pub fn download_as_directory(&self) -> io::Result<()> {
        let path = self.path();
        if let Err(e) = fs::create_dir(path) {
            let exists = e.kind() == io::ErrorKind::AlreadyExists;
            let dir_exists = exists && path.is_dir();
            if !dir_exists {
                return Err(e);
            }
        }
        self.set_time()?;
        Ok(())
    }
    
    pub async fn download_as_file(&self) -> io::Result<()> {
        todo!();
        self.set_time()?;
        Ok(())
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
        todo!();
        // TODO await in parallel
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

pub struct DownloadsMut {
    directories: Vec<Download>,
    files: Vec<Download>,
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
