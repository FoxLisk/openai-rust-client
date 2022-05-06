use crate::{Method, Request};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};
use std::borrow::Cow;
use std::collections::HashMap;

pub enum NullableOneOrMany<T> {
    None,
    One { one: T },
    Many { many: Vec<T> },
}

impl<T> Serialize for NullableOneOrMany<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        match self {
            NullableOneOrMany::None => serializer.serialize_none(),
            NullableOneOrMany::One { one } => one.serialize(serializer),
            NullableOneOrMany::Many { many } => many.serialize(serializer),
        }
    }
}

pub type Prompt = NullableOneOrMany<String>;
pub type Stop = NullableOneOrMany<String>;

/// Represents the create completion endpoint. see https://beta.openai.com/docs/api-reference/completions/create
/// use [CreateCompletionBuilder] to create

pub struct CreateCompletion {
    /// name of the engine to use; e.g. text-davinci-002
    engine_id: String,

    /// text prompt. this has certain limits I don't understand well yet.
    /// Can be empty (for blank prompt), a single, or multiple prompts
    prompt: Prompt,

    /// Suffix that comes after the completion text
    suffix: Option<String>,

    /// The maximum number of tokens to generate in the completion.
    /// Default 16
    max_tokens: Option<u16>,

    /// What sampling temperature to use. Higher values means the model will take more risks.
    temperature: Option<f32>,

    /// An alternative to sampling with temperature, called nucleus sampling, where the model considers the results of the tokens with top_p probability mass.
    top_p: Option<f32>,

    /// How many completions to generate for each prompt.
    /// Defaults to 1
    n: Option<u16>,

    // not implemented: stream
    /// Include the log probabilities on the logprobs most likely tokens, as well the chosen tokens.
    /// The docs say there's a max of 5; but the docs also mandate setting this to 10 for the content filter
    /// endpoint and mention that you can potentially ask for more than 5 if you ask them nicely.
    log_probs: Option<u16>,

    /// Echo back the prompt in addition to the completion
    echo: bool,

    /// Up to 4 sequences where the API will stop generating further tokens.
    /// The returned text will not contain the stop sequence.
    stop: Stop,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text
    /// so far, increasing the model's likelihood to talk about new topics.
    ///
    /// Default 0
    presence_penalty: Option<f32>,

    /// Generates best_of completions server-side and returns the "best"
    /// (the one with the lowest log probability per token). Results cannot be streamed.
    ///
    /// When used with n, best_of controls the number of candidate completions and n specifies how many to return –
    /// best_of must be greater than n.
    best_of: Option<u16>,

    /// Modify the likelihood of specified tokens appearing in the completion.
    logit: Option<HashMap<String, f32>>,
}

impl Serialize for CreateCompletion {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_map(None)?;
        match self.prompt {
            NullableOneOrMany::None => {}
            _ => {
                seq.serialize_entry("prompt", &self.prompt)?;
            }
        }
        if self.suffix.is_some() {
            seq.serialize_entry("suffix", &self.suffix)?;
        }
        if self.max_tokens.is_some() {
            seq.serialize_entry("max_tokens", &self.max_tokens)?;
        }
        if self.temperature.is_some() {
            seq.serialize_entry("temperature", &self.temperature)?;
        }
        if self.top_p.is_some() {
            seq.serialize_entry("top_p", &self.top_p)?;
        }
        if self.n.is_some() {
            seq.serialize_entry("n", &self.n)?;
        }
        if self.log_probs.is_some() {
            seq.serialize_entry("logprobs", &self.log_probs)?;
        }

        seq.serialize_entry("echo", &self.echo)?;
        if self.presence_penalty.is_some() {
            seq.serialize_entry("presence_penalty", &self.presence_penalty)?;
        }
        if self.best_of.is_some() {
            seq.serialize_entry("best_of", &self.best_of)?;
        }
        if self.logit.is_some() {
            seq.serialize_entry("logit", &self.logit)?;
        }
        seq.end()
    }
}


#[derive(Deserialize, Debug)]
pub struct LogProbs {
    /// The list of tokens from the completion.
    /// The concatenation of these is the full response; these were the tokens the engine used to create it
    pub tokens: Vec<String>,

    /// the logprobs of the actual tokens selected
    pub token_logprobs: Vec<f32>,

    /// the top choices for each token. Each element here should be a map of the top N or N+1 tokens by log probability,
    /// where N was given in the request. The actually-selected token is always included, so if it is was not one of the
    /// top N choices, it will be added (thus making the map N+1 elements)
    pub top_logprobs: Vec<HashMap<String, f32>>,

    /// The offset of the token in the text. This includes the given prompt; that is to say,
    /// if you have a 20 character prompt, the first element of this array will be 20.
    pub text_offset: Vec<u16>,
}

/// A Choice is effectively a completion.
#[derive(Deserialize, Debug)]
pub struct Choice {
    /// The completed text
    pub text: String,
    /// The index of the prompt this Choice was generated for
    pub index: usize,

    /// Log probabilities (if present)
    pub log_probs: Option<Vec<LogProbs>>,

    pub finish_reason: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
}

impl Request for CreateCompletion {
    type Resp = CreateCompletionResponse;
    type Body = Self;
    const METHOD: Method = Method::POST;

    fn endpoint(&self) -> Cow<str> {
        Cow::from(format!("engines/{}/completions", self.engine_id))
    }

    fn body(&self) -> Option<&Self::Body> {
        Some(&self)
    }
}

pub struct CreateCompletionBuilder {
    create_completion: Result<CreateCompletion, String>,
}

impl CreateCompletionBuilder {
    pub fn new<S:Into<String>>(engine_id: S) -> Self {
        Self {
            create_completion: Ok(CreateCompletion {
                engine_id: engine_id.into(),
                prompt: NullableOneOrMany::None,
                suffix: None,
                max_tokens: None,
                temperature: None,
                top_p: None,
                n: None,
                log_probs: None,
                echo: false,
                stop: NullableOneOrMany::None,
                presence_penalty: None,
                best_of: None,
                logit: None,
            }),
        }
    }

    pub fn prompt(mut self, prompt: Prompt) -> Self {
        match self.create_completion {
            Ok(ref mut cc) => {
                cc.prompt = prompt;
                self
            }
            Err(_) => self,
        }
    }

    pub fn suffix(mut self, suffix: String) -> Self {
        match self.create_completion {
            Ok(ref mut cc) => {
                cc.suffix = Some(suffix);
                self
            }
            Err(_) => self,
        }
    }

    pub fn max_tokens(mut self, max_tokens: u16) -> Self {
        match self.create_completion {
            Ok(ref mut cc) => {
                if max_tokens > 4096 {
                    self.create_completion =
                        Err("Max tokens cannot exceed 4096 on any model".to_string());
                    self
                } else {
                    cc.max_tokens = Some(max_tokens);
                    self
                }
            }
            Err(_) => self,
        }
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        match self.create_completion {
            Ok(ref mut cc) => {
                if !(0.0..=1.0).contains(&temperature) {
                    self.create_completion =
                        Err("Temperature must be in range [0, 1.0]".to_string());
                } else {
                    cc.temperature = Some(temperature);
                }
                self
            }
            Err(_) => self,
        }
    }
    pub fn top_p(mut self, top_p: f32) -> Self {
        match self.create_completion {
            Ok(ref mut cc) => {
                if !(0.0..=1.0).contains(&top_p) {
                    self.create_completion = Err("top_p must be in range [0, 1.0]".to_string());
                } else {
                    cc.top_p = Some(top_p);
                }
                self
            }
            Err(_) => self,
        }
    }
    pub fn n(mut self, n: u16) -> Self {
        match self.create_completion {
            Ok(ref mut cc) => {
                cc.n = Some(n);
                self
            }
            Err(_) => self,
        }
    }

    pub fn log_probs(mut self, log_probs: u16) -> Self {
        match self.create_completion {
            Ok(ref mut cc) => {
                cc.log_probs = Some(log_probs);
                self
            }
            Err(_) => self,
        }
    }
    pub fn echo(mut self, echo: bool) -> Self {
        match self.create_completion {
            Ok(ref mut cc) => {
                cc.echo = echo;
                self
            }
            Err(_) => self,
        }
    }
    pub fn stop(mut self, stop: Stop) -> Self {
        match self.create_completion {
            Ok(ref mut cc) => {
                cc.stop = stop;
                self
            }
            Err(_) => self,
        }
    }

    pub fn presence_penalty(mut self, presence_penalty: f32) -> Self {
        match self.create_completion {
            Ok(ref mut cc) => {
                if !(-2.0..=2.0).contains(&presence_penalty) {
                    self.create_completion =
                        Err("Temperature must be in range [0, 1.0]".to_string());
                } else {
                    cc.presence_penalty = Some(presence_penalty);
                }
                self
            }
            Err(_) => self,
        }
    }
    pub fn best_of(mut self, best_of: u16) -> Self {
        match self.create_completion {
            Ok(ref mut cc) => {
                cc.best_of = Some(best_of);
                self
            }
            Err(_) => self,
        }
    }

    pub fn logit(mut self, logit: HashMap<String, f32>) -> Self {
        match self.create_completion {
            Ok(ref mut cc) => {
                cc.logit = Some(logit);
                self
            }
            Err(_) => self,
        }
    }

    pub fn build(self) -> Result<CreateCompletion, String> {
        let cc = self.create_completion?;
        if let (Some(n), Some(best_of)) = (cc.n, cc.best_of) {
            if n <= best_of {
                return Err("If both are specified, best_of must be greater than n".to_string());
            }
        }
        Ok(cc)
    }
}
