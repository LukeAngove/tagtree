pub trait FromStr {
    fn from_str(path: &str) -> Option<Self> where Self: Sized;
}
