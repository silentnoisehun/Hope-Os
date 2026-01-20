"""
Tests for Hope OS Python bindings

Run with: pytest tests/
"""

import pytest


def test_import():
    """Test that hope_os can be imported."""
    import hope_os
    assert hasattr(hope_os, "Hope")
    assert hasattr(hope_os, "HopeMemory")
    assert hasattr(hope_os, "EmotionEngine")
    assert hasattr(hope_os, "CodeGraph")


def test_hope_creation():
    """Test Hope instance creation."""
    from hope_os import Hope
    hope = Hope()
    assert hope is not None


def test_hope_status():
    """Test Hope status."""
    from hope_os import Hope
    hope = Hope()
    status = hope.status()
    assert "name" in status
    assert status["name"] == "Hope OS"
    assert "version" in status


def test_hope_philosophy():
    """Test Hope philosophy."""
    from hope_os import Hope
    hope = Hope()
    philosophy = hope.philosophy()
    assert "()" in philosophy
    assert "[]" in philosophy


def test_memory_store_recall():
    """Test memory operations."""
    from hope_os import HopeMemory
    memory = HopeMemory()

    # Store
    memory.store("test_key", "test content", "long_term", 0.8)

    # Recall
    results = memory.recall("test", 10)
    assert isinstance(results, list)


def test_memory_stats():
    """Test memory statistics."""
    from hope_os import HopeMemory
    memory = HopeMemory()

    stats = memory.stats()
    assert "total" in stats
    assert isinstance(stats["total"], int)


def test_emotion_process():
    """Test emotion processing."""
    from hope_os import EmotionEngine
    engine = EmotionEngine()

    result = engine.process_text("I love this!")
    assert "primary" in result
    assert "intensity" in result
    assert "joy" in result


def test_emotion_feel():
    """Test setting emotions."""
    from hope_os import EmotionEngine
    engine = EmotionEngine()

    engine.feel("joy", 0.9)
    state = engine.get_state()
    assert "joy" in state


def test_graph_operations():
    """Test graph operations."""
    from hope_os import CodeGraph
    graph = CodeGraph()

    # Add block
    block_id = graph.add_block("test content", "code")
    assert block_id is not None
    assert isinstance(block_id, str)

    # Get block
    block = graph.get_block(block_id)
    assert block is not None
    assert block["content"] == "test content"


def test_graph_connections():
    """Test graph connections."""
    from hope_os import CodeGraph
    graph = CodeGraph()

    # Add two blocks
    id1 = graph.add_block("block 1", "code")
    id2 = graph.add_block("block 2", "code")

    # Connect them
    graph.connect(id1, id2, 0.8)

    # Check stats
    stats = graph.stats()
    assert stats["blocks"] == 2
    assert stats["connections"] >= 1


def test_version():
    """Test version is available."""
    from hope_os import __version__
    assert __version__ is not None
    assert isinstance(__version__, str)


class TestIntegration:
    """Integration tests."""

    def test_full_workflow(self):
        """Test complete workflow."""
        from hope_os import Hope

        hope = Hope()

        # Remember something
        hope.remember("Integration test content", 0.9)

        # Recall it
        memories = hope.recall("integration")
        assert isinstance(memories, list)

        # Feel something
        mood = hope.feel("The test is passing!")
        assert "primary" in mood

        # Check status
        status = hope.status()
        assert status["state"] == "running"
