// TODO: test
// TODO: document
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
    PathDiff,
    NotFile,
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

impl<T> Path<Relative, T> {
    pub fn new(path: impl AsRef<StdPath>) -> Result<Self, Error> {
        let working_dir = std::env::current_dir().map_err(|io_error| Error::IO { io_error })?;
        Self::with_virtual_working_dir(&path, &working_dir)
    }

    pub fn with_virtual_working_dir(
        path: impl AsRef<StdPath>,
        working_dir: impl AsRef<StdPath>,
    ) -> Result<Self, Error> {
        let path = path.as_ref();
        if path.is_relative() {
            return Ok(Self {
                inner: path.to_path_buf(),
                _phantom_ra: PhantomData {},
                _phantom_t: PhantomData {},
            });
        }

        let rel_path = pathdiff::diff_paths(&path, &working_dir).ok_or_else(|| Error::PathDiff)?;
        debug_assert!(rel_path.is_relative());
        Ok(Self {
            inner: rel_path,
            _phantom_ra: PhantomData {},
            _phantom_t: PhantomData {},
        })
    }
}

impl Path<Both, File> {
    pub fn new(path: impl AsRef<StdPath>) -> Result<Self, Error> {
        let path = path.as_ref();
        if path.is_file() {
            Ok(Self {
                inner: path.to_path_buf(),
                _phantom_ra: PhantomData {},
                _phantom_t: PhantomData {},
            })
        } else {
            Err(Error::NotFile)
        }
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
