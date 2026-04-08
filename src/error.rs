use std::fmt;
use std::io;

/// Errors returned by this crate.
#[derive(Debug)]
pub enum Error {
    /// An I/O error (socket, bind, send, recv).
    Io(io::Error),
    /// A received packet has an invalid or unexpected format.
    InvalidResponse(String),
    /// A string argument exceeds the field's maximum length.
    StringTooLong {
        field: &'static str,
        max: usize,
        actual: usize,
    },
    /// A received packet has an unrecognized 4-byte tag.
    UnknownTag([u8; 4]),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::InvalidResponse(msg) => write!(f, "Invalid response: {msg}"),
            Error::StringTooLong { field, max, actual } => {
                write!(f, "'{field}' too long: max {max} bytes, got {actual}")
            }
            Error::UnknownTag(tag) => match std::str::from_utf8(tag) {
                Ok(s) => write!(f, "Unknown message tag: {s}"),
                Err(_) => write!(f, "Unknown message tag: {tag:?}"),
            },
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}
