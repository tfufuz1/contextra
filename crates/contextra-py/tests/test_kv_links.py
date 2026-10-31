import numpy as np
import pytest
import os
import shutil

try:
    import contextra
except ImportError:
    pytest.skip("maturin develop noch nicht ausgeführt", allow_module_level=True)


@pytest.fixture
def db_path(tmp_path):
    path = str(tmp_path / "test_kv_links_db")
    yield path
    if os.path.exists(path):
        shutil.rmtree(path)


def test_put_kv_get_kv_roundtrip(db_path):
    db = contextra.open(db_path, dimension=4)
    col = db.collection("kv_test")

    nested_dict = {
        "string": "hello",
        "number": 42,
        "nested": {"key": "value", "list": [1, 2, 3]},
    }
    col.put_kv("kv1", nested_dict)

    res = col.get_kv("kv1")
    assert res == nested_dict

    assert col.get_kv("unknown_key") is None


def test_put_kv_if_absent_conflict(db_path):
    db = contextra.open(db_path, dimension=4)
    col = db.collection("kv_test")

    data1 = {"a": 1}
    data2 = {"a": 2}

    col.put_kv_if_absent("k1", data1)
    assert col.get_kv("k1") == data1

    with pytest.raises(Exception):
        col.put_kv_if_absent("k1", data2)

    # Original value remains untouched
    assert col.get_kv("k1") == data1


def test_kv_touches_no_indices(db_path):
    db = contextra.open(db_path, dimension=4)
    col = db.collection("kv_test")

    v = np.array([0.1, 0.2, 0.3, 0.4], dtype=np.float32)
    col.insert("doc1", v, metadata={"text": "vector doc"})

    assert col.len() == 1
    stats_before = col.stats()
    assert stats_before.num_vectors == 1

    col.put_kv("kv1", {"data": "non-vector key"})

    # KV entries do not increment document length or vector index count
    assert col.len() == 1
    stats_after = col.stats()
    assert stats_after.num_vectors == 1

    search_res = col.search(v, k=5)
    assert len(search_res) == 1
    assert search_res[0].id == "doc1"


def test_relate_bidirectional(db_path):
    db = contextra.open(db_path, dimension=4)
    col = db.collection("rel_test")
    v = np.array([0.1, 0.2, 0.3, 0.4], dtype=np.float32)

    col.insert("a", v)
    col.insert("b", v)

    col.relate_bidirectional("a", "b", "peer")

    rel_a = col.scan_prefix("__rel:a:peer:")
    assert len(rel_a) == 1
    assert rel_a[0][1]["from"] == "a"
    assert rel_a[0][1]["to"] == "b"

    rel_b = col.scan_prefix("__rel:b:peer:")
    assert len(rel_b) == 1
    assert rel_b[0][1]["from"] == "b"
    assert rel_b[0][1]["to"] == "a"


def test_link_memories_and_get_links_roundtrip(db_path):
    db = contextra.open(db_path, dimension=4)
    col = db.collection("links_test")
    v = np.array([0.1, 0.2, 0.3, 0.4], dtype=np.float32)

    col.insert("mem1", v)
    col.insert("mem2", v)

    col.link_memories("mem1", "mem2", "elaborates")

    links = col.get_links("mem1")
    assert len(links) == 1
    assert links[0]["relation"] == "elaborates"
    assert isinstance(links[0]["target"], int)
    assert isinstance(links[0]["created_at_tx"], int)

    # Self-link should fail
    with pytest.raises(Exception):
        col.link_memories("mem1", "mem1", "elaborates")


def test_link_memories_unknown_relation(db_path):
    db = contextra.open(db_path, dimension=4)
    col = db.collection("links_test")
    v = np.array([0.1, 0.2, 0.3, 0.4], dtype=np.float32)

    col.insert("mem1", v)
    col.insert("mem2", v)

    with pytest.raises(ValueError):
        col.link_memories("mem1", "mem2", "invalid_relation")


def test_invalid_and_empty_id(db_path):
    db = contextra.open(db_path, dimension=4)
    col = db.collection("invalid_id_test")

    with pytest.raises(ValueError):
        col.put_kv("", {"a": 1})

    with pytest.raises(ValueError):
        col.put_kv("   ", {"a": 1})

    with pytest.raises(ValueError):
        col.get_kv("")

    with pytest.raises(ValueError):
        col.get_links("")
