"""Procedural helpers: rules scoring, rule tables, circuits, digital, rounding.

Rule constants live in data/rules/<year>.toml and data/rules/penalties.toml. "legacy" reproduces most FS-Quiz
keys (FS Rules <= 2025), "2026" is FS-Rules 2026 v1.1, "2027" is FS-Rules 2027 v1.0 (the default).
"""

from __future__ import annotations

import math
from fractions import Fraction

import sympy as sp

from .core import tool
from .data import data_dir, load

# ---------------------------------------------------------------- scoring

RULES = {p.stem: load(f"rules/{p.name}") for p in (data_dir() / "rules").glob("*.toml") if p.stem != "penalties"}
PEN = load("rules/penalties.toml")
CURRENT = max(y for y in RULES if y.isdigit())


def _pmax(event: str, rules: str) -> float:
    r = RULES[rules]
    return r["points"][r["tmax_pmin"][event].get("points", event)]


def _score_expr(event: str, t, t_min: float, rules: str, pmax: float | None, finish: bool = True):
    r = RULES[rules]
    if r["dynamic_formula"] == "legacy":
        c = r["legacy_scoring"][event]
        t_max = c["tmax_factor"] * t_min
        floor = c["floor"] if finish or event != "endurance" else 0
        return c["scale"] * ((t_max / t) ** c["exponent"] - 1) / c["divisor"] + floor, t_max
    c = r["tmax_pmin"][event]
    p = pmax or _pmax(event, rules)
    t_max = c["tmax"] * t_min
    return (p - c["pmin"] * p) * ((t_max - t) / (t_max - t_min)) ** 2 + c["pmin"] * p, t_max


@tool
def corrected_time(
    t_raw: float,
    doo: float = 0,
    oc: float = 0,
    event: str = "autocross",
    flags: float = 0,
    out_of_order: float = 0,
    post: str = "",
) -> float:
    """Raw time + DOO/OC (per event) + 60 s per disobeyed flag + 120 s out of order + post-inspection group A/B."""
    pen = PEN["doo_oc"][event]
    if oc and "oc" not in pen:
        raise ValueError(f"an OC in {event} is a DQ")
    return (
        t_raw
        + doo * pen["doo"]
        + oc * pen.get("oc", 0)
        + PEN["flag"] * flags
        + PEN["out_of_order"] * out_of_order
        + (PEN["post_inspection"][post][event] if post else 0)
    )


@tool
def event_score(
    event: str,
    t_team: float,
    t_min: float,
    rules: str = CURRENT,
    pmax: float = 0,
    finish: float = 1,
) -> float:
    """Manual dynamic event score (2026/2027 D 9.1.1). event: skidpad accel autocross endurance, dc_skidpad dc_accel.
    Times already corrected (use corrected_time). rules=legacy reproduces older quiz keys
    (endurance finish=0 drops the +25, as the Q171 key does)."""
    _, t_max = _score_expr(event, 1.0, t_min, rules, pmax or None)
    score, _ = _score_expr(event, min(t_team, t_max), t_min, rules, pmax or None, bool(finish))
    if rules == "legacy":
        return score
    return min(max(score, 0.0), pmax or _pmax(event, rules))


@tool
def cones_from_score(event: str, score: float, t_raw: float, t_min: float, rules: str = CURRENT) -> str:
    """Inverse: which corrected time (and how many DOO) gives this score."""
    t = sp.Symbol("t", positive=True)
    expr, t_max = _score_expr(event, t, t_min, rules, None)
    sols = [float(x) for x in sp.solve(expr - score, t) if x.is_real and 0 < float(x) <= t_max * (1 + 1e-9)]
    d = PEN["doo_oc"][event]["doo"]
    return "; ".join(f"T_team={x:.4g} s -> {(x - t_raw) / d:.3g} DOO" for x in sols)


@tool
def dv_rank_score(rank: float, n_all: float, pmax: float = 75) -> float:
    """2026/2027 D 9.2.2 autonomous-mode skidpad/accel: Pmax (N_all + 1 - R) / N_all. Runs > 25 s raw are DQ."""
    return pmax * (n_all + 1 - rank) / n_all


@tool
def dc_autocross_score(t1: float, t2: float, t_min: float, t_max: float, pmax: float = 100) -> float:
    """2026/2027 D 9.3.2. t_max = lap length / 6 m/s. DNF/DQ runs: pass t_max."""
    tt = min(min(t1, t_max), (min(t1, t_max) + min(t2, t_max)) / 2)
    return 0.9 * pmax * (t_max - tt) / (t_max - t_min) + 0.1 * pmax


@tool
def trackdrive_score(t_team: float, t_fastest: float, laps: float = 10, pmax: float = 200) -> float:
    """2026/2027 D 9.3.3 + D 9.3.4: 0.75 Pmax (Tmax/Tteam - 1) with Tmax = 2 T_fastest, + 2.5 % Pmax per completed lap."""
    t_max = 2 * t_fastest
    return 0.75 * pmax * (t_max / min(t_team, t_max) - 1) + 0.025 * pmax * laps


@tool
def efficiency_score(
    t: float,
    e: float,
    ef_min: float,
    rules: str = CURRENT,
    pmax: float = 0,
    t_min: float = 0,
    e_min: float = 0,
    ef_max: float = 0,
) -> float:
    """2026/2027 D 9.4: EF = T^2 E (T uncorrected, E energy), score = Pmax ((EFmax - EF)/(EFmax - EFmin))^2,
    EFmax = 2 EFmin, EF clamped to EFmax (explicit in 2027).
    legacy: EF = (Tmin/T)(Emin/E), score = 100 (EF - EFmin)/(EFmax - EFmin) (pass t_min, e_min, ef_min, ef_max)."""
    p = pmax or RULES[rules]["points"]["efficiency"]
    if rules == "legacy":
        ef = (t_min / t) * (e_min / e)
        return p * (ef - ef_min) / (ef_max - ef_min)
    ef = t**2 * e
    ef_mx = 2 * ef_min
    return p * (max(0.0, ef_mx - ef) / (ef_mx - ef_min)) ** 2


@tool
def static_nonfinalist(
    p_team: float,
    p_best: float,
    event: str = "bpp",
    rules: str = CURRENT,
    n_finalists: float = 0,
) -> float:
    """Non-finalist static score scaled to the best NON-finalist. 2027: BPP 65 (S 2.4.6), cost 80 cap (S 3.7.5).
    2026: BPP 70, cost 80. legacy: BPP (75 - n_finalists), cost 95."""
    c = RULES[rules]["static_nonfinalist"]
    k = c[event] - (n_finalists if event == "bpp" and c.get("bpp_minus_finalists") else 0)
    return k * p_team / p_best


@tool
def max_points(events: str, rules: str = CURRENT) -> float:
    """Sum of maximum points, e.g. events=bpp,cost,skidpad,accel,efficiency."""
    return sum(RULES[rules]["points"][e] for e in events.split(","))


# ---------------------------------------------------------------- electrical rule tables


@tool
def pcb_spacing(v: float, mode: str = "creepage", rules: str = CURRENT) -> float:
    """TS-LV PCB spacing [mm]. 2026/2027 modes: clearance | creepage | coated. legacy: surface | air | coated."""
    t = RULES[rules]["pcb_spacing"]
    col = t["modes"].index(mode) + 1
    for row in t["rows"]:
        if v <= row[0]:
            return row[col]
    raise ValueError("above 600 V: TS max is 600 V (EV 4.1.1)")


@tool
def ts_rules(v_max: float, v_nom: float = 0, rules: str = CURRENT) -> str:
    """Voltage-dependent TS rule values for a rule year."""
    ts = RULES[rules]["ts"]
    test = next(tv for lim, tv in ts["insulation_tiers"] if v_max <= lim)
    nominal = ts["tsal_uses"] == "nominal"
    tsal_v = v_nom or v_max
    note = " (pass v_nom: this rule year uses NOMINAL voltage)" if nominal and not v_nom else ""
    return (
        f"insulation test {test:g} V, pass >= {500 * v_max / 1e3:g} kOhm (500 Ohm/V); "
        f"TSAL red threshold min(60, V/2) = {min(60, tsal_v / 2):g} V{note}; "
        "section <= 120 V, <= 6 MJ (max cell V x nominal Ah), <= 12 kg; TS <= 600 V; discharge <= 60 V in 5 s"
    )


@tool
def weight_penalty(delta_kg: float) -> float:
    """IN 12.1.7: 20 points per started kg beyond +-5 kg vs the technical inspection weight (6.2 kg -> 40)."""
    over = max(0.0, abs(delta_kg) - PEN["weight_tolerance_kg"])
    return PEN["weight_per_kg"] * math.ceil(over - 1e-9)


@tool
def segment_max_cells(v_cell_max: float, e_cell: float, v_lim: float = 120, e_lim: float = 6e6) -> int:
    """Max cells per section (2027 wording; 'segment' before): min(floor(V_lim/V_cell), floor(E_lim/E_cell)).
    E_cell = MAX cell voltage x nominal capacity (EV 5.1.2), in J: type 4.2V*20Ah. Mass limit 12 kg too."""
    return min(math.floor(v_lim / v_cell_max + 1e-9), math.floor(e_lim / e_cell + 1e-9))


# ---------------------------------------------------------------- electronics helpers

E_SERIES = {
    6: [1.0, 1.5, 2.2, 3.3, 4.7, 6.8],
    12: [1.0, 1.2, 1.5, 1.8, 2.2, 2.7, 3.3, 3.9, 4.7, 5.6, 6.8, 8.2],
    24: [
        1.0,
        1.1,
        1.2,
        1.3,
        1.5,
        1.6,
        1.8,
        2.0,
        2.2,
        2.4,
        2.7,
        3.0,
        3.3,
        3.6,
        3.9,
        4.3,
        4.7,
        5.1,
        5.6,
        6.2,
        6.8,
        7.5,
        8.2,
        9.1,
    ],
}


@tool
def e_series(value: float, series: float = 12) -> str:
    """Nearest standard values (below, nearest, above) in E6/E12/E24 (E96 computed)."""
    if series in E_SERIES:
        base = E_SERIES[int(series)]
    else:
        base = [round(10 ** (i / series), 2) for i in range(int(series))]
    dec = 10 ** math.floor(math.log10(value))
    cands = sorted({b * dec * k for b in base for k in (0.1, 1, 10)})
    below = max(c for c in cands if c <= value * (1 + 1e-9))
    above = min(c for c in cands if c >= value * (1 - 1e-9))
    near = min(cands, key=lambda c: abs(math.log(c / value)))
    return f"below {below:.4g}  nearest {near:.4g}  above {above:.4g}"


@tool
def gamma(z_load_re: float, z_load_im: float = 0, z0: float = 50) -> str:
    """Reflection coefficient and VSWR of a complex load."""
    zl = complex(z_load_re, z_load_im)
    g = (zl - z0) / (zl + z0)
    m = abs(g)
    return f"|Gamma| {m:.5g}  VSWR {(1 + m) / (1 - m):.5g}  transmitted {1 - m * m:.4%}"


@tool
def alias(f: float, fs: float) -> float:
    """Apparent frequency after sampling f at fs."""
    return abs(f - round(f / fs) * fs)


@tool
def twos(x: float, bits: float) -> str:
    """Two's complement representation."""
    n = int(bits)
    return format((1 << n) + int(x) if x < 0 else int(x), f"0{n}b")


@tool
def uart_time(n_bytes: float, baud: float, data: float = 8, parity: float = 0, stop: float = 1) -> float:
    """Seconds to send n bytes (start + data + parity + stop bits)."""
    return n_bytes * (1 + data + parity + stop) / baud


@tool
def can_frame_bits(data_bytes: float = 8, extended: float = 0, ifs: float = 3, stuffing: float = 0) -> int:
    """Bits per classic CAN frame (incl. IFS). stuffing=1 adds worst-case stuff bits."""
    base = (64 if extended else 44) + 8 * int(data_bytes)
    stuffed = base - 13  # fields subject to stuffing exclude CRC delimiter, ACK, EOF
    extra = math.floor((stuffed - 1) / 4) if stuffing else 0
    return base + extra + int(ifs)


@tool
def db_sum(levels: str) -> float:
    """Incoherent sum of sound levels: db_sum 90,92,85."""
    return 10 * math.log10(sum(10 ** (float(x) / 10) for x in levels.split(",")))


# ---------------------------------------------------------------- circuits


@tool
def nodal(netlist: str, ask: str = "") -> str:
    """Exact DC nodal analysis. Elements separated by ';':
      R<name> a b ohms | V<name> plus minus volts | I<name> from to amps (current flows from -> to inside source)
    Node 0 is ground. ask=Rab:a,b gives the equivalent resistance between a and b (sources ignored).
    Example: nodal "R1 1 2 3; R2 2 0 3; R3 1 0 3; V1 1 0 10"."""
    elems = [e.split() for e in netlist.replace("\n", ";").split(";") if e.strip()]
    if ask.startswith("Rab:"):
        a, b = ask[4:].split(",")
        elems = [e for e in elems if e[0][0].upper() == "R"] + [["Itest", b, a, "1"]]
    nodes = sorted({n for e in elems for n in e[1:3]} - {"0"})
    vsrc = [e for e in elems if e[0][0].upper() == "V"]
    sym = {n: sp.Symbol(f"v{n}") for n in nodes} | {"0": sp.Integer(0)}
    ivs = {e[0]: sp.Symbol(f"i_{e[0]}") for e in vsrc}
    kcl = {n: sp.Integer(0) for n in nodes}  # sum of currents leaving the node
    for e in elems:
        kind, a, b = e[0][0].upper(), e[1], e[2]
        val = sp.Rational(
            Fraction(e[3]).limit_denominator(10**9).numerator,
            Fraction(e[3]).limit_denominator(10**9).denominator,
        )
        if kind == "R":
            i = (sym[a] - sym[b]) / val
        elif kind == "I":
            i = val
        else:
            i = ivs[e[0]]
        if a != "0":
            kcl[a] += i
        if b != "0":
            kcl[b] -= i
    eqs = list(kcl.values()) + [sym[e[1]] - sym[e[2]] - sp.nsimplify(e[3]) for e in vsrc]
    unknowns = [sym[n] for n in nodes] + list(ivs.values())
    sol = sp.solve(eqs, unknowns, dict=True)[0]
    if ask.startswith("Rab:"):
        a, b = ask[4:].split(",")
        r = (sym[a] - sym[b]).subs(sol)
        return f"R_{a}{b} = {r} = {float(r):.6g} ohm"
    out = [f"V({n}) = {sol[sym[n]]} = {float(sol[sym[n]]):.6g} V" for n in nodes]
    for e in elems:
        if e[0][0].upper() == "R":
            i = ((sym[e[1]] - sym[e[2]]) / sp.nsimplify(e[3])).subs(sol)
            out.append(f"I({e[0]}, {e[1]}->{e[2]}) = {i} = {float(i):.6g} A")
    for name, s in ivs.items():
        out.append(f"I({name}, into + terminal) = {float(sol[s]):.6g} A")
    return "\n  ".join(out)


# ---------------------------------------------------------------- mechanics helpers


@tool
def two_dof(m1: float, m2: float, k1: float, k2: float) -> str:
    """Quarter car: sprung m1 on k1 (spring) over unsprung m2 on k2 (tyre). Natural frequencies [Hz]."""
    w2 = sp.Symbol("w2")
    det = (k1 - w2 * m1) * (k1 + k2 - w2 * m2) - k1**2
    fs = sorted(math.sqrt(float(r)) / (2 * math.pi) for r in sp.solve(det, w2))
    return "  ".join(f"{f:.4g} Hz" for f in fs)


@tool
def gauge(p: float, to: str = "abs", p_atm: float = 101325) -> float:
    """Convert pressure between gauge and absolute (Pa in, Pa out; type 2bar)."""
    return p + p_atm if to == "abs" else p - p_atm


@tool
def gear_train(n_in: float, teeth: str) -> float:
    """Output speed of a gear train: teeth = driver/driven,driver/driven (e.g. 20/45,24/55). Same unit as n_in."""
    r = 1.0
    for pair in teeth.split(","):
        a, b = pair.split("/")
        r *= float(a) / float(b)
    return n_in * r


@tool
def accel_then_cruise(s: float, a: float, v_max: float) -> float:
    """Time to cover s from rest accelerating at a up to v_max, then constant v_max."""
    s_acc = v_max**2 / (2 * a)
    if s <= s_acc:
        return math.sqrt(2 * s / a)
    return v_max / a + (s - s_acc) / v_max


@tool
def round_to(x: float, step: float = 1, mode: str = "nearest") -> float:
    """Round to a step: mode nearest | down | up (fuses/limits round down, 'at least' rounds up)."""
    f = {"nearest": round, "down": math.floor, "up": math.ceil}[mode]
    return f(x / step) * step
