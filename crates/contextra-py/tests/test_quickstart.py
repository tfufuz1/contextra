import pytest
import numpy as np

pytest.importorskip("contextra._contextra")
import contextra


def test_quickstart_readme(tmp_path):
    # Initialize database
    db_path = str(tmp_path / "quickstart_data")
    db = contextra.open(db_path, dimension=128)
    collection = db.collection("documents")

    # Insert document with vector and metadata
    vector = np.random.rand(128).astype(np.float32)
    collection.insert(
        "doc_1",
        vector,
        metadata={"text": "Contextra provides high-performance embedded vector search."},
    )

    # Perform hybrid search
    results = collection.hybrid_search("vector search", vector, k=5)
    assert len(results) > 0
    assert results[0].id == "doc_1"
    assert (
        results[0].metadata["text"]
        == "Contextra provides high-performance embedded vector search."
    )
