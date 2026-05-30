# Changelog

All notable changes to Caliper are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-05-30

Initial release.

### Added
- MCP stdio server (hand-rolled JSON-RPC 2.0, protocol version `2025-06-18`); dependencies
  limited to `serde` and `serde_json`.
- Core types: `Quantity`, `ScoreResult`, serde-tagged `CalcError`
  (`MissingRequiredInput` / `UnitRequired` / `UnknownUnit` / `OutOfRange`), and a validating
  `Inputs` accessor enforcing the unit/required invariants.
- Analyte-aware unit conversion (creatinine, bilirubin, sodium, urea/BUN, albumin, weight,
  age, platelets, aminotransferases, PaO2, FiO2, pressures, rates, SpO2, affine temperature).
- Data-driven registry over 14 scores: `meld-na`, `meld-3`, `ckd-epi-2021`,
  `cockcroft-gault`, `cha2ds2-vasc`, `has-bled`, `curb-65`, `wells-pe`, `news2`, `qsofa`,
  `sofa`, `gcs`, `child-pugh`, `fib-4`. MELD floors/caps/dialysis overrides and Na/albumin
  clamps are applied and recorded in `applied_rules`.
- Seven tools: `list_scores`, `score_inputs`, `compute_score`, `convert_units`, `solve_for`,
  `score_series`, `suggest_scores`.
- One worked-example fixture per score with a data-driven runner; unit-discipline,
  `solve_for`, `score_series`, and server round-trip tests (26 tests total).
- Docs (`docs/SCHEMA.md`), `examples/` request/response pairs, and CI
  (`fmt --check`, `clippy -D warnings`, `test` on stable).

### Verified
- Coefficient audit ([`docs/COEFFICIENT_AUDIT.md`](docs/COEFFICIENT_AUDIT.md)): every coefficient,
  threshold, clamp, and order of operations in all 14 scores checked against primary sources
  (cross-checked with MDCalc and other authoritative calculators); all 8 documented worked
  examples independently recomputed. Result: 14/14 clean, no discrepancies.
