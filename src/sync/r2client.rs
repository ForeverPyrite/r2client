use crate::R2Error;
use crate::mimetypes::Mime;
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

type HmacSHA256 = Hmac<Sha256>;

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

    fn sign(&self, key: &[u8], msg: &str) -> Vec<u8> {
        let mut mac = HmacSHA256::new_from_slice(key).expect("Invalid key length");
        mac.update(msg.as_bytes());
        mac.finalize().into_bytes().to_vec()
    }

    fn get_signature_key(&self, date_stamp: &str, region: &str, service: &str) -> Vec<u8> {
        let aws4_secret: String = format!("AWS4{}", self.secret_key);
        let k_date = self.sign(aws4_secret.as_bytes(), date_stamp);
        let k_region = self.sign(&k_date, region);
        let k_service = self.sign(&k_region, service);
        self.sign(&k_service, "aws4_request")
    }

    fn create_headers(
        &self,
        method: &str,
        bucket: &str,
        key: &str,
        payload_hash: &str,
        content_type: &str,
    ) -> Result<header::HeaderMap, R2Error> {
        // Robustly extract host from endpoint
        let endpoint = self.endpoint.trim_end_matches('/');
        // Not proud of this, it is really dumb and hard to read, but it'll work I suppose...I think...
        let host = endpoint
            .split("//")
            .nth(1)
            .unwrap_or(endpoint)
            .split('/')
            .next()
            .unwrap_or(endpoint)
            .split(':')
            .next()
            .unwrap_or(endpoint)
            .trim();
        if host.is_empty() {
            return Err(R2Error::Other(
                "Host header could not be determined from endpoint".to_string(),
            ));
        }
        let t = Utc::now();
        let amz_date = t.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = t.format("%Y%m%d").to_string();

        let mut headers_vec = [
            ("host", host),
            ("x-amz-date", &amz_date),
            ("x-amz-content-sha256", payload_hash),
            ("content-type", content_type),
        ];
        headers_vec.sort_by(|a, b| a.0.cmp(b.0));

        let signed_headers = headers_vec
            .iter()
            .map(|(k, _)| *k)
            .collect::<Vec<_>>()
            .join(";");
        let canonical_headers = headers_vec
            .iter()
            .map(|(k, v)| format!("{}:{}\n", k.to_lowercase(), v))
            .collect::<String>();

        let canonical_uri = format!("/{}/{}", bucket, key);
        let canonical_request = format!(
            "{method}\n{uri}\n\n{headers}\n{signed_headers}\n{payload_hash}",
            method = method,
            uri = canonical_uri,
            headers = canonical_headers,
            signed_headers = signed_headers,
            payload_hash = payload_hash
        );
        let credential_scope = format!("{}/{}/s3/aws4_request", date_stamp, "auto");
        let hashed_request = hex::encode(Sha256::digest(canonical_request.as_bytes()));
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{hashed_request}",
            amz_date = amz_date,
            credential_scope = credential_scope,
            hashed_request = hashed_request
        );
        let signing_key = self.get_signature_key(&date_stamp, "auto", "s3");
        let signature = hex::encode(self.sign(&signing_key, &string_to_sign));
        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
            self.access_key, credential_scope, signed_headers, signature
        );

        // Print all headers for debugging
        println!("[r2client] DEBUG: Built headers:");
        println!("  host: {}", host);
        println!("  x-amz-date: {}", amz_date);
        println!("  x-amz-content-sha256: {}", payload_hash);
        println!("  content-type: {}", content_type);
        println!("  authorization: {}", authorization);
        println!("  signed_headers: {}", signed_headers);
        println!(
            "  canonical_headers: {}",
            canonical_headers.replace("\n", "\\n")
        );
        println!(
            "  canonical_request: {}",
            canonical_request.replace("\n", "\\n")
        );
        println!("  string_to_sign: {}", string_to_sign.replace("\n", "\\n"));
        println!("  signature: {}", signature);

        let mut header_map = HeaderMap::new();
        header_map.insert(
            HeaderName::from_static("x-amz-date"),
            HeaderValue::from_str(&amz_date)
                .map_err(|e| R2Error::Other(format!("Invalid x-amz-date: {e}")))?,
        );
        header_map.insert(
            HeaderName::from_static("x-amz-content-sha256"),
            HeaderValue::from_str(payload_hash).map_err(|e| {
                R2Error::Other(format!(
                    "Invalid x-amz-content-sha256: {payload_hash:?}: {e}"
                ))
            })?,
        );
        header_map.insert(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(&authorization).map_err(|e| {
                R2Error::Other(format!(
                    "Invalid authorization: {e}\nValue: {authorization}"
                ))
            })?,
        );
        header_map.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_str(content_type)
                .map_err(|e| R2Error::Other(format!("Invalid content-type: {e}")))?,
        );
        header_map.insert(
            HeaderName::from_static("host"),
            HeaderValue::from_str(host)
                .map_err(|e| R2Error::Other(format!("Invalid host: {e}")))?,
        );
        Ok(header_map)
    }

    pub fn upload_file(
        &self,
        bucket: &str,
        local_file_path: &str,
        r2_file_key: &str,
    ) -> Result<(), R2Error> {
        let file_data = std::fs::read(local_file_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&file_data);
        let payload_hash = hex::encode(hasher.finalize());
        // let content_type = "application/octet-stream";
        let content_type = Mime::get_mimetype_from_fp(local_file_path);
        let headers =
            self.create_headers("PUT", bucket, r2_file_key, &payload_hash, content_type)?;
        let file_url = format!("{}/{}/{}", self.endpoint, bucket, r2_file_key);
        let client = reqwest::blocking::Client::new();
        let resp = client
            .put(&file_url)
            .headers(headers)
            .body(file_data)
            .send()?;
        let status = resp.status();
        let text = resp.text()?;
        if status.is_success() {
            Ok(())
        } else {
            Err(R2Error::Other(format!(
                "Upload failed with status {}: {}",
                status, text
            )))
        }
    }
    pub fn download_file(&self, bucket: &str, key: &str, local_path: &str) -> Result<(), R2Error> {
        let payload_hash = "UNSIGNED-PAYLOAD";
        let content_type = "application/octet-stream";
        let headers = self.create_headers("GET", bucket, key, payload_hash, content_type)?;
        let file_url = format!("{}/{}/{}", self.endpoint, bucket, key);
        let client = reqwest::blocking::Client::new();
        let resp = client.get(&file_url).headers(headers).send()?;
        let status = resp.status();
        let content = resp.bytes()?;
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
    fn get_bucket_listing(&self, bucket: &str) -> Result<String, R2Error> {
        let payload_hash = "UNSIGNED-PAYLOAD";
        let content_type = "application/xml";
        let headers = self.create_headers("GET", bucket, "", payload_hash, content_type)?;
        let url = self.build_url(bucket, None);
        let client = reqwest::blocking::Client::new();
        let resp = client
            .get(&url)
            .headers(headers)
            .send()
            .map_err(R2Error::from)?;
        let status = resp.status();
        if status.is_success() {
            resp.text().map_err(R2Error::from)
        } else {
            Err(R2Error::Other(format!("Failed to list bucket: {}", status)))
        }
    }

    /// List all files in the specified bucket. Returns a HashMap of folder -> `Vec<file>`.
    pub fn list_files(&self, bucket: &str) -> Result<HashMap<String, Vec<String>>, R2Error> {
        let xml = self.get_bucket_listing(bucket)?;
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

    /// List all folders in the specified bucket. Returns a Vec of folder names.
    pub fn list_folders(&self, bucket: &str) -> Result<Vec<String>, R2Error> {
        let xml = self.get_bucket_listing(bucket)?;
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
    fn test_sign_and_signature_key() {
        let client = R2Client::from_credentials(
            "AKIAEXAMPLE".to_string(),
            "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY".to_string(),
            "https://example.r2.cloudflarestorage.com".to_string(),
        );
        let key = b"testkey";
        let msg = "testmsg";
        let sig = client.sign(key, msg);
        assert_eq!(sig.len(), 32); // HMAC-SHA256 output is 32 bytes

        let date = "20250101";
        let region = "auto";
        let service = "s3";
        let signing_key = client.get_signature_key(date, region, service);
        assert_eq!(signing_key.len(), 32);
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
                "PUT",
                "bucket",
                "key",
                "deadbeef",
                "application/octet-stream",
            )
            .unwrap();
        assert!(headers.contains_key("x-amz-date"));
        assert!(headers.contains_key("authorization"));
        assert!(headers.contains_key("content-type"));
        assert!(headers.contains_key("host"));
    }
}
