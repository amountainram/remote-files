use assert_fs::fixture::{FileWriteStr, PathChild};
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
    let folder = assert_fs::TempDir::new().expect("temp dir to be created");
    let file = folder.child("file.txt");
    file.write_str("all good 👌!")
        .expect("bytes to be written on file");

    // pick remote working directory
    let random_folder = Uuid::new_v4().to_string();
    let segments = ["test", "rf", random_folder.as_str()];
    let url = common::Url::create_dir_url(segments.as_ref());
    let dest = url.path();

    // prepare remote file urls
    let filepath = url.end_with_file(
        file.file_name()
            .and_then(|f| f.to_str())
            .expect("file to have a name"),
    );
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
                client
                    .upload(&file.to_string_lossy(), dest, None)
                    .await
                    .unwrap();

                // download asset and assert
                let bytes = client
                    .download(files.get(0).expect("file to be uploaded").as_str())
                    .await
                    .unwrap();
                assert_eq!(Ok("all good 👌!"), String::from_utf8(bytes).as_deref());
            })
            .collect::<Vec<_>>(),
    )
    .await;
}
