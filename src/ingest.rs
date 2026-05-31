//! Deterministic ingestion: turn free-text / lab-dump prose into candidate, unit-typed
//! inputs **without** violating Caliper's invariants.
//!
//! This module is a *scanner*, not an LLM and not a guesser. It walks the text with a fixed
//! concept lexicon and a hand-written number/unit tokenizer. Its contract:
//!
//! - It **never fabricates a unit.** A value found without a recognized unit is reported as a
//!   `needs_unit` finding (carrying the analyte's canonical unit only as a *suggestion to
//!   confirm*), and is **not** placed into the ready-to-compute inputs.
//! - It is **stateless and side-effect free.** No persistence, no logging of the text, no
//!   network. The input text is borrowed and dropped.
//! - It **does not compute and does not diagnose.** It produces candidate inputs plus a
//!   provenance trail; the decision to compute stays with the caller.
//!
//! The design keeps the trust boundary intact: extraction is explicitly *advisory*. Every
//! extracted quantity records the exact substring it came from so a human/agent can audit it.

use serde_json::{json, Map, Value};

use crate::units;

/// What kind of value a concept resolves to once found in text.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Target {
    /// A unit-typed quantity for the given analyte (canonical unit suggested on `needs_unit`).
    Quantity(&'static str),
    /// A dimensionless ratio (e.g. INR) — a bare number is acceptable, no unit needed.
    Ratio,
    /// An integer (e.g. GCS total).
    Integer,
    /// A sex enum: resolves to "male"/"female".
    Sex,
    /// A boolean flag asserted by the presence of a phrase (optionally negated).
    Flag,
}

/// One concept the scanner knows how to look for.
struct Concept {
    /// The canonical input field name Caliper scores expect.
    field: &'static str,
    target: Target,
    /// Lower-cased trigger phrases. The longest match wins; word-boundary checked.
    triggers: &'static [&'static str],
}

/// The concept lexicon. Field names line up with the score input contracts so extracted
/// values can flow straight into `prepare_score` / `compute_score`.
const CONCEPTS: &[Concept] = &[
    // --- unit-typed quantities ---
    Concept {
        field: "creatinine",
        target: Target::Quantity("creatinine"),
        triggers: &["creatinine", "creat", "scr", "cr"],
    },
    Concept {
        field: "bilirubin",
        target: Target::Quantity("bilirubin"),
        triggers: &["total bilirubin", "bilirubin", "tbili", "bili"],
    },
    Concept {
        field: "sodium",
        target: Target::Quantity("sodium"),
        triggers: &["sodium", "na+", "na"],
    },
    Concept {
        field: "albumin",
        target: Target::Quantity("albumin"),
        triggers: &["albumin", "alb"],
    },
    Concept {
        field: "urea",
        target: Target::Quantity("urea"),
        triggers: &["urea", "bun"],
    },
    Concept {
        field: "ast",
        target: Target::Quantity("aminotransferase"),
        triggers: &["aspartate aminotransferase", "ast", "sgot"],
    },
    Concept {
        field: "alt",
        target: Target::Quantity("aminotransferase"),
        triggers: &["alanine aminotransferase", "alt", "sgpt"],
    },
    Concept {
        field: "platelets",
        target: Target::Quantity("platelets"),
        triggers: &["platelets", "platelet count", "plt"],
    },
    Concept {
        field: "weight",
        target: Target::Quantity("weight"),
        triggers: &["weight", "wt"],
    },
    Concept {
        field: "age",
        target: Target::Quantity("age"),
        triggers: &["age"],
    },
    Concept {
        field: "pao2",
        target: Target::Quantity("pao2"),
        triggers: &["pao2", "pa o2", "arterial po2"],
    },
    Concept {
        field: "fio2",
        target: Target::Quantity("fio2"),
        triggers: &["fio2", "fi o2"],
    },
    Concept {
        field: "respiratory_rate",
        target: Target::Quantity("rate_breaths"),
        triggers: &["respiratory rate", "resp rate", "rr"],
    },
    Concept {
        field: "heart_rate",
        target: Target::Quantity("rate_beats"),
        triggers: &["heart rate", "hr", "pulse"],
    },
    Concept {
        field: "spo2",
        target: Target::Quantity("spo2"),
        triggers: &["spo2", "sao2", "o2 sat", "oxygen saturation", "sats"],
    },
    Concept {
        field: "temperature",
        target: Target::Quantity("temperature"),
        triggers: &["temperature", "temp"],
    },
    // --- dimensionless ratio ---
    Concept {
        field: "inr",
        target: Target::Ratio,
        triggers: &["inr"],
    },
    // --- integer ---
    Concept {
        field: "gcs",
        target: Target::Integer,
        triggers: &["gcs", "glasgow coma"],
    },
    // --- sex --- (bare sex words act as their own triggers; "sex"/"gender" are labels that
    // introduce a value scanned just after them)
    Concept {
        field: "sex",
        target: Target::Sex,
        triggers: &["female", "woman", "male", "man", "sex", "gender"],
    },
    // --- boolean flags (presence asserts true; preceding negation asserts false) ---
    Concept {
        field: "dialysis",
        target: Target::Flag,
        triggers: &["dialysis", "hemodialysis", "haemodialysis", "crrt", "cvvhd"],
    },
    Concept {
        field: "confusion",
        target: Target::Flag,
        triggers: &[
            "confusion",
            "confused",
            "altered mental status",
            "disoriented",
        ],
    },
    Concept {
        field: "diabetes",
        target: Target::Flag,
        triggers: &["diabetes", "diabetic", "dm"],
    },
    Concept {
        field: "hypertension",
        target: Target::Flag,
        triggers: &["hypertension", "hypertensive", "htn"],
    },
];

/// Negation cues that, when immediately preceding a flag trigger, assert `false`.
const NEGATIONS: &[&str] = &[
    "no ",
    "not ",
    "without ",
    "denies ",
    "negative for ",
    "no history of ",
    "absent ",
    "nil ",
];

/// Run the scanner over `text`. Returns a structured result:
/// `{ inputs, needs_unit, ambiguous, flags, unrecognized_units, provenance, note, disclaimer }`.
///
/// `inputs` contains only values that are *ready to compute* (quantities with a recognized
/// unit, ratios, integers, sex, resolved flags). Values that need a unit are isolated in
/// `needs_unit` and deliberately excluded from `inputs`.
pub fn extract_inputs(text: &str) -> Value {
    let lower = text.to_lowercase();
    let bytes = lower.as_bytes();

    let mut inputs = Map::new();
    let mut needs_unit: Vec<Value> = Vec::new();
    let mut ambiguous: Vec<Value> = Vec::new();
    let mut unrecognized_units: Vec<Value> = Vec::new();
    let mut provenance: Vec<Value> = Vec::new();
    // Track which fields are already filled so the first confident hit wins and later weaker
    // (shorter-trigger) hits are reported as ambiguous rather than silently overwriting.
    let mut filled: Vec<&'static str> = Vec::new();

    // Collect all trigger occurrences (concept, field, trigger, match position, trigger len),
    // then sort so that, at any position, the longest trigger is considered first.
    let mut hits: Vec<Hit> = Vec::new();
    for (ci, c) in CONCEPTS.iter().enumerate() {
        for trig in c.triggers {
            let mut from = 0usize;
            while let Some(rel) = lower[from..].find(trig) {
                let start = from + rel;
                let end = start + trig.len();
                if is_word_bounded(bytes, start, end) {
                    hits.push(Hit {
                        concept_idx: ci,
                        start,
                        end,
                        trig_len: trig.len(),
                    });
                }
                from = start + 1;
            }
        }
    }
    // Longest trigger first; then earliest position. This makes "creatinine" beat "cr" and
    // "total bilirubin" beat "bili" at the same locus.
    hits.sort_by(|a, b| b.trig_len.cmp(&a.trig_len).then(a.start.cmp(&b.start)));

    // Suppress overlapping weaker hits (e.g. the "cr" inside "creatinine").
    let mut consumed: Vec<(usize, usize)> = Vec::new();

    for h in &hits {
        if consumed.iter().any(|&(s, e)| h.start < e && h.end > s) {
            continue;
        }
        let c = &CONCEPTS[h.concept_idx];

        match c.target {
            Target::Flag => {
                let negated = preceding_negation(&lower, h.start);
                if filled.contains(&c.field) {
                    continue;
                }
                inputs.insert(c.field.to_string(), json!(!negated));
                filled.push(c.field);
                consumed.push((h.start, h.end));
                provenance.push(json!({
                    "field": c.field,
                    "as": "flag",
                    "value": !negated,
                    "matched": &lower[h.start..h.end],
                    "negated": negated,
                }));
            }
            Target::Sex => {
                if filled.contains(&c.field) {
                    continue;
                }
                let matched = &lower[h.start..h.end];
                // A bare sex word is its own value; "sex"/"gender" introduce a value after.
                let resolved = match matched {
                    "female" | "woman" => Some(("female", h.start, h.end)),
                    "male" | "man" => Some(("male", h.start, h.end)),
                    _ => scan_sex(&lower, h.end),
                };
                if let Some((val, mstart, mend)) = resolved {
                    inputs.insert(c.field.to_string(), json!(val));
                    filled.push(c.field);
                    consumed.push((h.start, mend));
                    provenance.push(json!({
                        "field": c.field, "as": "enum", "value": val,
                        "matched": &lower[mstart..mend],
                    }));
                }
            }
            Target::Ratio | Target::Integer | Target::Quantity(_) => {
                let Some(num) = scan_number_after(&lower, h.end) else {
                    continue;
                };
                if filled.contains(&c.field) {
                    ambiguous.push(json!({
                        "field": c.field,
                        "reason": "multiple candidate values; first confident match kept",
                        "matched": &lower[h.start..num.value_end],
                    }));
                    continue;
                }
                consumed.push((h.start, num.value_end));

                match c.target {
                    Target::Ratio => {
                        inputs.insert(c.field.to_string(), json!(num.value));
                        filled.push(c.field);
                        provenance.push(json!({
                            "field": c.field, "as": "ratio", "value": num.value,
                            "matched": &lower[h.start..num.value_end],
                        }));
                    }
                    Target::Integer => {
                        if num.value.fract() == 0.0 {
                            inputs.insert(c.field.to_string(), json!(num.value as i64));
                            filled.push(c.field);
                            provenance.push(json!({
                                "field": c.field, "as": "integer", "value": num.value as i64,
                                "matched": &lower[h.start..num.value_end],
                            }));
                        }
                    }
                    Target::Quantity(analyte) => {
                        match num.unit {
                            Some(u) if units::is_known_unit(analyte, &u) => {
                                inputs.insert(
                                    c.field.to_string(),
                                    json!({ "value": num.value, "unit": u }),
                                );
                                filled.push(c.field);
                                provenance.push(json!({
                                    "field": c.field, "as": "quantity",
                                    "value": num.value, "unit": u, "analyte": analyte,
                                    "matched": &lower[h.start..num.value_end],
                                }));
                            }
                            Some(u) => {
                                // A unit was written but we don't recognize it for this
                                // analyte — never coerce; surface it.
                                unrecognized_units.push(json!({
                                    "field": c.field, "analyte": analyte,
                                    "value": num.value, "unit": u,
                                    "matched": &lower[h.start..num.value_end],
                                }));
                            }
                            None => {
                                // No unit at all: report, suggest the canonical unit to
                                // confirm, but DO NOT add to inputs.
                                needs_unit.push(json!({
                                    "field": c.field, "analyte": analyte,
                                    "value": num.value,
                                    "suggested_unit": units::canonical_unit(analyte),
                                    "matched": &lower[h.start..num.value_end],
                                }));
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    // Stable, readable ordering of provenance by source position.
    provenance.sort_by_key(|p| {
        p.get("matched")
            .and_then(Value::as_str)
            .and_then(|m| lower.find(m))
            .unwrap_or(usize::MAX)
    });

    json!({
        "inputs": Value::Object(inputs),
        "needs_unit": needs_unit,
        "unrecognized_units": unrecognized_units,
        "ambiguous": ambiguous,
        "provenance": provenance,
        "note": "Advisory extraction only. Values under 'needs_unit'/'unrecognized_units' were \
    NOT added to 'inputs' because Caliper never assumes a unit. Confirm units, then call \
    compute_score. This tool does not compute and does not diagnose.",
        "disclaimer": crate::DISCLAIMER,
    })
}

struct Hit {
    concept_idx: usize,
    start: usize,
    end: usize,
    trig_len: usize,
}

/// A number plus an optional unit scanned immediately after a trigger.
struct ScannedNumber {
    value: f64,
    unit: Option<String>,
    /// Byte offset just past the value (and unit, if any).
    value_end: usize,
}

/// True if `[start,end)` in `bytes` is bounded by non-alphanumeric chars (ASCII word bounds).
fn is_word_bounded(bytes: &[u8], start: usize, end: usize) -> bool {
    let before_ok = start == 0 || !is_word_byte(bytes[start - 1]);
    let after_ok = end >= bytes.len() || !is_word_byte(bytes[end]);
    before_ok && after_ok
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Check whether the text immediately before `pos` contains a negation cue (within a short
/// window, not crossing sentence punctuation).
fn preceding_negation(lower: &str, pos: usize) -> bool {
    let window_start = pos.saturating_sub(24);
    let prefix = &lower[window_start..pos];
    // Don't let a negation leak across a clause boundary.
    let clause = match prefix.rfind(['.', ',', ';', ':']) {
        Some(i) => &prefix[i + 1..],
        None => prefix,
    };
    NEGATIONS.iter().any(|n| clause.contains(n))
}

/// Scan a male/female token shortly after `pos`.
fn scan_sex(lower: &str, pos: usize) -> Option<(&'static str, usize, usize)> {
    let window_end = (pos + 16).min(lower.len());
    let window = &lower[pos..window_end];
    for (needle, val) in [
        ("female", "female"),
        ("woman", "female"),
        ("male", "male"),
        ("man", "male"),
        ("f", "female"),
        ("m", "male"),
    ] {
        if let Some(rel) = window.find(needle) {
            let s = pos + rel;
            let e = s + needle.len();
            if is_word_bounded(lower.as_bytes(), s, e) {
                return Some((val, s, e));
            }
        }
    }
    None
}

/// Scan a numeric value (and trailing unit token) starting at/after `pos`, skipping a short
/// run of separators like `: = ` and whitespace. Returns `None` if no number is found nearby.
fn scan_number_after(lower: &str, pos: usize) -> Option<ScannedNumber> {
    let b = lower.as_bytes();
    let mut i = pos;
    // Skip separators between the label and the number, but bail if it's clearly far away.
    let mut skipped = 0;
    while i < b.len() && skipped < 12 {
        let ch = b[i];
        if ch == b' ' || ch == b':' || ch == b'=' || ch == b'\t' || ch == b'~' || ch == b'(' {
            i += 1;
            skipped += 1;
        } else if lower[i..].starts_with("is ") || lower[i..].starts_with("of ") {
            // skip a connective word like "creatinine is 1.9" / "INR of 1.5"
            i += 3;
            skipped += 3;
        } else {
            break;
        }
    }
    // Optional leading sign.
    let num_start = i;
    if i < b.len() && (b[i] == b'-' || b[i] == b'+') {
        i += 1;
    }
    let mut saw_digit = false;
    let mut saw_dot = false;
    while i < b.len() {
        let ch = b[i];
        if ch.is_ascii_digit() {
            saw_digit = true;
            i += 1;
        } else if ch == b'.' && !saw_dot {
            saw_dot = true;
            i += 1;
        } else {
            break;
        }
    }
    if !saw_digit {
        return None;
    }
    let num_str = &lower[num_start..i];
    let value: f64 = num_str.parse().ok()?;

    // Scan an optional unit token immediately following (allowing one space).
    let mut j = i;
    if j < b.len() && b[j] == b' ' {
        j += 1;
    }
    let unit_start = j;
    while j < b.len() {
        let ch = b[j];
        // Unit chars: letters, digits, and the punctuation that appears in unit strings.
        if ch.is_ascii_alphanumeric()
            || matches!(
                ch,
                b'/' | b'%' | b'^' | b'.' | b'*' | b'-' | b'\xc2' | b'\xb5' | b'\xb0'
            )
        {
            j += 1;
        } else {
            break;
        }
    }
    let raw_unit = lower[unit_start..j].trim_matches(|c: char| c == '.' || c == '-');
    let (unit, value_end) = if raw_unit.is_empty() || raw_unit.chars().all(|c| c.is_ascii_digit()) {
        (None, i)
    } else {
        (Some(raw_unit.to_string()), j)
    };

    Some(ScannedNumber {
        value,
        unit,
        value_end,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj<'a>(v: &'a Value, key: &str) -> &'a Map<String, Value> {
        v.get(key).unwrap().as_object().unwrap()
    }
    fn arr<'a>(v: &'a Value, key: &str) -> &'a Vec<Value> {
        v.get(key).unwrap().as_array().unwrap()
    }

    #[test]
    fn extracts_quantity_with_unit() {
        let r = extract_inputs("Creatinine 1.9 mg/dL today");
        let cr = &obj(&r, "inputs")["creatinine"];
        assert_eq!(cr["value"], 1.9);
        assert_eq!(cr["unit"], "mg/dl");
    }

    #[test]
    fn value_without_unit_goes_to_needs_unit_not_inputs() {
        let r = extract_inputs("Sodium 130");
        assert!(!obj(&r, "inputs").contains_key("sodium"));
        let nu = arr(&r, "needs_unit");
        assert_eq!(nu.len(), 1);
        assert_eq!(nu[0]["field"], "sodium");
        assert_eq!(nu[0]["suggested_unit"], "mmol/L");
    }

    #[test]
    fn unrecognized_unit_is_isolated() {
        let r = extract_inputs("creatinine 1.9 furlongs");
        assert!(!obj(&r, "inputs").contains_key("creatinine"));
        let uu = arr(&r, "unrecognized_units");
        assert_eq!(uu.len(), 1);
        assert_eq!(uu[0]["unit"], "furlongs");
    }

    #[test]
    fn ratio_accepts_bare_number() {
        let r = extract_inputs("INR 1.5");
        assert_eq!(obj(&r, "inputs")["inr"], 1.5);
    }

    #[test]
    fn integer_field() {
        let r = extract_inputs("GCS 13");
        assert_eq!(obj(&r, "inputs")["gcs"], 13);
    }

    #[test]
    fn sex_resolves() {
        let r = extract_inputs("75 year old female with cirrhosis");
        assert_eq!(obj(&r, "inputs")["sex"], "female");
    }

    #[test]
    fn flag_presence_and_negation() {
        let yes = extract_inputs("patient on dialysis");
        assert_eq!(obj(&yes, "inputs")["dialysis"], true);
        let no = extract_inputs("no dialysis");
        assert_eq!(obj(&no, "inputs")["dialysis"], false);
    }

    #[test]
    fn longest_trigger_wins_over_substring() {
        // "creatinine" must win over the "cr" trigger at the same locus.
        let r = extract_inputs("creatinine 2.0 mg/dL");
        assert_eq!(obj(&r, "inputs")["creatinine"]["value"], 2.0);
    }

    #[test]
    fn umol_unit_is_recognized() {
        let r = extract_inputs("creatinine 150 umol/L");
        let cr = &obj(&r, "inputs")["creatinine"];
        assert_eq!(cr["value"], 150.0);
        assert_eq!(cr["unit"], "umol/l");
    }

    #[test]
    fn provenance_records_source_span() {
        let r = extract_inputs("Bilirubin 4.0 mg/dL");
        let prov = arr(&r, "provenance");
        assert!(prov
            .iter()
            .any(|p| p["field"] == "bilirubin" && p["matched"].as_str().unwrap().contains("4.0")));
    }
}
