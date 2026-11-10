# Generierter vs. handgeschriebener Code in contextra-wire

Dokumentation der Code-Klassifikation und Zeilenzahlen (LOC) für `contextra-wire`.

## 1. Generierter Code
Der FlatBuffers-Code wird maschinell durch den FlatBuffers-Compiler (`flatc 24.12.23`) aus `schemas/contextra.fbs` generiert und über das Modul `src/generated.rs` eingebunden und mit Lints gekapselt.

| Datei | Art | Brutto-Zeilen (`wc -l`) | Produktive LOC (`tools/loc.py`) |
|---|---|---:|---:|
| `crates/contextra-wire/src/contextra_generated.rs` | Generierter FlatBuffers Rust-Code | 1818 | 1281 |
| `crates/contextra-wire/src/generated.rs` | Modul-Wrapper & Lint-Unterdrückung | 38 | 17 |
| **Summe generiert** | | **1856** | **1298** |

## 2. Handgeschriebener Code (Produktion)
Alle übrigen Module im Crate wurden manuell entwickelt und unterliegen striktem `#![deny(unsafe_code)]` sowie Zero-Panic-Vorgaben.

| Datei | Verantwortung | Brutto-Zeilen (`wc -l`) | Produktive LOC (`tools/loc.py`) |
|---|---|---:|---:|
| `crates/contextra-wire/src/lib.rs` | Crate-Root, Fehler-Typen, Exporte | 76 | 25 |
| `crates/contextra-wire/src/adapter.rs` | `WireBuffer`-Adapter, Typzustände | 222 | 107 |
| `crates/contextra-wire/src/verification.rs` | Puffer-Validierung, Bounds-Checks | 342 | 191 |
| `crates/contextra-wire/src/jsonrpc.rs` | Gehärteter JSON-RPC 2.0 Parser | 738 | 510 |
| `crates/contextra-wire/build.rs` | Build-Skript (D-10 unberührt) | 179 | 116 |
| `crates/contextra-wire/examples/wire_standalone.rs` | Standalone-Einbettungsbeispiel | 73 | 55 |
| `crates/contextra-wire/fuzz/fuzz_targets/fuzz_flatbuffers_ipc.rs` | IPC Fuzz-Target | 55 | 32 |
| `crates/contextra-wire/fuzz/fuzz_targets/fuzz_jsonrpc.rs` | JSON-RPC Fuzz-Target | 46 | 32 |
| `crates/contextra-wire/fuzz/fuzz_targets/fuzz_verification.rs` | Verifikations-Fuzz-Target | 129 | 94 |
| **Summe handgeschriebener Code (Produktion & Tools)** | | **1860** | **1162** |

## 3. Tests & Benchmarks (Handgeschrieben)

| Datei | Art | Brutto-Zeilen (`wc -l`) |
|---|---|---:|
| `crates/contextra-wire/tests/adapter_tests.rs` | Integrationstest Adapter & Typzustand | 144 |
| `crates/contextra-wire/tests/ipc_tests.rs` | Integrationstests, Roundtrips & Fuzz-Iterationen | 1365 |
| `crates/contextra-wire/tests/schema_drift.rs` | Schema-Drift-Test & flatc-Vergleich | 341 |
| `crates/contextra-wire/tests/schema_checksum.rs` | Schema-Prüfsummentest (G-F2) | 185 |
| `crates/contextra-wire/tests/verification_robustness.rs` | Robustheits- und Bit-Flip-Tests | 292 |
| `crates/contextra-wire/benches/wire_bench.rs` | Criterion Benchmarks | 168 |
| **Summe Tests & Benchmarks** | | **2495** |

## 4. Gesamtauswertung
- **Gesamte Zeilen (`wc -l` über alle `.rs`):** 6211 Zeilen.
- **Produktions-LOC (`tools/loc.py`):** 2474 Zeilen (1298 generiert vs. 1176 handgeschrieben).
- **Verhältnis Handcode / Generierter Code (Produktion):** ~47.5% Handcode / ~52.5% generiert.
