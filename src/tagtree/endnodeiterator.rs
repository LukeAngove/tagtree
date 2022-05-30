use crate::{File, TagSet};

#[derive(Debug)]
pub struct EndNodeIterator {
    file_names: Vec<String>,
    tags: TagSet,
    current: usize,
}

impl EndNodeIterator {
    pub(crate) fn empty() -> EndNodeIterator {
        EndNodeIterator {
            file_names: vec![],
            tags: TagSet::new(),
            current: 0,
        }
    }

    pub(crate) fn new<'a, I: Iterator<Item = &'a String>>(file_names: I, tags: &TagSet) -> Self {
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

#[cfg(test)]
mod tests {

    use super::EndNodeIterator;
    use crate::{fromstr::FromStr, File, TagSet};

    #[test]
    fn empty_should_yeild_nothing() {
        let actual = EndNodeIterator::empty();
        let expected = Vec::<File>::new();
        assert_eq!(expected, actual.collect::<Vec<File>>());
    }

    #[test]
    fn non_empty_should_yeild_files() {
        let tags = TagSet::from_str("/one/two").unwrap();
        let actual =
            EndNodeIterator::new(vec!["blue".to_string(), "red".to_string()].iter(), &tags);
        let expected = vec![
            File::new("blue".to_string(), tags.clone()),
            File::new("red".to_string(), tags.clone()),
        ];
        assert_eq!(expected, actual.collect::<Vec<File>>());
    }
}
