use anyhow::{anyhow, Context, Result};
use remote_files_configuration::{CliState, Configuration};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    env::{self, VarError},
    io,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};
use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncWriteExt},
};

pub static RF_HOME_ENV_VAR_NAME: &str = "RF_HOME";

pub static CONFIG_FILENAME: &str = "configuration.json";

pub static CLI_STATE_FILENAME: &str = "rf.json";

fn get_utf8_path(bytes: &[u8]) -> Cow<'_, str> {
    String::from_utf8_lossy(bytes)
}

pub async fn get_home_folder() -> Result<PathBuf> {
    let home = env::var(RF_HOME_ENV_VAR_NAME)
        .map(PathBuf::from)
        .or_else(|err| match err {
            VarError::NotPresent => dirs::config_dir().map(|pb| pb.join("rf")).ok_or(anyhow!(
                "cannot access os configuration directory".to_string(),
            )),
            VarError::NotUnicode(os_string) => Err(anyhow!(
                "env var '{RF_HOME_ENV_VAR_NAME}' does not contain valid unicode: {}",
                get_utf8_path(os_string.as_bytes())
            )),
        })?;

    let home_str = get_utf8_path(home.as_os_str().as_bytes()).to_string();
    match tokio::fs::metadata(&home).await {
        Ok(m) => m
            .is_dir()
            .then_some(home)
            .ok_or(anyhow!("path '{home_str}' is not a directory")),
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => Ok(home),
            _ => Err(anyhow!("cannot stat path '{home_str}': {err}")),
        },
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

    pub async fn persist(mut self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.inner)?;
        let bytes = content.as_bytes();

        self.fd.rewind().await?;
        self.fd.set_len(content.len() as u64).await?;
        self.fd.write_all(bytes).await?;
        self.fd.flush().await?;

        Ok(())
    }
}

async fn read_or_create<D>(path: &Path) -> Result<D>
where
    for<'de> D: Default + Serialize + Deserialize<'de>,
{
    let path_str = String::from_utf8_lossy(path.as_os_str().as_bytes());
    match tokio::fs::metadata(path).await.map(|m| m.is_file()) {
        Ok(true) => {
            let content = tokio::fs::read(path)
                .await
                .with_context(|| format!("reading file '{path_str}'"))?;
            serde_json::from_slice(&content)
                .with_context(|| format!("deserializing content of file {path_str}"))
        }
        Ok(false) => Err(anyhow!("path '{path_str}' is a directory")),
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => {
                let default_content = D::default();
                let content = serde_json::to_vec_pretty(&default_content)
                    .with_context(|| "serializing default content")?;

                tokio::fs::write(path, &content)
                    .await
                    .with_context(|| format!("writing file at '{path_str}'",))?;

                Ok(default_content)
            }
            _ => Err(err).with_context(|| format!("retrieving metadata for path '{path_str}'")),
        },
    }
}

pub async fn try_init(home: &Path) -> Result<(Stored<CliState>, Stored<Configuration>)> {
    let state_filepath = home.join(CLI_STATE_FILENAME);
    let config_filepath = home.join(CONFIG_FILENAME);

    tokio::fs::create_dir_all(&home).await.with_context(|| {
        format!(
            "creating home directory at path '{}'",
            get_utf8_path(home.as_os_str().as_bytes())
        )
    })?;

    let (cli_state, cfg) = tokio::join!(
        read_or_create::<CliState>(&state_filepath),
        read_or_create::<Configuration>(&config_filepath)
    );

    let mut fd_cli_state_opts = tokio::fs::OpenOptions::new();
    let mut fd_config_opts = tokio::fs::OpenOptions::new();
    let (fd_cli_state, fd_cfg) = tokio::join!(
        fd_cli_state_opts
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&state_filepath),
        fd_config_opts
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&config_filepath)
    );

    let state_filepath = get_utf8_path(state_filepath.as_os_str().as_bytes());
    let config_filepath = get_utf8_path(config_filepath.as_os_str().as_bytes());

    Ok((
        Stored {
            inner: cli_state.with_context(|| format!("reading cli state at {state_filepath}"))?,
            fd: fd_cli_state
                .with_context(|| format!("opening cli state file at path '{state_filepath}'"))?,
        },
        Stored {
            inner: cfg.with_context(|| format!("reading configuration at {config_filepath}"))?,
            fd: fd_cfg.with_context(|| {
                format!("opening configuration file at path '{state_filepath}'")
            })?,
        },
    ))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::needless_return)]

    use super::try_init;
    use assert_fs::{
        prelude::{FileWriteStr, PathChild},
        TempDir,
    };
    use indoc::indoc;
    use std::env;

    #[tokio::test]
    async fn should_create_new_cfg_folder() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().to_path_buf();
        // this drops the directory
        drop(dir);

        env::set_var("RF_HOME", path.as_os_str());

        try_init(&path).await.unwrap();

        let cfg_path = path.join("configuration.json");
        let cli_state_path = path.join("rf.json");

        assert_eq!(
            tokio::fs::read_to_string(&cfg_path).await.unwrap().as_str(),
            "{}"
        );
        assert_eq!(
            tokio::fs::read_to_string(&cli_state_path)
                .await
                .unwrap()
                .as_str(),
            "{}"
        );
    }

    #[tokio::test]
    async fn should_load_a_valid_configuration() {
        let dir = TempDir::new().unwrap();
        let cfg = dir.child("configuration.json");

        let cfg_content = indoc! {r#"
            {
                "gcs":{
                    "type": "gcs",
                    "configuration": {
                        "name": "my-gcs-bucket"
                    }
                }
            }
        "#};
        cfg.write_str(cfg_content).unwrap();

        env::set_var("RF_HOME", dir.path().as_os_str());

        try_init(dir.path()).await.unwrap();

        let cfg_path = dir.path().join("configuration.json");
        let cli_state_path = dir.path().join("rf.json");

        assert_eq!(
            tokio::fs::read_to_string(&cfg_path).await.unwrap().as_str(),
            cfg_content
        );
        assert_eq!(
            tokio::fs::read_to_string(&cli_state_path)
                .await
                .unwrap()
                .as_str(),
            "{}"
        );
    }

    #[tokio::test]
    async fn should_persist_a_valid_configuration() {
        let dir = TempDir::new().unwrap();
        let cfg = dir.child("configuration.json");

        let cfg_content = indoc! {r#"
            {
                "gcs":{
                    "type": "gcs",
                    "configuration": {
                        "name": "my-gcs-bucket"
                    }
                }
            }
        "#};
        cfg.write_str(cfg_content).unwrap();

        env::set_var("RF_HOME", dir.path().as_os_str());

        let (_, mut cfg) = try_init(dir.path()).await.unwrap();

        cfg.inner.buckets.clear();
        cfg.persist().await.unwrap();

        assert_eq!(
            tokio::fs::read_to_string(dir.path().join("configuration.json"))
                .await
                .unwrap()
                .as_str(),
            "{}"
        );
    }
    // #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    // async fn should_load_a_valid_configuration() {
    //     let dir = TempDir::new().unwrap();
    //     let cfg = dir.child("configuration.json");

    //     cfg.write_str(indoc! {r#"
    //         {
    //             "gcs":{
    //                 "type": "gcs",
    //                 "configuration": {
    //                     "name": "my-gcs-bucket"
    //                 }
    //             }
    //         }
    //     "#});

    //     let cfg_path = dir.add_file("configuration.json", cfg).await.unwrap();
    //     let cfg_layer = ConfigurationLayer::try_init(Some(&cfg_path)).await.unwrap();
    //     let cfg = cfg_layer.get();

    //     assert!(cfg.contains_key("gcs"));
    // }

    //     #[ignore]
    //     #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    //     async fn should_persist_new_configuration() {
    //         let dir = TmpDir::create_tmp_dir();
    //         let cfg_path = dir.add_file("configuration.json", r#"{}"#).await.unwrap();

    //         // initialize configuration
    //         let mut state_layer = PersistenceLayer::try_init(Some(&cfg_path)).await.unwrap();
    //         let state = state_layer.get_mut();

    //         // in-place edit & persistence
    //         state.current = Some(String::from("gcs"));
    //         state_layer.persist().await.unwrap();

    //         let new_state_layer = PersistenceLayer::try_init(Some(&cfg_path)).await.unwrap();
    //         assert_eq!(Some("gcs"), new_state_layer.get().current.as_deref());
    //     }
}
