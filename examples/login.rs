use flickr_api::{ApiKey, FlickrAPI};
use std::error::Error;
use std::io::{self, Write};

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
    let client = FlickrAPI::new(ApiKey {
        key: prompt("API key: "),
        secret: prompt("API secret: "),
    })
    .login()
    .await?;

    let user = client.test().login().await?;

    println!("Successfully logged in as {} ({})", user.username, user.id);

    Ok(())
}
