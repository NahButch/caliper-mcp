//! CHA2DS2-VASc stroke-risk score for atrial fibrillation (Lip et al., 2010).

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "cha2ds2-vasc",
    name: "CHA2DS2-VASc",
    version: "Lip-2010",
    citation: "Lip GYH, Nieuwlaat R, Pisters R, Lane DA, Crijns HJGM. Refining clinical risk stratification for predicting stroke and thromboembolism in atrial fibrillation. Chest. 2010;137(2):263-272.",
    domain: "cardiology",
    keywords: &["atrial fibrillation", "afib", "stroke", "anticoagulation", "thromboembolism"],
    unit: "points",
    inputs: INPUTS,
    compute,
};

const SEX: &[&str] = &["male", "female"];

const INPUTS: &[InputSpec] = &[
    InputSpec::boolean("chf", "Congestive heart failure / LV dysfunction (1)."),
    InputSpec::boolean("hypertension", "History of hypertension (1)."),
    InputSpec::quantity(
        "age",
        "age",
        "years",
        &["years", "months"],
        "Age: 2 if >=75, 1 if 65-74.",
    ),
    InputSpec::boolean("diabetes", "Diabetes mellitus (1)."),
    InputSpec::boolean("stroke_tia", "Prior stroke / TIA / thromboembolism (2)."),
    InputSpec::boolean(
        "vascular_disease",
        "Vascular disease: prior MI, PAD, or aortic plaque (1).",
    ),
    InputSpec::enumerated("sex", SEX, "Sex category: female adds 1."),
];

fn interpret(total: i64) -> String {
    let band = match total {
        0 => "low annual stroke risk",
        1 => "low-to-intermediate annual stroke risk",
        _ => "elevated annual stroke risk",
    };
    format!("Score {total}/9: {band} (descriptive band only).")
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let mut total = 0i64;
    let mut rules = Vec::new();

    if i.boolean("chf")? {
        total += 1;
    }
    if i.boolean("hypertension")? {
        total += 1;
    }
    let age = i.quantity("age", "age")?;
    if age >= 75.0 {
        total += 2;
        rules.push("age >=75: +2".to_string());
    } else if age >= 65.0 {
        total += 1;
        rules.push("age 65-74: +1".to_string());
    }
    if i.boolean("diabetes")? {
        total += 1;
    }
    if i.boolean("stroke_tia")? {
        total += 2;
    }
    if i.boolean("vascular_disease")? {
        total += 1;
    }
    if i.enum_one("sex", SEX)? == "female" {
        total += 1;
        rules.push("female sex: +1".to_string());
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
