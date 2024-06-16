use crate::*;
use futures::Future;
use std::process::Command;
use tokio::sync::{mpsc, oneshot};
use warp::Filter;

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default)]
struct CallbackQuery {
    oauth_token: String,
    oauth_verifier: String,
}

fn setup_server() -> (u32, impl Future<Output = CallbackQuery>) {
    let (answer_tx, mut answer_rx) = mpsc::unbounded_channel::<CallbackQuery>();
    let authorization = warp::get()
        .and(warp::path!("authorization"))
        .and(warp::query::<CallbackQuery>())
        .and(warp::any().map(move || answer_tx.clone()))
        .and_then(
            |data, sender: mpsc::UnboundedSender<CallbackQuery>| async move {
                sender.send(data).ok();
                Result::<String, warp::Rejection>::Ok("<!DOCTYPE html><html><head>Authentication succeeded. You may close this window.<script>window.close();</script></head><body></body></html>".into())
            },
        ).map(|reply| {
        warp::reply::with_header(reply, "Content-Type", "text/html")
    });

    let (plug_tx, plug_rx) = oneshot::channel();
    let (_addr, server) =
        warp::serve(authorization).bind_with_graceful_shutdown(([127, 0, 0, 1], 8200), async {
            plug_rx.await.ok();
        });

    tokio::spawn(server);

    (8200, async move {
        let query = answer_rx.recv().await.unwrap();
        plug_tx.send(()).ok();
        query
    })
}

/// Top-level method enacting the procesure to receive an access token from a set of API keys
pub async fn get_token(api: &ApiKey) -> Result<OauthToken, Box<dyn Error>> {
    // Open an HTTP server on localhost to point the callback to
    let (port, answer) = setup_server();
    let callback_url = format!("http://localhost:{}/authorization", port);

    // Use the api keys to ask for a request token
    let response: oauth::OauthTokenAnswer = {
        let mut params = vec![("oauth_callback", callback_url)];
        oauth::build_request(
            oauth::RequestTarget::Get(URL_REQUEST),
            &mut params,
            &api,
            None,
        );
        let request = reqwest::Url::parse_with_params(URL_REQUEST, &params)?;
        let query = get_client().get(request).send().await?;
        serde_urlencoded::from_str(&query.text().await?)?
    };

    let request_token = response.to_result()?;

    // Prepare the link for the user to grant permission
    {
        let params = vec![
            ("oauth_token", request_token.token.clone()),
            ("perms", "write".to_string()),
        ];
        let url = reqwest::Url::parse_with_params(URL_AUTHORIZE, params)?.to_string();

        log::info!("OAuth link: {url}");

        #[cfg(target_os = "macos")]
        Command::new("open").args(vec![url]).spawn()?;

        #[cfg(target_os = "linux")]
        Command::new("xdg-open").args(vec![url]).spawn()?;
    }

    // Wait for the HTTP server to receive the callback query once the user accepted
    let callback_data = answer.await;

    // Exchange the request token for an access token with the verifier received above
    let response: oauth::OauthAccessAnswer = {
        let mut params = vec![("oauth_verifier", callback_data.oauth_verifier)];
        oauth::build_request(
            oauth::RequestTarget::Get(URL_ACCESS),
            &mut params,
            api,
            Some(&request_token),
        );
        let access = reqwest::Url::parse_with_params(URL_ACCESS, &params)?;
        let query = get_client().get(access).send().await?;
        serde_urlencoded::from_str(&query.text().await?)?
    };

    response.to_result().map_err(|e| e.into())
}
