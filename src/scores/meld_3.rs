//! MELD 3.0 (Kim et al., 2021; adopted by OPTN in 2023).

use crate::{apply_clamp, apply_floor, CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "meld-3",
    name: "MELD 3.0",
    version: "OPTN-2023",
    citation: "Kim WR, Mannalithara A, Heimbach JK, et al. MELD 3.0: the model for end-stage liver disease updated for the modern era. Gastroenterology. 2021;161(6):1887-1895; OPTN Policy update effective 2023.",
    domain: "hepatology",
    keywords: &["meld", "meld 3.0", "liver transplant", "cirrhosis", "albumin", "sodium"],
    unit: "points",
    inputs: INPUTS,
    compute,
};

const SEX: &[&str] = &["male", "female"];

const INPUTS: &[InputSpec] = &[
    InputSpec::enumerated("sex", SEX, "Sex: female adds 1.33 to the linear predictor."),
    InputSpec::quantity(
        "creatinine",
        "creatinine",
        "mg/dL",
        &["mg/dL", "umol/L"],
        "Serum creatinine; floored to 1.0, capped at 3.0; dialysis override sets 3.0.",
    )
    .with_floor(1.0)
    .with_ceiling(3.0),
    InputSpec::quantity(
        "bilirubin",
        "bilirubin",
        "mg/dL",
        &["mg/dL", "umol/L"],
        "Total bilirubin; floored to 1.0.",
    )
    .with_floor(1.0),
    InputSpec::ratio("inr", "INR; floored to 1.0."),
    InputSpec::quantity(
        "sodium",
        "sodium",
        "mmol/L",
        &["mmol/L", "mEq/L"],
        "Serum sodium; clamped to [125, 137].",
    )
    .with_floor(125.0)
    .with_ceiling(137.0),
    InputSpec::quantity(
        "albumin",
        "albumin",
        "g/dL",
        &["g/dL", "g/L"],
        "Albumin; clamped to [1.5, 3.5].",
    )
    .with_floor(1.5)
    .with_ceiling(3.5),
    InputSpec::boolean(
        "dialysis",
        "Two or more dialysis sessions (or 24h CVVHD) in the prior 7 days: sets creatinine to 3.0.",
    )
    .optional(),
];

fn interpret(score: f64) -> String {
    let band = match score as i64 {
        ..=9 => "lower 90-day mortality band",
        10..=19 => "intermediate band",
        20..=29 => "elevated band",
        30..=39 => "high band",
        _ => "very high band",
    };
    format!("MELD 3.0 {score}: {band} (descriptive only).")
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let mut rules = Vec::new();

    let female = i.enum_one("sex", SEX)? == "female";
    let mut cr = i.quantity("creatinine", "creatinine")?;
    let bili = i.quantity("bilirubin", "bilirubin")?;
    let inr = i.ratio("inr")?;
    let na = i.quantity("sodium", "sodium")?;
    let alb = i.quantity("albumin", "albumin")?;
    let dialysis = i.raw("dialysis").is_some() && i.boolean("dialysis")?;

    if dialysis {
        cr = 3.0;
        rules.push("dialysis override: creatinine set to 3.0".to_string());
    }
    cr = apply_floor(cr, 1.0, "creatinine", &mut rules);
    cr = crate::apply_ceiling(cr, 3.0, "creatinine", &mut rules);
    let bili = apply_floor(bili, 1.0, "bilirubin", &mut rules);
    let inr = apply_floor(inr, 1.0, "INR", &mut rules);
    let na = apply_clamp(na, 125.0, 137.0, "sodium", &mut rules);
    let alb = apply_clamp(alb, 1.5, 3.5, "albumin", &mut rules);

    let sex_term = if female {
        rules.push("female: +1.33".to_string());
        1.33
    } else {
        0.0
    };

    let predictor = sex_term + 4.56 * bili.ln() + 0.82 * (137.0 - na)
        - 0.24 * (137.0 - na) * bili.ln()
        + 9.09 * inr.ln()
        + 11.14 * cr.ln()
        + 1.85 * (3.5 - alb)
        - 1.83 * (3.5 - alb) * cr.ln()
        + 6.0;

    let score = apply_clamp(predictor.round(), 6.0, 40.0, "MELD 3.0", &mut rules);

    Ok(ScoreResult::new(
        DESCRIPTOR.id,
        DESCRIPTOR.version,
        score,
        DESCRIPTOR.unit,
        interpret(score),
        rules,
        DESCRIPTOR.citation,
    ))
}
