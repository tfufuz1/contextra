# Extraktion fuer `acta-ingest`

**Aufgabe:** Chunking, Normalisierung, Ressourcenlimits, Worker-Protokoll (ohne Parser-Abhaengigkeiten)  
**Level:** L1 · **Budget:** 3000 LOC · **Spezifikation:** docs/SPEC.md Kap. 9.1  
**Interne Abhaengigkeiten (Soll):** acta-base  
**Umfang des Archivs:** 44 Dateien, 11235 Zeilen (inkl. Tests)  

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
| `db/src/chunker/blocks.rs` | 230 |  |
| `db/src/chunker/config.rs` | 38 |  |
| `db/src/chunker/estimate.rs` | 93 | float=3 |
| `db/src/chunker/mod.rs` | 400 | hub=1 |
| `db/src/chunker/sections.rs` | 27 |  |
| `db/src/chunker/split.rs` | 250 |  |
| `db/src/chunker/unit_tests.rs` | 412 | float=2, panic=9 |
| `db/src/chunker/windows.rs` | 91 |  |
| `ingest/src/canonical.rs` | 324 | serde=1 |
| `ingest/src/chunking/common.rs` | 241 | float=3 |
| `ingest/src/chunking/config.rs` | 192 | serde=1 |
| `ingest/src/chunking/email/mod.rs` | 474 |  |
| `ingest/src/chunking/fallback.rs` | 79 | panic=1 |
| `ingest/src/chunking/meta.rs` | 167 | panic=1 |
| `ingest/src/chunking/mod.rs` | 129 |  |
| `ingest/src/chunking/paged/mod.rs` | 53 |  |
| `ingest/src/chunking/paged/page.rs` | 811 |  |
| `ingest/src/chunking/prose/markdown_parse.rs` | 857 | panic=3 |
| `ingest/src/chunking/prose/merge.rs` | 164 |  |
| `ingest/src/chunking/prose/mod.rs` | 517 |  |
| `ingest/src/chunking/prose/sentence.rs` | 156 | panic=9 |
| `ingest/src/chunking/prose/split.rs` | 420 |  |
| `ingest/src/chunking/structured/mod.rs` | 541 |  |
| `ingest/src/chunking/stubs.rs` | 81 |  |
| `ingest/src/chunking/tabular/helpers.rs` | 122 |  |
| `ingest/src/chunking/tabular/mod.rs` | 781 |  |
| `ingest/src/chunking/types.rs` | 72 | float=1, serde=1 |
| `ingest/src/error.rs` | 107 | serde=1 |
| `ingest/src/fuzz_support.rs` | 38 |  |
| `ingest/src/ir.rs` | 254 | serde=1 |
| `ingest/src/lib.rs` | 40 |  |
| `ingest/src/limits.rs` | 95 | serde=1 |
| `ingest/src/pipeline.rs` | 178 | panic=1 |
| `ingest/tests/chunk_email_tests.rs` | 292 | panic=11 |
| `ingest/tests/chunk_paged_acceptance.rs` | 519 | panic=13 |
| `ingest/tests/chunk_paged_proptest.rs` | 174 | panic=7 |
| `ingest/tests/chunk_prose_edge_cases.rs` | 201 | panic=8 |
| `ingest/tests/chunk_prose_parity.rs` | 101 | panic=1 |
| `ingest/tests/chunk_prose_proptest.rs` | 126 | panic=2 |
| `ingest/tests/chunk_structured_tests.rs` | 226 | panic=6 |
| `ingest/tests/chunk_tabular_tests.rs` | 583 | panic=38 |
| `ingest/tests/contract_tests.rs` | 370 | panic=7 |
| `ingest/tests/ingest_property_tests.rs` | 132 | panic=4 |
| `ingest/tests/inv_ingest_bounded_tests.rs` | 77 |  |
