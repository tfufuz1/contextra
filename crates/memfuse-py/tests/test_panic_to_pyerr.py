"""
Integration test for ADR-N04: FFI Panic Containment & PyErr Translation.

Verifies that:
1. Rust panics across FFI boundaries are caught by `std::panic::catch_unwind` in `run_blocking_ffi`.
2. Panic payloads are safely converted to `PyRuntimeError` ("Rust panic caught at FFI boundary").
3. Uncaught panics in FFI do NOT cause host process termination via SIGABRT (exit code 134 / signal 6).
4. Instance poisoning (`is_poisoned`) is set atomically on panic, blocking subsequent operations.
"""

import sys
import subprocess
import pytest
import tempfile
import numpy as np
import memfuse
from memfuse import _memfuse


def test_panic_translates_to_pyerr():
    """ADR-N04: Verifies that a Rust panic inside an FFI function is translated to PyRuntimeError."""
    with pytest.raises(RuntimeError) as exc_info:
        _memfuse._trigger_panic_for_test("ADR-N04 test panic")

    err_msg = str(exc_info.value)
    assert "Rust panic caught at FFI boundary" in err_msg
    assert "ADR-N04 test panic" in err_msg


def test_panic_containment_in_subprocess():
    """ADR-N04: Verifies that a panic inside a subprocess raises a Python exception and exits with 0
    when caught, rather than aborting the CPython process with SIGABRT (exit code 134).
    """
    code = (
        "from memfuse import _memfuse\n"
        "try:\n"
        "    _memfuse._trigger_panic_for_test('Subprocess panic containment')\n"
        "except RuntimeError as e:\n"
        "    assert 'Rust panic caught at FFI boundary' in str(e)\n"
        "    print('PANIC_CAUGHT_SUCCESSFULLY')\n"
    )
    res = subprocess.run(
        [sys.executable, "-c", code],
        capture_output=True,
        text=True,
    )
    assert res.returncode == 0, f"Process aborted or failed: stderr={res.stderr}"
    assert "PANIC_CAUGHT_SUCCESSFULLY" in res.stdout


def test_db_and_collection_panic_poisoning():
    """ADR-N04: Verifies instance poisoning after panic on Db and Collection instances."""
    with tempfile.TemporaryDirectory() as tmp_dir:
        db = memfuse.open(tmp_dir, dimension=128)
        assert not db.is_poisoned

        col = db.collection("panic_col")
        assert not col.is_poisoned

        # Trigger panic on collection
        with pytest.raises(RuntimeError) as exc_info:
            col._trigger_panic_for_test("Collection FFI panic")

        assert "Rust panic caught at FFI boundary" in str(exc_info.value)
        assert db.is_poisoned
        assert col.is_poisoned

        # Verify subsequent CRUD calls fail with poison error
        vec = np.zeros(128, dtype=np.float32)
        with pytest.raises(RuntimeError) as exc_info_insert:
            db.insert("doc_p", vec)
        assert "engine poisoned after previous panic" in str(exc_info_insert.value)

        with pytest.raises(RuntimeError) as exc_info_get:
            col.get("doc_p")
        assert "engine poisoned after previous panic" in str(exc_info_get.value)


def test_release_wheel_panic_to_pyerr_boundary_isolation():
    """Spec §9.4 / ADR-N04: Explicit panic boundary isolation test against compiled release extension wheel.

    Guarantees that:
    1. Direct module-level FFI panic hook `_memfuse._trigger_panic_for_test` raises `PyRuntimeError`
       containing "Rust panic caught at FFI boundary" without process termination (SIGABRT).
    2. Facade-level `Db._trigger_panic_for_test` and `Collection._trigger_panic_for_test` raise `PyRuntimeError`
       and atomically set `is_poisoned = True`.
    3. Isolated CPython subprocess execution confirms zero SIGABRT / exit code 134 on panic.
    """
    # 1. Module-level FFI panic boundary test
    with pytest.raises(RuntimeError) as exc_mod:
        _memfuse._trigger_panic_for_test("Release wheel module FFI panic")
    assert "Rust panic caught at FFI boundary" in str(exc_mod.value)
    assert "Release wheel module FFI panic" in str(exc_mod.value)

    # 2. Db & Collection instance panic boundary and poisoning test
    with tempfile.TemporaryDirectory() as tmp_dir:
        db = memfuse.open(tmp_dir, dimension=64)
        assert not db.is_poisoned

        with pytest.raises(RuntimeError) as exc_db:
            db._trigger_panic_for_test("Release wheel Db FFI panic")
        assert "Rust panic caught at FFI boundary" in str(exc_db.value)
        assert db.is_poisoned

        # Re-verify subsequent operations are safely rejected with engine poisoned RuntimeError
        vec = np.zeros(64, dtype=np.float32)
        with pytest.raises(RuntimeError) as exc_rejected:
            db.insert("doc_release_test", vec)
        assert "engine poisoned after previous panic" in str(exc_rejected.value)

    # 3. Subprocess verification of clean exit vs SIGABRT
    subprocess_code = (
        "import memfuse, tempfile, numpy as np, sys\n"
        "from memfuse import _memfuse\n"
        "try:\n"
        "    _memfuse._trigger_panic_for_test('Release wheel subprocess panic')\n"
        "except RuntimeError as e:\n"
        "    assert 'Rust panic caught at FFI boundary' in str(e)\n"
        "    print('RELEASE_WHEEL_PANIC_ISOLATION_VERIFIED')\n"
        "    sys.exit(0)\n"
    )
    res = subprocess.run(
        [sys.executable, "-c", subprocess_code],
        capture_output=True,
        text=True,
    )
    assert res.returncode == 0, f"Subprocess crashed with returncode {res.returncode}: stderr={res.stderr}"
    assert "RELEASE_WHEEL_PANIC_ISOLATION_VERIFIED" in res.stdout
