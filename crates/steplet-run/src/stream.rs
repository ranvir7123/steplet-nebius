//! Claude Code's event stream (`--output-format stream-json`), one JSON object per line, read into Steplet's
//! events. The shapes are the ones recorded from Claude Code 2.1.220 in phase 0.

use serde_json::{Map, Value};

use crate::types::Event;

/// The events of one line of the stream. A message can hold several blocks, so a line can give several events; a
/// blank line or the model's thinking gives none. Any line, however odd, gives a result, never a panic.
pub fn parse_line(line: &str) -> Vec<Event> {
    let line = line.trim();
    if line.is_empty() {
        return Vec::new();
    }
    let Ok(Value::Object(e)) = serde_json::from_str::<Value>(line) else {
        return vec![Event::Unreadable(line.into())];
    };
    let kind = match (text(&e, "type"), e.get("subtype").and_then(Value::as_str)) {
        (t, Some(s)) => format!("{t}/{s}"),
        (t, None) => t.to_owned(),
    };
    // Any form of hook event stops the run (D3), including one this version does not send yet.
    if kind.contains("hook") {
        let name = text(&e, "hook_name");
        return vec![Event::HookEvent { name: if name.is_empty() { kind } else { name.into() } }];
    }
    // A message is read by its type alone, whatever else the line carries, so no tool call in it is missed.
    let typ = text(&e, "type");
    if typ == "assistant" || typ == "user" {
        return match e.get("message").and_then(|m| m.get("content")).and_then(Value::as_array) {
            Some(blocks) => {
                // The line's record belongs to its result only when the line holds just one.
                let results = blocks.iter().filter(|b| b.get("type").and_then(Value::as_str) == Some("tool_result"));
                let plain = results.count() == 1 && plain_end(e.get("tool_use_result"));
                blocks.iter().filter_map(|b| block(typ, b, plain)).collect()
            }
            None => vec![Event::Other(kind)],
        };
    }
    match kind.as_str() {
        "system/init" => vec![started(&e)],
        "system/model_refusal_fallback" | "system/model_refusal_no_fallback" => vec![Event::ModelSwitch {
            from: text(&e, "original_model").into(),
            to: e.get("fallback_model").and_then(Value::as_str).map(str::to_owned),
            category: text(&e, "api_refusal_category").into(),
            explanation: text(&e, "api_refusal_explanation").into(),
        }],
        "system/thinking_tokens" => Vec::new(),
        "system/api_retry" => vec![Event::ApiRetry {
            status: e.get("error_status").and_then(Value::as_u64).and_then(|s| u16::try_from(s).ok()),
            error: text(&e, "error").into(),
            attempt: number(&e, "attempt"),
            max: number(&e, "max_retries"),
        }],
        k if k.starts_with("result") => {
            let status = e.get("api_error_status").and_then(Value::as_u64).and_then(|s| u16::try_from(s).ok());
            status.map(|status| Event::ApiError { status }).into_iter().chain([result(&e)]).collect()
        }
        _ => vec![Event::Other(kind)],
    }
}

fn number(e: &Map<String, Value>, key: &str) -> u32 {
    e.get(key).and_then(Value::as_u64).and_then(|n| u32::try_from(n).ok()).unwrap_or(0)
}

fn result(e: &Map<String, Value>) -> Event {
    Event::Result {
        subtype: text(e, "subtype").into(),
        is_error: e.get("is_error").and_then(Value::as_bool).unwrap_or(true),
        stop_reason: text(e, "stop_reason").into(),
        models: e.get("modelUsage").and_then(Value::as_object).map(|m| m.keys().cloned().collect()).unwrap_or_default(),
        text: text(e, "result").into(),
    }
}

fn started(e: &Map<String, Value>) -> Event {
    let tools = e
        .get("tools")
        .and_then(Value::as_array)
        .map(|t| t.iter().filter_map(Value::as_str).map(str::to_owned).collect());
    let gate_connected = e.get("mcp_servers").and_then(Value::as_array).is_some_and(|servers| {
        servers.iter().any(|s| {
            s.get("name").and_then(Value::as_str) == Some("gate")
                && s.get("status").and_then(Value::as_str) == Some("connected")
        })
    });
    Event::Started { model: text(e, "model").into(), tools: tools.unwrap_or_default(), gate_connected }
}

/// Claude Code's own record of a command (`tool_use_result`) shows a plain end: not interrupted, and none of the marks
/// Claude Code sets when the exit code does not mean what "no error" says (its own meaning for the exit code, a move to
/// the background, a time-out).
fn plain_end(record: Option<&Value>) -> bool {
    let Some(Value::Object(r)) = record else { return false };
    let set = |k: &&str| r.get(*k).is_some_and(|v| !matches!(v, Value::Null | Value::Bool(false)) && v != "");
    r.get("interrupted") == Some(&Value::Bool(false))
        && !["returnCodeInterpretation", "backgroundTaskId", "backgroundedByUser", "timedOutAfterMs"].iter().any(set)
}

/// One block of an `assistant` or `user` message. `plain_end` is the line's record of a result (see `plain_end`).
fn block(kind: &str, b: &Value, plain_end: bool) -> Option<Event> {
    let field = |name: &str| b.get(name).and_then(Value::as_str).unwrap_or("").to_owned();
    match (kind, b.get("type").and_then(Value::as_str).unwrap_or("")) {
        (_, "thinking" | "redacted_thinking") => None,
        ("assistant", "text") => Some(Event::AgentText(field("text"))),
        // `tool_use`, and any other kind of call (`server_tool_use`, `mcp_tool_use`): each is a tool call.
        ("assistant", t) if t.ends_with("tool_use") => Some(Event::ToolUse {
            id: field("id"),
            name: field("name"),
            input: b.get("input").cloned().unwrap_or(Value::Null),
        }),
        ("user", "tool_result") => Some(Event::ToolResult {
            id: field("tool_use_id"),
            is_error: b.get("is_error").and_then(Value::as_bool).unwrap_or(false),
            text: match b.get("content") {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Array(parts)) => parts
                    .iter()
                    .filter(|p| p.get("type").and_then(Value::as_str) == Some("text"))
                    .filter_map(|p| p.get("text").and_then(Value::as_str))
                    .collect::<Vec<_>>()
                    .join("\n"),
                _ => String::new(),
            },
            plain_end,
        }),
        (_, t) => Some(Event::Other(format!("{kind}/{t}"))),
    }
}

fn text<'a>(e: &'a Map<String, Value>, field: &str) -> &'a str {
    e.get(field).and_then(Value::as_str).unwrap_or("")
}
