# ADR-084: FlatBuffers Schema Drift Gate (IP-16) & Dead Types Recommendation

* **Status:** ✅ Final
* **Datum:** 2026-09-17
* **Target Path:** xtask/src/check_flatbuffers_drift.rs, schemas/memfuse.fbs, crates/memfuse-core-ipc-gen/src/memfuse_generated.rs
* **Kontext / Auslöser:** Implementierung des CI-Drift-Gates IP-16 zwischen `schemas/memfuse.fbs` und `crates/memfuse-core-ipc-gen/src/memfuse_generated.rs`. Evaluierung der beiden FlatBuffers-Typen `VectorIndexUpdate` und `Embedding` bezüglich workspace-weiter Konsumption.

## Entscheidung
1. **CI Drift-Gate & Developer Subcommands:**
   - Implementierung von `cargo xtask check-flatbuffers-drift` zur Erkennung von Abweichungen zwischen Schema und generiertem Rust-Code.
   - Implementierung von `cargo xtask regenerate-flatbuffers` zur direkten Aktualisierung des generierten Rust-Codes im Quellbaum.
   - Das Gate prüft das Vorhandensein des `flatc`-Compilers. Fehlt `flatc`, liefert das Gate eine klare `Err`-Meldung ohne Rust-Panic oder Silent-Skip.
2. **Umgang mit toten FlatBuffers-Typen (`VectorIndexUpdate` und `Embedding`):**
   - Ein Codebase-Audit ergab, dass `VectorIndexUpdate` und `Embedding` im generierten IPC-Code von keinen externen Workspace-Crates konsumiert werden.
   - **Empfehlung:** Die Typen werden vorerst NICHT aus `schemas/memfuse.fbs` entfernt, um Breaking Changes an der IPC-Schnittstellenspezifikation vor dem formellen v1.0 Interface-Freeze zu vermeiden. Eine finale Entfernung oder Anbindung an Konsumenten erfolgt in einer gesonderten IPC-Grooming-Phase.

## Begründung
- **Drift-Sicherheit:** Garantiert, dass Schema-Änderungen an `schemas/memfuse.fbs` nicht unbemerkt zu Drift im generierten Rust-Code führen.
- **Zero-Panic & Fail-Closed:** Stellt sicher, dass fehlende Build-Tools (`flatc`) in CI-Subcommands sauber abgefangen und gemeldet werden.
- **API-Stabilität:** Verhindert voreilige Breaking Changes an der IPC-Spezifikation vor der v1.0-Stabilisierung.

## Konsequenzen
- Entwickler können `cargo xtask regenerate-flatbuffers` nutzen, um den generierten Rust-Code nach Schema-Änderungen zu aktualisieren.
- Das CI-Gate `check-flatbuffers-drift` kann in Merge-Gates eingebunden werden.
