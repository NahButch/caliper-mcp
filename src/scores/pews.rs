//! PEWS — Brighton Pediatric Early Warning Score (Monaghan, 2005).
//!
//! Three component subscores (behaviour, cardiovascular, respiratory), each 0-3, plus an
//! optional +2 modifier (quarter-hourly nebulisers or persistent post-operative vomiting);
//! total 0-11. Several institutional PEWS variants exist (different parameter grids and
//! age-banded vital thresholds); Caliper implements the original Brighton three-component
//! grid and takes the per-component levels directly. See docs/COEFFICIENT_AUDIT.md for the
//! variant note (cf. the MEWS caveat).

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "pews",
    name: "Pediatric Early Warning Score (Brighton)",
    version: "Brighton-2005",
    citation: "Monaghan A. Detecting and managing deterioration in children. Paediatr Nurs. 2005;17(1):32-35.",
    domain: "pediatrics",
    keywords: &["early warning", "deterioration", "pediatric", "paediatric", "pews", "children", "ward"],
    unit: "points",
    inputs: INPUTS,
    compute,
};

const INPUTS: &[InputSpec] = &[
    InputSpec::integer(
        "behaviour",
        0,
        3,
        "Behaviour: playing/appropriate=0, sleeping=1, irritable=2, lethargic/confused or reduced response to pain=3.",
    ),
    InputSpec::integer(
        "cardiovascular",
        0,
        3,
        "Cardiovascular (colour / cap refill / heart rate vs age-normal): per Brighton grid, 0-3.",
    ),
    InputSpec::integer(
        "respiratory",
        0,
        3,
        "Respiratory (rate vs age-normal / accessory muscle use / FiO2 need): per Brighton grid, 0-3.",
    ),
    InputSpec::boolean(
        "nebuliser_or_postop_vomiting",
        "Modifier: +2 if quarter-hourly (15-min) nebulisers OR persistent post-operative vomiting.",
    )
    .optional(),
];

fn interpret(total: i64) -> String {
    let band = match total {
        0..=2 => "low (0-2)",
        3..=4 => "intermediate (3-4)",
        _ => "high (>=5)",
    };
    format!("Score {total}/11: {band} (descriptive band only; institutional escalation thresholds vary).")
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let behaviour = i.integer("behaviour", 0, 3)?;
    let cardiovascular = i.integer("cardiovascular", 0, 3)?;
    let respiratory = i.integer("respiratory", 0, 3)?;
    let modifier = i.raw("nebuliser_or_postop_vomiting").is_some()
        && i.boolean("nebuliser_or_postop_vomiting")?;

    let mut rules = vec![format!(
        "components behaviour {behaviour}, cardiovascular {cardiovascular}, respiratory {respiratory}"
    )];
    let mut total = behaviour + cardiovascular + respiratory;
    if modifier {
        total += 2;
        rules.push("nebuliser/post-op vomiting modifier: +2".to_string());
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
