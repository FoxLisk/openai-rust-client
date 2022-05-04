extern crate openai_rust_client;

use openai_rust_client::{ApiKey, OpenAIClient};
use openai_rust_client::endpoints::{ListEngines};
use tokio;

#[tokio::main]
async fn main() {
    let api_key = std::env::var("API_KEY").unwrap();
    let c = OpenAIClient::new(ApiKey::new(api_key));
    let le = ListEngines {};
    let es = c.send(le).await;
    println!("{:?}", es);
}