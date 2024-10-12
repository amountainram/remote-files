use opendal::{
    services::{Gcs, S3},
    Error, Operator,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display, Formatter},
    path::PathBuf,
    str::FromStr,
};
use typed_builder::TypedBuilder;
use url_path::{UrlDirPath, UrlPath};
use zeroize::ZeroizeOnDrop;

pub mod url_path;
pub mod util;

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

impl From<String> for Secret {
    fn from(content: String) -> Self {
        Self { content }
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

make_enum_with_variants!(GcsStorageClass, STANDARD, NEARLINE, COLDLINE, ARCHIVE);

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, TypedBuilder)]
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

impl TryInto<Operator> for GCSConfig {
    type Error = Error;

    fn try_into(self) -> Result<Operator, Self::Error> {
        let builder = opendal_builder!(
            Gcs::default().bucket(&self.name),
            self.endpoint.map(|p| p.to_string()).as_deref() => endpoint,
            self.credential.as_ref().map(|p| p.read()) => credential,
            self.credential_path.as_ref().map(|p| p.to_string_lossy()).as_deref() => credential_path,
            self.prefix.map(|p| p.to_string()).as_deref() => root,
            self.predefined_acl.as_deref() => predefined_acl,
            self.default_storage_class.map(|p| p.to_string()).as_deref() => default_storage_class
        );

        let operator = Operator::new(builder)?.finish();

        Ok(operator)
    }
}

make_enum_with_variants!(
    S3StorageClass,
    DEEP_ARCHIVE,
    GLACIER,
    GLACIER_IR,
    INTELLIGENT_TIERING,
    ONEZONE_IA,
    OUTPOSTS,
    REDUCED_REDUNDANCY,
    STANDARD,
    STANDARD_IA
);

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

make_enum_with_variants!(BucketVariant, gcs, s3);

impl TryInto<Operator> for S3Config {
    type Error = Error;

    fn try_into(self) -> Result<Operator, Self::Error> {
        let builder = opendal_builder!(
            S3::default().bucket(&self.name),
            self.endpoint.map(|p| p.to_string()).as_deref() => endpoint,
            self.prefix.map(|p| p.to_string()).as_deref() => root,
            self.region.as_deref() => region,
            self.access_key_id.as_ref().map(|p| p.read()) => access_key_id,
            self.secret_access_key.as_ref().map(|p| p.read()) => secret_access_key,
            self.default_storage_class.map(|p| p.to_string()).as_deref() => default_storage_class
        );

        let operator = Operator::new(builder)?.finish();

        Ok(operator)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "type", content = "configuration")]
pub enum Bucket {
    #[serde(rename = "gcs")]
    Gcs(GCSConfig),
    #[serde(rename = "s3")]
    S3(S3Config),
}

impl From<Bucket> for BucketVariant {
    fn from(value: Bucket) -> Self {
        match value {
            Bucket::Gcs(_) => BucketVariant::gcs,
            Bucket::S3(_) => BucketVariant::s3,
        }
    }
}

impl FromStr for Bucket {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Configuration {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    schema: Option<String>,
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
