use futures::future::join_all;
use uuid::Uuid;

mod common;

static TESTED_CLIENTS: [&str; 2] = ["gcs-test", "s3-test"];

///
/// This test uploads a text file to all available
/// clients and then downloads it to assert equality
///
/// Client cleanup is performed at the end of the task
///
/// `TESTED_CLIENTS` controls which configured client
/// is tested
///
#[tokio::test(flavor = "multi_thread")]
async fn should_upload_file() {
    // prepare local asset
    let folder = common::TmpDir::create_tmp_dir();
    let filename = "file.txt";
    let src = folder.add_file(filename, "all good 👌!").unwrap();
    let src = src.to_str().unwrap();

    // pick remote working directory
    let random_folder = Uuid::new_v4().to_string();
    let segments = ["test", "rf", random_folder.as_str()];
    let url = common::Url::create_dir_url(segments.as_ref());
    let dest = url.path();

    // prepare remote file urls
    let filepath = url.end_with_file(filename);
    let filepath = filepath.path();
    let files = [filepath.to_string()];

    // create clients
    let wrapped = common::WrappedClients::new(files.to_vec()).await;

    join_all(
        wrapped
            .clients
            .iter()
            .filter(|&(name, _)| TESTED_CLIENTS.contains(&name.as_ref()))
            .map(|(_, client)| async {
                // upload asset
                client.upload(src, dest, None).await.unwrap();

                // download asset and assert
                let bytes = client.download(filepath).await.unwrap();
                assert_eq!(Ok("all good 👌!"), String::from_utf8(bytes).as_deref());
            })
            .collect::<Vec<_>>(),
    )
    .await;
}
