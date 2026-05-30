//! eGFR by the 2021 race-free CKD-EPI creatinine equation (Inker et al., 2021).

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "ckd-epi-2021",
    name: "eGFR (CKD-EPI creatinine, 2021 race-free)",
    version: "CKD-EPI-2021",
    citation: "Inker LA, Eneanya ND, Coresh J, et al. New creatinine- and cystatin C-based equations to estimate GFR without race. N Engl J Med. 2021;385(19):1737-1749.",
    domain: "renal",
    keywords: &["egfr", "kidney", "renal function", "ckd", "gfr", "creatinine"],
    unit: "mL/min/1.73m^2",
    inputs: INPUTS,
    compute,
};

const SEX: &[&str] = &["male", "female"];

const INPUTS: &[InputSpec] = &[
    InputSpec::quantity(
        "creatinine",
        "creatinine",
        "mg/dL",
        &["mg/dL", "umol/L"],
        "Serum creatinine.",
    ),
    InputSpec::quantity("age", "age", "years", &["years"], "Age in years."),
    InputSpec::enumerated(
        "sex",
        SEX,
        "Sex (drives kappa/alpha and the female multiplier).",
    ),
];

fn interpret(egfr: f64) -> String {
    let stage = if egfr >= 90.0 {
        "G1 (normal/high)"
    } else if egfr >= 60.0 {
        "G2 (mildly decreased)"
    } else if egfr >= 45.0 {
        "G3a (mild-moderate)"
    } else if egfr >= 30.0 {
        "G3b (moderate-severe)"
    } else if egfr >= 15.0 {
        "G4 (severely decreased)"
    } else {
        "G5 (kidney failure)"
    };
    format!("eGFR {egfr:.1} mL/min/1.73m^2: KDIGO {stage} (descriptive only).")
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let scr = i.quantity("creatinine", "creatinine")?;
    let age = i.quantity("age", "age")?;
    let female = i.enum_one("sex", SEX)? == "female";

    if scr <= 0.0 {
        return Err(CalcError::OutOfRange {
            field: "creatinine".to_string(),
            message: "creatinine must be positive".to_string(),
        });
    }

    let (kappa, alpha) = if female { (0.7, -0.241) } else { (0.9, -0.302) };
    let ratio = scr / kappa;
    let min_term = ratio.min(1.0).powf(alpha);
    let max_term = ratio.max(1.0).powf(-1.200);
    let age_term = 0.9938f64.powf(age);
    let sex_term = if female { 1.012 } else { 1.0 };

    let egfr = 142.0 * min_term * max_term * age_term * sex_term;

    let mut rules = vec![format!(
        "kappa={kappa}, alpha={alpha}, female_multiplier={sex_term}"
    )];
    if female {
        rules.push("female coefficients applied".to_string());
    }

    Ok(ScoreResult::new(
        DESCRIPTOR.id,
        DESCRIPTOR.version,
        egfr,
        DESCRIPTOR.unit,
        interpret(egfr),
        rules,
        DESCRIPTOR.citation,
    ))
}
