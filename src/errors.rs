use handlebars::{RenderError, TemplateError};
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Serialization error (Serde)")]
    Serialization,
    #[error("File not found")]
    FileNotFound,
    #[error("IO error")]
    Io,
    #[error("Unsupported metadata version: {0}")]
    UnsupportedMetadataVersion(u64),
    #[error("Metadata already exists")]
    MetadataAlreadyExists,
    #[error("Parsing error")]
    Parse,
    #[error("Error in handlebars template")]
    HandlebarsTemplate,
    #[error("Error rendering handlebars")]
    HandlebarsRender,
    #[error("Error connecting to or using GitHub's API")]
    GitHub,
    #[error("Error parsing a user-supplied tag: `{0}`")]
    ParseTag(String),
    #[error("Error parsing a command line argument: `{0}`")]
    ParseArg(String),
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Error {
        Error::Serialization
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        dbg!(&e);
        match e.kind() {
            io::ErrorKind::NotFound => Error::FileNotFound,
            _ => Error::Io,
        }
    }
}

impl From<TemplateError> for Error {
    fn from(e: TemplateError) -> Error {
        dbg!(&e);
        Error::HandlebarsTemplate
    }
}

impl From<RenderError> for Error {
    fn from(e: RenderError) -> Error {
        dbg!(&e);
        Error::HandlebarsRender
    }
}

impl From<octocrab::Error> for Error {
    fn from(e: octocrab::Error) -> Error {
        dbg!(&e);
        Error::GitHub
    }
}

pub type Result<T> = std::result::Result<T, Error>;
