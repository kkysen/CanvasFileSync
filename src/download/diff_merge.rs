use crate::download::data::{RegularFile, GetFileBase, File, Directory, Id, FileTree};
use std::collections::HashMap;

trait GetFileBaseExt: GetFileBase {
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
    
    fn id_to_file_map_mut(&mut self) -> HashMap<Id, File> {
        self.files
            .drain(..)
            .map(|file| (file.id(), file))
            .collect()
    }
}

pub(crate) trait Diff where Self: Sized {
    fn diff(self, old: &Self) -> Option<Self>;
}

impl Diff for FileTree {
    fn diff(self, old: &Self) -> Option<Self> {
        let Self { domain, root } = self;
        Some(root)
            .map(|new| (new, &old.root))
            .filter(|(new, old)| {
                assert_eq!(new.id(), old.id());
                true
            })
            .and_then(|(new, old)| new.diff(old))
            .map(|root| Self { domain, root })
    }
}

// forced to export this
pub(crate) trait FileDiff where Self: Sized + GetFileBase {
    fn diff_id_unchecked(self, old: &Self) -> Option<Self>;
}

impl<T: FileDiff> Diff for T {
    fn diff(self, old: &Self) -> Option<Self> {
        Some(self)
            .filter(|new| new.id() == old.id())
            .and_then(|new| new.diff_id_unchecked(old))
    }
}

impl FileDiff for Directory {
    fn diff_id_unchecked(self, old: &Self) -> Option<Self> {
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
                        Some(old_file) => new_file.diff(old_file),
                    })
                    .collect();
                new.base.into_directory(files)
            })
    }
}

impl FileDiff for RegularFile {
    fn diff_id_unchecked(self, old: &Self) -> Option<Self> {
        assert_eq!(self.id(), old.id());
        Some(self)
            .filter(|it| it.is_newer_than(old))
    }
}

impl Diff for File {
    fn diff(self, old: &Self) -> Option<Self> {
        match (self, old) {
            (File::Directory(new), File::Directory(old)) => {
                new.diff_id_unchecked(old)
                    .map(File::Directory)
            }
            (File::RegularFile(new), File::RegularFile(old)) => {
                new.diff_id_unchecked(old)
                    .map(File::RegularFile)
            }
            (_, _) => {
                debug_assert!(false, "diff'ed Directory with RegularFile");
                None
            }
        }
    }
}


pub(crate) trait Merge where Self: Sized {
    // diff should already be a diff produced by other.diff(self)
    // diff should only contains Files not in self or newer than those in self
    fn merge(&mut self, diff: Self);
}

impl Merge for FileTree {
    fn merge(&mut self, diff: Self) {
        self.root.merge(diff.root);
    }
}

impl Merge for Directory {
    fn merge(&mut self, diff: Self) {
        self.base.time = diff.base.time;
        let mut old_files_map = self.id_to_file_map_mut();
        // can't push and merge at the same time b/c that'd use two mut borrows
        // so I rebuild whole self.files at once
        self.files = diff.files
            .into_iter()
            .map(|new_file| match old_files_map.remove(&new_file.id()) {
                None => new_file,
                Some(mut old_file) => {
                    old_file.merge(new_file);
                    old_file
                },
            })
            .collect();
    }
}

impl Merge for RegularFile {
    fn merge(&mut self, diff: Self) {
        let old = self.base_mut();
        let new = diff.into_base();
        old.time = new.time;
        old.size = new.size;
    }
}

impl Merge for File {
    fn merge(&mut self, diff: Self) {
        match (self, diff) {
            (File::Directory(old), File::Directory(new)) => old.merge(new),
            (File::RegularFile(old), File::RegularFile(new)) => old.merge(new),
            (_, _) => {
                panic!("merged Directory with RegularFile")
            }
        }
    }
}
