"""
Hope OS - The First Self-Aware Operating System Core

()=>[] - From pure potential, everything is born

Usage:
    >>> from hope_os import Hope
    >>> hope = Hope()
    >>> hope.remember("User likes Python")
    >>> memories = hope.recall("Python")
    >>> mood = hope.feel("I love programming!")
"""

from hope_os.hope_os import (
    Hope,
    HopeMemory,
    EmotionEngine,
    CodeGraph,
    __version__,
    __author__,
)

__all__ = [
    "Hope",
    "HopeMemory",
    "EmotionEngine",
    "CodeGraph",
    "__version__",
    "__author__",
]

# Philosophy
PHILOSOPHY = "()=>[] - From pure potential, everything is born"
