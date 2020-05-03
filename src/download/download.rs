use crate::download::data::{GetFileBase, FileBase, FileTime};
use std::path::{PathBuf, Path};
use chrono::{DateTime, Local};
use std::error::Error;
use crate::api::core::CoreApi;

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
    
    fn modification_time(&self) -> filetime::FileTime {
        FileTime::convert(self.file.time.modified())
    }
    
    fn set_time(&self) -> std::io::Result<()> {
        let mtime = self.modification_time();
        filetime::set_file_times(self.path(), mtime, mtime)?;
        Ok(())
    }
    
    pub(crate) fn download_as_directory(&self) -> std::io::Result<()> {
        let path = self.path();
        if let Err(e) = std::fs::create_dir(path) {
            let exists = e.kind() == std::io::ErrorKind::AlreadyExists;
            let dir_exists = exists && path.is_dir();
            if !dir_exists {
                return Err(e);
            }
        }
        self.set_time()?;
        Ok(())
    }
    
    pub(crate) async fn download_as_file(&self, api: &CoreApi) -> Result<(), Box<dyn Error>> {
        let mut file = async_std::fs::File::open(self.path()).await?;
        let mut resp = api.download(&self.file.id()).await?;
        async_std::io::copy(&mut resp, &mut file).await?;
        self.set_time()?;
        Ok(())
    }
    
    pub(crate) async fn download_as_file_into(self, api: &CoreApi) -> Result<Self, Box<dyn Error>> {
        self.download_as_file(api).await?;
        Ok(self)
    }
}
