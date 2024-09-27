use crate::*;
use reqwest::multipart::{Form, Part};
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

impl PhotoRequestBuilder {
    /// Access the "special" upload API and upload a photo from a given path
    pub async fn upload_from_path(&self, path: &std::path::Path) -> Result<String, Box<dyn Error>> {
        self.upload(
            &read(path).await?,
            Some(String::from(
                path.file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("unknown"),
            )),
        )
        .await
    }

    /// Access the "special" upload API and upload a photo from its contents
    pub async fn upload(
        &self,
        photo: &[u8],
        filename: Option<String>,
    ) -> Result<String, Box<dyn Error>> {
        let mut params = vec![];
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
