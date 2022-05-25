use crate::FileDB;
use crate::{fdb_trait::GetFileError, File, FileQuery};
use std::collections::btree_set::{BTreeSet, IntoIter};

pub struct TagTreeDBFS {
}

impl TagTreeDBFS {
    pub fn new() -> TagTreeDBFS {
        TagTreeDBFS {}
    }
}

impl FileDB for TagTreeDBFS {
    type FileIterator = IntoIter<File>;

    fn add_file(&mut self, _new_file: &File) -> Option<()> {
        None
    }

    fn get_files<F: FileQuery>(&self, _query: &F) -> Self::FileIterator {
        BTreeSet::<File>::new().into_iter()
    }

    fn get_file<F: FileQuery>(&self, _query: &F) -> Result<File, GetFileError> {
        Err(GetFileError::NoSuchFile)
    }
}
