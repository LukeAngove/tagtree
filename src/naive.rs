use std::collections::hash_set::HashSet;
use crate::file::{File, TagSet};

pub struct DBFS {
    files: HashSet<File>,
}

impl DBFS {
    pub fn new() -> DBFS {
        DBFS {
            files: HashSet::new(),
        }
    }

    pub fn add_files(&mut self, new_files: &HashSet<File>) {
        for f in new_files {
            self.files.insert(f.clone());
        }
    }

    pub fn get_files(&self, query: &str) -> HashSet<File> {
        let mut result = HashSet::new();
        let query_tags = TagSet::from_iter(query.split("/").filter(|x| x != &""));

        for f in &self.files {
            if f.has_tags(&query_tags) {
                result.insert(f.clone());
            }
        }
        result
    }
}
