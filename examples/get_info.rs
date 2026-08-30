use clap::Parser;
use flickr_api::{ApiKey, FlickrAPI};
use std::error::Error;
use std::io::{self, Write};
use std::process::ExitCode;

/// Query information about a Flickr photo given its ID.
#[derive(Parser)]
struct Args {
    /// ID of the photo to query
    id: String,

    /// Secret of the photo, required to bypass permission checks on private photos
    secret: Option<String>,

    /// Flickr API key (defaults to the FLICKR_API_KEY env var, prompts if unset)
    #[arg(long, env = "FLICKR_API_KEY")]
    api_key: Option<String>,

    /// Flickr API secret (defaults to the FLICKR_API_SECRET env var, prompts if unset)
    #[arg(long, env = "FLICKR_API_SECRET")]
    api_secret: Option<String>,
}

fn prompt(message: &str) -> String {
    let mut input = String::new();

    print!("{message}");
    io::stdout().flush().ok();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input.trim().to_string()
}

#[tokio::main]
async fn main() -> Result<ExitCode, Box<dyn Error>> {
    env_logger::init();

    let args = Args::parse();

    let client = FlickrAPI::new(ApiKey {
        key: args.api_key.unwrap_or_else(|| prompt("API key: ")),
        secret: args.api_secret.unwrap_or_else(|| prompt("API secret: ")),
    });

    let info = client
        .photos()
        .get_info(&args.id, args.secret.as_ref())
        .await?;

    println!("Title:       {}", info.title);
    println!(
        "Owner:       {} ({})",
        info.owner.realname, info.owner.username
    );
    println!("Uploaded:    {}", info.dateuploaded);
    println!("Taken:       {}", info.dates.taken);
    println!("Views:       {}", info.views);
    println!("Media:       {}", info.media);
    println!("License:     {}", info.license);

    if !info.description.is_empty() {
        println!("Description: {}", info.description);
    }

    let tags: Vec<&str> = info.tags.tag.iter().map(|t| t._content.as_str()).collect();
    if !tags.is_empty() {
        println!("Tags:        {}", tags.join(", "));
    }

    if let Some(flickr_api::get_info::Location::Full(loc)) = &info.location {
        println!(
            "Location:    {} lat, {} long ({}, {})",
            loc.latitude, loc.longitude, loc.locality, loc.country
        );
    }

    for url in &info.urls.url {
        println!("URL ({}): {}", url.urltype, url._content);
    }

    Ok(ExitCode::SUCCESS)
}
