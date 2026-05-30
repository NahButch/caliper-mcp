//! FIB-4 index for hepatic fibrosis (Sterling et al., 2006).

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "fib-4",
    name: "FIB-4 index",
    version: "Sterling-2006",
    citation: "Sterling RK, Lissen E, Clumeck N, et al. Development of a simple noninvasive index to predict significant fibrosis in patients with HIV/HCV coinfection. Hepatology. 2006;43(6):1317-1325.",
    domain: "hepatology",
    keywords: &["fibrosis", "liver", "nafld", "cirrhosis", "fib-4", "nash"],
    unit: "index",
    inputs: INPUTS,
    compute,
};

const INPUTS: &[InputSpec] = &[
    InputSpec::quantity("age", "age", "years", &["years"], "Age in years."),
    InputSpec::quantity(
        "ast",
        "aminotransferase",
        "U/L",
        &["U/L", "IU/L"],
        "Aspartate aminotransferase.",
    ),
    InputSpec::quantity(
        "alt",
        "aminotransferase",
        "U/L",
        &["U/L", "IU/L"],
        "Alanine aminotransferase.",
    ),
    InputSpec::quantity(
        "platelets",
        "platelets",
        "10^9/L",
        &["10^9/L", "10^3/uL"],
        "Platelet count.",
    ),
];

fn interpret(v: f64) -> String {
    let band = if v < 1.45 {
        "below 1.45: lower probability of advanced fibrosis"
    } else if v <= 3.25 {
        "1.45-3.25: indeterminate"
    } else {
        "above 3.25: higher probability of advanced fibrosis"
    };
    format!("FIB-4 = {v:.2}: {band} (descriptive only).")
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let age = i.quantity("age", "age")?;
    let ast = i.quantity("ast", "aminotransferase")?;
    let alt = i.quantity("alt", "aminotransferase")?;
    let plt = i.quantity("platelets", "platelets")?;

    if plt <= 0.0 {
        return Err(CalcError::OutOfRange {
            field: "platelets".to_string(),
            message: "platelet count must be positive".to_string(),
        });
    }
    if alt <= 0.0 {
        return Err(CalcError::OutOfRange {
            field: "alt".to_string(),
            message: "ALT must be positive".to_string(),
        });
    }

    let fib4 = (age * ast) / (plt * alt.sqrt());

    Ok(ScoreResult::new(
        DESCRIPTOR.id,
        DESCRIPTOR.version,
        fib4,
        DESCRIPTOR.unit,
        interpret(fib4),
        vec![],
        DESCRIPTOR.citation,
    ))
}
