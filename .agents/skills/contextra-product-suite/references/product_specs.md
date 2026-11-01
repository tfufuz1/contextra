# Contextra Product Stages Specifications

> **Normative Vorgabe gemäß docs/SPEC.md §1.3 und PLAN.md.**

Contextra v1 ist in vier inkrementelle, einzeln lieferbare Produktstufen unterteilt. Jede Stufe fügt eine Schicht hinzu, während die Vorstufe bereits ein vollwertiges Produkt darstellt:

## 1. Stufe S0: Gehärteter LSM-Dokumentenspeicher mit belegbarer Löschung
- **Umfang**: Transaktionaler LSM-Tree mit Löschcluster-Schlüsselraum (SPEC §9.1), Pflichtverschlüsselung (AES-256-GCM-SIV, D-05), Argon2id KDF (D-06), Mandantentrennung (D-07), Tresor (`vault`), Lösch-Saga, ReadGate (`ErasedSet`), Purge-Rewrite & Swap, Journal, E1/E2/E3 Messungen, Ed25519-Löschbericht (`ctx-erasure-report/1`), Erasure Lab und Verifier (`ctx-verify`).
- **Lieferbares Ergebnis**: Verschlüsselter Dokumenten- und Gedächtnisspeicher mit belegbarer physischer Löschung (Agentengedächtnis ohne Suche).
- **Crates**: `contextra-types`, `contextra-ports`, `contextra-sys`, `contextra-durable-fs`, `contextra-crypto`, `contextra-mvcc`, `contextra-store`, `contextra-erasure`, `contextra-engine`, `contextra`, `testkit`, `lab`, `ctx-verify`.

## 2. Stufe S1: Volltextsuche als abgeleiteter Segmentindex
- **Umfang**: BM25-Segmentindex auf unveränderlichen, AEAD-verschlüsselten Segmenten (SPEC §9.7, D-20). Kein Termklartext auf Platte (nur `term_tag`). Block-Max WAND. Dynamische Statistikaggregation. Segment-Rewrite bei Purge.
- **Lieferbares Ergebnis**: Volltextsuche mit denselben kryptografischen Lösch- und Messnachweisen.
- **Crate**: `contextra-text`.

## 3. Stufe S2: Vektorsuche mit löschbarer Persistenz
- **Umfang**: HNSW-Vektorsuche für f32-Einbettungen mit SIMD-Beschleunigung (`contextra-simd`). In-Memory Slot-Zeroing (`zero_slot`). Verschlüsselte Snapshot-Chunks (SPEC §9.8). Vollständiger Batch-Rebuild ab Tombstone-Schwelle.
- **Lieferbares Ergebnis**: RAG-Vektorkern mit garantierter physischer Löschbarkeit.
- **Crates**: `contextra-simd`, `contextra-vector`.

## 4. Stufe S3: Hybridsuche & Release Candidate v1.0.0
- **Umfang**: Reciprocal Rank Fusion (RRF, `contextra-rank`) mit deterministischem Tie-Break nach `doc_tag`. End-to-End Orchestrierung in `contextra-engine`. Öffentliche API-Politur in `contextra`. Verifikation aller Release-Gates G-A bis G-H und G0.
- **Lieferbares Ergebnis**: v1.0.0 Gesamtrelease.
