//! CRB-65 community-acquired pneumonia severity score (Lim et al., 2003).
//!
//! CRB-65 is the urea-free variant of CURB-65, for settings where blood urea is not
//! available (e.g. primary care). Same derivation paper.

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "crb-65",
    name: "CRB-65",
    version: "Lim-2003",
    citation: "Lim WS, van der Eerden MM, Laing R, et al. Defining community acquired pneumonia severity on presentation to hospital: an international derivation and validation study. Thorax. 2003;58(5):377-382.",
    domain: "pulmonary",
    keywords: &["pneumonia", "cap", "respiratory infection", "severity", "primary care"],
    unit: "points",
    inputs: INPUTS,
    compute,
};

const INPUTS: &[InputSpec] = &[
    InputSpec::boolean("confusion", "New-onset confusion / AMT <=8 (1)."),
    InputSpec::quantity(
        "respiratory_rate",
        "rate_breaths",
        "breaths/min",
        &["breaths/min", "/min"],
        "Respiratory rate >=30 (1).",
    ),
    InputSpec::quantity(
        "systolic_bp",
        "pressure",
        "mmHg",
        &["mmHg"],
        "Systolic BP <90 mmHg contributes to the BP criterion (1).",
    ),
    InputSpec::quantity(
        "diastolic_bp",
        "pressure",
        "mmHg",
        &["mmHg"],
        "Diastolic BP <=60 mmHg contributes to the BP criterion (1).",
    )
    .optional(),
    InputSpec::quantity("age", "age", "years", &["years", "months"], "Age >=65 (1)."),
];

fn interpret(total: i64) -> String {
    let band = match total {
        0 => "low severity",
        1..=2 => "intermediate severity",
        _ => "high severity",
    };
    format!("Score {total}/4: {band} (descriptive band only).")
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let mut total = 0i64;
    let mut rules = Vec::new();

    if i.boolean("confusion")? {
        total += 1;
    }
    let rr = i.quantity("respiratory_rate", "rate_breaths")?;
    if rr >= 30.0 {
        total += 1;
        rules.push("respiratory rate >=30: +1".to_string());
    }
    let sbp = i.quantity("systolic_bp", "pressure")?;
    let dbp = i.opt_quantity("diastolic_bp", "pressure")?;
    if sbp < 90.0 || dbp.map(|d| d <= 60.0).unwrap_or(false) {
        total += 1;
        rules.push("blood pressure low (SBP <90 or DBP <=60): +1".to_string());
    }
    let age = i.quantity("age", "age")?;
    if age >= 65.0 {
        total += 1;
        rules.push("age >=65: +1".to_string());
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
