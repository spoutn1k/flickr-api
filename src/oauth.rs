use data_encoding::BASE64;
use hmac::{Hmac, Mac};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use url::form_urlencoded;

type HmacSha1 = Hmac<sha1::Sha1>;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Token {
    pub token: String,
    pub secret: String,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct ApiKey {
    pub key: String,
    pub secret: String,
}

pub trait Resultable<T> {
    fn to_result(self) -> Result<T, String>;
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OauthAccessAnswer {
    Ok(OauthAccessGranted),
    Err(OauthErrorDescription),
}

impl Resultable<OauthAccessGranted> for OauthAccessAnswer {
    fn to_result(self) -> Result<OauthAccessGranted, String> {
        match self {
            OauthAccessAnswer::Ok(k) => Ok(k),
            OauthAccessAnswer::Err(e) => Err(e.oauth_problem),
        }
    }
}

impl Resultable<Token> for OauthAccessAnswer {
    fn to_result(self) -> Result<Token, String> {
        match self {
            OauthAccessAnswer::Ok(OauthAccessGranted {
                fullname: _,
                username: _,
                user_nsid: _,
                oauth_token: token,
                oauth_token_secret: secret,
            }) => Ok(Token { token, secret }),
            OauthAccessAnswer::Err(e) => Err(e.oauth_problem),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OauthTokenAnswer {
    Ok(OauthTokenGranted),
    Err(OauthErrorDescription),
}

impl Resultable<Token> for OauthTokenAnswer {
    fn to_result(self) -> Result<Token, String> {
        match self {
            OauthTokenAnswer::Ok(OauthTokenGranted {
                oauth_callback_confirmed: _,
                oauth_token: token,
                oauth_token_secret: secret,
            }) => Ok(Token { token, secret }),
            OauthTokenAnswer::Err(e) => Err(e.oauth_problem),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct OauthAccessGranted {
    pub fullname: String,
    pub username: String,
    pub user_nsid: String,
    pub oauth_token: String,
    pub oauth_token_secret: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct OauthTokenGranted {
    pub oauth_callback_confirmed: String,
    pub oauth_token: String,
    pub oauth_token_secret: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct OauthErrorDescription {
    pub oauth_problem: String,
    #[serde(default)]
    pub debug_sbs: String,
}

pub enum RequestTarget<'a> {
    Get(&'a str),
    Post(&'a str),
}

impl<'a> RequestTarget<'a> {
    fn uri(&'a self) -> &'a str {
        match self {
            RequestTarget::Get(val) => val,
            RequestTarget::Post(val) => val,
        }
    }
}

pub fn build_request(
    target: RequestTarget,
    params: &mut Vec<(&'static str, String)>,
    api: &ApiKey,
    oauth: Option<&Token>,
) {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    let nonce = seconds[1..9].to_string();

    params.extend(vec![
        ("oauth_consumer_key", api.key.clone()),
        ("oauth_nonce", nonce),
        ("oauth_signature_method", "HMAC-SHA1".to_string()),
        ("oauth_timestamp", seconds),
        ("oauth_version", "1.0".to_string()),
    ]);

    let key: String;

    match &oauth {
        Some(value) => {
            params.extend(vec![("oauth_token", value.token.clone())]);
            key = format!("{}&{}", api.secret, value.secret)
        }
        None => key = format!("{}&", api.secret),
    };

    params.sort_by(|a, b| a.0.cmp(b.0));

    let to_sign = params
        .iter()
        .filter(|(k, _)| !vec!["photo"].contains(k))
        .map(|(a, b)| {
            format!(
                "{a}={}",
                form_urlencoded::byte_serialize(&b.as_bytes()).collect::<String>(),
            )
        })
        .join("&");

    let uri = target.uri();
    let method = match target {
        RequestTarget::Get(_) => "GET",
        RequestTarget::Post(_) => "POST",
    };

    let raw = format!(
        "{method}&{}&{}",
        form_urlencoded::byte_serialize(&uri.as_bytes()).collect::<String>(),
        form_urlencoded::byte_serialize(&to_sign.as_bytes()).collect::<String>()
    );

    let mut mac = HmacSha1::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");
    mac.update(raw.as_bytes());
    let signature: String = BASE64.encode(&mac.finalize().into_bytes());

    params.push(("oauth_signature", signature));
}
