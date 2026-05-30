//! HAS-BLED major-bleeding-risk score for anticoagulated AF patients (Pisters et al., 2010).

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "has-bled",
    name: "HAS-BLED",
    version: "Pisters-2010",
    citation: "Pisters R, Lane DA, Nieuwlaat R, de Vos CB, Crijns HJGM, Lip GYH. A novel user-friendly score (HAS-BLED) to assess 1-year risk of major bleeding in patients with atrial fibrillation. Chest. 2010;138(5):1093-1100.",
    domain: "cardiology",
    keywords: &["atrial fibrillation", "afib", "bleeding", "anticoagulation", "hemorrhage"],
    unit: "points",
    inputs: INPUTS,
    compute,
};

const INPUTS: &[InputSpec] = &[
    InputSpec::boolean(
        "hypertension",
        "Uncontrolled hypertension, systolic BP >160 mmHg (1).",
    ),
    InputSpec::boolean(
        "abnormal_renal",
        "Abnormal renal function (dialysis, transplant, or creatinine >=2.26 mg/dL) (1).",
    ),
    InputSpec::boolean(
        "abnormal_liver",
        "Abnormal liver function (cirrhosis, or bilirubin >2x ULN with AST/ALT/ALP >3x ULN) (1).",
    ),
    InputSpec::boolean("stroke", "Prior stroke (1)."),
    InputSpec::boolean(
        "bleeding",
        "Prior major bleeding or predisposition to bleeding (1).",
    ),
    InputSpec::boolean(
        "labile_inr",
        "Labile INR / time-in-therapeutic-range <60% (1).",
    ),
    InputSpec::quantity(
        "age",
        "age",
        "years",
        &["years", "months"],
        "Elderly: age >65 (1).",
    ),
    InputSpec::boolean("drugs", "Concomitant antiplatelet or NSAID use (1)."),
    InputSpec::boolean("alcohol", "Alcohol >=8 units/week (1)."),
];

fn interpret(total: i64) -> String {
    let band = match total {
        0..=2 => "low major-bleeding risk",
        _ => "elevated major-bleeding risk",
    };
    format!("Score {total}/9: {band} (descriptive band only).")
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let mut total = 0i64;
    let mut rules = Vec::new();

    for field in [
        "hypertension",
        "abnormal_renal",
        "abnormal_liver",
        "stroke",
        "bleeding",
        "labile_inr",
        "drugs",
        "alcohol",
    ] {
        if i.boolean(field)? {
            total += 1;
        }
    }
    let age = i.quantity("age", "age")?;
    if age > 65.0 {
        total += 1;
        rules.push("age >65: +1".to_string());
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
