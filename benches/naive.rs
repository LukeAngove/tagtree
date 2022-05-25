use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rdb_fs::fromstr::FromStr;
use rdb_fs::{File, FileDB, FileQuery, HashTags2DBFS, HashTagsDBFS, NaiveDBFS, TagSet};
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
    let files = file_list_from_iter_str([
        "/etc/fine/shoes/make.txt",
        "/etc/fine/shoes/blue.png",
        "/mnt/partition/fourteen.one",
    ]);

    c.bench_function("naive_add_files", |b| {
        b.iter(|| add_files_to_db(black_box(&mut NaiveDBFS::new()), black_box(files.clone())))
    });

    c.bench_function("hashtags_add_files", |b| {
        b.iter(|| {
            add_files_to_db(
                black_box(&mut HashTagsDBFS::new()),
                black_box(files.clone()),
            )
        })
    });

    c.bench_function("hashtags2_add_files", |b| {
        b.iter(|| {
            add_files_to_db(
                black_box(&mut HashTags2DBFS::new()),
                black_box(files.clone()),
            )
        })
    });
}

pub fn criterion_benchmark_search(c: &mut Criterion) {
    let files = file_list_from_iter_str([
        "/etc/fine/shoes/make.txt",
        "/etc/fine/shoes/blue.png",
        "/mnt/partition/fourteen.one",
    ]);

    let mut naive = NaiveDBFS::new();
    add_files_to_db(&mut naive, files.clone());

    let mut hashtags = HashTagsDBFS::new();
    add_files_to_db(&mut hashtags, files.clone());

    let mut hashtags2 = HashTags2DBFS::new();
    add_files_to_db(&mut hashtags2, files.clone());

    let queries = query_list_from_iter_str(["/fine/etc/shoes", "/mnt/patition"]);

    c.bench_function("naive_get_files", |b| {
        b.iter(|| query_files_in_db(black_box(&naive), black_box(queries.clone())))
    });

    c.bench_function("hashtags_get_files", |b| {
        b.iter(|| query_files_in_db(black_box(&hashtags), black_box(queries.clone())))
    });

    c.bench_function("hashtags_get_files", |b| {
        b.iter(|| query_files_in_db(black_box(&hashtags2), black_box(queries.clone())))
    });
}

criterion_group!(benches, criterion_benchmark_add, criterion_benchmark_search);
criterion_main!(benches);
