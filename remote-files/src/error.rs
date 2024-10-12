use std::io;

pub type Result<T, E = Error> = std::prelude::v1::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Client {
    #[error("error while initializing client: {}", 0)]
    Initialization(opendal::Error),
    #[error("unhandled error: {}", 0)]
    Unhandled(opendal::Error),
    #[error("unknown entry mode for path '{}'", 0)]
    StatUnknownMode(String),
    #[error("path '{}' is not a directory", 0)]
    ListNotDirectory(String),
    #[error("invalid metadata for path '{}'", 0)]
    ListMetadata(String, opendal::Error),
    #[error("cannot download resource: {}", 0)]
    Download(opendal::Error),
    #[error("invalid path {}", 0)]
    UploadInvalidFilePath(String),
    #[error("cannot find file: {}", 0)]
    UploadFileNotFound(io::Error),
    #[error("error while reading file {}", 0)]
    UploadLoad(String, io::Error),
    #[error("cannot write to path {}", 0)]
    UploadWrite(String, opendal::Error),
    #[error("cannot delete path {}: {}", path, error)]
    Delete { path: String, error: opendal::Error },
}

#[derive(Debug, thiserror::Error)]
pub enum CliStateError {
    #[error("state initialization error {}", 0)]
    Initialization(String),
    #[error("os I/O operation error {:?}", 0)]
    IO(#[from] io::Error),
    #[error("serialization/deserialization error {:?}", 0)]
    JSON(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    State(#[from] CliStateError),
    Cli(#[from] Client),
}
