mod helpers;

use crate::helpers::hprops::{arb_file, arb_simple_file};
//use crate::helpers::hprops::arb_file;
use proptest::collection;
use proptest::proptest;

use rdb_fs::{FileDB, TagTreeDBFS};

proptest! {
    #[test]
    fn test_add_file(file in arb_file()) {
        let mut db = TagTreeDBFS::new();

        let added = db.add_file(&file);

        assert!(added.is_some());

        assert_eq!(file, db.get_file(&file).unwrap());
    }
}

proptest! {
    #[test]
    fn test_add_files(file_set in collection::btree_set(arb_simple_file(), 0..100)) {
        let mut db = TagTreeDBFS::new();


        for f in &file_set {
            let added = db.add_file(f);
            assert!(added.is_some());
        }

        for f in &file_set {
            assert_eq!(f, &db.get_file(f).unwrap());
        }
    }
}

use rdb_fs::File;
use std::collections::btree_set::BTreeSet;

#[test]
fn test_add_files_fail() {
    let mut db = TagTreeDBFS::new();
    let file_set: BTreeSet<File> = [
        File::new_cloned("-", ["A"]),
        File::new_cloned("-", ["B"]),
        File::new_cloned("-", ["C"]),
        File::new_cloned("-", ["D"]),
        File::new_cloned("-", ["E"]),
        File::new_cloned("-", ["F"]),
        File::new_cloned("-", ["G"]),
        File::new_cloned("-", ["H"]),
        File::new_cloned("-", ["I"]),
        File::new_cloned("-", ["J"]),
        File::new_cloned("-", ["K"]),
        File::new_cloned("-", ["L"]),
        File::new_cloned("-", ["M"]),
        File::new_cloned("-", ["N"]),
        File::new_cloned("-", ["O"]),
        File::new_cloned("-", ["P"]),
        File::new_cloned("-", ["Q"]),
        File::new_cloned("-", ["R"]),
        File::new_cloned("-", ["S"]),
        File::new_cloned("-", ["T"]),
        File::new_cloned("-", ["U"]),
        File::new_cloned("-", ["V"]),
        File::new_cloned("-", ["W"]),
        File::new_cloned("-", ["X"]),
        File::new_cloned("-", ["Y"]),
        File::new_cloned("-", ["Z"]),
        File::new_cloned("-", ["a"]),
        File::new_cloned("-", ["b"]),
        File::new_cloned("-", ["c"]),
        File::new_cloned("-", ["d"]),
        File::new_cloned("-", ["e"]),
        File::new_cloned("-", ["f"]),
        File::new_cloned("-", ["g"]),

        //File::new_cloned("-", ["-"]),
        //File::new_cloned("-", ["A"]),
        //File::new_cloned("-", ["_"]),
        //File::new_cloned("-", ["a"]),
        //File::new_cloned("0", ["-"]),
        //File::new_cloned("0", ["0"]),
        //File::new_cloned("0", ["a"]),
        //File::new_cloned("A", ["-"]),
        //File::new_cloned("f", ["0", "1"]),
        //File::new_cloned("A", ["2"]),
        //File::new_cloned("A", ["A"]),
        //File::new_cloned("A", ["b"]),
        //File::new_cloned("_", ["-"]),
        //File::new_cloned("_", ["."]),
        //File::new_cloned("_", ["0"]),
        //File::new_cloned("_", ["A"]),
        //File::new_cloned("l", ["1"]),
        //File::new_cloned("a", ["a"]),
    ]
    .iter()
    .cloned()
    .collect();

    for f in &file_set {
        let added = db.add_file(f);
        assert!(added.is_some());
    }

    for f in &file_set {
        assert_eq!(f, &db.get_file(f).unwrap());
    }
}

#[test]
fn tagtree_add_file_with_single_tag_and_multiple_tags() {
    let mut db = TagTreeDBFS::new();
    let file_set: BTreeSet<File> = [
        File::new_cloned("f", ["0", "1"]),
        File::new_cloned("l", ["1"]),
    ]
    .iter()
    .cloned()
    .collect();

    for f in &file_set {
        let added = db.add_file(f);
        assert!(added.is_some());
    }

    for f in &file_set {
        assert_eq!(f, &db.get_file(f).unwrap());
    }
}
