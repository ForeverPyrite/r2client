use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

type Hmac256 = Hmac<Sha256>;

const EMPTY_PAYLOAD_HASH: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

// --- Utility functions ---
fn lowercase(string: &str) -> String {
    string.to_lowercase()
}

fn hex<T: AsRef<[u8]>>(data: T) -> String {
    hex::encode(data)
}

fn sha256hash<T: AsRef<[u8]>>(data: T) -> [u8; 32] {
    Sha256::digest(data).into()
}

fn hmac_sha256(signing_key: &[u8], message: &str) -> Vec<u8> {
    let mut mac = Hmac256::new_from_slice(signing_key).expect("bad key :pensive:");
    mac.update(message.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn trim(string: &str) -> String {
    string.trim().to_string()
}

pub fn hash<T: AsRef<[u8]>>(payload: T) -> String {
    hex(sha256hash(payload))
}

pub fn url_encode(url: &str) -> String {
    let mut url = urlencoding::encode(url).into_owned();
    let encoded_to_replacement: [(&str, &str); 4] =
        [("+", "%20"), ("*", "%2A"), ("%7E", "~"), ("%2F", "/")];
    for (encoded_chars_pattern, replacement) in encoded_to_replacement {
        url = url.replace(encoded_chars_pattern, replacement)
    }
    url
}

// --- Signing Functions ---
// These don't use any parts of the Sigv4Client, so they are external

// --- Canonical request ---
fn create_canonical_request(
    method: http::Method,
    uri: http::Uri,
    mut headers: Vec<(String, String)>,
    hashed_payload: &str,
) -> (String, Vec<(String, String)>, String) {
    // HTTPMethod
    let http_method = method.to_string();

    // CanonicalURI = *path only* (spec forbids scheme+host here)
    let canonical_uri = if uri.path().is_empty() {
        "/".to_string()
    } else {
        uri.path().to_string()
    };

    // CanonicalQueryString (URL-encoded, sorted by key)
    let canonical_query_string = if let Some(query_string) = uri.query() {
        let mut pairs = query_string
            .split('&')
            .map(|query| {
                let (k, v) = query.split_once('=').unwrap_or((query, ""));
                (url_encode(k), url_encode(v))
            })
            .collect::<Vec<_>>();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        pairs
            .into_iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("&")
    } else {
        String::new()
    };

    // checks for proper host headers
    let host = uri
        .host()
        .expect("uri passed without a proper host")
        .to_string();
    if !headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("host")) {
        headers.push(("host".to_string(), host));
    }

    if !headers
        .iter()
        .any(|(k, _)| k.eq_ignore_ascii_case("x-amz-content-sha256"))
    {
        headers.push((
            "x-amz-content-sha256".to_string(),
            hashed_payload.to_owned(),
        ))
    }

    // CanonicalHeaders + SignedHeaders
    let mut http_headers = headers
        .iter()
        .map(|(name, value)| (lowercase(name), trim(value)))
        .collect::<Vec<_>>();
    http_headers.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

    let canonical_headers: String = http_headers
        .iter()
        .map(|(k, v)| format!("{k}:{v}\n"))
        .collect();

    let signed_headers: String = http_headers
        .iter()
        .map(|(k, _)| k.clone())
        .collect::<Vec<_>>()
        .join(";");

    // Final canonical request
    let canonical_request = format!(
        "{http_method}\n{canonical_uri}\n{canonical_query_string}\n{canonical_headers}\n{signed_headers}\n{hashed_payload}"
    );

    (canonical_request, http_headers, signed_headers)
}
fn calculate_signature(signing_key: &[u8], string_to_sign: &str) -> Vec<u8> {
    hmac_sha256(signing_key, string_to_sign)
}

fn string_to_sign(scope: &str, amz_date: &str, hashed_canonical_request: &str) -> String {
    format!(
        "{}\n{}\n{}\n{}",
        "AWS4-HMAC-SHA256", amz_date, scope, hashed_canonical_request
    )
}
/// Structure containing all the data relevant for an AWS Service utilizing SigV4.
/// Service: String containing the AWS service (e.g. "ec2" or "s3")
/// Region: String containing the AWS region you're working in (e.g. "auto" or "us-east-1")
/// Access Key: The "Access Key" to use with the AWS service (crazy, ik)
/// Secret Key: The "Secret Key" that is used for cryptographic signing for the AWS Service (woah)
///
/// ```
///
///
/// use aws_signing::Sigv4Client;
///
/// let s3_client = Sigv4Client::new(
///     "s3",
///     "us-east-1",
///     std::env::var("S3_ACCESS_KEY").unwrap(),
///     std::env::var("S3_SECRET_KEY").unwrap(),
/// )
/// let (_, request_headers) = s3client.signature(
///     http::Method::GET,
///     http::Uri::from_static("https://s3.us-east-1.amazonaws.com/example-bucket/file.txt"),
///     vec![("content-type", "text/plain")],
///     "" // Since it's a GET request, the payload is ""
///     )
/// ```
#[derive(Debug)]
// A more mature client would also have session_key: Option<String>, but not my problem
pub struct Sigv4Client {
    // Would it makes more sense for these to be type generics
    // with trait param ToString?
    // Either that or just &str or String...wait, union?
    // Nah there has to be a better way to do it than...that
    // but I don't wanna enum!!!
    service: String,
    region: String,
    // Would it makes more sense for these to be type generics
    // with trait param AsRef<[u8]>?
    access_key: String,
    secret_key: String,
}
/// NOTE: This only impliments functions that require one of the Sigv4Client fields.
/// For other functions related to the signing proccess, they are defined above, including the
/// prequisite functions defined at https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_sigv-create-signed-request.html
impl Sigv4Client {
    /// Creates a new instance of the Sigv4Client for a particular service, in a region, with your
    /// private and public access keys.
    ///
    /// For some reason this function will take any values that impl Into<String>, so you can pass
    /// &str, String, or something else if you decide to get freaky.
    pub fn new(
        service: impl Into<String>,
        region: impl Into<String>,
        pub_key: impl Into<String>,
        priv_key: impl Into<String>,
    ) -> Self {
        Self {
            service: service.into(),
            region: region.into(),
            access_key: pub_key.into(),
            secret_key: priv_key.into(),
        }
    }

    // In a more mature client, this might be an enum of AWSRegions
    // I also don't even know if this could ever be useful lol, wouldn't you have individual
    // clients for each region or use "auto" for AWS to figure it out for you? whatever.
    pub fn set_region(&mut self, region: impl Into<String>) {
        self.region = region.into()
    }

    fn credential_scope(&self, date: &str) -> String {
        format!(
            "{}/{}/{}/aws4_request",
            date,
            lowercase(&self.region),
            lowercase(&self.service)
        )
    }

    fn derive_signing_key(&self, date: &str) -> Vec<u8> {
        let secret_key = &self.secret_key;
        let key = format!("AWS4{secret_key}");
        let date_key = hmac_sha256(key.as_bytes(), date);
        let date_region_key = hmac_sha256(&date_key, &self.region);
        let date_region_service_key = hmac_sha256(&date_region_key, &self.service);
        hmac_sha256(&date_region_service_key, "aws4_request")
    }

    // --- API ---
    /// This is the only function to use <3
    pub fn signature<T: AsRef<[u8]>>(
        &self,
        method: http::Method,
        uri: http::Uri,
        // Should probably make this a header map, then turn it into a Vec(String, String) to sort
        // by header name cause Amazon said so.
        mut headers: Vec<(String, String)>,
        payload: T,
    ) -> (String, http::HeaderMap) {
        let auth_algorithm = "AWS4-HMAC-SHA256";
        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date = now.format("%Y%m%d").to_string();
        let payload_as_bytes = payload.as_ref();
        let payload_hash = if payload_as_bytes.is_empty() {
            EMPTY_PAYLOAD_HASH.to_string()
        } else {
            hash(payload_as_bytes)
        };

        // Add x-amz-date header if not already present
        if !headers
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("x-amz-date"))
        {
            headers.push(("x-amz-date".to_string(), amz_date.clone()));
        }

        // Canonical request
        let (canonical_request, mut headers, signed_headers) =
            create_canonical_request(method, uri, headers, &payload_hash);

        // String to sign
        let scope = self.credential_scope(&date);
        let hashed_canonical_request = hash(&canonical_request);
        let string_to_sign = string_to_sign(&scope, &amz_date, &hashed_canonical_request);

        // Signing key + signature
        let signing_key = self.derive_signing_key(&date);
        let signature = hex(calculate_signature(&signing_key, &string_to_sign));

        // Authorization header
        let access_key = &self.access_key;
        let credential = format!("{access_key}/{scope}");
        let auth_header = format!(
            "{auth_algorithm} Credential={credential}, SignedHeaders={signed_headers}, Signature={signature}"
        );

        println!("\n--- AWS SigV4 Debug ---");
        println!("1. CanonicalRequest:\n---\n{canonical_request}\n---");
        println!("2. StringToSign:\n---\n{string_to_sign}\n---");
        println!("3. SigningKey:\n---\n{}\n---", hex(&signing_key));
        println!("4. Signature:\n---\n{signature}\n---");
        println!("5. Authorization Header:\n---\n{auth_header}\n---");

        headers.push(("authorization".to_string(), auth_header));

        let mut header_map: http::HeaderMap = http::HeaderMap::new();
        for (header, value) in headers.clone() {
            header_map.insert(
                http::HeaderName::from_lowercase(header.to_lowercase().as_bytes()).unwrap(),
                http::HeaderValue::from_str(&value).unwrap(),
            );
        }
        (signature, header_map)
    }
}
