"""Export the Python-defined formulas, examples and units to the shared TOML data in ../../data.

One-off for the formulas and examples (the TOML becomes the source of truth); rerun for units.toml whenever the
unit whitelist changes.
"""

import json
from importlib import import_module
from pathlib import Path
from pkgutil import iter_modules

import pint

ROOT = Path(__file__).resolve().parents[3]
DATA = ROOT / "data"


def s(x) -> str:
    return json.dumps(x, ensure_ascii=False)


def export_formulas() -> None:
    from fsq_solver import core

    pkg = import_module("fsq_solver.formulas._common").__package__
    mods = ["_common"] + [m.name for m in iter_modules(import_module(pkg).__path__) if m.name != "_common"]
    seen_v, seen_f = set(), set()
    import_module(f"{pkg}._common")
    for i, name in enumerate(mods):
        import_module(f"{pkg}.{name}")
        new_v = [n for n in core.VARS if n not in seen_v]
        new_f = [k for k in core.FORMULAS if k not in seen_f]
        seen_v.update(new_v)
        seen_f.update(new_f)
        out = [f"# {name.strip('_')}: generated from the Python definitions, now the source of truth\n"]
        for n in new_v:
            v = core.VARS[n]
            line = f"[[var]]\nname = {s(n)}\nunit = {s(v.unit)}\ndesc = {s(v.desc)}\n"
            if not v.positive:
                line += "signed = true\n"
            if v.default is not None:
                line += f"default = {v.default!r}\n"
            out.append(line)
        for k in new_f:
            f = core.FORMULAS[k]
            line = f"[[formula]]\nkey = {s(k)}\ntitle = {s(f.title)}\neqs = [\n"
            line += "".join(f"    {s(e)},\n" for e in f.src) + "]\n"
            line += f"tags = {s(list(f.tags))}\n"
            if f.notes:
                line += f"notes = {s(f.notes)}\n"
            out.append(line)
        (DATA / "formulas" / f"{i:02d}_{name.strip('_')}.toml").write_text("\n".join(out))


def export_examples() -> None:
    from fsq_solver.examples import EXAMPLES

    out = ["# Past FS-Quiz questions solved with fsq. Each row is a regression test: the official answer must match.\n"]
    for qid, what, cmd, ans in EXAMPLES:
        out.append(f"[[example]]\nid = {qid}\nwhat = {s(what)}\ncmd = {s(cmd)}\nanswer = {ans!r}\n")
    (DATA / "examples.toml").write_text("\n".join(out))


PREFIXES = {"G": 1e9, "M": 1e6, "k": 1e3, "h": 1e2, "c": 1e-2, "m": 1e-3, "u": 1e-6, "µ": 1e-6, "n": 1e-9, "p": 1e-12}
PREFIXABLE = [
    "m",
    "g",
    "s",
    "A",
    "K",
    "N",
    "Pa",
    "J",
    "W",
    "V",
    "ohm",
    "F",
    "H",
    "C",
    "Hz",
    "Wh",
    "Ah",
    "l",
    "L",
    "bar",
    "rad",
    "S",
    "T",
    "Wb",
    "eV",
    "mol",
]
PLAIN = [
    "min",
    "h",
    "hour",
    "day",
    "km/h",
    "kph",
    "mph",
    "in",
    "inch",
    "ft",
    "yd",
    "mi",
    "nmi",
    "lb",
    "oz",
    "t",
    "tonne",
    "atm",
    "psi",
    "mmHg",
    "torr",
    "hp",
    "cal",
    "kcal",
    "rpm",
    "rps",
    "revolution",
    "deg",
    "degree",
    "degC",
    "degF",
    "delta_degC",
    "delta_degF",
    "percent",
    "%",
    "ppm",
    "g0",
    "gal",
    "kn",
    "knot",
    "Gal",
    "Ω",
    "ohm",
]
DIMS = ["[length]", "[mass]", "[time]", "[current]", "[temperature]", "[substance]", "[luminosity]"]


def export_units() -> None:
    from fsq_solver.core import ureg

    out = [
        "# Unit table generated from pint by legacy/python/scripts/export_data.py. Do not edit by hand.",
        "# dims = exponents of [m kg s A K mol cd]; value_si = factor * x + offset\n",
    ]
    out.append("prefixable = " + s(PREFIXABLE) + "\n")
    out.append("[prefixes]\n" + "".join(f"{s(p)} = {v!r}\n" for p, v in PREFIXES.items()))
    names = PREFIXABLE + [u for u in PLAIN if u not in PREFIXABLE]
    skipped = []
    for u in names:
        try:
            q = ureg.Quantity(1, u if u != "%" else "percent")
        except pint.errors.UndefinedUnitError:
            skipped.append(u)
            continue
        base = None if u in ("degC", "degF") else q.to_base_units()
        dims = [int(q.dimensionality.get(d, 0)) for d in DIMS]
        if u in ("degC", "degF"):
            k0 = ureg.Quantity(0, u).to("K").magnitude
            k1 = ureg.Quantity(1, u).to("K").magnitude
            factor, offset = k1 - k0, k0
        else:
            factor, offset = float(base.magnitude), 0.0
        line = f"[[unit]]\nname = {s(u)}\ndims = {dims}\nfactor = {factor!r}\n"
        if offset:
            line += f"offset = {offset!r}\n"
        out.append(line)
    (DATA / "units.toml").write_text("\n".join(out))
    if skipped:
        print("not in pint, skipped:", ", ".join(skipped))


def export_unit_golden() -> None:
    """Strings as typed in the CLI, with pint's SI value, for the Rust units tests."""
    from fsq_solver.core import ureg

    cases = [
        "100 km/h",
        "8000 rpm",
        "1.5 g0",
        "20 Ah",
        "60 degC",
        "2 bar",
        "10 mm**2",
        "0.017 uohm*m",
        "2 A/us",
        "10 ppm/K",
        "42 N/mm",
        "80 GPa",
        "9 pF",
        "1800 uF",
        "1 uH",
        "3.5 kW",
        "4.2 V * 20 Ah",
        "5 %",
        "12 in",
        "300 yd",
        "40000 ft",
        "760 mmHg",
        "1 hp",
        "1 kWh",
        "56 km/h",
        "0.5 kg/m**3",
        "1 cal",
        "1 revolution/s",
        "90 deg",
        "50 Hz",
        "1 MJ",
        "3 kN*m",
        "15 psi",
        "2 mi",
    ]
    out = ["# pint values for unit strings (SI base magnitude). Generated; do not edit.\n"]
    for c in cases:
        q = ureg.Quantity(c)
        out.append(f"[[case]]\ninput = {s(c)}\nsi = {float(q.to_base_units().magnitude)!r}\n")
    (DATA / "units_golden.toml").write_text("\n".join(out))


if __name__ == "__main__":
    import sys

    what = set(sys.argv[1:]) or {"units"}  # formulas/examples were one-off: the TOML is now the source
    if "formulas" in what:
        export_formulas()
    if "examples" in what:
        export_examples()
    if "units" in what:
        export_units()
        export_unit_golden()
    print("exported", ", ".join(sorted(what)), "to", DATA)
