import gc
import os
import shutil
import numpy as np
import pytest
import contextra

@pytest.fixture
def db_path(tmp_path):
    path = str(tmp_path / "test_j33closure_fb_db")
    yield path
    if os.path.exists(path):
        shutil.rmtree(path)

def test_j33closure_collection_and_db_flatbuffer_search(db_path):
    """Test search_fb and hybrid_search_fb Python calling capability on PyCollection and PyContextra (Db)."""
    db = contextra.open(db_path, dimension=4)
    col = db.collection("fb_closure_col")

    v1 = np.array([1.0, 0.0, 0.0, 0.0], dtype=np.float32)
    v2 = np.array([0.0, 1.0, 0.0, 0.0], dtype=np.float32)

    col.insert("doc_fb_1", v1, metadata={"title": "FlatBuffers integration test"})
    db.insert("db_doc_fb_1", v2, metadata={"title": "DB level FlatBuffers test"})

    # 1. Collection search_fb
    col_fb_bytes = col.search_fb(v1, k=1)
    assert isinstance(col_fb_bytes, bytes)
    assert len(col_fb_bytes) > 0

    # 2. Collection hybrid_search_fb
    col_hybrid_fb_bytes = col.hybrid_search_fb("FlatBuffers", v1, k=1)
    assert isinstance(col_hybrid_fb_bytes, bytes)
    assert len(col_hybrid_fb_bytes) > 0

    # 3. Db search_fb
    db_fb_bytes = db.search_fb(v2, k=1)
    assert isinstance(db_fb_bytes, bytes)
    assert len(db_fb_bytes) > 0

    # 4. Db hybrid_search_fb
    db_hybrid_fb_bytes = db.hybrid_search_fb("FlatBuffers", v2, k=1)
    assert isinstance(db_hybrid_fb_bytes, bytes)
    assert len(db_hybrid_fb_bytes) > 0

    # Verify memory GC safety on PyBytes
    gc.collect()
    assert len(col_fb_bytes) > 0
    assert len(db_hybrid_fb_bytes) > 0


def test_j33closure_flatbuffer_search_bounds_validation(db_path):
    """Test parameter validation for search_fb and hybrid_search_fb from Python."""
    db = contextra.open(db_path, dimension=4)
    col = db.collection("bounds_col")
    v = np.array([0.5, 0.5, 0.0, 0.0], dtype=np.float32)

    # Invalid k = 0
    with pytest.raises(ValueError):
        col.search_fb(v, k=0)

    with pytest.raises(ValueError):
        col.hybrid_search_fb("query", v, k=0)

    # Invalid k > 1000
    with pytest.raises(ValueError):
        col.search_fb(v, k=1001)

    with pytest.raises(ValueError):
        col.hybrid_search_fb("query", v, k=1001)
