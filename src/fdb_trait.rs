use crate::File;
use crate::FileQuery;

#[derive(PartialEq, Eq, Debug)]
pub enum GetFileError {
    NoSuchFile,
    TooManyFiles,
}

pub trait FileDB {
    type FileIterator: Iterator<Item = File>;

    // Add a file to the DB. Fail if it collides exatly
    // with an existing file.
    fn add_file(&mut self, new_files: &File) -> Option<()>;

    // Get the set of all files that match the given
    // query.
    fn get_files<F: FileQuery>(&self, query: &F) -> Self::FileIterator;

    // Get a single file, fail if there are multiple
    // or no matches.
    fn get_file<F: FileQuery>(&self, query: &F) -> Result<File, GetFileError>;
}
