use std::collections::btree_set::BTreeSet;

#[derive(PartialOrd, Ord, PartialEq, Eq, Debug, Hash, Clone)]
pub(crate) struct TagSet {
    pub items: BTreeSet<String>,
}

impl TagSet {
    pub fn from_iter<'a, I>(items: I) -> TagSet
    where
        I: IntoIterator<Item = &'a str>,
    {
        let b: BTreeSet<String> = items
            .into_iter()
            .map(|s| s.to_string())
            .filter(|x| x != "")
            .collect();
        TagSet { items: b }
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Debug, Clone)]
pub struct File {
    name: String,
    tags: TagSet,
}

impl File {
    pub(crate) fn new<'a, I>(name: &str, tags: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        File {
            name: name.to_string(),
            tags: TagSet::from_iter(tags),
        }
    }

    pub fn from_str(path: &str) -> Option<File> {
        let mut split = path.rsplit("/");
        Some(File::new(split.next()?, split))
    }

    pub(crate) fn has_tags(&self, tags: &TagSet) -> bool {
        self.tags.items.is_superset(&tags.items)
    }
}
