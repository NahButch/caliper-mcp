//! Child-Pugh score for chronic liver disease severity (Pugh et al., 1973).

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "child-pugh",
    name: "Child-Pugh",
    version: "1973",
    citation: "Pugh RNH, Murray-Lyon IM, Dawson JL, Pietroni MC, Williams R. Transection of the oesophagus for bleeding oesophageal varices. Br J Surg. 1973;60(8):646-649.",
    domain: "hepatology",
    keywords: &["cirrhosis", "liver", "child pugh", "hepatic", "varices"],
    unit: "points",
    inputs: INPUTS,
    compute,
};

const ASCITES: &[&str] = &["none", "mild", "moderate"];
const ENCEPH: &[&str] = &["none", "grade1-2", "grade3-4"];

const INPUTS: &[InputSpec] = &[
    InputSpec::quantity(
        "bilirubin",
        "bilirubin",
        "mg/dL",
        &["mg/dL", "umol/L"],
        "Total bilirubin: <2 =1, 2-3 =2, >3 =3.",
    ),
    InputSpec::quantity(
        "albumin",
        "albumin",
        "g/dL",
        &["g/dL", "g/L"],
        "Albumin: >3.5 =1, 2.8-3.5 =2, <2.8 =3.",
    ),
    InputSpec::ratio("inr", "INR: <1.7 =1, 1.7-2.3 =2, >2.3 =3."),
    InputSpec::enumerated(
        "ascites",
        ASCITES,
        "Ascites: none=1, mild=2, moderate(-severe)=3.",
    ),
    InputSpec::enumerated(
        "encephalopathy",
        ENCEPH,
        "Encephalopathy: none=1, grade1-2=2, grade3-4=3.",
    ),
];

fn interpret(total: i64) -> String {
    let class = match total {
        5..=6 => "Class A",
        7..=9 => "Class B",
        _ => "Class C",
    };
    format!("Score {total}/15: {class} (descriptive only).")
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let mut rules = Vec::new();

    let bili = i.quantity("bilirubin", "bilirubin")?;
    let bili_pts = if bili < 2.0 {
        1
    } else if bili <= 3.0 {
        2
    } else {
        3
    };

    let alb = i.quantity("albumin", "albumin")?;
    let alb_pts = if alb > 3.5 {
        1
    } else if alb >= 2.8 {
        2
    } else {
        3
    };

    let inr = i.ratio("inr")?;
    let inr_pts = if inr < 1.7 {
        1
    } else if inr <= 2.3 {
        2
    } else {
        3
    };

    let ascites = i.enum_one("ascites", ASCITES)?;
    let ascites_pts = match ascites.as_str() {
        "none" => 1,
        "mild" => 2,
        _ => 3,
    };

    let enceph = i.enum_one("encephalopathy", ENCEPH)?;
    let enceph_pts = match enceph.as_str() {
        "none" => 1,
        "grade1-2" => 2,
        _ => 3,
    };

    let total = bili_pts + alb_pts + inr_pts + ascites_pts + enceph_pts;
    rules.push(format!(
        "points: bilirubin {bili_pts}, albumin {alb_pts}, INR {inr_pts}, ascites {ascites_pts}, encephalopathy {enceph_pts}"
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
