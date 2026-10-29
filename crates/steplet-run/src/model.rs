//! The model behind Steplet's own agent (the Nebius plan, Task 2): what Steplet sends it and what comes back, in the
//! OpenAI-style chat shape Nebius Token Factory speaks. The model only proposes; every move it asks for is decided by
//! the one checker and run, if allowed, inside the fence. `TokenFactory` is the real model; `Scripted` is the stand-in
//! the tests use, so a test does the same thing every time and costs nothing.

use std::collections::VecDeque;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};

/// Who said a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// One message of the conversation Steplet keeps with the model.
#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub role: Role,
    pub content: Option<String>,
    /// The model's tool calls, on an assistant message.
    pub tool_calls: Vec<ToolCall>,
    /// The call a tool message answers.
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn system(text: &str) -> Message {
        Message { role: Role::System, content: Some(text.into()), tool_calls: Vec::new(), tool_call_id: None }
    }

    pub fn user(text: &str) -> Message {
        Message { role: Role::User, content: Some(text.into()), tool_calls: Vec::new(), tool_call_id: None }
    }

    /// The model's own turn, as it came back, so the conversation holds exactly what it proposed.
    pub fn assistant(reply: &ModelReply) -> Message {
        Message {
            role: Role::Assistant,
            content: reply.text.clone(),
            tool_calls: reply.calls.clone(),
            tool_call_id: None,
        }
    }

    /// Steplet's answer to one tool call.
    pub fn tool(call_id: &str, text: &str) -> Message {
        Message {
            role: Role::Tool,
            content: Some(text.into()),
            tool_calls: Vec::new(),
            tool_call_id: Some(call_id.into()),
        }
    }
}

/// One move the model asks for. `arguments` is the JSON text exactly as the model wrote it: Steplet reads it, and a
/// call it cannot read is answered, never run.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// A tool Steplet offers the model: its name, what it does in words the model reads, and its arguments' JSON schema.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: Value,
}

/// Why the model stopped writing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Finish {
    /// It finished its answer.
    Stop,
    /// It asked for moves.
    ToolCalls,
    /// It reached the most words Steplet allowed for one answer.
    Length,
    Other(String),
}

/// Word-pieces ("tokens") sent and received, as the model's service counted them; what it charges for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Usage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
}

/// One answer from the model.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelReply {
    pub text: Option<String>,
    pub calls: Vec<ToolCall>,
    pub finish: Finish,
    pub usage: Usage,
    /// The model that answered, as the service named it.
    pub model: String,
    /// The model's thinking, when the service sends it apart from the answer. Never sent back to the model.
    pub reasoning: Option<String>,
}

/// Why the model could not answer. `detail` is what the service said, cut short, with the key taken out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelError {
    /// The service refused the key (status 401 or 403).
    KeyRefused { status: u16, detail: String },
    /// The account has no credit left.
    NoCredit { status: u16, detail: String },
    /// The service is busy (status 429); `retry_after` is its own wait, in seconds, when it gave one.
    Busy { status: u16, detail: String, retry_after: Option<u64> },
    /// The service failed on its side (status 500 and up).
    Unavailable { status: u16, detail: String },
    /// The service refused this request (any other status); trying again would not help.
    Rejected { status: u16, detail: String },
    /// The service could not be reached.
    Network(String),
    /// No answer in time.
    TooSlow,
    /// An answer came that Steplet could not read.
    BadReply(String),
}

impl ModelError {
    /// Whether trying the same request again may work.
    pub fn worth_retrying(&self) -> bool {
        matches!(
            self,
            ModelError::Busy { .. } | ModelError::Unavailable { .. } | ModelError::Network(_) | ModelError::TooSlow
        )
    }
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelError::KeyRefused { .. } => f.write_str("Nebius did not accept the key. Change it in Setup."),
            ModelError::NoCredit { .. } => {
                f.write_str("Your Nebius account has no credit left. Add credit at tokenfactory.nebius.com.")
            }
            ModelError::Busy { status, .. } => write!(f, "Nebius is busy (status {status})."),
            ModelError::Unavailable { status, .. } => write!(f, "Nebius had a problem on its side (status {status})."),
            ModelError::Rejected { status, detail } => {
                write!(f, "Nebius refused the request (status {status}): {detail}")
            }
            ModelError::Network(why) => write!(f, "Steplet could not reach Nebius: {why}"),
            ModelError::TooSlow => f.write_str("Nebius did not answer in time."),
            ModelError::BadReply(why) => write!(f, "Nebius's answer could not be read: {why}"),
        }
    }
}

/// A model Steplet's agent can talk to.
pub trait Model: Send {
    /// The model's name, as Steplet asks for it.
    fn id(&self) -> &str;
    /// The model's next answer to the conversation so far, offered `tools`, writing at most `max_tokens`.
    fn next(&mut self, convo: &[Message], tools: &[ToolSpec], max_tokens: u32) -> Result<ModelReply, ModelError>;
}

/// The request body Steplet sends for one answer. The agent loop measures it before sending, to know the most the
/// call could cost (`price::worst_case`), so it is built in one place.
pub fn chat_body(model: &str, convo: &[Message], tools: &[ToolSpec], max_tokens: u32) -> Vec<u8> {
    let messages: Vec<Value> = convo.iter().map(message_json).collect();
    let mut body = json!({ "model": model, "messages": messages, "max_tokens": max_tokens });
    if !tools.is_empty() {
        let tools: Vec<Value> = tools
            .iter()
            .map(|t| {
                json!({ "type": "function",
                        "function": { "name": t.name, "description": t.description, "parameters": t.parameters } })
            })
            .collect();
        body["tools"] = Value::Array(tools);
        body["tool_choice"] = json!("auto");
    }
    serde_json::to_vec(&body).expect("a JSON value always serialises")
}

fn message_json(m: &Message) -> Value {
    let role = match m.role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "tool",
    };
    let mut v = json!({ "role": role, "content": m.content });
    if !m.tool_calls.is_empty() {
        let calls: Vec<Value> = m
            .tool_calls
            .iter()
            .map(
                |c| json!({ "id": c.id, "type": "function", "function": { "name": c.name, "arguments": c.arguments } }),
            )
            .collect();
        v["tool_calls"] = Value::Array(calls);
    }
    if let Some(id) = &m.tool_call_id {
        v["tool_call_id"] = json!(id);
    }
    v
}

/// Reads one chat answer. Tool calls whose arguments came as a JSON object, not as text, are kept as that object's
/// text; a call with no id gets one, so Steplet can still answer it.
pub fn read_reply(body: &[u8]) -> Result<ModelReply, ModelError> {
    let v: Value = serde_json::from_slice(body).map_err(|e| ModelError::BadReply(format!("not JSON ({e})")))?;
    let choice = v["choices"].get(0).ok_or_else(|| ModelError::BadReply("no answer in it".into()))?;
    let message = &choice["message"];
    let text = content_text(&message["content"]);
    let reasoning = content_text(&message["reasoning_content"]).or_else(|| content_text(&message["reasoning"]));
    let mut calls = Vec::new();
    if let Some(list) = message["tool_calls"].as_array() {
        for (n, c) in list.iter().enumerate() {
            let function = &c["function"];
            let name = function["name"].as_str().unwrap_or_default().to_string();
            let arguments = match &function["arguments"] {
                Value::String(s) => s.clone(),
                Value::Null => String::new(),
                other => other.to_string(),
            };
            let id = c["id"].as_str().filter(|s| !s.is_empty()).map(String::from).unwrap_or(format!("call_{n}"));
            calls.push(ToolCall { id, name, arguments });
        }
    }
    let finish = match choice["finish_reason"].as_str() {
        Some("stop") => Finish::Stop,
        Some("tool_calls") => Finish::ToolCalls,
        Some("length") => Finish::Length,
        Some(other) => Finish::Other(other.into()),
        None => Finish::Other(String::new()),
    };
    let usage = Usage {
        prompt_tokens: v["usage"]["prompt_tokens"].as_u64().unwrap_or(0),
        completion_tokens: v["usage"]["completion_tokens"].as_u64().unwrap_or(0),
    };
    if v["usage"].is_null() {
        return Err(ModelError::BadReply("it did not say how much it used".into()));
    }
    Ok(ModelReply { text, calls, finish, usage, model: v["model"].as_str().unwrap_or_default().into(), reasoning })
}

fn content_text(v: &Value) -> Option<String> {
    match v {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Array(parts) => {
            let joined: String = parts.iter().filter_map(|p| p["text"].as_str()).collect();
            (!joined.is_empty()).then_some(joined)
        }
        _ => None,
    }
}

/// What a `Scripted` model gives on one turn.
#[derive(Clone, Debug)]
pub enum Step {
    Answer(ModelReply),
    Fail(ModelError),
    /// Waits this long, then gives `ModelError::TooSlow`: a model that has gone silent.
    Hang(Duration),
}

/// What a `Scripted` model was asked, turn by turn.
#[derive(Clone, Debug, PartialEq)]
pub struct Seen {
    pub convo: Vec<Message>,
    pub tools: Vec<&'static str>,
    pub max_tokens: u32,
}

/// The stand-in model: gives its steps in order, records what it was asked, and fails the test's run with
/// `BadReply` once its steps run out.
pub struct Scripted {
    id: String,
    steps: VecDeque<Step>,
    seen: Arc<Mutex<Vec<Seen>>>,
}

impl Scripted {
    pub fn new(id: &str, steps: Vec<Step>) -> Scripted {
        Scripted { id: id.into(), steps: steps.into(), seen: Arc::new(Mutex::new(Vec::new())) }
    }

    /// Everything the model was asked, readable after the model has been handed to a run.
    pub fn seen(&self) -> Arc<Mutex<Vec<Seen>>> {
        Arc::clone(&self.seen)
    }

    /// An answer that asks for one move.
    pub fn call(id: &str, name: &str, arguments: Value) -> Step {
        Step::Answer(ModelReply {
            text: None,
            calls: vec![ToolCall { id: id.into(), name: name.into(), arguments: arguments.to_string() }],
            finish: Finish::ToolCalls,
            usage: Usage { prompt_tokens: 1000, completion_tokens: 50 },
            model: "scripted".into(),
            reasoning: None,
        })
    }

    /// A last answer, with no moves.
    pub fn done(text: &str) -> Step {
        Step::Answer(ModelReply {
            text: Some(text.into()),
            calls: Vec::new(),
            finish: Finish::Stop,
            usage: Usage { prompt_tokens: 1000, completion_tokens: 50 },
            model: "scripted".into(),
            reasoning: None,
        })
    }
}

impl Model for Scripted {
    fn id(&self) -> &str {
        &self.id
    }

    fn next(&mut self, convo: &[Message], tools: &[ToolSpec], max_tokens: u32) -> Result<ModelReply, ModelError> {
        let seen = Seen { convo: convo.to_vec(), tools: tools.iter().map(|t| t.name).collect(), max_tokens };
        self.seen.lock().unwrap_or_else(|p| p.into_inner()).push(seen);
        match self.steps.pop_front() {
            Some(Step::Answer(reply)) => Ok(reply),
            Some(Step::Fail(e)) => Err(e),
            Some(Step::Hang(d)) => {
                std::thread::sleep(d);
                Err(ModelError::TooSlow)
            }
            None => Err(ModelError::BadReply("the stand-in model has no more answers".into())),
        }
    }
}
