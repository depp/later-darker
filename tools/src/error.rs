use std::error;
use std::fmt;
use std::path::PathBuf;

/// An error with an associated filename.
#[derive(Debug)]
pub struct FileError {
    pub path: PathBuf,
    pub error: Box<dyn error::Error>,
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.error)
    }
}

impl error::Error for FileError {}
