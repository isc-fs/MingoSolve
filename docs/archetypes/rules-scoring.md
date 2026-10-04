# calc_a.md: solver archetypes

Source: all 101 questions in `calc_a.md`. Rulebook used: FS-Rules 2026 v1.1 (`rules.txt` extract). Most of the quiz keys come from older rule years (FS Rules 2017–2025, FSA/FSEast/FSS handbooks), so each archetype gives the **legacy** form (the one that reproduces the keys) and the **2026** form. Unless marked otherwise, "✓" means I recomputed the value in Python and it matches the key.

---

## 1. `dynamic_event_score` (manual-mode skidpad/accel/autocross/endurance). Domain: rules/scoring. Frequency: 12

**Legacy (reproduces every key that can be checked):**
| Event | Score | Tmax | Pmax |
|---|---|---|---|
| Skidpad | `71.5·((Tmax/Tteam)²−1)/0.5625 + 3.5` | 1.25·Tmin | 75 |
| Acceleration | `71.5·(Tmax/Tteam−1)/0.5 + 3.5` | 1.5·Tmin | 75 |
| Autocross | `95.5·(Tmax/Tteam−1)/0.25 + 4.5` | 1.25·Tmin | 100 |
| Endurance | `300·(Tmax/Tteam−1)/0.333 + 25` | **1.333**·Tmin (literal, not 4/3) | 325 |

- Tteam is the team's best corrected time (raw + penalties), capped at Tmax, so a finisher always gets at least the floor (3.5 / 3.5 / 4.5 / 25).
- Tmin is the fastest *corrected* time of all teams, so it includes the fastest team's own penalties.
- Some later legacy years write the constants as `0.95·Pmax … + 0.05·Pmax` (71.25 / 3.75). That changes the 2nd decimal: Q378 would give 41.24 instead of 41.12. **The keys use 71.5 / 3.5.**

**2026 (D 9.1.1, unified):** `SCORE = (Pmax−Pmin)·((Tmax−Tteam)/(Tmax−Tmin))² + Pmin`. Table 11 gives:
| Event | Tmax | Pmin | Pmax |
|---|---|---|---|
| Skidpad | 1.35·Tmin | 0.05·Pmax | 50 |
| Acceleration | 1.7·Tmin | 0.05·Pmax | 50 |
| Autocross | 1.4·Tmin | 0.1·Pmax | 100 |
| Endurance | 1.5·Tmin | 0.1·Pmax | 250 |
| [DC] Skidpad | 1.7·Tmin | 0.05·Pmax | |
| [DC] Acceleration | 2.25·Tmin | 0.05·Pmax | |

Pmax comes from Table 3.

**Penalties, D 10.1.7** (the same values in the legacy keys):
| | Acc | Skid | AutoX | Endu | TD |
|---|---|---|---|---|---|
| DOO | 2 s | 0.2 s | 2 s | 2 s | 2 s |
| OC | DQ | DQ | 10 s | 10 s | 10 s |
| USS | DQ | DQ | DQ | n/a | −50 pts |

Other time penalties:
- Flag disobeyed: +60 s (D 10.1.8).
- Running out of order in endurance: +120 s (D 10.2.1).
- Post-inspection (IN 12.1.4), Group A (no advantage) / Group B (advantage):
  | | Acc | Skid | AutoX | Endu | TD |
  |---|---|---|---|---|---|
  | Group A | 0.3 s | 0.2 s | 2 s | 30 s | 30 s |
  | Group B | 1 s | 0.6 s | 5 s | 120 s | 120 s |

**Variables:** Tteam, Tmin, Tmax in s; n_DOO, n_OC as counts; P in points.

**Inverse mode** (find the number of cones from a score): solve the score equation for Tteam, then `n = round((Tteam − Traw)/t_DOO)`.

**Questions:**
- Computable: Q4, Q22, Q171, Q378, Q387 (duplicate of Q378, key empty → 41.12), Q564, Q656, Q705.
- Need external data: Q322 (FSEast 2016 Tmin, ≈3.664 s implied), Q647 (run table missing), Q979 (team table missing; uses IN 12.1.4 Group A/B).
- Partly uses it: Q624.

**Regression tests:**
- Q378: skid, Traw 5.2, 2 DOO → Tteam 5.6; Tmin 5.1 → **41.12** ✓
- Q656: skid, Tteam 4.926, Tmin 4.756 → **61.5** ✓ (61.54)
- Q564: autoX, 68.2 + 3·2 = 74.2; Tmin 62.5 → **24.7** ✓
- Q4 (inverse): autoX, raw 86.3, score 73.1, Tmin 87.1 → Tteam 92.30 → **3 cones** ✓
- Q705 (inverse): raw 91.2, score 77.52, Tmin 92.6 → Tteam 97.18 → **3 cones** ✓
- Q22: skid 6.25 vs 5.0 → Tteam = Tmax → **3.5** ✓
- Q171: endu, Tmin = 22.25·60 + 5·2 = 1345 s; Tteam = 27.5·60 + 6·2 + 10 = 1672 s → time part **65.13** ✓

**Gotchas:**
- The Q171 key **omits the +25 finish points**. The full legacy score would be 90.13. Expose a flag for it.
- Use the literal 1.333 and 0.333; using 4/3 gives 65.38 in Q171.
- The question data uses decimal commas ("27,50 min" means 27.50 min, not 27:50).
- Tmin is a corrected time.
- Cap Tteam at Tmax before computing, otherwise the time term goes negative.

---

## 2. `skidpad_geometry` (lengths, speeds and cone counts). Domain: rules/track + physics. Frequency: 8

**Geometry (D 4.1.2):**
- Inner diameter 15.25 m, outer 21.25 m, centre spacing 18.25 m.
- The driving line is the centreline: diameter `Dc = (15.25+21.25)/2 = 18.25 m`, `r = 9.125 m`, `lap = π·Dc = 57.334 m`.
- A run is 4 laps. Only 2 are timed (one right, one left), and the official time is the average of the two timed laps (D 4.4.1 in legacy numbering).

**Formulas:**
- Average speed: `v = π·Dc / Tavg`
- Distance over the timed laps: `2·π·Dc = 114.67 m`
- Grip-limited speed: `v = √(μ·g·r)`

**Cone counts:**
- 2026 (D 4.1.3): 17 inside each inner circle + 13 outside each outer circle → 2·(17+13) = **60**.
- Legacy: 16 + 13 → 2·(16+13) = **58**.
- Q50 key "29" = 16 + 13 (the legacy count for one inner + one outer circle); treat it as a lookup.

**Questions:** Q99, Q193, Q447, Q517, Q543, Q134, Q893, Q50.

**Regression tests:**
- Q543: (4.901 + 5.031)/2 = 4.966 s → 57.334/4.966 = **11.55 m/s** ✓
- Q517: μ = 0.87, g = 9.81, r = 9.125 → 8.825 m/s = **31.77 km/h** (key option 31.75) ✓
- Q99: **18.25 m** ✓
- Q193 (key empty): 2·π·18.25 = **114.67 m** → option "114,69"
- Q447: **0.0**. The figure-eight runs equal laps in each direction, so the left and right wheels end with the same number of turns.

**Gotchas:**
- Cone count depends on the rule year: Q134 = 58 (legacy), Q893 = 60 (2026, yellow + blue only).
- Use the centreline radius (9.125 m), not the inner radius.

---

## 3. `straight_line_const_accel`. Domain: physics/kinematics (acceleration event). Frequency: 7

**Formulas:**
- `d = v0·t + ½·a·t²`
- `v = v0 + a·t`
- `v² = v0² + 2·a·d`
- From rest over the acceleration track: `t = 2d/v_f`, with `d = 75 m` (D 5.1.1).
- Average speed with linear acceleration and then linear deceleration: `v_avg = v_f/2` over both phases.
- Braking distance: `v²/(2a)`
- Reaction distance: `v·t_r`
- Force: `F = m·a`, with `a = Δv/Δt`

**Variables:** d in m, t in s, v in m/s (convert km/h with /3.6), a in m/s², m in kg, F in N.

**Questions:**
- Computable: Q7, Q201, Q460, Q492, Q570, Q850.
- Needs external data: Q905 (FSCzech 2023/2024 results). With constant force, `t ∝ √m`, so `Δm = m_tot·(1 − (t_win/t_team)²)`, with m_tot = 315 kg.

**Regression tests:**
- Q201: v_f = 35 m/s → t = 150/35 = **4.28 s** ✓
- Q460: 22.22·1.5 + 22.22²/20 = **58 m** ✓
- Q850: Coco covers 75 − 14 = 61 m → t = √(122/6) = 4.509 s → a_Fred = 150/t² = **7.4 m/s²** ✓
- Q570: the 1.473 s is a **0–100 km/h** time, not a 75 m time. Distance = 27.78/2·1.473 + 27.78·3 + 27.78²/(2·13.3) = 20.46 + 83.33 + 29.01 = **132.8 m** ✓
- Q7: a = 15/2 = 7.5 m/s² → m = 1350/7.5 = **180 kg** ✓
- Q492: v_avg = 75/4.8 = 15.625 m/s = **2952.76 ft/min** ✓

**Gotchas:**
- Read the question for which distance the time refers to (Q570).
- Unit conversions: ft/min (×196.85 from m/s), mph.

---

## 4. `static_nonfinalist_score` (BPP and Cost when finals are held). Domain: rules/scoring. Frequency: 4 computable + 2 lookups

**BPP:**
- 2026 S 2.4.6: `BPP = 70·Pteam/Pmax_nonfinal`.
- Legacy FS Rules (≈2018–19, e.g. FSEast 2018): `BPP = (75 − n_finalists)·Pteam/Pmax_nonfinal`.
- FSA uses 70.
- Finalists get 75 to 71 points (S 2.4.7).

**Cost & Manufacturing:**
- Legacy FS 2019: `C&M = 95·Pteam/Pmax_nonfinal` (Q200 ✓).
- 2026 S 3.8.5: non-finalists are capped at 80. No explicit formula is given; the analogous one is `80·Pteam/Pmax`.

**Variables:** points.

**Questions:**
- Computable: Q131, Q169, Q200, Q214.
- Lookups: Q103, Q658.

**Regression tests:**
- Q214: 59/67·(75 − 6) = **60.8** ✓
- Q131: 55/61·70 = **63.1** ✓
- Q200: 79/91·95 = **82.5** ✓
- Q169: key **72.63** = 75·92/95 (the official solution's formula). This is **inconsistent with Q200**: the rule-consistent value would be 75/92·95 = 77.45, which is not an option. Hard-code the key.

**Gotchas:**
- The constant depends on the event and year (70 / 75−n / 95 / 80).
- Pmax is the best **non-finalist** score, never the overall best.

---

## 5. `lever_moment_balance` (2D rigid-body ΣM = 0, ΣF = 0). Domain: mechanics. Frequency: 6 (+5 truss variants, see 5b)

**Equation:** `ΣM_pivot = 0` → `F_unknown = F·l_F / l_unknown`.

**Variables:** F in N, l in m (or mm, consistently).

**Questions:**
- Computable without the figure: Q376, Q384, Q421, Q146, Q603.
- Needs the figure: Q80, Q509.

**Regression tests:**
- Q376: 500·45/35 = **643 N** ✓
- Q384 (key empty): 880·55/45 = **1076 N** (positive, same sign convention as Q376)
- Q421: 231.6·1/30 = **7.72 m** ✓
- Q146: hanger load = Σ m_i·(g + a_i,up) = 5(9.80665 − 1.6) + 4(9.80665 + 0.9) + 3·9.80665 = **113.3 N** ✓. 90 Gal = 0.9 m/s²; constant speed means a = 0. The question was voided officially.
- Q603: (1.2·2400/2)·1.35 = 1944 N → **2000 N** (the next option up)

### 5b. `truss_statics` (method of joints/sections; figure required)
- Questions: Q432 (−2.0P), Q618 (zero-force members 6, 9), Q619 (16.0 kN), Q746 (−35.4 kN), Q1000 (−17.322 N).
- Sign convention: tension positive. Q746's official solution: ΣFx gives R = 45.71 kN; ΣM_A gives R_Gy = 35.355 kN → f_HG = −35.355 kN.
- A library can only provide a generic 2D truss solver. The geometry has to be entered by hand from the figure.

---

## 6. `corrected_elapsed_time` (endurance/run time + penalties, DQ logic). Domain: rules/scoring. Frequency: 5

**Formula:**
`T_corr = T_raw(excluding the driver-change lap) + 2·n_DOO + 10·n_OC + 60·n_flag + 120·out_of_order + IN 12.1.4 post-inspection penalty`

Notes on the terms:
- A mechanical black flag **with no fault found** adds 0. That time is officials' time (D 10.2.2).
- Missing a slalom: 2026 counts one OC per occurrence, however many gates are missed (D 10.1.5).
- Autonomous mode: an unsafe stop (USS) means **DQ → 0 points**. A USS includes not entering the finish state within 30 s (D 10.1.6).

**Questions:**
- Q824, Q798: computable.
- Q911: ambiguous, see below.
- Q979: table missing.
- Also feeds Q171 and Q564.

**Regression tests:**
- Q824: 1760 + 120 (out of order) + 0 (black flag, no defect) + 120 (rear wing too high = aero, Group B) = **2000 s** ✓. Gotcha: under 2026 IN 12.1.6, losing a part on track would mean DQ for the run; the key ignores this.
- Q798: ASSI not continuous blue within 30 s → USS → DQ → **0** ✓
- Q911 (key **1860**): the only combination that reaches 1860 is 2·820 + 200 (driver change **not** subtracted) + 5·2 + 10 (1 OC for the slalom) with no blue-flag penalty. That contradicts D 7.2.5 and D 10.1.8. The rule-consistent value, 1640 + 10 + 10 + 60 = 1720, is not an option. Hard-code the key and flag it.

---

## 7. `stirling_cycle_ideal_gas`. Domain: thermodynamics. Frequency: 2

**Cycle:** 1→2 isothermal, 2→3 isochoric, 3→4 isothermal, 4→1 isochoric (V4 = V1, V3 = V2).

**Equations:**
- `p·V = const` on the isotherms
- `p/T = const` on the isochores

**Variables:** p in bar, V in m³, T in K. Convert °C to K by adding 273.15.

**Regression tests:**
- Q66: p4 = p3·V3/V4 = 0.6·3.3/0.6 = **3.3 bar** ✓
- Q155: T3 = T4 = T1·p4/p1 = 800·1.2/6.5 = **147.7 K** ✓

**Gotcha:** the cycle type is only in the figure. Assume Stirling when the question says "V1 = V4, V2 = V3".

---

## 8. `continuity_bernoulli` (incompressible). Domain: fluids. Frequency: 2 (+2 hydraulic-figure variants)

**Equations:**
- Continuity: `A1·v1 = A2·v2` → `d2 = d1·√(v1/v2)`
- Bernoulli, horizontal pipe: `p2 = p1 + ½ρ(v1² − v2²)`

**Variables:** p in Pa, ρ in kg/m³, v in m/s, A in m².

**Regression tests:**
- Q162: 27/√2 = **19.1 mm** ✓
- Q613: v2 = 1.796 m/s → p2 = 1.2e5 + 498.5·(25 − 3.225) = **1.31 bar** ✓

**Related, figure required:** Q628 (hydraulic cylinder force, F = p·A; key 314.16 = 100π) and Q1060 (flow-control valve area).

---

## 9. `endurance_energy_budget` (road-load energy balance). Domain: vehicle dynamics/energy. Frequency: 4

**Equations:**
- Aero drag: `F_d = ½ρ·c_d·A·v²`
- Normal force with downforce: `N = m·g + ½ρ·|c_L|·A·v²`
- Rolling resistance: `F_r = μ_R·N`
- Braking losses per lap: `E_brake/lap = Σ k·½m(v_i² − v_f²)`
- Energy at the wheel: `E_wheel = D·(F_d + F_r) + laps·E_brake/lap`
- Battery: `E_start = E_wheel/η/(1 − SOC_min)`
- Fuel: `V_start = E_wheel/(η·e_fuel) + V_reserve`
- Simple average force: `F = E/d`

**Variables:**
- ρ in kg/m³, A in m², v in m/s, m in kg, D in m.
- E in J (1 kWh = 3.6e6 J), e_fuel in J/l.

**Regression tests:**
- Q1006: E_wheel = 15.4986 MJ → /0.69/0.9 → **6.93 kWh** ✓
- Q1085: 15.4986e6/(0.22·34e6) + 0.2 = **2.27 l** ✓
- Q787: 1e6/1100 = **909 N** ✓ (Q765 is a duplicate with an empty key)

**Gotchas:**
- The end-of-race SOC margin is applied by **division** (/0.9), not by adding 10 %.
- The fuel reserve is **added** in litres.
- The downforce sign: c_L = −4.5 increases the normal force, so use |c_L|.

**Q624** belongs here and to archetypes 1 and 10:
- Energy cap: E = P·v⁴ ≤ 6000 Wh → v = (6000/0.0005)^¼ = 58.86 km/h → lap = 3600/v = **61.17 s** ✓.
- The key is simply the energy-limited fastest pace. The official note checks that slowing down never gains more efficiency points than it loses in endurance.

---

## 10. `efficiency_score`. Domain: rules/scoring. Frequency: 3

**2026 (D 9.4):**
- `EF = T²·E`, where T is the uncorrected driving time and E is the used energy (EV) or corrected fuel mass (CV).
- `EFFICIENCY = Pmax·((EFmax − EFteam)/(EFmax − EFmin))²`, with `EFmax = 2·EFmin` and Pmax = 75.
- Only teams that scored at least Pmin in endurance are eligible (D 7.9.2).

**Legacy:** `EF = (Tmin/Tteam)·(Emin/Eteam)` (higher is better) and `Score = 100·(EFteam − EFmin)/(EFmax − EFmin)`, with Pmax = 100.

**Questions:** Q167, Q953, Q624.

**Q167 is unresolved.** Key 96.15. With the legacy formula and T = 24.30/28.20 min, E = 5.32/6.08 kWh: EF = 0.754 → score 74.05. The inverse form `100·(EFmin/EF − 1)/(EFmin/EFmax − 1)` gives 83.48. Thirty-two variants tested (mm:ss vs decimal minutes, corrected vs uncorrected times, exponents 0.5/1/2) do not reproduce 96.15. Treat the key as unverified.

**Q953:** needs the FSG 2024 results and has no key.

---

## 11. `document_deadline_penalty`. Domain: rules/admin. Frequency: 3

**2026 rules:**
- 10 points per late submission (A 5.4.1).
- De-registration if the document is still missing 24 h after the deadline (A 5.4.2).
- A correction must be submitted within 168 h (7 days) (A 5.3.1).
- Rejoining after de-registration: +20 points (A 5.5.3).

**Questions:**
- Q892: **10** ✓
- Q452: **"1, 7"** ✓ (days)
- Q512 (legacy group-A rule, key **50**): `10·(⌊49/24⌋ + ⌊77/24⌋) = 10·(2+3) = 50`. This is the only clean fit and has low confidence; the commenced-day reading `⌈⌉` gives 70. Under 2026 rules the team would be de-registered.

---

## 12. `finance_fx` (FX, ROI, earned value). Domain: business. Frequency: 3

**Equations:**
- FX exposure: `ΔEUR = X_USD/r_now − X_USD/r_sign`
- ROI: `(return_in_USD − invest)/invest`, with `return_in_USD = return_EUR/(EUR per USD)`
- SPI: `EV/PV`, with `EV = %done·BAC` and `PV = BAC·(planned units to date)/(total units)`
- CPI: `EV/AC`

**Regression tests:**
- Q952: 22420/1.21 − 22420/1.43 = **+2851** ✓
- Q694: 250000/0.9175 = 272480 USD → **172.48 %** ✓

**Q584 (key 0.09) is suspicious.**
- Count Jan 25 – Mar 1, 2023 inclusive with weekend days weighted ×2: 46 units in total, 3 of them by Jan 27.
- SPI = 0.05/(3/46) = **0.77**. This is one of the options and is the standard SPI.
- CPI = 0.54.
- 0.09 is not reproduced. Hard-code the key, or flag it.

---

## 13. `max_points_table` (sums over Table 3). Domain: rules. Frequency: 4

**Q510:**
- Key 425 = BPP 75 + C&M 100 + Skid 75 + Acc 75 + Eff 100 (the **legacy** table).
- With the 2026 table: 75 + 100 + 50 + 50 + 75 = 350.

**Lookups in the same table:**
- Q286: FSS endurance 300.
- Q236 / Q443: 25 finish points in legacy endurance.
- Q391: Cost Understanding 35 % (legacy). 2026 gives 25/100.
- Q788: figure.

**Gotcha:** make the points table year-keyed.

---

## One-off computable questions

| Q | How to solve | Answer |
|---|---|---|
| Q202 | Δh = 1 %·300 = 3 m. Sliding gives √(2gh) = 7.746 m/s; a rolling solid sphere gives 6.55. **Key 5.897 not reproduced** (it implies v²/(gh) ≈ 1.159). | key 5.897 m/s |
| Q257, Q315 | n = 60·f/(poles/2) = 60·50/2 | 1500 rpm ✓ |
| Q1072 | V = π/4·d²·s = π/4·7²·12 | 461.81 cm³ ✓ |
| Q945 | 8·700e3/250e6 = 22.4 ms, plus RTT/2 = 30 ms | 52 ms ✓ |
| Q464 | ρ ∝ M/a³ → 2/0.5³ | 16 ✓ |
| Q175 | 42h10 gives 42.000 / 41.900 (IT10 = 100 µm for 30–50 mm), so 42.10 is outside | 42,10 ✓ |
| Q508 | Minimum head restraint 150×150×40 mm (T 5.7.2) = 900 cm³ × 0.093 g/cm³ | 84 g ✓ |
| Q562 | Chord of length L on y = x²: the gap is h² = L²/(4(1+4c²)), so area = πL²/8 | 2π = 6.28 ✓ |
| Q607 | Boyle: 165·0.8/1.01325 = 130.3 l ÷ 1 l per shift | 130 ✓ |
| Q723 | Key 26 = 2·(75/5 − 2). The fencepost reading would give 28. Hard-code. | 26 |
| Q303 | 80 kW limit with a 500 ms moving average (D 10.4.1): E = 80·3.95²/(2·3.70) = 168.68 kJ (no average: 158 kJ). **No key; neither matches the options.** | none |
| Q635 | ⌈305 km/(2·1.3 km)⌉. Needs the map distance. | 118 (122 also accepted) |
| Q1079 | Hex → 14, 19, 22, 27, 30; differences alternate +5, +3 | 35 ✓ |
| Q711 | Count the closed loops in the digits (6 = 1, 0 = 1, 8 = 2) | 4 ✓ |
| Q594 | Two interleaved series: odd numbers and powers of 2 | 32 ✓ |
| Q701 | 1+2+3 = 1·2·3 | 1-2-3 ✓ |
| Q125 | A quadratic tetrahedron has 10 nodes × 3 DOF = 30. **Key 60 contradicts this.** | key 60 |
| Q166 | R2D sound must be 80–90 dBA at 2 m (EV 4.12.2) | 82 ✓ |
| Q865 | Critical path (CPM). Activity table missing. | A-B-H-J-M-N |
| Q944 | Sum of GWP factors × activities. Factor table missing. | 34.93 |
| Q69, Q70, Q256, Q591, Q628, Q859, Q867, Q898, Q989, Q1060, Q1061 | Figure required (logic gates, thin-wall I_x, wheel DOF, perimeter, hydraulics, shaft sizing by max-shear theory, tilt-test kinematics, mass/CoG, block-diagram reduction) | keys as in the file |

## Not computable (pure lookup / trivia / puzzle)

- **Rule lookups:** Q2, Q43, Q44, Q311, Q51, Q233, Q236, Q443, Q286, Q345, Q368, Q391, Q458, Q560, Q709, Q833, Q892, Q904, Q920, Q103, Q658, Q452, Q50.
- **Trivia:** Q402, Q405, Q446, Q699, Q700, Q901, Q955, Q973, Q1055.
- **Puzzles:** Q895 (minesweeper), Q899 (chess), Q1075 (crossword).

## Keys that look wrong or inconsistent (flag them in the solver)

| Q | Problem |
|---|---|
| Q167 | 96.15 not reproducible |
| Q169 | Contradicts Q200 |
| Q171 | Omits the +25 finish points |
| Q202 | 5.897 m/s not reproduced |
| Q584 | SPI computes to 0.77, key says 0.09 |
| Q911 | Driver change not subtracted |
| Q125 | 60 DOF vs 30 |
| Q824 | Ignores the 2026 lost-part DQ rule |
| Q512 | Legacy rule; fit not confirmed |
