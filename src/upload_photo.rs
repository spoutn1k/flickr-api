use crate::*;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "rsp")]
struct UploadXMLAnswer {
    stat: String,
    photoid: String,
}

pub fn upload_photo_path(
    path: &std::path::Path,
    api: &ApiKey,
    token: &OauthToken,
) -> Result<String, Box<dyn Error>> {
    let mut params = vec![];
    oauth::build_request(
        oauth::RequestTarget::Post(URL_UPLOAD),
        &mut params,
        api,
        Some(token),
    );

    let filename = String::from(path.to_str().unwrap_or("unknown"));
    let photo_part = reqwest::multipart::Part::bytes(read(path)?).file_name(filename.clone());

    let form = params
        .into_iter()
        .fold(reqwest::multipart::Form::new(), |form, (k, v)| {
            form.text(k, v)
        })
        .part("photo", photo_part);
    let request = block_on(get_client().post(URL_UPLOAD).multipart(form).send())?;
    let answer: UploadXMLAnswer = serde_xml_rs::from_str(&block_on(request.text())?)?;
    log::info!("Uploaded `{filename}` and received ID {}", answer.photoid);

    Ok(answer.photoid)
}
