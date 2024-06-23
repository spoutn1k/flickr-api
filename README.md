This crate exists because I needed to access the [flickr](https://www.flickr.com) API and nothing was up to date.

The API is described [here](https://www.flickr.com/services/api/).

This crate uses [`warp`](https://github.com/seanmonstar/warp) to receive HTTP callbacks and you can log in from the command line using it.

# Usage

Log in using your local browser and upload a photo from a path:

```rs
// Create a client and ask it to log you in
let client = FlickrAPI::new(ApiKey {
    key: String::from("Your API key"),
    secret: String::from("Your API secret"),
})
.login()
.await?;

// Upload a local file
client.photos().upload_from_path(&path).await
```

# Coverage

The flickr API is extensive and this crate is very barebones. However adding support for a specific endpoint can be done in minutes ! Please create an issue if you need anything added !
