use opendal::{services::S3, Error, Operator};
use serde::{Deserialize, Serialize};

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
        let mut builder = S3::default();

        builder.bucket(&self.name);

        self.endpoint
            .map(|endpoint| {
                builder.endpoint(&endpoint);
            })
            .unwrap_or_else(|| {
                builder.enable_virtual_host_style();
            });
        self.prefix.map(|root| builder.root(&root));
        self.region.map(|region| builder.region(&region));
        self.access_key_id.map(|id| builder.access_key_id(&id));
        self.secret_access_key
            .map(|key| builder.secret_access_key(&key));
        self.default_storage_class
            .map(|storage_class| builder.default_storage_class(&storage_class));

        let operator = Operator::new(builder)?.finish();

        Ok(operator)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct S3Bucket {
    pub configuration: S3Config,
}
