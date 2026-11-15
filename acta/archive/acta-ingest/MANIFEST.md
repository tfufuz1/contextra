# Extraktion fuer `acta-ingest`

**Aufgabe:** Chunking, Normalisierung, Ressourcenlimits, Worker-Protokoll (ohne Parser-Abhaengigkeiten)  
**Level:** L1 · **Budget:** 3000 LOC · **Spezifikation:** docs/SPEC.md Kap. 9.1  
**Interne Abhaengigkeiten (Soll):** acta-base  
**Umfang des Archivs:** 44 Dateien, 11191 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `ingest` | Chunking (9.1.1): auf Woerter nach UAX #29 und chunk_cfg v1 umstellen |
| `db` | Zweiter Chunker; als Vergleichsorakel, Duplikat nach Abgleich verwerfen |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 9 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 6 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| panic | 121 | unwrap/expect/panic (G-LINT) |
| hub | 1 | Hub-Import ex types/ports (HR-2/HR-3) |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `blake3` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-ingest/`) | LOC | Befunde |
|---|---:|---|
| `db/src/chunker/blocks.rs` | 229 |  |
| `db/src/chunker/config.rs` | 37 |  |
| `db/src/chunker/estimate.rs` | 92 | float=3 |
| `db/src/chunker/mod.rs` | 399 |  |
| `db/src/chunker/sections.rs` | 26 |  |
| `db/src/chunker/split.rs` | 249 |  |
| `db/src/chunker/unit_tests.rs` | 411 | panic=9, float=2 |
| `db/src/chunker/windows.rs` | 90 |  |
| `ingest/src/canonical.rs` | 323 | serde=1 |
| `ingest/src/chunking/common.rs` | 240 | float=3 |
| `ingest/src/chunking/config.rs` | 191 | serde=1 |
| `ingest/src/chunking/email/mod.rs` | 473 |  |
| `ingest/src/chunking/fallback.rs` | 78 | panic=1 |
| `ingest/src/chunking/meta.rs` | 166 | panic=1 |
| `ingest/src/chunking/mod.rs` | 128 |  |
| `ingest/src/chunking/paged/mod.rs` | 52 |  |
| `ingest/src/chunking/paged/page.rs` | 810 |  |
| `ingest/src/chunking/prose/markdown_parse.rs` | 856 | panic=3 |
| `ingest/src/chunking/prose/merge.rs` | 163 |  |
| `ingest/src/chunking/prose/mod.rs` | 516 |  |
| `ingest/src/chunking/prose/sentence.rs` | 155 | panic=9 |
| `ingest/src/chunking/prose/split.rs` | 419 |  |
| `ingest/src/chunking/structured/mod.rs` | 540 |  |
| `ingest/src/chunking/stubs.rs` | 80 |  |
| `ingest/src/chunking/tabular/helpers.rs` | 121 |  |
| `ingest/src/chunking/tabular/mod.rs` | 780 |  |
| `ingest/src/chunking/types.rs` | 71 | float=1, serde=1 |
| `ingest/src/error.rs` | 106 | serde=1 |
| `ingest/src/fuzz_support.rs` | 37 |  |
| `ingest/src/ir.rs` | 253 | serde=1 |
| `ingest/src/lib.rs` | 39 |  |
| `ingest/src/limits.rs` | 94 | serde=1 |
| `ingest/src/pipeline.rs` | 177 | panic=1 |
| `ingest/tests/chunk_email_tests.rs` | 291 | panic=11 |
| `ingest/tests/chunk_paged_acceptance.rs` | 518 | panic=13 |
| `ingest/tests/chunk_paged_proptest.rs` | 173 | panic=7 |
| `ingest/tests/chunk_prose_edge_cases.rs` | 200 | panic=8 |
| `ingest/tests/chunk_prose_parity.rs` | 100 | panic=1 |
| `ingest/tests/chunk_prose_proptest.rs` | 125 | panic=2 |
| `ingest/tests/chunk_structured_tests.rs` | 225 | panic=6 |
| `ingest/tests/chunk_tabular_tests.rs` | 582 | panic=38 |
| `ingest/tests/contract_tests.rs` | 369 | panic=7 |
| `ingest/tests/ingest_property_tests.rs` | 131 | panic=4 |
| `ingest/tests/inv_ingest_bounded_tests.rs` | 76 |  |
