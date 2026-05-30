//! National Early Warning Score 2 (Royal College of Physicians, 2017).

use crate::{CalcError, InputSpec, Inputs, ScoreDescriptor, ScoreResult};

pub const DESCRIPTOR: ScoreDescriptor = ScoreDescriptor {
    id: "news2",
    name: "NEWS2",
    version: "RCP-2017",
    citation: "Royal College of Physicians. National Early Warning Score (NEWS) 2: Standardising the assessment of acute-illness severity in the NHS. London: RCP, 2017.",
    domain: "acute",
    keywords: &["early warning", "deterioration", "news2", "ward", "escalation", "vital signs"],
    unit: "points",
    inputs: INPUTS,
    compute,
};

const SCALE: &[&str] = &["1", "2"];
const CONSCIOUSNESS: &[&str] = &["alert", "confusion", "voice", "pain", "unresponsive"];

const INPUTS: &[InputSpec] = &[
    InputSpec::quantity(
        "respiratory_rate",
        "rate_breaths",
        "breaths/min",
        &["breaths/min", "/min"],
        "Respiratory rate.",
    ),
    InputSpec::quantity("spo2", "spo2", "%", &["%"], "Oxygen saturation."),
    InputSpec::enumerated(
        "spo2_scale",
        SCALE,
        "SpO2 scale: '1' (default) or '2' (target range 88-92%).",
    ),
    InputSpec::boolean(
        "supplemental_oxygen",
        "Receiving supplemental oxygen (adds 2; also drives Scale 2 scoring).",
    ),
    InputSpec::quantity(
        "systolic_bp",
        "pressure",
        "mmHg",
        &["mmHg"],
        "Systolic blood pressure.",
    ),
    InputSpec::quantity(
        "pulse",
        "rate_beats",
        "bpm",
        &["bpm", "beats/min"],
        "Heart rate.",
    ),
    InputSpec::enumerated(
        "consciousness",
        CONSCIOUSNESS,
        "ACVPU: alert=0, any of confusion/voice/pain/unresponsive=3.",
    ),
    InputSpec::quantity(
        "temperature",
        "temperature",
        "°C",
        &["°C", "°F"],
        "Temperature.",
    ),
];

fn rr_points(rr: f64) -> i64 {
    if rr <= 8.0 {
        3
    } else if rr <= 11.0 {
        1
    } else if rr <= 20.0 {
        0
    } else if rr <= 24.0 {
        2
    } else {
        3
    }
}

fn spo2_scale1_points(spo2: f64) -> i64 {
    if spo2 <= 91.0 {
        3
    } else if spo2 <= 93.0 {
        2
    } else if spo2 <= 95.0 {
        1
    } else {
        0
    }
}

fn spo2_scale2_points(spo2: f64, on_oxygen: bool) -> i64 {
    // RCP NEWS2 Scale 2 (target SpO2 88-92%).
    if spo2 <= 83.0 {
        3
    } else if spo2 <= 85.0 {
        2
    } else if spo2 <= 87.0 {
        1
    } else if spo2 <= 92.0 {
        0
    } else if !on_oxygen {
        // 93%+ breathing air scores 0 on Scale 2.
        0
    } else if spo2 <= 94.0 {
        1
    } else if spo2 <= 96.0 {
        2
    } else {
        3
    }
}

fn bp_points(sbp: f64) -> i64 {
    if sbp <= 90.0 {
        3
    } else if sbp <= 100.0 {
        2
    } else if sbp <= 110.0 {
        1
    } else if sbp <= 219.0 {
        0
    } else {
        3
    }
}

fn pulse_points(p: f64) -> i64 {
    if p <= 40.0 {
        3
    } else if p <= 50.0 {
        1
    } else if p <= 90.0 {
        0
    } else if p <= 110.0 {
        1
    } else if p <= 130.0 {
        2
    } else {
        3
    }
}

fn temp_points(t: f64) -> i64 {
    if t <= 35.0 {
        3
    } else if t <= 36.0 {
        1
    } else if t <= 38.0 {
        0
    } else if t <= 39.0 {
        1
    } else {
        2
    }
}

fn interpret(total: i64, any_three: bool) -> String {
    let band = if total >= 7 {
        "high (aggregate >=7)"
    } else if total >= 5 {
        "medium (aggregate 5-6)"
    } else if any_three {
        "low-medium (aggregate 0-4 but a single parameter scores 3)"
    } else {
        "low (aggregate 0-4)"
    };
    format!("NEWS2 {total}: {band} clinical-risk band (descriptive only).")
}

pub fn compute(i: &Inputs) -> Result<ScoreResult, CalcError> {
    let mut rules = Vec::new();

    let on_oxygen = i.boolean("supplemental_oxygen")?;
    let scale = i.enum_one("spo2_scale", SCALE)?;
    let spo2 = i.quantity("spo2", "spo2")?;

    let rr = rr_points(i.quantity("respiratory_rate", "rate_breaths")?);
    let spo2_pts = if scale == "2" {
        rules.push("SpO2 scored on Scale 2 (target 88-92%)".to_string());
        spo2_scale2_points(spo2, on_oxygen)
    } else {
        spo2_scale1_points(spo2)
    };
    let oxygen_pts = if on_oxygen {
        rules.push("supplemental oxygen: +2".to_string());
        2
    } else {
        0
    };
    let bp = bp_points(i.quantity("systolic_bp", "pressure")?);
    let pulse = pulse_points(i.quantity("pulse", "rate_beats")?);
    let consciousness = if i.enum_one("consciousness", CONSCIOUSNESS)? == "alert" {
        0
    } else {
        3
    };
    let temp = temp_points(i.quantity("temperature", "temperature")?);

    let params = [rr, spo2_pts, bp, pulse, consciousness, temp];
    let any_three = params.contains(&3);
    let total: i64 = params.iter().sum::<i64>() + oxygen_pts;

    rules.push(format!(
        "components RR {rr}, SpO2 {spo2_pts}, O2 {oxygen_pts}, BP {bp}, pulse {pulse}, consciousness {consciousness}, temp {temp}"
    ));

    Ok(ScoreResult::new(
        DESCRIPTOR.id,
        DESCRIPTOR.version,
        total as f64,
        DESCRIPTOR.unit,
        interpret(total, any_three),
        rules,
        DESCRIPTOR.citation,
    ))
}
