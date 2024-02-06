use crate::{
    buckets::{GCSConfig, S3Config},
    error,
};
use futures::{stream, Future, Stream, StreamExt};
pub use opendal::EntryMode;
use opendal::{Entry, ErrorKind, Metakey, Operator};
use std::{path::Path, pin::Pin};
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};

type Result<T> = std::result::Result<T, error::Client>;

pub type StatEntry = (String, String, String, EntryMode);

const DEFAULT_LIST_LIMIT: usize = 10;

#[derive(Clone)]
pub struct Client {
    inner: Operator,
}

impl Client {
    pub async fn stat(&self, path: &str) -> Result<StatEntry> {
        let meta = self
            .inner
            .stat(path)
            .await
            .map_err(|err| error::Client::ListMetadata(path.to_string(), err))?;
        match meta.mode() {
            EntryMode::Unknown => Err(error::Client::StatUnknownMode(path.to_string())),
            EntryMode::FILE => Ok((
                path.to_string(),
                meta.content_type().unwrap_or_default().to_string(),
                meta.content_length().to_string(),
                EntryMode::FILE,
            )),
            EntryMode::DIR => Ok((
                path.to_string(),
                meta.content_type().unwrap_or_default().to_string(),
                String::from(""),
                EntryMode::DIR,
            )),
        }
    }

    async fn stat_entries(&self, path: &str, entries: Vec<Entry>) -> Vec<StatEntry> {
        let mut list = vec![];

        for entry in entries {
            let meta = self
                .inner
                .stat(entry.path())
                .await
                .map_err(|err| error::Client::ListMetadata(path.to_string(), err));

            if let Ok(meta) = meta {
                match meta.mode() {
                    EntryMode::Unknown => continue,
                    EntryMode::FILE => {
                        list.push((
                            entry.name().to_string(),
                            meta.content_type().unwrap_or_default().to_string(),
                            meta.content_length().to_string(),
                            EntryMode::FILE,
                        ));
                    }
                    EntryMode::DIR => {
                        list.push((
                            entry.name().to_string(),
                            meta.content_type().unwrap_or_default().to_string(),
                            String::from(""),
                            EntryMode::DIR,
                        ));
                    }
                }
            } else {
                println!("{:?}", meta.unwrap_err());
            }
        }

        list
    }

    pub async fn list<'a>(
        &'a self,
        path: &'a str,
        limit: Option<usize>,
    ) -> Result<Pin<Box<dyn Stream<Item = impl Future<Output = Vec<StatEntry>> + '_> + '_>>> {
        let should_paginate = limit.is_some();
        let limit = limit.unwrap_or(DEFAULT_LIST_LIMIT);

        let client = self.inner.clone();
        let entries = client
            .list_with(path)
            .metakey(Metakey::ContentLength)
            .await
            .map_err(|err| match err.kind() {
                ErrorKind::NotADirectory => error::Client::ListNotDirectory(path.to_string()),
                _ => error::Client::Unhandled(err),
            })?;

        let stream = stream::iter(entries).chunks(limit);

        if should_paginate {
            Ok(stream.map(|chunk| self.stat_entries(path, chunk)).boxed())
        } else {
            Ok(stream
                .take(1)
                .map(|chunk| self.stat_entries(path, chunk))
                .boxed())
        }
    }

    pub async fn download(&self, path: &str) -> Result<Vec<u8>> {
        self.inner.read(path).await.map_err(error::Client::Download)
    }

    pub async fn upload(&self, src: &str, dest: &str, content_type: Option<&str>) -> Result<()> {
        let filepath = Path::new(src);
        let filename = filepath
            .file_name()
            .ok_or_else(|| error::Client::UploadInvalidFilePath(src.to_string()))?;

        let file = File::open(filepath)
            .await
            .map_err(error::Client::UploadFileNotFound)?;
        let mut buffer: Vec<u8> = vec![];
        BufReader::new(file)
            .read_to_end(&mut buffer)
            .await
            .map_err(|err| error::Client::UploadLoad(src.to_string(), err))?;

        let dest = Path::new(dest).join(filename);
        let dest = dest.to_str().unwrap();
        match content_type {
            None => {
                self.inner
                    .write(dest, buffer)
                    .await
                    .map_err(|err| error::Client::UploadWrite(dest.to_string(), err))?;
            }
            Some(content_type) => {
                self.inner
                    .write_with(dest, buffer)
                    .content_type(content_type)
                    .await
                    .map_err(|err| error::Client::UploadWrite(dest.to_string(), err))?;
            }
        }

        Ok(())
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        println!("{}", path);
        self.inner
            .remove_all(path)
            .await
            .map_err(|error| error::Client::Delete {
                path: path.to_string(),
                error,
            })
    }
}

impl TryFrom<GCSConfig> for Client {
    type Error = error::Client;

    fn try_from(value: GCSConfig) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            inner: value.try_into().map_err(error::Client::Initialization)?,
        })
    }
}

impl TryFrom<S3Config> for Client {
    type Error = error::Client;

    fn try_from(value: S3Config) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            inner: value.try_into().map_err(error::Client::Initialization)?,
        })
    }
}
