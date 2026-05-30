//! qSOFA (quick SOFA) bedside sepsis risk (Sepsis-3, Seymour et al., 2016).

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "qsofa",
    name: "qSOFA",
    version: "Sepsis-3-2016",
    citation: "Seymour CW, Liu VX, Iwashyna TJ, et al. Assessment of clinical criteria for sepsis (Sepsis-3). JAMA. 2016;315(8):762-774.",
    domain: "icu",
    keywords: &["sepsis", "infection", "qsofa", "mortality", "deterioration"],
    unit: "points",
    inputs: INPUTS,
    compute,
};

const INPUTS: &[InputSpec] = &[
    InputSpec::quantity(
        "respiratory_rate",
        "rate_breaths",
        "breaths/min",
        &["breaths/min", "/min"],
        "Respiratory rate >=22 (1).",
    ),
    InputSpec::boolean("altered_mentation", "Altered mentation, GCS <15 (1)."),
    InputSpec::quantity(
        "systolic_bp",
        "pressure",
        "mmHg",
        &["mmHg"],
        "Systolic BP <=100 mmHg (1).",
    ),
];

fn interpret(total: i64) -> String {
    let band = if total >= 2 {
        "qSOFA >=2: higher risk of poor outcome from suspected infection"
    } else {
        "qSOFA <2: lower risk band"
    };
    format!("Score {total}/3: {band} (descriptive only).")
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let mut total = 0i64;
    let mut rules = Vec::new();

    let rr = i.quantity("respiratory_rate", "rate_breaths")?;
    if rr >= 22.0 {
        total += 1;
        rules.push("respiratory rate >=22: +1".to_string());
    }
    if i.boolean("altered_mentation")? {
        total += 1;
    }
    let sbp = i.quantity("systolic_bp", "pressure")?;
    if sbp <= 100.0 {
        total += 1;
        rules.push("systolic BP <=100: +1".to_string());
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
