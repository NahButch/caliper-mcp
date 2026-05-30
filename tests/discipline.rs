//! Unit-discipline and no-silent-defaults tests, exercised through the MCP tool layer.

use caliper::tools::call_tool;
use serde_json::json;

fn error_tag(args: serde_json::Value) -> (bool, String) {
    let out = call_tool("compute_score", &args);
    let tag = out
        .value
        .get("error")
        .and_then(|v| v.as_str())
        .unwrap_or("<none>")
        .to_string();
    (out.is_error, tag)
}

#[test]
fn missing_required_input_is_typed() {
    let (is_err, tag) = error_tag(json!({
        "id": "ckd-epi-2021",
        "inputs": { "age": { "value": 50, "unit": "years" }, "sex": "female" }
    }));
    assert!(is_err);
    assert_eq!(tag, "MissingRequiredInput");
}

#[test]
fn bare_number_where_unit_required_is_typed() {
    let (is_err, tag) = error_tag(json!({
        "id": "ckd-epi-2021",
        "inputs": { "creatinine": 0.9, "age": { "value": 50, "unit": "years" }, "sex": "female" }
    }));
    assert!(is_err);
    assert_eq!(tag, "UnitRequired");
}

#[test]
fn unknown_unit_is_typed() {
    let (is_err, tag) = error_tag(json!({
        "id": "ckd-epi-2021",
        "inputs": {
            "creatinine": { "value": 0.9, "unit": "furlongs" },
            "age": { "value": 50, "unit": "years" },
            "sex": "female"
        }
    }));
    assert!(is_err);
    assert_eq!(tag, "UnknownUnit");
}

#[test]
fn out_of_range_enum_is_typed() {
    let (is_err, tag) = error_tag(json!({
        "id": "ckd-epi-2021",
        "inputs": {
            "creatinine": { "value": 0.9, "unit": "mg/dL" },
            "age": { "value": 50, "unit": "years" },
            "sex": "intersex-unsupported-value"
        }
    }));
    assert!(is_err);
    assert_eq!(tag, "OutOfRange");
}

#[test]
fn success_carries_version_citation_and_disclaimer() {
    let out = call_tool(
        "compute_score",
        &json!({ "id": "gcs", "inputs": { "eye": 3, "verbal": 4, "motor": 5 } }),
    );
    assert!(!out.is_error);
    assert_eq!(out.value["value"], 12.0);
    assert_eq!(out.value["version"], "1974");
    assert_eq!(
        out.value["disclaimer"],
        "Calculation only. Not medical advice; not a medical device."
    );
    assert!(out.value["citation"].as_str().unwrap().contains("Teasdale"));
}

#[test]
fn dialysis_override_rule_fires() {
    let out = call_tool(
        "compute_score",
        &json!({
            "id": "meld-na",
            "inputs": {
                "creatinine": { "value": 1.2, "unit": "mg/dL" },
                "bilirubin": { "value": 4.0, "unit": "mg/dL" },
                "inr": 1.5,
                "sodium": { "value": 130, "unit": "mmol/L" },
                "dialysis": true
            }
        }),
    );
    assert!(!out.is_error);
    let rules = out.value["applied_rules"].as_array().unwrap();
    assert!(rules
        .iter()
        .any(|r| r.as_str().unwrap().contains("dialysis override")));
}

#[test]
fn meld_na_rules_match_value() {
    // cr=1.9 must NOT be floored; MELD(i)=22 (>11) so the sodium correction must apply.
    let out = call_tool(
        "compute_score",
        &json!({
            "id": "meld-na",
            "inputs": {
                "creatinine": { "value": 1.9, "unit": "mg/dL" },
                "bilirubin": { "value": 4.0, "unit": "mg/dL" },
                "inr": 1.5,
                "sodium": { "value": 130, "unit": "mmol/L" }
            }
        }),
    );
    assert_eq!(out.value["value"].as_f64().unwrap(), 26.0);
    let rules: Vec<String> = out.value["applied_rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_str().unwrap().to_string())
        .collect();
    assert!(
        !rules
            .iter()
            .any(|r| r.contains("creatinine") && r.contains("floored")),
        "creatinine 1.9 should not be floored; rules were {rules:?}"
    );
    assert!(
        rules
            .iter()
            .any(|r| r.contains("sodium correction applied")),
        "expected sodium correction; rules were {rules:?}"
    );
}

#[test]
fn meld_na_sodium_clamp_rule_fires() {
    let out = call_tool(
        "compute_score",
        &json!({
            "id": "meld-na",
            "inputs": {
                "creatinine": { "value": 1.9, "unit": "mg/dL" },
                "bilirubin": { "value": 4.0, "unit": "mg/dL" },
                "inr": 1.5,
                "sodium": { "value": 120, "unit": "mmol/L" }
            }
        }),
    );
    assert!(!out.is_error);
    let rules = out.value["applied_rules"].as_array().unwrap();
    assert!(rules.iter().any(|r| {
        let s = r.as_str().unwrap();
        s.to_lowercase().contains("sodium") && s.contains("clamped")
    }));
}
