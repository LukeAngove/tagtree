use proptest::collection;
use proptest::prelude::*;
use proptest::proptest;

use rdb_fs::fromstr::FromStr;
use rdb_fs::TagSet;

prop_compose! {
    pub fn arb_tagset()(ref tags in collection::btree_set("[^/\n]+", 1..100) ) -> TagSet {
        tags.clone()
    }
}

proptest! {
    #[test]
    // We use a set here because we don't support repeated tags, by intention.
    // This is different to traditional filesystems.
    fn test_from_str(ref first in "/{0,1}", ref raw_tags in collection::btree_set("[^/\n]+", 1..100)) {
        let a = first.clone() + &raw_tags.iter().cloned().collect::<Vec<String>>().join("/");
        let dut = TagSet::from_str(&a);

        assert!(dut.is_some());

        let d = dut.unwrap();

        let ntags = a.split('/').filter(|x| *x != "").count();

        assert_eq!(ntags, d.len());
    }
}
