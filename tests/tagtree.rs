mod helpers;

use crate::helpers::{add_files_to_db, file_list_from_iter_str};
use rdb_fs::fromstr::FromStr;
use rdb_fs::GetFileError;
use rdb_fs::TagSet;
use rdb_fs::TagTreeDBFS;
use rdb_fs::{File, FileDB, FileQuery};
use std::collections::hash_set::HashSet;

#[test]
fn tagtree_should_find_files() {
    let mut db = TagTreeDBFS::new();

    let files = file_list_from_iter_str([
        "/etc/fine/shoes/make.txt",
        "/etc/fine/shoes/blue.png",
        "/mnt/partition/fourteen.one",
    ]);

    add_files_to_db(&mut db, files);

    let query = TagSet::from_str("/etc/fine").unwrap();

    let expected_files: HashSet<File> =
        file_list_from_iter_str(["/etc/fine/shoes/make.txt", "/etc/fine/shoes/blue.png"]);

    let actual: HashSet<File> = db.get_files(&query).collect();

    assert_eq!(expected_files, actual);
}

#[test]
fn tagtree_should_find_files_with_different_query_order() {
    let mut db = TagTreeDBFS::new();
    let files = file_list_from_iter_str([
        "/etc/fine/shoes/make.txt",
        "/etc/fine/shoes/blue.png",
        "/mnt/partition/fourteen.one",
    ]);

    add_files_to_db(&mut db, files);

    let expected_files = HashSet::from_iter([
        File::new_cloned("make.txt", ["etc", "fine", "shoes"]),
        File::new_cloned("blue.png", ["etc", "fine", "shoes"]),
    ]);

    let query = TagSet::from_str("/fine/etc").unwrap();

    let actual: HashSet<File> = db.get_files(&query).collect();

    assert_eq!(expected_files, actual);
}

#[test]
fn tagtree_should_get_empty_list_for_file_that_doesnt_exist() {
    let mut db = TagTreeDBFS::new();
    let files = file_list_from_iter_str([
        "/etc/fine/shoes/make.txt",
        "/etc/fine/shoes/blue.png",
        "/mnt/partition/fourteen.one",
    ]);

    add_files_to_db(&mut db, files);

    let query = TagSet::from_str("/fine/etc/luke.txt").unwrap();

    let actual: HashSet<File> = db.get_files(&query).collect();

    let expected: HashSet<File> = HashSet::new();

    assert_eq!(expected, actual);
}

#[test]
fn tagtree_should_get_file_for_file_that_exists() {
    let mut db = TagTreeDBFS::new();
    let files = file_list_from_iter_str(["/etc/fine/shoes/make.txt"]);

    add_files_to_db(&mut db, files);

    let query = File::from_str("/etc/fine/shoes/make.txt").unwrap();

    let actual = db.get_file(&query);

    let expected = Ok(File::from_str("/etc/fine/shoes/make.txt").unwrap());

    assert_eq!(expected, actual);
}

#[test]
fn tagtree_should_get_file_with_more_tags_for_file_that_exists() {
    let mut db = TagTreeDBFS::new();
    let files = file_list_from_iter_str(["/etc/fine/shoes/make.txt"]);

    add_files_to_db(&mut db, files);

    let query = File::from_str("/etc/fine/make.txt").unwrap();

    let actual = db.get_file(&query);

    // The result should be the file from the DB, not the same as the query
    let expected = Ok(File::from_str("/etc/fine/shoes/make.txt").unwrap());

    assert_eq!(expected, actual);
}

#[test]
fn tagtree_should_get_no_such_file_for_file_that_doesnt_exist() {
    let mut db = TagTreeDBFS::new();
    let files = file_list_from_iter_str([
        "/etc/fine/shoes/make.txt",
        "/etc/fine/shoes/blue.png",
        "/mnt/partition/fourteen.one",
    ]);

    add_files_to_db(&mut db, files);

    let query = File::from_str("/fine/etc/luke.txt").unwrap();

    let actual = db.get_file(&query);

    assert_eq!(Err(GetFileError::NoSuchFile), actual);
}

#[test]
fn tagtree_should_get_too_many_files_for_query_with_multiple_matches() {
    let mut db = TagTreeDBFS::new();
    let files = file_list_from_iter_str(["/etc/fine/shoes/make.txt", "/etc/fine/extra/make.txt"]);

    add_files_to_db(&mut db, files);

    let query = File::from_str("/fine/etc/make.txt").unwrap();

    let actual = db.get_file(&query);

    assert_eq!(Err(GetFileError::TooManyFiles), actual);
}

#[test]
fn found_files_should_have_all_tags_in_query() {
    let mut db = TagTreeDBFS::new();
    let files = file_list_from_iter_str([
        "/etc/fine/shoes/make.txt",
        "/etc/fine/shoes/blue.png",
        "/mnt/partition/fourteen.one",
    ]);

    add_files_to_db(&mut db, files);

    let query = TagSet::from_str("/fine/etc").unwrap();

    let result_files = db.get_files(&query);

    assert_eq!(2, result_files.len());

    for i in result_files {
        assert!(query.could_match(&i));
    }
}
