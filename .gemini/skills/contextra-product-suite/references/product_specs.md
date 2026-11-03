# Referenz: Produktstufen

Nicht normativ. Maßgeblich: [`docs/SPEC.md`](../../../../../docs/SPEC.md), Abschnitte 1.3, 5.1 und Anhang A.

## S0: Dokumentenspeicher mit belegbarer Löschung

- **Funktionen (Anhang A):** FI-F01 Dokument-CRUD mit Mandanten und `DocRef`; FI-F02 Zeitreisen und Snapshots; FI-F03 Startprobe und Plattformstufen; FI-F04 `erase` mit Gate, Tresor, Purge und Bericht; FI-F05 Keyslots und Passphrasenwechsel; FI-F09 Restore-Guard; FI-F10 Gesundheitsstatus; FI-F11 Dateischlüssel-Vernichtung; FI-F12 `Restricted`; FI-F13 `ctx-verify`; FI-F14 Mandanten-Offboarding.
- **Crates:** `contextra-types`, `-ports`, `-sys`, `-durable-fs`, `-crypto`, `-mvcc`, `-store`, `-erasure`, `-engine`, `contextra`.
- **Ergebnis:** verschlüsselter Speicher mit gemessener Löschung. Ein Agentengedächtnis ohne Suche.

## S1: Volltextsuche als abgeleiteter Index

- **Funktionen:** FI-F06 BM25 mit Analyzer `de/1`, Phrasen über Positionen.
- **Crate:** `contextra-text`.
- **Ergebnis:** Suche mit denselben Lösch- und Messnachweisen. Terme liegen nur als `term_tag` vor.

## S2: Vektorsuche

- **Funktionen:** FI-F07 HNSW mit Surrogat, Kantenreparatur und Rebuild.
- **Crates:** `contextra-simd`, `contextra-vector`.
- **Ergebnis:** RAG-Kern. Löschung ohne Nullvektoren (D-77).

## S3: Hybridsuche und Release

- **Funktionen:** FI-F08 RRF mit `k_rrf = 60` und Tie-Break nach `doc_tag`.
- **Crates:** `contextra-rank`, `contextra-engine`, `contextra`.
- **Ergebnis:** v1.0.0, sofern G-H bestanden ist.
