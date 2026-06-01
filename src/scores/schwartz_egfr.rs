//! Bedside Schwartz pediatric eGFR (Schwartz et al., 2009).
//!
//! eGFR (mL/min/1.73m^2) = 0.413 * height(cm) / serum creatinine(mg/dL). The 0.413 constant
//! is the IDMS-traceable "bedside" coefficient for children/adolescents (~1-18 years).

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "schwartz-egfr",
    name: "Pediatric eGFR (bedside Schwartz, 2009)",
    version: "Schwartz-2009-bedside",
    citation: "Schwartz GJ, Munoz A, Schneider MF, et al. New equations to estimate GFR in children with CKD. J Am Soc Nephrol. 2009;20(3):629-637.",
    domain: "renal",
    keywords: &["egfr", "kidney", "pediatric", "paediatric", "children", "gfr", "schwartz", "creatinine"],
    unit: "mL/min/1.73m^2",
    inputs: INPUTS,
    compute,
};

const INPUTS: &[InputSpec] = &[
    InputSpec::quantity(
        "height",
        "height",
        "cm",
        &["cm", "m", "in"],
        "Standing height / recumbent length.",
    ),
    InputSpec::quantity(
        "creatinine",
        "creatinine",
        "mg/dL",
        &["mg/dL", "umol/L"],
        "Serum creatinine (IDMS-traceable enzymatic assay assumed by the 0.413 constant).",
    ),
];

fn interpret(egfr: f64) -> String {
    let stage = if egfr >= 90.0 {
        "G1 (normal/high)"
    } else if egfr >= 60.0 {
        "G2 (mildly decreased)"
    } else if egfr >= 45.0 {
        "G3a (mild-moderate)"
    } else if egfr >= 30.0 {
        "G3b (moderate-severe)"
    } else if egfr >= 15.0 {
        "G4 (severely decreased)"
    } else {
        "G5 (kidney failure)"
    };
    format!("eGFR {egfr:.1} mL/min/1.73m^2: KDIGO {stage} (descriptive only).")
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let height = i.quantity("height", "height")?;
    let scr = i.quantity("creatinine", "creatinine")?;

    if scr <= 0.0 {
        return Err(CalcError::OutOfRange {
            field: "creatinine".to_string(),
            message: "creatinine must be positive".to_string(),
        });
    }
    if height <= 0.0 {
        return Err(CalcError::OutOfRange {
            field: "height".to_string(),
            message: "height must be positive".to_string(),
        });
    }

    let egfr = 0.413 * height / scr;

    Ok(ScoreResult::new(
        DESCRIPTOR.id,
        DESCRIPTOR.version,
        egfr,
        DESCRIPTOR.unit,
        interpret(egfr),
        vec![format!(
            "0.413 * height_cm({height}) / creatinine_mgdl({scr})"
        )],
        DESCRIPTOR.citation,
    ))
}
