import pytest
import os
import sys

# Add the python directory to sys.path so we can import contextra
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "../python")))


def test_mcp_import():
    from contextra.mcp import create_mcp_server

    assert create_mcp_server is not None


def test_mcp_missing_embed_fn(tmp_path):
    from contextra.mcp import create_mcp_server

    with pytest.raises(ValueError, match="embed_fn is required"):
        create_mcp_server(str(tmp_path / "test_db"))


def test_fastmcp_import():
    try:
        from fastmcp import FastMCP

        assert FastMCP is not None
    except ImportError:
        pytest.skip("fastmcp is not installed")
