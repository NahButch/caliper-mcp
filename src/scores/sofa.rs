//! Sequential Organ Failure Assessment (SOFA) score (Vincent et al., 1996).

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "sofa",
    name: "SOFA",
    version: "1996",
    citation: "Vincent JL, Moreno R, Takala J, et al. The SOFA (Sepsis-related Organ Failure Assessment) score to describe organ dysfunction/failure. Intensive Care Med. 1996;22(7):707-710.",
    domain: "icu",
    keywords: &["sofa", "organ failure", "icu", "sepsis", "critical care", "mortality"],
    unit: "points",
    inputs: INPUTS,
    compute,
};

const VASOPRESSOR: &[&str] = &[
    "none",
    "dopamine_le5_or_dobutamine",
    "dopamine_gt5_or_epi_le0.1_or_norepi_le0.1",
    "dopamine_gt15_or_epi_gt0.1_or_norepi_gt0.1",
];

const INPUTS: &[InputSpec] = &[
    InputSpec::quantity(
        "pao2",
        "pao2",
        "mmHg",
        &["mmHg", "kPa"],
        "Arterial PaO2 (respiration component).",
    ),
    InputSpec::quantity(
        "fio2",
        "fio2",
        "fraction",
        &["fraction", "%"],
        "Fraction of inspired oxygen (respiration component).",
    ),
    InputSpec::boolean(
        "respiratory_support",
        "On mechanical ventilation / respiratory support (required to score 3-4 for respiration).",
    ),
    InputSpec::quantity(
        "platelets",
        "platelets",
        "10^9/L",
        &["10^9/L", "10^3/uL"],
        "Platelet count (coagulation component).",
    ),
    InputSpec::quantity(
        "bilirubin",
        "bilirubin",
        "mg/dL",
        &["mg/dL", "umol/L"],
        "Total bilirubin (liver component).",
    ),
    InputSpec::quantity(
        "map",
        "pressure",
        "mmHg",
        &["mmHg"],
        "Mean arterial pressure (cardiovascular component).",
    ),
    InputSpec::enumerated(
        "vasopressor",
        VASOPRESSOR,
        "Vasopressor category (cardiovascular component); state 'none' explicitly.",
    ),
    InputSpec::integer("gcs", 3, 15, "Glasgow Coma Scale total (CNS component)."),
    InputSpec::quantity(
        "creatinine",
        "creatinine",
        "mg/dL",
        &["mg/dL", "umol/L"],
        "Serum creatinine (renal component).",
    ),
    InputSpec::quantity(
        "urine_output",
        "urine_output",
        "mL/day",
        &["mL/day"],
        "24-hour urine output (optional; upgrades renal component).",
    )
    .optional(),
];

fn respiration_points(ratio: f64, support: bool, rules: &mut Vec<String>) -> i64 {
    if ratio >= 400.0 {
        0
    } else if ratio >= 300.0 {
        1
    } else if ratio >= 200.0 {
        2
    } else if ratio >= 100.0 {
        if support {
            3
        } else {
            rules.push(
                "respiration: PaO2/FiO2 <200 without respiratory support capped at 2".to_string(),
            );
            2
        }
    } else if support {
        4
    } else {
        rules.push(
            "respiration: PaO2/FiO2 <100 without respiratory support capped at 2".to_string(),
        );
        2
    }
}

fn coagulation_points(plt: f64) -> i64 {
    if plt >= 150.0 {
        0
    } else if plt >= 100.0 {
        1
    } else if plt >= 50.0 {
        2
    } else if plt >= 20.0 {
        3
    } else {
        4
    }
}

fn liver_points(bili: f64) -> i64 {
    if bili < 1.2 {
        0
    } else if bili < 2.0 {
        1
    } else if bili < 6.0 {
        2
    } else if bili < 12.0 {
        3
    } else {
        4
    }
}

fn cardiovascular_points(map: f64, vasopressor: &str) -> i64 {
    match vasopressor {
        "dopamine_le5_or_dobutamine" => 2,
        "dopamine_gt5_or_epi_le0.1_or_norepi_le0.1" => 3,
        "dopamine_gt15_or_epi_gt0.1_or_norepi_gt0.1" => 4,
        _ => {
            if map >= 70.0 {
                0
            } else {
                1
            }
        }
    }
}

fn cns_points(gcs: i64) -> i64 {
    if gcs >= 15 {
        0
    } else if gcs >= 13 {
        1
    } else if gcs >= 10 {
        2
    } else if gcs >= 6 {
        3
    } else {
        4
    }
}

fn renal_points(cr: f64, urine: Option<f64>, rules: &mut Vec<String>) -> i64 {
    let by_cr = if cr < 1.2 {
        0
    } else if cr < 2.0 {
        1
    } else if cr < 3.5 {
        2
    } else if cr < 5.0 {
        3
    } else {
        4
    };
    let by_uo = urine.map(|u| {
        if u < 200.0 {
            4
        } else if u < 500.0 {
            3
        } else {
            0
        }
    });
    match by_uo {
        Some(u) if u > by_cr => {
            rules.push(format!(
                "renal: urine-output criterion ({u}) exceeded creatinine criterion ({by_cr})"
            ));
            u
        }
        _ => by_cr,
    }
}

fn interpret(total: i64) -> String {
    format!("SOFA {total}/24 (descriptive only; higher totals indicate greater organ dysfunction).")
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let mut rules = Vec::new();

    let pao2 = i.quantity("pao2", "pao2")?;
    let fio2 = i.quantity("fio2", "fio2")?;
    if fio2 <= 0.0 {
        return Err(CalcError::OutOfRange {
            field: "fio2".to_string(),
            message: "FiO2 must be positive".to_string(),
        });
    }
    let support = i.boolean("respiratory_support")?;
    let ratio = pao2 / fio2;
    let resp = respiration_points(ratio, support, &mut rules);

    let coag = coagulation_points(i.quantity("platelets", "platelets")?);
    let liver = liver_points(i.quantity("bilirubin", "bilirubin")?);
    let cardio = cardiovascular_points(
        i.quantity("map", "pressure")?,
        &i.enum_one("vasopressor", VASOPRESSOR)?,
    );
    let cns = cns_points(i.integer("gcs", 3, 15)?);
    let renal = renal_points(
        i.quantity("creatinine", "creatinine")?,
        i.opt_quantity("urine_output", "urine_output")?,
        &mut rules,
    );

    let total = resp + coag + liver + cardio + cns + renal;
    rules.push(format!(
        "components resp {resp}, coag {coag}, liver {liver}, cardio {cardio}, cns {cns}, renal {renal}"
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
