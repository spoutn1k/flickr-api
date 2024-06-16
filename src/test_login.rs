use crate::*;

#[derive(Serialize, Deserialize, Debug, Hash)]
struct TestLoginAnswerSuccess {
    stat: String,
    user: UserData,
}

#[derive(Serialize, Deserialize, Debug, Hash)]
#[serde(untagged)]
enum TestLoginAnswer {
    Ok(TestLoginAnswerSuccess),
    Err(FlickrError),
}

impl Resultable<UserData> for TestLoginAnswer {
    fn to_result(self) -> Result<UserData, String> {
        match self {
            TestLoginAnswer::Ok(TestLoginAnswerSuccess { stat: _, user }) => Ok(user),
            TestLoginAnswer::Err(e) => Err(format!("{e}")),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Hash)]
pub struct UserData {
    pub id: String,
    #[serde(deserialize_with = "deserialize_content")]
    pub username: String,
}

pub fn test_login(api: &ApiKey, token: &OauthToken) -> Result<UserData, Box<dyn Error>> {
    let mut params = vec![
        ("method", "flickr.test.login".into()),
        ("format", "json".into()),
        ("nojsoncallback", "1".into()),
    ];

    oauth::build_request(
        oauth::RequestTarget::Get(URL_API),
        &mut params,
        api,
        Some(token),
    );

    let url = reqwest::Url::parse_with_params(URL_API, &params)?;
    let request = block_on(get_client().get(url).send())?;
    let login: TestLoginAnswer = block_on(request.json())?;

    login.to_result().map_err(|e| e.into())
}
