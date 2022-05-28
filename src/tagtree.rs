use crate::{fdb_trait::GetFileError, File, FileDB, FileQuery, TagSet};
use std::collections::btree_set::{BTreeSet, Iter};
use std::collections::hash_map::HashMap;
use std::vec::Vec;

#[derive(Debug)]
enum Node {
    BranchNode,
    EndNode,
    EmptyNode,
}

#[derive(Debug)]
struct BranchNode {
    masks: HashMap<String, u32>,
    nodes: [Node; 32],
}

#[derive(Debug)]
struct EndNode {
    file_names: BTreeSet<String>,
    tags: TagSet,
}

#[derive(Clone, Copy, Debug)]
struct EmptyNode {}

struct EndNodeIterator {
    file_names: Vec<String>,
    tags: TagSet,
    current: usize,
}

impl EndNodeIterator {
    fn empty() -> EndNodeIterator {
        EndNodeIterator {
            file_names: vec![],
            tags: TagSet::new(),
            current: 0,
        }
    }

    fn new(file_names: Iter<'_, String>, tags: &TagSet) -> Self {
        Self {
            file_names: file_names.cloned().collect(),
            tags: tags.iter().cloned().collect(),
            current: 0,
        }
    }
}

impl Iterator for EndNodeIterator {
    type Item = File;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(file_name) = self.file_names.get(self.current) {
            self.current += 1;
            return Some(File::new(file_name.to_string(), self.tags.clone()));
        } else {
            return None;
        }
    }
}

impl ExactSizeIterator for EndNodeIterator {
    fn len(&self) -> usize {
        self.file_names.len()
    }
}

pub struct MultiEndNodeIterator {
    backing: Vec<EndNodeIterator>,
    current: usize,
}

impl MultiEndNodeIterator {
    fn empty() -> Self {
        Self {
            backing: vec![],
            current: 0,
        }
    }
}

impl Iterator for MultiEndNodeIterator {
    type Item = File;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(iter) = self.backing.get_mut(self.current) {
                if let Some(file) = iter.next() {
                    return Some(file);
                }
            } else {
                if self.current < self.backing.len() {
                    self.current += 1;
                } else {
                    return None;
                }
            }
        }
    }
}

impl ExactSizeIterator for MultiEndNodeIterator {
    fn len(&self) -> usize {
        return self.backing.iter().map(|x| x.len()).sum();
    }
}

impl BranchNode {
    fn new() -> BranchNode {
        BranchNode {
            masks: HashMap::<String, u32>::new(),
            nodes: (0..32)
                .map(|_| Node::EmptyNode)
                .collect::<Vec<Node>>()
                .try_into()
                .unwrap(),
        }
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
        if *query.tags() == self.tags {
            EndNodeIterator::new(self.file_names.iter(), &self.tags)
        } else {
            EndNodeIterator::empty()
        }
    }

    fn get_file<F: FileQuery>(&self, _query: &F) -> Result<File, GetFileError> {
        Err(GetFileError::NoSuchFile)
    }
}

impl FileDB for BranchNode {
    type FileIterator = MultiEndNodeIterator;

    fn add_file(&mut self, _new_files: &File) -> Option<()> {
        None
    }

    fn get_files<F: FileQuery>(&self, _query: &F) -> Self::FileIterator {
        MultiEndNodeIterator::empty()
    }

    fn get_file<F: FileQuery>(&self, _query: &F) -> Result<File, GetFileError> {
        Err(GetFileError::NoSuchFile)
    }
}

impl FileDB for EmptyNode {
    type FileIterator = EndNodeIterator;

    fn add_file(&mut self, _new_files: &File) -> Option<()> {
        None
    }

    fn get_files<F: FileQuery>(&self, _query: &F) -> Self::FileIterator {
        EndNodeIterator::empty()
    }

    fn get_file<F: FileQuery>(&self, _query: &F) -> Result<File, GetFileError> {
        Err(GetFileError::NoSuchFile)
    }
}

pub struct TagTreeDBFS {
    root: BranchNode,
}

impl TagTreeDBFS {
    pub fn new() -> TagTreeDBFS {
        TagTreeDBFS {
            root: BranchNode::new(),
        }
    }
}

impl FileDB for TagTreeDBFS {
    type FileIterator = MultiEndNodeIterator;

    fn add_file(&mut self, _new_file: &File) -> Option<()> {
        None
    }

    fn get_files<F: FileQuery>(&self, _query: &F) -> Self::FileIterator {
        MultiEndNodeIterator::empty()
    }

    fn get_file<F: FileQuery>(&self, _query: &F) -> Result<File, GetFileError> {
        Err(GetFileError::NoSuchFile)
    }
}
