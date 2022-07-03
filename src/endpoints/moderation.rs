use std::borrow::Cow;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::DeserializeOwned;
use crate::{Method, Request};

pub enum ModerationsModel {
    TextModerationStable,
    TextModerationLatest,
}

impl Serialize for ModerationsModel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self {
            ModerationsModel::TextModerationStable => {
                serializer.serialize_str("text-moderation-stable")
            }
            ModerationsModel::TextModerationLatest => {
                serializer.serialize_str("text-moderation-latest")
            }
        }
    }
}

#[derive(Serialize)]
pub struct Moderations {
    /// The input text to classify
    // N.B. technically you can pass a single string for classification too, but I don't feel like
    // that's worth implementing
    pub input: Vec<String>,
    /// The model to use. If unspecified, the default at time of writing is to use
    ///  `text-moderation-latest`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ModerationsModel>
}

#[non_exhaustive]
#[derive(Deserialize, Debug)]
/// A list of all the categories that the model potentially classifies content into
pub struct Categories<T> {
    pub hate: T,
    #[serde(rename = "hate/threatening")]
    pub hate_threatening: T,
    #[serde(rename="self-harm")]
    pub self_harm: T,
    pub sexual: T,
    #[serde(rename="sexual/minors")]
    pub sexual_minors: T,
    pub violence: T,
    #[serde(rename="violence/graphic")]
    pub violence_graphic: T
}


#[derive(Deserialize, Debug)]
pub struct ModerationsResult {
    /// each field holds a 1 if that category was flagged for moderation and a 0 otherwise
    pub categories: Categories<u8>,
    /// "Contains a dictionary of per-category raw scores output by the model,
    /// denoting the model's confidence that the input violates the OpenAI's policy for
    /// the category. The value is between 0 and 1, where higher values denote higher confidence.
    /// The scores should not be interpreted as probabilities."
    pub category_scores: Categories<f64>,
    /// this will be a 1 if *any* category was flagged for moderation and a 0 otherwise
    pub flagged: u8,
}

#[derive(Deserialize, Debug)]
pub struct ModerationsResponse {
    /// an arbitrary ID identifying this moderation
    pub id: String,
    /// The name of the text model used to generate this moderation
    pub model: String,
    pub results: Vec<ModerationsResult>
}

impl Request for Moderations {
    type Resp = ModerationsResponse;
    type Body = Self;
    const METHOD: Method = Method::POST;

    fn endpoint(&self) -> Cow<str> {
        Cow::from("moderations")
    }

    fn body(&self) -> Option<&Self::Body> {
        Some(&self)
    }
}