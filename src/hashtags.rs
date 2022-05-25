use crate::FileDB;
use crate::{fdb_trait::GetFileError, File, FileQuery};
use std::collections::btree_set::{BTreeSet, IntoIter};
use std::collections::hash_map::HashMap;

pub struct HashTagsDBFS {
    files: HashMap<String, BTreeSet<File>>,
}

impl HashTagsDBFS {
    pub fn new() -> HashTagsDBFS {
        HashTagsDBFS {
            files: HashMap::new(),
        }
    }
}

impl FileDB for HashTagsDBFS {
    type FileIterator = IntoIter<File>;

    fn add_file(&mut self, new_file: &File) -> Option<()> {
        match self.get_file(new_file) {
            Ok(_) => None,
            Err(GetFileError::TooManyFiles) => None,
            Err(GetFileError::NoSuchFile) => {
                for t in new_file.tags() {
                    match self.files.get_mut(t) {
                        Some(map) => {
                            map.insert(new_file.clone());
                        }
                        None => {
                            let mut new_tag_files = BTreeSet::new();
                            new_tag_files.insert(new_file.clone());
                            self.files.insert(t.to_string(), new_tag_files);
                        }
                    }
                }
                Some(())
            }
        }
    }

    fn get_files<F: FileQuery>(&self, query: &F) -> Self::FileIterator {
        let mut iter = query.tags().into_iter();

        // Initialise our union with the first value; initialising with
        // an empty set won't work, as union with empty is empty.
        let mut result = self.files[iter.next().unwrap()].clone();

        // Iterate through the rest of the elements
        for t in iter {
            if !self.files.contains_key(t) {
                // If we found a single key that doesn't exist
                return BTreeSet::new().into_iter();
            }

            // Union is lazy, and doesn't support chaining, so we need the
            // whole thing, so we have to clone and collect.
            result = result.union(&self.files[t]).cloned().collect();
        }

        result.into_iter()
    }

    fn get_file<F: FileQuery>(&self, query: &F) -> Result<File, GetFileError> {
        // Just fail if the query isn't a well formed file.
        let mut result: Result<File, GetFileError> = Err(GetFileError::NoSuchFile);

        for k in query.tags() {
            match self.files.get(k) {
                Some(file_set) => {
                    for f in file_set {
                        if query.could_match(&f) {
                            match &result {
                                Ok(current) => {
                                    if current != f {
                                        // At least one tag has multiple matches, so we fail.
                                        return Err(GetFileError::TooManyFiles);
                                    }
                                }
                                Err(GetFileError::NoSuchFile) => {
                                    result = Ok(f.clone());
                                }
                                Err(GetFileError::TooManyFiles) => {}
                            }
                        }
                    }
                }
                None => {
                    // If the tag doesn't exist, then we must
                    // not have this file.
                    return Err(GetFileError::NoSuchFile);
                }
            }
        }
        // Either we found a single match, or no match at all, so return what
        // we have.
        result
    }
}
