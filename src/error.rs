use thiserror::Error;

#[derive(Error, Debug)]
pub enum R2Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("XML parse error: {0}")]
    Xml(#[from] xmltree::ParseError),
    #[error("Missing environment varibles: {0}")]
    Env(String),
    #[error("Other: {0}")]
    Other(String),
}
