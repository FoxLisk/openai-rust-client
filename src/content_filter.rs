//! Helps to make using the OpenAI content filtering easier
//! This filtering is required by their terms of service for any output that users will see. Or, well, it's
//! "recommended" - I'm pretty sure they'll be mad if you don't conform to their recommendations.
//! See https://beta.openai.com/docs/engines/content-filter

use crate::endpoints::{CreateCompletion, CreateCompletionBuilder, Prompt, CreateCompletionResponse};
use crate::{OpenAIClient, Error};

use std::fmt::{Display, Formatter};
use serde::Serialize;

/// Creates a content filter completion request.
/// Parameters are as prescribed in https://beta.openai.com/docs/engines/content-filter
pub fn create_content_filter_request<S: Display>(text: S) -> Result<CreateCompletion, String> {
    CreateCompletionBuilder::new("content-filter-alpha")
        .max_tokens(1)
        .temperature(0.0)
        .top_p(0.0)
        .log_probs(10)
        .prompt(
            Prompt::One {one: format!("<|endoftext|>{}\n--\nLabel:", text) }
        ).build()
}

#[derive(Debug, PartialEq, Eq, Serialize, Clone)]
pub enum FilterLabel {
    Safe,
    Sensitive,
    Unsafe
}

impl Display for FilterLabel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterLabel::Safe => {write!(f, "FilterLabel::Safe")}
            FilterLabel::Sensitive => {write!(f, "FilterLabel::Sensitive")}
            FilterLabel::Unsafe => {write!(f, "FilterLabel::Unsafe")}
        }
    }
}

#[derive(Debug)]
pub enum ClassificationError {
    NoLabelFound,
    TooManyLabels,
    UnexpectedLabel,
    MissingLogProbs
}

const TOXIC_THRESHOLD: f32 = -0.355;

/// Return the filter label if it can be determined, or an error
/// In the event of an error you should probably treat the text as "unsafe"
pub fn determine_filter_label(mut resp: CreateCompletionResponse) -> Result<FilterLabel, ClassificationError> {
    println!("filter resp: {:?}", resp);
    let label_choice = resp.choices.pop().ok_or(ClassificationError::NoLabelFound)?;
    if ! resp.choices.is_empty() {
        return Err(ClassificationError::TooManyLabels);
    }
    if label_choice.text == "0" {
        Ok(FilterLabel::Safe)
    } else if label_choice.text == "1" {
        Ok(FilterLabel::Sensitive)
    } else if label_choice.text == "2" {
        let lps = label_choice.log_probs.ok_or(ClassificationError::MissingLogProbs)?;
        let lp = lps.get(0).ok_or(ClassificationError::MissingLogProbs)?;
        let top_lp = lp.top_logprobs.get(0).ok_or(ClassificationError::MissingLogProbs)?;
        let unsafe_lp = top_lp.get("2").ok_or(ClassificationError::MissingLogProbs)?;
        if unsafe_lp >= &TOXIC_THRESHOLD {
            return Ok(FilterLabel::Unsafe);
        }

        let safe_lp = top_lp.get("0");
        let sensitive_lp = top_lp.get("1");
        Ok(match (safe_lp, sensitive_lp) {
            (Some(safe), Some(sensitive)) => {
                if safe >= sensitive {
                    FilterLabel::Safe
                } else {
                    FilterLabel::Sensitive
                }
            },
            (Some(_), None) => {
                FilterLabel::Safe
            },
            (None, Some(_)) => {
                FilterLabel::Sensitive
            },
            (None, None) => {FilterLabel::Unsafe},
        })


    } else {
        Err(ClassificationError::UnexpectedLabel)
    }
}

/// Runs all the steps in https://beta.openai.com/docs/engines/content-filter for you
#[deprecated(since="0.1.1", note="Use the moderations endpoint instead")]
pub async fn filter_content<S: Display>(text: S, c: &OpenAIClient) -> Result<FilterLabel, String> {
    let req = create_content_filter_request(text)?;
    let resp = match c.send(&req).await {
        Ok(r) => r,
        Err(e) => {
            match e {
                Error::HttpError { err } => {
                    // retry once
                    println!("Error getting content filtering; going to retry once. Err: {}", err);
                    c.send(&req).await.map_err(|e| e.to_string())?
                }
                Error::ClientError { err, status } => {
                    return Err(format!(
                        "Error making content filter request: status {status} | error {err}", status=status, err=err
                    ))
                }
                Error::DeserializeError { err } => {
                    return Err(format!("Error deserializing content filter response: {err}", err=err))
                }
            }
        }
    };

    determine_filter_label(resp).map_err(
        |_| "Error classifying text. You should treat this like an Unsafe classification".to_string()
    )

}