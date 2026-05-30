//! Caliper MCP server: newline-delimited JSON-RPC 2.0 over stdio.
//!
//! Transport is intentionally tiny and synchronous: read one JSON message per line from
//! stdin, route it through `caliper::mcp::handle_message`, and write the response (if any) as
//! one line to stdout. No async runtime, no threads, no logging of inputs.

use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<serde_json::Value>(trimmed) {
            Ok(req) => caliper::mcp::handle_message(&req),
            Err(e) => Some(serde_json::json!({
                "jsonrpc": "2.0",
                "id": serde_json::Value::Null,
                "error": { "code": -32700, "message": format!("parse error: {e}") },
            })),
        };
        if let Some(resp) = response {
            let line = serde_json::to_string(&resp)?;
            out.write_all(line.as_bytes())?;
            out.write_all(b"\n")?;
            out.flush()?;
        }
    }
    Ok(())
}
