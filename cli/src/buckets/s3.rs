use opendal::{services::S3, Error, Operator};
use serde::{Deserialize, Serialize};

use crate::opendal_builder;

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
