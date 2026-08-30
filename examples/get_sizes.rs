use clap::Parser;
use flickr_api::{ApiKey, FlickrAPI};
use std::error::Error;
use std::io::{self, Write};
use std::process::ExitCode;

/// List the available sizes for a Flickr photo given its ID.
#[derive(Parser)]
struct Args {
    /// ID of the photo to query
    id: String,

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

    let sizes = client.photos().get_sizes(&args.id).await?;

    for size in &sizes {
        println!(
            "{:<12} {:>5}x{:<5} {}",
            size.label, size.width, size.height, size.source
        );
    }

    Ok(ExitCode::SUCCESS)
}
