use std::marker::PhantomData;

pub type StdPath = std::path::Path;
pub type StdPathBuf = std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path<RA, T> {
    inner: StdPathBuf,
    _phantom_ra: PhantomData<RA>,
    _phantom_t: PhantomData<T>,
}

pub struct Absolute;
pub struct Relative;
pub struct Both;

pub struct File;
pub struct Directory;
pub struct Exist;
pub struct NotExist;
pub struct Any;

impl<RA, T> AsRef<StdPath> for Path<RA, T> {
    fn as_ref(&self) -> &StdPath {
        &self.inner
    }
}

pub enum Error {
    IO { io_error: std::io::Error },
}
