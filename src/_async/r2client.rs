use crate::mimetypes::Mime;
use crate::{R2Error, aws_signing};
use http::Method;
use reqwest::header::{self, HeaderName, HeaderValue};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct R2Client {
    access_key: String,
    secret_key: String,
    endpoint: String,
}

impl R2Client {
    fn get_env() -> Result<(String, String, String), R2Error> {
        let keys = ["R2_ACCESS_KEY", "R2_SECRET_KEY", "R2_ENDPOINT"];
        let values = keys
            .map(|key| { std::env::var(key).map_err(|_| R2Error::Env(key.to_owned())) }.unwrap());
        Ok(values.into())
    }

    pub fn new() -> Self {
        let (access_key, secret_key, endpoint) = Self::get_env().unwrap();

        Self {
            access_key,
            secret_key,
            endpoint,
        }
    }

    pub fn from_credentials(access_key: String, secret_key: String, endpoint: String) -> Self {
        Self {
            access_key,
            secret_key,
            endpoint,
        }
    }

    fn create_headers(
        &self,
        method: http::Method,
        bucket: &str,
        key: Option<&str>,
        payload_hash: &str,
        content_type: Option<&str>,
    ) -> Result<header::HeaderMap, R2Error> {
        let uri = http::Uri::from_str(&self.build_url(bucket, key))
            .expect("invalid uri rip (make sure the build_url function works as intended)");
        let mut headers = Vec::new();
        if method == Method::GET {
            headers.push((
                "x-amz-content-sha256".to_string(),
                "UNSIGNED-PAYLOAD".to_string(),
            ))
        }
        if let Some(content_type) = content_type {
            headers.push(("content-type".to_string(), content_type.to_owned()))
        }

        let (_, headers) = aws_signing::signature(
            method,
            uri,
            headers,
            payload_hash,
            "s3",
            "us-east-1",
            &self.secret_key,
            &self.access_key,
        );
        let mut header_map = header::HeaderMap::new();
        for header in headers {
            header_map.insert(
                HeaderName::from_lowercase(&header.0.to_lowercase().as_bytes())
                    .expect("shit tragic"),
                HeaderValue::from_str(&header.1).expect("shit more tragic"),
            );
        }
        Ok(header_map)
    }

    pub async fn upload_file(
        &self,
        bucket: &str,
        local_file_path: &str,
        r2_file_key: &str,
        content_type: Option<&str>,
    ) -> Result<(), R2Error> {
        // --- Hash Payload --
        let file_data = std::fs::read(local_file_path)?;
        let payload_hash = hex::encode(Sha256::digest(&file_data));

        // Set HTTP Headers
        let content_type = if let Some(content_type) = content_type {
            Some(content_type)
        } else {
            Some(Mime::get_mimetype_from_fp(local_file_path))
        };
        let headers = self.create_headers(
            Method::PUT,
            bucket,
            Some(r2_file_key),
            &payload_hash,
            content_type,
        )?;
        let file_url = format!("{}/{}/{}", self.endpoint, bucket, r2_file_key);
        let client = reqwest::Client::new();
        let resp = client
            .put(&file_url)
            .headers(headers)
            .body(file_data)
            .send()
            .await?;
        let status = resp.status();
        let text = resp.text().await?;
        if status.is_success() {
            Ok(())
        } else {
            Err(R2Error::Other(format!(
                "Upload failed with status {}: {}",
                status, text
            )))
        }
    }
    pub async fn download_file(
        &self,
        bucket: &str,
        key: &str,
        local_path: &str,
    ) -> Result<(), R2Error> {
        let payload_hash = hex::encode(Sha256::digest(""));
        let content_type = Mime::get_mimetype_from_fp(local_path);
        let headers = self.create_headers(
            Method::GET,
            bucket,
            Some(key),
            &payload_hash,
            Some(content_type),
        )?;
        let file_url = format!("{}/{}/{}", self.endpoint, bucket, key);
        let client = reqwest::Client::new();
        let resp = client.get(&file_url).headers(headers).send().await?;
        let status = resp.status();
        let content = resp.bytes().await?;
        if status.is_success() {
            std::fs::write(local_path, &content)?;
            Ok(())
        } else {
            Err(R2Error::Other(format!(
                "Download failed with status {}",
                status
            )))
        }
    }
    async fn get_bucket_listing(&self, bucket: &str) -> Result<String, R2Error> {
        let payload_hash = "UNSIGNED-PAYLOAD";
        let content_type = "application/xml";
        let headers =
            self.create_headers(Method::GET, bucket, None, payload_hash, Some(content_type))?;
        let url = self.build_url(bucket, None);
        let client = reqwest::Client::new();
        let resp = client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(R2Error::from)?;
        let status = resp.status();
        if status.is_success() {
            resp.text().await.map_err(R2Error::from)
        } else {
            Err(R2Error::Other(format!("Failed to list bucket: {}", status)))
        }
    }

    pub async fn list_files(&self, bucket: &str) -> Result<HashMap<String, Vec<String>>, R2Error> {
        let xml = self.get_bucket_listing(bucket).await?;
        let mut files_dict: HashMap<String, Vec<String>> = HashMap::new();
        let root = xmltree::Element::parse(xml.as_bytes()).map_err(R2Error::from)?;
        for content in root
            .children
            .iter()
            .filter_map(|c| c.as_element())
            .filter(|e| e.name == "Contents")
        {
            let key_elem = content.get_child("Key").and_then(|k| k.get_text());
            if let Some(file_key) = key_elem {
                let (folder, file_name): (String, String) = if let Some(idx) = file_key.rfind('/') {
                    (file_key[..idx].to_string(), file_key[idx + 1..].to_string())
                } else {
                    ("".to_string(), file_key.to_string())
                };
                files_dict.entry(folder).or_default().push(file_name);
            }
        }
        Ok(files_dict)
    }

    pub async fn list_folders(&self, bucket: &str) -> Result<Vec<String>, R2Error> {
        let xml = self.get_bucket_listing(bucket).await?;
        let mut folders = std::collections::HashSet::new();
        let root = xmltree::Element::parse(xml.as_bytes()).map_err(R2Error::from)?;
        for content in root
            .children
            .iter()
            .filter_map(|c| c.as_element())
            .filter(|e| e.name == "Contents")
        {
            let key_elem = content.get_child("Key").and_then(|k| k.get_text());
            if let Some(file_key) = key_elem {
                if let Some(idx) = file_key.find('/') {
                    folders.insert(file_key[..idx].to_string());
                }
            }
        }
        Ok(folders.into_iter().collect())
    }

    fn build_url(&self, bucket: &str, key: Option<&str>) -> String {
        match key {
            Some(k) => format!("{}/{}/{}", self.endpoint, bucket, k),
            None => format!("{}/{}/", self.endpoint, bucket),
        }
    }
}
impl Default for R2Client {
    fn default() -> Self {
        let (access_key, secret_key, endpoint) = Self::get_env().unwrap();

        Self {
            access_key,
            secret_key,
            endpoint,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r2client_from_env() -> R2Client {
        unsafe {
            std::env::set_var("R2_ACCESS_KEY", "AKIAEXAMPLE");
            std::env::set_var("R2_SECRET_KEY", "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY");
            std::env::set_var("R2_ENDPOINT", "https://example.r2.cloudflarestorage.com");
        }
        R2Client::new()
    }

    #[test]
    fn r2client_env() {
        let r2client = r2client_from_env();

        assert_eq!(r2client.access_key, "AKIAEXAMPLE");
        assert_eq!(
            r2client.secret_key,
            "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY"
        );
        assert_eq!(
            r2client.endpoint,
            "https://example.r2.cloudflarestorage.com"
        );
    }

    #[test]
    fn test_create_headers() {
        let client = R2Client::from_credentials(
            "AKIAEXAMPLE".to_string(),
            "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY".to_string(),
            "https://example.r2.cloudflarestorage.com".to_string(),
        );
        let headers = client
            .create_headers(
                Method::PUT,
                "bucket",
                Some("key"),
                "deadbeef",
                Some("application/octet-stream"),
            )
            .unwrap();
        assert!(headers.contains_key("x-amz-date"));
        assert!(headers.contains_key("authorization"));
        assert!(headers.contains_key("content-type"));
        assert!(headers.contains_key("host"));
    }
}
