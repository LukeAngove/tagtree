mod file;
mod naive;

pub use crate::naive::DBFS;
pub use crate::file::File;

#[cfg(test)]
mod tests {

    use crate::naive::DBFS;
    use crate::file::File;
    use crate::file::TagSet;
    use std::collections::btree_set::BTreeSet;
    use std::collections::hash_set::HashSet;

    fn file_list_from_iter_str<'a, I>(items: I) -> HashSet<File>
    where
        I: IntoIterator<Item = &'a str>,
    {
        items
            .into_iter()
            .map(|s| File::from_str(s).unwrap())
            .collect()
    }

    #[test]
    fn should_get_file_list_from_str_list() {
        let input = [
            "/etc/fine/shoes/make.txt",
            "/etc/fine/shoes/blue.png",
            "/mnt/partition/fourteen.one",
        ];

        let actual = file_list_from_iter_str(input);

        let expected: HashSet<File> = HashSet::from([
            File::new("make.txt", ["etc", "fine", "shoes"]),
            File::new("blue.png", ["etc", "fine", "shoes"]),
            File::new("fourteen.one", ["mnt", "partition"]),
        ]);

        assert_eq!(expected, actual);
    }

    #[test]
    fn should_convert_str_list_to_set() {
        let input = ["etc", "fine", "shoes", "make.txt"];

        let expected = BTreeSet::from_iter([
            "etc".to_string(),
            "fine".to_string(),
            "shoes".to_string(),
            "make.txt".to_string(),
        ]);

        let actual = TagSet::from_iter(input);

        assert_eq!(expected, actual.items);
    }

    #[test]
    fn should_make_file_from_str() {
        let input = "/etc/fine/shoes/make.txt";

        let expected = File::new("make.txt", ["etc", "fine", "shoes"]);

        let actual = File::from_str(&input);

        assert_eq!(Some(expected), actual);
    }

    #[test]
    fn dbfs_should_find_files() {
        let mut db = DBFS::new();

        let files = file_list_from_iter_str([
            "/etc/fine/shoes/make.txt",
            "/etc/fine/shoes/blue.png",
            "/mnt/partition/fourteen.one",
        ]);

        db.add_files(&files);

        let query = "/etc/fine";

        let expected_files: HashSet<File> =
            file_list_from_iter_str(["/etc/fine/shoes/make.txt", "/etc/fine/shoes/blue.png"]);

        assert_eq!(expected_files, db.get_files(query));
    }

    #[test]
    fn dbfs_should_find_files_with_different_query_order() {
        let mut db = DBFS::new();
        let files = file_list_from_iter_str([
            "/etc/fine/shoes/make.txt",
            "/etc/fine/shoes/blue.png",
            "/mnt/partition/fourteen.one",
        ]);

        db.add_files(&files);

        let expected_files = HashSet::from_iter([
            File::new("make.txt",  ["etc", "fine", "shoes"]),
            File::new("blue.png", ["etc", "fine", "shoes"]),
        ]);

        let query = "/fine/etc";

        assert_eq!(expected_files, db.get_files(query));
    }

    #[test]
    fn found_files_should_have_all_tags_in_query() {
        let mut db = DBFS::new();
        let files = file_list_from_iter_str([
            "/etc/fine/shoes/make.txt",
            "/etc/fine/shoes/blue.png",
            "/mnt/partition/fourteen.one",
        ]);

        db.add_files(&files);

        let expected_tags = TagSet::from_iter(["fine", "etc"]);
        let query = "/fine/etc";

        for i in db.get_files(query) {
            assert!(i.has_tags(&expected_tags));
        }
    }
}
