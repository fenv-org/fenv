use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use tempfile::{NamedTempFile, TempDir};

#[derive(Debug, Clone)]
pub struct PathLike {
    inner: PathLikeInner,
}

impl PathLike {
    pub fn path(&self) -> &Path {
        match &self.inner {
            PathLikeInner::FromPath(path) => path,
            PathLikeInner::FromString(path) => Path::new(path),
        }
    }

    pub fn exists(&self) -> bool {
        self.path().exists()
    }

    pub fn is_dir(&self) -> bool {
        self.path().is_dir()
    }

    pub fn is_file(&self) -> bool {
        self.path().is_file()
    }

    pub fn join<P: AsRef<Path>>(&self, new_path: P) -> PathLike {
        PathLike::from(&self.path().join(new_path))
    }

    pub fn parent(&self) -> Option<PathLike> {
        self.path().parent().map(PathLike::from)
    }

    pub fn create_dir(&self) -> std::io::Result<()> {
        if !self.is_dir() {
            std::fs::create_dir(self.path())?
        }
        std::io::Result::Ok(())
    }

    pub fn create_dir_all(&self) -> std::io::Result<()> {
        if !self.is_dir() {
            std::fs::create_dir_all(self.path())?
        }
        std::io::Result::Ok(())
    }

    /// Opens a file in write-only mode.
    ///
    /// This function will create a file if it does not exist,
    /// and will truncate it if it does.
    pub fn create_file(&self) -> std::io::Result<std::fs::File> {
        if let Some(parent) = &self.parent() {
            parent.create_dir_all()?
        }
        std::fs::File::create(self.path())
    }

    /// Read the entire contents of a file into a string.
    ///
    /// This is a convenience function for using [`File::open`] and [`read_to_string`]
    /// with fewer imports and without an intermediate variable.
    ///
    /// [`read_to_string`]: Read::read_to_string
    pub fn read_to_string(&self) -> std::io::Result<String> {
        std::fs::read_to_string(self.path())
    }
}

impl AsRef<Path> for PathLike {
    fn as_ref(&self) -> &Path {
        self.path()
    }
}

impl From<&PathBuf> for PathLike {
    fn from(value: &PathBuf) -> Self {
        Self {
            inner: PathLikeInner::FromPath(value.clone()),
        }
    }
}

impl From<&Path> for PathLike {
    fn from(value: &Path) -> Self {
        Self {
            inner: PathLikeInner::FromPath(value.into()),
        }
    }
}

impl From<&str> for PathLike {
    fn from(value: &str) -> Self {
        Self {
            inner: PathLikeInner::FromString(value.to_string()),
        }
    }
}

impl From<&TempDir> for PathLike {
    fn from(value: &TempDir) -> Self {
        Self {
            inner: PathLikeInner::FromPath(value.path().to_path_buf()),
        }
    }
}

impl From<&NamedTempFile> for PathLike {
    fn from(value: &NamedTempFile) -> Self {
        Self {
            inner: PathLikeInner::FromPath(value.path().to_path_buf()),
        }
    }
}

impl From<&PathLike> for PathLike {
    fn from(value: &PathLike) -> Self {
        value.clone()
    }
}

impl Display for PathLike {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

#[derive(Debug, Clone)]
enum PathLikeInner {
    FromString(String),
    FromPath(PathBuf),
}

impl Display for PathLikeInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathLikeInner::FromString(path) => write!(f, "{}", path),
            PathLikeInner::FromPath(path) => write!(f, "{}", path.display()),
        }
    }
}
