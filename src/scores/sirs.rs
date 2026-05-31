//! SIRS — Systemic Inflammatory Response Syndrome criteria (ACCP/SCCM, Bone et al., 1992).
//!
//! Four criteria; >=2 met defines SIRS. The respiratory criterion is satisfied by either
//! tachypnea or hypocapnia; the leukocyte criterion by leukocytosis, leukopenia, or bandemia.

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "sirs",
    name: "SIRS criteria",
    version: "ACCP-SCCM-1992",
    citation: "Bone RC, Balk RA, Cerra FB, et al. Definitions for sepsis and organ failure and guidelines for the use of innovative therapies in sepsis. ACCP/SCCM Consensus Conference. Chest. 1992;101(6):1644-1655.",
    domain: "icu",
    keywords: &["sepsis", "inflammation", "sirs", "infection", "systemic"],
    unit: "criteria met",
    inputs: INPUTS,
    compute,
};

const INPUTS: &[InputSpec] = &[
    InputSpec::quantity(
        "temperature",
        "temperature",
        "°C",
        &["°C", "°F"],
        "Criterion if >38 or <36 °C.",
    ),
    InputSpec::quantity(
        "heart_rate",
        "rate_beats",
        "bpm",
        &["bpm", "beats/min"],
        "Criterion if >90 bpm.",
    ),
    InputSpec::quantity(
        "respiratory_rate",
        "rate_breaths",
        "breaths/min",
        &["breaths/min", "/min"],
        "Respiratory criterion if RR >20 (or PaCO2 <32 mmHg).",
    ),
    InputSpec::quantity(
        "paco2",
        "paco2",
        "mmHg",
        &["mmHg", "kPa"],
        "Respiratory criterion also met if PaCO2 <32 mmHg.",
    )
    .optional(),
    InputSpec::quantity(
        "wbc",
        "wbc",
        "10^9/L",
        &["10^9/L", "10^3/uL"],
        "Leukocyte criterion if WBC >12 or <4 (x10^9/L).",
    ),
    InputSpec::boolean(
        "bands_over_10pct",
        "Leukocyte criterion also met if >10% immature neutrophils (bands).",
    )
    .optional(),
];

fn interpret(met: i64) -> String {
    if met >= 2 {
        format!("{met}/4 criteria: SIRS present (>=2 met). Descriptive only.")
    } else {
        format!("{met}/4 criteria: SIRS not met (<2). Descriptive only.")
    }
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let mut met = 0i64;
    let mut rules = Vec::new();

    let temp = i.quantity("temperature", "temperature")?;
    if !(36.0..=38.0).contains(&temp) {
        met += 1;
        rules.push("temperature >38 or <36: +1".to_string());
    }
    let hr = i.quantity("heart_rate", "rate_beats")?;
    if hr > 90.0 {
        met += 1;
        rules.push("heart rate >90: +1".to_string());
    }
    let rr = i.quantity("respiratory_rate", "rate_breaths")?;
    let paco2 = i.opt_quantity("paco2", "paco2")?;
    if rr > 20.0 || paco2.map(|p| p < 32.0).unwrap_or(false) {
        met += 1;
        rules.push("respiratory: RR >20 or PaCO2 <32: +1".to_string());
    }
    let wbc = i.quantity("wbc", "wbc")?;
    let bands = i.raw("bands_over_10pct").is_some() && i.boolean("bands_over_10pct")?;
    if !(4.0..=12.0).contains(&wbc) || bands {
        met += 1;
        rules.push("leukocyte: WBC >12 or <4 or >10% bands: +1".to_string());
    }

    Ok(ScoreResult::new(
        DESCRIPTOR.id,
        DESCRIPTOR.version,
        met as f64,
        DESCRIPTOR.unit,
        interpret(met),
        rules,
        DESCRIPTOR.citation,
    ))
}
