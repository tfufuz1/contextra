# Verifikations-Index aller 15 Prompts und Spezifikationen

Dieses Dokument indexiert und verlinkt alle Verifikationstabellen der 15 Arbeits- und Umsetzungspakete für Contextra.

## Übersicht der Verifikationstabellen

| Nr. | Prompt- / Modul-ID | Dokument | Gegenstand |
|---|---|---|---|
| 01 | P-01 | [P-01.md](P-01.md) | `contextra-types` Kern-Typen & Unveränderlichkeit |
| 02 | P-02 | [P-02.md](P-02.md) | `contextra-ports` Trait-Definitionen & Schnittstellen |
| 03 | P-03 | [P-03.md](P-03.md) | `contextra-sys` Unsafe-Betriebssystem-Insel |
| 04 | P-04 | [P-04.md](P-04.md) | `contextra-simd` Distanzkerne & Vektorisierung |
| 05 | P-05 | [P-05.md](P-05.md) | `contextra-durable-fs` Crash-Konsistenz & VFS |
| 06 | P-06 | [P-06.md](P-06.md) | `contextra-crypto` Schlüsselhierarchie & AEAD |
| 07 | P-07 | [P-07.md](P-07.md) | `contextra-mvcc` MVCC-Snapshots & SSI |
| 08 | P-08 | [P-08.md](P-08.md) | `contextra-store` LSM-Engine & WAL-Durability |
| 09 | P-09 | [P-09.md](P-09.md) | `contextra-text` BM25-Index & Tokenizer |
| 10 | P-10 | [P-10.md](P-10.md) | `contextra-vector` HNSW & Vektor-Indizierung |
| 11 | P-11 | [P-11.md](P-11.md) | `contextra-rank` RRF Fusion & Re-Ranking |
| 12 | P-12 | [P-12.md](P-12.md) | `contextra-erasure` DSGVO Art. 17 Löschkern |
| 13 | P-13 | [P-13.md](P-13.md) | `contextra-engine` RAG-Orchestrierung |
| 14 | P-14 | [P-14.md](P-14.md) | `contextra` Öffentliche Fassade |
| 15 | K-C  | [K-C.md](K-C.md)   | Mess- und Audit-Rückgrat (Testkit, Harness, CI) |

## Globale Prüfregeln

- Alle Vorgabe-IDs (`PO-`, `SY-`, `DF-`, `MV-`, `ST-`, `TY-` etc.) sind lückenlos verlinkt und maschinell geprüft.
- CI-Gate: `scripts/check_spec_coverage.py` erzwingt die Existenz aller Tabelleneinträge und zugehöriger Tests.
