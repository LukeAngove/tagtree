use rdb_fs::fromstr::FromStr;
use rdb_fs::File;
use rdb_fs::FileDB;
use std::collections::hash_set::HashSet;

pub(crate) fn file_list_from_iter_str<'a, I>(items: I) -> HashSet<File>
where
    I: IntoIterator<Item = &'a str>,
{
    items
        .into_iter()
        .map(|s| File::from_str(s).unwrap())
        .collect()
}

pub(crate) fn add_files_to_db<'a, DB, I>(db: &mut DB, files: I) -> Option<()>
where
    DB: FileDB,
    I: IntoIterator<Item = File>,
{
    for f in files.into_iter() {
        db.add_file(&f)?;
    }
    Some(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::File;
    use std::collections::hash_set::HashSet;

    #[test]
    fn should_get_file_list_from_str_list() {
        let input = [
            "/etc/fine/shoes/make.txt",
            "/etc/fine/shoes/blue.png",
            "/mnt/partition/fourteen.one",
        ];

        let actual = file_list_from_iter_str(input);

        let expected: HashSet<File> = HashSet::from([
            File::new_cloned("make.txt", ["etc", "fine", "shoes"]),
            File::new_cloned("blue.png", ["etc", "fine", "shoes"]),
            File::new_cloned("fourteen.one", ["mnt", "partition"]),
        ]);

        assert_eq!(expected, actual);
    }
}
