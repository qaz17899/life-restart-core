"""Type stubs for life_restart_core module."""

from collections.abc import Awaitable
from typing import Any

__version__: str
__all__: list[str]

class GameSession:
    """Stateful session holding simulation results in Rust heap.

    This class allows Python to hold a handle to the simulation result
    without serializing the entire data structure. Data is lazily accessed
    through getter methods.
    """

    @property
    def total_years(self) -> int:
        """Total number of years in the trajectory."""
        ...

    @property
    def total_pages(self) -> int:
        """Total number of pages (50 years per page by default)."""
        ...

    @property
    def total_score(self) -> int:
        """Total score from summary."""
        ...

    @property
    def final_age(self) -> int:
        """Final age (age of the last year)."""
        ...

    @property
    def is_ended(self) -> bool:
        """Whether the simulation has ended."""
        ...

    def get_page_data(
        self, page: int, years_per_page: int | None = None
    ) -> list[dict[str, Any]]:
        """Get paginated trajectory data.

        Args:
            page: Page number (1-indexed)
            years_per_page: Number of years per page (default: 50)

        Returns:
            List of year dicts for the requested page, or empty list if out of bounds
        """
        ...

    def get_year(self, index: int) -> dict[str, Any] | None:
        """Get a single year by index.

        Args:
            index: Year index (0-indexed)

        Returns:
            Year dict or None if out of bounds
        """
        ...

    def get_years_range(self, start: int, end: int) -> list[dict[str, Any]]:
        """Get a range of years.

        Args:
            start: Start index (inclusive)
            end: End index (exclusive)

        Returns:
            List of year dicts, or empty list if out of bounds
        """
        ...

    def get_year_formatted(self, index: int) -> dict[str, Any] | None:
        """Get pre-formatted year content.

        Args:
            index: Year index (0-indexed)

        Returns:
            Dict with pre-rendered text, or None if out of bounds
        """
        ...

    def get_summary(self) -> dict[str, Any]:
        """Get the summary with pre-rendered progress bars."""
        ...

    def get_new_achievements(self) -> list[dict[str, Any]]:
        """Get new achievements unlocked during simulation."""
        ...

    def get_triggered_events(self) -> list[int]:
        """Get triggered event IDs."""
        ...

    def get_replacements(self) -> list[dict[str, Any]]:
        """Get talent replacements."""
        ...

def init_config(
    config: dict[str, Any],
    emoji_map: dict[int, str] | None = None,
) -> None:
    """Initialize the game configuration (call once at startup).

    This caches the configuration in Rust memory, eliminating the need to
    deserialize it on every simulation call. Call this once when your bot starts.

    Args:
        config: Game configuration containing talents, events, ages, achievements
        emoji_map: Optional emoji map for grade-to-emoji conversion
                   (default: {0: "⚪", 1: "🔵", 2: "🟣", 3: "🟠"})
    """
    ...

def is_config_initialized() -> bool:
    """Check if config is initialized."""
    ...

def simulate_full_life(
    talent_ids: list[int],
    properties: dict[str, int],
    achieved_ids: set[int],
) -> GameSession:
    """Simulate a complete life trajectory (fast version using cached config).

    Args:
        talent_ids: List of selected talent IDs
        properties: Initial property allocation {CHR, INT, STR, MNY}
        achieved_ids: Set of already achieved achievement IDs

    Returns:
        A GameSession object containing the simulation results

    Raises:
        RuntimeError: If init_config was not called first
    """
    ...

def simulate_async(
    talent_ids: list[int],
    properties: dict[str, int],
    achieved_ids: set[int],
) -> Awaitable[GameSession]:
    """Simulate a complete life trajectory asynchronously.

    This function runs the simulation in a background thread using Tokio's
    spawn_blocking, allowing Python's asyncio event loop to remain responsive.
    The GIL is automatically released during the CPU-intensive simulation.

    Args:
        talent_ids: List of selected talent IDs
        properties: Initial property allocation {CHR, INT, STR, MNY}
        achieved_ids: Set of already achieved achievement IDs

    Returns:
        A Python awaitable that resolves to a GameSession object

    Raises:
        RuntimeError: If init_config was not called first

    Example:
        session = await simulate_async([1, 2, 3], {"CHR": 5, "INT": 5}, set())
        print(session.total_years)
    """
    ...
