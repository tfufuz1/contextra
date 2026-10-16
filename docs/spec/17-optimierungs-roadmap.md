---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "17"
---
## 17. Priorisierte Optimierungs-Roadmap (Opus-Analyse)

Die Reihenfolge der Stufen ist **verbindlich**: Stufe 0 blockiert bzw. gefährdet den Betrieb, Stufe 1 ist der
größte Hebel für Latenz/Durchsatz, Stufe 2 betrifft Speicherverbrauch und Struktur, Stufe 3 ist Governance.

### Stufe 0 — Korrektheit und Betriebssicherheit (ZUERST)

| ID | Maßnahme | Problem | Aufwand |
|---|---|---|---|
| **0.1** | **WAL-Replay-Panic entschärfen** | Dateigröße separat von `mmap.len()`, Direktindizierung → Panic bei veränderter Datei, Crash-Loop möglich | Gering |
| **0.2** | **Bandit-Dimensionsprüfung** | `debug_assert` statt `Result` bei Dimensionsmismatch → stilles Teil-Skalarprodukt nach Modellwechsel | Gering |
| **0.3** | **Drift-Bandit-Kopplung verdrahten** | Drift-Wächter erkennt Verteilungsverschiebung, ruft aber Bandit-Reaktionsmethode nie auf | Gering |
| **0.4** | **Cloud-Egress-Klassifizierung** | Egress-Methode in gleicher Policy-Kategorie wie lokale Lesezugriffe | Gering |
| **0.5** | **Recovery-Pfad differenzieren** | Offene Intents werden pauschal vorwärts committet, egal ob Erfolg oder Abbruch | Mittel |

### Stufe 1 — Hot-Path-Performance (größter Hebel)

| ID | Maßnahme | Problem | Aufwand |
|---|---|---|---|
| **1.1** | **HNSW: Nachbarlisten ohne Allokation** | Pro Knoten eine Heap-Allokation für Adjazenzliste, größter Einzelfaktor im Suchpfad | Gering → Hoch |
| **1.2** | **HNSW: Backlink O(1)** | Lineare Suche über Einfüge-Operationen bei Batch-Insert, quadratisch bei großen Batches | Gering |
| **1.3** | **HNSW: Distanzpfad Lock/Allokation** | Lock auf Quantisierer pro Kandidat, Mmap-Vektor Element-für-Element dekodiert | Mittel |
| **1.4** | **SSTable: Zero-Copy-Slice** | Vollständige Blockkopie nach CRC-Prüfung, obwohl Puffertyp Slicing unterstützt | Trivial |
| **1.5** | **AES-Schlüsselplan wiederverwenden** | Key-Schedule-Neuaufbau pro Verschlüsselung/Entschlüsselung | Mittel |
| **1.6** | **MemTable: Range-Sharding** | Hash-Sharding zerstört Flush-Sortierung und Präfix-Scans | Hoch |
| **1.7** | **Block-Cache: Byte-basierte Kapazität** | Eintragsbasierte Kapazität bei variabler Blockgröße → unvorhersehbarer Speicher | Mittel |
| **1.8** | **RRF: `build_provenance`-Struct** | 14 positionelle Parameter desselben Typs → stille Vertauschung möglich | Gering-Mittel |
| **1.9** | **Text: Posting-Format umstellen** | Jedes Posting als Einzelschlüssel → massive Schreib-/Leseverstärkung | Hoch |
| **1.10** | **RRF/Text: Top-k-Selektion** | Volle Sortierung statt linearer k-Selektion | Gering |

### Stufe 2 — Speicher und Struktur

| ID | Maßnahme | Problem | Aufwand |
|---|---|---|---|
| **2.1** | **Graph: Inkrementelle Kompaktierung** | PPR löst vollständigen CSR-Rebuild pro Anfrage aus | Hoch |
| **2.2** | **CSR: Sentinel statt `Option`** | `Vec<Option<T>>` ohne Nischenoptimierung → doppelter Speicher pro Kante | Mittel |
| **2.3** | **Checkpoint: Indizes zusammenführen** | Zwei separate Locks für Sequenz- und Namens-Index → Inkonsistenzfenster | Gering-Mittel |
| **2.4** | **Manifest: Batch-Fsync** | Ein `fsync` pro Einzeleintrag statt pro Zustandsübergang | Mittel |
| **2.5** | **`contextra-py` in Workspace** | FFI-Grenze nicht von `cargo test --workspace` erfasst | Gering |

### Stufe 3 — Governance und Prozess

| ID | Maßnahme | Problem | Aufwand |
|---|---|---|---|
| **3.1** | **Feature-Kombinationen in CI** | Opt-in-Features werden nie in Kombination gebaut | Gering (Einrichtung) |
| **3.2** | **Panic-Inventar kontinuierlich** | Gate unterscheidet nicht zwischen Test- und Produktivcode | Gering |

### Kurzübersicht nach Aufwand/Nutzen

| Sofort umsetzbar (gering, hoher Nutzen) | Mittelfristig (mittel) | Struktureller Umbau (hoch) |
|---|---|---|
| 0.1 WAL-Replay-Bounds | 1.3 Distanzpfad-Lock/Allokation | 1.1 HNSW-Nachbarformat |
| 0.2 Bandit-Dimensionsprüfung | 1.5 AES-Schlüsselplan | 1.6 MemTable Range-Sharding |
| 0.3 Drift-Bandit-Kopplung | 1.7 Byte-basierte Cache-Kapazität | 1.9 Text-Posting-Format |
| 0.4 Egress-Kategorisierung | 1.8 build_provenance-Struct | 2.1 Inkrementelle Graph-Kompaktierung |
| 1.2 Backlink-Lookup | 2.2 CSR-Sentinel statt Option | |
| 1.4 SSTable-Zero-Copy-Slice | 2.3 Checkpoint-Index-Merge | |
| 2.5 contextra-py in Workspace | 2.4 Manifest-Batch-Fsync | |
| 3.1 / 3.2 Governance-Gates | 0.5 Transaktions-Intent-Status | |

### SOTA-Algorithmen-Roadmap (Fassung 4, §21, Priorisierung siehe Teil A4.5)

Diese vier Punkte sind **zusätzlich** zur obigen Opus-Analyse und laufen parallel zu ihr; sie sind nicht in
die Stufen 0–3 oben einsortiert, weil sie fachlich vier verschiedene Crates betreffen, methodisch aber
denselben Reifegrad-Prozess (Teil A.2/A.3) durchlaufen müssen wie jede andere Änderung.

| ID | Verfahren | Ziel-Crate | Priorität (A4.5) | Voraussetzung / Gate | Aufwand |
|---|---|---|---|---|---|
| **S.1** | LeanRAG Semantic Aggregation (dritte Pipeline-Stufe) | `contextra-cognition` (+ `contextra-graph`, `schemas/`) | 1 | `child_edge_ids`-Schema-Erweiterung + H5-Cascade-Rekursion (A4.4.1) MUSS zuerst grün sein; `ConsolidationConfig`/`SynthesisPhaseResult`-Namensraumbereinigung (A4.6 Punkt 3) davor | Mittel — Infrastruktur größtenteils vorhanden (A4.2.1) |
| **S.2** | TL-HFD | `contextra-graph` | 2 | Läuft zunächst nur im bestehenden `ShadowMode`-Pfad (AK-16); Default-Umstellung erst nach Auswertung der Diskrepanz-Logs (Teil A4.7) | Mittel — reine Ring-0-Algorithmus-Ersetzung in einem Crate |
| **S.3** | DiBud | `contextra-rank` (+ `contextra-vector`, `contextra-text`) | 3 | **Zweistufig:** zuerst Streaming-`Iterator<Item = DocId>`-Grenzen in DiskANN/BM25 (AK-17, A4.4.2), danach unabhängiger Benchmark gegen das Paper (Teil C C.1), erst danach der Fusions-Algorithmus selbst | Hoch — Mehr-Crate-Schnittstellenänderung, nicht lokal |
| **S.4** | Flow-Corrected Thompson Sampling | `contextra-adapt` | 4 | Als neue `BanditImplementation::FlowCorrectedThompson`-Variante neben bestehendem `ShermanMorrison` (nicht als Ersatz); Drift-Update nur aus Ring 3 (AK-18) | Mittel — zusätzlicher Speicherbedarf für Rolling-Window |

---

<a id="18-roadmap"></a>
