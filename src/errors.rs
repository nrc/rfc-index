use std::io;

// TODO Display impl
#[derive(Debug)]
pub enum Error {
    Serialization,
    FileNotFound,
    Io,
    UnsupportedMetadataVersion(u64),
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Error {
        Error::Serialization
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        match e.kind() {
            io::ErrorKind::NotFound => Error::FileNotFound,
            _ => Error::Io,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
