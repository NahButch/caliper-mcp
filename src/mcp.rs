//! Minimal, hand-rolled MCP JSON-RPC 2.0 routing over a single message at a time.
//!
//! [`handle_message`] is pure: it takes a parsed request value and returns the response value
//! (or `None` for notifications). The stdio framing lives in the server binary; keeping the
//! routing pure makes the full `initialize` -> `tools/call` round-trip testable without I/O.

use crate::{tools, MCP_PROTOCOL_VERSION, SERVER_VERSION};
use serde_json::{json, Value};

const SERVER_NAME: &str = "caliper-mcp";

const INSTRUCTIONS: &str = "Caliper exposes deterministic, version-pinned, unit-typed clinical \
calculations. Every physical quantity must be supplied as {value, unit}; missing inputs and \
unknown units return typed errors. Results are calculation-only and carry a citation, the \
formula version, and a disclaimer. Discover scores with list_scores, inspect a contract with \
score_inputs, then call compute_score.";

fn success(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// Route a single parsed JSON-RPC message. Returns `None` for notifications (no `id`).
pub fn handle_message(req: &Value) -> Option<Value> {
    let id = req.get("id").cloned();
    let method = req.get("method").and_then(Value::as_str);
    let is_notification = id.is_none();

    let method = match method {
        Some(m) => m,
        None => {
            if is_notification {
                return None;
            }
            return Some(error(id.unwrap_or(Value::Null), -32600, "missing 'method'"));
        }
    };

    // Notifications: act if relevant, never respond.
    if is_notification {
        return None;
    }
    let id = id.unwrap();
    let params = req.get("params").cloned().unwrap_or(json!({}));

    match method {
        "initialize" => Some(success(
            id,
            json!({
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": { "name": SERVER_NAME, "version": SERVER_VERSION },
                "instructions": INSTRUCTIONS,
            }),
        )),
        "ping" => Some(success(id, json!({}))),
        "tools/list" => Some(success(id, json!({ "tools": tool_defs() }))),
        "tools/call" => Some(handle_tools_call(id, &params)),
        other => Some(error(id, -32601, &format!("method not found: {other}"))),
    }
}

fn handle_tools_call(id: Value, params: &Value) -> Value {
    let name = match params.get("name").and_then(Value::as_str) {
        Some(n) => n,
        None => return error(id, -32602, "tools/call requires 'name'"),
    };
    let args = params.get("arguments").cloned().unwrap_or(json!({}));
    let outcome = tools::call_tool(name, &args);
    let text = serde_json::to_string_pretty(&outcome.value)
        .unwrap_or_else(|_| "{\"error\":\"Serialization\"}".to_string());
    success(
        id,
        json!({
            "content": [ { "type": "text", "text": text } ],
            "structuredContent": outcome.value,
            "isError": outcome.is_error,
        }),
    )
}

/// JSON-Schema tool definitions for `tools/list`.
pub fn tool_defs() -> Vec<Value> {
    vec![
        json!({
            "name": "list_scores",
            "description": "List available clinical scores, optionally filtered by domain and/or a free-text query.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "domain": { "type": "string", "description": "Filter by clinical domain (see list_scores output for the set)." },
                    "query": { "type": "string", "description": "Free-text match against id/name/keywords." }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "score_inputs",
            "description": "Return the full input contract for a score: each field's kind, allowed units, required flag, floors/ceilings, and notes.",
            "inputSchema": {
                "type": "object",
                "properties": { "id": { "type": "string" } },
                "required": ["id"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": "compute_score",
            "description": "Compute a score. Physical quantities must be {value, unit}. Returns a ScoreResult or a typed CalcError (missing input, unit required, unknown unit, out of range).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "inputs": { "type": "object", "description": "Field -> value map per score_inputs." }
                },
                "required": ["id", "inputs"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": "convert_units",
            "description": "Analyte-aware unit conversion. Returns the converted value, the multiplicative factor, and the conversion basis (e.g. molar mass).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "analyte": { "type": "string", "description": "e.g. creatinine, bilirubin, sodium, temperature." },
                    "value": { "type": "number" },
                    "from": { "type": "string" },
                    "to": { "type": "string" }
                },
                "required": ["analyte", "value", "from", "to"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": "solve_for",
            "description": "Monotone bisection over one numeric quantity input holding the others fixed; returns the threshold value of that input that produces the target score.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "target": { "type": "number", "description": "Desired score value." },
                    "solve": {
                        "type": "object",
                        "properties": { "field": { "type": "string" }, "unit": { "type": "string" } },
                        "required": ["field"]
                    },
                    "fixed": { "type": "object", "description": "Other inputs held constant." },
                    "bounds": { "type": "array", "items": { "type": "number" }, "description": "Optional [lo, hi] search bracket in the field's canonical unit." }
                },
                "required": ["id", "target", "solve", "fixed"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": "score_series",
            "description": "Compute a score across a time series; returns per-point values with deltas and trend labels, plus an overall trend.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "series": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": { "t": {}, "inputs": { "type": "object" } },
                            "required": ["inputs"]
                        }
                    }
                },
                "required": ["id", "series"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": "suggest_scores",
            "description": "Suggest candidate scores for a clinical context (domain, free text, and/or available inputs). Does NOT compute.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "context": {
                        "type": "object",
                        "properties": {
                            "domain": { "type": "string" },
                            "question": { "type": "string" },
                            "available_inputs": { "type": "array", "items": { "type": "string" } }
                        }
                    }
                },
                "additionalProperties": true
            }
        }),
    ]
}
