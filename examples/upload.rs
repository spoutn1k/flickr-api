use flickr_api::*;
use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::path::Path;

fn prompt(message: &str) -> String {
    // Create a new String to hold the user's input
    let mut input = String::new();

    // Print a prompt message to the console
    print!("{message}");

    // Ensure the prompt message is printed immediately
    io::stdout().flush().ok();

    // Read the user's input from standard input
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    // Remove the trailing newline character
    input.trim().to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let arg = env::args().nth(1).unwrap();
    let path = Path::new(&arg);
    println!("Uploading {path:?}");

    let client = FlickrAPI::new(ApiKey {
        key: prompt("API key: "),
        secret: prompt("API secret: "),
    })
    .login()
    .await?;

    let id = client.photos().upload_from_path(&path).await?;
    println!("Uploaded {path:?} and was given {id}");

    Ok(())
}
