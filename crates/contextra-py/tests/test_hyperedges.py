import os
import shutil
import pytest
import numpy as np

pytest.importorskip("contextra._contextra")
import contextra


@pytest.fixture
def db_path(tmp_path):
    path = str(tmp_path / "hyperedge_db")
    yield path
    if os.path.exists(path):
        shutil.rmtree(path)


def test_relate_n_ary_success_and_unique_ids(db_path):
    db = contextra.open(db_path, dimension=4)
    col = db.collection("docs")
    v = np.array([0.1, 0.2, 0.3, 0.4], dtype=np.float32)

    col.insert("a", v)
    col.insert("b", v)
    col.insert("c", v)

    # (a) relate_n_ary with 3 participants returns an integer hyperedge ID
    he_id1 = col.relate_n_ary(
        "collaborated",
        [("a", "author"), ("b", "reviewer"), ("c", "reviewer")],
    )
    assert isinstance(he_id1, int)

    # Two calls return distinct hyperedge IDs
    he_id2 = col.relate_n_ary(
        "collaborated",
        [("a", "author"), ("b", "reviewer"), ("c", "reviewer")],
    )
    assert isinstance(he_id2, int)
    assert he_id1 != he_id2


def test_relate_n_ary_validation_insufficient_participants(db_path):
    db = contextra.open(db_path, dimension=4)
    col = db.collection("docs")
    v = np.array([0.1, 0.2, 0.3, 0.4], dtype=np.float32)
    col.insert("a", v)

    # (b) <2 participants raises ValueError/ContextraValueError
    with pytest.raises((contextra.ContextraValueError, ValueError)) as excinfo:
        col.relate_n_ary("solo", [("a", "author")])
    assert "at least 2 participants" in str(excinfo.value)

    with pytest.raises((contextra.ContextraValueError, ValueError)) as excinfo2:
        col.relate_n_ary("empty", [])
    assert "at least 2 participants" in str(excinfo2.value)


def test_relate_n_ary_validation_empty_role_and_predicate(db_path):
    db = contextra.open(db_path, dimension=4)
    col = db.collection("docs")
    v = np.array([0.1, 0.2, 0.3, 0.4], dtype=np.float32)
    col.insert("a", v)
    col.insert("b", v)

    # (c) empty role raises ValueError/ContextraValueError
    with pytest.raises((contextra.ContextraValueError, ValueError)) as excinfo:
        col.relate_n_ary("collaborated", [("a", "author"), ("b", "")])
    assert "cannot be empty" in str(excinfo.value)

    # empty predicate raises ValueError/ContextraValueError
    with pytest.raises((contextra.ContextraValueError, ValueError)) as excinfo2:
        col.relate_n_ary("", [("a", "author"), ("b", "reviewer")])
    assert "cannot be empty" in str(excinfo2.value)


def test_relate_n_ary_source_doc_id(db_path):
    db = contextra.open(db_path, dimension=4)
    col = db.collection("docs")
    v = np.array([0.1, 0.2, 0.3, 0.4], dtype=np.float32)

    col.insert("a", v)
    col.insert("b", v)
    col.insert("src_doc", v)

    # (d) source_doc_id is accepted
    he_id = col.relate_n_ary(
        "derived_from",
        [("a", "target"), ("b", "target")],
        source_doc_id="src_doc",
    )
    assert isinstance(he_id, int)


def test_relate_n_ary_poisoned_collection(db_path):
    db = contextra.open(db_path, dimension=4)
    col = db.collection("docs")
    v = np.array([0.1, 0.2, 0.3, 0.4], dtype=np.float32)
    col.insert("a", v)
    col.insert("b", v)

    # Trigger panic on col
    with pytest.raises(RuntimeError) as exc_info:
        col._trigger_panic_for_test("Trigger panic for hyperedge test")
    assert "Rust panic caught at FFI boundary" in str(exc_info.value)
    assert col.is_poisoned

    # (e) Calling relate_n_ary on poisoned collection follows existing error behavior
    with pytest.raises(RuntimeError) as exc_info2:
        col.relate_n_ary("collaborated", [("a", "author"), ("b", "reviewer")])
    assert "engine poisoned after previous panic" in str(exc_info2.value)
