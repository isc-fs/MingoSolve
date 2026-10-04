import re

import pytest

from fsq_solver.cli import run
from fsq_solver.examples import EXAMPLES


def load_known() -> dict:
    import tomllib
    from pathlib import Path

    return tomllib.loads(
        (Path(__file__).resolve().parents[3] / "tests" / "golden" / "known_differences.toml").read_text()
    )


NUM = re.compile(r"-?\d+(?:\.\d+)?(?:e[-+]?\d+)?")


RUST_ONLY = set(load_known()["rust_only_tools"])


@pytest.mark.parametrize(("qid", "what", "cmd", "answer"), EXAMPLES, ids=[f"Q{e[0]}" for e in EXAMPLES])
def test_example(qid, what, cmd, answer, capsys):
    if cmd.split()[0] in RUST_ONLY:
        pytest.skip("tool exists only in the Rust engine (tests/golden/known_differences.toml)")
    run(cmd)
    out = capsys.readouterr().out
    nums = [float(n) for n in NUM.findall(out)]
    assert any(abs(n - answer) <= 0.005 * abs(answer) + 1e-9 for n in nums), f"Q{qid}: {answer} not in\n{out}"
