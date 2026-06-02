# Coefficient audit

This document records the primary-source verification of every score's coefficients,
thresholds/bin boundaries, clamps/floors/caps, and order of operations.

> This audit verifies that the implemented numbers match the cited formulas. It is a
> correctness check of the math, not clinical validation or regulatory clearance. Caliper
> remains "calculation only — not medical advice; not a medical device."

Reading the **verification** column: "live" means the authoritative table was fetched and read
directly during the audit; "reproductions + consistency" means a live fetch of the original
journal table was not achievable, so the values were checked against multiple authoritative
reproductions (MDCalc etc.) and the score's own internal structure, with any conflicting
variant noted. Where a score has published variants, Caliper implements the **cited version**
and says so.

---

## v0.1.0 audit (2026-05-30) — the original 14 scores

**Scope:** every coefficient, threshold, clamp, and order of operations.
**Method:** each implementation read line-by-line and compared to the cited primary source;
sources fetched and read directly (papers, OPTN policy, the RCP NEWS2 chart, the NKF eGFR
calculator); every documented worked example independently recomputed.
**Result: 14 / 14 CLEAN. No discrepancies.**

| id | version | verification | worked example (recomputed) |
|----|---------|--------------|------------------------------|
| meld-na | OPTN-2016 | live | cr 1.9, bili 4.0, INR 1.5, Na 130 → **26** |
| meld-3 | OPTN-2023 / Kim 2021 | live | F, cr 1.9, bili 4.0, INR 1.5, Na 130, alb 3.0 → **28** |
| ckd-epi-2021 | CKD-EPI-2021 | live (NEJM Table 2) | F, Scr 0.9, age 50 → **77.88** mL/min/1.73m² |
| cockcroft-gault | 1976 | live | M, age 70, 70 kg, Scr 1.0 → **68.06** mL/min |
| cha2ds2-vasc | Lip-2010 | live | F, age 75, HTN, DM → **5** |
| has-bled | Pisters-2010 | live | HTN, stroke, labile INR, age 70, drugs → **5** |
| curb-65 | Lim-2003 | live | confusion, urea 8, RR 32, SBP 85, age 70 → **5** |
| wells-pe | Wells-2000 | live | DVT signs, PE likely, HR 110 → **7.5** |
| news2 | RCP-2017 | live (RCP chart) | RR 22, SpO₂ 93 (Sc.1), O₂, SBP 105, HR 110, alert, 38.5 °C → **9** |
| qsofa | Sepsis-3-2016 | live | RR 24, altered mentation, SBP 95 → **3** |
| sofa | 1996 | live | P/F 125 + support, plt 80, bili 3.0, MAP 65, GCS 13, cr 2.5 → **11** |
| gcs | 1974 | live | E3 V4 M5 → **12** (moderate) |
| child-pugh | 1973 | live | bili 2.5, alb 3.0, INR 1.8, mild ascites, no enceph → **9** (Class B) |
| fib-4 | Sterling-2006 | live | age 55, AST 60, ALT 40, plt 120 → **4.35** |

### Key boundary findings (the error-prone bits, all verified correct)

- **MELD-Na** — base coefficients 0.957 / 0.378 / 1.120 / 0.643 (×10); cr, bili, INR floored to
  1.0; cr capped 4.0; dialysis → cr 4.0 (before floor/cap); Na clamped [125, 137]; sodium
  correction only when MELD(i) **> 11** (strict); MELD(i) rounded before the Na term, final
  rounded and bounded [6, 40].
- **MELD 3.0** — all 8 coefficients (4.56, 0.82, −0.24, 9.09, 11.14, 1.85, −1.83, +6) and the
  1.33 female addend; both interaction terms negative; cr cap **3.0**, albumin clamp **[1.5,3.5]**.
- **CKD-EPI 2021** — 142, κ 0.7/0.9, α −0.241/−0.302, max-term exponent −1.200, age base 0.9938,
  female ×1.012 — all match Inker NEJM 2021 Table 2.
- **CHA₂DS₂-VASc** — age ≥75 = 2, age 65–74 = 1 (≥75 checked first); only stroke and age ≥75 are
  2-point items. **HAS-BLED** — age strictly **> 65**; renal/liver separate items.
- **CURB-65** — urea strictly **> 7 mmol/L**; BP = SBP **< 90** OR DBP **≤ 60** (asymmetric); age ≥ 65.
- **NEWS2** — full table incl. the error-prone **SpO₂ Scale 2** air-vs-oxygen logic; risk bands
  with the supplemental-O₂ +2 correctly excluded from the "any parameter = 3" check.
- **SOFA** — respiration 3–4 require respiratory support (capped at 2 otherwise, recorded);
  renal takes the worse of creatinine and urine-output.
- **Child-Pugh** — boundary values (bilirubin 2.0/3.0, albumin 2.8/3.5, INR 1.7/2.3) land in the
  middle band; classes A 5–6 / B 7–9 / C 10–15.

---

## v0.2.0 audit (2026-05-31) — the 6 new scores

**Scope:** every threshold/bin and point weight in the new scores; the worked example per
score recomputed.
**Method:** implementations read line-by-line; **Glasgow-Blatchford and Padua verified live**
against authoritative tables; CRB-65, PERC, SIRS verified against primary definitions + MDCalc;
MEWS verified against the original Subbe grid with the caveat below. Independent audit agents
fetched sources directly where possible.
**Result: 6 / 6 CLEAN. No code fixes required.**

| id | version | verification | worked example (recomputed) |
|----|---------|--------------|------------------------------|
| crb-65 | Lim-2003 | reproductions + consistency | confusion, RR 32, SBP 85, age 70 → **4** |
| perc | Kline-2004 | reproductions + consistency | age 40, HR 80, SpO₂ 98, no history → **0** (PERC-negative) |
| mews | Subbe-2001 | reproductions + consistency (see note) | SBP 85, HR 115, RR 22, 38.6 °C, voice → **8** |
| sirs | ACCP-SCCM-1992 | reproductions + consistency | T 38.5, HR 110, RR 24, WBC 14 → **4/4** |
| glasgow-blatchford | Blatchford-2000 | **live** (full table) | M, urea 8.0, Hgb 11.5, SBP 105, pulse 110, melena → **9** |
| padua-vte | Barbar-2010 | **live** | active cancer, reduced mobility, age 75 → **7** (high) |

### Key findings

- **Glasgow-Blatchford (live).** Every band confirmed: urea (mmol/L) <6.5→0 / 6.5–7.9→2 /
  8.0–9.9→3 / 10.0–24.9→4 / ≥25→6; Hgb men ≥13→0 / 12–12.9→1 / 10–11.9→3 / <10→6; women ≥12→0 /
  10–11.9→1 / <10→6 (no 3-point band for women — correct); SBP ≥110→0 / 100–109→1 / 90–99→2 /
  <90→3; pulse ≥100→1, melena→1, syncope→2, hepatic→2, cardiac→2; max **23**. Boundary
  inclusivity (urea 8.0→3; Hgb 12.0 men→1 vs women→0; SBP 100→1, 110→0) all correct.
- **Padua VTE (live).** Weights {cancer 3, prior VTE 3, reduced mobility 3, thrombophilia 3,
  recent trauma/surgery 2, age ≥70 1, heart/resp failure 1, MI/stroke 1, infection/rheum 1,
  obesity 1, hormonal 1}; high risk ≥4; max 20. Confirmed.
- **SIRS.** temp >38 or <36; HR >90; RR >20 or PaCO₂ <32; WBC >12 or <4 or >10% bands; ≥2 of 4.
  Strict-inequality operators confirmed.
- **PERC.** Eight criteria, positive when age ≥50 / HR ≥100 / SpO₂ <95 / any history item;
  rule-out (PERC-negative) only at count 0. Confirmed; the tool reports the positive count and
  does not assess pretest probability.
- **CRB-65.** Urea-free CURB-65 variant: confusion, RR ≥30, BP (SBP <90 or DBP ≤60), age ≥65;
  max 4. Confirmed.

### MEWS verification note (read this)

MEWS has **multiple published variants.** Caliper implements the original **Subbe et al. 2001**
(QJM 94:521-526), as the `Subbe-2001` version string promises. The QJM table itself is
paywalled and was not byte-fetched this session. One online calculator (MDApp) shows a
**variant** with temperature `<35 → 3` and AVPU `Unresponsive → 2`. Caliper does **not** follow
that variant. The implemented grid — temperature `<35 → 2` and `≥38.5 → 2` (symmetric; no
temperature cell scores 3) and AVPU `Alert 0 / Voice 1 / Pain 2 / Unresponsive 3` — is the
canonical 3-2-1-0-1-2-3 Subbe grid and the dominant academic reproduction; the audit confirmed
this against multiple secondary sources and the grid's internal structure (the MDApp variant
breaks both the symmetric temperature column and the monotone AVPU ladder). For a
byte-level match to a specific institutional MEWS chart, confirm its temperature and AVPU rows
first. The worked example (=8) is the same under either variant.

A minor transcription point at **HR = 40**: the original is usually written "<40 → 2" / "41–50 →
1", leaving 40 in a gap; Caliper scores `≤40 → 2` (consistent with MDCalc). Affects only an
input of exactly 40 bpm.

---

## v0.3.0 audit (2026-06-01) — the 5 pediatric / weight-band scores

**Scope:** every point weight / band boundary and the formula constants in the new scores; the
worked example per score recomputed.
**Method:** implementations read line-by-line and compared to the cited primary definitions and
authoritative reproductions (MDCalc etc.); the bedside Schwartz constant and the APLS bands
checked against their published forms.
**Result: 5 / 5 CLEAN. No code fixes required.**

| id | version | verification | worked example (recomputed) |
|----|---------|--------------|------------------------------|
| schwartz-egfr | Schwartz-2009-bedside | reproductions + consistency | height 120 cm, Scr 0.5 mg/dL → **99.12** mL/min/1.73m² |
| apgar | Apgar-1953 | reproductions + consistency | A2 P2 G1 A2 R2 → **9** |
| westley-croup | Westley-1978 | reproductions + consistency | max: consc 5 + cyanosis 5 + stridor 2 + air entry 2 + retractions 3 → **17** |
| pews | Brighton-2005 | reproductions + consistency (see note) | behaviour 2 + CV 1 + resp 2 + modifier 2 → **7** |
| apls-weight | APLS-2011 | reproductions + consistency | age 4 y → (2 × 4) + 8 = **16** kg |

### Key findings

- **Bedside Schwartz (2009).** Constant **0.413**, eGFR = 0.413 × height(cm) / Scr(mg/dL),
  expressed in mL/min/1.73m². The 0.413 coefficient assumes an IDMS-traceable (enzymatic)
  creatinine; documented in the input notes. Height is unit-converted (cm canonical).
- **APGAR.** Five signs (appearance, pulse, grimace, activity, respiration), each 0/1/2; total
  0–10; bands 7–10 / 4–6 / 0–3. Caliper takes the per-sign points directly (the assessor maps
  the clinical sign to 0/1/2 per Apgar's original definition, restated per field).
- **Westley croup.** Unevenly weighted levels confirmed: consciousness {normal 0, disoriented
  5}; cyanosis {none 0, with agitation 4, at rest 5}; stridor {0,1,2}; air entry {0,1,2};
  retractions {0,1,2,3}; max **17**. Bands ≤2 / 3–7 / 8–11 / ≥12.
- **APLS weight.** Three bands: <1 y → (0.5 × months) + 4; 1–5 y → (2 × age) + 8; 6–12 y →
  (3 × age) + 7. Validity capped at 0–12 years (`OutOfRange` beyond). Continuous at the 1-year
  boundary (both give 10 kg); the 6-year step is inherent to the APLS formula. This is an
  *estimate of an unmeasured weight*, returned descriptively — never a dosing directive.

### PEWS variant note (read this)

PEWS has **many institutional variants** — different parameter grids and, critically,
**age-banded vital-sign thresholds** ("heart rate >X above normal for age"). Caliper implements
the original **Brighton PEWS (Monaghan 2005)** three-component structure (behaviour /
cardiovascular / respiratory, each 0–3) plus the **+2 modifier** (quarter-hourly nebulisers or
persistent post-operative vomiting), max **11**, as the `Brighton-2005` version string
promises. It takes the **assessed component level** directly rather than re-deriving levels from
raw age-banded vitals — that derivation is exactly the part that varies between sites, and
auto-grading it would silently pin one local chart. Escalation thresholds also vary by site, so
the interpretation band is descriptive only. For a byte-level match to a specific
institutional PEWS chart, confirm its component grid first.

---

## Primary sources

v0.1.0: OPTN/UNOS 2016 sodium policy & Kim NEJM 2008 (MELD-Na); Kim *Gastroenterology* 2021 &
OPTN 2023 (MELD 3.0); Inker *NEJM* 2021 (CKD-EPI 2021); Cockcroft & Gault *Nephron* 1976; Lip
*Chest* 2010 (CHA₂DS₂-VASc); Pisters *Chest* 2010 (HAS-BLED); Lim *Thorax* 2003 (CURB-65); Wells
*Thromb Haemost* 2000; RCP *NEWS2* 2017; Seymour *JAMA* 2016 (qSOFA); Vincent *Intensive Care
Med* 1996 (SOFA); Teasdale & Jennett *Lancet* 1974 (GCS); Pugh *Br J Surg* 1973 (Child-Pugh);
Sterling *Hepatology* 2006 (FIB-4).

v0.2.0: Lim *Thorax* 2003 (CRB-65); Kline *J Thromb Haemost* 2004 (PERC); Subbe *QJM* 2001
(MEWS); Bone *Chest* 1992 ACCP/SCCM (SIRS); Blatchford *Lancet* 2000 (Glasgow-Blatchford);
Barbar *J Thromb Haemost* 2010 (Padua).

v0.3.0: Schwartz *J Am Soc Nephrol* 2009 (bedside pediatric eGFR); Apgar *Curr Res Anesth Analg*
1953 (APGAR); Westley *Am J Dis Child* 1978 (croup score); Monaghan *Paediatr Nurs* 2005
(Brighton PEWS); *Advanced Paediatric Life Support* 5th ed. 2011 & Luscombe & Owens *Arch Dis
Child* 2007 (age-based weight estimation).

## Notes for future maintainers

- **Unit conversion is part of the trust boundary.** Analyte point bins (bilirubin, creatinine,
  urea, hemoglobin, WBC, …) are hard-coded in canonical units; correctness of a non-canonical
  input depends on `units.rs` converting first. Covered by the `units.rs` unit tests.
- **Version pinning is deliberate.** Scores implement the *cited* version, not later
  refinements (CHA₂DS₂-VASc uses original 2010 female = +1; FIB-4 uses Sterling 1.45/3.25; MEWS
  follows original Subbe 2001). A version bump updates the `version` string and citation together.
- **When live primary-source fetch is unavailable, don't claim a live byte-check.** Record the
  verification method honestly (per the per-score column) and flag any unresolved variant.
