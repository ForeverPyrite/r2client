use crate::R2Error;
use crate::mimetypes::get_mimetype_from_fp;
use aws_sigv4::SigV4Credentials;
use http::Method;
use log::trace;
use reqwest::header::HeaderMap;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug)]
pub struct R2Client {
    sigv4: SigV4Credentials,
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
            sigv4: SigV4Credentials::new("s3", "auto", access_key, secret_key),
            endpoint,
        }
    }

    pub fn from_credentials(access_key: String, secret_key: String, endpoint: String) -> Self {
        Self {
            sigv4: SigV4Credentials::new("s3", "auto", access_key, secret_key),
            endpoint,
        }
    }

    fn create_headers(
        &self,
        method: http::Method,
        bucket: &str,
        key: Option<&str>,
        payload: impl AsRef<[u8]>,
        content_type: Option<&str>,
        extra_headers: Option<Vec<(String, String)>>,
    ) -> Result<HeaderMap, R2Error> {
        let uri = http::Uri::from_str(&self.build_url(bucket, key))
            .expect("invalid uri rip (make sure the build_url function works as intended)");
        let mut headers = extra_headers.unwrap_or_default();
        headers.push((
            "host".to_string(),
            uri.host().expect("Should have host in URI").to_owned(),
        ));
        if let Some(content_type) = content_type {
            headers.push(("content-type".to_string(), content_type.to_owned()))
        }

        let (_, header_map) = self.sigv4.signature(method, uri, headers, payload);
        Ok(header_map)
    }

    pub fn upload_file(
        &self,
        bucket: &str,
        local_file_path: &str,
        r2_file_key: &str,
        content_type: Option<&str>,
    ) -> Result<(), R2Error> {
        // Payload (file data)
        let payload = std::fs::read(local_file_path)?;
        trace!(
            "[upload_file] Payload hash for signing: {}",
            aws_sigv4::hash(&payload)
        );

        // Set HTTP Headers
        let content_type = if let Some(content_type) = content_type {
            Some(content_type)
        } else {
            Some(get_mimetype_from_fp(local_file_path))
        };
        let headers = self.create_headers(
            Method::PUT,
            bucket,
            Some(r2_file_key),
            &payload,
            content_type,
            None,
        )?;
        trace!("[upload_file] Headers sent to request: {headers:#?}");
        let file_url = self.build_url(bucket, Some(r2_file_key));
        let client = reqwest::blocking::Client::new();
        let resp = client
            .put(&file_url)
            .headers(headers)
            .body(payload)
            .send()?;
        let status = resp.status();
        let text = resp.text()?;
        if status.is_success() {
            Ok(())
        } else {
            Err(R2Error::FailedRequest(
                format!(
                    "upload file {local_file_path} to bucket \"{bucket}\" under file key \"{r2_file_key}\""
                ),
                status,
                text,
            ))
        }
    }
    pub fn download_file(
        &self,
        bucket: &str,
        key: &str,
        local_path: &str,
        extra_headers: Option<Vec<(String, String)>>,
    ) -> Result<(), R2Error> {
        // https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_sigv-create-signed-request.html#:~:text=For%20Amazon%20S3%2C%20include%20the%20literal%20string%20UNSIGNED%2DPAYLOAD%20when%20constructing%20a%20canonical%20request%2C%20and%20set%20the%20same%20value%20as%20the%20x%2Damz%2Dcontent%2Dsha256%20header%20value%20when%20sending%20the%20request.
        // I don't know if I should trust it though, I don't see public impls with this.
        let payload = "";
        trace!("[download_file] Payload for signing: (empty)");
        let headers =
            self.create_headers(Method::GET, bucket, Some(key), payload, None, extra_headers)?;
        trace!("[download_file] Headers sent to request: {headers:#?}");
        let file_url = self.build_url(bucket, Some(key));
        let client = reqwest::blocking::Client::new();
        let resp = client.get(&file_url).headers(headers).send()?;
        let status = resp.status();
        if status.is_success() {
            std::fs::write(local_path, resp.bytes()?)?;
            Ok(())
        } else {
            Err(R2Error::FailedRequest(
                format!("dowloading file \"{key}\" from bucket \"{bucket}\""),
                status,
                resp.text()?,
            ))
        }
    }
    pub fn delete(&self, bucket: &str, remote_key: &str) -> Result<(), R2Error> {
        let payload = "";
        trace!("[delete_file] Payload for signing: (empty)");
        let headers = self.create_headers(
            Method::DELETE,
            bucket,
            Some(remote_key),
            payload,
            None,
            None,
        )?;
        trace!("[delete_file] Headers sent to request: {headers:#?}");
        let file_url = self.build_url(bucket, Some(remote_key));
        let client = reqwest::blocking::Client::new();
        let resp = client.delete(&file_url).headers(headers).send()?;
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(R2Error::FailedRequest(
                format!("deleting file \"{remote_key}\" from bucket \"{bucket}\""),
                status,
                resp.text()?,
            ))
        }
    }
    fn get_bucket_listing(&self, bucket: &str) -> Result<String, R2Error> {
        let payload = "";
        trace!("[get_bucket_listing] Payload for signing: (empty)");
        let headers = self.create_headers(Method::GET, bucket, None, payload, None, None)?;
        trace!("[get_bucket_listing] Headers sent to request: {headers:#?}");
        let url = self.build_url(bucket, None);
        let client = reqwest::blocking::Client::new();
        let resp = client
            .get(&url)
            .headers(headers)
            .send()
            .map_err(R2Error::from)?;
        let status = resp.status();
        if status.is_success() {
            Ok(resp.text().map_err(R2Error::from)?)
        } else {
            Err(R2Error::FailedRequest(
                String::from("list bucket...folders or something idfk"),
                status,
                resp.text().map_err(R2Error::from)?,
            ))
        }
    }

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
            if let Some(file_key) = key_elem
                && let Some(idx) = file_key.find('/')
            {
                folders.insert(file_key[..idx].to_string());
            }
        }
        Ok(folders.into_iter().collect())
    }

    fn build_url(&self, bucket: &str, key: Option<&str>) -> String {
        match key {
            Some(k) => {
                let encoded_key = aws_sigv4::url_encode(k);
                format!("{}/{}/{}", self.endpoint, bucket, encoded_key)
            }
            None => format!("{}/{}/", self.endpoint, bucket),
        }
    }
}
impl Default for R2Client {
    fn default() -> Self {
        Self::new()
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

        // Sorry but I don't know if I should have the keys on the sigv4 pub or not yet
        // assert_eq!(r2client.access_key, "AKIAEXAMPLE");
        // assert_eq!(
        //     r2client.secret_key,
        //     "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY"
        // );
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
                None,
            )
            .unwrap();
        assert!(headers.contains_key("x-amz-date"));
        assert!(headers.contains_key("authorization"));
        assert!(headers.contains_key("content-type"));
        assert!(headers.contains_key("host"));
    }
}
