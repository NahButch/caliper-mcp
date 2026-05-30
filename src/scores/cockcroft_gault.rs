//! Cockcroft-Gault creatinine clearance (Cockcroft & Gault, 1976).

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "cockcroft-gault",
    name: "Cockcroft-Gault creatinine clearance",
    version: "Cockcroft-Gault-1976",
    citation: "Cockcroft DW, Gault MH. Prediction of creatinine clearance from serum creatinine. Nephron. 1976;16(1):31-41.",
    domain: "renal",
    keywords: &["creatinine clearance", "crcl", "kidney", "drug dosing", "renal"],
    unit: "mL/min",
    inputs: INPUTS,
    compute,
};

const SEX: &[&str] = &["male", "female"];

const INPUTS: &[InputSpec] = &[
    InputSpec::quantity("age", "age", "years", &["years"], "Age in years."),
    InputSpec::quantity("weight", "weight", "kg", &["kg", "lb"], "Body weight."),
    InputSpec::quantity(
        "creatinine",
        "creatinine",
        "mg/dL",
        &["mg/dL", "umol/L"],
        "Serum creatinine.",
    ),
    InputSpec::enumerated("sex", SEX, "Sex (female multiplies result by 0.85)."),
];

fn interpret(crcl: f64) -> String {
    format!("Estimated creatinine clearance {crcl:.1} mL/min (uncorrected for body surface area; descriptive only).")
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let age = i.quantity("age", "age")?;
    let weight = i.quantity("weight", "weight")?;
    let scr = i.quantity("creatinine", "creatinine")?;
    let female = i.enum_one("sex", SEX)? == "female";

    if scr <= 0.0 {
        return Err(CalcError::OutOfRange {
            field: "creatinine".to_string(),
            message: "creatinine must be positive".to_string(),
        });
    }

    let mut crcl = ((140.0 - age) * weight) / (72.0 * scr);
    let mut rules = Vec::new();
    if female {
        crcl *= 0.85;
        rules.push("female: x0.85".to_string());
    }

    Ok(ScoreResult::new(
        DESCRIPTOR.id,
        DESCRIPTOR.version,
        crcl,
        DESCRIPTOR.unit,
        interpret(crcl),
        rules,
        DESCRIPTOR.citation,
    ))
}
