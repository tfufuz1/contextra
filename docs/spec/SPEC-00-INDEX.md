# SPEC-00 — Contextra Spezifikationsindex

**Version 2.0 · November 2026 · Status: Final**

> Diese Spezifikation beschreibt den normativen Soll-Zustand von Contextra, einem LLM-nativen, versionierten Storage-System in Rust. Alle Performance-Zahlen sind Designziele; Validierung durch Prototypen.

---

## Dokumentenstruktur

| Dokument | Inhalt | Fokus |
|---|---|---|
| **SPEC-00** (dieses Dokument) | Index, Querverweise, Glossar | Navigation |
| [**SPEC-01** — Architektur](SPEC-01-ARCHITECTURE.md) | Vision, Prinzipien, Schichten, Crate-Layout, Entscheidungsregister | Warum und wie |
| [**SPEC-02** — Datenmodell](SPEC-02-DATA-MODEL.md) | Cells, Facets, Zoom, Namespaces, Handles, Bitemporalität, Chunking, Sequenzen | Was wird gespeichert |
| [**SPEC-03** — Storage Engine](SPEC-03-STORAGE-ENGINE.md) | Packs, Extents, Hot-Head/Cold-Tail, Locator, Kompression, Verschlüsselung | Wie es auf Disk liegt |
| [**SPEC-04** — Versionierung](SPEC-04-VERSIONING.md) | Prolly-Tree, Delta-Log, Commits, Branches, Merge, Idempotenz, Zeitreise | Wie Zustand verwaltet wird |
| [**SPEC-05** — Indizes](SPEC-05-INDEXES.md) | Vektor (Vamana), FTS (BM25), Graph (CSR/PPR), Temporal, Token-Summen | Wie gesucht wird |
| [**SPEC-06** — Context Engine](SPEC-06-CONTEXT-ENGINE.md) | Recall-Pipeline, Zoom-Knapsack, Fusion, Ranking, Work Queue, Lineage | Wie Kontext optimiert wird |
| [**SPEC-07** — Durability](SPEC-07-DURABILITY.md) | WAL, Manifest, Recovery, GC, Nebenläufigkeit, Tiering, Replikation, Sicherheit | Wie Daten geschützt werden |
| [**SPEC-08** — API](SPEC-08-API.md) | Rust-API, MCP, gRPC, SQL, CLI, Fehlerkatalog | Wie man es benutzt |
| [**SPEC-09** — Betrieb](SPEC-09-OPERATIONS.md) | Ingest, Konfiguration, Tests, Performance, Monitoring, Roadmap, Risiken | Wie man es betreibt |

---

## Lesepfade

### Für Architektur-Reviews
SPEC-01 → SPEC-02 → SPEC-03 → SPEC-04

### Für RAG/Retrieval-Fokus
SPEC-02 (§4–5 Zoom) → SPEC-05 → SPEC-06

### Für Storage/Durability-Fokus
SPEC-03 → SPEC-04 (§1 Prolly-Tree) → SPEC-07

### Für API-Integration
SPEC-08 → SPEC-02 (§1–2 Namespace/Cell) → SPEC-06 (§1–2 Recall)

### Für Betrieb und Testing
SPEC-09 → SPEC-07 (§3 Recovery, §4 GC)

---

## Geschlossene offene Fragen

Die Originalspezifikation (SPEC.md v1.0) hatte 5 explizite offene Fragen. Alle sind entschieden:

| # | Frage | Entscheidung | Dokument |
|---|---|---|---|
| 1 | Konsistenzmodell der Zugriffsstatistik | **Eventual Consistency** — Soft-Signal, gesampelt (10 %), Sum-CRDT bei Merge | SPEC-02 §9.1 |
| 2 | Handle-Stabilität über Branches | **Global pro Repository** — gemeinsamer Zähler im Manifest, Union-Merge | SPEC-02 §5.2 |
| 3 | WASM-Merge-Funktionen: Overhead/Determinismus | **wasmtime + Fuel-Limits (10M)**, kein WASI, kein Float, Fallback auf ConflictCell | SPEC-04 §6.3 |
| 4 | Eigener FTS vs. Tantivy | **Eigener FTS** — Tantivy-Segmente nicht content-addressed, kein RootSet-Anker, kein Sharing | SPEC-01 §7 (C-17) |
| 5 | Deployment-Modell Priorität | **Embedded-first** — Library ist Produkt, Server ab Phase 2 als dünner Wrapper | SPEC-01 §1.2 |

---

## Entscheidungsregister (Gesamtübersicht)

| ID | Entscheidung | Dokument |
|---|---|---|
| C-01 | Prolly-Bäume als Versionierungsgrundlage | SPEC-01 §7 |
| C-02 | Schichten L0–L6, Abhängigkeiten nur nach unten | SPEC-01 §4 |
| C-03 | Packs als Quelle der Wahrheit, Index rekonstruierbar | SPEC-01 §7 |
| C-04 | Zwei Ebenen: Transaktion und Commit | SPEC-01 §7 |
| C-05 | Keyed ObjectIds bei Verschlüsselung | SPEC-03 §8.3 |
| C-06 | Sitzung = Branch mit Overlay | SPEC-04 §5 |
| C-07 | Idempotenzschlüssel + LLM-lesbarer Fehlerkatalog | SPEC-04 §8, SPEC-08 §6 |
| C-08 | SQL über SQLite Virtual Tables | SPEC-08 §4 |
| C-09 | Indizes im RootSet verankert | SPEC-05 §6 |
| C-10 | Cell als Primitive, Hot-Head/Cold-Tail | SPEC-02 §2, SPEC-03 §2.2 |
| C-11 | Zoom-Knapsack statt Top-k | SPEC-06 §3.4 |
| C-12 | Work Queue statt Engine-internes LLM | SPEC-06 §7.4 |
| C-13 | GC mit SATB+Remark | SPEC-07 §4 |
| C-14 | Ein Repo = eine Instanz, Single-Process | SPEC-01 §1.2 |
| C-15 | Trust-Labels mit Minimum-Vererbung | SPEC-07 §9.1 |
| C-16 | Gelerntes Layout mit 5% Exploration | SPEC-03 §4 |
| C-17 | Eigener FTS statt Tantivy | SPEC-01 §7 |
| C-18 | Handles global pro Repository | SPEC-02 §5.2 |
| C-19 | Eventual Consistency für Zugriffsstatistik | SPEC-02 §9.1 |
| C-20 | WASM-Merge mit Fuel-Limits | SPEC-04 §6.3 |
| C-21 | Embedded-first, Server ab Phase 2 | SPEC-01 §1.2 |

---

## Glossar

| Begriff | Bedeutung | Definiert in |
|---|---|---|
| **Cell** | Wissenseinheit mit Facetten (Inhalt, Tokens, Embeddings, Zoom, Kanten, Zeit, Trust) | SPEC-02 §2 |
| **CellId** | Logische, versionsübergreifend stabile Identität einer Cell (NsId + CellKey) | SPEC-02 §2.1 |
| **CellKind** | Typ einer Cell (Episode, Fact, Doc, etc.) — exhaustive u8-Enumeration | SPEC-02 §2.3 |
| **ChunkId** | Content-addressed Hash (BLAKE3-256) des kanonischen Cell-Inhalts | SPEC-02 §2.1 |
| **Facet / Lane** | Physisch getrennt gespeicherter Teil einer Cell (meta, emb.q, raw, etc.) | SPEC-02 §3 |
| **Zoom-Leiter** | Auflösungsstufen einer Cell (L0 voll … Ln Titel) mit bekannten Token-Kosten | SPEC-02 §4 |
| **Handle** | Kurzer, token-sparsamer Alias einer Cell (4–6 Crockford-Base32-Zeichen) | SPEC-02 §5 |
| **Namespace** | Sammlung von Cells mit gemeinsamer Policy | SPEC-02 §1 |
| **Chunk** | Unveränderliches, content-addressed Objekt (BLAKE3) | SPEC-03 §1.1 |
| **Extent** | Physische I/O-Einheit (256 KiB–1 MiB) mit Hot Head und Cold Tail | SPEC-03 §1.2 |
| **Pack** | Datei aus Extents (.cxp, 64 MiB–1 GiB) | SPEC-03 §1.3 |
| **Locator-Index** | Abbildung ChunkId → physischer Ort | SPEC-03 §3 |
| **Prolly-Tree** | Probabilistischer, history-unabhängiger, content-addressed B-Baum | SPEC-04 §1 |
| **Delta-Log / MemDelta** | LSM-artiger In-Memory-Schreibpuffer vor dem Prolly-Tree-Fold | SPEC-04 §1.4 |
| **Fold** | Batch-Überführung des MemDelta in neue Prolly-Knoten und Packs | SPEC-04 §1.5 |
| **Commit** | Versionierungspunkt mit Parents, Changes, Intent, optionalem RootSet | SPEC-04 §2 |
| **RootSet** | Wurzelobjekt des gesamten versionierten Zustands inkl. aller Indizes | SPEC-04 §2 |
| **HLC** | Hybrid Logical Clock — 48 Bit physisch + 16 Bit logisch | SPEC-02 §6.1 |
| **ConflictCell** | Versionierte Darstellung eines Merge-Konflikts als abfragbare Cell | SPEC-04 §6.4 |
| **Zoom-Knapsack** | Multiple-Choice-Knapsack zur Auswahl der Zoom-Stufe je Treffer unter Budget | SPEC-06 §3.4 |
| **Derivation Cache** | Memoisierung abgeleiteter Facetten pro (Quellhash, Producer) | SPEC-06 §7.1 |
| **Work Queue** | At-least-once-Aufgabenwarteschlange für externe Worker | SPEC-06 §7.4 |
| **Access-Trace** | Gesampelter Strom von Lesezugriffen für Layout und Ranking | SPEC-03 §4.2 |
| **Salience** | Berechnete Wichtigkeit einer Cell | SPEC-02 §9.2 |
| **Progressive Disclosure** | Muster: kompakte Handles → selektives Expand | SPEC-06 §4 |
| **Prefix-Digest** | Hash der stabilen Präfix-Token für KV-Cache-Kompatibilität | SPEC-06 §3.5 |
| **RRF** | Reciprocal Rank Fusion — robuste Score-Kombination | SPEC-06 §3.2 |
| **MMR** | Maximal Marginal Relevance — Diversitätsoptimierung | SPEC-06 §3.4 |
| **SATB** | Snapshot-at-the-Beginning — GC-Protokoll für concurrent Writes | SPEC-07 §4.1 |
| **CSR** | Compressed Sparse Row — Graph-Adjazenz für schnelle Traversal | SPEC-05 §3.2 |
| **FST** | Finite State Transducer — platzspartes Term-Dictionary | SPEC-05 §2.4 |
| **Vamana/DiskANN** | Graph-basierter ANN-Index, Disk-optimiert | SPEC-05 §1.3 |
| **Capability** | Macaroon-artiges, attenuierbares Autorisierungs-Token | SPEC-07 §9.2 |
| **TrustLabel** | Vertrauensstufe einer Cell (System > User > ToolVerified > ToolUnverified > External) | SPEC-07 §9.1 |
| **Evidence Sufficiency** | Schätzung, ob Retrieval-Ergebnis ausreichend ist | SPEC-06 §6 |
| **Provenance** | Herkunftsmetadaten einer Cell (Autor, Quelle, Modell, Confidence) | SPEC-02 §2.6 |

---

*Letzte Aktualisierung: November 2026. Alle Fragen geschlossen, alle Entscheidungen final.*
