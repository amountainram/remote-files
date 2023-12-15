use futures::future::join_all;
use remote_files::{
    client::Client,
    configuration::{Bucket, ConfigurationLayer},
};
use std::{collections::HashMap, env};

#[derive(Default)]
pub struct WrappedClients {
    dropped: bool,
    files: Vec<String>,
    pub clients: HashMap<String, Client>,
}

impl WrappedClients {
    pub async fn new(files: Vec<String>) -> Self {
        let working_dir = env::current_dir().unwrap();
        let cfg_dir = working_dir.join(".rf").join("configuration.test.json");

        let cfg_layer = ConfigurationLayer::try_init(Some(&cfg_dir)).await.unwrap();
        let cfg = cfg_layer.get().to_owned();

        let clients = cfg
            .into_iter()
            .map(|(key, cfg)| {
                let client = match cfg {
                    Bucket::Gcs(gcs) => gcs.configuration.try_into(),
                    Bucket::S3(s3) => s3.configuration.try_into(),
                }
                .unwrap();

                (key, client)
            })
            .collect::<HashMap<_, Client>>();

        Self {
            dropped: Default::default(),
            files,
            clients,
        }
    }

    pub fn cleanup(&mut self) {
        futures::executor::block_on(async {
            tokio::spawn(join_all(self.clients.drain().map(|(_, client)| {
                let cloned_paths = self.files.clone();
                let cleanup = async move {
                    let _ = join_all(
                        cloned_paths
                            .iter()
                            .map(|path| async { client.delete(path.as_str()).await }),
                    )
                    .await;
                };

                cleanup
            })))
            .await
            .unwrap();
        })
    }
}

impl Drop for WrappedClients {
    fn drop(&mut self) {
        if !self.dropped {
            let mut dropped_clients = WrappedClients::default();
            std::mem::swap(&mut dropped_clients, self);
            dropped_clients.dropped = true;

            dropped_clients.cleanup();
        }
    }
}
