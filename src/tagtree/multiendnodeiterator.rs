use super::nodeiterator::NodeIterator;
use crate::File;

#[derive(Debug)]
pub struct MultiNodeIterator {
    backing: Vec<NodeIterator>,
    current: usize,
}

impl MultiNodeIterator {
    pub(crate) fn new<I: Iterator<Item=NodeIterator>>(iter: I) -> Self {
        Self {
            backing: iter.collect(),
            current: 0,
        }
    }

    pub(crate) fn empty() -> Self {
        Self {
            backing: vec![],
            current: 0,
        }
    }
}

impl Iterator for MultiNodeIterator {
    type Item = File;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(iter) = self.backing.get_mut(self.current) {
                if let Some(file) = iter.next() {
                    return Some(file);
                } else {
                    if self.current < self.backing.len() {
                        self.current += 1;
                    } else {
                        return None;
                    }
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

impl ExactSizeIterator for MultiNodeIterator {
    fn len(&self) -> usize {
        return self.backing.iter().map(|x| x.len()).sum();
    }
}

#[cfg(test)]
mod tests {

    use super::MultiNodeIterator;
    use crate::tagtree::nodeiterator::NodeIterator;
    use crate::tagtree::endnodeiterator::EndNodeIterator;
    use crate::{fromstr::FromStr, File, TagSet};

    #[test]
    fn empty_should_yield_nothing() {
        let actual = MultiNodeIterator::empty();
        let expected = Vec::<File>::new();
        assert_eq!(expected, actual.collect::<Vec<File>>());
    }

    #[test]
    fn non_empty_should_yield_files() {
        let tags = TagSet::from_str("/one/two").unwrap();
        let internal =
            EndNodeIterator::new(vec!["blue".to_string(), "red".to_string()].iter(), &tags);
        let actual =
            MultiNodeIterator::new(vec![NodeIterator::EndNodeIter(internal)].into_iter());
 
        let expected = vec![
            File::new("blue".to_string(), tags.clone()),
            File::new("red".to_string(), tags.clone()),
        ];
        assert_eq!(expected, actual.collect::<Vec<File>>());
    }

    #[test]
    fn mutliple_sources_should_yield_files() {
        let tags1 = TagSet::from_str("/one/two").unwrap();
        let tags2 = TagSet::from_str("/one/three").unwrap();
        let internal1 =
            EndNodeIterator::new(vec!["blue".to_string(), "red".to_string()].iter(), &tags1);
        let internal2 =
            EndNodeIterator::new(vec!["blue".to_string(), "red".to_string()].iter(), &tags2);
 
        let actual =
            MultiNodeIterator::new(vec![NodeIterator::EndNodeIter(internal1), NodeIterator::EndNodeIter(internal2)].into_iter());
 
        let expected = vec![
            File::new("blue".to_string(), tags1.clone()),
            File::new("red".to_string(), tags1.clone()),
            File::new("blue".to_string(), tags2.clone()),
            File::new("red".to_string(), tags2.clone()),
        ];

        assert_eq!(expected, actual.collect::<Vec<File>>());
    }
}
