use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

type Hmac256 = Hmac<Sha256>;

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

fn hash<T: AsRef<[u8]>>(payload: T) -> String {
    hex(sha256hash(payload))
}

fn url_encode(url: &str) -> String {
    let mut url = urlencoding::encode(url).into_owned();
    let encoded_to_replacement: [(&str, &str); 4] =
        [("+", "%20"), ("*", "%2A"), ("%7E", "~"), ("%2F", "/")];
    for (encoded_chars_pattern, replacement) in encoded_to_replacement {
        url = url.replace(encoded_chars_pattern, replacement)
    }
    url
}

// --- Canonical request ---
fn create_canonical_request(
    method: http::Method,
    uri: http::Uri,
    mut headers: Vec<(String, String)>,
    hashed_payload: &str,
) -> (String, String, String) {
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
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&")
    } else {
        String::new()
    };

    // Ensure required headers (host and x-amz-date) are present
    let host = uri
        .host()
        .expect("uri passed without a proper host")
        .to_string();
    if !headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("host")) {
        headers.push(("host".to_string(), host));
    }

    // CanonicalHeaders + SignedHeaders
    let mut http_headers = headers
        .iter()
        .map(|(name, value)| (lowercase(name), trim(value)))
        .collect::<Vec<_>>();
    http_headers.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

    let canonical_headers: String = http_headers
        .iter()
        .map(|(k, v)| format!("{}:{}\n", k, v))
        .collect();

    let signed_headers: String = http_headers
        .iter()
        .map(|(k, _)| k.clone())
        .collect::<Vec<_>>()
        .join(";");

    // Final canonical request
    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        http_method,
        canonical_uri,
        canonical_query_string,
        canonical_headers,
        signed_headers,
        hashed_payload
    );

    (canonical_request, signed_headers, canonical_headers)
}

fn credential_scope(date: &str, region: &str, service: &str) -> String {
    format!(
        "{}/{}/{}/aws4_request",
        date,
        lowercase(region),
        lowercase(service)
    )
}

fn string_to_sign(scope: &str, amz_date: &str, canonical_request: &str) -> String {
    format!(
        "{}\n{}\n{}\n{}",
        "AWS4-HMAC-SHA256",
        amz_date,
        scope,
        hex(sha256hash(canonical_request))
    )
}

fn derive_signing_key(key: &str, date: &str, region: &str, service: &str) -> Vec<u8> {
    let secret_key = format!("AWS4{}", key);
    let date_key = hmac_sha256(secret_key.as_bytes(), date);
    let date_region_key = hmac_sha256(&date_key, region);
    let date_region_service_key = hmac_sha256(&date_region_key, service);
    hmac_sha256(&date_region_service_key, "aws4_request")
}

fn calculate_signature(signing_key: &[u8], string_to_sign: &str) -> Vec<u8> {
    hmac_sha256(signing_key, string_to_sign)
}

// --- API ---
pub fn signature(
    method: http::Method,
    uri: http::Uri,
    mut headers: Vec<(String, String)>,
    hashed_payload: &str,
    service: &str,
    region: &str,
    secret_key: &str,
    access_key: &str,
) -> (String, Vec<(String, String)>) {
    let now = Utc::now();
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let date_stamp = now.format("%Y%m%d").to_string();

    // Add x-amz-date header if not already present
    if !headers
        .iter()
        .any(|(k, _)| k.eq_ignore_ascii_case("x-amz-date"))
    {
        headers.push(("x-amz-date".to_string(), amz_date.clone()));
    }

    // Canonical request
    let (canonical_request, signed_headers, _canonical_headers) =
        create_canonical_request(method, uri, headers.clone(), hashed_payload);

    // String to sign
    let scope = credential_scope(&date_stamp, region, service);
    let string_to_sign = string_to_sign(&scope, &amz_date, &canonical_request);

    // Signing key + signature
    let signing_key = derive_signing_key(secret_key, &date_stamp, region, service);
    let signature = hex(calculate_signature(&signing_key, &string_to_sign));

    // Authorization header
    let credential = format!("{}/{}", access_key, scope);
    let auth_header = format!(
        "{} Credential={}, SignedHeaders={}, Signature={}",
        "AWS4-HMAC-SHA256", credential, signed_headers, signature
    );

    headers.push(("authorization".to_string(), auth_header));

    (signature, headers)
}
