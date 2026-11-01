---
name: contextra-product-suite
description: >-
  Use this skill to guide the testing and release validation of Contextra's product stages (SPEC §1.3):
  S0 Hardened Storage & Erasure, S1 Fulltext Search, S2 Vector Search, and S3 Hybrid Retrieval v1.0.0.
---

# Contextra Product Suite – Produktstufen & Release-Validierung

Dieser Skill unterstützt Entwickler und Agenten bei der inkrementellen Validierung und Paketierung der Contextra-Produktstufen nach [docs/SPEC.md §1.3](file:///home/freddy/Projekte/contextra/docs/SPEC.md) und [PLAN.md](file:///home/freddy/Projekte/contextra/PLAN.md).

---

## 1. Die vier Produktstufen (SPEC §1.3)

Jede Stufe ist in sich geschlossen und einzeln lieferbar:

| Stufe | Inhalt | Lieferbares Ergebnis |
|---|---|---|
| **S0** | Gehärteter LSM-Speicher mit Löschcluster-Schlüsselraum, Pflichtverschlüsselung, Tresor, Gate, Saga, Purge, Journal, Bericht, Lab, Verifier. Dokumente, Zeitreisen, Mandanten. | Verschlüsselter Dokumentenspeicher mit belegbarer Löschung. Vollwertiges Produkt (Agentengedächtnis ohne Suche). |
| **S1** | Textsuche als abgeleiteter, segmentierter, AEAD-verschlüsselter BM25-Index | Volltextsuche mit denselben Lösch- und Messnachweisen |
| **S2** | Vektorsuche (HNSW) mit Slot-Zeroing, verschlüsselten Snapshot-Chunks und Batch-Rebuild | RAG-Vektorkern |
| **S3** | Hybridsuche (RRF), API-Politur, Dokumentation, Release-Candidate | v1.0.0 Komplettsystem |

---

## 2. Gate-Prüfpfade (PLAN.md)

1. **Gate G-C (Speicherkern)**: LSM-Tree gehärtet, WAL-Integrität, Cancel-Safety, Vfs-Migration, Zero-Panic, Crash-Matrix (T-CRASH-1..4).
2. **Gate G-D (Löschung Dokumente)**: Saga, ReadGate (`ErasedSet`), Purge-Barrier, SSTable-Rewrite/Swap, Snapshot-Widerruf.
3. **Gate G-E (Indizes)**: Segment-Rewrite BM25 (WP-4.1), HNSW Slot-Zeroing & Rebuild (WP-4.2), Blockcache-Eviction.
4. **Gate G-F (Nachweis & Bericht)**: Erasure Lab, signierter Bericht `ctx-erasure-report/1`, Verifier `ctx-verify`.
5. **Gate G-G (Härtung)**: Fuzzing, Concurrency, Mutationstests, Unsafe-Audit.
6. **Gate G-H (Freigabe)**: Externes Review, Dokumentation, v1.0.0 Release Candidate.

---

## 3. Test-Kommandos

```bash
# Speicherkern testen (Stufe S0)
cargo test -p contextra-store -p contextra-mvcc -p contextra-crypto

# Textsuche testen (Stufe S1)
cargo test -p contextra-text

# Vektorsuche testen (Stufe S2)
cargo test -p contextra-vector -p contextra-simd

# Hybridsuche & Gesamtsystem testen (Stufe S3)
cargo test -p contextra-rank -p contextra-engine -p contextra
```
