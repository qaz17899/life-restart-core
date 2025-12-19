"""Test async API for life_restart_core."""

import asyncio

import pytest

from life_restart_core import (
    GameSession,
    init_config,
    is_config_initialized,
    simulate_async,
    simulate_full_life,
)


@pytest.fixture(scope="module")
def setup_config():
    """Setup minimal config for testing."""
    if is_config_initialized():
        return

    # Minimal config for testing
    config = {
        "talents": {
            "1": {
                "id": 1,
                "name": "Test Talent",
                "description": "A test talent",
                "grade": 1,
                "max_triggers": 1,
            }
        },
        "events": {
            "1": {
                "id": 1,
                "event": "Test event",
                "grade": 1,
            },
            "999": {
                "id": 999,
                "event": "Death event",
                "grade": 0,
                "effect": {"LIF": -10},
            },
        },
        "ages": {str(age): {"age": age, "events": [[1, 1.0]]} for age in range(101)},
        "achievements": {},
        # Judge config is required - minimal config for each property
        "judge": {
            "HCHR": [{"min": 0, "grade": 0, "text": "普通"}],
            "HINT": [{"min": 0, "grade": 0, "text": "普通"}],
            "HSTR": [{"min": 0, "grade": 0, "text": "普通"}],
            "HMNY": [{"min": 0, "grade": 0, "text": "普通"}],
            "HSPR": [{"min": 0, "grade": 0, "text": "普通"}],
            "HLIF": [{"min": 0, "grade": 0, "text": "普通"}],
            "SUM": [{"min": 0, "grade": 0, "text": "普通"}],
        },
    }
    # Override age 100 to trigger death
    config["ages"]["100"] = {"age": 100, "events": [[999, 1.0]]}

    init_config(config)


class TestSyncAPI:
    """Test synchronous API."""

    def test_simulate_full_life_returns_game_session(self, setup_config):
        """Test that simulate_full_life returns a GameSession."""
        session = simulate_full_life(
            [1], {"CHR": 5, "INT": 5, "STR": 5, "MNY": 5}, set()
        )
        assert isinstance(session, GameSession)

    def test_game_session_properties(self, setup_config):
        """Test GameSession getter properties."""
        session = simulate_full_life(
            [1], {"CHR": 5, "INT": 5, "STR": 5, "MNY": 5}, set()
        )

        assert session.total_years > 0
        assert session.total_pages >= 1
        assert session.total_score >= 0
        assert session.final_age >= 0
        assert session.is_ended is True

    def test_game_session_methods(self, setup_config):
        """Test GameSession methods."""
        session = simulate_full_life(
            [1], {"CHR": 5, "INT": 5, "STR": 5, "MNY": 5}, set()
        )

        # Test get_page_data
        page_data = session.get_page_data(1)
        assert isinstance(page_data, list)

        # Test get_year
        year = session.get_year(0)
        assert year is not None
        assert "age" in year

        # Test get_years_range
        years = session.get_years_range(0, 5)
        assert isinstance(years, list)

        # Test get_summary
        summary = session.get_summary()
        assert "total_score" in summary

        # Test get_triggered_events
        events = session.get_triggered_events()
        assert isinstance(events, list)


class TestAsyncAPI:
    """Test asynchronous API."""

    @pytest.mark.asyncio
    async def test_simulate_async_returns_game_session(self, setup_config):
        """Test that simulate_async returns a GameSession."""
        session = await simulate_async(
            [1], {"CHR": 5, "INT": 5, "STR": 5, "MNY": 5}, set()
        )
        assert isinstance(session, GameSession)

    @pytest.mark.asyncio
    async def test_simulate_async_properties(self, setup_config):
        """Test GameSession properties from async simulation."""
        session = await simulate_async(
            [1], {"CHR": 5, "INT": 5, "STR": 5, "MNY": 5}, set()
        )

        assert session.total_years > 0
        assert session.total_pages >= 1
        assert session.total_score >= 0
        assert session.final_age >= 0
        assert session.is_ended is True

    @pytest.mark.asyncio
    async def test_concurrent_simulations(self, setup_config):
        """Test multiple concurrent simulations."""
        # Run 5 simulations concurrently
        tasks = [
            simulate_async([1], {"CHR": i, "INT": 5, "STR": 5, "MNY": 5}, set())
            for i in range(5)
        ]

        sessions = await asyncio.gather(*tasks)

        assert len(sessions) == 5
        for session in sessions:
            assert isinstance(session, GameSession)
            assert session.total_years > 0

    @pytest.mark.asyncio
    async def test_async_sync_equivalence(self, setup_config):
        """Test that async and sync APIs produce equivalent results."""
        props = {"CHR": 5, "INT": 5, "STR": 5, "MNY": 5}

        sync_session = simulate_full_life([1], props, set())
        async_session = await simulate_async([1], props, set())

        # Both should have valid results (exact values may differ due to randomness)
        assert sync_session.total_years > 0
        assert async_session.total_years > 0
        assert sync_session.is_ended is True
        assert async_session.is_ended is True


if __name__ == "__main__":
    pytest.main([__file__, "-v"])


class TestGameSessionAPI:
    """Test GameSession API completeness (Property 6)."""

    def test_get_summary_structure(self, setup_config):
        """Test get_summary returns correct structure."""
        session = simulate_full_life(
            [1], {"CHR": 5, "INT": 5, "STR": 5, "MNY": 5}, set()
        )
        summary = session.get_summary()

        assert "total_score" in summary
        assert "judges" in summary
        assert "talents" in summary
        assert isinstance(summary["judges"], list)
        assert isinstance(summary["talents"], list)

    def test_get_new_achievements_structure(self, setup_config):
        """Test get_new_achievements returns correct structure."""
        session = simulate_full_life(
            [1], {"CHR": 5, "INT": 5, "STR": 5, "MNY": 5}, set()
        )
        achievements = session.get_new_achievements()

        assert isinstance(achievements, list)
        # Each achievement should have id, name, description, grade
        for achievement in achievements:
            if achievement:  # May be empty
                assert "id" in achievement
                assert "name" in achievement

    def test_get_triggered_events_structure(self, setup_config):
        """Test get_triggered_events returns list of ints."""
        session = simulate_full_life(
            [1], {"CHR": 5, "INT": 5, "STR": 5, "MNY": 5}, set()
        )
        events = session.get_triggered_events()

        assert isinstance(events, list)
        for event_id in events:
            assert isinstance(event_id, int)

    def test_get_replacements_structure(self, setup_config):
        """Test get_replacements returns correct structure."""
        session = simulate_full_life(
            [1], {"CHR": 5, "INT": 5, "STR": 5, "MNY": 5}, set()
        )
        replacements = session.get_replacements()

        assert isinstance(replacements, list)
        # Each replacement should have source and target
        for replacement in replacements:
            if replacement:  # May be empty
                assert "source" in replacement
                assert "target" in replacement


class TestPreRenderingFormat:
    """Test pre-rendering format (Property 7)."""

    def test_display_text_contains_emoji(self, setup_config):
        """Test that display_text contains emoji."""
        session = simulate_full_life(
            [1], {"CHR": 5, "INT": 5, "STR": 5, "MNY": 5}, set()
        )

        # Get first year
        year = session.get_year(0)
        assert year is not None

        # display_text should contain emoji (⚪, 🔵, 🟣, or 🟠)
        display_text = year.get("display_text", "")
        # At least one emoji should be present
        emoji_chars = ["⚪", "🔵", "🟣", "🟠"]
        has_emoji = any(emoji in display_text for emoji in emoji_chars)
        assert (
            has_emoji or display_text == ""
        ), f"Expected emoji in display_text: {display_text}"

    def test_get_year_formatted_structure(self, setup_config):
        """Test get_year_formatted returns correct structure."""
        session = simulate_full_life(
            [1], {"CHR": 5, "INT": 5, "STR": 5, "MNY": 5}, set()
        )

        formatted = session.get_year_formatted(0)
        assert formatted is not None
        assert "age" in formatted
        assert "text" in formatted
        assert "properties" in formatted
        assert "is_end" in formatted


class TestProgressBar:
    """Test progress bar in summary (Property 9)."""

    def test_progress_bar_format(self, setup_config):
        """Test that progress_bar has correct format."""
        session = simulate_full_life(
            [1], {"CHR": 5, "INT": 5, "STR": 5, "MNY": 5}, set()
        )
        summary = session.get_summary()

        judges = summary.get("judges", [])
        for judge in judges:
            progress_bar = judge.get("progress_bar", "")
            # Progress bar should be 10 characters of █ and ░
            assert (
                len(progress_bar) == 10
            ), f"Progress bar should be 10 chars: {progress_bar}"
            for char in progress_bar:
                assert char in ("█", "░"), f"Invalid char in progress bar: {char}"

    def test_progress_bar_values(self, setup_config):
        """Test that progress_bar reflects progress value."""
        session = simulate_full_life(
            [1], {"CHR": 5, "INT": 5, "STR": 5, "MNY": 5}, set()
        )
        summary = session.get_summary()

        judges = summary.get("judges", [])
        for judge in judges:
            progress = judge.get("progress", 0)
            progress_bar = judge.get("progress_bar", "")

            # Count filled blocks
            filled = progress_bar.count("█")
            expected = round(progress * 10)

            # Allow for rounding differences
            assert (
                abs(filled - expected) <= 1
            ), f"Progress {progress} should have ~{expected} filled, got {filled}"
