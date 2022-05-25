use crate::FileDB;
use crate::TagSet;
use crate::{fdb_trait::GetFileError, File, FileQuery};
use std::collections::btree_set::{BTreeSet, IntoIter};
use std::collections::hash_map::HashMap;

pub struct HashTags2DBFS {
    files: HashMap<String, BTreeSet<String>>,
}

impl HashTags2DBFS {
    pub fn new() -> HashTags2DBFS {
        HashTags2DBFS {
            files: HashMap::new(),
        }
    }

    // Assumes we've already checked all keys in query exist.
    fn get_tags_not_in_query<F: FileQuery>(&self, query: &F) -> BTreeSet<String> {
        self.files
            .keys()
            .filter(|&x| !query.tags().contains(x))
            .cloned()
            .collect()
    }

    // Assumes we've already checked all keys in query exist.
    fn resolve_actual_file(&self, file: &File) -> File {
        let all_extra_tags = self.get_tags_not_in_query(file);
        let file_name = &file.name;
        let mut extra_tags = BTreeSet::<String>::new();

        for t in &all_extra_tags {
            if self.files[t].contains(file_name) {
                extra_tags.insert(t.to_string());
            }
        }

        let all_file_tags: BTreeSet<String> = file.tags.union(&extra_tags).cloned().collect();

        File::new(file.name.clone(), all_file_tags)
    }

    fn do_all_keys_exist(&self, to_check: &TagSet) -> bool {
        let mut a = BTreeSet::new();
        let mut b = BTreeSet::new();
        for i in ["a", "b", "c"] {
            a.insert(i);
            b.insert(i);
        }
        self.get_key_set().is_superset(to_check)
    }

    fn file_name_to_file(&self, file_name: &str) -> File {
        let keys = self
            .files
            .iter()
            .filter(|(_, v)| v.contains(file_name))
            .map(|(k, _)| k);

        File::new(file_name.to_string(), BTreeSet::from_iter(keys.cloned()))
    }

    fn get_key_set(&self) -> BTreeSet<String> {
        self.files.keys().cloned().collect()
    }
}

impl FileDB for HashTags2DBFS {
    type FileIterator = IntoIter<File>;

    fn add_file(&mut self, new_file: &File) -> Option<()> {
        match self.get_file(new_file) {
            Ok(_) => None,
            Err(GetFileError::TooManyFiles) => None,
            Err(GetFileError::NoSuchFile) => {
                for t in new_file.tags() {
                    match self.files.get_mut(t) {
                        Some(map) => {
                            map.insert(new_file.name.clone());
                        }
                        None => {
                            let mut new_tag_files = BTreeSet::new();
                            new_tag_files.insert(new_file.name.clone());
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
        let mut file_names = self.files[iter.next().unwrap()].clone();

        // Iterate through the rest of the elements
        for t in iter {
            if !self.files.contains_key(t) {
                // If we found a single key that doesn't exist, fail
                return BTreeSet::new().into_iter();
            }

            // Union is lazy, and doesn't support chaining, so we need the
            // whole thing, so we have to clone and collect.
            file_names = file_names.union(&self.files[t]).cloned().collect();
        }

        let mut result = BTreeSet::<File>::new();

        for f in file_names {
            let tmp_file = File::new(f.clone(), query.tags().clone());
            result.insert(self.resolve_actual_file(&tmp_file));
        }
        result.into_iter()
    }

    fn get_file<F: FileQuery>(&self, query: &F) -> Result<File, GetFileError> {
        if !self.do_all_keys_exist(query.tags()) {
            return Err(GetFileError::NoSuchFile);
        }

        if let Some(name) = query.name() {
            // We only need to match the first tag, as if we can't
            // find the named file for ANY tag, then it's a failure.
            // We check for all tags in the query later.
            if let Some(k) = query.tags().iter().next() {
                if let Some(file_set) = self.files.get(k) {
                    if file_set.contains(name) {
                        let candidate = self.file_name_to_file(name);
                        // Because we only store the file name for each tag,
                        // we can only really have one file of each name on the
                        // whole system. We therefore know that if this one is
                        // a match, then it's the right one, and if it's not,
                        // then there's no match. This is a bug; we should
                        // support multiple files of the same name.
                        if query.could_match(&candidate) {
                            return Ok(candidate);
                        }
                    }
                }
            }
            // This is a fallthrough, if we don't get a match for any tag
            // (or if the given tags are empty, which we ignore by default...)
            return Err(GetFileError::NoSuchFile);
        } else {
            let mut possible_files = self.get_files(query);
            if possible_files.len() == 1 {
                // We know there's exactly 1, so next is safe to unwrap.
                return Ok(possible_files.next().unwrap());
            } else if possible_files.len() > 1 {
                return Err(GetFileError::TooManyFiles);
            } else {
                return Err(GetFileError::NoSuchFile);
            }
        }
    }
}
