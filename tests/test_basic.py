"""Basic tests for life_restart_core module."""

import pytest


def test_import():
    """Test that the module can be imported."""
    from life_restart_core import simulate_full_life

    assert hasattr(simulate_full_life, "__call__")


def test_simulate_full_life_basic():
    """Test basic simulation with minimal config."""
    from life_restart_core import simulate_full_life

    talent_ids = [1]
    properties = {"CHR": 5, "INT": 5, "STR": 5, "MNY": 5}
    achieved_list = []

    # Minimal config
    config = {
        "talents": {
            "1": {
                "id": 1,
                "name": "Test Talent",
                "description": "A test talent",
                "grade": 1,
                "max_triggers": 1,
                "condition": None,
                "effect": None,
                "exclusive": False,
                "exclude": None,
                "replacement": None,
                "status": 0,
            }
        },
        "events": {
            "1": {
                "id": 1,
                "event": "Test event",
                "grade": 1,
                "NoRandom": False,
                "include": None,
                "exclude": None,
                "effect": {
                    "CHR": 0,
                    "INT": 0,
                    "STR": 0,
                    "MNY": 0,
                    "SPR": 0,
                    "LIF": 0,
                    "AGE": 0,
                    "RDM": 0,
                },
                "branch": None,
                "postEvent": None,
            },
            "999": {
                "id": 999,
                "event": "Death",
                "grade": 0,
                "NoRandom": False,
                "include": None,
                "exclude": None,
                "effect": {
                    "CHR": 0,
                    "INT": 0,
                    "STR": 0,
                    "MNY": 0,
                    "SPR": 0,
                    "LIF": -10,
                    "AGE": 0,
                    "RDM": 0,
                },
                "branch": None,
                "postEvent": None,
            },
        },
        "ages": {
            str(i): {
                "age": i,
                "talents": None,
                "events": [(1, 1.0)] if i < 100 else [(999, 1.0)],
            }
            for i in range(101)
        },
        "achievements": {},
        "judge": {},
    }

    result = simulate_full_life(talent_ids, properties, achieved_list, config)

    # Verify result structure
    assert "trajectory" in result
    assert "summary" in result
    assert "new_achievements" in result
    assert "triggered_events" in result
    assert "replacements" in result

    # Verify trajectory
    assert len(result["trajectory"]) > 0
    assert result["trajectory"][-1]["is_end"] == True

    # Verify summary
    assert "total_score" in result["summary"]
    assert "judges" in result["summary"]
    assert "talents" in result["summary"]


def test_simulate_full_life_with_achievements():
    """Test simulation with achievements."""
    from life_restart_core import simulate_full_life

    talent_ids = [1]
    properties = {"CHR": 10, "INT": 5, "STR": 5, "MNY": 5}
    achieved_list = []

    config = {
        "talents": {
            "1": {
                "id": 1,
                "name": "Test Talent",
                "description": "A test talent",
                "grade": 1,
                "max_triggers": 1,
                "condition": None,
                "effect": None,
                "exclusive": False,
                "exclude": None,
                "replacement": None,
                "status": 0,
            }
        },
        "events": {
            "999": {
                "id": 999,
                "event": "Death",
                "grade": 0,
                "NoRandom": False,
                "include": None,
                "exclude": None,
                "effect": {
                    "CHR": 0,
                    "INT": 0,
                    "STR": 0,
                    "MNY": 0,
                    "SPR": 0,
                    "LIF": -10,
                    "AGE": 0,
                    "RDM": 0,
                },
                "branch": None,
                "postEvent": None,
            },
        },
        "ages": {
            "0": {
                "age": 0,
                "talents": None,
                "events": [(999, 1.0)],
            },
        },
        "achievements": {
            "1": {
                "id": 1,
                "name": "High CHR",
                "description": "Start with high CHR",
                "grade": 1,
                "opportunity": "START",
                "condition": "CHR>=10",
            }
        },
        "judge": {},
    }

    result = simulate_full_life(talent_ids, properties, achieved_list, config)

    # Should have unlocked the achievement
    assert len(result["new_achievements"]) >= 1
    assert any(a["id"] == 1 for a in result["new_achievements"])
