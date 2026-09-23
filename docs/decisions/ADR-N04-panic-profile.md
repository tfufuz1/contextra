# ADR-N04: Panic-Profilierung — Workspace unwind vs. Release-Abort & FFI Panic-Isolation

* **Status:** Final
* **Datum:** 2026-09-22
* **Kontext / Auslöser:**
  In Rust-Projekten, die C- oder Python-FFI-Schnittstellen exponieren (wie `contextra-py` via PyO3), führt eine unbedachte Verwendung von `panic = "abort"` im Release-Profil dazu, dass Unhandled Panics im CPython-Interpreter-Prozess direkt zu unkontrollierten Prozessabstürzen via `SIGABRT` (exit code 134) führen.
  Dies verletzt die Stabilitätsanforderungen von FFI-Grenzschichten. Gleichzeitig benötigen eigenständige Server- und CLI-Binaries ohne FFI-Anbindung maximale Binärgrößen-Optimierungen und deterministischen Abbruch.

## Entscheidungen

1. **Gezielte Profil-Konfiguration im Root-`Cargo.toml`:**
   * **Standard Release-Profil (`[profile.release]`):** Verwendet ausnahmslos `panic = "unwind"`, um Stack-Unwinding über FFI-Grenzen hinweg sowie geordnetes `catch_unwind` zu ermöglichen.
   * **`release-abort` Profil (`[profile.release-abort]`):** Erbt von `release` und setzt explizit `panic = "abort"`. Dieses Profil gilt ausschließlich für reine Standalone-Binaries ohne FFI/PyO3-Grenzen.

2. **FFI Panic Isolation in `contextra-py`:**
   * Sämtliche FFI-Aufrufe in `contextra-py` fangen Rust-Panics an der FFI-Schnittstellen-Grenze mittels `std::panic::catch_unwind` (über die Hilfsfunktion `run_blocking_ffi`) ab.
   * Abgefangene Panics werden kontrolliert in strukturierte Python `PyRuntimeError`-Exceptions ("Rust panic caught at FFI boundary") übersetzt (ADR-056 / ADR-059).
   * Bei einem Panic-Ereignis werden betroffene `Db`- und `Collection`-Instanzen atomar als vergiftet (`is_poisoned = true`) markiert. Nachfolgende Operationen auf vergifteten Instanzen werden geordnet mit einem Poison-Fehler abgelehnt.

3. **Verifikation durch FFI-Panic-Testsuite:**
   Die Korrektheit der Panic-Isolation wird automatisiert über die Testsuite `crates/contextra-py/tests/test_panic_to_pyerr.py` verifiziert. Sie prüft:
   * Übersetzung von Rust-Panics in `PyRuntimeError`.
   * Subprozess-Prozessüberleben ohne SIGABRT-Absturz.
   * Atomare Poisoning-Sperre nach Panics.

## Konsequenzen

* **Interpreter-Stabilität:** Python-Anwendungen und Jupyter-Notebooks, die `contextra-py` nutzen, stürzen bei internen Rust-Fehlern nicht ab, sondern erhalten behandelbare Python-Exceptions.
* **Prozess-Sicherheit:** Vergiftete Datenbank-Instanzen verhindern nach einem Panic folgenschwere Folgeinkonsistenzen auf SSTable-/WAL-Ebene.
