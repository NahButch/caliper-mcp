//! Ingestion-layer tests, exercised through the MCP tool layer (extract_inputs, prepare_score)
//! and an end-to-end prepare -> compute round-trip.

use caliper::tools::call_tool;
use serde_json::json;

fn ok(args: serde_json::Value, tool: &str) -> serde_json::Value {
    let out = call_tool(tool, &args);
    assert!(!out.is_error, "{tool} errored: {}", out.value);
    out.value
}

#[test]
fn extract_inputs_unit_typed_and_provenance() {
    let v = ok(
        json!({ "text": "65M, creatinine 1.9 mg/dL, bilirubin 4 mg/dL, INR 1.5, Na 130 mmol/L" }),
        "extract_inputs",
    );
    let inputs = &v["inputs"];
    assert_eq!(inputs["creatinine"]["value"], 1.9);
    assert_eq!(inputs["creatinine"]["unit"], "mg/dl");
    assert_eq!(inputs["inr"], 1.5);
    assert_eq!(inputs["sodium"]["value"], 130.0);
    // provenance present for at least the creatinine hit
    assert!(v["provenance"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["field"] == "creatinine"));
}

#[test]
fn extract_never_assumes_unit() {
    // "Na 130" with no unit must NOT land in inputs; it goes to needs_unit.
    let v = ok(json!({ "text": "Na 130, INR 1.4" }), "extract_inputs");
    assert!(v["inputs"].get("sodium").is_none());
    let nu = v["needs_unit"].as_array().unwrap();
    assert!(nu
        .iter()
        .any(|n| n["field"] == "sodium" && n["suggested_unit"] == "mmol/L"));
    // the ratio still comes through (no unit needed)
    assert_eq!(v["inputs"]["inr"], 1.4);
}

#[test]
fn extract_isolates_unrecognized_unit() {
    let v = ok(
        json!({ "text": "creatinine 1.9 furlongs" }),
        "extract_inputs",
    );
    assert!(v["inputs"].get("creatinine").is_none());
    let uu = v["unrecognized_units"].as_array().unwrap();
    assert_eq!(uu.len(), 1);
    assert_eq!(uu[0]["unit"], "furlongs");
}

#[test]
fn extract_flag_negation() {
    let v = ok(
        json!({ "text": "CKD on dialysis; no diabetes" }),
        "extract_inputs",
    );
    assert_eq!(v["inputs"]["dialysis"], true);
    assert_eq!(v["inputs"]["diabetes"], false);
}

#[test]
fn extract_missing_tool_arg_is_error() {
    let out = call_tool("extract_inputs", &json!({}));
    assert!(out.is_error);
    assert_eq!(out.value["error"], "InvalidParams");
}

#[test]
fn prepare_score_reports_not_ready_with_needs_unit() {
    // MELD-Na needs creatinine, bilirubin, inr, sodium. Sodium has no unit here.
    let v = ok(
        json!({
            "id": "meld-na",
            "text": "creatinine 1.9 mg/dL, bilirubin 4 mg/dL, INR 1.5, sodium 130"
        }),
        "prepare_score",
    );
    assert_eq!(v["ready"], false);
    let missing = v["missing_required"].as_array().unwrap();
    assert!(missing.iter().any(|m| m["field"] == "sodium"));
    // The three with units are satisfied and present in the assembled inputs.
    assert_eq!(v["inputs"]["creatinine"]["value"], 1.9);
    assert!(v["inputs"].get("sodium").is_none());
}

#[test]
fn prepare_score_ready_when_units_present() {
    let v = ok(
        json!({
            "id": "meld-na",
            "text": "creatinine 1.9 mg/dL, bilirubin 4 mg/dL, INR 1.5, sodium 130 mmol/L"
        }),
        "prepare_score",
    );
    assert_eq!(v["ready"], true, "expected ready, got {v}");
    assert!(v["missing_required"].as_array().unwrap().is_empty());
}

#[test]
fn prepare_score_explicit_overrides_extracted() {
    // Text says 1.9; explicit inputs override to 2.5.
    let v = ok(
        json!({
            "id": "meld-na",
            "text": "creatinine 1.9 mg/dL, bilirubin 4 mg/dL, INR 1.5, sodium 130 mmol/L",
            "inputs": { "creatinine": { "value": 2.5, "unit": "mg/dL" } }
        }),
        "prepare_score",
    );
    assert_eq!(v["inputs"]["creatinine"]["value"], 2.5);
}

#[test]
fn prepare_then_compute_end_to_end() {
    // Prepare from text, confirm ready, then feed the assembled inputs straight to compute.
    let prep = ok(
        json!({
            "id": "meld-na",
            "text": "creatinine 1.9 mg/dL, bilirubin 4 mg/dL, INR 1.5, sodium 130 mmol/L"
        }),
        "prepare_score",
    );
    assert_eq!(prep["ready"], true);
    let inputs = prep["inputs"].clone();

    let result = ok(
        json!({ "id": "meld-na", "inputs": inputs }),
        "compute_score",
    );
    assert_eq!(result["value"], 26.0);
    assert_eq!(result["version"], "OPTN-2016");
}

#[test]
fn prepare_score_unknown_id_errors() {
    let out = call_tool("prepare_score", &json!({ "id": "nope", "text": "x" }));
    assert!(out.is_error);
    assert_eq!(out.value["error"], "UnknownScore");
}
