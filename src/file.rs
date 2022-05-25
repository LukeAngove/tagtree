use crate::fromstr::FromStr;
use crate::FileQuery;
use crate::TagSet;

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Debug, Clone)]
pub struct File {
    pub(crate) name: String,
    pub(crate) tags: TagSet,
}

impl File {
    pub fn new(name: String, tags: TagSet) -> Self {
        File { name, tags }
    }

    pub fn new_cloned<I, S>(name: &str, tags: I) -> Self
    where
        S: ToString,
        I: IntoIterator<Item = S>,
    {
        File {
            name: name.to_string(),
            tags: tags.into_iter().map(|x| x.to_string()).collect(),
        }
    }

    pub(crate) fn has_tags(&self, tags: &TagSet) -> bool {
        self.tags.is_superset(&tags)
    }
}

impl FromStr for File {
    fn from_str(path: &str) -> Option<File> {
        let mut split = path.rsplit("/");
        // Filter out empty strings to remove "" before the first '/'
        Some(File::new(
            split.next()?.to_string(),
            split.filter(|x| x != &"").map(|x| x.to_string()).collect(),
        ))
    }
}

impl FileQuery for File {
    fn could_match(&self, to_match: &File) -> bool {
        println!("Checking file: {:?}", to_match);
        println!("could_match: {:?}", self.tags.could_match(to_match));
        println!("name: {:?}", self.name == to_match.name);
        self.tags.could_match(to_match) && self.name == to_match.name
    }

    fn tags(&self) -> &TagSet {
        self.tags.tags()
    }

    fn name(&self) -> Option<&str> {
        Some(self.name.as_ref())
    }
}

#[cfg(test)]
mod tests {

    use crate::file::File;
    use crate::file::TagSet;
    use crate::fromstr::FromStr;
    use std::collections::btree_set::BTreeSet;

    #[test]
    fn should_convert_str_list_to_set() {
        let input = ["etc", "fine", "shoes", "make.txt"];

        let expected = BTreeSet::from_iter([
            "etc".to_string(),
            "fine".to_string(),
            "shoes".to_string(),
            "make.txt".to_string(),
        ]);

        let actual = TagSet::from_iter(input.into_iter().map(|x| x.to_string()));

        assert_eq!(expected, actual);
    }

    #[test]
    fn should_make_file_from_str() {
        let input = "/etc/fine/shoes/make.txt";

        let expected = File::new_cloned("make.txt", ["etc", "fine", "shoes"]);

        let actual = File::from_str(input);

        assert_eq!(Some(expected), actual);
    }
}
