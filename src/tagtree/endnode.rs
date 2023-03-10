use super::endnodeiterator::EndNodeIterator;
use crate::{fdb_trait::GetFileError, File, FileDB, FileQuery, TagSet};
use std::collections::btree_set::BTreeSet;

#[derive(Debug, Clone)]
pub struct EndNode {
    file_names: BTreeSet<String>,
    tags: TagSet,
}

impl EndNode {
    pub fn new(tags: TagSet) -> Self {
        Self {
            file_names: BTreeSet::new(),
            tags,
        }
    }

    pub(crate) fn all_tags(&self) -> TagSet {
        self.tags.clone()
    }
}

impl FileDB for EndNode {
    type FileIterator = EndNodeIterator;

    fn add_file(&mut self, new_file: &File) -> Option<()> {
        if new_file.tags == self.tags {
            self.file_names.insert(new_file.name.clone());
            Some(())
        } else {
            None
        }
    }

    fn get_files<F: FileQuery>(&self, query: &F) -> Self::FileIterator {
        if self.tags.is_superset(query.tags()) {
            if let Some(file_name) = query.name() {
                if self.file_names.contains(file_name) {
                    EndNodeIterator::new([file_name.to_string()].iter(), &self.tags)
                } else {
                    EndNodeIterator::empty()
                }
            } else {
                EndNodeIterator::new(self.file_names.iter(), &self.tags)
            }
        } else {
            EndNodeIterator::empty()
        }
    }

    fn get_file<F: FileQuery>(&self, query: &F) -> Result<File, GetFileError> {
        if !self.tags.is_superset(query.tags()) {
            return Err(GetFileError::NoSuchFile);
        }

        if let Some(name) = query.name() {
            if self.file_names.contains(name) {
                return Ok(File::new(name.to_string(), self.tags.clone()));
            } else {
                return Err(GetFileError::NoSuchFile);
            }
        } else {
            if self.file_names.len() == 1 {
                let only_file = self.file_names.iter().next().unwrap();
                return Ok(File::new(only_file.to_string(), self.tags.clone()));
            } else {
                return Err(GetFileError::TooManyFiles);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::{EndNode, FileDB};
    use crate::{fromstr::FromStr, File, TagSet};
    use std::collections::hash_set::HashSet;

    #[test]
    fn endnode_should_find_files() {
        let tags = TagSet::from_str("/one/two/three").unwrap();

        let mut db = EndNode::new(tags.clone());

        let files: HashSet<File> = [
            File::new_cloned("file.txt", tags.clone()),
            File::new_cloned("two.txt", tags.clone()),
            File::new_cloned("four.txt", tags.clone()),
        ]
        .iter()
        .cloned()
        .collect();

        files.iter().for_each(|x| db.add_file(x).unwrap());

        let query = TagSet::from_str("/one/two/three").unwrap();

        let actual: HashSet<File> = db.get_files(&query).collect();

        assert_eq!(files, actual);
    }

    #[test]
    fn endnode_should_find_single_file() {
        let tags = TagSet::from_str("/one/two/three").unwrap();

        let mut db = EndNode::new(tags.clone());

        let files: HashSet<File> = [
            File::new_cloned("file.txt", tags.clone()),
            File::new_cloned("two.txt", tags.clone()),
            File::new_cloned("four.txt", tags.clone()),
        ]
        .iter()
        .cloned()
        .collect();

        files.iter().for_each(|x| db.add_file(x).unwrap());

        let query = File::from_str("/one/two/three/file.txt").unwrap();

        let mut expected: HashSet<File> = HashSet::new();
        expected.insert(query.clone());

        let actual: HashSet<File> = db.get_files(&query).collect();

        assert_eq!(expected, actual);
    }

    #[test]
    fn endnode_should_not_find_unmatched_file_with_matching_tags() {
        let tags = TagSet::from_str("/one/two/three").unwrap();

        let mut db = EndNode::new(tags.clone());

        let files: HashSet<File> = [
            File::new_cloned("file.txt", tags.clone()),
            File::new_cloned("two.txt", tags.clone()),
            File::new_cloned("four.txt", tags.clone()),
        ]
        .iter()
        .cloned()
        .collect();

        files.iter().for_each(|x| db.add_file(x).unwrap());

        let query = File::from_str("/one/two/three/blue.txt").unwrap();

        let expected: HashSet<File> = HashSet::new();

        let actual: HashSet<File> = db.get_files(&query).collect();

        assert_eq!(expected, actual);
    }

    #[test]
    fn endnode_should_find_partial_match_files() {
        let tags = TagSet::from_str("/one/two/three").unwrap();

        let mut db = EndNode::new(tags.clone());

        let files: HashSet<File> = [
            File::new_cloned("file.txt", tags.clone()),
            File::new_cloned("two.txt", tags.clone()),
            File::new_cloned("four.txt", tags.clone()),
        ]
        .iter()
        .cloned()
        .collect();

        files.iter().for_each(|x| db.add_file(x).unwrap());

        let query = TagSet::from_str("/one/two").unwrap();

        let actual: HashSet<File> = db.get_files(&query).collect();

        assert_eq!(files, actual);
    }

    #[test]
    fn endnode_should_not_find_unmatched_files() {
        let tags = TagSet::from_str("/one/two/three").unwrap();

        let mut db = EndNode::new(tags.clone());

        let files: HashSet<File> = [
            File::new_cloned("file.txt", tags.clone()),
            File::new_cloned("two.txt", tags.clone()),
            File::new_cloned("four.txt", tags.clone()),
        ]
        .iter()
        .cloned()
        .collect();

        files.iter().for_each(|x| db.add_file(x).unwrap());

        let query = TagSet::from_str("/one/two/four").unwrap();

        let actual: HashSet<File> = db.get_files(&query).collect();

        let expected: HashSet<File> = HashSet::new();

        assert_eq!(expected, actual);
    }
}
