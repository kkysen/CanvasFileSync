use crate::api::data::{GetFileBase, FileBase, FileTime};
use std::path::{PathBuf, Path};
use chrono::{DateTime, Local};
use std::{io, fs};

pub struct Download {
    file: FileBase,
    pub(crate) path: PathBuf,
}

pub(crate) trait GetFileBaseExt: GetFileBase {
    fn to_path(&self, path: &Path) -> PathBuf {
        let mut path = path.to_owned();
        path.push(&self.base().id.name);
        path
    }
    
    fn into_download(self, path: &Path) -> Download {
        let path = self.to_path(path);
        Download {
            file: self.into_base(),
            path,
        }
    }
}

impl<T: GetFileBase> GetFileBaseExt for T {}

impl FileTime {
    fn convert(time: DateTime<Local>) -> filetime::FileTime {
        let seconds = time.timestamp();
        let nanoseconds = time.timestamp_subsec_nanos();
        filetime::FileTime::from_unix_time(seconds, nanoseconds)
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
