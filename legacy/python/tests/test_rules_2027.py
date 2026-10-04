import pytest

from fsq_solver import tools as t


@pytest.mark.parametrize(
    ("event", "pmax", "pmin"),
    [
        ("skidpad", 50, 2.5),
        ("accel", 50, 2.5),
        ("autocross", 100, 10),
        ("endurance", 250, 25),
        ("dc_skidpad", 75, 3.75),
        ("dc_accel", 75, 3.75),
    ],
)
def test_score_bounds(event, pmax, pmin):
    assert t.event_score(event, 50, 50) == pytest.approx(pmax)
    assert t.event_score(event, 500, 50) == pytest.approx(pmin)


def test_score_formula_midpoint():
    # Tmax = 1.4 * 60 = 84, Tteam 72 -> halfway -> 90 * 0.25 + 10
    assert t.event_score("autocross", 72, 60, "2027") == pytest.approx(32.5)


def test_efficiency_clamped_at_ef_max():
    assert t.efficiency_score(t=1000, e=10, ef_min=1e6) == 0
    assert t.efficiency_score(t=1000, e=1, ef_min=1e6) == pytest.approx(75)


def test_bpp_nonfinalist_65_in_2027():
    assert t.static_nonfinalist(55, 61) == pytest.approx(65 * 55 / 61)
    assert t.static_nonfinalist(55, 61, rules="2026") == pytest.approx(70 * 55 / 61)


def test_weight_penalty_rule_example():
    assert t.weight_penalty(6.2) == 40
    assert t.weight_penalty(-5) == 0


def test_insulation_test_voltage():
    assert "insulation test 1000 V" in t.ts_rules(550)
    assert "insulation test 500 V" in t.ts_rules(550, rules="2026")
    assert "insulation test 250 V" in t.ts_rules(250)


def test_pcb_spacing_by_year():
    assert t.pcb_spacing(400) == 20
    assert t.pcb_spacing(400, "clearance") == 3.0
    assert t.pcb_spacing(237, "surface", "legacy") == 9.5


def test_trackdrive_and_dc():
    assert t.trackdrive_score(100, 100) == pytest.approx(150 + 50)
    assert t.dv_rank_score(1, 20) == pytest.approx(75)
