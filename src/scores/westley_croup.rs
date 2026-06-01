//! Westley Croup Score for croup severity (Westley et al., 1978).
//!
//! Five clinical components with unevenly weighted levels; total 0-17. Descriptive band only.

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "westley-croup",
    name: "Westley Croup Score",
    version: "Westley-1978",
    citation: "Westley CR, Cotton EK, Brooks JG. Nebulized racemic epinephrine by IPPB for the treatment of croup: a double-blind study. Am J Dis Child. 1978;132(5):484-487.",
    domain: "pulmonary",
    keywords: &["croup", "stridor", "laryngotracheobronchitis", "pediatric", "paediatric", "westley"],
    unit: "points",
    inputs: INPUTS,
    compute,
};

const CONSCIOUSNESS: &[&str] = &["normal", "disoriented"];
const CYANOSIS: &[&str] = &["none", "with_agitation", "at_rest"];
const STRIDOR: &[&str] = &["none", "with_agitation", "at_rest"];
const AIR_ENTRY: &[&str] = &["normal", "decreased", "markedly_decreased"];
const RETRACTIONS: &[&str] = &["none", "mild", "moderate", "severe"];

const INPUTS: &[InputSpec] = &[
    InputSpec::enumerated(
        "consciousness",
        CONSCIOUSNESS,
        "Level of consciousness: normal (incl. sleep)=0, disoriented=5.",
    ),
    InputSpec::enumerated(
        "cyanosis",
        CYANOSIS,
        "Cyanosis: none=0, with agitation=4, at rest=5.",
    ),
    InputSpec::enumerated(
        "stridor",
        STRIDOR,
        "Inspiratory stridor: none=0, with agitation=1, at rest=2.",
    ),
    InputSpec::enumerated(
        "air_entry",
        AIR_ENTRY,
        "Air entry: normal=0, decreased=1, markedly decreased=2.",
    ),
    InputSpec::enumerated(
        "retractions",
        RETRACTIONS,
        "Retractions: none=0, mild=1, moderate=2, severe=3.",
    ),
];

fn interpret(total: i64) -> String {
    let band = match total {
        0..=2 => "mild (<=2)",
        3..=7 => "moderate (3-7)",
        8..=11 => "severe (8-11)",
        _ => "impending respiratory failure (>=12)",
    };
    format!("Score {total}/17: {band} (descriptive band only).")
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let consciousness = match i.enum_one("consciousness", CONSCIOUSNESS)?.as_str() {
        "normal" => 0,
        _ => 5,
    };
    let cyanosis = match i.enum_one("cyanosis", CYANOSIS)?.as_str() {
        "none" => 0,
        "with_agitation" => 4,
        _ => 5,
    };
    let stridor = match i.enum_one("stridor", STRIDOR)?.as_str() {
        "none" => 0,
        "with_agitation" => 1,
        _ => 2,
    };
    let air_entry = match i.enum_one("air_entry", AIR_ENTRY)?.as_str() {
        "normal" => 0,
        "decreased" => 1,
        _ => 2,
    };
    let retractions = match i.enum_one("retractions", RETRACTIONS)?.as_str() {
        "none" => 0,
        "mild" => 1,
        "moderate" => 2,
        _ => 3,
    };

    let total = consciousness + cyanosis + stridor + air_entry + retractions;
    Ok(ScoreResult::new(
        DESCRIPTOR.id,
        DESCRIPTOR.version,
        total as f64,
        DESCRIPTOR.unit,
        interpret(total),
        vec![format!(
            "points: consciousness {consciousness}, cyanosis {cyanosis}, stridor {stridor}, air entry {air_entry}, retractions {retractions}"
        )],
        DESCRIPTOR.citation,
    ))
}
