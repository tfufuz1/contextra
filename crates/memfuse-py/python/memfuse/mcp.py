"""
MCP (Model Context Protocol) integration for MemFuse using FastMCP.
"""
from typing import Optional, List, Dict, Any, Callable, Union
import inspect
import asyncio
import numpy as np
import memfuse


def create_mcp_server(
    db_path: str,
    embed_fn: Optional[Callable[[str], Any]] = None,
    dimension: int = 768,
    name: str = "MemFuse",
):
    """
    Creates a FastMCP server instance for MemFuse.

    Args:
        db_path: Path to the MemFuse database directory.
        embed_fn: Callable mapping text strings to embedding vectors (numpy arrays). Required.
        dimension: Embedding vector dimension (default 768).
        name: Name of the FastMCP server instance.

    Returns:
        FastMCP server instance with registered tools and resources.
    """
    if embed_fn is None:
        raise ValueError(
            "embed_fn is required for FastMCP integration in memfuse.mcp. "
            "Pass a callable embed_fn: Callable[[str], np.ndarray]."
        )

    try:
        from fastmcp import FastMCP
    except ImportError as e:
        raise ImportError(
            "FastMCP is required for MCP integration. Install it with 'pip install memfuse[mcp]'"
        ) from e

    mcp = FastMCP(name)

    # Open database instance once in closure scope
    db = memfuse.open(db_path, dimension=dimension)

    async def _embed(text: str) -> np.ndarray:
        res = embed_fn(text)
        if inspect.iscoroutine(res) or asyncio.iscoroutine(res):
            res = await res
        return np.asarray(res, dtype=np.float32)

    @mcp.tool()
    async def memfuse_insert(
        id: str,
        text: str,
        collection: str = "default",
        metadata: Optional[Dict[str, Any]] = None,
    ) -> str:
        """Insert a document into MemFuse."""
        vector = await _embed(text)
        col = db.collection(collection)
        meta = dict(metadata) if metadata else {}
        meta["text"] = text
        col.insert(id, vector, meta)
        return f"Document '{id}' inserted successfully into collection '{collection}'."

    @mcp.tool()
    async def memfuse_search(
        query: str, collection: str = "default", k: int = 5
    ) -> List[Dict[str, Any]]:
        """Search documents in MemFuse."""
        vector = await _embed(query)
        col = db.collection(collection)
        results = col.hybrid_search(query, vector, k=k)
        return [{"id": r.id, "score": r.score, "metadata": r.metadata} for r in results]

    @mcp.tool()
    async def memfuse_get(
        id: str, collection: str = "default"
    ) -> Optional[Dict[str, Any]]:
        """Get a document by ID."""
        col = db.collection(collection)
        doc = col.get(id)
        if doc:
            return {"id": doc.id, "metadata": doc.metadata}
        return None

    @mcp.tool()
    async def memfuse_collections() -> List[str]:
        """List all collections."""
        return db.list_collections()

    @mcp.resource("memfuse://stats")
    async def memfuse_stats() -> str:
        """Get database statistics."""
        return str(db.stats())

    return mcp
