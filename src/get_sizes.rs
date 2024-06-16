use crate::*;

#[derive(Serialize, Deserialize, Debug, Hash, Clone)]
pub struct FlickrSize {
    pub label: String,
    pub width: u32,
    pub height: u32,
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

impl FlickrGetSizesAnswer {
    fn ok(self) -> Result<FlickrSizeWrapper, Box<dyn Error>> {
        match self {
            FlickrGetSizesAnswer::Ok(k) => Ok(k),
            FlickrGetSizesAnswer::Err(e) => {
                Err(format!("flickr API call failed with code {}: {}", e.code, e.message).into())
            }
        }
    }
}

pub fn photos_getsizes(
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
    let fetch = block_on(get_client().get(url).send())?;
    let answer: FlickrGetSizesAnswer = block_on(fetch.json())?;

    let FlickrSizeWrapper {
        sizes: FlickrSizes { size: information },
    } = answer.ok()?;

    Ok(information)
}
