//! Data-driven fixture runner. Each `tests/fixtures/<score-id>.json` is a worked example of
//! the form `{ inputs, expected, source, tolerance? }`. The filename stem is the score id.
//!
//! Integer scores assert exact equality (tolerance 0); continuous scores assert to the
//! tolerance written in the fixture.

use caliper::{registry, Inputs};
use serde_json::Value;
use std::fs;
use std::path::Path;

#[test]
fn all_fixtures_match() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut count = 0usize;
    let mut failures: Vec<String> = Vec::new();

    let mut entries: Vec<_> = fs::read_dir(&dir)
        .expect("read tests/fixtures")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();
    entries.sort();

    for path in entries {
        let id = path.file_stem().unwrap().to_str().unwrap().to_string();
        let raw = fs::read_to_string(&path).unwrap();
        let fx: Value =
            serde_json::from_str(&raw).unwrap_or_else(|e| panic!("{id}: invalid JSON: {e}"));

        let inputs_obj = fx
            .get("inputs")
            .and_then(Value::as_object)
            .unwrap_or_else(|| panic!("{id}: missing 'inputs' object"))
            .clone();
        let expected = fx
            .get("expected")
            .and_then(Value::as_f64)
            .unwrap_or_else(|| panic!("{id}: missing numeric 'expected'"));
        let tol = fx.get("tolerance").and_then(Value::as_f64).unwrap_or(0.0);
        assert!(
            fx.get("source").and_then(Value::as_str).is_some(),
            "{id}: fixture must cite a 'source'"
        );

        let desc = registry::find(&id)
            .unwrap_or_else(|| panic!("fixture filename '{id}' does not match any score id"));
        let inputs = Inputs::new(&inputs_obj);
        match (desc.compute)(&inputs) {
            Ok(r) => {
                if (r.value - expected).abs() > tol {
                    failures.push(format!(
                        "{id}: expected {expected} (tol {tol}), got {}",
                        r.value
                    ));
                }
                count += 1;
            }
            Err(e) => failures.push(format!("{id}: unexpected compute error: {e:?}")),
        }
    }

    assert!(
        count >= registry::all().len(),
        "expected at least one fixture per score ({} scores), ran {count}",
        registry::all().len()
    );
    assert!(
        failures.is_empty(),
        "fixture mismatches:\n{}",
        failures.join("\n")
    );
}
