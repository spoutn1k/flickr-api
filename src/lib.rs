#![allow(dead_code)]
use image::io::Reader;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::error::Error;
use std::io::Cursor;
use std::rc::Rc;
use warp::hyper::body::Bytes;

mod oauth;
pub use oauth::{ApiKey, Token as OauthToken};

pub mod get_info;
pub mod get_sizes;
pub mod login;
pub mod test_login;
pub mod upload_photo;

static URL_ACCESS: &str = "https://www.flickr.com/services/oauth/access_token";
static URL_AUTHORIZE: &str = "https://www.flickr.com/services/oauth/authorize";
static URL_REQUEST: &str = "https://www.flickr.com/services/oauth/request_token";

static URL_API: &str = "https://api.flickr.com/services/rest/";

static URL_UPLOAD: &str = "https://up.flickr.com/services/upload/";

pub use get_sizes::FlickrSize;
pub use test_login::UserData;

/// This is meant to turn the abominations the XML conversion creates into easier on the eyes
/// structs:
/// ```json
/// "username": {
///   "_contents": "itsame"
/// }
/// ```
/// Becomes:
/// ```rs
/// username: String // "itsame"
/// ```
fn deserialize_content<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let v: Value = Deserialize::deserialize(deserializer)?;
    Ok(v["_content"].as_str().unwrap_or("").to_string())
}

trait Resultable<T, E> {
    fn to_result(self) -> Result<T, E>;
}

/// Common error type for all flickr API answers
#[derive(Deserialize, Debug, Hash)]
pub struct FlickrError {
    pub stat: String,
    pub code: u32,
    pub message: String,
}

impl std::error::Error for FlickrError {}

use std::fmt::Display;
impl Display for FlickrError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(formatter, "{} (code {})", self.message, self.code)
    }
}

/// Convenience function to download an image using the library's client
pub async fn download_image(url: &String) -> Result<Reader<Cursor<Bytes>>, Box<dyn Error>> {
    let res = reqwest::get(url).await?;

    Ok(Reader::new(Cursor::new(res.bytes().await?)))
}

#[derive(Clone)]
struct FlickrAPIData {
    client: reqwest::Client,
    key: ApiKey,
    token: Option<OauthToken>,
}

/// API client
pub struct FlickrAPI {
    data: Rc<FlickrAPIData>,
}

pub struct PhotoRequestBuilder {
    handle: Rc<FlickrAPIData>,
}

pub struct TestRequestBuilder {
    handle: Rc<FlickrAPIData>,
}

impl FlickrAPI {
    pub fn new(key: ApiKey) -> Self {
        let data = Rc::new(FlickrAPIData {
            client: reqwest::Client::new(),
            key,
            token: None,
        });

        FlickrAPI { data }
    }

    pub fn with_token(self, token: OauthToken) -> Self {
        let mut data = (*self.data).clone();
        data.token = Some(token);

        FlickrAPI {
            data: Rc::new(data),
        }
    }

    pub fn token(&self) -> Option<OauthToken> {
        self.data.token.clone()
    }

    pub fn photos(&self) -> PhotoRequestBuilder {
        PhotoRequestBuilder {
            handle: self.data.clone(),
        }
    }

    pub fn test(&self) -> TestRequestBuilder {
        TestRequestBuilder {
            handle: self.data.clone(),
        }
    }
}
