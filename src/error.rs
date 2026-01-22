use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum DupyError {
    IoError(io::Error),
    InvalidPath(PathBuf),
    InvalidArguments(String),
}

impl fmt::Display for DupyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DupyError::IoError(err) => write!(f, "I/O error: {}", err),
            DupyError::InvalidPath(path) => {
                write!(f, "Invalid or inaccessible path: {}", path.display())
            }
            DupyError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
        }
    }
}

impl std::error::Error for DupyError {}

impl From<io::Error> for DupyError {
    fn from(err: io::Error) -> Self {
        DupyError::IoError(err)
    }
}

pub type Result<T> = std::result::Result<T, DupyError>;
