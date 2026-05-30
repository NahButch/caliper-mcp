# Examples

Each file is a real JSON-RPC request paired with the response Caliper produced over stdio.

| file | demonstrates |
|------|--------------|
| [`compute_score.meld-na.json`](compute_score.meld-na.json) | a successful `compute_score`: MELD-Na with the sodium-correction rule recorded in `applied_rules` |
| [`convert_units.creatinine.json`](convert_units.creatinine.json) | analyte-aware unit conversion (150 µmol/L creatinine → mg/dL) with the molar-mass basis |
| [`compute_score.error-missing-input.json`](compute_score.error-missing-input.json) | the no-silent-defaults invariant: a missing input returns a typed `MissingRequiredInput` |

To reproduce, pipe an `initialize` call plus the `request` objects (one JSON object per line)
into the server:

```sh
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{}}}' \
  '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"convert_units","arguments":{"analyte":"creatinine","value":150,"from":"umol/L","to":"mg/dL"}}}' \
  | ./target/release/caliper-mcp
```
