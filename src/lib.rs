// TODO: test
// TODO: document
use std::borrow::Borrow;
use std::borrow::Cow;
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
pub enum AbsolutePathError {
    Absolutize { io_error: std::io::Error },
}

#[derive(Debug)]
pub enum RelativePathError {
    NoWorkingDirectory { io_error: std::io::Error },
    PathDiff,
    Absolutize { io_error: std::io::Error },
}

#[derive(Debug)]
pub enum FilePathError {
    ReadLink { io_error: std::io::Error },
    NotFile,
}

#[derive(Debug)]
pub enum DirectoryPathError {
    ReadLink { io_error: std::io::Error },
    NotDirectory,
}

#[derive(Debug, PartialEq)]
pub enum ExistingPathError {
    NotExist,
}

#[derive(Debug, PartialEq)]
pub enum NotExistingPathError {
    Exists,
}

impl Path<Absolute, Any> {
    pub fn new(path: impl AsRef<StdPath>) -> Result<Self, AbsolutePathError> {
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
            .map_err(|io_error| AbsolutePathError::Absolutize { io_error })?;
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
    ) -> Result<Self, AbsolutePathError> {
        let path = path.as_ref();
        if path.is_absolute() {
            return Ok(Self {
                inner: path.to_path_buf(),
                _phantom_ra: PhantomData {},
                _phantom_t: PhantomData {},
            });
        }

        let abs_path = path
            .absolutize_from(working_dir.borrow().as_ref())
            .map_err(|io_error| AbsolutePathError::Absolutize { io_error })?;
        debug_assert!(abs_path.is_absolute());
        Ok(Self {
            inner: abs_path.to_path_buf(),
            _phantom_ra: PhantomData {},
            _phantom_t: PhantomData {},
        })
    }
}

impl Path<Relative, Any> {
    pub fn new(path: impl AsRef<StdPath>) -> Result<Self, RelativePathError> {
        let working_dir = std::env::current_dir()
            .map_err(|io_error| RelativePathError::NoWorkingDirectory { io_error })?;
        Self::with_virtual_working_dir(&path, &working_dir)
    }

    pub fn with_virtual_working_dir(
        path: impl AsRef<StdPath>,
        working_dir: impl AsRef<StdPath>,
    ) -> Result<Self, RelativePathError> {
        let path = path.as_ref();
        let path = if path.is_relative() {
            path.absolutize()
                .map_err(|io_error| RelativePathError::Absolutize { io_error })?
        } else {
            Cow::Borrowed(path)
        };

        let rel_path =
            pathdiff::diff_paths(&path, &working_dir).ok_or_else(|| RelativePathError::PathDiff)?;
        debug_assert!(rel_path.is_relative());
        Ok(Self {
            inner: rel_path,
            _phantom_ra: PhantomData {},
            _phantom_t: PhantomData {},
        })
    }
}

impl Path<Any, File> {
    pub fn new(path: impl AsRef<StdPath>) -> Result<Self, FilePathError> {
        let path = path.as_ref();
        let path = if path.is_symlink() {
            match path.read_link() {
                Ok(p) => p,
                Err(io_error) => return Err(FilePathError::ReadLink { io_error }),
            }
        } else {
            path.to_path_buf()
        };

        if path.is_file() {
            Ok(Self {
                inner: path,
                _phantom_ra: PhantomData {},
                _phantom_t: PhantomData {},
            })
        } else {
            Err(FilePathError::NotFile)
        }
    }
}

impl Path<Any, Directory> {
    pub fn new(path: impl AsRef<StdPath>) -> Result<Self, DirectoryPathError> {
        let path = path.as_ref();
        let path = if path.is_symlink() {
            match path.read_link() {
                Ok(p) => p,
                Err(io_error) => return Err(DirectoryPathError::ReadLink { io_error }),
            }
        } else {
            path.to_path_buf()
        };

        if path.is_dir() {
            Ok(Self {
                inner: path,
                _phantom_ra: PhantomData {},
                _phantom_t: PhantomData {},
            })
        } else {
            Err(DirectoryPathError::NotDirectory)
        }
    }
}

impl Path<Any, Exist> {
    pub fn new(path: impl AsRef<StdPath>) -> Result<Self, ExistingPathError> {
        let path = path.as_ref();
        if path.exists() {
            Ok(Self {
                inner: path.to_path_buf(),
                _phantom_ra: PhantomData {},
                _phantom_t: PhantomData {},
            })
        } else {
            Err(ExistingPathError::NotExist)
        }
    }
}

impl Path<Any, NotExist> {
    pub fn new(path: impl AsRef<StdPath>) -> Result<Self, NotExistingPathError> {
        let path = path.as_ref();
        if !path.exists() {
            Ok(Self {
                inner: path.to_path_buf(),
                _phantom_ra: PhantomData {},
                _phantom_t: PhantomData {},
            })
        } else {
            Err(NotExistingPathError::Exists)
        }
    }
}

impl Path<Any, Any> {
    pub fn new(path: impl AsRef<StdPath>) -> Self {
        Self {
            inner: path.as_ref().to_path_buf(),
            _phantom_ra: PhantomData {},
            _phantom_t: PhantomData {},
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
