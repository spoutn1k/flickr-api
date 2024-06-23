use crate::*;
use reqwest::multipart::{Form, Part};
use tokio::fs::read;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "rsp")]
struct UploadXMLAnswer {
    stat: String,
    photoid: String,
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

        let answer: UploadXMLAnswer = serde_xml_rs::from_str(&request.text().await?)?;

        Ok(answer.photoid)
    }
}
