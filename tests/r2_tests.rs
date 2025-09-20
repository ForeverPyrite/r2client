use std::fs;
use std::io::Write;

fn create_test_file(path: &str, content: &str) {
    let mut file = fs::File::create(path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
}

#[cfg(feature = "sync")]
mod sync_tests {
    use super::create_test_file;
    use r2client::sync::R2Bucket;
    use std::env;
    use std::fs;

    fn setup_bucket() -> R2Bucket {
        dotenv::dotenv().ok();
        let bucket = env::var("R2_BUCKET").expect("R2_BUCKET not set for integration tests");
        let access_key = env::var("R2_ACCESS_KEY").expect("R2_ACCESS_KEY not set");
        let secret_key = env::var("R2_SECRET_KEY").expect("R2_SECRET_KEY not set");
        let endpoint = env::var("R2_ENDPOINT").expect("R2_ENDPOINT not set");
        R2Bucket::from_credentials(bucket, access_key, secret_key, endpoint)
    }

    #[test]
    fn test_sync_e2e() {
        let bucket = setup_bucket();
        let test_content = "Hello, R2 sync world!";
        let local_upload_path = "test_upload_sync.txt";
        let r2_file_key = "test/test_upload_sync.txt";
        let local_download_path = "test_download_sync.txt";

        create_test_file(local_upload_path, test_content);

        // 1. Upload file
        bucket
            .upload_file(local_upload_path, r2_file_key)
            .expect("Sync upload failed");

        // 2. List files and check if it exists
        let files = bucket.list_files().expect("Sync list_files failed");
        assert!(
            files
                .get("test")
                .unwrap()
                .contains(&"test_upload_sync.txt".to_string())
        );

        // 3. List folders and check if it exists
        let folders = bucket.list_folders().expect("Sync list_folders failed");
        assert!(folders.contains(&"test".to_string()));

        // 4. Download file
        bucket
            .download_file(r2_file_key, local_download_path)
            .expect("Sync download failed");

        // 5. Verify content
        let downloaded_content = fs::read_to_string(local_download_path).unwrap();
        assert_eq!(test_content, downloaded_content);

        // Cleanup
        fs::remove_file(local_upload_path).unwrap();
        fs::remove_file(local_download_path).unwrap();
    }
}

#[cfg(feature = "async")]
mod async_tests {
    use super::create_test_file;
    use r2client::R2Bucket;
    use std::env;
    use std::fs;

    fn setup_bucket() -> R2Bucket {
        dotenv::dotenv().ok();
        let bucket = env::var("R2_BUCKET").expect("R2_BUCKET not set for integration tests");
        let access_key = env::var("R2_ACCESS_KEY").expect("R2_ACCESS_KEY not set");
        let secret_key = env::var("R2_SECRET_KEY").expect("R2_SECRET_KEY not set");
        let endpoint = env::var("R2_ENDPOINT").expect("R2_ENDPOINT not set");
        R2Bucket::from_credentials(bucket, access_key, secret_key, endpoint)
    }

    #[tokio::test]
    async fn test_async_e2e() {
        let bucket = setup_bucket();
        let test_content = "Hello, R2 async world!";
        let local_upload_path = "test_upload_async.txt";
        let r2_file_key = "test/test_upload_async.txt";
        let local_download_path = "test_download_async.txt";

        create_test_file(local_upload_path, test_content);

        // 0. List files to see if a get request will go through lol
        let files = bucket.list_files().await.expect("Async list_files failed");
        println!("{files:#?}");

        // 1. Upload file
        bucket
            .upload_file(local_upload_path, r2_file_key)
            .await
            .expect("Async upload failed");

        // 2. List files and check if it exists
        let files = bucket.list_files().await.expect("Async list_files failed");
        assert!(
            files
                .get("test")
                .unwrap()
                .contains(&"test_upload_async.txt".to_string())
        );

        // 3. List folders and check if it exists
        let folders = bucket
            .list_folders()
            .await
            .expect("Async list_folders failed");
        assert!(folders.contains(&"test".to_string()));

        // 4. Download file
        bucket
            .download_file(r2_file_key, local_download_path)
            .await
            .expect("Async download failed");

        // 5. Verify content
        let downloaded_content = fs::read_to_string(local_download_path).unwrap();
        assert_eq!(test_content, downloaded_content);

        // Cleanup
        fs::remove_file(local_upload_path).unwrap();
        fs::remove_file(local_download_path).unwrap();

        // 6. Delete file
        bucket.delete_file(r2_file_key).await.unwrap();
    }
}
