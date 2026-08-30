#![allow(dead_code)]
use crate::*;

#[derive(Deserialize, Debug, Hash)]
#[serde(untagged)]
enum FlickrGetInfoAnswer {
    Ok(FlickrGetInfoSuccess),
    Err(FlickrError),
}

impl Resultable<PhotoInfo, Error> for FlickrGetInfoAnswer {
    fn to_result(self) -> Result<PhotoInfo, Error> {
        match self {
            FlickrGetInfoAnswer::Ok(info) => Ok(info.photo),
            FlickrGetInfoAnswer::Err(e) => Err(Error::Api(e)),
        }
    }
}

#[derive(Deserialize, Debug, Hash)]
struct FlickrGetInfoSuccess {
    stat: String,
    photo: PhotoInfo,
}

#[derive(Deserialize, Debug, Hash)]
pub struct PhotoInfo {
    pub dateuploaded: String,
    pub farm: u32,
    pub id: String,
    pub isfavorite: u32,
    pub license: String,
    pub originalformat: String,
    pub originalsecret: String,
    pub rotation: u32,
    pub safety_level: String,
    pub secret: String,
    pub server: String,
    pub views: String,
    pub media: String,

    pub owner: Owner,
    pub dates: Dates,

    #[serde(deserialize_with = "deserialize_content")]
    pub title: String,

    #[serde(deserialize_with = "deserialize_content")]
    pub description: String,

    #[serde(deserialize_with = "deserialize_content")]
    pub comments: String,

    pub visibility: Visibility,

    // Only present when authenticated as the photo's owner
    pub permissions: Option<Permissions>,
    pub editability: Editability,
    pub publiceditability: Editability,

    // Absent if the photo has no geodata, otherwise either "" or a whole object
    pub location: Option<Location>,

    // Only present when authenticated
    pub geoperms: Option<GeoPerms>,

    pub notes: NoteWrapper,
    pub tags: TagWrapper,
    pub urls: UrlWrapper,
    pub usage: Usage,
}

#[derive(Deserialize, Debug, Hash)]
pub struct Owner {
    pub nsid: String,
    pub username: String,
    pub realname: String,
    pub location: String,
    pub iconserver: String,
    pub iconfarm: u32,
    pub path_alias: Option<String>,
}

#[derive(Deserialize, Debug, Hash)]
pub struct Dates {
    pub posted: String,
    pub taken: String,
    pub takengranularity: u32,
    pub takenunknown: String,
    pub lastupdate: String,
}

#[derive(Deserialize, Debug, Hash)]
pub struct Visibility {
    pub ispublic: u32,
    pub isfriend: u32,
    pub isfamily: u32,
}

#[derive(Deserialize, Debug, Hash)]
pub struct Permissions {
    pub permcomment: u32,
    pub permaddmeta: u32,
}

#[derive(Deserialize, Debug, Hash)]
pub struct Editability {
    pub cancomment: u32,
    pub canaddmeta: u32,
}

#[derive(Deserialize, Debug, Hash)]
pub struct Usage {
    pub candownload: u32,
    pub canblog: u32,
    // Absent on unauthenticated responses
    pub canprint: Option<u32>,
    pub canshare: u32,
}

#[derive(Deserialize, Debug, Hash)]
#[serde(untagged)]
pub enum Location {
    Full(LocationData),
    Empty(String),
}

#[derive(Deserialize, Debug, Hash)]
pub struct LocationData {
    pub latitude: String,
    pub longitude: String,
    pub accuracy: String,
    pub context: String,
    #[serde(deserialize_with = "deserialize_content")]
    pub locality: String,
    #[serde(deserialize_with = "deserialize_content")]
    pub county: String,
    #[serde(deserialize_with = "deserialize_content")]
    pub region: String,
    #[serde(deserialize_with = "deserialize_content")]
    pub country: String,
    #[serde(deserialize_with = "deserialize_content")]
    pub neighbourhood: String,
}

#[derive(Deserialize, Debug, Hash)]
pub struct GeoPerms {
    pub ispublic: u32,
    pub iscontact: u32,
    pub isfriend: u32,
    pub isfamily: u32,
}

#[derive(Deserialize, Debug, Hash)]
pub struct NoteWrapper {
    pub note: Vec<Note>,
}

#[derive(Deserialize, Debug, Hash)]
pub struct Note {
    pub id: String,
    pub photo_id: String,
    pub author: String,
    pub authorname: String,
    pub authorrealname: String,
    pub authorispro: u32,
    pub authorisdeleted: u32,
    pub x: String,
    pub y: String,
    pub w: String,
    pub h: String,
    pub _content: String,
}

#[derive(Deserialize, Debug, Hash)]
pub struct TagWrapper {
    pub tag: Vec<Tag>,
}

#[derive(Deserialize, Debug, Hash)]
pub struct Tag {
    pub id: String,
    pub author: String,
    pub authorname: String,
    pub raw: String,
    pub _content: String,
    pub machine_tag: u32,
}

#[derive(Deserialize, Debug, Hash)]
pub struct UrlWrapper {
    pub url: Vec<Url>,
}

#[derive(Deserialize, Debug, Hash)]
pub struct Url {
    #[serde(rename = "type")]
    pub urltype: String,
    pub _content: String,
}

impl PhotoRequestBuilder {
    /// [flickr.photos.getInfo](https://www.flickr.com/services/api/flickr.photos.getInfo.html)
    /// endpoint. Returns information associated with the photo of the given ID.
    ///
    /// `secret` allows bypassing the permission checks if given. Does not require authentication but
    /// will authenticate the user if given the token.
    pub async fn get_info(&self, id: &String, secret: Option<&String>) -> Result<PhotoInfo, Error> {
        let mut params = vec![
            ("method", "flickr.photos.getInfo".into()),
            ("photo_id", id.clone()),
            ("nojsoncallback", "1".into()),
            ("format", "json".into()),
            ("api_key", self.handle.key.key.clone()),
        ];
        if let Some(value) = secret {
            params.push(("secret", value.clone()));
        }
        oauth::build_request(
            oauth::RequestTarget::Get(URL_API),
            &mut params,
            &self.handle.key,
            self.handle.token.as_ref(),
        );

        let url = reqwest::Url::parse_with_params(URL_API, &params)?;
        let fetch = self.handle.client.get(url).send().await?;
        let raw = fetch.text().await?;
        #[cfg(debug_assertions)]
        log::debug!("Received {raw}");
        let answer: FlickrGetInfoAnswer = serde_json::from_str(&raw)?;

        answer.to_result()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unauthenticated calls omit `permissions` and `geoperms`, and photos without geodata omit
    /// `location` entirely rather than sending an empty value.
    #[test]
    fn parses_unauthenticated_response_without_permissions_geoperms_or_location() {
        let raw = r#"{"photo":{"id":"1","secret":"a","server":"1","farm":1,"dateuploaded":"1",
            "isfavorite":0,"license":"0","safety_level":"0","rotation":0,"originalsecret":"b",
            "originalformat":"jpg","owner":{"nsid":"1@N00","username":"u","realname":"r",
            "location":"","iconserver":"1","iconfarm":1,"path_alias":null},
            "title":{"_content":"t"},"description":{"_content":"d"},
            "visibility":{"ispublic":1,"isfriend":0,"isfamily":0},
            "dates":{"posted":"1","taken":"2026-01-25 15:00:01","takengranularity":0,
            "takenunknown":"0","lastupdate":"1"},"views":"1",
            "editability":{"cancomment":0,"canaddmeta":0},
            "publiceditability":{"cancomment":1,"canaddmeta":0},
            "usage":{"candownload":1,"canblog":0,"canshare":1},
            "comments":{"_content":"0"},"notes":{"note":[]},"tags":{"tag":[]},
            "urls":{"url":[{"type":"photopage","_content":"https://example.com/"}]},
            "media":"photo"},"stat":"ok"}"#;

        let answer: FlickrGetInfoAnswer = serde_json::from_str(raw).unwrap();
        let info = answer.to_result().unwrap();

        assert!(info.permissions.is_none());
        assert!(info.geoperms.is_none());
        assert!(info.location.is_none());
        assert!(info.usage.canprint.is_none());
    }
}
