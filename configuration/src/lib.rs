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
};
use zeroize::ZeroizeOnDrop;

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
#[serde(rename_all = "camelCase")]
pub struct GCSConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential: Option<Secret>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_storage_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predefined_acl: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct S3Config {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_key_id: Option<Secret>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_access_key: Option<Secret>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_storage_class: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "type", content = "configuration")]
pub enum Bucket {
    #[serde(rename = "gcs")]
    Gcs(GCSConfig),
    #[serde(rename = "s3")]
    S3(S3Config),
}

pub type Configuration = HashMap<String, Bucket>;

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CliState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<String>,
}

//pub fn create_client(profile: &str, cfg: &Configuration) -> Result<Option<Client>, error::Client> {
//    if let Some(bucket) = cfg.get(profile) {
//        let client: Client = match bucket {
//            Bucket::Gcs(gcs) => gcs.configuration.clone().try_into()?,
//            // .map_err(|err| ClientError::Initialization(err))?,
//            Bucket::S3(s3) => s3.configuration.clone().try_into()?, // .map_err(|err| ClientError::Initialization(err))?,
//        };
//
//        Ok(Some(client))
//    } else {
//        Ok(None)
//    }
//}
//
//pub struct Stored<T>
//where
//    T: DeserializeOwned + Serialize,
//{
//    inner: T,
//    fd: File,
//}
//
//impl<T> Stored<T>
//where
//    T: DeserializeOwned + Serialize,
//{
//    pub fn get(&self) -> &T {
//        &self.inner
//    }
//
//    pub fn get_mut(&mut self) -> &mut T {
//        &mut self.inner
//    }
//
//    pub async fn persist(mut self) -> Result<(), StoredError> {
//        let content = serde_json::to_string_pretty(&self.inner)?;
//        let bytes = content.as_bytes();
//
//        self.fd.rewind().await?;
//        self.fd.set_len(content.len() as u64).await?;
//        self.fd.write_all(bytes).await?;
//        self.fd.flush().await?;
//
//        Ok(())
//    }
//}
//
//pub type PersistenceLayer = Stored<Persistence>;
//
//impl PersistenceLayer {
//    pub async fn try_init(value: Option<&Path>) -> Result<Self, StoredError> {
//        let main_folder = get_default_folder()?;
//
//        let default_persistence_filepath = main_folder.join("rf.json");
//        let persistence_filepath = value
//            .map(PathBuf::from)
//            .unwrap_or(default_persistence_filepath);
//
//        let mut persistence_fd = open_rw_fd(persistence_filepath.as_path()).await?;
//
//        Ok(PersistenceLayer {
//            inner: read(&mut persistence_fd).await?,
//            fd: persistence_fd,
//        })
//    }
//}
//
//pub type ConfigurationLayer = Stored<Configuration>;
//
//impl ConfigurationLayer {
//    pub async fn try_init(value: Option<&Path>) -> Result<Self, StoredError> {
//        let main_folder = get_default_folder()?;
//
//        let default_cfg_filepath = main_folder.join("configuration.json");
//        let cfg_filepath = value.map(PathBuf::from).unwrap_or(default_cfg_filepath);
//
//        let mut cfg_fd = open_rw_fd(cfg_filepath.as_path()).await?;
//
//        Ok(ConfigurationLayer {
//            inner: read(&mut cfg_fd).await?,
//            fd: cfg_fd,
//        })
//    }
//}
//
//#[cfg(test)]
//mod tests {
//    use super::*;
//    use std::{
//        env, error, fs,
//        path::{Path, PathBuf},
//    };
//    use tokio::fs::OpenOptions;
//    use uuid::Uuid;
//
//    struct TmpDir(PathBuf);
//
//    impl TmpDir {
//        fn create_tmp_dir() -> Self {
//            let id = Uuid::new_v4().to_string();
//            let tmp_dir = env::temp_dir().join(id.as_str());
//
//            fs::create_dir(&tmp_dir).unwrap();
//
//            TmpDir(tmp_dir)
//        }
//
//        async fn add_file<P>(
//            &self,
//            path: P,
//            content: &str,
//        ) -> Result<PathBuf, Box<dyn error::Error>>
//        where
//            P: AsRef<Path>,
//        {
//            let dst = self.0.join(path);
//            let mut fd = OpenOptions::new()
//                .create(true)
//                .truncate(true)
//                .write(true)
//                .open(&dst)
//                .await?;
//
//            fd.write_all(content.as_bytes()).await?;
//            fd.flush().await?;
//
//            Ok(dst)
//        }
//    }
//
//    impl Drop for TmpDir {
//        fn drop(&mut self) {
//            fs::remove_dir_all(&self.0)
//                .unwrap_or_else(|_| panic!("cannot cleanup temp dir '{}'", self.0.display()));
//        }
//    }
//
//    #[ignore]
//    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
//    async fn should_load_a_valid_configuration() {
//        let dir = TmpDir::create_tmp_dir();
//        let cfg = r#"{
//            "gcs":{
//                "type": "gcs",
//                "configuration": {
//                    "name": "my-gcs-bucket"
//                }
//            }
//        }"#;
//
//        let cfg_path = dir.add_file("configuration.json", cfg).await.unwrap();
//        let cfg_layer = ConfigurationLayer::try_init(Some(&cfg_path)).await.unwrap();
//        let cfg = cfg_layer.get();
//
//        assert!(cfg.contains_key("gcs"));
//    }
//
//    #[ignore]
//    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
//    async fn should_persist_new_configuration() {
//        let dir = TmpDir::create_tmp_dir();
//        let cfg_path = dir.add_file("configuration.json", r#"{}"#).await.unwrap();
//
//        // initialize configuration
//        let mut state_layer = PersistenceLayer::try_init(Some(&cfg_path)).await.unwrap();
//        let state = state_layer.get_mut();
//
//        // in-place edit & persistence
//        state.current = Some(String::from("gcs"));
//        state_layer.persist().await.unwrap();
//
//        let new_state_layer = PersistenceLayer::try_init(Some(&cfg_path)).await.unwrap();
//        assert_eq!(Some("gcs"), new_state_layer.get().current.as_deref());
//    }
//}

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
                "name": "my-bucket"
            }
        }
    }))]
    fn configuration_spec(#[case] input: Value) {
        assert!(serde_json::from_value::<Configuration>(input).is_ok())
    }
}
