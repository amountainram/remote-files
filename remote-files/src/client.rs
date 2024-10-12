use crate::error;
use anyhow::Error;
use bytes::Buf;
use futures::{stream, FutureExt, Stream, StreamExt, TryStreamExt};
use human_format::{Formatter, Scales};
pub use opendal::EntryMode;
use opendal::{Entry, ErrorKind, Metakey, Operator};
use remote_files_configuration::{url_path::UrlDirPath, Bucket};
use std::{io::Read, path::Path};
use tabled::Tabled;
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};

type Result<T> = std::result::Result<T, error::Client>;

fn display_content_length(size: &Option<u64>) -> String {
    if let Some(size) = size {
        Formatter::new()
            .with_scales(Scales::Binary())
            .with_units("B")
            .format(*size as f64)
    } else {
        "".into()
    }
}

fn display_with_icons(entry: &EntryMode) -> String {
    match entry {
        EntryMode::DIR => "📂",
        _ => "",
    }
    .into()
}

#[derive(Clone, Tabled)]
pub struct StatEntry {
    #[tabled(rename = "name")]
    pub path: String,
    #[tabled(rename = "content-type")]
    pub content_type: String,
    #[tabled(rename = "size", display_with = "display_content_length")]
    pub content_length: Option<u64>,
    #[tabled(rename = "type", display_with = "display_with_icons")]
    pub r#type: EntryMode,
}

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
            EntryMode::FILE => Ok(StatEntry {
                path: path.to_string(),
                content_type: meta.content_type().unwrap_or_default().to_string(),
                content_length: meta.content_length().into(),
                r#type: EntryMode::FILE,
            }),
            EntryMode::DIR => Ok(StatEntry {
                path: path.to_string(),
                content_type: meta.content_type().unwrap_or_default().to_string(),
                content_length: None,
                r#type: EntryMode::DIR,
            }),
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
                        list.push(StatEntry {
                            path: entry.name().to_string(),
                            content_type: meta.content_type().unwrap_or_default().to_string(),
                            content_length: meta.content_length().into(),
                            r#type: EntryMode::FILE,
                        });
                    }
                    EntryMode::DIR => {
                        list.push(StatEntry {
                            path: entry.name().to_string(),
                            content_type: meta.content_type().unwrap_or_default().to_string(),
                            content_length: None,
                            r#type: EntryMode::DIR,
                        });
                    }
                }
            } else {
                println!("{:?}", meta.unwrap_err());
            }
        }

        list
    }

    pub async fn list(
        &self,
        path: &UrlDirPath,
        limit: Option<usize>,
    ) -> Result<impl Stream<Item = Vec<StatEntry>> + Unpin + Send + 'static> {
        let should_paginate = limit.is_some();
        let limit = limit.unwrap_or(DEFAULT_LIST_LIMIT);

        let path = path.to_string();
        let client = self.inner.clone();
        let entries = client
            .list_with(&path)
            .metakey(Metakey::ContentLength)
            .await
            .map_err(|err| match err.kind() {
                ErrorKind::NotADirectory => error::Client::ListNotDirectory(path.clone()),
                _ => error::Client::Unhandled(err),
            })?;

        let client = self.clone();
        let stream = stream::iter(entries);
        let stream = if should_paginate {
            stream.chunks(limit).boxed()
        } else {
            stream.chunks(10).take(1).boxed()
        };

        Ok(stream::unfold(
            (stream, client, path),
            |(mut stream, client, path)| async move {
                if let Some(next) = stream.next().await {
                    Some((
                        client.stat_entries(&path, next).await,
                        (stream, client, path),
                    ))
                } else {
                    None
                }
            },
        )
        .boxed())
    }

    pub async fn download(&self, path: &str) -> Result<Vec<u8>> {
        self.inner
            .read(path)
            .await
            .map_err(error::Client::Download)
            .and_then(|b| {
                let mut buffer = vec![];
                let mut reader = b.reader();
                reader.read_to_end(&mut buffer).map_err(|err| {
                    error::Client::Download(opendal::Error::new(
                        opendal::ErrorKind::Unexpected,
                        err.to_string(),
                    ))
                })?;
                Ok(buffer)
            })
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

impl TryFrom<Bucket> for Client {
    type Error = Error;
    fn try_from(value: Bucket) -> std::result::Result<Self, Self::Error> {
        match value {
            Bucket::Gcs(gcsconfig) => Ok(Client {
                inner: gcsconfig.try_into()?,
            }),
            Bucket::S3(s3_config) => Ok(Client {
                inner: s3_config.try_into()?,
            }),
        }
    }
}
