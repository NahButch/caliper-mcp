//! PERC — Pulmonary Embolism Rule-out Criteria (Kline et al., 2004).
//!
//! PERC is applied only to patients already judged LOW pretest probability. If all eight
//! criteria are negative (count 0), PE can be ruled out without further testing. This tool
//! reports the count of POSITIVE criteria and whether the rule-out condition is met; it does
//! not assess pretest probability and does not recommend testing.

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "perc",
    name: "PERC Rule",
    version: "Kline-2004",
    citation: "Kline JA, Mitchell AM, Kabrhel C, Richman PB, Courtney DM. Clinical criteria to prevent unnecessary diagnostic testing in emergency department patients with suspected pulmonary embolism. J Thromb Haemost. 2004;2(8):1247-1255.",
    domain: "pulmonary",
    keywords: &["pulmonary embolism", "pe", "rule out", "perc", "d-dimer", "low risk"],
    unit: "positive criteria",
    inputs: INPUTS,
    compute,
};

const INPUTS: &[InputSpec] = &[
    InputSpec::quantity("age", "age", "years", &["years"], "Positive if age >=50."),
    InputSpec::quantity(
        "heart_rate",
        "rate_beats",
        "bpm",
        &["bpm", "beats/min"],
        "Positive if heart rate >=100.",
    ),
    InputSpec::quantity(
        "spo2",
        "spo2",
        "%",
        &["%"],
        "Positive if SpO2 on room air <95%.",
    ),
    InputSpec::boolean(
        "unilateral_leg_swelling",
        "Positive if unilateral leg swelling present.",
    ),
    InputSpec::boolean("hemoptysis", "Positive if hemoptysis present."),
    InputSpec::boolean(
        "recent_surgery_trauma",
        "Positive if surgery or trauma requiring hospitalization in the prior 4 weeks.",
    ),
    InputSpec::boolean("prior_vte", "Positive if prior DVT or PE."),
    InputSpec::boolean(
        "hormone_use",
        "Positive if exogenous estrogen use (oral contraceptives, HRT).",
    ),
];

fn interpret(positive: i64) -> String {
    if positive == 0 {
        "0 positive criteria: PERC-negative (rule-out criteria met for a low pretest-probability patient).".to_string()
    } else {
        format!("{positive} positive criterion(s): PERC-positive (rule-out criteria NOT met). Descriptive only.")
    }
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let mut positive = 0i64;
    let mut rules = Vec::new();

    let age = i.quantity("age", "age")?;
    if age >= 50.0 {
        positive += 1;
        rules.push("age >=50: positive".to_string());
    }
    let hr = i.quantity("heart_rate", "rate_beats")?;
    if hr >= 100.0 {
        positive += 1;
        rules.push("heart rate >=100: positive".to_string());
    }
    let spo2 = i.quantity("spo2", "spo2")?;
    if spo2 < 95.0 {
        positive += 1;
        rules.push("SpO2 <95%: positive".to_string());
    }
    for field in [
        "unilateral_leg_swelling",
        "hemoptysis",
        "recent_surgery_trauma",
        "prior_vte",
        "hormone_use",
    ] {
        if i.boolean(field)? {
            positive += 1;
        }
    }

    Ok(ScoreResult::new(
        DESCRIPTOR.id,
        DESCRIPTOR.version,
        positive as f64,
        DESCRIPTOR.unit,
        interpret(positive),
        rules,
        DESCRIPTOR.citation,
    ))
}
