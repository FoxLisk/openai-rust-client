extern crate openai_rust_client;

use openai_rust_client::{ApiKey, OpenAIClient};
use openai_rust_client::endpoints::{CreateCompletionBuilder, Moderations, Prompt};
use tokio;

#[tokio::main]
async fn main() {
    let api_key = std::env::var("API_KEY").unwrap();
    let c = OpenAIClient::new(ApiKey::new(api_key));
    let m = Moderations {
        input: vec!["I am going to kill myself".to_string()],
        model: None
    };

    println!("{:?}",  c.send(&m).await);
}