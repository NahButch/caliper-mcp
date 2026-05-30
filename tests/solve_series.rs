//! Tests for solve_for (bisection over one input) and score_series (trend over time).

use caliper::tools::call_tool;
use serde_json::json;

#[test]
fn solve_for_egfr_threshold() {
    let out = call_tool(
        "solve_for",
        &json!({
            "id": "ckd-epi-2021",
            "target": 60.0,
            "solve": { "field": "creatinine" },
            "fixed": { "age": { "value": 50, "unit": "years" }, "sex": "female" }
        }),
    );
    assert!(!out.is_error, "solve_for errored: {}", out.value);
    let scr = out.value["threshold"]["value"].as_f64().unwrap();

    let check = call_tool(
        "compute_score",
        &json!({
            "id": "ckd-epi-2021",
            "inputs": {
                "creatinine": { "value": scr, "unit": "mg/dL" },
                "age": { "value": 50, "unit": "years" },
                "sex": "female"
            }
        }),
    );
    let egfr = check.value["value"].as_f64().unwrap();
    assert!((egfr - 60.0).abs() < 0.05, "recomputed eGFR {egfr}");
}

#[test]
fn solve_for_meld_na_threshold() {
    // With creatinine 1.9 / INR 1.5 / Na 130, the bilirubin-1.0 floor puts the minimum
    // achievable MELD-Na at 22, so 28 is a reachable target on the bilirubin axis.
    let target = 28.0;
    let out = call_tool(
        "solve_for",
        &json!({
            "id": "meld-na",
            "target": target,
            "solve": { "field": "bilirubin" },
            "fixed": {
                "creatinine": { "value": 1.9, "unit": "mg/dL" },
                "inr": 1.5,
                "sodium": { "value": 130, "unit": "mmol/L" }
            }
        }),
    );
    assert!(!out.is_error, "solve_for errored: {}", out.value);
    let bili = out.value["threshold"]["value"].as_f64().unwrap();
    let check = call_tool(
        "compute_score",
        &json!({
            "id": "meld-na",
            "inputs": {
                "creatinine": { "value": 1.9, "unit": "mg/dL" },
                "bilirubin": { "value": bili, "unit": "mg/dL" },
                "inr": 1.5,
                "sodium": { "value": 130, "unit": "mmol/L" }
            }
        }),
    );
    assert_eq!(check.value["value"].as_f64().unwrap(), target);
}

#[test]
fn score_series_sofa_multiday() {
    let day1 = json!({
        "pao2": { "value": 75, "unit": "mmHg" },
        "fio2": { "value": 0.6, "unit": "fraction" },
        "respiratory_support": true,
        "platelets": { "value": 80, "unit": "10^9/L" },
        "bilirubin": { "value": 3.0, "unit": "mg/dL" },
        "map": { "value": 65, "unit": "mmHg" },
        "vasopressor": "none",
        "gcs": 13,
        "creatinine": { "value": 2.5, "unit": "mg/dL" }
    });
    // Day 2 worsens: vasopressor up (cardio 1->3) and platelets down (coag 2->3): 11 -> 14.
    let mut day2 = day1.clone();
    day2["vasopressor"] = json!("dopamine_gt5_or_epi_le0.1_or_norepi_le0.1");
    day2["platelets"] = json!({ "value": 40, "unit": "10^9/L" });
    // Day 3 improves markedly.
    let day3 = json!({
        "pao2": { "value": 95, "unit": "mmHg" },
        "fio2": { "value": 0.3, "unit": "fraction" },
        "respiratory_support": false,
        "platelets": { "value": 200, "unit": "10^9/L" },
        "bilirubin": { "value": 1.0, "unit": "mg/dL" },
        "map": { "value": 80, "unit": "mmHg" },
        "vasopressor": "none",
        "gcs": 15,
        "creatinine": { "value": 1.0, "unit": "mg/dL" }
    });

    let out = call_tool(
        "score_series",
        &json!({
            "id": "sofa",
            "series": [
                { "t": "day1", "inputs": day1 },
                { "t": "day2", "inputs": day2 },
                { "t": "day3", "inputs": day3 }
            ]
        }),
    );
    assert!(!out.is_error);
    let points = out.value["points"].as_array().unwrap();
    assert_eq!(points.len(), 3);
    assert_eq!(points[0]["value"].as_f64().unwrap(), 11.0);
    assert_eq!(points[1]["value"].as_f64().unwrap(), 14.0);
    assert_eq!(points[1]["delta"].as_f64().unwrap(), 3.0);
    assert_eq!(points[1]["trend"].as_str().unwrap(), "rising");
    assert_eq!(out.value["overall"]["trend"].as_str().unwrap(), "falling");
}
