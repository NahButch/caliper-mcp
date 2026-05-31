//! Glasgow-Blatchford Bleeding Score for upper GI bleeding (Blatchford et al., 2000).
//!
//! Risk-stratifies patients with upper GI bleeding by need for intervention (transfusion,
//! endoscopic therapy, surgery). A score of 0 identifies very low-risk patients.

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "glasgow-blatchford",
    name: "Glasgow-Blatchford Bleeding Score",
    version: "Blatchford-2000",
    citation: "Blatchford O, Murray WR, Blatchford M. A risk score to predict need for treatment for upper-gastrointestinal haemorrhage. Lancet. 2000;356(9238):1318-1321.",
    domain: "gastroenterology",
    keywords: &["gi bleed", "upper gi bleeding", "hematemesis", "melena", "blatchford", "transfusion"],
    unit: "points",
    inputs: INPUTS,
    compute,
};

const SEX: &[&str] = &["male", "female"];

const INPUTS: &[InputSpec] = &[
    InputSpec::quantity(
        "urea",
        "urea",
        "mmol/L",
        &["mmol/L", "mg/dL"],
        "Blood urea (mmol/L); banded 6.5-7.9=2, 8.0-9.9=3, 10.0-24.9=4, >=25=6.",
    ),
    InputSpec::quantity(
        "hemoglobin",
        "hemoglobin",
        "g/dL",
        &["g/dL", "g/L"],
        "Hemoglobin; sex-specific bands.",
    ),
    InputSpec::enumerated("sex", SEX, "Sex (drives hemoglobin bands)."),
    InputSpec::quantity(
        "systolic_bp",
        "pressure",
        "mmHg",
        &["mmHg"],
        "Systolic BP; 100-109=1, 90-99=2, <90=3.",
    ),
    InputSpec::quantity(
        "pulse",
        "rate_beats",
        "bpm",
        &["bpm", "beats/min"],
        "Pulse >=100 (1).",
    ),
    InputSpec::boolean("melena", "Presentation with melena (1)."),
    InputSpec::boolean("syncope", "Presentation with syncope (2)."),
    InputSpec::boolean("hepatic_disease", "History of hepatic disease (2)."),
    InputSpec::boolean("cardiac_failure", "History of cardiac failure (2)."),
];

fn urea_points(u: f64) -> i64 {
    if u < 6.5 {
        0
    } else if u < 8.0 {
        2
    } else if u < 10.0 {
        3
    } else if u < 25.0 {
        4
    } else {
        6
    }
}

fn hgb_points(hgb: f64, female: bool) -> i64 {
    if female {
        if hgb >= 12.0 {
            0
        } else if hgb >= 10.0 {
            1
        } else {
            6
        }
    } else if hgb >= 13.0 {
        0
    } else if hgb >= 12.0 {
        1
    } else if hgb >= 10.0 {
        3
    } else {
        6
    }
}

fn sbp_points(s: f64) -> i64 {
    if s >= 110.0 {
        0
    } else if s >= 100.0 {
        1
    } else if s >= 90.0 {
        2
    } else {
        3
    }
}

fn interpret(total: i64) -> String {
    let band = if total == 0 {
        "score 0: very low risk"
    } else {
        "score >0: not low-risk"
    };
    format!("Glasgow-Blatchford {total}/23: {band} (descriptive only).")
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let mut rules = Vec::new();

    let urea = i.quantity("urea", "urea")?;
    let up = urea_points(urea);

    let female = i.enum_one("sex", SEX)? == "female";
    let hgb = i.quantity("hemoglobin", "hemoglobin")?;
    let hp = hgb_points(hgb, female);

    let sbp = i.quantity("systolic_bp", "pressure")?;
    let sp = sbp_points(sbp);

    let mut total = up + hp + sp;

    let pulse = i.quantity("pulse", "rate_beats")?;
    if pulse >= 100.0 {
        total += 1;
        rules.push("pulse >=100: +1".to_string());
    }
    if i.boolean("melena")? {
        total += 1;
    }
    if i.boolean("syncope")? {
        total += 2;
    }
    if i.boolean("hepatic_disease")? {
        total += 2;
    }
    if i.boolean("cardiac_failure")? {
        total += 2;
    }

    rules.push(format!(
        "urea band {up}, hemoglobin band {hp}, SBP band {sp}"
    ));

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
