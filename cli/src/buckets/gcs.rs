use crate::opendal_builder;
use opendal::{services::Gcs, Error, Operator};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GCSConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
    #[serde(rename = "credentialPath", skip_serializing_if = "Option::is_none")]
    pub credential_path: Option<String>,
    #[serde(
        rename = "defaultStorageClass",
        skip_serializing_if = "Option::is_none"
    )]
    pub default_storage_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    #[serde(rename = "predefinedAcl", skip_serializing_if = "Option::is_none")]
    pub predefined_acl: Option<String>,
}

impl TryInto<Operator> for GCSConfig {
    type Error = Error;

    fn try_into(self) -> Result<Operator, Self::Error> {
        let builder = opendal_builder!(
            Gcs::default().bucket(&self.name),
            self.endpoint.as_deref() => endpoint,
            self.credential.as_deref() => credential,
            self.credential_path.as_deref() => credential_path,
            self.prefix.as_deref() => root,
            self.predefined_acl.as_deref() => predefined_acl,
            self.default_storage_class.as_deref() => default_storage_class
        );

        let operator = Operator::new(builder)?.finish();

        Ok(operator)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GCSBucket {
    pub configuration: GCSConfig,
}
