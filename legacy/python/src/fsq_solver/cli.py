from __future__ import annotations

import inspect
import shlex
import sys

from . import formulas, tools  # noqa: F401  (registers everything)
from .core import (
    FORMULAS,
    TOOLS,
    VARS,
    Q,
    chain,
    conflicts,
    fmt,
    search,
    solve,
    tool_arg,
    ureg,
)
from .examples import EXAMPLES

HELP = """\
  find <words>                       search formulas (key, title, tags, variable descriptions)
  show <formula>                     equations and variables of a formula
  solve <formula> k=v ... [@x=unit]  solve a formula for whatever is missing
  chain <target> k=v ... [@x=unit] [only=tag,tag]
                                     chain all formulas until <target> is found
  vars [words]                       list variable names, units and meaning
  calc <expr> [-> unit]              unit-aware calculator: calc 0.5*300kg*(100km/h)**2 -> kJ
  ex <words>                         past quiz questions solved with fsq (how to enter them)
  tools [words]                      list procedural tools (scoring, rule tables, circuits...)
  <tool> args | k=v ...              run a tool: event_score skidpad t_team=5.6 t_min=5.1 rules=legacy
  help | quit
values take units: v=100km/h  n=8000rpm  m=280kg  (bare numbers are in the variable's SI unit)"""


def _parse(args: list[str]) -> tuple[dict[str, str], dict[str, str], set[str] | None]:
    given, out, only = {}, {}, None
    for a in args:
        k, _, v = a.partition("=")
        if k == "only":
            only = set(v.split(","))
        elif k.startswith("@"):
            out[k[1:]] = v
        elif v and v != "?":
            given[k] = v if any(c.isalpha() or c == "%" for c in v) else float(v.replace(",", "."))
    return given, out, only


def _show(key: str) -> None:
    f = FORMULAS[key]
    print(f"{f.key}: {f.title}")
    for s in f.src:
        print(f"    {s}")
    for n in f.names:
        print(f"  {n:<10} [{VARS[n].unit}] {VARS[n].desc}")
    if f.notes:
        print(f"  note: {f.notes}")


def _call_tool(fn, args: list[str]):
    params = inspect.signature(fn).parameters
    names = list(params)
    pos, kw = [], {}
    for a in args:
        k, eq, v = a.partition("=")
        if eq and k in params:
            kw[k] = v
        else:
            pos.append(a)
    bound = dict(zip(names, pos)) | kw
    conv = {k: v if params[k].annotation == "str" else tool_arg(v) for k, v in bound.items()}
    out = fn(**conv)
    return f"{out:.6g}" if isinstance(out, float) else out


def run(line: str) -> None:
    parts = shlex.split(line)
    if not parts:
        return
    cmd, args = parts[0], parts[1:]
    if cmd in ("help", "?"):
        print(HELP)
    elif cmd == "find":
        for f in search(" ".join(args)):
            print(f"  {f.key:<28} {f.title}   ({', '.join(f.names)})")
    elif cmd == "show":
        _show(args[0])
    elif cmd == "vars":
        w = " ".join(args).lower()
        for n, v in sorted(VARS.items()):
            if w in f"{n} {v.desc}".lower():
                print(f"  {n:<12} [{v.unit}] {v.desc}")
    elif cmd == "solve":
        given, out, _ = _parse(args[1:])
        res = solve(args[0], **given)
        if not res:
            print("  nothing determinable from these inputs:")
            _show(args[0])
        for n, v in res.defaults.items():
            print(f"  (default {n} = {fmt(n, v)})")
        for n, vals in res.items():
            shown = " | ".join(fmt(n, x, out.get(n)) for x in vals)
            flag = "   <- several roots" if len(vals) > 1 else ""
            print(f"  {n} = {shown}{flag}   ({VARS[n].desc})")
    elif cmd == "chain":
        target = args[0]
        given, out, only = _parse(args[1:])
        steps, known, used = chain(target, given, only)
        for n, v in used.items():
            print(f"  (default {n} = {fmt(n, v)})")
        for s in steps:
            print(f"  [{s.formula}] {s.var} = {fmt(s.var, s.values[0], out.get(s.var))}")
        if target in known:
            print(f"  => {target} = {fmt(target, known[target], out.get(target))}")
        else:
            print(f"  could not reach {target}; known: {', '.join(sorted(known))}")
        for c in conflicts(known):
            print(f"  ! inconsistent: {c}")
    elif cmd == "calc":
        expr, _, unit = " ".join(args).partition("->")
        q = ureg.parse_expression(expr.strip())
        if unit.strip():
            q = q.to(unit.strip())
        print(f"  {q:~.6gP}" if isinstance(q, Q) else f"  {q:.6g}")
    elif cmd in ("quit", "exit", "q"):
        raise EOFError
    elif cmd == "ex":
        w = " ".join(args).lower()
        for qid, what, c, ans in EXAMPLES:
            if all(x in f"q{qid} {what} {c}".lower() for x in w.split()):
                print(f"  Q{qid}: {what}  -> {ans}\n      {c}")
    elif cmd == "tools":
        w = " ".join(args).lower()
        for name, fn in TOOLS.items():
            doc = (fn.__doc__ or "").strip()
            if w in f"{name} {doc}".lower():
                print(
                    f"  {name}{str(inspect.signature(fn)).replace(chr(39), '')}\n      {doc.splitlines()[0] if doc else ''}"
                )
    elif cmd in FORMULAS:
        run("solve " + line)
    elif cmd in TOOLS:
        print("  " + str(_call_tool(TOOLS[cmd], args)))
    else:
        print(f"  unknown command {cmd!r}; try help")


def main() -> None:
    if len(sys.argv) > 1:
        run(shlex.join(sys.argv[1:]))
        return
    try:
        import readline  # noqa: F401
    except ImportError:
        pass
    print(f"fsq: {len(FORMULAS)} formulas, {len(VARS)} variables. 'help' for commands.")
    while True:
        try:
            line = input("fsq> ")
            run(line)
        except (EOFError, KeyboardInterrupt):
            print()
            return
        except Exception as e:  # keep the REPL alive during a quiz
            print(f"  error: {e}")
