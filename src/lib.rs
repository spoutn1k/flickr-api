use futures::{executor::block_on, Future};
use image::io::Reader;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fs::read;
use std::io::Cursor;
use std::process::Command;
use tokio::sync::{mpsc, oneshot};
use warp::hyper::body::Bytes;
use warp::Filter;

pub mod oauth;
use oauth::Resultable;
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

pub static mut CLIENT: Option<reqwest::Client> = None;

pub use get_info::photos_getinfo;
pub use get_sizes::photos_getsizes;
pub use get_token::get_token;
pub use test_login::test_login;
pub use upload_photo::upload_photo_path;

fn get_client() -> &'static reqwest::Client {
    unsafe {
        if CLIENT.is_none() {
            CLIENT = Some(reqwest::Client::new())
        }

        CLIENT.as_ref().unwrap()
    }
}

pub fn deserialize_content<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let v: Value = Deserialize::deserialize(deserializer)?;
    Ok(v["_content"].as_str().unwrap_or("").to_string())
}

/// Common error type for all flickr api answers
#[derive(Serialize, Deserialize, Debug, Hash)]
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
pub fn download_image(url: &String) -> Result<Reader<Cursor<Bytes>>, Box<dyn Error>> {
    let res = block_on(get_client().get(url).send())?;

    Ok(Reader::new(Cursor::new(block_on(res.bytes())?)))
}
