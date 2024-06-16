#![allow(dead_code)]
use crate::*;

#[derive(Deserialize, Debug, Hash)]
#[serde(untagged)]
enum FlickrGetInfoAnswer {
    Ok(FlickrGetInfoSuccess),
    Err(FlickrError),
}

impl Resultable<PhotoInfo> for FlickrGetInfoAnswer {
    fn to_result(self) -> Result<PhotoInfo, String> {
        match self {
            FlickrGetInfoAnswer::Ok(info) => Ok(info.photo),
            FlickrGetInfoAnswer::Err(e) => Err(format!("{e}")),
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

    pub permissions: Permissions,
    pub editability: Editability,
    pub publiceditability: Editability,

    // Either "" or a whole object
    pub location: Location,

    pub geoperms: GeoPerms,

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
    pub canprint: u32,
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

pub fn photos_getinfo(
    id: &String,
    secret: Option<&String>,
    api: &ApiKey,
    oauth: Option<&OauthToken>,
) -> Result<PhotoInfo, Box<dyn Error>> {
    let mut params = vec![
        ("method", "flickr.photos.getInfo".into()),
        ("photo_id", id.clone()),
        ("nojsoncallback", "1".into()),
        ("format", "json".into()),
        ("api_key", api.key.clone()),
    ];
    if let Some(value) = secret {
        params.push(("secret", value.clone()));
    }
    oauth::build_request(oauth::RequestTarget::Get(URL_API), &mut params, api, oauth);

    let url = reqwest::Url::parse_with_params(URL_API, &params)?;
    let fetch = block_on(get_client().get(url).send())?;
    let raw = block_on(fetch.text())?;
    #[cfg(debug_assertions)]
    log::debug!("Received {raw}");
    let answer: FlickrGetInfoAnswer = serde_json::from_str(&raw)?;

    answer.to_result().map_err(|e| e.into())
}
