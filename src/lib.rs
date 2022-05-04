pub mod endpoints;

use aliri_braid::braid;
use std::borrow::Cow;
use reqwest::Client as ReqwestClient;
use serde::de::DeserializeOwned;
use serde::Serialize;

const BASE_URL: &str = "https://api.openai.com/v1";

fn build_url(endpoint: Cow<str>) -> String {
    format!("{}/{}", BASE_URL, endpoint)
}

#[braid]
pub struct ApiKey;

pub enum Method {
    GET,
    POST,
}

#[derive(Debug)]
pub enum Error {
    /// an otherwise-unhandled error occurred making the http request
    HttpError { err: String },
    /// a 4xx-series error occurred
    ClientError { err: String, status: u16 },
    /// Error deserializing the payload
    DeserializeError { err: String },
}

impl From<Method> for reqwest::Method {
    fn from(m: Method) -> Self {
        match m {
            Method::GET => reqwest::Method::GET,
            Method::POST => reqwest::Method::POST
        }
    }
}

pub trait Request {
    type Resp: DeserializeOwned;
    type Body: Serialize;
    const METHOD: Method;

    fn endpoint(&self) -> Cow<str>;

    fn body(&self) -> Option<&Self::Body> {
        None
    }
}

pub struct OpenAIClient {
    api_key: ApiKey,
    client: ReqwestClient,
}

impl OpenAIClient {
    pub fn new(api_key: ApiKey) -> Self {
        Self {
            api_key,
            client: ReqwestClient::new(),
        }
    }
    pub async fn send<R: Request>(&self, req: R) -> Result<R::Resp, Error> {

        let mut http_req = self.client.request(R::METHOD.into(), build_url(req.endpoint()))
            .bearer_auth(self.api_key.clone());
        if let Some(b) = req.body() {

            http_req = http_req.json(b);
        }
        // return Err(Error::HttpError {err: "asdf".to_string()});
        let resp = http_req
            .send().await
            .map_err(|e| Error::HttpError {err: e.to_string()})?;
        let status = resp.status();
        if status.is_client_error() {
            let err = resp.text().await.map_err(|e| Error::HttpError {err: e.to_string()})?;
            return Err(Error::ClientError { status: status.as_u16(), err });
        }
        println!("{:?}", resp);
        resp.json().await
            .map_err(|e| Error::DeserializeError {err: e.to_string()})
    }
}

