use crate::api::data::{RegularFile, GetFileBase, File, Directory, Id};
use std::collections::HashMap;

trait GetFileBaseExt: GetFileBase {
    fn id(&self) -> u64 {
        self.base().id.id
    }
    
    fn is_newer_than(&self, other: &impl GetFileBase) -> bool {
        self.base().time.modified() > other.base().time.modified()
    }
}

impl<T: GetFileBase> GetFileBaseExt for T {}

impl Directory {
    fn id_to_file_map(&self) -> HashMap<Id, &File> {
        self.files
            .iter()
            .map(|file| (file.id(), file))
            .collect()
    }
}

pub(crate) trait Diff where Self: Sized + GetFileBase {
    fn diff_id_unchecked(self, old: &Self) -> Option<Self>;
    
    fn diff(self, old: &Self) -> Option<Self> {
        Some(self)
            .filter(|new| new.id() == old.id())
            .and_then(|new| new.diff_id_unchecked(old))
    }
}

impl Diff for Directory {
    fn diff_id_unchecked(self, old: &Self) -> Option<Self> {
        assert_eq!(self.id(), old.id());
        // TODO check if in canvas the directory time is updated when one of its files is updated
        Some(self)
            .filter(|new| new.is_newer_than(old))
            .map(|new| {
                let old_files_map = old.id_to_file_map();
                let files = new
                    .files
                    .into_iter()
                    .filter_map(|new_file| match old_files_map.get(&new_file.id()) {
                        None => Some(new_file),
                        Some(old_file) => new_file.diff_id_unchecked(old_file),
                    })
                    .collect();
                new.base.into_directory(files)
            })
    }
}

impl Diff for RegularFile {
    fn diff_id_unchecked(self, old: &Self) -> Option<Self> {
        assert_eq!(self.id(), old.id());
        Some(self)
            .filter(|it| it.is_newer_than(old))
    }
}

impl Diff for File {
    fn diff_id_unchecked(self, old: &Self) -> Option<Self> {
        match (self, old) {
            (File::Directory(new), File::Directory(old)) => new.diff_id_unchecked(old).map(File::Directory),
            (File::RegularFile(new), File::RegularFile(old)) => {
                new.diff_id_unchecked(old).map(File::RegularFile)
            }
            (_, _) => {
                debug_assert!(false, "diff'ed Directory with RegularFile");
                panic!("diff'ed Directory with RegularFile")
            }
        }
    }
}
