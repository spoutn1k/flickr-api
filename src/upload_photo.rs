use crate::*;
use reqwest::multipart::{Form, Part};
use std::path::Path;
use tokio::fs::read;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "rsp")]
struct UploadXMLAnswer {
    stat: String,

    #[serde(flatten)]
    content: UploadXMLPayload,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum UploadXMLPayload {
    #[serde(rename = "photoid")]
    PhotoId {
        #[serde(rename = "$value")]
        value: String,
    },

    #[serde(rename = "err")]
    Err { code: String, msg: String },
}

impl UploadXMLAnswer {
    fn to_result(self) -> Result<String, FlickrError> {
        match self.content {
            UploadXMLPayload::PhotoId { value } => Ok(value),
            UploadXMLPayload::Err { code, msg } => Err(FlickrError {
                stat: self.stat,
                code: code.parse().unwrap_or(0),
                message: msg,
            }),
        }
    }
}

/// Content safety rating applied to an uploaded photo.
///
/// See the [upload API docs](https://www.flickr.com/services/api/upload.api.html) for details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyLevel {
    Safe,
    Moderate,
    Restricted,
}

impl SafetyLevel {
    fn as_str(self) -> &'static str {
        match self {
            SafetyLevel::Safe => "1",
            SafetyLevel::Moderate => "2",
            SafetyLevel::Restricted => "3",
        }
    }
}

/// The kind of content being uploaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Photo,
    Screenshot,
    Other,
}

impl ContentType {
    fn as_str(self) -> &'static str {
        match self {
            ContentType::Photo => "1",
            ContentType::Screenshot => "2",
            ContentType::Other => "3",
        }
    }
}

/// Optional metadata to attach to an uploaded photo.
///
/// Fields left as `None` are omitted from the request, and Flickr applies its own defaults (generally taken from the account's upload settings).
#[derive(Debug, Clone, Default)]
pub struct UploadOptions {
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_public: Option<bool>,
    pub is_friend: Option<bool>,
    pub is_family: Option<bool>,
    pub safety_level: Option<SafetyLevel>,
    pub content_type: Option<ContentType>,
    /// When `Some(true)`, the photo is excluded from public search results.
    pub hidden: Option<bool>,
}

impl UploadOptions {
    fn into_params(self) -> Vec<(&'static str, String)> {
        let mut params = vec![];

        if let Some(title) = self.title {
            params.push(("title", title));
        }
        if let Some(description) = self.description {
            params.push(("description", description));
        }
        if let Some(tags) = self.tags {
            params.push(("tags", tags.join(",")));
        }
        if let Some(is_public) = self.is_public {
            params.push(("is_public", (is_public as u8).to_string()));
        }
        if let Some(is_friend) = self.is_friend {
            params.push(("is_friend", (is_friend as u8).to_string()));
        }
        if let Some(is_family) = self.is_family {
            params.push(("is_family", (is_family as u8).to_string()));
        }
        if let Some(safety_level) = self.safety_level {
            params.push(("safety_level", safety_level.as_str().to_string()));
        }
        if let Some(content_type) = self.content_type {
            params.push(("content_type", content_type.as_str().to_string()));
        }
        if let Some(hidden) = self.hidden {
            params.push(("hidden", if hidden { "2" } else { "1" }.to_string()));
        }

        params
    }
}

impl PhotoRequestBuilder {
    /// Access the "special" upload API and upload a photo from a given path
    pub async fn upload_from_path(
        &self,
        path: &Path,
        options: UploadOptions,
    ) -> Result<String, Error> {
        self.upload(
            &read(path).await?,
            Some(String::from(
                path.file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("unknown"),
            )),
            options,
        )
        .await
    }

    /// Access the "special" upload API and upload a photo from its contents
    pub async fn upload(
        &self,
        photo: &[u8],
        filename: Option<String>,
        options: UploadOptions,
    ) -> Result<String, Error> {
        // Metadata fields must be signed as part of the OAuth request, so they need to be in `params` before `build_request` computes the signature.
        let mut params = options.into_params();
        oauth::build_request(
            oauth::RequestTarget::Post(URL_UPLOAD),
            &mut params,
            &self.handle.key,
            self.handle.token.as_ref(),
        );

        // Filename is apparently required and request will fail if not set
        let photo_part =
            Part::bytes(Vec::from(photo)).file_name(filename.unwrap_or("unknown".to_string()));

        let form = params
            .into_iter()
            .fold(Form::new(), |form, (k, v)| form.text(k, v))
            .part("photo", photo_part);

        let request = self
            .handle
            .client
            .post(URL_UPLOAD)
            .multipart(form)
            .send()
            .await?;

        let text = request.text().await?;

        log::trace!("Upload response: {:?}", text);

        let id = serde_xml_rs::from_str::<UploadXMLAnswer>(&text)?.to_result()?;

        Ok(id)
    }
}

#[test]
fn test_upload_answer_error() {
    let anwser = r#"<?xml version="1.0" encoding="utf-8" ?><rsp stat="fail"><err code="5" msg="Filetype was not recognised"/></rsp>"#;

    let answer = serde_xml_rs::from_str::<UploadXMLAnswer>(anwser).unwrap();

    assert_eq!(
        answer,
        UploadXMLAnswer {
            stat: "fail".to_string(),
            content: UploadXMLPayload::Err {
                code: "5".to_string(),
                msg: "Filetype was not recognised".to_string()
            }
        }
    );
}

#[test]
fn test_upload_answer_ok() {
    let anwser = r#"<?xml version="1.0" encoding="utf-8" ?><rsp stat="ok"><photoid>54026462270</photoid></rsp>"#;

    let answer = serde_xml_rs::from_str::<UploadXMLAnswer>(anwser).unwrap();

    assert_eq!(
        answer,
        UploadXMLAnswer {
            stat: "ok".to_string(),
            content: UploadXMLPayload::PhotoId {
                value: "54026462270".to_string()
            }
        }
    );
}
