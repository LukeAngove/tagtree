use super::endnode::EndNode;
use super::multiendnodeiterator::MultiNodeIterator;
use super::tagmaskbits::TagMaskBits;
use super::Node;
use crate::{fdb_trait::GetFileError, File, FileDB, FileQuery};
use std::collections::hash_map::HashMap;

type TagMasks = HashMap<String, TagMaskBits>;

#[derive(Debug)]
pub struct BranchNode {
    masks: TagMasks,
    nodes: [Box<Node>; TagMaskBits::BITS],
    empty: TagMaskBits,
}

impl BranchNode {
    pub(crate) fn new() -> Self {
        Self {
            masks: TagMasks::new(),
            nodes: (0..TagMaskBits::BITS)
                .map(|_| Box::new(Node::Empty))
                .collect::<Vec<Box<Node>>>()
                .try_into()
                .unwrap(),
            empty: TagMaskBits::MAX,
        }
    }

    fn get_union<'a, I: Iterator<Item = &'a String>>(&self, tags: I) -> TagMaskBits {
        let mut result = TagMaskBits::CLEAR;
        for t in tags {
            if let Some(mask) = self.masks.get(t) {
                result.set_union(mask);
            } else {
                // If we can't find one tag, then we can't have a match
                return TagMaskBits::CLEAR;
            }
        }
        result
    }

    fn get_intersect<'a, I: Iterator<Item = &'a String>>(&self, tags: I) -> TagMaskBits {
        let mut result = TagMaskBits::MAX;
        for t in tags {
            if let Some(mask) = self.masks.get(t) {
                result.set_intersect(mask);
            } else {
                // If we can't find one tag, then we can't have a match
                return TagMaskBits::CLEAR;
            }
        }
        result
    }

    fn set_index_for_tags<'a, I: Iterator<Item = &'a String>>(&mut self, idx: usize, tags: I) {
        for t in tags {
            let current = self
                    .masks
                    .entry(t.to_string())
                    .or_insert(TagMaskBits::CLEAR);
                current.set_bit(idx);
        }
     
    }

    fn add_file_to_child(&mut self, idx: usize, file: &File) -> Option<()> {
        match self.nodes[idx].as_mut() {
            Node::End(node) => {
                return node.add_file(file);
            }
            Node::Branch(node) => {
                return node.add_file(file);
            }
            Node::Empty => {
                // Mark bit as no longer empty
                self.empty.unset_bit(idx);
                let new_tags = file.tags().clone();
                let mut new_node = EndNode::new(new_tags);
                let res = new_node.add_file(file);
                if res.is_some() {
                    self.nodes[idx] = Box::new(Node::End(new_node));
                }
                return res;
            }
        }
    }
}

impl FileDB for BranchNode {
    type FileIterator = MultiNodeIterator;

    fn add_file(&mut self, file: &File) -> Option<()> {
        // Best match is all tags match entry
        let mut all_match = self.get_intersect(file.tags.iter());
        for idx in &mut all_match {
            // No need to add tags in this case; we have a full match.
            // If we fail here, we should probably fork; this would be a good
            // place to split an End into a Branch.
            if let Some(res) = self.add_file_to_child(idx, file) {
                return Some(res);
            }
        }

        // If we can't match perfectly, try anything that has a match at all.
        let mut any_match = self.get_union(file.tags.iter());
        for idx in &mut any_match {
            // Check that the add was successful before annotating all the tags.
            // This is more correct, and should work better for thread safety;
            // make sure all targets exist before they can be searched for.
            if let Some(res) = self.add_file_to_child(idx, file) {
                self.set_index_for_tags(idx, file.tags().iter());
                return Some(res);
            }
        }
 
        let mut empty_items = self.empty.clone();
        for idx in &mut empty_items {
            // Check that the add was successful before annotating all the tags.
            // This is more correct, and should work better for thread safety;
            // make sure all targets exist before they can be searched for.
            if let Some(res) = self.add_file_to_child(idx, file) {
                self.set_index_for_tags(idx, file.tags().iter());
                return Some(res);
            }
        }
        None       
    }

    fn get_files<F: FileQuery>(&self, query: &F) -> Self::FileIterator {
        // Use intersect, as the target MUST have every tag, not just a subset
        let mut mask = self.get_intersect(query.tags().iter());
        MultiNodeIterator::new(mask.map(|x| self.nodes[x].get_files(query)))
    }

    fn get_file<F: FileQuery>(&self, query: &F) -> Result<File, GetFileError> {
        // Use intersect, as the target MUST have every tag, not just a subset
        let mut mask = self.get_intersect(query.tags().iter());

        let mut result: Result<File, GetFileError> = Err(GetFileError::NoSuchFile);

        for node_idx in &mut mask {
            match self.nodes[node_idx].get_file(query) {
                // Too many files in one branch, so that's the result
                Err(GetFileError::TooManyFiles) => {
                    return Err(GetFileError::TooManyFiles);
                }
                // Ignore nothing found; we might find it elsewhere
                Err(GetFileError::NoSuchFile) => {}
                Ok(e) => {
                    // We found a single result, fail if we did this more than once
                    if result.is_ok() {
                        return Err(GetFileError::TooManyFiles);
                    } else {
                        result = Ok(e);
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::{BranchNode, FileDB};
    use crate::{fromstr::FromStr, File, TagSet};
    use std::collections::hash_set::HashSet;

    #[test]
    fn branchnode_should_allow_differently_tagged_files() {
        let tags1 = TagSet::from_str("/one/two/three").unwrap();
        let tags2 = TagSet::from_str("/one/two/four").unwrap();

        let mut db = BranchNode::new();

        let files: HashSet<File> = [
            File::new_cloned("file.txt", tags1.clone()),
            File::new_cloned("four.txt", tags2.clone()),
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
    fn branchnode_should_find_files() {
        let tags = TagSet::from_str("/one/two/three").unwrap();

        let mut db = BranchNode::new();

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
    fn branchnode_should_find_single_file() {
        let tags = TagSet::from_str("/one/two/three").unwrap();

        let mut db = BranchNode::new();

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
    fn branchnode_should_not_find_unmatched_file_with_matching_tags() {
        let tags = TagSet::from_str("/one/two/three").unwrap();

        let mut db = BranchNode::new();

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
    fn branchnode_should_find_partial_match_files() {
        let tags = TagSet::from_str("/one/two/three").unwrap();

        let mut db = BranchNode::new();

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
    fn branchnode_should_not_find_unmatched_files() {
        let tags = TagSet::from_str("/one/two/three").unwrap();

        let mut db = BranchNode::new();

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
