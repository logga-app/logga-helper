use std::num::ParseIntError;
use std::{fmt, io};

#[derive(Debug)]
pub enum TailError {
    FileOpenError(String),
    ReadMetadataError(io::Error),
    StartSeekError(io::Error),
    BufferReadError(io::Error),
    FileOperationError(io::Error),
    CastError(ParseIntError),
}

impl fmt::Display for TailError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TailError::FileOpenError(path) => write!(f, "failed to open {}", path),
            TailError::ReadMetadataError(err) => write!(f, "read metadata: {:?}", err),
            TailError::StartSeekError(err) => write!(f, "start seeking: {:?}", err),
            TailError::BufferReadError(err) => write!(f, "buffer reader: {:?}", err),
            TailError::FileOperationError(err) => write!(f, "detecting file operation: {:?}", err),
            TailError::CastError(err) => write!(f, "cast string to int: {:?}", err),
        }
    }
}

impl From<std::io::Error> for TailError {
    fn from(err: std::io::Error) -> Self {
        TailError::ReadMetadataError(err)
    }
}
