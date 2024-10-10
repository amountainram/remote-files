use opendal::{services::S3, Error, Operator};
use serde::{Deserialize, Serialize};

use crate::opendal_builder;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct S3Config {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(rename = "accessKeyId", skip_serializing_if = "Option::is_none")]
    pub access_key_id: Option<String>,
    #[serde(rename = "secretAccessKey", skip_serializing_if = "Option::is_none")]
    pub secret_access_key: Option<String>,
    #[serde(
        rename = "defaultStorageClass",
        skip_serializing_if = "Option::is_none"
    )]
    pub default_storage_class: Option<String>,
}

impl TryInto<Operator> for S3Config {
    type Error = Error;

    fn try_into(self) -> Result<Operator, Self::Error> {
        let builder = opendal_builder!(
            S3::default().bucket(&self.name),
            self.endpoint.as_deref() => endpoint,
            self.prefix.as_deref() => root,
            self.region.as_deref() => region,
            self.access_key_id.as_deref() => access_key_id,
            self.secret_access_key.as_deref() => secret_access_key,
            self.default_storage_class.as_deref() => default_storage_class
        );

        let operator = Operator::new(builder)?.finish();

        Ok(operator)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct S3Bucket {
    pub configuration: S3Config,
}
