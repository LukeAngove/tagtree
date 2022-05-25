use crate::FileDB;
use crate::{fdb_trait::GetFileError, File, FileQuery};
use std::collections::hash_set::{HashSet, IntoIter};

pub struct NaiveDBFS {
    files: HashSet<File>,
}

impl NaiveDBFS {
    pub fn new() -> NaiveDBFS {
        NaiveDBFS {
            files: HashSet::new(),
        }
    }
}

impl FileDB for NaiveDBFS {
    type FileIterator = IntoIter<File>;

    fn add_file(&mut self, new_file: &File) -> Option<()> {
        if self.files.contains(new_file) {
            None
        } else {
            self.files.insert(new_file.clone());
            Some(())
        }
    }

    fn get_files<F: FileQuery>(&self, query: &F) -> Self::FileIterator {
        let mut result = HashSet::new();

        for f in &self.files {
            if query.could_match(&f) {
                result.insert(f.clone());
            }
        }
        result.into_iter()
    }

    fn get_file<F: FileQuery>(&self, query: &F) -> Result<File, GetFileError> {
        // Just fail if the query isn't a well formed file.
        let mut result: Result<File, GetFileError> = Err(GetFileError::NoSuchFile);

        for f in &self.files {
            if query.could_match(&f) {
                if result == Err(GetFileError::NoSuchFile) {
                    result = Ok(f.clone());
                } else {
                    // Early return, as if there are two matches
                    // then we know this is a failure; we only want
                    // one file.
                    return Err(GetFileError::TooManyFiles);
                }
            }
        }
        // Either we found a single match, or no match at all, so return what
        // we have.
        result
    }
}
