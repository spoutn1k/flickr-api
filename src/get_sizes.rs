use crate::*;

/// A size descriptor as returned by flickr
#[derive(Serialize, Deserialize, Debug, Hash, Clone)]
pub struct FlickrSize {
    /// Internal label for the size format
    pub label: String,
    pub width: u32,
    pub height: u32,
    /// The url of the photo
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug, Hash)]
struct FlickrSizes {
    size: Vec<FlickrSize>,
}

#[derive(Serialize, Deserialize, Debug, Hash)]
struct FlickrSizeWrapper {
    sizes: FlickrSizes,
}

#[derive(Serialize, Deserialize, Debug, Hash)]
#[serde(untagged)]
enum FlickrGetSizesAnswer {
    Ok(FlickrSizeWrapper),
    Err(FlickrError),
}

impl Resultable<Vec<FlickrSize>> for FlickrGetSizesAnswer {
    fn to_result(self) -> Result<Vec<FlickrSize>, String> {
        match self {
            FlickrGetSizesAnswer::Ok(FlickrSizeWrapper {
                sizes: FlickrSizes { size },
            }) => Ok(size),
            FlickrGetSizesAnswer::Err(e) => Err(format!("{e}")),
        }
    }
}

/// [flickr.photos.getSizes](https://www.flickr.com/services/api/flickr.photos.getSizes.html)
/// endpoint. Returns the available sizes for the photo of the given ID.
pub async fn photos_getsizes(
    id: &String,
    api: &ApiKey,
    oauth: Option<&OauthToken>,
) -> Result<Vec<FlickrSize>, Box<dyn Error>> {
    let mut params = vec![
        ("nojsoncallback", "1".into()),
        ("method", "flickr.photos.getSizes".into()),
        ("format", "json".into()),
        ("api_key", api.key.clone()),
        ("photo_id", id.clone()),
    ];
    oauth::build_request(oauth::RequestTarget::Get(URL_API), &mut params, api, oauth);

    let url = reqwest::Url::parse_with_params(URL_API, &params)?;
    let fetch = get_client().get(url).send().await?;
    let answer: FlickrGetSizesAnswer = fetch.json().await?;

    answer.to_result().map_err(|e| e.into())
}
