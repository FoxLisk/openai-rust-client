use crate::{Request, Method};
use std::borrow::Cow;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Engine {
    id: String,
    object: String,
    owner: String,
    ready: bool,
}

#[derive(Deserialize, Debug)]
pub struct ListEnginesResponse {
    data: Vec<Engine>,
    object: String,
}

#[deprecated(since="0.1.1", note="Engines deprecated in favour of Models")]
pub struct ListEngines {}


impl Request for ListEngines {
    type Resp = ListEnginesResponse;
    type Body = ();
    const METHOD: Method = Method::GET;

    fn endpoint(&self) -> Cow<str> {
        Cow::from("engines")
    }
}
