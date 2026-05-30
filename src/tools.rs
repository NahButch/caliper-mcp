//! The seven MCP tools, all derived from the registry. Pure functions over `serde_json`:
//! no I/O, no state. The JSON-RPC layer ([`crate::mcp`]) wraps these.

use crate::{registry, units, CalcError, InputKind, InputSpec, Inputs};
use serde_json::{json, Map, Value};

/// The outcome of a tool call: a JSON payload plus whether it represents an error.
pub struct ToolOutcome {
    pub value: Value,
    pub is_error: bool,
}

impl ToolOutcome {
    fn ok(value: Value) -> Self {
        ToolOutcome {
            value,
            is_error: false,
        }
    }
    fn err(value: Value) -> Self {
        ToolOutcome {
            value,
            is_error: true,
        }
    }
    fn calc(e: CalcError) -> Self {
        ToolOutcome {
            value: serde_json::to_value(&e).unwrap_or_else(|_| json!({"error": "Serialization"})),
            is_error: true,
        }
    }
    fn invalid(message: &str) -> Self {
        ToolOutcome::err(json!({ "error": "InvalidParams", "message": message }))
    }
}

/// Names of all tools, for `tools/list`.
pub const TOOL_NAMES: &[&str] = &[
    "list_scores",
    "score_inputs",
    "compute_score",
    "convert_units",
    "solve_for",
    "score_series",
    "suggest_scores",
];

/// Dispatch a tool call by name.
pub fn call_tool(name: &str, args: &Value) -> ToolOutcome {
    match name {
        "list_scores" => list_scores(args),
        "score_inputs" => score_inputs(args),
        "compute_score" => compute_score(args),
        "convert_units" => convert_units(args),
        "solve_for" => solve_for(args),
        "score_series" => score_series(args),
        "suggest_scores" => suggest_scores(args),
        other => ToolOutcome::err(json!({
            "error": "UnknownTool",
            "message": format!("no such tool '{other}'"),
        })),
    }
}

fn str_arg<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Value::as_str)
}

fn input_kind_json(kind: &InputKind) -> Value {
    match kind {
        InputKind::Quantity { analyte, canonical } => json!({
            "type": "quantity",
            "analyte": analyte,
            "canonical_unit": canonical,
        }),
        InputKind::Ratio => json!({ "type": "ratio" }),
        InputKind::Bool => json!({ "type": "boolean" }),
        InputKind::Enum(allowed) => json!({ "type": "enum", "allowed": allowed }),
        InputKind::Integer { min, max } => json!({ "type": "integer", "min": min, "max": max }),
    }
}

fn input_spec_json(spec: &InputSpec) -> Value {
    json!({
        "field": spec.field,
        "kind": input_kind_json(&spec.kind),
        "required": spec.required,
        "allowed_units": spec.allowed_units,
        "notes": spec.notes,
        "floor": spec.floor,
        "ceiling": spec.ceiling,
    })
}

fn descriptor_summary(d: &crate::ScoreDescriptor) -> Value {
    json!({
        "id": d.id,
        "name": d.name,
        "version": d.version,
        "domain": d.domain,
        "unit": d.unit,
        "citation": d.citation,
    })
}

// ---- list_scores -----------------------------------------------------------

fn list_scores(args: &Value) -> ToolOutcome {
    let domain = str_arg(args, "domain");
    let query = str_arg(args, "query");
    let scores: Vec<Value> = registry::filter(domain, query)
        .iter()
        .map(|d| descriptor_summary(d))
        .collect();
    ToolOutcome::ok(json!({
        "count": scores.len(),
        "domains": registry::domains(),
        "scores": scores,
    }))
}

// ---- score_inputs ----------------------------------------------------------

fn score_inputs(args: &Value) -> ToolOutcome {
    let id = match str_arg(args, "id") {
        Some(id) => id,
        None => return ToolOutcome::invalid("missing required field 'id'"),
    };
    let d = match registry::find(id) {
        Some(d) => d,
        None => return unknown_score(id),
    };
    let inputs: Vec<Value> = d.inputs.iter().map(input_spec_json).collect();
    ToolOutcome::ok(json!({
        "id": d.id,
        "name": d.name,
        "version": d.version,
        "unit": d.unit,
        "citation": d.citation,
        "inputs": inputs,
    }))
}

fn unknown_score(id: &str) -> ToolOutcome {
    ToolOutcome::err(json!({
        "error": "UnknownScore",
        "message": format!("no score with id '{id}'"),
        "available": registry::all().iter().map(|d| d.id).collect::<Vec<_>>(),
    }))
}

// ---- compute_score ---------------------------------------------------------

fn compute_score(args: &Value) -> ToolOutcome {
    let id = match str_arg(args, "id") {
        Some(id) => id,
        None => return ToolOutcome::invalid("missing required field 'id'"),
    };
    let d = match registry::find(id) {
        Some(d) => d,
        None => return unknown_score(id),
    };
    let inputs_obj = match args.get("inputs").and_then(Value::as_object) {
        Some(o) => o,
        None => return ToolOutcome::invalid("missing required field 'inputs' (object)"),
    };
    let inputs = Inputs::new(inputs_obj);
    match (d.compute)(&inputs) {
        Ok(result) => ToolOutcome::ok(serde_json::to_value(result).unwrap()),
        Err(e) => ToolOutcome::calc(e),
    }
}

// ---- convert_units ---------------------------------------------------------

fn convert_units(args: &Value) -> ToolOutcome {
    let analyte = match str_arg(args, "analyte") {
        Some(a) => a,
        None => return ToolOutcome::invalid("missing required field 'analyte'"),
    };
    let value = match args.get("value").and_then(Value::as_f64) {
        Some(v) => v,
        None => return ToolOutcome::invalid("missing required numeric field 'value'"),
    };
    let from = match str_arg(args, "from") {
        Some(f) => f,
        None => return ToolOutcome::invalid("missing required field 'from'"),
    };
    let to = match str_arg(args, "to") {
        Some(t) => t,
        None => return ToolOutcome::invalid("missing required field 'to'"),
    };
    match units::convert(analyte, value, from, to) {
        Ok(c) => ToolOutcome::ok(json!({
            "analyte": analyte,
            "input": { "value": value, "unit": c.from },
            "output": { "value": c.value, "unit": c.to },
            "factor": c.factor,
            "basis": c.basis,
        })),
        Err(e) => ToolOutcome::calc(e),
    }
}

// ---- solve_for -------------------------------------------------------------

fn default_bounds(analyte: &str) -> (f64, f64) {
    match analyte {
        "creatinine" => (0.1, 30.0),
        "bilirubin" => (0.1, 80.0),
        "sodium" => (90.0, 170.0),
        "albumin" => (0.5, 6.0),
        "age" => (0.0, 120.0),
        "weight" => (1.0, 400.0),
        "platelets" => (1.0, 1000.0),
        "aminotransferase" => (1.0, 5000.0),
        "pao2" => (20.0, 700.0),
        "pressure" => (20.0, 300.0),
        "rate_breaths" | "rate_beats" => (0.0, 300.0),
        "spo2" => (0.0, 100.0),
        _ => (1e-3, 1e6),
    }
}

fn solve_for(args: &Value) -> ToolOutcome {
    let id = match str_arg(args, "id") {
        Some(id) => id,
        None => return ToolOutcome::invalid("missing required field 'id'"),
    };
    let d = match registry::find(id) {
        Some(d) => d,
        None => return unknown_score(id),
    };
    let target = match args.get("target").and_then(Value::as_f64) {
        Some(t) => t,
        None => return ToolOutcome::invalid("missing required numeric field 'target'"),
    };
    let solve = match args.get("solve").and_then(Value::as_object) {
        Some(o) => o,
        None => return ToolOutcome::invalid("missing required field 'solve' ({field, unit?})"),
    };
    let field = match solve.get("field").and_then(Value::as_str) {
        Some(f) => f,
        None => return ToolOutcome::invalid("'solve.field' is required"),
    };

    // The field must be a unit-typed quantity input; find its analyte/canonical.
    let spec = match d.inputs.iter().find(|s| s.field == field) {
        Some(s) => s,
        None => return ToolOutcome::invalid(&format!("'{field}' is not an input of '{id}'")),
    };
    let (analyte, canonical) = match spec.kind {
        InputKind::Quantity { analyte, canonical } => (analyte, canonical),
        _ => {
            return ToolOutcome::invalid(&format!(
                "'{field}' is not a unit-typed quantity; solve_for varies quantities only"
            ))
        }
    };

    let fixed = match args.get("fixed").and_then(Value::as_object) {
        Some(o) => o.clone(),
        None => Map::new(),
    };

    // f(x) = compute(score with field=x canonical) - target, evaluated in canonical units.
    let eval = |x: f64| -> Result<f64, CalcError> {
        let mut m = fixed.clone();
        m.insert(field.to_string(), json!({ "value": x, "unit": canonical }));
        let inputs = Inputs::new(&m);
        (d.compute)(&inputs).map(|r| r.value)
    };

    let (lo, hi) = args
        .get("bounds")
        .and_then(Value::as_array)
        .and_then(|a| Some((a.first()?.as_f64()?, a.get(1)?.as_f64()?)))
        .unwrap_or_else(|| default_bounds(analyte));

    let g = |x: f64| eval(x).map(|v| v - target);
    let glo = match g(lo) {
        Ok(v) => v,
        Err(e) => return ToolOutcome::calc(e),
    };
    let ghi = match g(hi) {
        Ok(v) => v,
        Err(e) => return ToolOutcome::calc(e),
    };
    if glo == 0.0 {
        return solve_result(field, canonical, lo, target, 0);
    }
    if ghi == 0.0 {
        return solve_result(field, canonical, hi, target, 0);
    }
    if glo.signum() == ghi.signum() {
        return ToolOutcome::err(json!({
            "error": "NotBracketed",
            "message": format!(
                "target {target} not bracketed on [{lo}, {hi}] (score spans [{}, {}]); pass explicit 'bounds'",
                glo + target, ghi + target
            ),
        }));
    }

    // Monotone bisection.
    let (mut a, mut b) = (lo, hi);
    let (mut ga, _gb) = (glo, ghi);
    let mut mid = a;
    let mut iterations = 0;
    for _ in 0..200 {
        iterations += 1;
        mid = 0.5 * (a + b);
        let gm = match g(mid) {
            Ok(v) => v,
            Err(e) => return ToolOutcome::calc(e),
        };
        if gm.abs() < 1e-9 || (b - a).abs() < 1e-9 {
            break;
        }
        if ga.signum() == gm.signum() {
            a = mid;
            ga = gm;
        } else {
            b = mid;
        }
    }
    solve_result(field, canonical, mid, target, iterations)
}

fn solve_result(field: &str, canonical: &str, x: f64, target: f64, iterations: u32) -> ToolOutcome {
    ToolOutcome::ok(json!({
        "field": field,
        "threshold": { "value": x, "unit": canonical },
        "target": target,
        "method": "monotone bisection",
        "iterations": iterations,
        "disclaimer": crate::DISCLAIMER,
    }))
}

// ---- score_series ----------------------------------------------------------

fn score_series(args: &Value) -> ToolOutcome {
    let id = match str_arg(args, "id") {
        Some(id) => id,
        None => return ToolOutcome::invalid("missing required field 'id'"),
    };
    let d = match registry::find(id) {
        Some(d) => d,
        None => return unknown_score(id),
    };
    let series = match args.get("series").and_then(Value::as_array) {
        Some(s) => s,
        None => {
            return ToolOutcome::invalid("missing required field 'series' (array of {t, inputs})")
        }
    };

    let mut points = Vec::new();
    let mut prev: Option<f64> = None;
    let mut first: Option<f64> = None;
    let mut last: Option<f64> = None;

    for (idx, entry) in series.iter().enumerate() {
        let t = entry.get("t").cloned().unwrap_or(json!(idx));
        let inputs_obj = match entry.get("inputs").and_then(Value::as_object) {
            Some(o) => o,
            None => {
                points.push(json!({
                    "t": t,
                    "error": { "error": "InvalidParams", "message": "entry missing 'inputs' object" },
                }));
                continue;
            }
        };
        let inputs = Inputs::new(inputs_obj);
        match (d.compute)(&inputs) {
            Ok(result) => {
                let value = result.value;
                let delta = prev.map(|p| value - p);
                let trend = delta.map(trend_label);
                points.push(json!({
                    "t": t,
                    "value": value,
                    "delta": delta,
                    "trend": trend,
                    "interpretation": result.interpretation,
                }));
                if first.is_none() {
                    first = Some(value);
                }
                last = Some(value);
                prev = Some(value);
            }
            Err(e) => {
                points.push(json!({
                    "t": t,
                    "error": serde_json::to_value(&e).unwrap(),
                }));
            }
        }
    }

    let overall = match (first, last) {
        (Some(f), Some(l)) => json!({
            "first": f,
            "last": l,
            "net_delta": l - f,
            "trend": trend_label(l - f),
        }),
        _ => Value::Null,
    };

    ToolOutcome::ok(json!({
        "id": d.id,
        "version": d.version,
        "points": points,
        "overall": overall,
        "disclaimer": crate::DISCLAIMER,
    }))
}

fn trend_label(delta: f64) -> &'static str {
    if delta > 1e-9 {
        "rising"
    } else if delta < -1e-9 {
        "falling"
    } else {
        "stable"
    }
}

// ---- suggest_scores --------------------------------------------------------

fn suggest_scores(args: &Value) -> ToolOutcome {
    let context = args.get("context").unwrap_or(args);

    // Free text to match against keywords/name/domain.
    let text = [
        context.get("question").and_then(Value::as_str),
        context.get("text").and_then(Value::as_str),
        context.get("notes").and_then(Value::as_str),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ")
    .to_ascii_lowercase();

    let domain = context.get("domain").and_then(Value::as_str);

    let available: Vec<String> = context
        .get("available_inputs")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(|s| s.to_ascii_lowercase())
                .collect()
        })
        .unwrap_or_default();

    let mut scored: Vec<(i64, Value)> = Vec::new();
    for d in registry::all() {
        let mut score = 0i64;
        let mut why: Vec<String> = Vec::new();

        if let Some(dm) = domain {
            if d.domain.eq_ignore_ascii_case(dm) {
                score += 3;
                why.push(format!("matches domain '{dm}'"));
            }
        }
        for kw in d.keywords {
            if !text.is_empty() && text.contains(&kw.to_ascii_lowercase()) {
                score += 2;
                why.push(format!("context mentions '{kw}'"));
            }
        }
        if !text.is_empty()
            && (text.contains(&d.id.to_ascii_lowercase())
                || text.contains(&d.name.to_ascii_lowercase()))
        {
            score += 3;
            why.push("context names the score".to_string());
        }

        let needed: Vec<&str> = d
            .inputs
            .iter()
            .filter(|s| s.required)
            .map(|s| s.field)
            .collect();
        if !available.is_empty() {
            let have = needed
                .iter()
                .filter(|f| available.contains(&f.to_ascii_lowercase()))
                .count();
            if have > 0 {
                score += have as i64;
                why.push(format!("{have}/{} required inputs available", needed.len()));
            }
        }

        if score > 0 {
            scored.push((
                score,
                json!({
                    "id": d.id,
                    "name": d.name,
                    "domain": d.domain,
                    "why": why,
                    "needed_inputs": needed,
                    "match_score": score,
                }),
            ));
        }
    }

    scored.sort_by_key(|(s, _)| std::cmp::Reverse(*s));
    let candidates: Vec<Value> = scored.into_iter().map(|(_, v)| v).collect();

    ToolOutcome::ok(json!({
        "candidates": candidates,
        "note": "Suggestions only; this tool does not compute. Choose a score and call compute_score.",
        "disclaimer": crate::DISCLAIMER,
    }))
}
