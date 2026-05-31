//! MCP JSON-RPC round-trip tests over the pure `handle_message` router. Proves the
//! initialize handshake, tools/list, and a compute_score tools/call (success + typed error)
//! without spawning a process or an async runtime.

use caliper::mcp::handle_message;
use serde_json::json;

#[test]
fn initialize_handshake() {
    let resp = handle_message(&json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": { "protocolVersion": "2025-06-18", "capabilities": {} }
    }))
    .expect("initialize must respond");
    assert_eq!(resp["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(resp["result"]["serverInfo"]["name"], "caliper-mcp");
    assert!(resp["result"]["capabilities"]["tools"].is_object());
}

#[test]
fn initialized_notification_has_no_response() {
    let resp = handle_message(&json!({
        "jsonrpc": "2.0", "method": "notifications/initialized"
    }));
    assert!(resp.is_none());
}

#[test]
fn tools_list_exposes_all_tools() {
    let resp = handle_message(&json!({
        "jsonrpc": "2.0", "id": 2, "method": "tools/list"
    }))
    .unwrap();
    let tools = resp["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 9);
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    for expected in [
        "list_scores",
        "score_inputs",
        "compute_score",
        "convert_units",
        "solve_for",
        "score_series",
        "suggest_scores",
        "extract_inputs",
        "prepare_score",
    ] {
        assert!(names.contains(&expected), "missing tool {expected}");
    }
}

#[test]
fn tools_call_compute_score_roundtrip() {
    let resp = handle_message(&json!({
        "jsonrpc": "2.0", "id": 3, "method": "tools/call",
        "params": {
            "name": "compute_score",
            "arguments": { "id": "gcs", "inputs": { "eye": 3, "verbal": 4, "motor": 5 } }
        }
    }))
    .unwrap();
    let result = &resp["result"];
    assert_eq!(result["isError"], false);
    assert_eq!(result["structuredContent"]["value"], 12.0);
    assert_eq!(result["structuredContent"]["id"], "gcs");
    let text = result["content"][0]["text"].as_str().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(parsed["value"], 12.0);
}

#[test]
fn tools_call_surfaces_typed_calc_error() {
    let resp = handle_message(&json!({
        "jsonrpc": "2.0", "id": 4, "method": "tools/call",
        "params": {
            "name": "compute_score",
            "arguments": { "id": "gcs", "inputs": { "eye": 3, "verbal": 4 } }
        }
    }))
    .unwrap();
    assert_eq!(resp["result"]["isError"], true);
    assert_eq!(
        resp["result"]["structuredContent"]["error"],
        "MissingRequiredInput"
    );
    assert_eq!(resp["result"]["structuredContent"]["field"], "motor");
}

#[test]
fn unknown_method_is_jsonrpc_error() {
    let resp = handle_message(&json!({
        "jsonrpc": "2.0", "id": 5, "method": "does/not/exist"
    }))
    .unwrap();
    assert_eq!(resp["error"]["code"], -32601);
}
