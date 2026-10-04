"""Snapshot of the Python CLI for the structured comparison with the Rust engine (tests/cli_compare.rs).

Runs every data/examples.toml command and tests/golden/cli_commands.txt line, parses the output into structure
(defaults, solved variables with all roots, chain steps, target, conflicts, other numbers) and writes
tests/golden/cli_snapshot.json. The parser is mirrored in the Rust test; keep them in sync.
"""

import contextlib
import io
import json
import re
from pathlib import Path

from fsq_solver.cli import run
from fsq_solver.examples import EXAMPLES

ROOT = Path(__file__).resolve().parents[3]
NUM = re.compile(r"-?\d+(?:\.\d+)?(?:e[-+]?\d+)?")


def first_num(s: str) -> float | None:
    m = NUM.search(s)
    return float(m.group()) if m else None


def parse(out: str) -> dict:
    r = {"defaults": {}, "solved": {}, "steps": [], "target": None, "conflicts": [], "numbers": [], "error": False}
    for raw in out.splitlines():
        line = raw.strip()
        if m := re.match(r"\(default (\w+) = (.*)\)$", line):
            r["defaults"][m[1]] = first_num(m[2])
        elif m := re.match(r"\[(\w+)\] (\w+) = (.*)$", line):
            r["steps"].append([m[1], m[2], first_num(m[3])])
        elif m := re.match(r"=> (\w+) = (.*)$", line):
            r["target"] = [m[1], first_num(m[2])]
        elif line.startswith("! inconsistent:"):
            r["conflicts"].append(line.split(":")[1].strip())
        elif line.startswith("error:"):
            r["error"] = True
        elif m := re.match(r"(\w+) = (.*?)(?:   <- several roots)?   \(.*\)$", line):
            r["solved"][m[1]] = [first_num(p) for p in m[2].split(" | ")]
        else:
            r["numbers"] += [float(x) for x in NUM.findall(line)]
    return r


def main() -> None:
    cmds = [cmd for _, _, cmd, _ in EXAMPLES]
    for line in (ROOT / "tests" / "golden" / "cli_commands.txt").read_text().splitlines():
        if line.strip() and not line.startswith("#"):
            cmds.append(line.strip())
    snap = []
    for cmd in cmds:
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            run(cmd)
        snap.append({"cmd": cmd, "python": parse(buf.getvalue()), "raw": buf.getvalue()})
    out = ROOT / "tests" / "golden" / "cli_snapshot.json"
    out.write_text(json.dumps(snap, indent=1, ensure_ascii=False))
    print(f"{len(snap)} commands -> {out}")


if __name__ == "__main__":
    main()
