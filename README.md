# Life Restart Core

High-performance life restart simulator core engine written in Rust with Python bindings.

## Features

- 🚀 **10-25x faster** than pure Python implementation
- 🐍 **Python 3.14+** compatible via PyO3
- 🔧 **Drop-in replacement** for existing Python code
- 📦 **Pre-built wheels** for Linux, Windows, and macOS

## Installation

```bash
pip install life-restart-core
```

## Usage

```python
from life_restart_core import simulate_full_life

result = simulate_full_life(
    talent_ids=[1001, 1002, 1003],
    properties={"CHR": 5, "INT": 5, "STR": 5, "MNY": 5},
    achieved_list=[],
    config={
        "talents": {...},
        "events": {...},
        "ages": {...},
        "achievements": {...},
        "judge": {...},
    }
)

# Result contains:
# - trajectory: List of year results
# - summary: Life summary with scores
# - new_achievements: Newly unlocked achievements
# - triggered_events: List of triggered event IDs
# - replacements: Talent replacement results
```

## Development

### Prerequisites

- Rust 1.70+
- Python 3.14+
- maturin

### Build

```bash
# Install maturin
pip install maturin

# Build and install in development mode
maturin develop

# Build release wheel
maturin build --release
```

### Test

```bash
# Run Rust tests
cargo test

# Run Python tests
pytest tests/
```

### Benchmark

```bash
cargo bench
```

## License

MIT
