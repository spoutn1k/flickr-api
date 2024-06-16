use image::io::Reader;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::error::Error;
use std::io::Cursor;
use warp::hyper::body::Bytes;

mod oauth;
pub use oauth::{ApiKey, Token as OauthToken};

mod get_info;
mod get_sizes;
mod get_token;
mod test_login;
mod upload_photo;

static URL_ACCESS: &str = "https://www.flickr.com/services/oauth/access_token";
static URL_AUTHORIZE: &str = "https://www.flickr.com/services/oauth/authorize";
static URL_REQUEST: &str = "https://www.flickr.com/services/oauth/request_token";

static URL_API: &str = "https://api.flickr.com/services/rest/";

static URL_UPLOAD: &str = "https://up.flickr.com/services/upload/";

/// We use a single HTTP client as recreating it leads to some connection issues
static mut CLIENT: Option<reqwest::Client> = None;

pub use get_info::photos_getinfo;
pub use get_sizes::{photos_getsizes, FlickrSize};
pub use get_token::get_token;
pub use test_login::{test_login, UserData};
pub use upload_photo::upload_photo_path;

fn get_client() -> &'static reqwest::Client {
    unsafe {
        if CLIENT.is_none() {
            CLIENT = Some(reqwest::Client::new())
        }

        CLIENT.as_ref().unwrap()
    }
}

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

trait Resultable<T> {
    fn to_result(self) -> Result<T, String>;
}

/// Common error type for all flickr API answers
#[derive(Deserialize, Debug, Hash)]
pub struct FlickrError {
    pub stat: String,
    pub code: u32,
    pub message: String,
}

use std::fmt::Display;
impl Display for FlickrError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(formatter, "{} (code {})", self.message, self.code)
    }
}

/// Convenience function to download an image using the library's client
pub async fn download_image(url: &String) -> Result<Reader<Cursor<Bytes>>, Box<dyn Error>> {
    let res = get_client().get(url).send().await?;

    Ok(Reader::new(Cursor::new(res.bytes().await?)))
}
