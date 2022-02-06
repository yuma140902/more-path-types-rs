use std::marker::PhantomData;

use path_absolutize::Absolutize;

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

#[derive(Debug)]
pub enum Error {
    IO { io_error: std::io::Error },
}

impl<T> Path<Absolute, T> {
    pub fn new(path: impl AsRef<StdPath>) -> Result<Self, Error> {
        let path = path.as_ref();
        if path.is_absolute() {
            return Ok(Self {
                inner: path.to_path_buf(),
                _phantom_ra: PhantomData {},
                _phantom_t: PhantomData {},
            });
        }
        let abs_path = path
            .absolutize()
            .map_err(|io_error| Error::IO { io_error })?;
        debug_assert!(abs_path.is_absolute());
        Ok(Self {
            inner: abs_path.to_path_buf(),
            _phantom_ra: PhantomData {},
            _phantom_t: PhantomData {},
        })
    }

    pub fn with_virtual_working_dir(
        path: impl AsRef<StdPath>,
        working_dir: impl AsRef<StdPath>,
    ) -> Result<Self, Error> {
        let path = path.as_ref();
        if path.is_absolute() {
            return Ok(Self {
                inner: path.to_path_buf(),
                _phantom_ra: PhantomData {},
                _phantom_t: PhantomData {},
            });
        }
        let abs_path = path
            .absolutize_virtually(working_dir)
            .map_err(|io_error| Error::IO { io_error })?;
        debug_assert!(abs_path.is_absolute());
        Ok(Self {
            inner: abs_path.to_path_buf(),
            _phantom_ra: PhantomData {},
            _phantom_t: PhantomData {},
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abs_path_new() {
        let path = Path::<Absolute, Any>::new("/absolute/path");
        assert!(path.is_ok()); //TODO
        let path = Path::<Absolute, Any>::new("/absolute/path/with/../dots");
        assert!(path.is_ok());
        let path = Path::<Absolute, Any>::new("relative/path");
        assert!(path.is_ok());
        let path = Path::<Absolute, Any>::new("./relative/path/with/dots");
        assert!(path.is_ok());
    }
}
