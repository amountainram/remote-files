use crate::{
    buckets::{GCSBucket, S3Bucket},
    client::Client,
    error::{self, CliStateError},
};
use configuration::{CliState, Configuration};
use futures::TryFutureExt;
use lazy_static::lazy_static;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::HashMap,
    env::{self, VarError},
    io,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};

pub static RF_HOME_ENV_VAR_NAME: &str = "RF_HOME";

pub static CONFIG_FILENAME: &str = "configuration.json";

pub static CLI_STATE_FILENAME: &str = "rf.json";

async fn get_home_folder() -> Result<PathBuf, CliStateError> {
    let home = env::var(RF_HOME_ENV_VAR_NAME)
        .map(PathBuf::from)
        .or_else(|err| match err {
            VarError::NotPresent => {
                dirs::config_dir()
                    .map(|pb| pb.join("rf"))
                    .ok_or(CliStateError::Initialization(
                        "cannot access os configuration directory".to_string(),
                    ))
            }
            VarError::NotUnicode(os_string) => Err(CliStateError::Initialization(format!(
                "env var '{RF_HOME_ENV_VAR_NAME}' does not contain valid unicode: {}",
                String::from_utf8_lossy(os_string.as_bytes())
            ))),
        })?;

    match tokio::fs::metadata(&home).await {
        Ok(m) => m
            .is_dir()
            .then_some(home.clone())
            .ok_or(CliStateError::Initialization(format!(
                "path '{}' is not a directory",
                String::from_utf8_lossy(home.as_os_str().as_bytes())
            ))),
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => Ok(home),
            _ => Err(CliStateError::Initialization(format!(
                "cannot stat path '{}': {}",
                String::from_utf8_lossy(home.as_os_str().as_bytes()),
                err
            ))),
        },
    }
}

async fn open_rw_fd<P>(path: P) -> Result<File, CliStateError>
where
    P: AsRef<Path>,
{
    let fd = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .await?;

    Ok(fd)
}

async fn read<D>(file: &mut File) -> Result<D, CliStateError>
where
    D: DeserializeOwned,
{
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).await?;

    if buffer.is_empty() {
        buffer.clear();
        buffer.push_str("{}");
    }

    let configuration = serde_json::from_str(&buffer)?;

    Ok(configuration)
}

pub fn create_client(profile: &str, cfg: &Configuration) -> Result<Option<Client>, error::Client> {
    if let Some(bucket) = cfg.get(profile) {
        let client: Client = match bucket {
            Bucket::Gcs(gcs) => gcs.configuration.clone().try_into()?,
            // .map_err(|err| ClientError::Initialization(err))?,
            Bucket::S3(s3) => s3.configuration.clone().try_into()?, // .map_err(|err| ClientError::Initialization(err))?,
        };

        Ok(Some(client))
    } else {
        Ok(None)
    }
}

pub struct Stored<T> {
    inner: T,
    fd: File,
}

impl<T> Stored<T>
where
    for<'de> T: Serialize + Deserialize<'de>,
{
    pub fn get(&self) -> &T {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub async fn persist(mut self) -> Result<(), CliStateError> {
        let content = serde_json::to_string_pretty(&self.inner)?;
        let bytes = content.as_bytes();

        self.fd.rewind().await?;
        self.fd.set_len(content.len() as u64).await?;
        self.fd.write_all(bytes).await?;
        self.fd.flush().await?;

        Ok(())
    }
}

async fn read_or_create<D>(path: &Path) -> Result<D, CliStateError>
where
    for<'de> D: Default + Serialize + Deserialize<'de>,
{
    let path_str = String::from_utf8_lossy(path.as_os_str().as_bytes());
    match tokio::fs::metadata(path).await.map(|m| m.is_file()) {
        Ok(true) => {
            let content = tokio::fs::read(path).await.map_err(|err| {
                CliStateError::Initialization(format!("cannot read config file: {err}"))
            })?;
            serde_json::from_slice(&content).map_err(|err| {
                CliStateError::Initialization(format!(
                    "cannot deserialize config file with default content: {err}"
                ))
            })
        }
        Ok(false) => Err(CliStateError::Initialization(format!(
            "path '{path_str}' is a directory",
        ))),
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => {
                let default_content = D::default();
                let content = serde_json::to_vec_pretty(&default_content).map_err(|_| {
                    CliStateError::Initialization(
                        "cannot serialize config file with default content".into(),
                    )
                })?;

                tokio::fs::write(path, &content).await.map_err(|err| {
                    CliStateError::Initialization(format!(
                        "cannot write file at '{path_str}': {err}",
                    ))
                })?;

                Ok(default_content)
            }
            _ => Err(CliStateError::Initialization(format!(
                "cannot stat file at '{path_str}': {err}",
            ))),
        },
    }
}

pub async fn try_init() -> Result<(Stored<CliState>, Configuration), CliStateError> {
    let home = get_home_folder().await?;

    let state_filepath = home.join(CLI_STATE_FILENAME);
    let config_filepath = home.join(CONFIG_FILENAME);

    tokio::fs::create_dir_all(&home).map_err(|err| {
        CliStateError::Initialization(format!(
            "cannot create home directory '{}': {err}",
            String::from_utf8_lossy(home.as_os_str().as_bytes())
        ))
    });

    let (cli_state, cfg) = tokio::join!(
        read_or_create::<CliState>(&state_filepath),
        read_or_create::<Configuration>(&config_filepath)
    );

    let fd = tokio::fs::File::open(state_filepath)
        .await
        .map_err(|_| CliStateError::Initialization("cannot open state file".into()))?;

    Ok((
        Stored {
            inner: cli_state?,
            fd,
        },
        cfg?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env, error, fs,
        path::{Path, PathBuf},
    };
    use tokio::fs::OpenOptions;
    use uuid::Uuid;

    struct TmpDir(PathBuf);

    impl TmpDir {
        fn create_tmp_dir() -> Self {
            let id = Uuid::new_v4().to_string();
            let tmp_dir = env::temp_dir().join(id.as_str());

            fs::create_dir(&tmp_dir).unwrap();

            TmpDir(tmp_dir)
        }

        async fn add_file<P>(
            &self,
            path: P,
            content: &str,
        ) -> Result<PathBuf, Box<dyn error::Error>>
        where
            P: AsRef<Path>,
        {
            let dst = self.0.join(path);
            let mut fd = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&dst)
                .await?;

            fd.write_all(content.as_bytes()).await?;
            fd.flush().await?;

            Ok(dst)
        }
    }

    impl Drop for TmpDir {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0)
                .unwrap_or_else(|_| panic!("cannot cleanup temp dir '{}'", self.0.display()));
        }
    }

    #[ignore]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_load_a_valid_configuration() {
        let dir = TmpDir::create_tmp_dir();
        let cfg = r#"{
            "gcs":{
                "type": "gcs",
                "configuration": {
                    "name": "my-gcs-bucket"
                }
            }
        }"#;

        let cfg_path = dir.add_file("configuration.json", cfg).await.unwrap();
        let cfg_layer = ConfigurationLayer::try_init(Some(&cfg_path)).await.unwrap();
        let cfg = cfg_layer.get();

        assert!(cfg.contains_key("gcs"));
    }

    #[ignore]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_persist_new_configuration() {
        let dir = TmpDir::create_tmp_dir();
        let cfg_path = dir.add_file("configuration.json", r#"{}"#).await.unwrap();

        // initialize configuration
        let mut state_layer = PersistenceLayer::try_init(Some(&cfg_path)).await.unwrap();
        let state = state_layer.get_mut();

        // in-place edit & persistence
        state.current = Some(String::from("gcs"));
        state_layer.persist().await.unwrap();

        let new_state_layer = PersistenceLayer::try_init(Some(&cfg_path)).await.unwrap();
        assert_eq!(Some("gcs"), new_state_layer.get().current.as_deref());
    }
}
