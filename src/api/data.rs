use chrono::{DateTime, Local, NaiveDateTime};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use itertools::{Either, Itertools};
use std::borrow::Borrow;
use std::{io, fs};
use std::path::{Path, PathBuf};
use optional::Optioned;
use std::error::Error;
use std::collections::HashMap;

pub type Id = u64;

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

pub struct FileBase {
    id: IdName,
    time: FileTime,
    size: Optioned<u64>,
}

pub struct FileTime {
    created_at: DateTime<Local>,
    updated_at: Option<DateTime<Local>>,
    modified_at: Option<DateTime<Local>>,
}

pub struct Directory {
    base: FileBase,
    files: Vec<File>,
}

pub struct RegularFile {
    base: FileBase,
}

pub enum File {
    Directory(Directory),
    RegularFile(RegularFile),
}

fn to_directories<T: Into<Directory>>(vec: Vec<T>) -> impl Iterator<Item = File> {
    vec.into_iter()
        .map(|it| it.into())
        .map(|it: Directory| File::Directory(it))
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
        filetime::FileTime::from_unix_time(time.timestamp(), time.timestamp_subsec_nanos())
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
            base: FileBase::directory(id, created_at),
            files: files.into_iter().map(|it| File::RegularFile(it)).collect(),
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
        path.push(self.base.id.name);
        path
    }
    
    pub fn into_download(self, path: &Path) -> Download {
        Download {
            file: self,
            path: self.to_path(path),
        }
    }
}

trait GetFileBase {
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

trait Diff {
    fn diff(self, old: &Self) -> Option<Self>;
}

impl Directory {
    fn id_to_file_map(self) -> HashMap<Id, File> {
        self.files
            .into_iter()
            .map(|file| (file.id(), file))
            .collect()
    }
}

impl Diff for Directory {
    fn diff(self, old: &Self) -> Option<Self> {
        assert_eq!(self.id(), old.id());
        // TODO check if in canvas the directory time is updated when one of its files is updated
        if self.is_newer_than(old) {
            return None;
        }
        let file_map = self.id_to_file_map();
        let new_files = old.files
            .iter()
            .filter_map(|old_file|
                file_map.get(&old_file.id()).map(|new_file| (new_file, old_file))
            )
            .filter_map(|(new, old)| new.diff(old))
            .collect();
        let new_dir = Directory {
            files: new_files,
            ..self
        };
        Some(new_dir)
    }
}

impl Diff for RegularFile {
    fn diff(self, old: &Self) -> Option<Self> {
        assert_eq!(self.id(), old.id());
        Some(self)
            .filter(|it| it.is_newer_than(old))
    }
}

impl Diff for File {
    fn diff(self, old: &Self) -> Option<Self> {
        match (self, old) {
            (File::Directory(new), File::Directory(old)) =>
                new.diff(old).map(File::Directory),
            (File::RegularFile(new), File::RegularFile(old)) =>
                new.diff(old).map(File::RegularFile),
            (_, _) => panic!("diff'ed Directory with RegularFile")
        }
    }
}

impl Directory {
    fn add_to_downloads_internal(self, path: &Path, downloads: &mut Downloads) {
        let download = self.base.into_download(path);
        if !downloads.add(download, true) {
            return;
        }
        let path = download.path();
        for file in self.files {
            match file {
                File::Directory(dir) => {
                    dir.into_downloads(path, downloads);
                }
                File::RegularFile(file) => {
                    downloads.add(file.base.into_download(path), false);
                }
            }
        }
    }
    
    pub fn add_to_downloads(self, downloads: &mut Downloads) {
        self.add_to_downloads_internal(downloads.root(), downloads);
    }
}

impl Downloads {
    pub fn add_file_tree(&mut self, dir: Directory) {
        dir.diff(&self.current).add_to_downloads(self)
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
        let path = dir.path();
        if let Err(e) = fs::create_dir(path) {
            let exists = e.kind() == io::ErrorKind::AlreadyExists;
            let dir_exists = exists && path.is_dir();
            if !dir_exists {
                return Err(e);
            }
        }
        dir.set_time()?;
        Ok(())
    }
    
    pub async fn download_as_file(&self) -> io::Result<()> {
        todo!();
        self.set_time();
        Ok(())
    }
}

impl Downloads {
    pub fn create_directories(&self) -> io::Result<()> {
        for dir in &self.directories {
            dir.download_as_directory()?;
        }
        Ok(())
    }
    
    pub async fn download_files(&self) -> io::Result<()> {
        todo!();
        // TODO await in parallel
        self.files
            .iter()
            .map(|file| file.download_as_file());
        Ok(())
    }
    
    pub async fn download(&self) -> io::Result<()> {
        self.create_directories()?;
        self.download_files().await?;
        Ok(())
    }
}

impl Directory {
    pub fn from_path(path: &Path) -> Result<Directory, dyn Error> {
        let bytes = fs::read(path)?;
        serde_json::from_slice(bytes.as_ref())
    }
    
    pub fn from_dir(dir: &Path) -> Result<Directory, dyn Error> {
        let mut dir = dir.to_owned();
        dir.push("file_tree.json");
        Directory::from_path(dir.as_ref())
    }
}

pub struct Downloads {
    root: PathBuf,
    ignore: Gitignore,
    current: Directory,
    directories: Vec<Download>,
    files: Vec<Download>,
}

impl<'a> Downloads {
    pub fn new(root: PathBuf) -> Result<Downloads, dyn Error> {
        Ok(Downloads {
            root,
            ignore: GitignoreBuilder::new(root.as_ref()).build()?,
            current: Directory::from_dir(root.as_ref())?,
            directories: Vec::new(),
            files: Vec::new(),
        })
    }
    
    pub fn root(&self) -> &Path {
        self.root.as_ref()
    }
    
    fn downloads(&mut self, is_dir: bool) -> &mut Vec<Download> {
        match is_dir {
            true => &mut self.directories,
            false => &mut self.files,
        }
    }
    
    fn can_add(&self, download: &Download, is_dir: bool) -> bool {
        !self.ignore.matched(download.path.as_ref(), is_dir).is_ignore()
    }
    
    pub fn add(&mut self, download: Download, is_dir: bool) -> bool {
        let added = self.can_add(&download, is_dir);
        if added {
            self.downloads(is_dir).push(download)
        }
        added
    }
}
