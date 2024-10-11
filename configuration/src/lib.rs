//use crate::{
//    buckets::{GCSBucket, S3Bucket},
//    client::Client,
//    error::{self, StoredError},
//};
//use serde::{de::DeserializeOwned, Deserialize, Serialize};
//use std::{
//    collections::HashMap,
//    path::{Path, PathBuf},
//};
//use tokio::{
//    fs::{File, OpenOptions},
//    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
//};
//
//pub static CONFIGURATION_FILEPATH_ENV_VAR: &str = "RF_CFG_FILEPATH";
//
//pub fn get_default_folder() -> Result<PathBuf, StoredError> {
//    dirs::config_dir()
//        .map(|pb| pb.join("rf"))
//        .ok_or(StoredError::Initialization(
//            "cannot access configuration directory".to_string(),
//        ))
//}
//
//async fn open_rw_fd<P>(path: P) -> Result<File, StoredError>
//where
//    P: AsRef<Path>,
//{
//    let fd = OpenOptions::new()
//        .read(true)
//        .write(true)
//        .create(true)
//        .truncate(true)
//        .open(path)
//        .await?;
//
//    Ok(fd)
//}
//
//async fn read<D>(file: &mut File) -> Result<D, StoredError>
//where
//    D: DeserializeOwned,
//{
//    let mut buffer = String::new();
//    file.read_to_string(&mut buffer).await?;
//
//    if buffer.is_empty() {
//        buffer.clear();
//        buffer.push_str("{}");
//    }
//
//    let configuration = serde_json::from_str(&buffer)?;
//
//    Ok(configuration)
//}
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display, Formatter},
    path::PathBuf,
};
use url_path::{UrlDirPath, UrlPath};
use zeroize::ZeroizeOnDrop;

pub mod url_path;

/// Represents a plaintext secret. It is read as-is from the configuration file.
#[derive(Clone, PartialEq, ZeroizeOnDrop, Deserialize, Serialize, JsonSchema)]
#[serde(transparent, rename_all = "camelCase")]
pub struct Secret {
    #[zeroize]
    content: String,
}

impl Secret {
    /// Reads the content of a secret.
    pub fn read(&self) -> &str {
        self.content.as_str()
    }
}

impl Debug for Secret {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl Display for Secret {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl AsRef<str> for Secret {
    fn as_ref(&self) -> &str {
        self.read()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum GcsStorageClass {
    STANDARD,
    NEARLINE,
    COLDLINE,
    ARCHIVE,
}

impl From<&GcsStorageClass> for &'static str {
    fn from(value: &GcsStorageClass) -> Self {
        match value {
            GcsStorageClass::STANDARD => "STANDARD",
            GcsStorageClass::NEARLINE => "NEARLINE",
            GcsStorageClass::COLDLINE => "COLDLINE",
            GcsStorageClass::ARCHIVE => "ARCHIVE",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GCSConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential: Option<Secret>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_storage_class: Option<GcsStorageClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<UrlPath>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<UrlDirPath>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predefined_acl: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum S3StorageClass {
    #[allow(non_camel_case_types)]
    DEEP_ARCHIVE,
    GLACIER,
    #[allow(non_camel_case_types)]
    GLACIER_IR,
    #[allow(non_camel_case_types)]
    INTELLIGENT_TIERING,
    #[allow(non_camel_case_types)]
    ONEZONE_IA,
    OUTPOSTS,
    #[allow(non_camel_case_types)]
    REDUCED_REDUNDANCY,
    STANDARD,
    #[allow(non_camel_case_types)]
    STANDARD_IA,
}

impl From<&S3StorageClass> for &'static str {
    fn from(value: &S3StorageClass) -> Self {
        match value {
            S3StorageClass::DEEP_ARCHIVE => "DEEP_ARCHIVE",
            S3StorageClass::GLACIER => "GLACIER",
            S3StorageClass::GLACIER_IR => "GLACIER_IR",
            S3StorageClass::INTELLIGENT_TIERING => "INTELLIGENT_TIERING",
            S3StorageClass::ONEZONE_IA => "ONEZONE_IA",
            S3StorageClass::OUTPOSTS => "OUTPOSTS",
            S3StorageClass::REDUCED_REDUNDANCY => "REDUCED_REDUNDANCY",
            S3StorageClass::STANDARD => "STANDARD",
            S3StorageClass::STANDARD_IA => "STANDARD_IA",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct S3Config {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<UrlPath>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<UrlDirPath>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_key_id: Option<Secret>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_access_key: Option<Secret>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_storage_class: Option<S3StorageClass>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "type", content = "configuration")]
pub enum Bucket {
    #[serde(rename = "gcs")]
    Gcs(GCSConfig),
    #[serde(rename = "s3")]
    S3(S3Config),
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Configuration {
    #[serde(rename = "$schema")]
    _schema: String,
    #[serde(flatten)]
    pub buckets: HashMap<String, Bucket>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CliState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::Configuration;
    use rstest::rstest;
    use serde_json::{json, Value};

    #[rstest]
    #[case(json!({}))]
    #[case(json!({
        "my-bucket": {
            "type": "gcs",
            "configuration": {
                "name": "my-bucket",
                "prefix": "a-prefix",
                "credentialPath": "/path/to/file",
                "defaultStorageClass": "COLDLINE"
            }
        }
    }))]
    fn configuration_spec(#[case] input: Value) {
        assert!(serde_json::from_value::<Configuration>(input).is_ok())
    }
}
