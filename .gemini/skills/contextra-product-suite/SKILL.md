---
name: contextra-product-suite
description: >-
  Use this skill to guide testing and release validation of the Contextra product stages (SPEC 1.3): S0 storage and erasure,
  S1 full-text search, S2 vector search, S3 hybrid search and release, including the gates G-A to G-H and G0.
---

# Produktstufen und Release-Gates

Maßgeblich: [`docs/SPEC.md`](../../../../docs/SPEC.md), Abschnitte 1.3, 15.3 und 15.4.

## 1. Produktstufen (SPEC 1.3)

Jede Stufe ist einzeln lieferbar.

| Stufe | Inhalt | Ergebnis |
|---|---|---|
| **S0** | Gehärteter LSM-Speicher, Löschcluster-Schlüsselraum, Pflichtverschlüsselung, Tresor, Gate, Saga, Purge, Journal, Bericht, Lab, Verifier. Dokumente, Zeitreisen, Mandanten | verschlüsselter Dokumentenspeicher mit belegbarer Löschung |
| **S1** | Textsuche als abgeleiteter, segmentierter, verschlüsselter Index | Volltextsuche mit denselben Nachweisen |
| **S2** | Vektorsuche (HNSW) mit Surrogat, Kantenreparatur, verschlüsseltem Snapshot, deterministischem Rebuild | RAG-Kern |
| **S3** | Hybridsuche (RRF), API-Politur, Dokumentation, Release-Candidate | v1.0.0 |

**Release-Regel (D-59).** v1.0.0 erscheint erst, wenn S0 bis S3 Gate G-H bestehen. Gibt es bis G-G keine tragfähige Vektor- oder Hybridschicht, greift die Cut-Linie: v1.0.0 = S0 und S1, S2 und S3 folgen als v1.1. Eine halb fertige Löschschicht gibt es in keinem Release.

## 2. Gates (SPEC 15.3)

| Gate | Bedingung |
|---|---|
| G-A | `check --workspace --all-targets --locked` und `fmt --check` grün |
| G-B | Rückbau nach D-84 vollständig; Features und Tokio nach D-54; `deny` ohne offene Ausnahmen; ein `Vfs` |
| G0 | Speicherentscheid mit Messprotokoll (SPEC 11.9) |
| G-C | Kern-Crates ohne Zero-Panic-Verstöße; Clippy mit `-D warnings`; Pflichtverschlüsselung |
| G-D | Löschkern S0: T-ISO, T-GATE, T-VAULT, T-KEYCOV, T-FK, T-CRASH, T-RAW, T-PROOF, T-RESTORE grün; Lab MK1 bis MK3; Mutationstests ohne Überlebende; BM-ERASE-LAT, BM-ERASE-O1, BM-ERASE-WA erfüllt |
| G-E | S1 Textindex |
| G-F | Keyslots, HEADER, Plattformprobe |
| G-G | S2 Vektorsuche |
| G-H | Release S0 bis S3 (oder Cut-Linie); alle T- und BM-IDs; SBOM; signierte Artefakte |
| G-LEGAL | extern, blockiert nur Löschaussagen über 6.4 hinaus |

Reihenfolge: A, B, G0, C, D, F, E, G, H. G-LEGAL läuft parallel.

## 3. Prüfbefehle je Stufe

```bash
# S0: Speicherkern
cargo test -p contextra-store -p contextra-mvcc -p contextra-crypto --locked
# S1: Textsuche
cargo test -p contextra-text --locked
# S2: Vektorsuche
cargo test -p contextra-simd -p contextra-vector --locked
# S3: Hybridsuche
cargo test -p contextra-rank -p contextra-engine --locked
```

Ein Befehl gilt als bestanden, wenn er grün ist und das Protokoll im Evidenzbündel liegt. Nicht vorhandene Crates (z. B. `contextra-erasure` vor WP-3.1) werden als offen gemeldet, nicht übergangen.

Details: [`references/product_specs.md`](references/product_specs.md).
