use crate::R2Error;
use crate::sync::R2Client;

#[derive(Clone, Debug)]
pub struct R2Bucket {
    bucket: String,
    pub client: R2Client,
}

impl R2Bucket {
    pub fn new(bucket: String, client: R2Client) -> Self {
        Self { bucket, client }
    }

    pub fn from_credentials(
        bucket: String,
        access_key: String,
        secret_key: String,
        endpoint: String,
    ) -> Self {
        let client = R2Client::from_credentials(access_key, secret_key, endpoint);
        Self { bucket, client }
    }

    pub fn upload_file(&self, local_file_path: &str, r2_file_key: &str) -> Result<(), R2Error> {
        self.client
            .upload_file(&self.bucket, local_file_path, r2_file_key)
    }

    pub fn download_file(&self, r2_file_key: &str, local_path: &str) -> Result<(), R2Error> {
        self.client
            .download_file(&self.bucket, r2_file_key, local_path)
    }

    pub fn list_files(&self) -> Result<std::collections::HashMap<String, Vec<String>>, R2Error> {
        self.client.list_files(&self.bucket)
    }

    pub fn list_folders(&self) -> Result<Vec<String>, R2Error> {
        self.client.list_folders(&self.bucket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::R2Bucket;
    use std::env;

    fn get_test_bucket() -> R2Bucket {
        dotenv::dotenv().ok();
        let access_key =
            env::var("R2_ACCESS_KEY").unwrap_or_else(|_| "test_access_key".to_string());
        let secret_key =
            env::var("R2_SECRET_KEY").unwrap_or_else(|_| "test_secret_key".to_string());
        let endpoint = env::var("R2_ENDPOINT")
            .unwrap_or_else(|_| "https://example.r2.cloudflarestorage.com".to_string());
        let client = R2Client::from_credentials(access_key, secret_key, endpoint);
        R2Bucket::new("test-bucket".to_string(), client)
    }

    #[test]
    fn test_bucket_construction() {
        let bucket = get_test_bucket();
        assert_eq!(bucket.bucket, "test-bucket");
    }
}
