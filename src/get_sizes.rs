use crate::*;

/// A size descriptor as returned by flickr
#[derive(Deserialize, Debug, Hash, Clone)]
pub struct FlickrSize {
    /// Internal label for the size format
    pub label: String,
    pub width: u32,
    pub height: u32,
    /// The direct url of the image file for this size
    pub source: String,
    /// The Flickr photo page url for this size
    pub url: String,
    pub media: String,
}

/// The available sizes for a photo, along with the calling user's permissions on it
#[derive(Deserialize, Debug, Hash)]
pub struct PhotoSizes {
    pub canblog: u32,
    pub canprint: u32,
    pub candownload: u32,
    pub size: Vec<FlickrSize>,
}

#[derive(Deserialize, Debug, Hash)]
struct FlickrSizeWrapper {
    sizes: PhotoSizes,
}

#[derive(Deserialize, Debug, Hash)]
#[serde(untagged)]
enum FlickrGetSizesAnswer {
    Ok(FlickrSizeWrapper),
    Err(FlickrError),
}

impl Resultable<PhotoSizes, Error> for FlickrGetSizesAnswer {
    fn to_result(self) -> Result<PhotoSizes, Error> {
        match self {
            FlickrGetSizesAnswer::Ok(FlickrSizeWrapper { sizes }) => Ok(sizes),
            FlickrGetSizesAnswer::Err(e) => Err(Error::Api(e)),
        }
    }
}

impl PhotoRequestBuilder {
    /// [flickr.photos.getSizes](https://www.flickr.com/services/api/flickr.photos.getSizes.html)
    /// endpoint. Returns the available sizes for the photo of the given ID.
    pub async fn get_sizes(&self, id: &String) -> Result<PhotoSizes, Error> {
        let mut params = vec![
            ("nojsoncallback", "1".into()),
            ("method", "flickr.photos.getSizes".into()),
            ("format", "json".into()),
            ("api_key", self.handle.key.key.clone()),
            ("photo_id", id.clone()),
        ];
        oauth::build_request(
            oauth::RequestTarget::Get(URL_API),
            &mut params,
            &self.handle.key,
            self.handle.token.as_ref(),
        );

        let url = reqwest::Url::parse_with_params(URL_API, &params)?;
        let fetch = self.handle.client.get(url).send().await?;
        let answer: FlickrGetSizesAnswer = fetch.json().await?;

        answer.to_result()
    }
}
