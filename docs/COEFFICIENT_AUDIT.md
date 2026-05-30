# Coefficient audit (v0.1.0)

**Date:** 2026-05-30
**Scope:** every formula coefficient, threshold/bin boundary, clamp/floor/cap, and order of
operations in all 14 scores, verified against **primary sources** and cross-checked against
MDCalc and other authoritative calculators.
**Method:** each score's Rust implementation was read line-by-line and compared
coefficient-by-coefficient to the cited primary source. Sources were fetched and read
directly (papers, OPTN policy documents, the RCP NEWS2 chart, the NKF eGFR calculator), not
taken from search-result summaries. Each documented worked example was independently
recomputed.
**Result:** **14 / 14 CLEAN. No discrepancies. No fixes required.**

> This audit verifies that the implemented numbers match the cited formulas. It is a
> correctness check of the math, not clinical validation or regulatory clearance. Caliper
> remains "calculation only — not medical advice; not a medical device."

## Summary

| id | version | verdict | worked example (recomputed) |
|----|---------|---------|------------------------------|
| meld-na | OPTN-2016 | CLEAN | cr 1.9, bili 4.0, INR 1.5, Na 130 → **26** |
| meld-3 | OPTN-2023 / Kim 2021 | CLEAN | F, cr 1.9, bili 4.0, INR 1.5, Na 130, alb 3.0 → **28** |
| ckd-epi-2021 | CKD-EPI-2021 | CLEAN | F, Scr 0.9, age 50 → **77.88** mL/min/1.73m² |
| cockcroft-gault | 1976 | CLEAN | M, age 70, 70 kg, Scr 1.0 → **68.06** mL/min |
| cha2ds2-vasc | Lip-2010 | CLEAN | F, age 75, HTN, DM → **5** |
| has-bled | Pisters-2010 | CLEAN | HTN, stroke, labile INR, age 70, drugs → **5** |
| curb-65 | Lim-2003 | CLEAN | confusion, urea 8, RR 32, SBP 85, age 70 → **5** |
| wells-pe | Wells-2000 | CLEAN | DVT signs, PE likely, HR 110 → **7.5** |
| news2 | RCP-2017 | CLEAN | RR 22, SpO₂ 93 (Sc.1), O₂, SBP 105, HR 110, alert, 38.5 °C → **9** |
| qsofa | Sepsis-3-2016 | CLEAN | RR 24, altered mentation, SBP 95 → **3** |
| sofa | 1996 | CLEAN | P/F 125 + support, plt 80, bili 3.0, MAP 65, GCS 13, cr 2.5 → **11** |
| gcs | 1974 | CLEAN | E3 V4 M5 → **12** (moderate) |
| child-pugh | 1973 | CLEAN | bili 2.5, alb 3.0, INR 1.8, mild ascites, no enceph → **9** (Class B) |
| fib-4 | Sterling-2006 | CLEAN | age 55, AST 60, ALT 40, plt 120 → **4.35** |

## Key boundary findings (the error-prone bits, all verified correct)

- **MELD-Na** — base coefficients 0.957 / 0.378 / 1.120 / 0.643 (×10) confirmed; cr, bili, INR
  each floored to 1.0; cr capped 4.0; dialysis → cr 4.0 (applied before floor/cap, no-op
  ordering correct); Na clamped [125, 137]; sodium correction applies only when MELD(i) **> 11**
  (strict), form `MELD(i) + 1.32·(137−Na) − 0.033·MELD(i)·(137−Na)`; MELD(i) rounded before the
  Na term, final rounded and bounded [6, 40].
- **MELD 3.0** — all 8 coefficients (4.56, 0.82, −0.24, 9.09, 11.14, 1.85, −1.83, +6) and the
  1.33 female addend confirmed; both interaction terms correctly negative; cr cap is **3.0**
  (not 4.0), albumin clamped **[1.5, 3.5]**, dialysis → cr 3.0.
- **CKD-EPI 2021** — leading 142, κ 0.7/0.9, α −0.241/−0.302, max-term exponent −1.200 (both
  sexes), age base 0.9938, female ×1.012 — all match Inker NEJM 2021 Table 2.
- **CHA₂DS₂-VASc** — age ≥75 = 2, age 65–74 = 1 (mutually exclusive, ≥75 checked first); only
  stroke and age ≥75 are 2-point items.
- **HAS-BLED** — age threshold is strictly **> 65** (not ≥); renal and liver are separate
  1-point items.
- **CURB-65** — urea strictly **> 7 mmol/L**; BP point combines SBP **< 90** OR DBP **≤ 60**
  (asymmetric operators, both correct); age **≥ 65**.
- **Wells PE** — point weights 3/3/1.5/1.5/1.5/1/1; HR strictly **> 100**; three-tier
  low `<2` / moderate `[2, 6]` / high `>6`; two-tier `≤4` unlikely / `>4` likely.
- **NEWS2** — full bin table confirmed including the error-prone **SpO₂ Scale 2** air-vs-oxygen
  logic (≥93 on air → 0; 93–94 on O₂ → 1; 95–96 on O₂ → 2; ≥97 on O₂ → 3); risk bands
  low 0–4 / single-parameter-3 / medium 5–6 / high ≥7, with the supplemental-O₂ +2 correctly
  excluded from the "any parameter = 3" check.
- **SOFA** — all six components' bins confirmed; respiration points 3–4 correctly require
  respiratory support (capped at 2 otherwise, with a transparency rule recorded); renal takes
  the worse of the creatinine and urine-output criteria.
- **Child-Pugh** — every boundary value (bilirubin 2.0/3.0, albumin 2.8/3.5, INR 1.7/2.3) lands
  in the middle band per the standard tables; classes A 5–6 / B 7–9 / C 10–15.

## Primary sources

- **MELD-Na:** OPTN/UNOS policy effective 2016-01-11; Kim WR et al., *N Engl J Med* 2008;359:1018-1026.
- **MELD 3.0:** Kim WR et al., *Gastroenterology* 2021;161(6):1887-1895; OPTN MELD 3.0 implementation (2023).
- **CKD-EPI 2021:** Inker LA et al., *N Engl J Med* 2021;385(19):1737-1749 (Table 2); National Kidney Foundation eGFR calculator.
- **Cockcroft-Gault:** Cockcroft DW, Gault MH, *Nephron* 1976;16(1):31-41.
- **CHA₂DS₂-VASc:** Lip GYH et al., *Chest* 2010;137(2):263-272 (PMID 19762550).
- **HAS-BLED:** Pisters R et al., *Chest* 2010;138(5):1093-1100 (PMID 20299623).
- **CURB-65:** Lim WS et al., *Thorax* 2003;58(5):377-382 (PMC1746657).
- **Wells PE:** Wells PS et al., *Thromb Haemost* 2000;83(3):416-420.
- **NEWS2:** Royal College of Physicians, *National Early Warning Score (NEWS) 2*, London 2017.
- **qSOFA:** Seymour CW et al., *JAMA* 2016;315(8):762-774.
- **SOFA:** Vincent JL et al., *Intensive Care Med* 1996;22(7):707-710.
- **GCS:** Teasdale G, Jennett B, *Lancet* 1974;2(7872):81-84.
- **Child-Pugh:** Pugh RNH et al., *Br J Surg* 1973;60(8):646-649 (PMID 4541913).
- **FIB-4:** Sterling RK et al., *Hepatology* 2006;43(6):1317-1325 (PMID 16729309).

## Notes for future maintainers

- **Unit conversion is part of the trust boundary.** Score point bins for analytes
  (bilirubin, creatinine, etc.) are hard-coded in canonical units (mg/dL); correctness of a
  non-canonical input depends on `units.rs` converting first. This path is covered by the
  `units.rs` unit tests (e.g. `creatinine_umol_to_mgdl`, `bilirubin_umol_to_mgdl`).
- **Version pinning is deliberate.** Scores implement the *cited* version, not later guideline
  refinements (e.g. CHA₂DS₂-VASc uses the original 2010 female = +1 rule, not ESC-2020 sex as a
  modifier; FIB-4 uses Sterling's original 1.45/3.25 cutoffs, not the NAFLD age-adjusted
  scheme). A future version bump should update the `version` string and citation together.
