//! Wells score for pulmonary embolism (Wells et al., 2000).

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "wells-pe",
    name: "Wells Criteria (PE)",
    version: "Wells-2000",
    citation: "Wells PS, Anderson DR, Rodger M, et al. Derivation of a simple clinical model to categorize patients probability of pulmonary embolism. Thromb Haemost. 2000;83(3):416-420.",
    domain: "pulmonary",
    keywords: &["pulmonary embolism", "pe", "vte", "pretest probability", "d-dimer"],
    unit: "points",
    inputs: INPUTS,
    compute,
};

const INPUTS: &[InputSpec] = &[
    InputSpec::boolean("clinical_dvt", "Clinical signs/symptoms of DVT (3)."),
    InputSpec::boolean(
        "pe_most_likely",
        "PE is the most likely diagnosis / alternative less likely (3).",
    ),
    InputSpec::quantity(
        "heart_rate",
        "rate_beats",
        "bpm",
        &["bpm", "beats/min"],
        "Heart rate >100 bpm (1.5).",
    ),
    InputSpec::boolean(
        "immobilization_surgery",
        "Immobilization >=3 days or surgery in previous 4 weeks (1.5).",
    ),
    InputSpec::boolean(
        "prior_dvt_pe",
        "Previous objectively diagnosed DVT or PE (1.5).",
    ),
    InputSpec::boolean("hemoptysis", "Hemoptysis (1)."),
    InputSpec::boolean(
        "malignancy",
        "Active malignancy (treatment within 6 months or palliative) (1).",
    ),
];

fn interpret(total: f64) -> String {
    let three_tier = if total < 2.0 {
        "low"
    } else if total <= 6.0 {
        "moderate"
    } else {
        "high"
    };
    let two_tier = if total <= 4.0 {
        "PE unlikely"
    } else {
        "PE likely"
    };
    format!(
        "Score {total}: three-tier pretest probability = {three_tier}; two-tier = {two_tier} (descriptive only)."
    )
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let mut total = 0.0f64;
    let mut rules = Vec::new();

    if i.boolean("clinical_dvt")? {
        total += 3.0;
    }
    if i.boolean("pe_most_likely")? {
        total += 3.0;
    }
    let hr = i.quantity("heart_rate", "rate_beats")?;
    if hr > 100.0 {
        total += 1.5;
        rules.push("heart rate >100: +1.5".to_string());
    }
    if i.boolean("immobilization_surgery")? {
        total += 1.5;
    }
    if i.boolean("prior_dvt_pe")? {
        total += 1.5;
    }
    if i.boolean("hemoptysis")? {
        total += 1.0;
    }
    if i.boolean("malignancy")? {
        total += 1.0;
    }

    Ok(ScoreResult::new(
        DESCRIPTOR.id,
        DESCRIPTOR.version,
        total,
        DESCRIPTOR.unit,
        interpret(total),
        rules,
        DESCRIPTOR.citation,
    ))
}
