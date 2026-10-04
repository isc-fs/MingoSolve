"""Golden corpus for the Rust port: what the Python engine returns for each formula and each hidden unknown.

For every formula: sample a consistent point (random inputs, solve the rest), then hide each variable (and each
pair when the formula has two or more equations) and record Python's roots. Seeded, long sympy timeouts. Output:
tests/golden/solve_corpus.json at the repo root. Cases where Python misses the truth are kept and flagged.
"""

import json
import random
import sys
import time
from itertools import combinations
from pathlib import Path

from fsq_solver import core, formulas  # noqa: F401
from fsq_solver.core import FORMULAS, VARS, solve_step

OUT = Path(__file__).resolve().parents[3] / "tests" / "golden" / "solve_corpus.json"
POINTS_PER_FORMULA = 3
TYPICAL = {  # SI magnitude for a variable's unit; anything else ~1
    "Pa": 1e5, "F": 1e-4, "H": 1e-5, "m**4": 1e-8, "m**2": 1e-3, "m**3": 1e-4, "1/K": 1e-3, "ohm*m": 1e-7,
    "N/m": 3e4, "J": 1e4, "W": 1e3, "Hz": 1e2, "kg*m**2": 1e-1, "K": 300, "C": 3e4, "N": 1e3, "rad/s": 300,
    "A/s": 1e5, "N*s/m": 2e3, "J/(kg*K)": 1e3, "J/kg": 3e5, "Pa*s": 1e-3, "kg/m**3": 1e3,
}
SPECIFIC = {"E_y": 2e11, "G_sh": 8e10, "sigma_y": 3e8, "bmep": 1e6, "alpha": 4e-3, "alpha_th": 1e-5, "mu_r": 1e3,
            "n_bits": 12, "N_s": 100, "eta": 0.8, "eta_g": 0.9, "eta_tx": 0.5, "Gm": 0.3, "frac_in": 0.3, "MR": 1.2,
            "D": 0.4, "H": 0.7, "eps": 1e-3, "kappa": 1.4, "k_p": 1.2, "CR": 12, "k_ord": 2, "n_h": 2}


def sample(name: str, rng: random.Random) -> float:
    v = VARS[name]
    if v.default is not None:
        return v.default
    base = SPECIFIC.get(name, TYPICAL.get(v.unit, 1.0))
    return base * 10 ** rng.uniform(-0.5, 0.5)


def truth_point(f, rng: random.Random) -> dict[str, float] | None:
    names = f.names
    k = len(f.eqs)
    for _ in range(40):
        solve_for = set(rng.sample(names, k))
        known = {n: sample(n, rng) for n in names if n not in solve_for}
        got = solve_step(f, known)
        if not solve_for <= got.keys():
            continue
        point = known | {n: vals[0] for n, vals in got.items()}
        if not core.conflicts_in(f, point):
            return point
    return None


def main() -> None:
    core.TIMEOUT_SCALE = float(__import__('os').environ.get('FSQ_TIMEOUT_SCALE', '3'))
    rng = random.Random(20261004)
    cases, misses = [], 0
    only = set(sys.argv[1:])
    for key, f in FORMULAS.items():
        if only and key not in only:
            continue
        k = len(f.eqs)
        for _ in range(POINTS_PER_FORMULA):
            point = truth_point(f, rng)
            if point is None:
                print(f"  {key}: no consistent point found", file=sys.stderr)
                break
            for h in range(1, min(k, 2) + 1):
                for hidden in combinations(f.names, h):
                    given = {n: x for n, x in point.items() if n not in hidden}
                    t0 = time.perf_counter()
                    got = solve_step(f, given)
                    dt = time.perf_counter() - t0
                    if dt > 2:
                        print(f"    slow {key} hide {hidden}: {dt:.1f} s", file=sys.stderr)
                    found = all(
                        n in got and any(abs(r - point[n]) <= 1e-6 * max(1.0, abs(point[n])) for r in got[n])
                        for n in hidden
                    )
                    misses += not found
                    cases.append({"formula": key, "given": given, "hidden": list(hidden),
                                  "truth": {n: point[n] for n in hidden},
                                  "python": {n: v for n, v in got.items()}, "python_finds_truth": found})
        print(f"{key}: {sum(c['formula'] == key for c in cases)} cases", file=sys.stderr)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps({"seed": 20261004, "cases": cases}, indent=1))
    print(f"{len(cases)} cases, Python misses the truth in {misses}; wrote {OUT}")


if __name__ == "__main__":
    main()
