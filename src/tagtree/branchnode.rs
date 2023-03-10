use super::endnode::EndNode;
use super::multiendnodeiterator::MultiNodeIterator;
use super::tagmaskbits::TagMaskBits;
use super::Node;
use crate::{fdb_trait::GetFileError, File, FileDB, FileQuery, TagSet};
use std::collections::hash_map::HashMap;

type TagMasks = HashMap<String, TagMaskBits>;

#[derive(Debug, Clone)]
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
            empty: TagMaskBits::ALL,
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
        let mut result = TagMaskBits::ALL;
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

    fn add_file_to_child(&mut self, idx: usize, file: &File, do_end_replace: bool) -> Option<()> {
        match self.nodes[idx].as_mut() {
            // Matches are in order; Empty -> End -> Branch as more things are added.
            Node::Empty => {
                // Mark bit as no longer empty
                self.replace_node(idx, Some(file.tags()));
                let res = self.nodes[idx].add_file(file);
                return res;
            }
            Node::End(node) => {
                // Only exact matches can be added
                if let Some(res) = node.add_file(file) {
                    return Some(res);
                } else {
                    // We only need to make a new branch if there isn't already a better option in
                    // self. Rely on the caller telling is if it's better to turn into a branch or
                    // not with 'do_end_replace'.

                    // Remaining will include *this* iteration. If we are the only one, then we
                    // need to split. Otherwise, don't. This will push the last match to always be
                    // the one that splits. That's probably not great for the number of iterations
                    // we do but it probably isn't that big an overhead.
                    if do_end_replace {
                        self.replace_node(idx, None);
                        // Adding to node that replaced self, this appears to cause recurssion to
                        // be infinite.
                        match self.nodes[idx].as_mut() {
                            Node::Branch(new_node) => {
                                // Add a node with the exact definition we expect, so that
                                // recursion ends when we add after this match.
                                assert!(new_node.empty.is_set(1));
                                new_node.replace_node(1, Some(file.tags()));
                                new_node.set_index_for_tags(1, file.tags().iter());
                            }
                            _ => {
                                panic!("Created a branch node, but then immediate found it to be something else.");
                            }
                        }
                        return self.nodes[idx].add_file(file);
                        // We know exactly what we did here, so we should just forcibly add a new
                        // "Node" where we want it.
                    } else {
                        // Fail to add to this one if we have other options.
                        return None;
                    }
                }
            }
            Node::Branch(node) => {
                // If we fail for a branch we must have run out of space. Splitting the node will
                // give us more space.
                if let Some(success) = node.add_file(file) {
                    return Some(success);
                } else {
                    self.replace_node(idx, None);
                    return None;
                }
            }
        }
    }

    fn make_branch_replacement_node(&self, target_nodes: TagMaskBits) -> Self {
        let mut replacement = BranchNode::new();

        for i in &mut target_nodes.clone() {
            // TODO: We shouldn't actually clone here, we should have two boxes pointing
            // at the same thing, then remove the old Box when we are ready. This will be
            // 'unsafe', so will need some testing.
            replacement.nodes[i] = Box::new(*(self.nodes[i]).clone());
        }

        for (k, v) in &self.masks {
            let value = v.intersect(&target_nodes);

            if value != TagMaskBits::CLEAR {
                replacement
                    .masks
                    .insert(k.to_string(), v.intersect(&target_nodes));
            }
        }

        replacement
    }

    // Effectively makes a clone of self, but with all nodes moved to two sub-nodes.
    // This is to replace 'self' with another node atomically, which must be done by
    // the parent of 'self'.
    pub(crate) fn make_half_split_node(&self) -> Self {
        let upper = self.make_branch_replacement_node(TagMaskBits::UPPER_HALF);
        let lower = self.make_branch_replacement_node(TagMaskBits::LOWER_HALF);

        let mut replacement = BranchNode::new();

        // Note: We should change the order of setting the masks and
        //       adding the nodes. The current order gets around move
        //       semantics.
        replacement.set_index_for_tags(0, lower.masks.keys());
        replacement.set_index_for_tags(1, upper.masks.keys());

        replacement.nodes[0] = Box::new(Node::Branch(lower));
        replacement.nodes[1] = Box::new(Node::Branch(upper));

        replacement
    }

    pub(crate) fn make_branch_from_end(end: &EndNode) -> Self {
        let mut replacement = BranchNode::new();
        // TODO: it would be good to note clone here...
        replacement.nodes[0] = Box::new(Node::End(end.clone()));
        let tags = end.all_tags();
        for t in tags {
            replacement.masks.insert(t, TagMaskBits::FIRST);
        }

        replacement
    }

    pub(crate) fn turn_empty_into_end(tags: &TagSet) -> EndNode {
        EndNode::new(tags.clone())
    }

    pub(crate) fn make_replacement_node(node: &Node, tags: Option<&TagSet>) -> Node {
        match node {
            // In order of progression as new files are added, Empty -> End -> Branch -> Branch
            Node::Empty => Node::End(BranchNode::turn_empty_into_end(tags.unwrap())),
            Node::End(node) => Node::Branch(BranchNode::make_branch_from_end(&node)),
            Node::Branch(node) => Node::Branch(node.make_half_split_node()),
        }
    }

    // This function must assume that all current content is live until at least the end of this
    // call (currently there is no checking to make sure this is safe even afterwards). There may
    // still be searchers access the data through the 'stale' pointers.
    fn replace_node(&mut self, to_replace: usize, tags: Option<&TagSet>) {
        let replacement = BranchNode::make_replacement_node(&*(self.nodes[to_replace]), tags);

        // This should be made atomic.
        self.nodes[to_replace] = Box::new(replacement);

        // Unsetting empty doesn't need to be atomic with above, as we know that there are
        // no searchers currently inside it; it's empty. This does nothing for other node
        // types, and thus is safe, unless two nodes try to write this at the same time, e.g.:
        // if two unset empties happen at the same time, they will race, and one will remain set
        // when it shouldn't. We should make this atomic to fix this race.

        // We only need to do this for an 'Empty' replacement, but just always do it because
        // it's fast and makes the logic simpler.
        self.empty.unset_bit(to_replace);
    }
}

impl FileDB for BranchNode {
    type FileIterator = MultiNodeIterator;

    fn add_file(&mut self, file: &File) -> Option<()> {
        // Best match is all tags match entry
        let mut all_match = self.get_intersect(file.tags.iter());
        let last = all_match.last_idx();
        for idx in &mut all_match {
            // No need to add tags in this case; we have a full match.
            // If we fail here, we should probably fork; this would be a good
            // place to split an End into a Branch.

            // It's safe to unwrap last because we *must* have a last if we're in the loop
            // as all.
            if let Some(res) = self.add_file_to_child(idx, file, idx==last.unwrap()) {
                return Some(res);
            }
        }

        // If we can't match perfectly, try anything that has a match at all.
        let mut any_match = self.get_union(file.tags.iter());
        let mut first_match: Option<usize> = None;
        let last = any_match.last_idx();
        for idx in &mut any_match {
            // Check that the add was successful before annotating all the tags.
            // This is more correct, and should work better for thread safety;
            // make sure all targets exist before they can be searched for.
            if let Some(res) = self.add_file_to_child(idx, file, idx==last.unwrap()) {
                self.set_index_for_tags(idx, file.tags().iter());
                return Some(res);
            }
            if first_match == None {
                first_match = Some(idx);
            }
        }

        // If we just didn't have space, make space in the first partial match.
        // Assume the first is the best match.
        if let Some(idx) = first_match {
            return self.add_file_to_child(idx, file, true);
        }

        let mut empty_items = self.empty.clone();
        let last = empty_items.last_idx();
        for idx in &mut empty_items {
            // Check that the add was successful before annotating all the tags.
            // This is more correct, and should work better for thread safety;
            // make sure all targets exist before they can be searched for.
            if let Some(res) = self.add_file_to_child(idx, file, idx==last.unwrap()) {
                self.set_index_for_tags(idx, file.tags().iter());
                return Some(res);
            }
        }

        // If we get here, then our parent will split this node and try again.
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
