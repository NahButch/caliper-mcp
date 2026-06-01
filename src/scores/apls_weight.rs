//! APLS / Luscombe age-based weight estimation for children (APLS 2011; Luscombe & Owens 2007).
//!
//! Three age bands, each a linear estimate of body weight from age. This is an *estimate* of an
//! unmeasured quantity (e.g. when a child cannot be weighed in an emergency), not a clinical
//! grading score. It is a calculation only and never a dosing directive; confirm by measurement
//! where possible.

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "apls-weight",
    name: "Pediatric weight estimate (APLS, age-based)",
    version: "APLS-2011",
    citation: "Advanced Paediatric Life Support, 5th ed. (2011), age-based weight formulae; cf. Luscombe MD, Owens BD. Weight estimation in resuscitation: is the current formula still valid? Arch Dis Child. 2007;92(5):412-415.",
    domain: "pediatrics",
    keywords: &["weight estimation", "pediatric", "paediatric", "children", "resuscitation", "apls", "broselow"],
    unit: "kg",
    inputs: INPUTS,
    compute,
};

const INPUTS: &[InputSpec] = &[InputSpec::quantity(
    "age",
    "age",
    "years",
    &["years", "months"],
    "Age (0-12 years). <1y uses (0.5 x months)+4; 1-5y uses (2 x years)+8; 6-12y uses (3 x years)+7.",
)];

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let age_years = i.quantity("age", "age")?;
    if age_years < 0.0 {
        return Err(CalcError::OutOfRange {
            field: "age".to_string(),
            message: "age cannot be negative".to_string(),
        });
    }
    if age_years > 12.0 {
        return Err(CalcError::OutOfRange {
            field: "age".to_string(),
            message: "APLS age-based weight estimation is defined for ages 0-12 years".to_string(),
        });
    }

    let (weight, rule) = if age_years < 1.0 {
        let months = age_years * 12.0;
        (
            (0.5 * months) + 4.0,
            format!("infant band: (0.5 x {months} months) + 4"),
        )
    } else if age_years < 6.0 {
        (
            (2.0 * age_years) + 8.0,
            format!("1-5y band: (2 x {age_years}) + 8"),
        )
    } else {
        (
            (3.0 * age_years) + 7.0,
            format!("6-12y band: (3 x {age_years}) + 7"),
        )
    };

    let interpretation = format!(
        "Estimated body weight {weight:.1} kg for a child aged {age_years} years \
         (APLS age-based estimate; confirm by measurement where possible)."
    );

    Ok(ScoreResult::new(
        DESCRIPTOR.id,
        DESCRIPTOR.version,
        weight,
        DESCRIPTOR.unit,
        interpretation,
        vec![rule],
        DESCRIPTOR.citation,
    ))
}
