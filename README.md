<p align="center">
  <img src="files/logo-lockup.svg" alt="Caliper" width="520">
</p>

# Caliper

**Deterministic, version-pinned, unit-typed clinical calculations over MCP.**

Caliper is a Rust [Model Context Protocol](https://modelcontextprotocol.io) server that exposes
a curated set of clinical scores and equations as tools over stdio. It is built for one job and
to do it precisely: take **unit-typed** inputs, apply a **version-pinned, cited** formula, and
return a number with its interpretation band, the rules that fired, and a citation — or a
**typed error**. It never guesses a unit, never substitutes a default, and never offers a
diagnosis or treatment.

> **Calculation only. Not medical advice; not a medical device.**

## Invariants

Every tool enforces these, every time:

1. **Unit-typed inputs.** Every physical quantity is `{ "value": <n>, "unit": "<u>" }`. A bare
   number where a unit is required is an error (`UnitRequired`); an unrecognized unit is
   `UnknownUnit`. Conversions are analyte-aware (creatinine ≠ bilirubin molar mass).
2. **No silent defaults.** A missing required input returns `MissingRequiredInput`. Caliper
   never fills in a value to make a calculation succeed.
3. **Versioned + cited.** Every result carries the exact formula `version` (e.g. `OPTN-2016`,
   `CKD-EPI-2021`) and a primary-source `citation`.
4. **Stateless.** No persistence, no PHI retention, no global mutable state, no logging of
   inputs. No network calls at runtime.
5. **Calculation only.** Every result carries a constant disclaimer. Interpretation bands are
   descriptive, never directive.

## Where Caliper sits in the pipeline

Caliper covers **ingest → compute**: it validates unit-typed inputs and returns a cited,
reproducible number with the rules that were applied. It deliberately stops there. Turning a
MELD of 26 or an eGFR of 78 into a decision is the clinician's job, not the tool's. Free-text /
lab-dump / PDF ingestion and anything resembling a recommendation are out of scope (see
[Roadmap](#roadmap)).

## Scores

| id | name | version | domain |
|----|------|---------|--------|
| `meld-na` | MELD-Na | OPTN-2016 | hepatology |
| `meld-3` | MELD 3.0 | OPTN-2023 | hepatology |
| `ckd-epi-2021` | eGFR (CKD-EPI creatinine, 2021 race-free) | CKD-EPI-2021 | renal |
| `cockcroft-gault` | Cockcroft-Gault creatinine clearance | 1976 | renal |
| `cha2ds2-vasc` | CHA2DS2-VASc | Lip-2010 | cardiology |
| `has-bled` | HAS-BLED | Pisters-2010 | cardiology |
| `curb-65` | CURB-65 | Lim-2003 | pulmonary |
| `wells-pe` | Wells Criteria (PE) | Wells-2000 | pulmonary |
| `news2` | NEWS2 | RCP-2017 | acute |
| `qsofa` | qSOFA | Sepsis-3-2016 | icu |
| `sofa` | SOFA | 1996 | icu |
| `gcs` | Glasgow Coma Scale | 1974 | neuro |
| `child-pugh` | Child-Pugh | 1973 | hepatology |
| `fib-4` | FIB-4 index | Sterling-2006 | hepatology |

See [docs/SCHEMA.md](docs/SCHEMA.md) for the full tool and input contract.

## Tools

| tool | purpose |
|------|---------|
| `list_scores` | list scores, filter by domain / free-text query |
| `score_inputs` | the input contract for a score (fields, units, clamps, notes) |
| `compute_score` | compute a score → `ScoreResult` or typed `CalcError` |
| `convert_units` | analyte-aware unit conversion with the conversion basis |
| `solve_for` | monotone bisection: find the input value that hits a target score |
| `score_series` | compute across a time series with per-point deltas and trends |
| `suggest_scores` | suggest candidate scores for a context (does **not** compute) |

## Quickstart

Requires a stable Rust toolchain.

```sh
cargo build --release
cargo test            # 26 tests, including one worked-example fixture per score
```

The server speaks newline-delimited JSON-RPC 2.0 on stdin/stdout. A minimal session:

```sh
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{}}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"compute_score","arguments":{"id":"gcs","inputs":{"eye":3,"verbal":4,"motor":5}}}}' \
  | ./target/release/caliper-mcp
```

More request/response pairs are in [examples/](examples/).

## Transport decision (rmcp vs hand-rolled)

Caliper hand-rolls the JSON-RPC 2.0 stdio loop rather than depending on the official `rmcp`
SDK. At build time `rmcp` 1.7.0 was the current stable release — stable and ergonomic — but it
brings an async/tokio runtime and a large transitive tree. For a calculation-only server whose
value is determinism, auditability, and a minimal trust surface, a synchronous
request → compute → response loop keeps the whole crate at two dependencies (`serde`,
`serde_json`), makes tool dispatch genuinely data-driven from the registry, and makes the full
`initialize → tools/call` round-trip a pure, unit-testable function. Protocol version pinned to
`2025-06-18`.

## Register with Claude Desktop (Windows)

Build the release binary, then add Caliper to
`%APPDATA%\Claude\claude_desktop_config.json` (create the file if it does not exist):

```json
{
  "mcpServers": {
    "caliper": {
      "command": "D:\\code\\caliper-mcp\\target\\release\\caliper-mcp.exe",
      "args": []
    }
  }
}
```

Use the absolute path to your built `caliper-mcp.exe`, escape backslashes, and restart Claude
Desktop. Caliper will appear as a tool provider; try *"list the renal scores"* or *"compute
MELD-Na for creatinine 1.9 mg/dL, bilirubin 4 mg/dL, INR 1.5, sodium 130 mmol/L."*

## Roadmap

Out of scope for `0.1.0`, noted here only:

- Free-text / lab-dump / PDF ingestion.
- Pediatric / weight-band scores beyond those listed.
- Anything beyond `score + interpretation band + citation` (no treatment or guideline
  directives).

## License

[Apache-2.0](LICENSE).
