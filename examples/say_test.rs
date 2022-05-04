extern crate openai_rust_client;

use openai_rust_client::{ApiKey, OpenAIClient};
use openai_rust_client::endpoints::{CreateCompletionBuilder, Prompt};
use tokio;

#[tokio::main]
async fn main() {
    let api_key = std::env::var("API_KEY").unwrap();
    let c = OpenAIClient::new(ApiKey::new(api_key));
    let cc = CreateCompletionBuilder::new("text-davinci-002".to_string())
        .prompt(Prompt::One { one: "Say this is a test.".to_string() })
        .max_tokens(12)
        .build()
        .unwrap();
    println!("{:?}",  c.send(cc).await);
}