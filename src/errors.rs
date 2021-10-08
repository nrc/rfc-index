use handlebars::{RenderError, TemplateError};
use std::io;

// TODO Display impl
#[derive(Debug)]
pub enum Error {
    Serialization,
    FileNotFound,
    Io,
    UnsupportedMetadataVersion(u64),
    MetadataAlreadyExists,
    Parse,
    HandlebarsTemplate,
    HandlebarsRender,
    GitHub,
    MissingMetadata,
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

impl From<TemplateError> for Error {
    fn from(e: TemplateError) -> Error {
        eprintln!("{}", e);
        Error::HandlebarsTemplate
    }
}

impl From<RenderError> for Error {
    fn from(_: RenderError) -> Error {
        Error::HandlebarsRender
    }
}

impl From<octocrab::Error> for Error {
    fn from(_: octocrab::Error) -> Error {
        Error::GitHub
    }
}

pub type Result<T> = std::result::Result<T, Error>;
