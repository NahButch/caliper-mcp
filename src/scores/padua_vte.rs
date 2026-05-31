//! Padua Prediction Score for VTE risk in hospitalized medical patients (Barbar et al., 2010).

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "padua-vte",
    name: "Padua Prediction Score (VTE)",
    version: "Barbar-2010",
    citation: "Barbar S, Noventa F, Rossetto V, et al. A risk assessment model for the identification of hospitalized medical patients at risk for venous thromboembolism: the Padua Prediction Score. J Thromb Haemost. 2010;8(11):2450-2457.",
    domain: "hematology",
    keywords: &["vte", "dvt", "thromboprophylaxis", "venous thromboembolism", "padua", "prophylaxis"],
    unit: "points",
    inputs: INPUTS,
    compute,
};

const INPUTS: &[InputSpec] = &[
    InputSpec::boolean(
        "active_cancer",
        "Active cancer (local/distant metastases and/or chemo/radio within 6 months) (3).",
    ),
    InputSpec::boolean(
        "previous_vte",
        "Previous VTE, excluding superficial vein thrombosis (3).",
    ),
    InputSpec::boolean(
        "reduced_mobility",
        "Reduced mobility (bedrest with bathroom privileges >=3 days) (3).",
    ),
    InputSpec::boolean("thrombophilia", "Known thrombophilic condition (3)."),
    InputSpec::boolean(
        "recent_trauma_surgery",
        "Recent (<=1 month) trauma and/or surgery (2).",
    ),
    InputSpec::quantity("age", "age", "years", &["years"], "Elderly age >=70 (1)."),
    InputSpec::boolean(
        "heart_resp_failure",
        "Heart and/or respiratory failure (1).",
    ),
    InputSpec::boolean(
        "mi_or_stroke",
        "Acute myocardial infarction and/or ischemic stroke (1).",
    ),
    InputSpec::boolean(
        "infection_rheum",
        "Acute infection and/or rheumatologic disorder (1).",
    ),
    InputSpec::boolean("obesity", "Obesity (BMI >=30) (1)."),
    InputSpec::boolean("hormonal_treatment", "Ongoing hormonal treatment (1)."),
];

fn interpret(total: i64) -> String {
    let band = if total >= 4 {
        "high VTE risk (>=4)"
    } else {
        "low VTE risk (<4)"
    };
    format!("Padua {total}: {band} (descriptive only).")
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let mut total = 0i64;
    let mut rules = Vec::new();

    for (field, weight) in [
        ("active_cancer", 3),
        ("previous_vte", 3),
        ("reduced_mobility", 3),
        ("thrombophilia", 3),
        ("recent_trauma_surgery", 2),
        ("heart_resp_failure", 1),
        ("mi_or_stroke", 1),
        ("infection_rheum", 1),
        ("obesity", 1),
        ("hormonal_treatment", 1),
    ] {
        if i.boolean(field)? {
            total += weight;
        }
    }
    let age = i.quantity("age", "age")?;
    if age >= 70.0 {
        total += 1;
        rules.push("age >=70: +1".to_string());
    }

    Ok(ScoreResult::new(
        DESCRIPTOR.id,
        DESCRIPTOR.version,
        total as f64,
        DESCRIPTOR.unit,
        interpret(total),
        rules,
        DESCRIPTOR.citation,
    ))
}
