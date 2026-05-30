//! MELD-Na (OPTN/UNOS, effective January 2016).

use crate::{apply_clamp, apply_floor, CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "meld-na",
    name: "MELD-Na",
    version: "OPTN-2016",
    citation: "OPTN Policy 9 (MELD-Na, effective 2016-01-11); Kim WR, et al. Hyponatremia and mortality among patients on the liver-transplant waiting list. N Engl J Med. 2008;359(10):1018-1026.",
    domain: "hepatology",
    keywords: &["meld", "liver transplant", "cirrhosis", "mortality", "sodium", "hepatic"],
    unit: "points",
    inputs: INPUTS,
    compute,
};

const INPUTS: &[InputSpec] = &[
    InputSpec::quantity(
        "creatinine",
        "creatinine",
        "mg/dL",
        &["mg/dL", "umol/L"],
        "Serum creatinine; floored to 1.0, capped at 4.0; dialysis override sets 4.0.",
    )
    .with_floor(1.0)
    .with_ceiling(4.0),
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
    InputSpec::boolean(
        "dialysis",
        "Two or more dialysis sessions (or 24h CVVHD) in the prior 7 days: sets creatinine to 4.0.",
    )
    .optional(),
];

fn interpret(score: f64) -> String {
    let band = match score as i64 {
        ..=9 => "lower 3-month mortality band",
        10..=19 => "intermediate band",
        20..=29 => "elevated band",
        30..=39 => "high band",
        _ => "very high band",
    };
    format!("MELD-Na {score}: {band} (descriptive only).")
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let mut rules = Vec::new();

    let mut cr = i.quantity("creatinine", "creatinine")?;
    let bili = i.quantity("bilirubin", "bilirubin")?;
    let inr = i.ratio("inr")?;
    let na = i.quantity("sodium", "sodium")?;
    let dialysis = i.raw("dialysis").is_some() && i.boolean("dialysis")?;

    // Dialysis override precedes the cap.
    if dialysis {
        cr = 4.0;
        rules.push("dialysis override: creatinine set to 4.0".to_string());
    }
    cr = apply_floor(cr, 1.0, "creatinine", &mut rules);
    cr = crate::apply_ceiling(cr, 4.0, "creatinine", &mut rules);
    let bili = apply_floor(bili, 1.0, "bilirubin", &mut rules);
    let inr = apply_floor(inr, 1.0, "INR", &mut rules);
    let na = apply_clamp(na, 125.0, 137.0, "sodium", &mut rules);

    // Lab MELD(i), x10, rounded.
    let meld_i_raw = 0.957 * cr.ln() + 0.378 * bili.ln() + 1.120 * inr.ln() + 0.643;
    let meld_i = (meld_i_raw * 10.0).round();

    let meld_na = if meld_i > 11.0 {
        let adjusted = meld_i + 1.32 * (137.0 - na) - 0.033 * meld_i * (137.0 - na);
        rules.push("MELD(i) > 11: sodium correction applied".to_string());
        adjusted
    } else {
        rules.push("MELD(i) <= 11: no sodium correction".to_string());
        meld_i
    };

    let score = apply_clamp(meld_na.round(), 6.0, 40.0, "MELD-Na", &mut rules);

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
