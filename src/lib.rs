#[cfg(test)]
extern crate proptest;

mod fdb_trait;
mod file;
mod filequery;
pub mod fromstr;
mod hashtags;
mod hashtags2;
mod naive;
mod tagset;
mod tagtree;

pub use crate::fdb_trait::FileDB;
pub use crate::fdb_trait::GetFileError;
pub use crate::file::File;
pub use crate::filequery::FileQuery;
pub use crate::hashtags::HashTagsDBFS;
pub use crate::hashtags2::HashTags2DBFS;
pub use crate::naive::NaiveDBFS;
pub use crate::tagset::TagSet;
pub use crate::tagtree::TagTreeDBFS;
