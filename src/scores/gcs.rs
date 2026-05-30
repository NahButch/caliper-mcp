//! Glasgow Coma Scale (Teasdale & Jennett, 1974).

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "gcs",
    name: "Glasgow Coma Scale",
    version: "1974",
    citation: "Teasdale G, Jennett B. Assessment of coma and impaired consciousness. Lancet. 1974;2(7872):81-84.",
    domain: "neuro",
    keywords: &["coma", "consciousness", "head injury", "neuro", "trauma", "gcs"],
    unit: "points",
    inputs: INPUTS,
    compute,
};

const INPUTS: &[InputSpec] = &[
    InputSpec::integer("eye", 1, 4, "Eye-opening response (1-4)."),
    InputSpec::integer("verbal", 1, 5, "Verbal response (1-5)."),
    InputSpec::integer("motor", 1, 6, "Motor response (1-6)."),
];

fn interpret(total: i64) -> String {
    let band = match total {
        13..=15 => "mild impairment (GCS 13-15)",
        9..=12 => "moderate impairment (GCS 9-12)",
        _ => "severe impairment (GCS 3-8)",
    };
    format!("Total {total}/15: {band}.")
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let eye = i.integer("eye", 1, 4)?;
    let verbal = i.integer("verbal", 1, 5)?;
    let motor = i.integer("motor", 1, 6)?;
    let total = eye + verbal + motor;
    Ok(ScoreResult::new(
        DESCRIPTOR.id,
        DESCRIPTOR.version,
        total as f64,
        DESCRIPTOR.unit,
        interpret(total),
        vec![format!("components E{eye} V{verbal} M{motor}")],
        DESCRIPTOR.citation,
    ))
}
