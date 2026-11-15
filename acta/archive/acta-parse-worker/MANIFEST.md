# Extraktion fuer `acta-parse-worker`

**Aufgabe:** Sandbox-Prozess fuer Parser; keine Schluessel, kein Netz  
**Level:** L9 · **Budget:** 600 LOC · **Spezifikation:** docs/SPEC.md Kap. 9.1, 13.2  
**Interne Abhaengigkeiten (Soll):** acta-ingest, acta-parse, acta-sys  
**Umfang des Archivs:** 11 Dateien, 2252 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `sandbox` | Sandbox-Grenzen/Abuse-Matrix als Orakel; Umsetzung per Landlock+seccomp statt WASM (13.2) |
| `wire` | Rahmenprotokoll u32 BE <= 64 MiB (9.1 Stufe 2) |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| serde | 1 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| async | 71 | tokio/async (G-NOTOKIO: Kern synchron) |
| zeit_zufall | 2 | Zeit/Zufall direkt (G3: nur ueber Ports Clock/CryptoRng) |
| alt_spec | 18 | Alt-Referenzen (D-/ER-/INV-Kennungen): durch SPEC-Verweise ersetzen |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `anyhow`, `bytes`, `flatbuffers`, `tracing`, `wasmtime` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-parse-worker/`) | LOC | Befunde |
|---|---:|---|
| `sandbox/src/capabilities.rs` | 184 | alt_spec=3 |
| `sandbox/src/error.rs` | 39 |  |
| `sandbox/src/executor/mod.rs` | 161 | async=4, zeit_zufall=2 |
| `sandbox/src/executor/runner.rs` | 152 | async=6 |
| `sandbox/src/executor/state.rs` | 93 | async=1, alt_spec=3 |
| `sandbox/src/lib.rs` | 34 |  |
| `sandbox/src/output.rs` | 30 |  |
| `sandbox/tests/abuse_test_matrix.rs` | 916 | async=60 |
| `wire/src/adapter.rs` | 223 |  |
| `wire/src/lib.rs` | 77 | serde=1 |
| `wire/src/verification.rs` | 343 | alt_spec=12 |
