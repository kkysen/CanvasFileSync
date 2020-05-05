use chrono::{DateTime, Local};
use optional::Optioned;
use serde::{Deserialize, Serialize};
use crate::api::core::CoreApi;
use std::fmt::Display;
use serde::export::Formatter;
use std::fmt;

pub type Id = u64;

#[derive(Serialize, Deserialize, Clone)]
pub struct IdName {
    pub id: Id,
    pub name: String,
}

pub struct CanvasBase {
    pub api: CoreApi,
    pub id: IdName,
}

pub struct Canvas {
    base: CanvasBase,
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

#[derive(Serialize, Deserialize, Clone)]
pub struct FileBase {
    pub(crate) id: IdName,
    pub(crate) time: FileTime,
    pub(crate) size: Optioned<u64>,
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize)]
pub struct FileTree {
    pub(crate) api: CoreApi,
    pub(crate) root: Directory,
}

impl Display for CanvasBase {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}", self.api.domain, self.id.name)?;
        Ok(())
    }
}

fn to_directories<T: Into<Directory>>(vec: Vec<T>) -> impl Iterator<Item = File> {
    vec.into_iter()
        .map(|it| it.into())
        .map(File::Directory)
}

impl FileTime {
    fn created_at(created_at: DateTime<Local>) -> FileTime {
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
}

impl Default for FileTime {
    fn default() -> Self {
        FileTime::created_at(Local::now())
    }
}

impl FileBase {
    fn directory(id: IdName, time: DateTime<Local>) -> FileBase {
        FileBase {
            id,
            time: FileTime::created_at(time),
            size: Optioned::none(),
        }
    }
    
    pub(crate) fn into_file(self) -> RegularFile {
        RegularFile {
            base: self,
        }
    }
    
    pub(crate) fn into_directory(self, files: Vec<File>) -> Directory {
        Directory {
            base: self,
            files,
        }
    }
}

impl From<Canvas> for FileTree {
    fn from(canvas: Canvas) -> Self {
        let Canvas {
            base: CanvasBase {
                id,
                api,
            },
            users,
        } = canvas;
        Self {
            api,
            root: Directory {
                base: FileBase::directory(id, Local::now()),
                files: to_directories(users).collect(),
            },
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
        Self {
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
        Self {
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
        Self {
            base: FileBase::directory(id, completed_at),
            files: files
                .into_iter()
                .map(File::RegularFile)
                .collect(),
        }
    }
}

pub(crate) trait GetFileBase where Self: Sized {
    fn base(&self) -> &FileBase;
    
    fn base_mut(&mut self) -> &mut FileBase;
    
    fn into_base(self) -> FileBase;
    
    fn id(&self) -> u64 {
        self.base().id.id
    }
}

impl GetFileBase for FileBase {
    fn base(&self) -> &FileBase {
        self
    }
    
    fn base_mut(&mut self) -> &mut FileBase {
        self
    }
    
    fn into_base(self) -> FileBase {
        self
    }
}

impl GetFileBase for Directory {
    fn base(&self) -> &FileBase {
        &self.base
    }
    
    fn base_mut(&mut self) -> &mut FileBase {
        &mut self.base
    }
    
    fn into_base(self) -> FileBase {
        self.base
    }
}

impl GetFileBase for RegularFile {
    fn base(&self) -> &FileBase {
        &self.base
    }
    
    fn base_mut(&mut self) -> &mut FileBase {
        &mut self.base
    }
    
    fn into_base(self) -> FileBase {
        self.base
    }
}

impl GetFileBase for File {
    fn base(&self) -> &FileBase {
        match self {
            File::Directory(dir) => dir.base(),
            File::RegularFile(file) => file.base(),
        }
    }
    
    fn base_mut(&mut self) -> &mut FileBase {
        match self {
            File::Directory(dir) => dir.base_mut(),
            File::RegularFile(file) => file.base_mut(),
        }
    }
    
    fn into_base(self) -> FileBase {
        match self {
            File::Directory(dir) => dir.into_base(),
            File::RegularFile(file) => file.into_base(),
        }
    }
}
