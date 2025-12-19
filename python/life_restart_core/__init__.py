"""Life Restart Core - High-performance life restart simulator engine.

This module provides a Rust implementation of the life restart simulator
with Python bindings via PyO3.
"""

from life_restart_core.life_restart_core import simulate_full_life

__all__ = ["simulate_full_life"]
__version__ = "0.1.0"
