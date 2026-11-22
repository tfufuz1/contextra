import time
import threading
import pytest
import numpy as np
import contextra


def test_typed_exceptions_hierarchy():
    assert issubclass(contextra.TenantError, contextra.ContextraError)
    assert issubclass(contextra.NotFoundError, contextra.ContextraError)
    assert issubclass(contextra.IntegrityError, contextra.ContextraError)
    assert issubclass(contextra.ContextraIOError, Exception)
    assert issubclass(contextra.ContextraIndexError, Exception)
    assert issubclass(contextra.ContextraValueError, ValueError)


def test_collection_lifecycle_and_roundtrips(tmp_path):
    db_path = str(tmp_path / "lifecycle_db")
    db = contextra.open(db_path, dimension=4)

    # Collection creation and document insert
    col = db.collection("test_col")
    v1 = np.array([0.1, 0.2, 0.3, 0.4], dtype=np.float32)
    col.insert("doc1", v1, {"title": "lifecycle_test"})

    # Fetch document
    doc = col.get("doc1")
    assert doc is not None
    assert doc.id == "doc1"
    assert doc.metadata == {"title": "lifecycle_test"}

    # Update document
    v2 = np.array([0.5, 0.6, 0.7, 0.8], dtype=np.float32)
    col.update("doc1", v2, {"title": "updated_title"})
    updated_doc = col.get("doc1")
    assert updated_doc.metadata == {"title": "updated_title"}

    # Search in collection
    results = col.search(v2, k=1)
    assert len(results) == 1
    assert results[0].id == "doc1"

    # Delete document
    col.delete("doc1")
    assert col.get("doc1") is None


def test_gil_release_concurrency_python_threads(tmp_path):
    db_path = str(tmp_path / "gil_test_db")
    db = contextra.open(db_path, dimension=16)

    # Insert initial documents
    dummy_vec = np.zeros(16, dtype=np.float32)
    for i in range(20):
        db.insert(f"doc_{i}", dummy_vec)

    counter = [0]
    running = True

    def background_python_worker():
        nonlocal running
        while running:
            counter[0] += 1
            time.sleep(0.001)

    bg_thread = threading.Thread(target=background_python_worker)
    bg_thread.start()

    def search_worker():
        q = np.ones(16, dtype=np.float32)
        for _ in range(10):
            db.search(q, k=5)

    threads = [threading.Thread(target=search_worker) for _ in range(4)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    running = False
    bg_thread.join()

    assert counter[0] > 0, "Background Python thread was blocked by GIL"


def test_tenant_isolation_and_drop_collection(tmp_path):
    db_path = str(tmp_path / "tenant_db")
    db = contextra.open(db_path, dimension=4)

    col1 = db.collection("tenant_a")
    col2 = db.collection("tenant_b")

    v = np.array([0.1, 0.1, 0.1, 0.1], dtype=np.float32)
    col1.insert("shared_id", v, {"tenant": "A"})
    col2.insert("shared_id", v, {"tenant": "B"})

    doc1 = col1.get("shared_id")
    doc2 = col2.get("shared_id")

    assert doc1.metadata == {"tenant": "A"}
    assert doc2.metadata == {"tenant": "B"}

    # Drop tenant_a collection
    db.drop_collection("tenant_a")
    assert "tenant_a" not in db.list_collections()
    assert "tenant_b" in db.list_collections()
