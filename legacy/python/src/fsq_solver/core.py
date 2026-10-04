from __future__ import annotations

import signal
from collections.abc import Callable
from dataclasses import dataclass, field
from itertools import combinations

import pint
import sympy as sp

ureg = pint.UnitRegistry()
ureg.define("g0 = 9.81 m/s**2")
Q = ureg.Quantity


@dataclass(frozen=True)
class Var:
    name: str
    unit: str
    desc: str
    positive: bool = True
    default: float | None = None

    @property
    def sym(self) -> sp.Symbol:
        return sp.Symbol(self.name, real=True)


VARS: dict[str, Var] = {}


def var(name: str, unit: str, desc: str, positive: bool = True, default: float | None = None) -> Var:
    v = Var(name, unit, desc, positive, default)
    old = VARS.get(name)
    if old and old != v:
        raise ValueError(f"variable {name!r} redefined: {old} vs {v}")
    VARS[name] = v
    return v


def vars_(spec: str) -> None:
    """One var per line: `name | unit | description`. `~name`: can be negative. `name=9.81`: default value."""
    for line in spec.strip().splitlines():
        name, unit, desc = (p.strip() for p in line.split("|", 2))
        signed = name.startswith("~")
        name, _, default = name.lstrip("~").partition("=")
        var(
            name,
            unit,
            desc,
            positive=not signed,
            default=float(default) if default else None,
        )


@dataclass
class Formula:
    key: str
    title: str
    eqs: list[sp.Eq]
    tags: tuple[str, ...] = ()
    notes: str = ""
    src: list[str] = field(default_factory=list)

    @property
    def names(self) -> list[str]:
        seen: dict[str, None] = {}
        for e in self.eqs:
            for s in sorted(e.free_symbols, key=lambda s: s.name):
                seen[s.name] = None
        return list(seen)


FORMULAS: dict[str, Formula] = {}


def _symbols() -> dict[str, sp.Symbol]:
    return {n: v.sym for n, v in VARS.items()} | {"pi": sp.pi}


def formula(key: str, title: str, eqs: str, tags: str = "", notes: str = "") -> Formula:
    """eqs: one `lhs = rhs` per line, using names registered with var()/vars_()."""
    if key in FORMULAS:
        raise ValueError(f"duplicate formula {key!r}")
    syms = _symbols()
    parsed, src = [], []
    for line in eqs.strip().splitlines():
        lhs, rhs = line.split("=")
        e = sp.Eq(sp.parse_expr(lhs, local_dict=syms), sp.parse_expr(rhs, local_dict=syms))
        unknown = {s.name for s in e.free_symbols} - VARS.keys()
        if unknown:
            raise ValueError(f"{key}: unregistered variables {unknown}")
        parsed.append(e)
        src.append(line.strip())
    f = Formula(key, title, parsed, tuple(tags.split()), notes, src)
    FORMULAS[key] = f
    return f


def to_si(name: str, value: str | float) -> float:
    v = VARS[name]
    if isinstance(value, (int, float)):
        return float(value)
    q = ureg.Quantity(value.replace(",", "."))
    if q.dimensionless and not ureg.Quantity(1, v.unit).dimensionless:
        return float(q.magnitude)
    return float(q.to(v.unit).magnitude)


def _pick(sols, v: Var) -> list[float]:
    out = []
    for s in sols:
        try:
            c = complex(sp.N(s))
        except (TypeError, ValueError):
            continue
        if abs(c.imag) > 1e-9 * max(1, abs(c.real)):
            continue
        if v.positive and c.real < 0:
            continue
        out.append(c.real)
    out.sort()
    return [x for i, x in enumerate(out) if i == 0 or abs(x - out[i - 1]) > 1e-9 * max(1, abs(x))]


TIMEOUT_SCALE = 1.0  # the golden-corpus generator raises this


class _Timeout(Exception):
    pass


def _limited(fn, seconds: float):
    def handler(*_):
        raise _Timeout

    old = signal.signal(signal.SIGALRM, handler)
    signal.setitimer(signal.ITIMER_REAL, seconds)
    try:
        return fn()
    except _Timeout:
        return None
    except Exception:
        return None
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
        signal.signal(signal.SIGALRM, old)


_GUESSES = [s * 10.0**k for k in range(-9, 10, 1) for s in (1, -1)]


def _solve1(expr: sp.Expr, x: sp.Symbol) -> list[float]:
    v = VARS[x.name]
    num, _ = sp.fraction(sp.together(expr))
    try:
        poly = sp.Poly(sp.expand(num), x)
        if poly.total_degree() > 0 and all(c.is_number for c in poly.all_coeffs()):
            return _pick(poly.nroots(n=15), v)
    except sp.PolynomialError:
        pass
    sols = _limited(lambda: sp.solve(expr, x), 2.0 * TIMEOUT_SCALE)
    vals = _pick(sols or [], v)
    if vals:
        return vals
    roots = []
    for g in _GUESSES:
        if v.positive and g < 0:
            continue
        try:
            roots.append(float(sp.nsolve(expr, x, g, verify=True)))
        except Exception:
            continue
    return _pick(roots, v)


def _solve_system(eqs: list[sp.Expr], xs: list[sp.Symbol]) -> dict[str, list[float]] | None:
    sols = _limited(lambda: sp.solve(eqs, xs, dict=True), 3.0 * TIMEOUT_SCALE)
    good = []
    for sol in sols or []:
        if all(x in sol and _pick([sol[x]], VARS[x.name]) for x in xs):
            good.append({x.name: float(sp.re(sp.N(sol[x]))) for x in xs})
    if not good:
        for g in (1.0, 10.0, 0.1, 100.0, 1000.0):
            try:
                r = sp.nsolve(eqs, xs, [g] * len(xs))
                cand = {x.name: float(r[i]) for i, x in enumerate(xs)}
                if all(_pick([cand[x.name]], VARS[x.name]) for x in xs):
                    good.append(cand)
                    break
            except Exception:
                continue
    if not good:
        return None
    tuples = sorted({tuple(g[x.name] for x in xs) for g in good})
    return {x.name: [t[i] for t in tuples] for i, x in enumerate(xs)}


def solve_step(f: Formula, known: dict[str, float]) -> dict[str, list[float]]:
    """Solve whatever the known values make determinable in one formula."""
    found: dict[str, list[float]] = {}
    work = dict(known)
    while True:
        subs = {VARS[n].sym: val for n, val in work.items()}
        pending = []
        for e in f.eqs:
            expr = (e.lhs - e.rhs).subs(subs)
            if expr.free_symbols:
                pending.append(expr)
        progress = False
        for expr in pending:
            if len(expr.free_symbols) == 1:
                x = next(iter(expr.free_symbols))
                vals = _solve1(expr, x)
                if vals:
                    found[x.name] = vals
                    work[x.name] = vals[0]
                    progress = True
                    break
        if progress:
            continue
        for size in range(2, len(pending) + 1):
            for group in combinations(pending, size):
                unk = set().union(*(g.free_symbols for g in group))
                if len(unk) != size:
                    continue
                xs = sorted(unk, key=lambda s: s.name)
                res = _solve_system(list(group), xs)
                if res:
                    for n, vals in res.items():
                        found[n] = vals
                        work[n] = vals[0]
                    progress = True
                    break
            if progress:
                break
        if not progress:
            return found


def defaults(names, given) -> dict[str, float]:
    return {n: VARS[n].default for n in names if n not in given and VARS[n].default is not None}


class Result(dict):
    defaults: dict[str, float]


def solve(key: str, **given: str | float) -> Result:
    """Solve one formula. Defaults (g, air density...) fill in unless given; they are reported as found=[value]."""
    f = FORMULAS[key]
    known = {n: to_si(n, v) for n, v in given.items()}
    bad = set(known) - set(f.names)
    if bad:
        raise KeyError(f"{key} has no variables {bad}; it uses {f.names}")
    used = defaults(f.names, known)
    res = Result(solve_step(f, known | used))
    res.defaults = used
    return res


@dataclass
class Step:
    formula: str
    var: str
    values: list[float]


def chain(
    target: str, given: dict[str, str | float], only: set[str] | None = None
) -> tuple[list[Step], dict[str, float], dict[str, float]]:
    """Apply formulas repeatedly until `target` is known. Returns the steps that led to it."""
    known = {n: to_si(n, v) for n, v in given.items()}
    used = defaults(VARS, known)
    known |= used
    pool = [f for f in FORMULAS.values() if not only or only & ({f.key} | set(f.tags))]
    steps: list[Step] = []
    deps: dict[str, tuple[str, set[str]]] = {}
    while target not in known:
        progress = False
        for f in pool:
            new = solve_step(f, known)
            for n, vals in new.items():
                if n in known:
                    continue
                known[n] = vals[0]
                steps.append(Step(f.key, n, vals))
                deps[n] = (f.key, set(f.names) - {n})
                progress = True
            if target in known:
                break
        if not progress:
            break
    if target not in known:
        return steps, known, {}
    need, todo = set(), [target]
    while todo:
        n = todo.pop()
        if n in deps and n not in need:
            need.add(n)
            todo.extend(deps[n][1])
    used = {n: v for n, v in used.items() if any(n in FORMULAS[s.formula].names for s in steps if s.var in need)}
    return [s for s in steps if s.var in need], known, used


def conflicts_in(f: Formula, known: dict[str, float], tol: float = 1e-3) -> list[str]:
    """Equations of one formula that the known values violate (all its variables must be known)."""
    out = []
    sub = {VARS[n].sym: known[n] for n in f.names}
    for e, src in zip(f.eqs, f.src, strict=True):
        lhs, rhs = float(sp.N(e.lhs.subs(sub))), float(sp.N(e.rhs.subs(sub)))
        if abs(lhs - rhs) > tol * max(abs(lhs), abs(rhs), 1e-12):
            out.append(f"{f.key}: {src}  ({lhs:.6g} != {rhs:.6g})")
    return out


def conflicts(known: dict[str, float], tol: float = 1e-3) -> list[str]:
    """Formulas whose variables are all known but whose equations don't hold."""
    return [c for f in FORMULAS.values() if set(f.names) <= known.keys() for c in conflicts_in(f, known, tol)]


def fmt(name: str, value: float, unit: str | None = None) -> str:
    v = VARS[name]
    q = Q(value, v.unit)
    if unit:
        q = q.to(unit)
    mag = q.magnitude
    s = f"{mag:.6g}"
    u = f"{q.units:~P}" if str(q.units) != "dimensionless" else ""
    return f"{s} {u}".strip()


TOOLS: dict[str, Callable] = {}


def tool(fn: Callable) -> Callable:
    """Procedural helper (tables, rounding rules, circuit solving). Args come in SI unless a unit is typed."""
    TOOLS[fn.__name__] = fn
    return fn


def tool_arg(value: str) -> float | str:
    try:
        return float(value.replace(",", ".")) if "," not in value or value.count(",") == 1 else value
    except ValueError:
        pass
    try:
        q = ureg.Quantity(value)
        return float(q.to_base_units().magnitude)
    except Exception:
        return value


def search(query: str) -> list[Formula]:
    words = query.lower().split()
    out = []
    for f in FORMULAS.values():
        hay = " ".join([f.key, f.title, *f.tags, f.notes, *(VARS[n].desc for n in f.names)]).lower()
        if all(w in hay for w in words):
            out.append(f)
    return out
