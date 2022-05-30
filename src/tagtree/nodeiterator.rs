use super::endnodeiterator::EndNodeIterator;
use super::multiendnodeiterator::MultiNodeIterator;
use crate::File;

#[derive(Debug)]
pub enum NodeIterator {
    EndNodeIter(EndNodeIterator),
    MultiNodeIter(MultiNodeIterator),
}

impl Iterator for NodeIterator {
    type Item = File;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::EndNodeIter(node) => node.next(),
            Self::MultiNodeIter(node) => node.next(),
        }
    }
}

impl ExactSizeIterator for NodeIterator {
    fn len(&self) -> usize {
        match self {
            Self::EndNodeIter(node) => node.len(),
            Self::MultiNodeIter(node) => node.len(),
        }
    }
}
