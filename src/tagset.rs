use crate::fromstr::FromStr;
use crate::File;
use crate::FileQuery;
use std::collections::btree_set::BTreeSet;

pub type TagSet = BTreeSet<String>;

impl FromStr for TagSet {
    fn from_str(path: &str) -> Option<TagSet> {
        let split = path.rsplit("/");
        // Filter out empty strings to remove "" before the first '/'
        Some(split.filter(|x| x != &"").map(|x| x.to_string()).collect())
    }
}

impl<'a> FileQuery for TagSet {
    fn could_match(&self, to_match: &File) -> bool {
        println!("Checking file (tags): {:?}", to_match);
        println!("Against: {:?}", self);
        to_match.has_tags(self)
    }

    fn tags(&self) -> &Self {
        self
    }

    fn name(&self) -> Option<&str> {
        None
    }
}

#[test]
fn found_files_should_have_all_tags_in_query() {
    let f = File::from_str("/etc/fine/red.txt").unwrap();

    let query = TagSet::from_str("/fine/etc").unwrap();

    assert!(query.could_match(&f));
}
