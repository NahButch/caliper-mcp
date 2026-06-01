//! APGAR score for newborn condition (Apgar, 1953).
//!
//! Five signs, each scored 0/1/2; total 0-10. Caliper takes the five component points
//! directly (the assessor maps the clinical sign to 0/1/2 per the original definition,
//! documented per field) and sums them. Descriptive band only.

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "apgar",
    name: "APGAR Score",
    version: "Apgar-1953",
    citation: "Apgar V. A proposal for a new method of evaluation of the newborn infant. Curr Res Anesth Analg. 1953;32(4):260-267.",
    domain: "neonatology",
    keywords: &["newborn", "neonate", "neonatal", "birth", "delivery", "resuscitation", "apgar"],
    unit: "points",
    inputs: INPUTS,
    compute,
};

const INPUTS: &[InputSpec] = &[
    InputSpec::integer(
        "appearance",
        0,
        2,
        "Appearance/colour: blue or pale all over=0, acrocyanosis (body pink, extremities blue)=1, completely pink=2.",
    ),
    InputSpec::integer(
        "pulse",
        0,
        2,
        "Pulse/heart rate: absent=0, <100/min=1, >=100/min=2.",
    ),
    InputSpec::integer(
        "grimace",
        0,
        2,
        "Grimace/reflex irritability: no response=0, grimace=1, cry/cough/sneeze (active withdrawal)=2.",
    ),
    InputSpec::integer(
        "activity",
        0,
        2,
        "Activity/muscle tone: limp=0, some flexion=1, active motion=2.",
    ),
    InputSpec::integer(
        "respiration",
        0,
        2,
        "Respiration: absent=0, weak/irregular/gasping=1, good/crying=2.",
    ),
];

fn interpret(total: i64) -> String {
    let band = match total {
        7..=10 => "reassuring (7-10)",
        4..=6 => "moderately abnormal (4-6)",
        _ => "low (0-3)",
    };
    format!("Total {total}/10: {band} (descriptive band only).")
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let appearance = i.integer("appearance", 0, 2)?;
    let pulse = i.integer("pulse", 0, 2)?;
    let grimace = i.integer("grimace", 0, 2)?;
    let activity = i.integer("activity", 0, 2)?;
    let respiration = i.integer("respiration", 0, 2)?;
    let total = appearance + pulse + grimace + activity + respiration;
    Ok(ScoreResult::new(
        DESCRIPTOR.id,
        DESCRIPTOR.version,
        total as f64,
        DESCRIPTOR.unit,
        interpret(total),
        vec![format!(
            "components A{appearance} P{pulse} G{grimace} A{activity} R{respiration}"
        )],
        DESCRIPTOR.citation,
    ))
}
