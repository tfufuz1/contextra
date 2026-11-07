# AGENTS.md — `contextra-py`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-py` stellt die offizielle Python-Anbindung (PyO3) für das Contextra-Baukastensystem bereit (Ring 4).
Es übersetzt die asynchronen Rust-APIs in synchrone und asynchrone Python-Aufrufe, konvertiert Datentypen verlustfrei (NumPy Vektoren, Dicts, Listen) und kapselt die FFI-Sicherheitsgrenze.
Als eigenständiger Adapter-Baustein fängt er Panics an der Sprachgrenze ab und übersetzt Domain-Fehler in idiomatische Python-Exceptions.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 4 (Sprach-Bindings & FFI-Boundary)
- **Klasse:** Erweiterung `[E]`
- **Reifegrad:** `beta`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-wire`, `contextra-db` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **GIL-Release-Disziplin (AGT-PY-001):** Blockierende oder rechenintensive Rust-Aufrufe geben den Python Global Interpreter Lock zwingend über `py.allow_threads` frei.
- **FFI Panic-Sicherheit (`run_blocking_ffi`):** Unwinding-Panics im Rust-Code werden an der FFI-Grenze abgefangen, das Instanz-Objekt wird als vergiftet markiert und eine saubere Python-Exception geworfen.
- **Typisiertes Error-Mapping:** Rust `ContextraError` wird 1:1 auf dedizierte Python-Fehlerklassen abgebildet (`TenantError`, `NotFoundError`, `IntegrityError`, etc.).
- **NumPy Zero-Copy & Validierung:** NumPy-Arrays müssen 1D, `float32` und C-kontinuierlich sein; NaN- und Inf-Werte werden strikt abgewiesen.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()` und `expect()` im produktiven Binding-Code.

## 4. Verbotene Muster
- Halten des Python GIL während asynchroner `.await`- oder schwerer Berechnungsphasen.
- Rohe Pointer-Arithmetik oder `unsafe` außerhalb der geprüften PyO3-Makros.
- Verwendung von `contextra-core` (statdessen direkte Nutzung von `contextra-types` und `contextra-ports`).
- Unkontrolliertes Durchreichen unvollständiger FFI-Typen.

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Exportiert:** Python-Modul `contextra` (`PyContextra`, `PyCollection`, FlatBuffers-Suchmethoden).
- **Konsumiert:** `contextra-types`, `contextra-wire`, `contextra-db`.

## 6. Tests, Sabotage & Qualitätssicherung
- **FFI-Panic-Abfangtests:** Prüfung des Verhaltens bei künstlich injizierten Panics (via `test-hooks`).
- **Python Integrationstests:** Pytest-Suite zur Validierung von GIL-Freigabe, Multithreading und NumPy-Konvertierungen.
- **Sabotage-Testing:** Injektion korrumpierter Vektoren (NaN, nicht-kontinuierlich) zur Bestätigung der Fail-Closed Validierung.

## 7. Offene Punkte & Technische Schulden
- DEBT-04: Übergang von direkter `contextra-db` Kopplung hin zu generischen Ports.
