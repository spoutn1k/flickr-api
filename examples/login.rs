use flickr_api::{get_token, ApiKey};
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
    let api = ApiKey {
        key: prompt("API key: "),
        secret: prompt("API secret: "),
    };

    let token = get_token(&api).await?;
    println!("Received token: {} {}", token.token, token.secret);

    Ok(())
}
