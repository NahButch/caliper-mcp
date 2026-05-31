//! MEWS — Modified Early Warning Score (Subbe et al., 2001).
//!
//! A five-parameter bedside deterioration score. Caliper implements the original Subbe 2001
//! grid (temperature extremes both score 2; AVPU 0/1/2/3). See docs/COEFFICIENT_AUDIT.md for
//! the note on variant tables. NEWS2 is the modern RCP-standardised successor; both provided.

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "mews",
    name: "MEWS",
    version: "Subbe-2001",
    citation: "Subbe CP, Kruger M, Rutherford P, Gemmel L. Validation of a modified Early Warning Score in medical admissions. QJM. 2001;94(10):521-526.",
    domain: "acute",
    keywords: &["early warning", "deterioration", "mews", "ward", "vital signs"],
    unit: "points",
    inputs: INPUTS,
    compute,
};

const AVPU: &[&str] = &["alert", "voice", "pain", "unresponsive"];

const INPUTS: &[InputSpec] = &[
    InputSpec::quantity(
        "systolic_bp",
        "pressure",
        "mmHg",
        &["mmHg"],
        "Systolic blood pressure.",
    ),
    InputSpec::quantity(
        "heart_rate",
        "rate_beats",
        "bpm",
        &["bpm", "beats/min"],
        "Heart rate.",
    ),
    InputSpec::quantity(
        "respiratory_rate",
        "rate_breaths",
        "breaths/min",
        &["breaths/min", "/min"],
        "Respiratory rate.",
    ),
    InputSpec::quantity(
        "temperature",
        "temperature",
        "°C",
        &["°C", "°F"],
        "Temperature.",
    ),
    InputSpec::enumerated(
        "avpu",
        AVPU,
        "AVPU consciousness: alert=0, voice=1, pain=2, unresponsive=3.",
    ),
];

fn sbp_points(s: f64) -> i64 {
    if s <= 70.0 {
        3
    } else if s <= 80.0 {
        2
    } else if s <= 100.0 {
        1
    } else if s <= 199.0 {
        0
    } else {
        2
    }
}

fn hr_points(h: f64) -> i64 {
    if h <= 40.0 {
        2
    } else if h <= 50.0 {
        1
    } else if h <= 100.0 {
        0
    } else if h <= 110.0 {
        1
    } else if h <= 129.0 {
        2
    } else {
        3
    }
}

fn rr_points(r: f64) -> i64 {
    if r < 9.0 {
        2
    } else if r <= 14.0 {
        0
    } else if r <= 20.0 {
        1
    } else if r <= 29.0 {
        2
    } else {
        3
    }
}

fn temp_points(t: f64) -> i64 {
    if t < 35.0 {
        2
    } else if t <= 38.4 {
        0
    } else {
        2
    }
}

fn avpu_points(a: &str) -> i64 {
    match a {
        "alert" => 0,
        "voice" => 1,
        "pain" => 2,
        _ => 3,
    }
}

fn interpret(total: i64) -> String {
    let band = if total >= 5 {
        "higher-risk band (aggregate >=5)"
    } else {
        "lower-risk band (aggregate 0-4)"
    };
    format!("MEWS {total}: {band} (descriptive only).")
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let sbp = sbp_points(i.quantity("systolic_bp", "pressure")?);
    let hr = hr_points(i.quantity("heart_rate", "rate_beats")?);
    let rr = rr_points(i.quantity("respiratory_rate", "rate_breaths")?);
    let temp = temp_points(i.quantity("temperature", "temperature")?);
    let avpu = avpu_points(&i.enum_one("avpu", AVPU)?);

    let total = sbp + hr + rr + temp + avpu;
    let rules = vec![format!(
        "components SBP {sbp}, HR {hr}, RR {rr}, temp {temp}, AVPU {avpu}"
    )];

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
