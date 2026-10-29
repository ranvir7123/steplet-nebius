//! Nemotron on Nebius Token Factory (the Nebius plan, Task 2): the real model behind Steplet's own agent. It runs on
//! Windows, in Steplet's own program; the key goes only to Nebius, in one header, and never into Steplet's Linux.
//! Trying again after a busy or failed answer is the agent loop's job, so the live view can show each try.

use serde_json::Value;

use crate::gate::ApiKey;
use crate::model::{Message, Model, ModelError, ModelReply, ToolSpec, chat_body, read_reply};
use crate::winhttp::{Answer, Endpoint, HttpError, HttpLimits, send};

/// Nebius Token Factory's address.
pub const BASE: &str = "https://api.tokenfactory.nebius.com/v1/";

/// The most of the service's own words kept in an error: enough to see what it said, never a whole page.
const DETAIL: usize = 300;

pub struct TokenFactory {
    to: Endpoint,
    key: ApiKey,
    model: String,
    limits: HttpLimits,
}

impl TokenFactory {
    /// The real service, with Steplet's usual limits.
    pub fn new(key: ApiKey, model: &str) -> TokenFactory {
        TokenFactory::at(BASE, key, model, HttpLimits::default()).expect("BASE is a secure address")
    }

    /// Another address: `https` only, or plain `http` to this computer for the tests' stand-in server.
    pub fn at(base: &str, key: ApiKey, model: &str, limits: HttpLimits) -> Result<TokenFactory, String> {
        Ok(TokenFactory { to: Endpoint::parse(base)?, key, model: model.into(), limits })
    }

    /// The models the service offers, by name.
    pub fn models(&self) -> Result<Vec<String>, ModelError> {
        let answer = self.send("GET", "models", &[])?;
        let v: Value = serde_json::from_slice(&answer.body)
            .map_err(|e| ModelError::BadReply(format!("the list of models is not JSON ({e})")))?;
        let list = v["data"].as_array().ok_or_else(|| ModelError::BadReply("no list of models in it".into()))?;
        Ok(list.iter().filter_map(|m| m["id"].as_str().map(String::from)).collect())
    }

    /// One request; an answer whose status is not 200 becomes the error it stands for.
    fn send(&self, method: &str, path: &str, body: &[u8]) -> Result<Answer, ModelError> {
        let headers = format!(
            "Authorization: Bearer {}\r\nContent-Type: application/json\r\nAccept: application/json\r\n",
            self.key.text()
        );
        let answer = send(&self.to, method, path, &headers, body, &self.limits).map_err(|e| match e {
            HttpError::TooSlow => ModelError::TooSlow,
            HttpError::TooBig => ModelError::BadReply(format!("it was larger than {} bytes", self.limits.max_bytes)),
            HttpError::Network(why) => ModelError::Network(why),
        })?;
        if answer.status == 200 {
            return Ok(answer);
        }
        Err(self.error(&answer))
    }

    fn error(&self, answer: &Answer) -> ModelError {
        let detail = self.detail(&answer.body);
        let status = answer.status;
        let lower = detail.to_ascii_lowercase();
        let no_credit = ["insufficient", "balance", "credit", "payment"].iter().any(|w| lower.contains(w));
        match status {
            402 => ModelError::NoCredit { status, detail },
            400..=499 if no_credit => ModelError::NoCredit { status, detail },
            401 | 403 => ModelError::KeyRefused { status, detail },
            429 => ModelError::Busy {
                status,
                detail,
                retry_after: answer.retry_after.as_deref().and_then(|s| s.trim().parse().ok()),
            },
            500..=599 => ModelError::Unavailable { status, detail },
            _ => ModelError::Rejected { status, detail },
        }
    }

    /// The service's own words, cut short, with the key taken out wherever it shows.
    fn detail(&self, body: &[u8]) -> String {
        let text = String::from_utf8_lossy(body);
        let v: Option<Value> = serde_json::from_str(&text).ok();
        let said = v
            .as_ref()
            .and_then(|v| {
                v["error"]["message"].as_str().or(v["detail"].as_str()).or(v["message"].as_str()).map(String::from)
            })
            .unwrap_or_else(|| text.into_owned());
        let said = if self.key.text().is_empty() { said } else { said.replace(self.key.text(), "[the key]") };
        let said: String = said.split_whitespace().collect::<Vec<_>>().join(" ");
        match said.char_indices().nth(DETAIL) {
            Some((at, _)) => format!("{}…", &said[..at]),
            None => said,
        }
    }
}

impl Model for TokenFactory {
    fn id(&self) -> &str {
        &self.model
    }

    fn next(&mut self, convo: &[Message], tools: &[ToolSpec], max_tokens: u32) -> Result<ModelReply, ModelError> {
        let body = chat_body(&self.model, convo, tools, max_tokens);
        let answer = self.send("POST", "chat/completions", &body)?;
        read_reply(&answer.body)
    }
}
