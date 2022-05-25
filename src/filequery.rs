use crate::File;
use crate::TagSet;

pub trait FileQuery {
    fn could_match(&self, to_match: &File) -> bool;
    fn tags(&self) -> &TagSet;
    fn name(&self) -> Option<&str>;
}
