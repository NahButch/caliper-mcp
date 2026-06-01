# Caliper schema

This document is the contract for Caliper's MCP tools and score inputs. It mirrors the
registry the server uses at runtime; `list_scores` / `score_inputs` return the
machine-readable form of everything below.

- **MCP protocol version:** `2025-06-18`
- **Transport:** newline-delimited JSON-RPC 2.0 over stdio
- **Server name / version:** `caliper-mcp` / `0.1.0`

## Invariants

1. **Unit-typed inputs.** Every physical quantity is an object `{ "value": <number>, "unit": "<string>" }`.
   A bare number where a unit is required returns `UnitRequired`.
2. **No silent defaults.** A missing required input returns `MissingRequiredInput`. No default
   is ever substituted to make a calculation succeed.
3. **Versioned + cited.** Every result carries the exact `version` string and a `citation`.
4. **Stateless.** No persistence, no PHI retention, no global mutable state, no logging of inputs.
5. **Calculation only.** Every result carries the disclaimer
   `"Calculation only. Not medical advice; not a medical device."`

## Core types

### Quantity
```json
{ "value": 1.9, "unit": "mg/dL" }
```

### ScoreResult
```json
{
  "id": "meld-na",
  "version": "OPTN-2016",
  "value": 26.0,
  "unit": "points",
  "interpretation": "MELD-Na 26: elevated band (descriptive only).",
  "applied_rules": ["MELD(i) > 11: sodium correction applied"],
  "citation": "OPTN Policy 9 ...",
  "disclaimer": "Calculation only. Not medical advice; not a medical device."
}
```

### CalcError
Serialized with an `"error"` discriminator:

| `error` | extra fields | meaning |
|---|---|---|
| `MissingRequiredInput` | `field` | a required input was absent |
| `UnitRequired` | `field` | a quantity field was given without a unit (e.g. a bare number) |
| `UnknownUnit` | `field`, `unit` | the unit is not recognized for that analyte |
| `OutOfRange` | `field`, `message` | value outside the permitted domain (bad enum, impossible magnitude) |

## Input kinds

`score_inputs` returns one descriptor per field. The `kind` is one of:

| kind | JSON value shape | notes |
|---|---|---|
| `quantity` | `{ "value", "unit" }` | converted to the analyte's canonical unit; `allowed_units` lists accepted units |
| `ratio` | number, or `{ "value", "unit": "ratio" }` | dimensionless (e.g. INR) |
| `boolean` | `true`/`false` or `"yes"`/`"no"` | |
| `enum` | string | must be one of `allowed` (case-insensitive) |
| `integer` | integer | constrained to `[min, max]` |

Quantity fields may also carry `floor` / `ceiling`: clamps applied to the canonical value
before use and recorded in `applied_rules`.

## Tools

### `list_scores(domain?, query?)`
Returns `{ count, domains, scores: [ {id, name, version, domain, unit, citation} ] }`.
`query` matches id/name/keywords (substring, case-insensitive).

### `score_inputs(id)`
Returns the full input contract: `{ id, name, version, unit, citation, inputs: [InputSpec] }`.

### `compute_score(id, inputs)`
The core tool. `inputs` is a field→value map per `score_inputs`. Returns a `ScoreResult`
(`isError:false`) or a `CalcError` (`isError:true`). Demonstrated failure modes: success,
`MissingRequiredInput`, `UnitRequired`, `UnknownUnit`, `OutOfRange`.

### `convert_units(analyte, value, from, to)`
Analyte-aware conversion. Returns
`{ analyte, input:{value,unit}, output:{value,unit}, factor, basis }`. `basis` states the
physical basis (e.g. molar mass). Temperature is handled affinely.

### `solve_for(id, target, solve, fixed, bounds?)`
Monotone bisection over one numeric **quantity** input (`solve.field`) holding `fixed`
constant, finding the input value that yields `target`. Returns
`{ field, threshold:{value,unit}, target, method, iterations }`. `bounds` overrides the
default `[lo, hi]` search bracket (in the field's canonical unit). Errors with `NotBracketed`
if the target is not spanned on the bracket.

### `score_series(id, series)`
`series` is `[ { t?, inputs } ]`. Returns per-point `{ t, value, delta, trend, interpretation }`
(or a per-point `error`) plus `overall: { first, last, net_delta, trend }`. `trend` is one of
`rising` / `falling` / `stable`.

### `suggest_scores(context)`
Ranks candidate scores for a `context` (`{ domain?, question?, available_inputs? }`).
Returns `{ candidates: [ {id, name, domain, why, needed_inputs, match_score} ], note }`.
**Does not compute.**

### `extract_inputs(text)`
Deterministic scanner that turns free text / lab-dump text into candidate unit-typed inputs.
Returns `{ inputs, needs_unit, unrecognized_units, ambiguous, provenance, note, disclaimer }`.
`inputs` holds only ready-to-compute values; a value whose unit was not stated goes to
`needs_unit` (with a suggested unit to confirm), an unrecognized unit to `unrecognized_units`.
**Never fabricates a unit. Does not compute.** A parenthetical unit before the value
(`Sodium (mmol/L): 130`) is read; a trailing numeric reference range (`130 mmol/L (135-145)`)
is ignored.

### `prepare_score(id, text?, inputs?)`
Assembles inputs from `text` (via `extract_inputs`) and/or an explicit `inputs` object (explicit
overrides extracted), then reports readiness against the score's contract. Returns
`{ id, name, version, ready, inputs, satisfied, missing_required, off_contract_extracted, next,
extraction?, disclaimer }`. **Does not compute** — the `compute_score` call (and any unit
confirmation) stays with the caller.

## Score registry

25 scores across 11 domains:

| id | name | version | domain | unit |
|----|------|---------|--------|------|
| meld-na | MELD-Na | OPTN-2016 | hepatology | points |
| meld-3 | MELD 3.0 | OPTN-2023 | hepatology | points |
| ckd-epi-2021 | eGFR (CKD-EPI creatinine, 2021 race-free) | CKD-EPI-2021 | renal | mL/min/1.73m^2 |
| cockcroft-gault | Cockcroft-Gault creatinine clearance | Cockcroft-Gault-1976 | renal | mL/min |
| schwartz-egfr | Pediatric eGFR (bedside Schwartz, 2009) | Schwartz-2009-bedside | renal | mL/min/1.73m^2 |
| cha2ds2-vasc | CHA2DS2-VASc | Lip-2010 | cardiology | points |
| has-bled | HAS-BLED | Pisters-2010 | cardiology | points |
| curb-65 | CURB-65 | Lim-2003 | pulmonary | points |
| crb-65 | CRB-65 | Lim-2003 | pulmonary | points |
| wells-pe | Wells Criteria (PE) | Wells-2000 | pulmonary | points |
| perc | PERC Rule | Kline-2004 | pulmonary | positive criteria |
| westley-croup | Westley Croup Score | Westley-1978 | pulmonary | points |
| news2 | NEWS2 | RCP-2017 | acute | points |
| mews | MEWS | Subbe-2001 | acute | points |
| qsofa | qSOFA | Sepsis-3-2016 | icu | points |
| sirs | SIRS criteria | ACCP-SCCM-1992 | icu | criteria met |
| sofa | SOFA | 1996 | icu | points |
| gcs | Glasgow Coma Scale | 1974 | neuro | points |
| child-pugh | Child-Pugh | 1973 | hepatology | points |
| fib-4 | FIB-4 index | Sterling-2006 | hepatology | index |
| glasgow-blatchford | Glasgow-Blatchford Bleeding Score | Blatchford-2000 | gastroenterology | points |
| padua-vte | Padua Prediction Score (VTE) | Barbar-2010 | hematology | points |
| apgar | APGAR Score | Apgar-1953 | neonatology | points |
| pews | Pediatric Early Warning Score (Brighton) | Brighton-2005 | pediatrics | points |
| apls-weight | Pediatric weight estimate (APLS, age-based) | APLS-2011 | pediatrics | kg |

Call `score_inputs(<id>)` for each score's exact field list, units, and clamps.
