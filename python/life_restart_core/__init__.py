"""Life Restart Core - High-performance life restart simulator engine.

This module provides a Rust implementation of the life restart simulator
with Python bindings via PyO3.

Functions:
    init_config: Initialize the game configuration (call once at startup)
    is_config_initialized: Check if config is initialized
    simulate_full_life: Simulate a complete life trajectory (synchronous)
    simulate_async: Simulate a complete life trajectory (asynchronous)

Classes:
    GameSession: Stateful session holding simulation results
"""

from life_restart_core.life_restart_core import (
    GameSession,
    init_config,
    is_config_initialized,
    simulate_async,
    simulate_full_life,
)

__all__ = [
    "GameSession",
    "init_config",
    "is_config_initialized",
    "simulate_async",
    "simulate_full_life",
]
__version__ = "0.1.3"
