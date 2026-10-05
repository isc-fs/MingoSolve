# Archetypes for calc_c.md (electrical: electronics / HV / DV), 120 questions

All numbers marked ✓ were reproduced in Python (`scratchpad/chk.py`, `chk2.py`). "fig" means the question needs an image or table that isn't in the text.

## 1. `battery_thevenin_load` (HV/LV): source plus internal R plus load. 9 questions
- Eqs:
  - `Eq(V_oc, N_s*V_cell)`
  - `Eq(R_pack, N_s*R_cell/N_p)`. If the stack/string R is given: `R_stack/N_stacks_parallel`.
  - `Eq(V_t, V_oc - I*R_pack)`
  - `Eq(P_load, I*V_t)`, so `P = I*V_oc - I**2*R`. Quadratic: take the **smaller root** `I=(V_oc-sqrt(V_oc**2-4*R*P))/(2*R)`.
  - `Eq(P_loss, I**2*R_pack)`
  - Max power transfer: `Eq(P_max, V_oc**2/(4*R))`
  - Ohmic load from its rating: `R_L=V_rated/I_rated`, `I=V_oc/(R_L+R_i)`
- Vars: V_cell/OCV [V], N_s, N_p, R_cell [Ω] (mΩ in text), I [A], P [W].
- IDs: Q20, Q34, Q113, Q183(fig: OCV-vs-SoC curve), Q370, Q912, Q617, Q1070(datasheets), Q118/Q1086 (cable drop, see 2).
- Regression tests:
  - Q34: 4 stacks of 103 cells in parallel, R_stack = 0.32 Ω (so R = 0.08), V_cell 3.80, P = 30 kW → **77.9 A** ✓
  - Q113: 112S3P, 0.32/3 Ω, 3.80 V, 35 kW → **84.0 A** ✓
  - Q370: 12 V/30 A device, 14.4 V, R_i 0.1 → P_loss **82.9 W** ✓
  - Q912: 12 V, 20 mΩ → **1800 W** ✓
  - Q20: answer 48.8 kW at 135 A, OCV 3.92, R_cell 2.3 mΩ. The config is in the figure; 2×(48s2p) = 96s2p reproduces 48.79 kW ✓. The 24.4 kW option is the one-pack trap.
- Q617 (short-circuit breaker):
  - `R_w=1.3*(2*l)/(κ*A)` with l = 10 m, κ = 56, A = 6 mm² (go + return)
  - `I_sc_min = V0/(Ri+R_w)`, `In = floor10(I_sc_min/10)`
  - `I_sc_max = V0/Ri`, `Icn = ceil10k(I_sc_max)` → **560 A, 30 kA**. The 1.3 factor applies to the wire only.
- Q183: R = 142·1.5 mΩ/2, V_oc = 71·(V40% + V80%) with OCVs rounded to 0.1 V, P = 80 kW (rule max) → 153–156 A.
- Gotchas:
  - "Stacks of N cells (NsMp)" means M stacks in **parallel**.
  - The resistance is sometimes given per stack, not per cell.
  - Rule power limit is 80 kW.

## 2. `conductor_resistance` and `resistive_self_heating`. 9 questions
- Eqs:
  - `Eq(R, rho*L/A)`, or `Eq(R, L/(kappa*A))`
  - `Eq(R_T, R_0*(1+alpha*(T-T_0)))`
  - Equal R per length: `Eq(rho1/A1, rho2/A2)`
  - Self-heating, steady state:
    - `Eq(T, T_amb + I**2*R_T*R_th)` coupled with R_T
    - for a cable: `Eq(I**2*rho_T/A, lam*pi*d_avg*(T_max-T_amb)/t_ins)`, where `d_avg = d_cond + t_ins` (mean of bare and insulated diameter) and `d_cond = sqrt(4A/pi)`
  - Starter efficiency: `eta = 1 - I*R_cables/V`
- IDs: Q118, Q1086, Q489, Q730, Q732, Q617, Q71, Q956, Q877(fig/table, transient).
- Tests:
  - Q118 / Q1086: ρ = 0.017 µΩm, 5.7 m, 10 mm², 150 A, 12 V → **87.9 %** ✓
  - Q489: Cu Ø10 mm, Al busbar 5 mm thick → **24.20 mm** ✓. Uses ρCu = 1.72e-8 and ρAl = 2.65e-8.
  - Q956: 1 Ω, 10 ppm/K, R_th 1 K/W, 25 °C, 13 A → **13023 mV** ✓ (solve the fixed point)
  - Q71: A 2 mm², t 0.36 mm, λ 6 mW/(K·m), ΔT 20 K, ρ(70 °C) = 26.4n·(1 + 0.0041·50) → I = 11.35 → fuse **11 A** (floor) ✓
  - Q730: nichrome ρ = 1.0e-6 Ωm, L 20.24 m, A 1.5e-6 → **13.49 Ω**
  - Q732: 100·(1 + 0.0038·30) → **111.40**
- Gotchas:
  - Use the total loop length (supply + ground).
  - Fuse sizes are floored (largest step not exceeding the limit).
  - ρ in the temperature formula is referenced to 20 °C.
  - Q877 answer: 36.5 ± 0.2 °C.

## 3. `energy_budget` (E = P·t, Ah budgets, kinetic energy). 15 questions
- Eqs:
  - `Eq(E, P*t)`
  - `Eq(Q_Ah, sum(I_i*duty_i)*t_h)`
  - `Eq(E, N_cells*V*C_Ah)`
  - `Eq(E_mech, eta*E_el)`, `Eq(d, E_mech/F)`
  - `Eq(KE, m*v**2/2)`, `Eq(P_avg, KE/t)`
  - Linear power ramp: `Eq(E, P_max*t/2)`
  - Charging: `P = min(P_charger, I_set*V_batt)`, `t = ΔSoC*E_nom/P`
  - Pack count: `n = ceil(I*t/(C*DoD))`
  - Balancing: `t = ΔSoC*C/I_bal`
  - I²R losses: `E = Σ I_k**2*R_pack_k*t_k`, with separate charge and discharge R
- IDs: Q186, Q371, Q449, Q496, Q520, Q559, Q609, Q648, Q754, Q940, Q954, Q997, Q357, Q863, Q308(rules).
- Tests:
  - Q186: 10×12 V×160 Ah, η 0.8, F 1.2 kN → **46.08 km** (range 44–48)
  - Q371: (2 + 6 + 4 + 0.75·10) A × 25/60 h → **8.125 Ah**
  - Q449: 80 kW, 3.95 s → **158 kJ**
  - Q609: 3.66 kWh / 5.5 kW → **40 min**. The 12 A·510 V = 6.1 kW limit is above 5.5 kW, so the charger power limit wins.
  - Q954: 4.2 A·8 h / (10 Ah·0.86) = 3.9 → **4**
  - Q357: R_dis = 120·0.5m/3, R_ch = 120·0.7m/3, 73 % at 30 A and 18 % at 3 A over 1440 s → **18987 J** ✓ (no official answer)
- Gotchas:
  - Q520 says "work" but the answer is average power (32.8 kW).
  - Q496: "consumed 8 kWh" is the old pack: `E_drive = 8·1.2`, `E_new = E_drive/1.6` = **6**.
  - Q997: 2.5 kW is the *thermal* need.
    - Per car: `P_el = 2.5/0.7/0.9 = 3.97 kW` DC, which is under 10 A·400 V.
    - Energy: 15.5 kWh / 6.5 kWh → **3** cars.
  - Q648: 230·16 = 3680 W → unplug **2** laptops.
  - Q559: 13.9 kWh.
  - Q940: 21.0 kWh.
  - Q754: 1.5 h.
  - Q308 (no official answer): the energy-meter rule depends on the rule version. Net is 15.176 MJ; 19.912 − 0.9·4.736 = 15.650 MJ is also an option. Unresolved.
  - Q863:
    - Part 1: `Σ_{v=90..109 kph} ½·220·v²/0.936` = **0.5 kWh** ✓.
    - Parts 2 and 3 (17 runs; 0.356 kWh): the official regen model doesn't reproduce with `regen = 0.94·η·KE`, which gives 0.113 kWh.
    - Back-solved: recovered energy is ≈ 30.7 % of the gross draw.
    - Part 3 rule: `Σ_{1..19} net + gross_20`.

## 4. `rules_lookup` (table or threshold functions, no algebra). 14 questions
- PCB spacing:
  - Table rows (V range: over surface / through air with cut / under coating):
    - ≤50 V: 1.6 / 1.6 / 1 mm
    - 50–150 V: 6.4 / 3.2 / 2 mm
    - 150–300 V: 9.5 / 6.4 / 3 mm
    - 300–600 V: 12.7 / 9.5 / 4 mm
  - Q16: 237 V, no cut → **9.5 mm**
  - Q403: 336 V, coated → **4.0 mm**
- Q557: TS-GLV in the same enclosure above 200 V → **30 mm** through air (EV4.3.3).
- Q558: TSAL/AIR LED threshold = min(60 V, V_max/2) → **50 V**.
- Q299 / Q339: insulation test voltage is 250 V if V_max ≤ 250 V, else 500 V → Q299 **500 V**, Q339 **250 V** (no official answers).
- Q354: TS measuring-point resistors are 5k (≤200 V), 10k (≤400 V), 15k (≤600 V), 20k (≤800 V) → **10 kΩ** (no official answer, from memory).
- Other rules:
  - Q199: **2** TS voltage measurements.
  - Q45: catch can is 10 % of oil volume → **1.4 l**.
  - Q521 / Q545: ground clearance **30 mm** (T2.3.2).
  - Q511: **1** year.
  - Q687: impact attenuator 7350 J = **0.0020416667 kWh**.
- Q795 (DV EBS): `s = v*t_d + v**2/(2*a)` with v = 40 km/h, t_d = 0.2 s, a = 5 m/s² (ABS-failure case) → **14.568 m**. The distractors use a = 10 or no delay.
- Gotcha: rule numbers and limits change by year. Keep these as editable tables.

## 5. `cell_energy_segment_limits`. 5 questions
- Eqs:
  - `Eq(E_cell, V_x*C_nom)`
  - Segment limits: `n_max = min(floor(120/V_max_cell), floor(6e6/E_cell_J))` (the 12 kg mass limit appears only as a distractor)
  - Pack energy: `Eq(E_pack, N_s*N_p*E_cell)`
  - Specific energy: `Eq(m_cell, E_cell/e_spec)`
- IDs: Q577, Q770, Q781, Q396, Q934(fig).
- Tests:
  - Q577: 4.2 V·20 Ah = 302.4 kJ → 6 MJ/302.4 kJ = **19** ✓
  - Q781 (= Q770): 4.25·14.7 Ah = 224.9 kJ → **26**. Use the datasheet V_max, not the 4.05 V charge limit.
  - Q396: previous cell 42 kg/420 = 0.1 kg × 160 Wh/kg = 16 Wh / 3.7 V → **4.32 Ah**
- Gotchas:
  - **V_x is inconsistent across questions.** Q577 and Q781 use V_max; Q396 ("Rules 2022") uses V_nom.
  - Expose V_x as a parameter and check which option set matches.
  - Q934: remaining E = 240·E_cell(EV5.1.2)·SoH 0.9·SoC(OCV 3.88 from the figure); laps = floor(E/303 Wh) → **12**.

## 6. `rc_rl_transient` (discharge, latch timing, inductive step, capacitor energy). 9 questions
- Eqs:
  - `Eq(v, V_f + (V_0-V_f)*exp(-t/(R*C)))`, so `t = R*C*ln((V_0-V_f)/(V_x-V_f))`
  - Discharge rule (≤60 V within 5 s): `R_max = 5/(C*ln(V0/60))`
  - Peak power: `P_pk = V0**2/R`
  - Average power over the window: `P_avg = C*(V0**2-60**2)/(2*5)`
  - Relay holding: `I0 = V/R_coil`, `t = R*C*ln(I0/I_drop)` ≤ 250 ms
  - Inductor step: `Eq(V_load, V_bat - I*R - L*dIdt)`
  - Capacitor energy: `Eq(E, C*(V1**2-V2**2)/2)`
  - Constant-current sink: `t = C*ΔV/I`, with `I = (V_gate - V_GSth)/R_s` (figure)
- IDs: Q245, Q330, Q358, Q159(fig), Q724(fig), Q430(fig), Q497, Q536, Q753.
- Tests:
  - Q497: 12 − 2·1 − 1µH·2 A/µs → **8.0 V**
  - Q536: 12 − 3.75 − 1.5 → **6.75 V** (accepted range 6.7–6.8)
  - Q753: ½·2·(100² − 50²) → **7.5 kJ**
  - Q245: C = 2×900 µF, V0 = **V_nom** 120·3.3 = 396 V → R = 1472 Ω, P = V0²/R = **106.5 W** ✓. Using V_max 438 V gives 137.3 W, which is a distractor.
  - Q330: average power with V0 = V_min 336 V: C(V0² − 60²)/10 = 19.67 → **19.6 W** (truncated). V_nom gives 27.6 and V_max gives 33.9; both are distractors.
  - Q358: 13.8/15 = 0.92 A → 0.2 A, t = 250 ms → C = 0.25/(15·ln 4.6) = 10.92 mF → **10.9 mF** (no official answer)
- Gotchas:
  - The source voltage for the 60 V/5 s rule varies (V_nom, V_min, V_max). "Minimum possible" points to V_min.
  - Inverter capacitances add per inverter.
  - Q159 answer: 399.4 ms, 13.8 s (R1 68k, R2 2.2M, C 3.3 µ, Vf 0.7; topology in figure).
  - Q724 answer: 5000.0 ms (figure).

## 7. `sampling_adc` (Nyquist, aliasing, ADC code, resolution). 7 questions
- Eqs:
  - `Eq(fs_min, 2*f_max)`
  - Alias: `f_a = abs(f - k*fs)` minimised over integer k
  - ADC code: `Eq(code, floor(V_in*k_div/V_ref*2**n))`
  - Temperature step for a sensor scaled so that T_max maps to V_ref: `dT = (V_ref/2**n)/(S*k)`, i.e. `dT = T_max_K/2**n`
  - Discrete cosine: `cos(w0*n*T) = (-1)**n` gives `w0 = (2k+1)*pi/T`
- IDs: Q228, Q383, Q82, Q843, Q242(fig divider), Q1007, Q1008(datasheet).
- Tests:
  - Q228 → **440 Hz**. Q383 → **380 Hz**.
  - Q82: |400 − 340| → **60 Hz**
  - Q1007: 423.15 K/1024 → **0.413 K** ✓
  - Q843: **1, 3, 5** (×π·10³ rad/s)
  - Q242: answer 2835…3780 ⇒ k_div ≈ 0.692 (e.g. 9/13), with 4096 counts.
- Q1008: HASS: `Vout = 2.5 + 0.625*I/I_PN`. Pick the smallest I_PN (200-S), then `code = Vout/3.3*4096`. Official 3617 implies I ≈ 132.5 A. Not reproduced: the stated I_max = 120 A gives 3568.

## 8. `ac_impedance_filters_rf`. 9 questions
- Eqs:
  - Impedances: `Eq(X_C, 1/(2*pi*f*C))`, `X_L = 2*pi*f*L`
  - 1st-order RC low-pass: `Eq(fc, 1/(2*pi*R*C))`, `Eq(phi, -atan(f/fc))`, `|H| = 1/sqrt(1+(f/fc)**2)`
  - Pick the nearest E12 value
  - RF:
    - Transmission line: n·λ/2 → `Z_in = Z_L`
    - `Γ = (Z_L-Z0)/(Z_L+Z0)`
    - `VSWR = (1+|Γ|)/(1-|Γ|)`
    - Transmitted power fraction: `1-|Γ|**2`, with `|Γ| = (VSWR-1)/(VSWR+1)`
  - Notch frequency: `f = 1/(2*pi*sqrt(L*C))` or the twin-T formula (figure)
- IDs: Q341, Q386, Q665, Q666, Q1029, Q1030, Q840(fig), Q842(fig, 3-D network), Q881(fig, superposition of sine sources → 3.000 V).
- Tests:
  - Q665: −atan(200/300) → **−33.7°**
  - Q386: −atan(0.6) = **−30.96°**
  - Q341: **21.221 Ω**
  - Q666: 1/(2π·300·220n) = 2411 Ω → E12 **2.2 kΩ**
  - Q1029: Z = 50 + 0.085 + 0.045 = 50.13 Ω, VSWR 1.00. With 9 pF: X = −40.84 Ω, VSWR **2.21** ✓
  - Q1030: VSWR 6.47 → **46 %**
- Gotchas:
  - Phase is negative for a low-pass filter.
  - Q840 answer: 72.34 Hz. Q842 answer: 1.93 kΩ.

## 9. `circuit_network_solve` (procedural: MNA/nodal, series-parallel reduction, superposition). 7 questions
- Algorithm: build a netlist and solve it with Modified Nodal Analysis in sympy (`linsolve`). Use exact `Rational` arithmetic for fraction answers (Q67 needs `6/7`).
- IDs:
  - Q67 (fig, U_AB = **6/7·Uq**)
  - Q516 (fig, R = 3 Ω each → **2.2 Ω**)
  - Q532 (text)
  - Q682 (fig, I3 = **−0.5**)
  - Q870 (fig, **50, −6.25**)
  - Q942 (fig, **335 Ω**)
  - Q965 (fig, I5 = **95.8**)
- Test Q532: V = 80·1.25 = 100 V; I_x = 10 − 4 − 1.25 − 2 = 2.75 A → **36.36 Ω**.
- Gotcha: keep the sign convention from the figure (Q682 and Q870 answers are negative).

## 10. `divider_bridge_sensor` (NTC/PTC divider, Wheatstone, strain gauge). 5 questions
- Eqs:
  - Divider: `Eq(V_out, V_dd*R2/(R1+R2))`
  - NTC: `Eq(R_T, R25*exp(B*(1/T-1/298.15)))`
  - Bridge: `V_AB = V*(R2/(R1+R2) - R4/(R3+R4))`
  - Strain: `eps = F/(E*pi*d**2/4)`, `dR/R = GF*eps`
  - Worst case: pick tolerances so V_out is **lowest** at the limit temperature
- IDs: Q820, Q1005, Q429(fig), Q433(fig, answer 2.44 V), Q242.
- Tests:
  - Q1005: T_lim = min(datasheet 70, rule 60) = 60 °C → R2 = 100·0.9 = 90, R1 = 55 → **3.10 V** ✓
  - Q820: T_lim = 55 °C (datasheet lower) → R2 = interp 100 Ω·0.95 = 95, R1 = 55 → 3.1667 → **3.16 V**
- Gotchas:
  - Cell temperature limit is min(datasheet, 60 °C rule).
  - Round **down** (the answer must guarantee no trip).

## 11. `opamp_stage`. 4 questions
- Eqs:
  - Improved Howland: `Eq(I_out, (R3/R1)*(Vp-Vn)/R4)`
  - Ideal inverting/non-inverting gains and in-amp gain (figures)
- IDs: Q748, Q946(fig: diode + 3R, answer −10368 mV), Q433, Q159.
- Test Q748: 5 V, R4 100 Ω → **50.0 mA**.

## 12. `digital_timing_numeric` (UART, CAN, timers, two's complement, float32). 6 questions
- UART: `t = N_bytes*(1+8+parity+stop)/baud`. Q326: 200·10/115200 → **17.36 ms**
- Timer: `ARR = t*f_clk/prescaler`. Q319 → **36 000 000**. Here "prescaler value 2" is the literal divide factor (no −1).
- CAN 2.0A best case: 64 data bits/frame; 108 bits + 3 IFS = 111 bits/frame.
  - Q675: 290 s·128 kbit/64 = **580000** frames; ·111/500k = **129 s**
- Two's complement: `(2**n + x) % 2**n` to n-bit binary. Q959 (n = 15, −20) → **111111111101100**
- Float32 accumulation stall: simulate with `np.float32` until `x+y == x`. Q448 → **32768** ✓
- Noise margin: `NM = min(VOH-VIH, VIL-VOL)`, `n_buf = ceil(k*L/NM) - 1`. Q844: LVTTL NM = 0.4, 0.75 V → **1**

## 13. `battery_capacity_effects`. 4 questions
- Peukert: `Eq(C2, C1*(I1/I2)**(k-1))`. Q780 (= Q768): 10·0.1^0.2 → **6.31 Ah**
- Balancing: Q754 (see 3); Q1045 is a procedural simulation from the figure (answer 3112).

## One-offs (1-line method)
- Q165: angular momentum `ω_body = I_tail·ω_tail/I_body`, `t_turn = π/ω_body` = **2.13 s**. Fall with quadratic drag: `h = vt²/g·ln cosh(g t/vt)` with `vt = sqrt(2mg/(ρ cw A))` → **1.80 s** ✓
- Q419: adiabatic `V2 = V1·(p1/p2)^(1/1.4)` → **5.18 l**
- Q490 / Q535: `ΔCOG = Δclearance·m_sprung/m_total`, with Δclearance from the 30 mm minimum.
  - Q490 → **279.06 mm**
  - Q535: 20·190/235 → **16.2 mm**
- Q500: s = ½·4·5² + 20·5 = 150 m → **164.042 yd**
- Q520: see 3.
- Q530: Q = I·t → **720 C**
- Q534: `L = μ0 N² A/l`, `E = ½ L I²` → **0.15 J**
- Q933: `ψ = N·(Br·A·Pc/(Pc+Pm) + N·i/(Rc+1/Pm))`, Pc = 1/Rc → **16.67 Vs**
- Q1032: buck `D = Vo/(Vin·η)`, `ΔI = (Vin−Vo)·D/(L·f)` → **2.2 A** (η goes into D)
- Q849:
  - `½CV² = ½mv² + m g l sinα`, launch point (l cosα, l sinα)
  - Projectile through the target: `v² = g dx²/(2cos²α (dx tanα − dy))` → **46 V** ✓
  - Including the rail's potential energy matters: without it you get 42.
- Q853: sphere cap `V = πh²(3R−h)/3` = 171·0.3 l (you + 170). Opening top sits at 0.45 m height (R = 0.25, opening r = 0.15) → x = 0.45 − h → **10 cm** ✓
- Q969: `v = sqrt(GM/R)` → **7.9 km/s**; `T = 2πR/v` → **84.3 min**
- Q1069: Reynolds match `v_w = v_a·(ν_w/ν_a)/scale`. ν comes from the figure table; the official answer is 8.40. Pump check: Q = v·0.2 m² (full section) → 100800 > 95000 l/min → **No**
- Q173: available minutes (8 + 10 + 11.5 h = 1770 min) × 4 stations / 40 cars → **177 min/car**
- Q434:
  - Ray `((u−cx)/fx, (v−cy)/fy, 1)`, normalised × d
  - `p_v = Rz(yaw)·Ry(pitch)·Rx(roll)·p_c + t` → **1.636, 1.405, 0.262** ✓
- Q413: `I = 400/1.33 = 301 A` (3·In), then read the fuse chart → **6 s**
- Q864 (removed): `P_new = P·T_old/(T_old + 1/(2π·50)… )`. The official uses τ = 20 ms: 100·30/50 → **59.988**
- Q854: 0/1 knapsack (DP), table in figure. Two accepted answers.

## Not computable from text (need image, file, datasheet or external data)
- Images/figures: Q67, Q516, Q682, Q870, Q942, Q965, Q840, Q842, Q881, Q159, Q724, Q430, Q429, Q433, Q946, Q242, Q183, Q934, Q20, Q1045, Q854, Q413, Q1068 (GD&T, 0,117), Q1069 (ν table), Q877
- Files or datasheets:
  - Q622: FFT of ACC_data.txt → 40 Hz
  - Q982: tensile file → 0.5 GPa
  - Q978: termination resistors, EIA-96 → 66.5 / 59 / 61.9
  - Q1008: LEM datasheet
  - Q1070: Eaton/Molicel datasheets → 70–80 ms
- External results: Q960 (FSG 2024 points, no answer).
- Removed or garbled: Q855 (388,8, removed).

## Chains (multi-step, worth composite solvers)
- Q617: wire R → I_sc min/max → In/Icn step rounding
- Q245 / Q330: rule 60 V/5 s → R_max → peak or average power
- Q1008: cell I_max → sensor range pick → Vout → ADC code
- Q934: OCV → SoC (linear interpolation) → E_cell (rule) × SoH → laps
- Q183: OCV(SoC) curve → pack V_oc → quadratic I at 80 kW
- Q863: KE series → efficiency → regen → cumulative budget
- Q997: thermal → electrical → per-car current cap → energy → ceil
- Q849: projectile → capacitor energy
- Q853: geometry → cap volume root
- Q490 / Q535: rule clearance → COG shift
- Q820 / Q1005: rule temperature → interpolation → worst-case tolerances → divider
