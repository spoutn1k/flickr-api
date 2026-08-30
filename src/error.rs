use crate::FlickrError;
use thiserror::Error as ThisError;

/// Common error type returned by all fallible operations in this crate
#[derive(ThisError, Debug)]
pub enum Error {
    /// The Flickr API answered the request with an error
    #[error(transparent)]
    Api(#[from] FlickrError),

    /// The OAuth handshake was rejected
    #[error("oauth error: {0}")]
    OAuth(String),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Xml(#[from] serde_xml_rs::Error),

    #[error(transparent)]
    UrlEncoded(#[from] serde_urlencoded::de::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl From<String> for Error {
    fn from(problem: String) -> Self {
        Error::OAuth(problem)
    }
}
