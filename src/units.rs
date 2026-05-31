//! Analyte-aware unit conversion.
//!
//! Conversions are *analyte-specific*: converting creatinine between mg/dL and µmol/L uses
//! creatinine's molar mass, which differs from bilirubin's. Each analyte declares a
//! canonical unit and a table of accepted units with their multiplicative factor *to* the
//! canonical unit. Temperature is affine and handled specially.
//!
//! Nothing here touches the filesystem, the network, or global state.

use crate::CalcError;

/// A converted value plus an audit of how it was produced.
#[derive(Debug, Clone, PartialEq)]
pub struct Converted {
    pub value: f64,
    /// Multiplicative factor applied (or the affine result's effective ratio for temperature).
    pub factor: f64,
    pub from: String,
    pub to: String,
    /// Human-readable basis for the conversion (e.g. molar mass).
    pub basis: String,
}

struct UnitDef {
    /// Lowercased aliases for this unit.
    aliases: &'static [&'static str],
    /// Multiply a value in this unit by `factor` to get the canonical unit.
    factor: f64,
}

struct AnalyteDef {
    name: &'static str,
    canonical: &'static str,
    basis: &'static str,
    units: &'static [UnitDef],
}

const ANALYTES: &[AnalyteDef] = &[
    AnalyteDef {
        name: "creatinine",
        canonical: "mg/dL",
        basis: "creatinine molar mass 113.12 g/mol; 1 mg/dL = 88.42 µmol/L",
        units: &[
            UnitDef {
                aliases: &["mg/dl"],
                factor: 1.0,
            },
            UnitDef {
                aliases: &["mg/l"],
                factor: 0.1,
            },
            UnitDef {
                aliases: &["umol/l", "µmol/l", "micromol/l"],
                factor: 1.0 / 88.42,
            },
        ],
    },
    AnalyteDef {
        name: "bilirubin",
        canonical: "mg/dL",
        basis: "bilirubin molar mass 584.66 g/mol; 1 mg/dL = 17.104 µmol/L",
        units: &[
            UnitDef {
                aliases: &["mg/dl"],
                factor: 1.0,
            },
            UnitDef {
                aliases: &["mg/l"],
                factor: 0.1,
            },
            UnitDef {
                aliases: &["umol/l", "µmol/l", "micromol/l"],
                factor: 1.0 / 17.104,
            },
        ],
    },
    AnalyteDef {
        name: "sodium",
        canonical: "mmol/L",
        basis: "sodium is monovalent; mmol/L = mEq/L",
        units: &[
            UnitDef {
                aliases: &["mmol/l"],
                factor: 1.0,
            },
            UnitDef {
                aliases: &["meq/l"],
                factor: 1.0,
            },
        ],
    },
    AnalyteDef {
        name: "urea",
        canonical: "mmol/L",
        basis: "mg/dL is interpreted as blood urea nitrogen (BUN); BUN mg/dL x 0.357 = urea mmol/L",
        units: &[
            UnitDef {
                aliases: &["mmol/l"],
                factor: 1.0,
            },
            UnitDef {
                aliases: &["mg/dl", "mg/dl bun", "bun mg/dl"],
                factor: 0.357,
            },
        ],
    },
    AnalyteDef {
        name: "albumin",
        canonical: "g/dL",
        basis: "1 g/dL = 10 g/L",
        units: &[
            UnitDef {
                aliases: &["g/dl"],
                factor: 1.0,
            },
            UnitDef {
                aliases: &["g/l"],
                factor: 0.1,
            },
        ],
    },
    AnalyteDef {
        name: "weight",
        canonical: "kg",
        basis: "1 lb = 0.45359237 kg",
        units: &[
            UnitDef {
                aliases: &["kg"],
                factor: 1.0,
            },
            UnitDef {
                aliases: &["g"],
                factor: 0.001,
            },
            UnitDef {
                aliases: &["lb", "lbs", "pound", "pounds"],
                factor: 0.45359237,
            },
        ],
    },
    AnalyteDef {
        name: "age",
        canonical: "years",
        basis: "12 months = 1 year",
        units: &[
            UnitDef {
                aliases: &["years", "year", "yr", "yrs", "y", "a"],
                factor: 1.0,
            },
            UnitDef {
                aliases: &["months", "month", "mo"],
                factor: 1.0 / 12.0,
            },
        ],
    },
    AnalyteDef {
        name: "platelets",
        canonical: "10^9/L",
        basis: "10^9/L = 10^3/µL (= K/µL)",
        units: &[
            UnitDef {
                aliases: &["10^9/l", "x10^9/l", "*10^9/l", "10e9/l", "g/l-platelets"],
                factor: 1.0,
            },
            UnitDef {
                aliases: &["10^3/ul", "10^3/µl", "k/ul", "k/µl", "/nl", "thousand/ul"],
                factor: 1.0,
            },
            UnitDef {
                aliases: &["10^6/l", "x10^6/l"],
                factor: 0.001,
            },
            UnitDef {
                aliases: &["/l", "cells/l"],
                factor: 1.0e-9,
            },
            UnitDef {
                aliases: &["/ul", "cells/ul", "/µl"],
                factor: 1.0e-3,
            },
        ],
    },
    AnalyteDef {
        name: "aminotransferase",
        canonical: "U/L",
        basis: "U/L = IU/L for aminotransferases",
        units: &[UnitDef {
            aliases: &["u/l", "iu/l", "units/l"],
            factor: 1.0,
        }],
    },
    AnalyteDef {
        name: "pao2",
        canonical: "mmHg",
        basis: "1 kPa = 7.50062 mmHg",
        units: &[
            UnitDef {
                aliases: &["mmhg", "torr"],
                factor: 1.0,
            },
            UnitDef {
                aliases: &["kpa"],
                factor: 7.50062,
            },
        ],
    },
    AnalyteDef {
        name: "fio2",
        canonical: "fraction",
        basis: "FiO2 as a fraction in [0,1]; percent divided by 100",
        units: &[
            UnitDef {
                aliases: &["fraction", "ratio", "frac"],
                factor: 1.0,
            },
            UnitDef {
                aliases: &["%", "percent", "pct"],
                factor: 0.01,
            },
        ],
    },
    AnalyteDef {
        name: "pressure",
        canonical: "mmHg",
        basis: "blood pressure in mmHg",
        units: &[UnitDef {
            aliases: &["mmhg", "torr"],
            factor: 1.0,
        }],
    },
    AnalyteDef {
        name: "rate_breaths",
        canonical: "breaths/min",
        basis: "respiratory rate per minute",
        units: &[UnitDef {
            aliases: &["breaths/min", "/min", "brpm", "rpm", "bpm"],
            factor: 1.0,
        }],
    },
    AnalyteDef {
        name: "rate_beats",
        canonical: "bpm",
        basis: "heart rate per minute",
        units: &[UnitDef {
            aliases: &["bpm", "beats/min", "/min"],
            factor: 1.0,
        }],
    },
    AnalyteDef {
        name: "spo2",
        canonical: "%",
        basis: "peripheral oxygen saturation, percent",
        units: &[UnitDef {
            aliases: &["%", "percent", "pct"],
            factor: 1.0,
        }],
    },
    AnalyteDef {
        name: "urine_output",
        canonical: "mL/day",
        basis: "urine output volume per 24 hours",
        units: &[UnitDef {
            aliases: &["ml/day", "ml/24h", "ml/d", "cc/day"],
            factor: 1.0,
        }],
    },
];

/// Normalize a unit string for matching: trim, lowercase, collapse internal spaces,
/// and map common micro/degree glyphs.
fn normalize(unit: &str) -> String {
    let mut s = unit.trim().to_ascii_lowercase();
    s = s.replace('μ', "µ"); // GREEK SMALL MU -> MICRO SIGN
    s = s.replace("micro", "µ"); // only after the explicit "micromol/l" alias is also listed
    s = s.replace(' ', "");
    s
}

fn find_analyte(analyte: &str) -> Option<&'static AnalyteDef> {
    ANALYTES.iter().find(|a| a.name == analyte)
}

/// Convert a value of `analyte` from unit `from` to unit `to`, returning the result with an
/// audit ([`Converted`]). Used by the `convert_units` tool.
pub fn convert(analyte: &str, value: f64, from: &str, to: &str) -> Result<Converted, CalcError> {
    if analyte.eq_ignore_ascii_case("temperature") {
        return convert_temperature(value, from, to);
    }
    let def = find_analyte(analyte).ok_or_else(|| CalcError::OutOfRange {
        field: "analyte".to_string(),
        message: format!("unknown analyte '{analyte}'"),
    })?;
    let from_factor = unit_factor(def, from).ok_or_else(|| CalcError::UnknownUnit {
        field: "from".to_string(),
        unit: from.to_string(),
    })?;
    let to_factor = unit_factor(def, to).ok_or_else(|| CalcError::UnknownUnit {
        field: "to".to_string(),
        unit: to.to_string(),
    })?;
    // value -> canonical -> target
    let canonical = value * from_factor;
    let out = canonical / to_factor;
    Ok(Converted {
        value: out,
        factor: from_factor / to_factor,
        from: from.to_string(),
        to: to.to_string(),
        basis: def.basis.to_string(),
    })
}

fn unit_factor(def: &AnalyteDef, unit: &str) -> Option<f64> {
    let n = normalize(unit);
    // The "fraction"/empty unit for FiO2: allow empty string to mean fraction.
    for u in def.units {
        if u.aliases.iter().any(|a| normalize(a) == n) {
            return Some(u.factor);
        }
    }
    None
}

/// Convert a unit-typed input value to the analyte's canonical unit, mapping failures onto a
/// specific field name for clean error reporting.
pub fn to_canonical(analyte: &str, value: f64, unit: &str, field: &str) -> Result<f64, CalcError> {
    if analyte.eq_ignore_ascii_case("temperature") {
        return to_celsius(value, unit, field);
    }
    let def = find_analyte(analyte).ok_or_else(|| CalcError::OutOfRange {
        field: field.to_string(),
        message: format!("no unit table for analyte '{analyte}'"),
    })?;
    let factor = unit_factor(def, unit).ok_or_else(|| CalcError::UnknownUnit {
        field: field.to_string(),
        unit: unit.to_string(),
    })?;
    Ok(value * factor)
}

/// Canonical unit string for an analyte (used by docs/contracts).
pub fn canonical_unit(analyte: &str) -> Option<&'static str> {
    if analyte.eq_ignore_ascii_case("temperature") {
        return Some("°C");
    }
    find_analyte(analyte).map(|a| a.canonical)
}

/// Whether `unit` is a recognized unit for `analyte`, without converting. Used by the
/// ingestion scanner to tell "found a known unit" from "found an unrecognized unit" — it
/// never fabricates or assumes a unit on the caller's behalf.
pub fn is_known_unit(analyte: &str, unit: &str) -> bool {
    if analyte.eq_ignore_ascii_case("temperature") {
        return matches!(
            normalize(unit).as_str(),
            "°c" | "c" | "celsius" | "degc" | "°f" | "f" | "fahrenheit" | "degf" | "k" | "kelvin"
        );
    }
    find_analyte(analyte)
        .map(|def| unit_factor(def, unit).is_some())
        .unwrap_or(false)
}

fn to_celsius(value: f64, unit: &str, field: &str) -> Result<f64, CalcError> {
    match normalize(unit).as_str() {
        "°c" | "c" | "celsius" | "degc" => Ok(value),
        "°f" | "f" | "fahrenheit" | "degf" => Ok((value - 32.0) / 1.8),
        "k" | "kelvin" => Ok(value - 273.15),
        _ => Err(CalcError::UnknownUnit {
            field: field.to_string(),
            unit: unit.to_string(),
        }),
    }
}

fn convert_temperature(value: f64, from: &str, to: &str) -> Result<Converted, CalcError> {
    let celsius = to_celsius(value, from, "from")?;
    let out = match normalize(to).as_str() {
        "°c" | "c" | "celsius" | "degc" => celsius,
        "°f" | "f" | "fahrenheit" | "degf" => celsius * 1.8 + 32.0,
        "k" | "kelvin" => celsius + 273.15,
        _ => {
            return Err(CalcError::UnknownUnit {
                field: "to".to_string(),
                unit: to.to_string(),
            })
        }
    };
    let factor = if value != 0.0 { out / value } else { f64::NAN };
    Ok(Converted {
        value: out,
        factor,
        from: from.to_string(),
        to: to.to_string(),
        basis: "temperature is affine: °C = (°F − 32) / 1.8; K = °C + 273.15".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-6, "{a} != {b}");
    }

    #[test]
    fn creatinine_umol_to_mgdl() {
        // 88.42 µmol/L == 1.0 mg/dL
        approx(
            to_canonical("creatinine", 88.42, "umol/L", "creatinine").unwrap(),
            1.0,
        );
    }

    #[test]
    fn bilirubin_umol_to_mgdl() {
        approx(
            to_canonical("bilirubin", 17.104, "µmol/L", "bilirubin").unwrap(),
            1.0,
        );
    }

    #[test]
    fn sodium_meq_equals_mmol() {
        approx(
            to_canonical("sodium", 137.0, "mEq/L", "sodium").unwrap(),
            137.0,
        );
    }

    #[test]
    fn weight_lb_to_kg() {
        // 1 kg = 2.2046226218 lb exactly (inverse of 0.45359237).
        approx(
            to_canonical("weight", 2.2046226218, "lb", "weight").unwrap(),
            1.0,
        );
    }

    #[test]
    fn temperature_f_to_c() {
        approx(
            to_canonical("temperature", 98.6, "°F", "temperature").unwrap(),
            37.0,
        );
    }

    #[test]
    fn convert_round_trip_factor() {
        let c = convert("creatinine", 1.0, "mg/dL", "umol/L").unwrap();
        approx(c.value, 88.42);
    }

    #[test]
    fn unknown_unit_is_typed_error() {
        let e = to_canonical("creatinine", 1.0, "furlongs", "creatinine").unwrap_err();
        assert!(matches!(e, CalcError::UnknownUnit { .. }));
    }

    #[test]
    fn fio2_percent_to_fraction() {
        approx(to_canonical("fio2", 40.0, "%", "fio2").unwrap(), 0.40);
    }
}
