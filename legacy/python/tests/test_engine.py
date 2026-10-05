import pytest

import fsq_solver.formulas  # noqa: F401
from fsq_solver.core import FORMULAS, chain, conflicts, solve, to_si


def test_solves_any_unknown():
    assert solve("speed", s=75, t=3.5)["v_avg"] == [pytest.approx(21.4286, rel=1e-4)]
    assert solve("speed", v_avg="100 km/h", t=3)["s"] == [pytest.approx(83.333, rel=1e-4)]
    assert solve("speed", v_avg=20, s=100)["t"] == [pytest.approx(5)]


def test_units_and_decimal_comma():
    assert to_si("v", "100 km/h") == pytest.approx(27.7778, rel=1e-4)
    assert to_si("s", "1,5 km") == pytest.approx(1500)


def test_system_with_two_unknowns():
    r = solve("uniform_accel", v0=0, s=75, v="100 km/h")
    assert r["t"] == [pytest.approx(5.4)]
    assert r["a"] == [pytest.approx(5.1440, rel=1e-4)]


def test_chain_and_conflicts():
    _, known, _ = chain("t", {"v0": 0, "s": 75, "v": "100 km/h"})
    assert known["t"] == pytest.approx(5.4)
    assert not conflicts(known)
    assert conflicts({"v0": 0, "s": 75, "v": 27.78, "t": 2.7, "a": 10.29})


def test_every_formula_parses_and_has_variables():
    for f in FORMULAS.values():
        assert f.eqs and f.names, f.key


def test_system_roots_stay_paired():
    r = solve("battery_load", N_s=103, V_cell=3.8, R_pack=0.08, P="30 kW")
    for i, current in enumerate(r["I"]):
        assert r["V_t"][i] == pytest.approx(r["V_oc"][0] - current * 0.08)
