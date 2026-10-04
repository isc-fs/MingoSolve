import re

import pytest

from fsq_solver.cli import run
from fsq_solver.examples import EXAMPLES

NUM = re.compile(r"-?\d+(?:\.\d+)?(?:e[-+]?\d+)?")


@pytest.mark.parametrize(("qid", "what", "cmd", "answer"), EXAMPLES, ids=[f"Q{e[0]}" for e in EXAMPLES])
def test_example(qid, what, cmd, answer, capsys):
    run(cmd)
    out = capsys.readouterr().out
    nums = [float(n) for n in NUM.findall(out)]
    assert any(abs(n - answer) <= 0.005 * abs(answer) + 1e-9 for n in nums), f"Q{qid}: {answer} not in\n{out}"
