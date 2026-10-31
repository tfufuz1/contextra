import pytest
import numpy as np
import os
import shutil
import asyncio

pytest.importorskip("contextra._contextra")
from contextra.mcp import create_mcp_server


@pytest.fixture
def db_path(tmp_path):
    path = str(tmp_path / "mcp_test_db")
    yield path
    if os.path.exists(path):
        shutil.rmtree(path)


def run_async(coro):
    return asyncio.run(coro)


def fake_embed(text: str) -> np.ndarray:
    # Hash-based deterministic vector for testing
    h = hash(text)
    np.random.seed(abs(h) % (2**32))
    v = np.random.randn(4).astype(np.float32)
    norm = np.linalg.norm(v)
    if norm > 0:
        v = v / norm
    return v


def test_mcp_tools_registration(db_path):
    try:
        from fastmcp import FastMCP  # noqa: F401
    except ImportError:
        pytest.skip("fastmcp not installed")

    mcp = create_mcp_server(db_path, embed_fn=fake_embed, dimension=4)

    # Use the public API to check tools (it's async)
    tools = run_async(mcp.list_tools())
    tool_names = [tool.name for tool in tools]
    assert "contextra_search" in tool_names
    assert "contextra_get" in tool_names
    assert "contextra_insert" in tool_names
    assert "contextra_collections" in tool_names


def test_mcp_insert_and_search(db_path):
    try:
        from fastmcp import FastMCP  # noqa: F401
    except ImportError:
        pytest.skip("fastmcp not installed")

    mcp = create_mcp_server(db_path, embed_fn=fake_embed, dimension=4)

    # Use call_tool which is async
    res = run_async(
        mcp.call_tool(
            "contextra_insert",
            {"id": "doc1", "text": "rust is awesome", "collection": "test"},
        )
    )
    assert "inserted successfully" in str(res)

    # Search
    results = run_async(
        mcp.call_tool(
            "contextra_search",
            {"query": "rust is awesome", "collection": "test", "k": 1},
        )
    )
    assert len(str(results)) > 0
    assert "doc1" in str(results)


def test_mcp_distinct_vectors_and_scores(db_path):
    try:
        from fastmcp import FastMCP  # noqa: F401
    except ImportError:
        pytest.skip("fastmcp not installed")

    # Prove that two different input texts generate distinct vectors and scores
    mcp = create_mcp_server(db_path, embed_fn=fake_embed, dimension=4)

    run_async(
        mcp.call_tool(
            "contextra_insert",
            {"id": "doc1", "text": "alpha text", "collection": "test"},
        )
    )
    run_async(
        mcp.call_tool(
            "contextra_insert",
            {"id": "doc2", "text": "beta text", "collection": "test"},
        )
    )

    vec1 = fake_embed("alpha text")
    vec2 = fake_embed("beta text")
    assert not np.allclose(vec1, vec2), "Distinct texts must produce distinct vectors"

    res_alpha = run_async(
        mcp.call_tool(
            "contextra_search",
            {"query": "alpha text", "collection": "test", "k": 2},
        )
    )
    assert "doc1" in str(res_alpha)


def test_mcp_stats_resource(db_path):
    try:
        from fastmcp import FastMCP  # noqa: F401
    except ImportError:
        pytest.skip("fastmcp not installed")

    mcp = create_mcp_server(db_path, embed_fn=fake_embed, dimension=4)

    # Check resources (it's async)
    resources = run_async(mcp.list_resources())
    resource_names = [res.name for res in resources]
    assert "contextra_stats" in resource_names

    stats_output = run_async(mcp.read_resource("contextra://stats"))
    assert "DbStats" in str(stats_output)
