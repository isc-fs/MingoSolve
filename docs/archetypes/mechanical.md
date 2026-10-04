# Archetypes for calc_b.md (mechanical questions)

The whole file was read (about 140 questions). Answers marked ✓ were recomputed in Python or by hand and match the official answer. A ⚠ marks a mismatch or an unclear official method.

Global conventions to bake into the library:
- Accept inputs in km/h, rpm, bar (gauge or absolute), °C, mm, inch (0.0254 m), mmHg (133.322 Pa) and g.
- Convert everything to SI internally.
- Gauge vs. absolute pressure is the #1 trap. Add an explicit `gauge=True` flag and a `p_atm` input.
- Skidpad centreline radius is R = 9.125 m (15.25/2 + 1.5). The rules constants needed are listed in the next line.
- Rules constants needed: minimum wheelbase 1525 mm; minimum ride height 30 mm; baseline steel yield 305 MPa; bend radius ≥ 3·OD; edge distance e ≥ 1.5·D; skidpad inner diameter 15.25 m with a 3 m track.

---

## 1. load_transfer_axle_loads (vehicle dynamics), 12 questions
Equations:
- Static: `Eq(Fzf, m*g*b/L)`, `Eq(Fzr, m*g*a/L)`, `Eq(a+b, L)`. Here a is the distance CG→front axle and b is CG→rear axle; front fraction = b/L.
- Longitudinal transfer: `Eq(dFx, m*ax*h/L)`.
- Lateral transfer on one axle, with that axle's share of mass: `Eq(dFy_axle, m_axle*ay*h/t)`. Wheel load = `Fz_axle/2 ± dFy_axle`.
- Rollover (stiff car): `Eq(ay*h, g*t/2)`.
- Inside-wheel fraction: `Eq(frac_in, 1/2 - ay*h/(g*t))`.
- Wheelie / front lift: `Eq(m*g*b, m*ax*h)` with `ax = mu*g`, which gives `h = b/mu`.
- Tilt-scale CG height: `Eq(h, r + (Ff_tilt - Ff_flat)*L/(m*g*tan(theta)))` with `sin(theta)=dz/L`.
- Aero split by CoP: `Eq(Ff_aero, F_L*(L - x_cop)/L)`.
- Wing load with moments about the axles (Q65 convention):
  `Eq(dFr, (F_L*(a + yW) + F_D*(zs + zW))/L)`.
  yW and zW are measured from the CG. At constant speed the drag is reacted at the ground, so there is no inertial relief.

Variables: m [kg], g, L wheelbase [m], t track [m], h CG height [m], ay/ax [m/s²], mu, weight distribution %, F_L/F_D [N].

Questions: Q27, Q65, Q180, Q289, Q495, Q515, Q522, Q858, Q918 (step), Q923, Q1065, Q185 (scaling only).

Regression tests:
- Q27: L=1.525, 70 % rear → b=0.3·1.525, mu=2 → h=0.23 m ✓
- Q1065: m=250, 45 % front, t_f=1.2, h=0.28, ay=1.8 g, right turn → FL = 0.45·m·g/2 + 0.45·m·1.8g·h/t_f = 1015 N ✓
- Q515: h=0.205, r=14.5, t=1.24 → v=sqrt(g·r·t/(2h)) = 74.7 km/h ✓
- Q858: t=1.2, ay=1.2 g, 25 % inside → h=0.250 m ✓
- Q65: rho=1.2, v=20 m/s, ClA=0.6, CdA=0.3, L=1.6, a=0.51·1.6, yW=0.8, zW=0.4, zs=0.3 → per rear tyre ΔF/g = 9.02 kg ✓

Gotchas:
- In Q65 the sketch distances are measured from the CG. If you take them from the axle, or subtract the drag inertia (zW−zs), you get 11.2 kg or 8.3 kg.
- Q180: the first skidpad lap is a right turn, so the left wheels are outside.
- Q289: the scale readings do not change when tilted, so h equals the wheel radius (222 mm).
- Q522: CG from rear = front% · L (the "????" unit is mm).
- Q495: the given values are missing from the text (image), so it is not computable. Official answer 838 N.
- Q923 was removed. Sprung roll transfer m_s,f·ay·(h−h_rc)/t gives about 200 N, but the official answer is 2000 N (looks like a ×10 error).

## 2. gear_train_ratio (powertrain), 11 questions
Equations:
- Simple/compound train: `Eq(n_out, n_in*Product(z_driver/z_driven))`. Chain = gear for the ratio; idlers cancel.
- Ratio: `Eq(i, z_out/z_in)`.
- Willis (planetary): `Eq((n_s - n_c)/(n_r - n_c), -z_r/z_s)`.
  - Ring fixed: `n_c = n_s*z_s/(z_s+z_r)`.
  - Arm-driven external epicyclic with A fixed: `n_B = n_C*(1 + z_A/z_B)`.
- Compound planetary (sun→P1, P2→ring): `Eq(i, 1 + z_r*z_p1/(z_s*z_p2))`.
  z_p2 comes from equal centre distance: `m1*(z_s+z_p1) = m2*(z_r - z_p2)`.
- Shift time (dog ring): `Eq(t, (theta/360)*60/abs(n_gear_new - n_shaft))`.

Questions: Q426, Q572, Q598, Q612, Q664, Q672, Q736, Q751, Q805, Q757 (step), Q811 (step).

Regression tests:
- Q612: 3600·(20/45)(24/55)(30/43)(15/53) = 137.9 ✓
- Q572: z_s=25, z_p1=76, m1=2, m2=3.015, z_r=86 → z_p2=19 → i=14.76 ✓
- Q426: 200·(1+40/50) = 360 ✓
- Q805: primary 3000 rpm. The secondary turns at 3000·17/32 (2nd) while the 3rd gear wheel turns at 3000·19/28. Relative speed = 441.96 rpm, 30° → 11 ms ✓

Gotchas:
- Q598 and Q736 need tooth counts from the figure. Q598's 0.272 = 3/11, i.e. z_s/(z_s+z_r) with z_s=30, z_r=80.
- Q751: z2 = 16·2.5 = 40, then 42/16 = 2.6.
- Q805: use the relative speed between the gears, not the shaft speed.

## 3. aero_force_dynamic_pressure (aero), 12 questions
Equations:
- `Eq(F, Rational(1,2)*rho*C*A*v**2)` (lift or drag). Some questions give the product C·A·rho directly (Q363).
- Scaling: `F2/F1 = (v2/v1)**2`, `F2/F1 = rho2/rho1`.
- Reynolds matching: `Eq(v_m*L_m, v_p*L_p)`.
- Terminal velocity: `Eq(vt, sqrt(2*m*g/(rho*Cd*A)))`.
- Fall time from height H: `Eq(t, vt/g*acosh(exp(g*H/vt**2)))`.

Questions: Q185, Q366, Q630, Q769, Q785, Q1056, Q1057, Q1063, Q631, plus as a step in Q65, Q75, Q826, Q883, Q811.

Regression tests:
- Q785: rho=1.22, Cd=1.6, A=0.7, 50 km/h → 131.8 N ✓
- Q630: 1 − 0.945/1.225 = 23 % ✓
- Q366: 65·0.8/0.6 = 86.67 km/h ✓
- Q185: (120/100)²−1 = 44 % ✓
- Q1056/1057: m=165, CdA=1.54, rho=1.2, H=40000 ft → 41.85 m/s, 294 s ✓

Gotchas:
- Sea-level density is 1.225 (Q630).
- 40000 ft = 12192 m.
- Q631 needs context ("question 8") and Q1063 needs missing data, so neither is computable.
- Q185: the geometry is a red herring.

## 4. point_mass_cornering_with_downforce (dynamics/aero), 9 questions
Equations:
- `Eq(m*v**2/R, mu*(m*g + Rational(1,2)*rho*ClA*v**2))`, which gives `v**2 = mu*g*R/(1 - mu*rho*ClA*R/(2*m))`.
- Skidpad lap: `Eq(t_lap, 2*pi*R/v)`. Pure kinematic version: `Eq(t, 2*pi*sqrt(R/ay))`.
- Ceiling driving (Q792): `Eq((mu - c_rr)*(Rational(1,2)*rho*ClA*v**2 - m*g), Rational(1,2)*rho*CdA*v**2)`.

Questions: Q90, Q142, Q197, Q363, Q695, Q762, Q792, Q974, Q180 (step), Q1024 (variant).

Regression tests:
- Q90: mu=1.4, m=240, ClA=3.2, rho=1.1, R=9.125 → 11.76 m/s ✓ (range 11.7–12.1)
- Q363: rho·ClA = 6 kg/m (the sign means downforce), m=260, mu=1.5, R=20 → 76.4 km/h ✓
- Q695: run 1 m=200, ClA=3.36; run 2 m=180, ClA=−0.27 (lift); mu=1.5, rho=1.1, R=9.125 → −7.1 % ✓
- Q142: R = 7.625 + 1.45/2 = 8.35, ay=15 → 4.688 s ✓
- Q792: m=180+15, rho=1.225 (ISO 13443: 15 °C, 101.325 kPa), g=9.81, ClA=3.5·2.35, CdA=0.8·2.35, mu=1.8, crr=0.1 → 20.95 m/s ≈ 75 km/h ✓

Gotchas:
- Negative cl means downforce in some questions (Q75, Q1024) and positive in others.
- Q974 needs a trivia lookup: the Handbook day is 14 (both quadratic roots are 14 and 35; 14 is the one used). That gives a driver mass of 70.
- Q197: computes to 0.632, which matches no option, and the official CORRECT field is empty.

## 5. constant_accel_kinematics (dynamics), 9 questions
Equations:
- `Eq(v**2, v0**2 + 2*a*s)`, `Eq(v, v0 + a*t)`, `Eq(s, v0*t + a*t**2/2)`.
- Accel then v-limit (Q868): t = vmax/a + (s − vmax²/(2a))/vmax.
- Projectile from a crest: `Eq(h, g*t**2/2)`, `Eq(s, v*t)`.
- Coast-down drag: `Eq(m*(v1-v2)/dt, Rational(1,2)*rho*SCx*vmean**2)`.
- Braking with mean aero: `Eq(s, m*v**2/(2*(F_brake + k*Rational(1,2)*rho*CdA*v**2)))` with k = 2/3.
- Fixed force over a fixed distance: `t ∝ sqrt(m)`.

Questions: Q28, Q115, Q142, Q602, Q620, Q856, Q868, Q900, Q1064 (chain).

Regression tests:
- Q868: vmax=56 km/h, a=4, s=75 → 6.77 s ✓
- Q856: m=760, 300→265 km/h in 2 s, rho=1.225 → SCx=0.98; distance = mean v·t = 156.9 m ✓
- Q620: 0.87² → lose 24 % ✓

Gotchas:
- Q28: the exact answer is 510.00. The official 509.85 rounds v to 14.14 m/s before the second phase. Make intermediate rounding an option.
- Q115: the naive answer is 79.7 km/h, but the official is 81.3 km/h. The sketch geometry is needed. ⚠
- Q602: computes to 21.54 m. The closest option, and the official one, is 21.97 m. ⚠
- Q900: the parameters are in an image, so it is not computable.

## 6. heat_capacity_energy_balance (thermo), 9 questions
Equations:
- `Eq(Q, m*c*(T2-T1))`, `Eq(P*t, Q/eta)`, `Eq(Qdot, mdot*c*dT)`.
- Latent heat: `Eq(Q, m*L)`.
- Thermal expansion: `Eq(dL, alpha*L0*dT)`. For the wire in Q164: `dL = alpha*E/(rho*A*c)`, independent of length.

Questions: Q83, Q130, Q164, Q417, Q526, Q830, Q1062, Q1071 (C), Q862 (expansion + geometry).

Regression tests:
- Q130: 2.4·900·55/2000 = 59.4 s → 59 ✓
- Q83: 17·60·4.19·(55−40) = 64107 kJ/h ✓
- Q526: 1.2 kg·4186·82/860 = 479 s ✓
- Q1062: c_ice=2, c_w=4.187, Lf=335, Lv=2200 → 14.15, 1.77, 26977.14 ✓

Gotchas:
- Tea questions use water at 1 kg/L and c ≈ 4186, heated to 100 °C.
- Q830: water volume = pool + 15 L, then divide by eta and by 4 kWh/kg.
- Q1071 C: rear brakes are dead, so 2 front discs share 70 % of E.
- Q862 needs the figure.

## 7. ideal_gas_pneumatics (thermo/dynamics), 7 questions
Equations:
- `Eq(p1/T1, p2/T2)` with p absolute and T in K.
- `Eq(p*V, m*Rs*T)`.
- Isothermal bottle: `Eq(dp*V_bottle, p_ref*V_ref)`.
- Shot count: `Eq(n, p_gauge*V_bottle/(p_atm*V_shot))`.

Questions: Q79, Q178, Q254, Q456, Q596, Q727, Q733.

Regression tests:
- Q178: 0.7 bar gauge + 1.013 at 20 °C → 60 °C → 0.934 gauge ✓
- Q456: annulus V = pi/4·(OD²−ID²)·w (18/10/7 in), Rs=287, p_atm=1.013 bar at 20 °C → 1.5 bar gauge at 60 °C → Δm = 29 g ✓
- Q79: 51 bar abs, 4 L, minus 20 L at 1 bar → 45.0 bar gauge ✓
- Q733: 115·0.6/(0.99992·0.5) = 138 ✓ (115 bar is treated as gauge; temperature is irrelevant)

Gotchas:
- Q254 treats 1.8 bar as absolute: 1.8·303.15/358.15 = 1.52.
- Q596 and Q727 do not reproduce. Standard physics gives 1.43 gauge / 2.41 abs (Q596) and 1.05 gauge / 2.03 abs (Q727), but the official answers are 2.09 and 0.50. Pick by option elimination. ⚠

## 8. section_properties_beam (structures), 10 questions
Equations:
- Tube: `Eq(I, pi/64*(D**4-d**4))`. Rectangular hollow: `Eq(I, (B**4-b**4)/12)`. Solid rectangle: `Eq(I, b*h**3/12)`.
- Sandwich, two skins, core ignored: `Eq(I, 2*(b*t**3/12 + b*t*((d+t)/2)**2))`.
- 3-point bend at yield: `Eq(F, 4*sigma_y*I/(c*L))`. Centre-load deflection: `Eq(delta, F*L**3/(48*E*I))`.
- Axial: `Eq(E, F*L/(A*dL))`.
- Strain: `eps_x = diff(u, x)`.

Questions: Q1, Q938, Q77, Q259, Q316, Q473, Q481, Q499, Q507, Q749 (+ Q657, Q1058, Q903, Q925/Q1043, Q566 below).

Regression tests:
- Q473: 25×25×2.2, sigma_y=305, L=400 → 4.28 kN ✓
- Q481: 25×2.5 round → 2.76 kN ✓
- Q77: 25.4×1.6 → 30×1.5 gives +60.7 % EI, +12.3 % mass ✓
- Q499: b=200, t=1, d=10 mm → 1.21 cm⁴ ✓
- Q507: I=11320, d=22 → D=26.1 ✓
- Q749: F=500, L=200, b=50, δ=3; Al E=70 GPa, 2.7 g/cm³, 8 kgCO2/kg → 0.986 ✓ (the CFRP option gives 1.021)

Gotchas:
- Yield strength "from the rules" = 305 MPa.
- Q507: the wall-thickness info is redundant.

## 9. spring_suspension_vibration (dynamics), 8 questions
Equations:
- Wheel rate (motion ratio as wheel/spring travel, the official convention): `Eq(kw, ks/MR**2)`.
- Series tyre: `Eq(kr, kw*kt/(kw+kt))`.
- Critical damping: `Eq(c_crit, 2*sqrt(k*m))`.
- 2-DOF eigen: `det(K - w**2*M) = 0` with `K=[[k1,-k1],[-k1,k1+k2]]`, `M=diag(m1,m2)`, `f=w/(2*pi)`.
- Coil spring rate: `Eq(k, G*d**4/(8*D**3*n))`.
- Spring surge (fixed–fixed): `Eq(f, Rational(1,2)*sqrt(k/m_s))` with `m_s = rho*pi**2*d**2*D*n/4`.
- Coulomb-damped oscillator: amplitude drops 2F/k per half-cycle, stop when |u| ≤ F/k; E_lost = ½k(u0² − u_end²).

Questions: Q86, Q93, Q128, Q408, Q649, Q747, Q826, Q981.

Regression tests:
- Q86: m1=800, m2=60, k1=17000, k2=180000 N/m → 0.70 Hz ✓
- Q649: G=80 GPa, d=8, D=50+8=58 mm, k=42 N/mm → n=5.0 ✓
- Q826: CLA=5.7, 108 km/h, rho=1.1, 45 % front, ks=55, MR=1.2, kt=95 → deflection 23.3 mm; ride height 30−23.3 = 6.7 mm ✓
- Q981: k = 500 N/yd = 546.8 N/m, F = 0.4·98.1 → 273.40 J ✓

Gotchas:
- Q747: the tyre is ignored. Front% = kwf/(kwf+kwr) = 25/57 = 43.9 %.
- Q93 uses the whole mass: c = 9.8 Ns/mm.
- Q128 (168 Hz) works with n=12, rho=7850 (167.2 Hz). Q408 has the same data but an official answer of 154 Hz. That only fits about 14 coils (12 active + 2 dead). ⚠
- Q649: inside diameter + d = mean diameter.

## 10. engine_geometry (powertrain), 9 questions
Equations:
- `Eq(Vd_cyl, pi/4*B**2*S)`, `Eq(CR, (Vd+Vc)/Vc)`.
- Deck/gasket change: `Eq(dh, (Vc1-Vc2)/(pi/4*B**2))`.
- Mean piston speed: `Eq(cm, 2*S*n/60)`.
- Brake power, 4-stroke: `Eq(P, bmep*Vd_tot*n/120)`.
- Slider–crank: `Eq(x, r*(1-cos(th)) + l*(1-sqrt(1-(r/l*sin(th))**2)))`.
- Engine order: `Eq(f, order*rpm/60)` (I4 is order 2). Firing frequency of an I4 4-stroke = rpm/30.

Questions: Q465, Q621, Q661, Q775, Q777, Q806, Q995, Q1087, Q1039.

Regression tests:
- Q806: B=67, S=42.5, CR 12→13.5, gasket −0.1 → 0.364 mm ✓
- Q995: B=80, S=68.6, CR 11→13.5 → 1.272 mm ✓
- Q775: 65 kW, 8400 rpm, cm=18, bmep=12 atm, 3 cyl → 71.0 mm ✓
- Q621: r=26, l=66, x=2 mm → 19.2° ✓
- Q1087: A = 31728/80e5, Vd = A·0.15, CR = 649/(649−594.9) = 12.0 ✓

Gotchas:
- Q465: n·cyl^(1/3) scaling gives 50·4^(1/3) = 79.37 hp.
- Q621: the stated displacement is inconsistent and should be ignored.
- Q1039: semitone ratios are 2^(k/12); the options imply intervals 0,0,1,0,8,2.

## 11. wheel_velocity_slip_angle / steering geometry (dynamics), 7 questions
Equations:
- Rigid-body wheel velocity: `vx_w = vx - r*y_w`, `vy_w = vy + r*x_w` (r = yaw rate, x forward, y left).
- Slip angle: `Eq(alpha, delta_w - atan(vy_w/vx_w))` (toe adds/subtracts per side).
- Yaw rate: `Eq(r, v/R)`.
- Ackermann: `Eq(cot(d_o) - cot(d_i), t/L)`.
- Neutral steer with equal cornering stiffness: CG at L/2.

Questions: Q92, Q377, Q382, Q470, Q750, Q1001, Q1004.

Regression tests:
- Q92: r=1.3, vx=42, vy=−3 km/h, L=1.53, T=1.2, δ=10° → front-left α = 9.15° ✓
- Q377: 18.29° ✓; Q382: 22.43° (no official answer)
- Q750: toe = −0.5° (official solution given); Q1001 (L=1.525) → −0.46 ✓
- Q1004: car 230 kg, 51 % rear, L=1.525, driver at 0.71 m → driver 66.81 kg, lose 8.19 kg ✓

Gotchas:
- Q1004: lap time and speed are red herrings.
- Q470: single-track parameters are in an image, so it is not computable.

## 12. fasteners_shear_bearing (structures), 8 questions
Equations:
- Tension: `Eq(A_core, F/(n*sigma_allow))`, `d = sqrt(4*A/pi)`.
- Shear: `Eq(F, n*tau*pi*d**2/4)`.
- Bearing: `Eq(p, F/(d*t))`.
- Torque → couple: `F = T*SF/d_mean`.
- Edge distance: `e = 1.5*D`.

Questions: Q46, Q114, Q139, Q158, Q226, Q389, Q410, Q537.

Regression tests:
- Q158: 80 kN, 4 bolts, 32 MPa → 28.2 mm → M36 ✓
- Q139: T=20·1.5, mean diameter 22.5, t=2.5, p=60 → d = 8.9 → 10 mm ✓ (using the outer diameter gives the 8 mm distractor)
- Q226: D = 10.1 → 15.15 ✓; Q389: D = 12.1 → 18.15

Gotchas:
- Q46: the official answer is the ratio 640/900 = 71.1 %, not the difference (40.6 %).
- Q410: head bearing π/4(10²−6.5²)·240 = 10885 N is a distractor. The official is 9531 N. ⚠
- Q114: 4·150·π·3² = 16965 N, but the method for 35 g is unclear (implies 49.4 kg). ⚠
- Q537 lacks F, t, h (figure).

## 13. work_energy (dynamics), 6 questions
Equations:
- `Eq(E, Rational(1,2)*m*(v2**2-v1**2))`.
- Rolling wheel: `E = Rational(1,2)*(m + I/r**2)*v**2`, minus F_rr·s.
- Constant power (official convention, Q381/385): `m = P*t**3/(2*s**2)` from v_end = 2s/t.
  The exact physics is `m = 8*P*t**3/(9*s**2)`.
- Spun cylinder on a surface: `v_f = w0*R/3`, `a = mu*g` until rolling.

Questions: Q76, Q360, Q381, Q385, Q1064, Q1071 (A).

Regression tests:
- Q76: sqrt(100² − 30²) = 95.4 km/h. For Fred, the lost 0.75 kg takes its KE at 20 km/h → 59.2 ✓
- Q360: 72 km/h, r=0.20574, I=0.093, m=5, 10 N·40 m → 17 m/s ✓
- Q381: 20 kW, 75 m, 4.0 vs 4.1 s → 8.7 kg (official convention). The exact answer, 15.6, is also an option. Trap.
- Q1064: 13.04 s ✓

Gotchas:
- Q385 has no official answer. Use the same convention as Q381 → 17.0 (the exact method gives 30.2, also an option).

## 14. brake_torque_clamp (powertrain), 3 questions
Equations:
- Disc torque: `Eq(T, mu_pad*p*A_piston*n_pistons*R_m)` (opposed caliper, both faces counted via the total piston count).
- Wheel: `Eq(F_x*r_dyn, T)`.
- Rear lock (Q918): `F_r = mu_tyre*m*g*(wr - ax*h/L)` per axle × SF.

Questions: Q451, Q918, Q1071 (B).

Regression tests:
- Q918 → D = 35.8 → 36.0 mm ✓
- Q1071 B: 55e3·0.38·0.175 = 3657.5 Nm ✓ (single μ·F·r, no ×2)

Gotchas:
- Q451: the official 5.0 bar implies a ≈ 6.5 m/s². With a = 8 m/s² the result is 6.19 bar. The rule constant behind IN 11.2 needs checking. ⚠

## 15. thermodynamic_cycles_heat_transfer (thermo), 7 questions
Equations:
- COP_cool = `(Q_h - W)/W`.
- Carnot: `eta = 1 - Tc/Th`.
- Joule/Brayton refrigerator: `Q_h/Q_c = (p_h/p_l)**((k-1)/k)`. The manometer values are gauge, so add p_atm.
- Newton cooling: `Eq(T, Ta + (T0-Ta)*exp(-k*t))` with `k = h*A/(m*c)`.
- LMTD: `Eq(Q, U*A*(dT1-dT2)/log(dT1/dT2))`.

Questions: Q10, Q503, Q683, Q689, Q745, Q860, Q927.

Regression tests:
- Q10: 850/150 = 5.6667 ✓
- Q860: 0.5/2.5 bar gauge + 0.99992 → −12.74 / −2.74 ✓
- Q927: A excludes the 30×20 base (A = 0.21 m²) → 1232.3 s ✓
- Q689: 0.2897 ✓
- Q503: Carnot gives 68.06 % → 68 % (no official answer)

Gotchas:
- Q683 needs figure data, so it is not computable.

## 16. acoustics (powertrain), 4 questions
Equations:
- Closed pipe, first overtone (3rd harmonic): `Eq(f, 3*c/(4*L))`.
- Incoherent dB sum: `Eq(Lt, 10*log(sum(10**(Li/10)), 10))`, after removing the IEC A/C weighting.

Questions: Q380, Q388, Q677, Q869.

Regression tests:
- Q677: 880 Hz → 0.2923 m ✓
- Q869: the official answer is 2.2077. My result is 2.2073; rounding each pipe length to 4 decimals gives 2.2074.
- Q380/388: C(30 Hz) = −3.3, A(2 kHz) = +1.2 → 106 dB (no official answer)

## 17. traction_power_limited_speed (powertrain), 3 questions
Equations:
- Traction-limited torque: `Eq(n_w*T/r, mu*(m*g + Rational(1,2)*rho*ClA*v**2))`.
- Power corner: `Eq(v, P*r/T)`.
- Top speed: `min(v_rpm, v_power)` with `v_rpm = n_max/i*2*pi/60*r` and `eta*P = (Rational(1,2)*rho*CdA + k_f)*v**3`.
- Battery recuperation: `V = OCV + I*R`, then P through the efficiencies, then wheel torque.

Questions: Q75, Q757, Q811.

Regression tests:
- Q75: 63.4, 110.8 ✓
- Q811: 95.32 km/h ✓
- Q757: 390 Nm ✓ (solution given)

## Multi-step chains
| Q | Chain |
|---|---|
| Q65 | aero force (3) → axle-load moments (1) |
| Q180 | cornering with downforce (4) → aero force (3) → lateral transfer (1) |
| Q826 | aero force (3) → front split (1) → wheel rate + tyre in series (9) → 30 mm rule |
| Q883 | aero force (3) → axle split (1) → springs (9) → 20 mm lowering. Then y+ ∝ 1/U and the venturi continuity needs the drawing. Official x = 0.4000 |
| Q918 | load transfer (1) → μ·Fz·SF → brake torque (14) |
| Q451 | kinematics/rule decel (5) → brake torque (14) |
| Q1071 | energy (13) → torque (14) → ΔT (6) |
| Q75, Q811, Q757 | aero (3) + power/torque (17) |
| Q1064 | rolling dynamics (13) → kinematics (5) |
| Q76 | energy (13), then a mass-drop step |
| Q974 | cornering (4) + trivia lookup |
| Q1004 | load fraction (1) + neutral steer (11) |
| Q749 | beam deflection (8) → mass → CO2 |
| Q862 | thermal expansion (6) + linkage geometry (figure) |
| Q566 | Euler buckling `F=pi**2*E*I/(K*l)**2` + member force from figure angles → min wall. Official 0.1 mm |

## One-offs (computable)
- Q29: steering torque. Model: μ·N·w/6 per wheel, both front wheels, ÷e × d/2 → 11.0 Nm. Official 8.5. ⚠
- Q40: −1340 + 356 = −984.
- Q64: tanθ = 1.41 → 54.7° (complement 35.3°; both accepted).
- Q657: optimal support spacing d = (2−√2)·L, with L = 1000 → 585.8.
- Q667: (400000 + 5·20000 + 1000·50)/1000 = 550.
- Q693: 350000 + 130n = 425n + 5000 → 1170.
- Q949: reliability product, removed. Official 88.69.
- Q970: Kepler fall time = T_orbit/(4√2) ≈ 65 d.
- Q983/984: buoyancy. ρ = 0.86·ρ_coolant (water ⇒ 0.86 kg/dm³); then fully submerged in oil → 0.
- Q616: camera pinhole back-projection + ZYX rotation + translation → 2.507, 1.590, 0.037.
- Q626: tolerance stack-up (drawing).
- Q1058: section properties (figure).
- Q1024: slalom arcs (geometry + cornering 4).
- Q988 and Q1066: CAD/FEA.

## Not computable from text (rules lookup, figure or trivia)
- Rules: Q42, Q95, Q122/409, Q160 (3·OD = 90), Q335, Q359, Q379/390, Q395, Q397, Q411, Q514, Q544, Q569, Q580, Q632, Q655, Q668, Q696, Q713, Q728, Q937, Q1042.
- Figure or data missing: Q81, Q157, Q470, Q485, Q495, Q504, Q537, Q598, Q631, Q683, Q736, Q900, Q903, Q925/1043, Q941, Q958, Q963 (F_E = F·b/c = 0.5, F_G = F + F_E = 1.5 kN; computable with the moment about G), Q1037, Q1059.
- Strategy: Q961.
