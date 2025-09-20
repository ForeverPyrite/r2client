use crate::_async::R2Client;
use crate::R2Error;

#[derive(Debug)]
pub struct R2Bucket {
    bucket: String,
    pub client: R2Client,
}

impl R2Bucket {
    pub fn new(bucket: String) -> Self {
        Self {
            bucket,
            client: R2Client::new(),
        }
    }

    pub fn from_client(bucket: String, client: R2Client) -> Self {
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

    pub async fn upload_file(
        &self,
        local_file_path: &str,
        r2_file_key: &str,
    ) -> Result<(), R2Error> {
        self.client
            // I'm pasing None to let the R2Client derive the content type from the local_file_path
            .upload_file(&self.bucket, local_file_path, r2_file_key, None)
            .await
    }

    pub async fn download_file(&self, r2_file_key: &str, local_path: &str) -> Result<(), R2Error> {
        self.client
            .download_file(&self.bucket, r2_file_key, local_path, None)
            .await
    }

    pub async fn list_files(
        &self,
    ) -> Result<std::collections::HashMap<String, Vec<String>>, R2Error> {
        self.client.list_files(&self.bucket).await
    }

    pub async fn list_folders(&self) -> Result<Vec<String>, R2Error> {
        self.client.list_folders(&self.bucket).await
    }

    pub async fn delete_file(&self, r2_file_key: &str) -> Result<(), R2Error> {
        self.client.delete(&self.bucket, r2_file_key).await
    }
}
