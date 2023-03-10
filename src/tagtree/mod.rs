pub(crate) mod branchnode;
pub(crate) mod endnode;
pub(crate) mod endnodeiterator;
pub(crate) mod multiendnodeiterator;
pub(crate) mod nodeiterator;
mod tagmaskbits;

use crate::{fdb_trait::GetFileError, File, FileDB, FileQuery};
use branchnode::BranchNode;
use endnode::EndNode;
use multiendnodeiterator::MultiNodeIterator;
use nodeiterator::NodeIterator;

#[derive(Debug, Clone)]
pub enum Node {
    Branch(BranchNode),
    End(EndNode),
    Empty,
}

impl FileDB for Node {
    type FileIterator = NodeIterator;

    fn add_file(&mut self, new_file: &File) -> Option<()> {
        match self {
            Self::Branch(node) => node.add_file(new_file),
            Self::End(node) => node.add_file(new_file),
            Self::Empty => None,
        }
    }

    fn get_files<F: FileQuery>(&self, query: &F) -> Self::FileIterator {
        match self {
            Self::Branch(node) => NodeIterator::MultiNodeIter(node.get_files(query)),
            Self::End(node) => NodeIterator::EndNodeIter(node.get_files(query)),
            Self::Empty => NodeIterator::MultiNodeIter(MultiNodeIterator::empty()),
        }
    }

    fn get_file<F: FileQuery>(&self, query: &F) -> Result<File, GetFileError> {
        match self {
            Self::Branch(node) => node.get_file(query),
            Self::End(node) => node.get_file(query),
            Self::Empty => Err(GetFileError::NoSuchFile),
        }
    }
}

#[derive(Debug)]
pub struct TagTreeDBFS {
    root: Node,
}

impl TagTreeDBFS {
    pub fn new() -> TagTreeDBFS {
        TagTreeDBFS {
            root: Node::Branch(BranchNode::new()),
        }
    }

    fn replace_node(&mut self) {
        // It is safe to pass None here, because we know that the our node is
        // always a Branch (in fact, we should replace this just with split).
        let replacement = BranchNode::make_replacement_node(&self.root, None);

        // This should be made atomic.
        self.root = replacement;
    }
}

impl FileDB for TagTreeDBFS {
    type FileIterator = NodeIterator;

    fn add_file(&mut self, new_file: &File) -> Option<()> {
        if let Some(res) = self.root.add_file(new_file) {
            return Some(res);
        } else {
            self.replace_node();
            return self.root.add_file(new_file);
        }
    }

    fn get_files<F: FileQuery>(&self, query: &F) -> Self::FileIterator {
        self.root.get_files(query)
    }

    fn get_file<F: FileQuery>(&self, query: &F) -> Result<File, GetFileError> {
        self.root.get_file(query)
    }
}
