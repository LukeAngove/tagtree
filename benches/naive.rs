use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rdb_fs::fromstr::FromStr;
use rdb_fs::{
    File, FileDB, FileQuery, HashTags2DBFS, HashTagsDBFS, NaiveDBFS, TagSet, TagTreeDBFS,
};
use std::fs::File as OSFile;
use serde_yaml::from_reader;
use std::collections::hash_set::HashSet;

pub fn load_file_names(file_name: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let file = OSFile::open(file_name)?;
    let conf: Vec<String> = from_reader(file)?;
    Ok(conf)
}


fn file_list_from_iter_str(items: impl IntoIterator<Item=impl AsRef<str>>) -> HashSet<File>
{
    items
        .into_iter()
        .map(|s| File::from_str(s.as_ref()).unwrap())
        .collect()
}

fn query_list_from_iter_str<'a, I>(items: I) -> HashSet<TagSet>
where
    I: IntoIterator<Item = &'a str>,
{
    items
        .into_iter()
        .map(|s| TagSet::from_str(s).unwrap())
        .collect()
}

fn add_files_to_db<'a, DB, I>(db: &mut DB, files: I) -> Option<()>
where
    DB: FileDB,
    I: IntoIterator<Item = File>,
{
    for f in files.into_iter() {
        db.add_file(&f)?;
    }
    Some(())
}

fn query_files_in_db<'a, DB, I, FQ>(db: &DB, queries: I) -> Option<()>
where
    DB: FileDB,
    I: IntoIterator<Item = FQ>,
    FQ: FileQuery,
{
    let mut results = HashSet::<File>::new();

    for q in queries.into_iter() {
        results = db.get_files(&q).collect();
    }

    if results.len() > 1 {
        Some(())
    } else {
        None
    }
}

pub fn criterion_benchmark_add(c: &mut Criterion) {
    let file_names = load_file_names("test_files2.yml").unwrap();
    let files = file_list_from_iter_str(file_names.into_iter());

    c.bench_function("naive_add_files", |b| {
        b.iter(|| assert_eq!(Some(()), add_files_to_db(black_box(&mut NaiveDBFS::new()), black_box(files.clone()))))
    });

    c.bench_function("hashtags_add_files", |b| {
        b.iter(|| {
            assert_eq!(Some(()), add_files_to_db(
                black_box(&mut HashTagsDBFS::new()),
                black_box(files.clone()),
            ))
        })
    });

    c.bench_function("hashtags2_add_files", |b| {
        b.iter(|| {
            assert_eq!(Some(()), add_files_to_db(
                black_box(&mut HashTags2DBFS::new()),
                black_box(files.clone()),
            ))
        })
    });

    c.bench_function("tagtree_add_files", |b| {
        b.iter(|| assert_eq!(Some(()), add_files_to_db(black_box(&mut TagTreeDBFS::new()), black_box(files.clone()))))
    });
}

pub fn criterion_benchmark_search(c: &mut Criterion) {
    let file_names = load_file_names("test_files2.yml").unwrap();
    let files = file_list_from_iter_str(file_names.into_iter());

    let mut naive = NaiveDBFS::new();
    add_files_to_db(&mut naive, files.clone());

    let mut hashtags = HashTagsDBFS::new();
    add_files_to_db(&mut hashtags, files.clone());

    let mut hashtags2 = HashTags2DBFS::new();
    add_files_to_db(&mut hashtags2, files.clone());

    let mut tagtree = TagTreeDBFS::new();
    add_files_to_db(&mut tagtree, files.clone());

    let queries = query_list_from_iter_str(["/home/luke/.cache", "/home/luke/Downloads"]);

    c.bench_function("naive_get_files", |b| {
        b.iter(|| query_files_in_db(black_box(&naive), black_box(queries.clone())))
    });

    c.bench_function("hashtags_get_files", |b| {
        b.iter(|| query_files_in_db(black_box(&hashtags), black_box(queries.clone())))
    });

    c.bench_function("hashtags_get_files", |b| {
        b.iter(|| query_files_in_db(black_box(&hashtags2), black_box(queries.clone())))
    });

    c.bench_function("tagtree_get_files", |b| {
        b.iter(|| query_files_in_db(black_box(&tagtree), black_box(queries.clone())))
    });
}

criterion_group!(benches, criterion_benchmark_add, criterion_benchmark_search);
criterion_main!(benches);
