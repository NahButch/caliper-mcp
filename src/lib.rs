//! # Caliper
//!
//! Deterministic, version-pinned, unit-typed clinical calculations exposed over the
//! Model Context Protocol (MCP).
//!
//! ## Invariants (enforced everywhere)
//!
//! 1. **Unit-typed inputs** — every physical quantity is `{ value, unit }`. A bare number
//!    where a unit is required is an error ([`CalcError::UnitRequired`]); an unrecognized
//!    unit is [`CalcError::UnknownUnit`].
//! 2. **No silent defaults** — a missing required input is
//!    [`CalcError::MissingRequiredInput`]; no default is ever substituted to make a
//!    calculation succeed.
//! 3. **Versioned + cited** — every [`ScoreResult`] carries the exact formula `version`
//!    and a `citation`.
//! 4. **Stateless** — no persistence, no PHI retention, no global mutable state, no logging
//!    of inputs.
//! 5. **Calculation only** — never a diagnosis or treatment recommendation. Every result
//!    carries [`DISCLAIMER`].

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub mod mcp;
pub mod registry;
pub mod scores;
pub mod tools;
pub mod units;

/// The constant disclaimer attached to every successful calculation.
pub const DISCLAIMER: &str = "Calculation only. Not medical advice; not a medical device.";

/// MCP protocol revision this server speaks.
pub const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

/// This crate's version, surfaced over MCP `initialize`.
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A physical quantity: a magnitude paired with an explicit unit.
///
/// Inputs are never assumed to be in a canonical unit; the [`units`] module converts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Quantity {
    pub value: f64,
    pub unit: String,
}

/// The result of a successful calculation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoreResult {
    /// Score id, e.g. `"meld-na"`.
    pub id: String,
    /// Exact formula version string, e.g. `"OPTN-2016"`.
    pub version: String,
    /// The numeric result.
    pub value: f64,
    /// The unit of `value` (e.g. `"points"`, `"mL/min/1.73m^2"`).
    pub unit: String,
    /// Human-readable interpretation band (descriptive, never directive).
    pub interpretation: String,
    /// Audit trail of clamps/floors/overrides that fired during the calculation.
    pub applied_rules: Vec<String>,
    /// Primary-source citation for the formula.
    pub citation: String,
    /// Constant [`DISCLAIMER`].
    pub disclaimer: String,
}

impl ScoreResult {
    /// Build a result, stamping the constant disclaimer.
    pub fn new(
        id: &str,
        version: &str,
        value: f64,
        unit: &str,
        interpretation: String,
        applied_rules: Vec<String>,
        citation: &str,
    ) -> Self {
        ScoreResult {
            id: id.to_string(),
            version: version.to_string(),
            value,
            unit: unit.to_string(),
            interpretation,
            applied_rules,
            citation: citation.to_string(),
            disclaimer: DISCLAIMER.to_string(),
        }
    }
}

/// A typed calculation error. Serializes with an `"error"` tag discriminator.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "error")]
pub enum CalcError {
    /// A required input field was absent.
    MissingRequiredInput { field: String },
    /// A field that must be a unit-typed quantity was supplied without a unit
    /// (e.g. a bare number).
    UnitRequired { field: String },
    /// The supplied unit is not recognized for the field's analyte/dimension.
    UnknownUnit { field: String, unit: String },
    /// A value was outside the permitted domain (e.g. a categorical not in the allowed set,
    /// or a physiologically impossible magnitude).
    OutOfRange { field: String, message: String },
}

impl std::fmt::Display for CalcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalcError::MissingRequiredInput { field } => {
                write!(f, "missing required input: {field}")
            }
            CalcError::UnitRequired { field } => {
                write!(
                    f,
                    "field '{field}' requires an explicit unit (got a bare number)"
                )
            }
            CalcError::UnknownUnit { field, unit } => {
                write!(f, "field '{field}': unknown unit '{unit}'")
            }
            CalcError::OutOfRange { field, message } => {
                write!(f, "field '{field}' out of range: {message}")
            }
        }
    }
}

impl std::error::Error for CalcError {}

/// What kind of input a field accepts. Used by the registry to publish the input contract
/// and by [`Inputs`] to enforce it.
#[derive(Debug, Clone, Copy)]
pub enum InputKind {
    /// A unit-typed physical quantity converted via the named analyte/dimension.
    Quantity {
        analyte: &'static str,
        canonical: &'static str,
    },
    /// A dimensionless ratio (e.g. INR). Accepts a bare number or `{value, unit:"ratio"}`.
    Ratio,
    /// A boolean flag. Accepts JSON `true`/`false` or the strings `"yes"`/`"no"`.
    Bool,
    /// One of an enumerated set of string values.
    Enum(&'static [&'static str]),
    /// An integer in an inclusive range (e.g. a GCS sub-score).
    Integer { min: i64, max: i64 },
}

/// A single field in a score's input contract.
#[derive(Debug, Clone, Copy)]
pub struct InputSpec {
    pub field: &'static str,
    pub kind: InputKind,
    pub required: bool,
    /// Allowed unit strings (for `Quantity`/`Ratio`); empty otherwise.
    pub allowed_units: &'static [&'static str],
    pub notes: &'static str,
    /// Optional floor applied to the canonical value before use (recorded in `applied_rules`).
    pub floor: Option<f64>,
    /// Optional ceiling applied to the canonical value before use.
    pub ceiling: Option<f64>,
}

impl InputSpec {
    /// A required unit-typed quantity.
    pub const fn quantity(
        field: &'static str,
        analyte: &'static str,
        canonical: &'static str,
        allowed_units: &'static [&'static str],
        notes: &'static str,
    ) -> Self {
        InputSpec {
            field,
            kind: InputKind::Quantity { analyte, canonical },
            required: true,
            allowed_units,
            notes,
            floor: None,
            ceiling: None,
        }
    }
    pub const fn ratio(field: &'static str, notes: &'static str) -> Self {
        InputSpec {
            field,
            kind: InputKind::Ratio,
            required: true,
            allowed_units: &["ratio", "(dimensionless)"],
            notes,
            floor: None,
            ceiling: None,
        }
    }
    pub const fn boolean(field: &'static str, notes: &'static str) -> Self {
        InputSpec {
            field,
            kind: InputKind::Bool,
            required: true,
            allowed_units: &[],
            notes,
            floor: None,
            ceiling: None,
        }
    }
    pub const fn enumerated(
        field: &'static str,
        allowed: &'static [&'static str],
        notes: &'static str,
    ) -> Self {
        InputSpec {
            field,
            kind: InputKind::Enum(allowed),
            required: true,
            allowed_units: &[],
            notes,
            floor: None,
            ceiling: None,
        }
    }
    pub const fn integer(field: &'static str, min: i64, max: i64, notes: &'static str) -> Self {
        InputSpec {
            field,
            kind: InputKind::Integer { min, max },
            required: true,
            allowed_units: &[],
            notes,
            floor: None,
            ceiling: None,
        }
    }
    pub const fn optional(mut self) -> Self {
        self.required = false;
        self
    }
    pub const fn with_floor(mut self, floor: f64) -> Self {
        self.floor = Some(floor);
        self
    }
    pub const fn with_ceiling(mut self, ceiling: f64) -> Self {
        self.ceiling = Some(ceiling);
        self
    }
}

/// Static description of a score: metadata, input contract, and the compute function.
/// Everything `list_scores`/`score_inputs`/`suggest_scores` reports is derived from this.
#[derive(Clone, Copy)]
pub struct ScoreDescriptor {
    pub id: &'static str,
    pub name: &'static str,
    pub version: &'static str,
    pub citation: &'static str,
    pub domain: &'static str,
    /// Short keywords used by `suggest_scores` to match free-text context.
    pub keywords: &'static [&'static str],
    pub unit: &'static str,
    pub inputs: &'static [InputSpec],
    pub compute: fn(&Inputs) -> Result<ScoreResult, CalcError>,
}

/// The `Score` trait. The registry stores function pointers, but this trait documents the
/// contract and allows scores to be referred to abstractly.
pub trait Score {
    fn descriptor() -> ScoreDescriptor;
}

/// A thin, validating view over a JSON object of inputs.
///
/// All accessors enforce the unit/required invariants and return [`CalcError`] on violation.
pub struct Inputs<'a> {
    map: &'a Map<String, Value>,
}

impl<'a> Inputs<'a> {
    pub fn new(map: &'a Map<String, Value>) -> Self {
        Inputs { map }
    }

    /// Borrow the raw value for a field, if present.
    pub fn raw(&self, field: &str) -> Option<&Value> {
        self.map.get(field)
    }

    fn require(&self, field: &str) -> Result<&Value, CalcError> {
        self.map.get(field).filter(|v| !v.is_null()).ok_or_else(|| {
            CalcError::MissingRequiredInput {
                field: field.to_string(),
            }
        })
    }

    /// A required unit-typed quantity, converted to its canonical unit.
    pub fn quantity(&self, field: &str, analyte: &str) -> Result<f64, CalcError> {
        let v = self.require(field)?;
        let obj = v.as_object().ok_or_else(|| {
            // A bare number (or any non-object) is the classic "unit required" failure.
            CalcError::UnitRequired {
                field: field.to_string(),
            }
        })?;
        let value = obj.get("value").and_then(Value::as_f64).ok_or_else(|| {
            CalcError::MissingRequiredInput {
                field: format!("{field}.value"),
            }
        })?;
        let unit = obj
            .get("unit")
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| CalcError::UnitRequired {
                field: field.to_string(),
            })?;
        units::to_canonical(analyte, value, unit, field)
    }

    /// An optional unit-typed quantity. `Ok(None)` if absent; still validated if present.
    pub fn opt_quantity(&self, field: &str, analyte: &str) -> Result<Option<f64>, CalcError> {
        if self.map.get(field).filter(|v| !v.is_null()).is_none() {
            return Ok(None);
        }
        self.quantity(field, analyte).map(Some)
    }

    /// A dimensionless ratio (INR, etc). Accepts a bare number or `{value, unit:"ratio"}`.
    pub fn ratio(&self, field: &str) -> Result<f64, CalcError> {
        let v = self.require(field)?;
        if let Some(n) = v.as_f64() {
            return Ok(n);
        }
        if let Some(obj) = v.as_object() {
            if let Some(n) = obj.get("value").and_then(Value::as_f64) {
                return Ok(n);
            }
            return Err(CalcError::MissingRequiredInput {
                field: format!("{field}.value"),
            });
        }
        Err(CalcError::OutOfRange {
            field: field.to_string(),
            message: "expected a dimensionless number".to_string(),
        })
    }

    /// A boolean. Accepts JSON booleans and the strings yes/no/true/false (case-insensitive).
    pub fn boolean(&self, field: &str) -> Result<bool, CalcError> {
        let v = self.require(field)?;
        if let Some(b) = v.as_bool() {
            return Ok(b);
        }
        if let Some(s) = v.as_str() {
            match s.trim().to_ascii_lowercase().as_str() {
                "yes" | "true" | "y" | "1" => return Ok(true),
                "no" | "false" | "n" | "0" => return Ok(false),
                _ => {}
            }
        }
        Err(CalcError::OutOfRange {
            field: field.to_string(),
            message: "expected a boolean (true/false or yes/no)".to_string(),
        })
    }

    /// One of an allowed enumerated set. Matched case-insensitively; returns the canonical
    /// (allowed-list) spelling.
    pub fn enum_one(&self, field: &str, allowed: &[&str]) -> Result<String, CalcError> {
        let v = self.require(field)?;
        let s = v.as_str().ok_or_else(|| CalcError::OutOfRange {
            field: field.to_string(),
            message: format!("expected one of {allowed:?}"),
        })?;
        let needle = s.trim().to_ascii_lowercase();
        for &a in allowed {
            if a.to_ascii_lowercase() == needle {
                return Ok(a.to_string());
            }
        }
        Err(CalcError::OutOfRange {
            field: field.to_string(),
            message: format!("'{s}' is not one of {allowed:?}"),
        })
    }

    /// An integer constrained to `[min, max]`.
    pub fn integer(&self, field: &str, min: i64, max: i64) -> Result<i64, CalcError> {
        let v = self.require(field)?;
        let n = v
            .as_i64()
            .or_else(|| v.as_f64().filter(|f| f.fract() == 0.0).map(|f| f as i64))
            .ok_or_else(|| CalcError::OutOfRange {
                field: field.to_string(),
                message: "expected an integer".to_string(),
            })?;
        if n < min || n > max {
            return Err(CalcError::OutOfRange {
                field: field.to_string(),
                message: format!("must be in [{min}, {max}], got {n}"),
            });
        }
        Ok(n)
    }
}

/// Apply an optional floor, recording the rule if it fired.
pub fn apply_floor(value: f64, floor: f64, label: &str, rules: &mut Vec<String>) -> f64 {
    if value < floor {
        rules.push(format!("{label}: floored {value} -> {floor}"));
        floor
    } else {
        value
    }
}

/// Apply an optional ceiling, recording the rule if it fired.
pub fn apply_ceiling(value: f64, ceiling: f64, label: &str, rules: &mut Vec<String>) -> f64 {
    if value > ceiling {
        rules.push(format!("{label}: capped {value} -> {ceiling}"));
        ceiling
    } else {
        value
    }
}

/// Clamp into `[lo, hi]`, recording the rule if either bound bit.
pub fn apply_clamp(value: f64, lo: f64, hi: f64, label: &str, rules: &mut Vec<String>) -> f64 {
    if value < lo {
        rules.push(format!("{label}: clamped {value} -> {lo}"));
        lo
    } else if value > hi {
        rules.push(format!("{label}: clamped {value} -> {hi}"));
        hi
    } else {
        value
    }
}
