# FS-Rules 2027 v1.0: what changes quiz answers

Source: [FS_Rules_2027_v1.0.pdf](https://www.formulastudent.de/fileadmin/user_upload/all/2027/rules/FS_Rules_2027_v1.0.pdf)
(released 2026-09-29), compared with FS-Rules 2026 v1.1. Most of the changelog is rewording; these are the
changes that move a number.

| Rule | 2026 | 2027 | In fsq |
|---|---|---|---|
| S 2.4.6 BPP non-finalists | 70 · P/Pmax | **65** · P/Pmax | `static_nonfinalist` |
| D 9.4.1 Efficiency | EF above EFmax not stated | EF clamped to EFmax (score 0) | `efficiency_score` |
| A 1.2.8 | — | every discipline and overall bounded 0 ≤ P ≤ Pmax | `event_score` |
| IN 4.1.1 insulation test | 250 V (≤250 V) / 500 V | 250 / 500 (≤500 V) / **1000 V (>500 V)** | `ts_rules` |
| EV 5.1.3-4, 5.3.4 | 120 V / 6 MJ / 12 kg per *segment* | same limits per *section* (mechanical sub-division); *segment* is now electrical | `segment_max_cells` |
| T 8.2 aero box | front devices < 500 mm / < 250 mm ahead of front axle | ahead of front tyre leading edge < **350 mm**; middle zone below tyre tops; 700-1100 mm zone; rear overhang ≤ 250 mm behind rear tyres | lookup |

Unchanged and worth knowing (same as 2026, but different from the old keys):
- Dynamic scoring (D 9.1.1 squared formula, Table 9 Tmax/Pmin), DV/DC scoring, penalties (D 10.1.7), post-inspection (IN 12.1.4).
- Points table: skidpad/accel 50, DV skidpad/accel 75, autocross 100, endurance 250, efficiency 75, trackdrive 200.
- PCB TS-LV spacing is clearance/creepage (e.g. 300-600 V: 3 mm clearance, 20 mm creepage, 4 mm coated).
  The old surface/air/coated table (237 V → 9.5 mm) is `rules=legacy` only.
- TSAL red light: min(60 V, half the **nominal** TS voltage). Old keys used max voltage.
- Cell energy = max cell voltage × nominal capacity (EV 5.1.2).
- Weight change > ±5 kg: 20 points per started kg (IN 12.1.7).
