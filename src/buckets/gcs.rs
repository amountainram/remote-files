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
        let mut builder = Gcs::default();

        builder.bucket(&self.name);

        self.endpoint.map(|endpoint| builder.endpoint(&endpoint));
        self.credential
            .map(|credential| builder.credential(&credential));
        self.credential_path
            .map(|credential_path| builder.credential_path(&credential_path));
        self.prefix.map(|root| builder.root(&root));
        self.predefined_acl
            .map(|predefined_acl| builder.predefined_acl(&predefined_acl));
        self.default_storage_class
            .map(|storage_class| builder.default_storage_class(&storage_class));

        let operator = Operator::new(builder)?.finish();

        Ok(operator)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GCSBucket {
    pub configuration: GCSConfig,
}
