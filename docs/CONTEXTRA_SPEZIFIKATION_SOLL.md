# Contextra — Normative Spezifikation des Soll-Zustands

**Version:** 1.0 (final) · **Stand:** 26.11.2026 · **Geltung:** ersetzt `docs/SPEC.md` (v15) und alle darin enthaltenen Löschketten-Entscheidungen
**Autor/Rolle:** Principal Rust Engineer (Systemprogrammierung, Durability, Concurrency, RAG-Speicherkerne)
**Methode:** ausschließlich statische Planung. Es wurde nichts installiert, kompiliert oder ausgeführt. Grundlage sind (a) das geklonte Repository `tfufuz1/contextra` (Gliederung, Crate-Größen, Manifeste, Spezifikation v15, ausgewählte Quelldateien) und (b) die elf Konzeptdokumente aus dem Anhang.

---

## 0. Geltung, Lesehilfe, Belegstufen

### 0.1 Geltung

Dieses Dokument beschreibt den **normativen Soll-Zustand**. Es ist unabhängig vom Ist-Code. Bei Widerspruch zu `docs/SPEC.md`, zu Vetos (`F-02`, `OP-03`), zu ADRs oder zu Code-Kommentaren, die auf der Löschkette beruhen, gilt dieses Dokument. Das Projekt ist bei Version 0.1.0 ohne Produktivdaten; **Datei- und API-Kompatibilität zum Ist-Stand ist kein Ziel**.

### 0.2 Schlüsselwörter

**MUSS / MUSS NICHT** = zwingende Anforderung. **SOLL / SOLL NICHT** = Regelfall, Abweichung nur per Entscheidungsregister (Abschnitt 19). **KANN** = optional.

### 0.3 Belegstufen

| Marker | Bedeutung |
|---|---|
| **[Ist-V]** | Im Repository gelesen und bestätigt (Manifest, Dateikopf, Quelltext, Spezifikation v15) |
| **[Ist-D]** | Aus Dokumentation/Spezifikation v15 übernommen, Code nicht selbst gelesen |
| **[Ziel]** | Zielwert oder Arbeitsannahme, **nicht gemessen**; wird in Phase 0 des Plans durch Benchmarks validiert (Abschnitt 17) |
| **[Entscheidung]** | Bewusste Festlegung dieser Spezifikation, begründet im Entscheidungsregister |

Zahlenwerte in Tabellen („Default") sind **konfigurierbare Startwerte**, keine Leistungsversprechen.

---

## 1. Produkt und Scope

### 1.1 Mission

Contextra ist die **Gedächtnis- und Retrieval-Engine für Unternehmenswissen und KI-Agenten**. Sie verbindet Unternehmensdaten sicher, schnell und ressourcenschonend mit LLMs, indem sie **hybrides Retrieval (Vektor + Volltext + Graph)**, **token-bewusste Kontextaufbereitung** und **agentengerechte Schreibsemantik (Branches, Merge-Gates, Idempotenz)** als Eigenschaften der Speicher-Engine bereitstellt, nicht als Anwendungscode darüber.

### 1.2 Zielgruppe und Einsatzprofile

- Mittelstand und Großunternehmen in Deutschland und Europa (keine Behörden-Spezialanforderungen).
- Einsatz: (1) **eingebettete Rust-Bibliothek** im Prozess der Anwendung, (2) **einzelner Daemon** (eine Binärdatei) mit gRPC, HTTP und MCP, (3) lokale/on-premises LLM-Stacks (Ollama, Candle, ONNX) auf Standard-Hardware.
- Größenordnung je Knoten **[Ziel]**: 10⁵ bis 10⁸ Chunks, 384 bis 3072 Dimensionen, bis 10⁹ Kanten. Ein Knoten, ein Schreibprozess je Store.

| Ressourcenprofil | Zielhardware | Chunks | Vektorindex-Strategie |
|---|---|---|---|
| `lite` | Laptop/Edge: 4 Kerne, ≤ 8 GB RAM | bis 2·10⁶ | Flat + HNSW im RAM (SQ8) |
| `standard` | Server: 16 Kerne, 64 GB, NVMe | bis 5·10⁷ | HNSW (SQ8) + Vamana-Segmente |
| `scale` | Server: 32+ Kerne, 256 GB, NVMe-RAID | bis 10⁸ | Vamana/DiskANN mit RAM-Codes, SSD-Graph |

Profile sind **reine Parametersätze** (Budgets, Schwellen). Sie schalten keine Funktionen ab. Es gibt **keine** funktional gestaffelten Produktringe in der Engine; Lizenzierung ist Sache der Verpackung außerhalb des Kerns (Abschnitt 16.3).

### 1.3 Hauptfunktion und Differenzierung

1. **Ein Datenbestand, drei Zugriffsarten.** Vektor-, Volltext- und Graphzugriff laufen gegen **denselben Snapshot** über **denselben Dokumentadressraum**. Keine verteilte Transaktion, keine Daten-Duplikation, keine Eventual-Consistency-Lücke.
2. **Kontext statt Zeilen.** Die Engine liefert ein budgetiertes, zitierfähiges **Evidence Pack** (Handles, Quellen, Warnungen, Erklärung), nicht „Top-k-Zeilen".
3. **Agentensichere Schreibsemantik.** Sitzungen schreiben in Branches; Promotion nur über Merge-Gates; Retries sind idempotent.
4. **Inkrementelle Unternehmensdaten.** Dokument-Revisionen werden auf Chunk-Ebene per Inhalts-Hash abgeglichen: unveränderte Chunks behalten Identität, Embeddings und Zusammenfassungen.
5. **Ressourcenschonend.** Embeddings, Token-Zählungen und Zusammenfassungen werden pro Inhalts-Hash einmalig berechnet und gecacht; Speicherbudgets sind harte Grenzen.

### 1.4 Explizite Nicht-Ziele

| Nicht-Ziel | Begründung / Konsequenz |
|---|---|
| **DSGVO-Löschkette, Löschbeweise, Crypto-Shredding, signierte Löschnachweise, Revocation-Logs, Audit-Chain-Exporte, AVV-/Art.-30-/BSI-Generatoren** | Vollständig entfernt (Abschnitt 16.2). Löschung ist ein normaler, versionierter Vorgang (Abschnitt 13.5). |
| Kryptografisch beweisbare Zustände (Merkle-Wurzel über den Gesamtzustand, signierte Commits) | Kostet Schreibdurchsatz und Komplexität ohne Produktnutzen im Zielmarkt. Integrität wird durch Prüfsummen und Hash-Verkettung der Manifeste (Korruptionserkennung, kein Sicherheitsmerkmal) erreicht. |
| Verteiltes Multi-Node-System, Sharding, Raft, P2P-Sync | v1 ist Single-Node. Die Formate sind so geschnitten, dass Replikation später möglich bleibt (Abschnitt 6.12). |
| SQL-Kompatibilität, eigene Abfragesprache in v1 | Typisierte Rust-API, JSON-Plan und MCP-Werkzeuge. Eine textuelle Sprache ist optional nach v1. |
| LLM-Inferenz im Kern | Der Kern ruft Modelle nur über Ports (`Embedder`, `Reranker`, `Llm`, `Tokenizer`). Adapter liegen in Satelliten-Crates. |
| Bandit-Routing, Thompson-Sampling, Lyapunov-Drift-Watcher, Shadow-Mode-Gates im Kern | Forschungsnah, ohne validierten Bedarf. Aus dem Kern entfernt (Abschnitt 16.1), Reaktivierung nur nach Bedarfsnachweis. |
| WASM-Sandbox und Python-FFI im Kernprodukt | Optional außerhalb des Kern-Workspaces. |
| Realtime-Audio/Voice | Ausgeschlossen (bleibt gültig). |

### 1.5 Qualitätsziele **[Ziel]**

| Dimension | Zielwert | Messbedingung |
|---|---|---|
| Hybrid-Query p99 (k = 20) | < 30 ms warm, < 80 ms kalt | 10 Mio. Chunks, 1024 d, NVMe, Profil `standard`, ohne Embedding-Inferenz |
| Batch-Ingest | ≥ 30 000 Chunks/s je Branch | ohne Embedding-Berechnung, Profil `standard` |
| Commit-Latenz p99 | < 5 ms (`Group`) | NVMe, Gruppengröße ≤ 64 |
| Branch-Fork | < 100 µs, O(1) | beliebige Größe |
| Merge/Diff | O(Δ · log N) | Δ = Änderungen des Branches |
| Recovery | < 10 s je 500 MB WAL | Neustart ohne Indexneubau (Indizes fallen auf Flat zurück) |
| Recall@10 Vektor-Leg | ≥ 0,95 | Ziel-`ef`, gegen exakte Suche, je Datensatz zu messen |
| Speicherbedarf pro Chunk (Index) | ≤ 12 % der Rohvektorgröße | PQ/SQ8/RaBitQ-Codes plus Graph |

Öffentliche Zahlen MÜSSEN Messbedingungen nennen und MÜSSEN ausweisen, ob Embedding-Inferenz enthalten ist (typisch +10…500 ms).

---

## 2. Analyse: Auswertung der Konzepte und des Ist-Stands

### 2.1 Auswertung der elf Konzeptdokumente

Bewertungsmaßstab: Nutzen für Unternehmens-RAG und Agenten, Implementierbarkeit in Rust mit vertretbarem Risiko, Verträglichkeit mit einem Single-Node-Kern ohne Löschkette.

| Quelle | Kernidee | Entscheidung |
|---|---|---|
| **Claude.md (EngramDB)** | Truth ≠ Index; gemeinsamer Docid-Raum mit Filter-Pushdown; Session-Branches, Merge-Regeln pro Tabelle; `PACK CONTEXT`; bitemporal; Durability-Klassen; Idempotenz; Deterministic-Simulation-Tests | **Rückgrat übernommen.** Abweichung: kein Prolly-Tree/Merkle-DAG als Zustandsstruktur, keine signierten Commits (D-01, D-02). |
| **VERSIONED_STORAGE_ARCHITECTURE.md** | Seam-Traits, Entscheidungs- und Blindfleck-Register, Index-Konsistenzvertrag `(index_id, root)`, GC als Phasen-Gate, Fehlerkatalog mit `remediation` | **Übernommen:** Register-Methodik, Konsistenzvertrag (als Watermark), Fehlerkatalog, GC-Disziplin. Nicht übernommen: Tabellen-/SQL-Schicht (D-14). |
| **MNEMOS_CONCEPT.md** | Cell mit Facetten, Zoom-Leiter, Zoom-Knapsack, Token-Summen, Derivation Cache + Lineage, Work Queue, Handles, Trust/Taint, Progressive Disclosure, cache-freundliche Ordnung | **Stark übernommen** (Packer, Zoom, Derivation Cache, Work Queue, Handles, Trust). Nicht übernommen: gelerntes Layout/Affinity-Re-Layout (zurückgestellt, Abschnitt 8.9), Per-Subject-Keys und Crypto-Shredding (Löschkette). |
| **Optimal-LLM-Datenbank.md** (Engram/Gemini/Grok/CairnDB) | Co-located Context Slabs, Locality-aware Segmente, Trajectory-aware GC, **Index-Watermarks**, Embeddings als versionierte Daten, Speicherklassen (Working/Episodic/Semantic/Procedural/Artifacts) | **Übernommen:** Watermarks, Embeddings als abgeleitete versionierte Daten, Speicherklassen, Locality im Segment (Hot-Head/Cold-Tail als Spaltenschnitt, nicht als Slab-Allokator). Slab-Suballokator verworfen (Komplexität ohne Beleg). |
| **ChatGPT.md** | LSM als Kern, kanonisches Datenmodell, gemeinsame Objektidentität, immutable HNSW-Segmente, adaptive Ausführungsreihenfolge, Group Commit | **Übernommen:** LSM-artiger Kern, immutable Index-Segmente, adaptive Reihenfolge. |
| **Grok.md (AetherDB)** | Einheitlicher Object-Store, ephemere Agent-Branches, Provenance-Zitierpflicht | **Teilweise.** Gemeinsame Nachbarschaftsrelationen von ANN-Graph und Property-Graph verworfen (vermischt Semantik; D-09). |
| **Qwen.md (AxiomDB)** | Graph-∩-Text-Bitmap als harte Constraint in der ANN-Traversierung; Streaming-MMR; U-förmige Platzierung; CRDT-artige Kantenmengen; asynchroner Widerspruchs-Critic | **Übernommen:** Bitmap-Constraint, MMR-Operator, U-Form, Add-Wins-Kantenmengen, Critic als Job. Verworfen: 16-KiB-Merkle-Seiten über alle Daten (Write-Amplification), „Roaring-Akkumulator für Vektoren" (nicht tragfähig), „wait-free Writes"-Behauptung. |
| **Gemini.md / Gemini-2.md** | Multi-Modal-Segmente (IMMS), Trust-Propagation über Pfade (`C(P)=∏conf·γ^(k-1)`), Write-Time-Contradiction für **funktionale Prädikate**, Query-Level-Trust-Gates, Zero-Overhead-Token-Accounting | **Übernommen:** Pfad-Konfidenz, funktionale Prädikate, Trust-Gates, Token-Accounting. Signaturen je Provenance-Record verworfen. |
| **Perplexity.md (Helix)** | Claim-Extraktion, Citation-Binding, Entailment-Check, Entity-Resolution, Tool-Call-Replay, Quarantine-Zone, Reviewable Merge | **Übernommen** (Abschnitte 11, 12). |
| **Deepseek.md (Mnemosyne)** | Schichten, Commit-DAG, Reproduzierbarkeit, Implementierungsreihenfolge | Bestätigt Grundlinie; keine zusätzlichen Mechanismen übernommen. |

### 2.2 Befund am Ist-Stand (Repository)

Bestätigt durch Manifest, Spezifikation v15 und Stichproben im Quelltext.

| # | Befund | Beleg | Folgerung |
|---|---|---|---|
| F1 | **Vier getrennte Persistenzsysteme** (LSM-KV, HNSW, BM25-Inverted, CSR-Graph) werden über ein **4-Index-2-Phase-Commit mit kompensierenden Transaktionen** zusammengehalten; die Rollback-Doku warnt selbst vor „Split-Brain". | Dateikopf `contextra-engine/src/transaction/mod.rs` **[Ist-V]** | Strukturelle Schwäche. Soll: **ein** Commit-Punkt (Log), alle Indizes **abgeleitet** mit Watermark (Abschnitt 8.7). 2PC und Kompensation entfallen. |
| F2 | Der Vektorindex ist **mutierbar** (Soft-Delete, Nachbarschafts-Reparatur, atomarer Rebuild, `GraphRepairAttestation`); Veto F-02 und das Feature `partial-index-rebuild` stehen im Konflikt. | Spezifikation B.3.1, A.1.4 **[Ist-D]** | Die Reparatur existiert wegen des Löschbeweises. Soll: **immutable Segmente + Liveness-Bitmaps**; keine Graph-Reparatur nötig. |
| F3 | Die Löschkette ist **quer durch mindestens 12 Crates** verteilt (Fundstellen per Dateisuche, Abschnitt 16.2). | grep über `crates/*/src` **[Ist-V]** | Ausbau erfolgt als Schnitt, nicht als Patch. |
| F4 | `contextra-store` ist mit ~27 k Zeilen (`src`) das größte Crate und trägt KV-Shredding, Lösch-Modi, Tenant-Codec mit Präfixschlüsseln, HMAC-verkettetes WAL. | Zeilenzählung, Cargo-Features `deletion-proof`, `legacy-wal-key` **[Ist-V]** | Neuschnitt des Store-Kerns (Abschnitt 6) statt Refaktor. |
| F5 | **SSI-Lesepfade registrieren mit `snapshot_seq = u64::MAX`**, wodurch Konflikterkennung für zuerst so gelesene Schlüssel wirkungslos sein kann (Befund B-16, als Verdacht geführt). | Spezifikation J.13 **[Ist-D]** | Soll: Transaktionsobjekt besitzt den Snapshot; Lesen und Registrieren sind **nicht trennbar** (Typestate, Abschnitt 7.4). Der Fehlerklasse wird konstruktiv vorgebeugt. |
| F6 | Schlüsselraum ist **stringbasiert** (`__col:{name}:\x00…`, `t:{tenant}:{col}:{type}:{id}`), Dokument-IDs sind Strings, Graph-Knoten teilen den String-Schlüssel des Dokuments (implizite Invariante, bei Verletzung leeres Graph-Signal). | Spezifikation B.1, B.3.5, I.4 **[Ist-D]** | Soll: typisierte IDs und **dichter globaler Adressraum** (Abschnitt 5.1). |
| F7 | Decay **löscht** Erinnerungen unter einer Schwelle (`DECAY_DELETION_THRESHOLD = 0.05`). | Spezifikation B.1.1 **[Ist-D]** | Stilles destruktives Verhalten ist für Unternehmensdaten ungeeignet. Soll: Decay beeinflusst Ranking und Archivierungsvorschläge, löscht nie (Abschnitt 11.4). |
| F8 | Retrieval-Varianten wuchern: PPR in sechs Algorithmen (Dense, ForwardPush, Shadow, TL-HFD, APPRH, k-Path), PathRAG (nicht snapshot-fähig), Perkolation, Synapsen-Verstärkung, Bandit-/Router-Schichten. | Spezifikation B.3.3, B.5 **[Ist-D]** | Ein validierter Standard je Aufgabe (Abschnitt 8.5), Rest entfernt oder zurückgestellt. |
| F9 | Strangler-Schalen (`contextra-core` „deprecated", `contextra-db` 2 k Zeilen) und Governance-Werkzeuge (`xtask`: > 100 Prüfungen, `.jules`-Harness) sind größer als manche Fachkomponenten. | Dateiliste **[Ist-V]** | Schalen löschen; Governance auf das Nötige reduzieren (Abschnitt 15.7). |
| F10 | `contextra-ingest` (~21 k Zeilen) unterstützt PDF, Office, HTML, E-Mail, CSV (Abhängigkeiten `lopdf`, `calamine`, `html5ever`, `mail-parser`, `quick-xml`). | Workspace-Abhängigkeiten, Zeilenzahl **[Ist-V]** | Echter Unternehmensnutzen. **Bleibt**, bekommt strukturbewusstes Chunking und Revisions-Diff (Abschnitt 11.1). |

### 2.3 Zentrale Architekturentscheidungen (Kurzfassung)

1. **Truth ≠ Index** [D-03]: Wahrheit = Commit-Log + immutable Truth-Segmente. Indizes = abgeleitete, jederzeit neu aufbaubare Segmente mit Watermark.
2. **LSM + MVCC + Overlay-Branches statt Prolly-Tree** [D-01]: Gleiche Git-Semantik (O(1)-Fork, O(Δ)-Diff/Merge, Zeitreise) bei deutlich geringerer Schreibverstärkung und einfacherem Index-Co-Design.
3. **Ein globaler dichter Adressraum (`Gid`)** [D-04]: Graph-Ergebnisse, Textfilter, ACL und Zeitfilter sind Roaring-Bitmaps über denselben Raum; sie schränken ANN- und BM25-Suche direkt ein.
4. **Embeddings, Zusammenfassungen, Token-Zählungen sind Derivate** [D-05], per `(ContentHash, ProducerId)` memoisiert.
5. **Löschung = versionierter Tombstone + Retention-Compaction** [D-06], ohne Nachweiskette.
6. **Ein Satz von Kernalgorithmen** [D-09]: HNSW/Vamana, BM25F/BMW, CSR + Forward-Push-PPR + Leiden, RRF, Multiple-Choice-Knapsack + MMR.

---

## 3. Leitprinzipien und Systeminvarianten

### 3.1 Prinzipien

| # | Prinzip | Konsequenz |
|---|---|---|
| P1 | **Truth ≠ Index** | Indexfehler gefährden nie Commit, Daten oder Merge. Indizes sind verwerfbar. |
| P2 | **Immutability by default** | Segmente, Blobs und Log-Segmente werden nie überschrieben. Veränderlich: Memtables, Manifest-Log, WAL-Tail. Der Block-Cache hält nur unveränderliche Daten (keine Dirty Pages). |
| P3 | **Ein Commit-Punkt** | Ein Commit ist durable, sobald sein WAL-Record die Durability-Klasse erfüllt. Kein mehrstufiges Protokoll über mehrere Stores. |
| P4 | **Jede Aussage hat Herkunft** | Provenance ist Pflichtfeld für Chunks, Fakten, Kanten, Zusammenfassungen. |
| P5 | **Agenten schreiben nie direkt nach `main`** | Standardpfad: Session-Branch → Merge-Gates → Merge. Direktes Schreiben nach `main` ist privilegiert und konfigurierbar. |
| P6 | **Optimiere Tokens, nicht Zeilen** | Der Packer löst ein Auswahlproblem unter Token-Budget. |
| P7 | **Zeit ist eine Dimension** | Transaktionszeit (Commit) und Gültigkeitszeit (Realwelt). Widersprüche und Veralten sind Datenmodellkonzepte. |
| P8 | **Ehrliche Garantien** | Truth Layer ist deterministisch und prüfbar; Recall und Latenz der Indizes sind statistisch und werden als solche gemeldet (Lane-Vollständigkeit, Abdeckung). |
| P9 | **Fail-closed in der Sicherheit, fail-open in der Verfügbarkeit** | Fehlende ACL/Trust-Information verbirgt Treffer; fehlender Index fällt auf Flat-Suche zurück. |
| P10 | **Determinismus ist injizierbar** | Uhr, Zufall, Dateisystem, Netzwerk nur über Ports; jede Abfrage trägt `now` und `snapshot` und ist damit reproduzierbar. |
| P11 | **Safe Rust im Kern** | `unsafe` nur in ausgewiesenen Inseln (`sys`, `simd`, Mmap-Lesepfad), je mit Safety-Invarianten, Miri/Loom/Fuzz. |
| P12 | **Nichts wird still zerstört** | Kein Verfahren (Decay, Konsolidierung, Merge-Konflikt, Compaction) darf sichtbare Daten verwerfen, ohne dass es ein Tombstone-Commit oder eine Retention-Regel ausweist. |

### 3.2 Systeminvarianten (normativ, testbar)

| ID | Invariante |
|---|---|
| INV-1 | **Snapshot-Einheit:** Alle Signale (Vektor, Text, Graph, Filter, Fusion, Packer) einer Abfrage lesen genau **einen** Snapshot `(branch, seq)`. |
| INV-2 | **Prefix-Konsistenz:** Nach Absturz entspricht der sichtbare Zustand einem Präfix der Commit-Folge, die laut Durability-Klasse bestätigt wurde. |
| INV-3 | **Index ⊆ Truth:** Jede Trefferzeile eines Index wird gegen den Truth-Snapshot validiert (Liveness, Sichtbarkeit, ACL, Zeit); ein Index kann nie Daten „erfinden" oder verborgene Daten offenlegen. |
| INV-4 | **ACL vor Ranking:** Zugriffsfilter wirken als Bitmap **innerhalb** von ANN-/BM25-/Graph-Operatoren, nie nur nachgelagert. Weder Treffer noch Zählungen, Scores oder Fehlertexte dürfen unberechtigte Inhalte verraten. |
| INV-5 | **Idempotenz:** Ein wiederholter Commit mit gleichem Idempotenzschlüssel liefert dasselbe Ergebnis und erzeugt keinen zweiten Commit. |
| INV-6 | **Branch-Isolation:** Ein Branch ändert die Sicht eines anderen Branches nie, außer durch Merge/Rebase. |
| INV-7 | **Main-Konsistenz:** `main` enthält nie ungelöste Konflikte oder Merge-Zwischenzustände; Konflikte leben im Merge-Request-Objekt. |
| INV-8 | **Derivat-Rückführbarkeit:** Jedes Derivat nennt Quell-`ContentHash`, `ProducerId` und Version. Ändert sich die Quelle, ist das Derivat per Konstruktion als veraltet erkennbar. |
| INV-9 | **Kein stilles Verwerfen** (P12): Jede Verringerung des sichtbaren Bestands ist ein Commit (Tombstone) oder eine ausgewiesene Retention-Aktion. |
| INV-10 | **Reproduzierbarkeit:** `(snapshot, query, params, now, index-generationen)` reproduziert Treffer und Reihenfolge, soweit die Indizes deterministisch gebaut wurden; andernfalls ist die Abweichung auf Index-Nichtdeterminismus begrenzt und wird im Explain ausgewiesen. |
| INV-11 | **Speichergrenzen:** Kein Teilsystem überschreitet sein Speicherbudget (Abschnitt 14.1); Überschreitung führt zu Backpressure, nie zu OOM. |
| INV-12 | **Format-Versionierung:** Jede Datei trägt Magic, Formatversion, Feature-Flags; unbekannte *inkompatible* Flags verhindern das Öffnen (fail-closed), unbekannte *kompatible* Flags werden ignoriert. |

---

## 4. Architektur-Überblick

### 4.1 Schichtenmodell

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ R5  Zugriff     Rust-API │ gRPC + Arrow Flight │ HTTP │ MCP (stdio/HTTP) │ CLI│
├─────────────────────────────────────────────────────────────────────────────┤
│ R4  Dienste     Agent-Runtime (Sessions, Gates) │ Ingest │ Cognition-Jobs     │
│                 Security (ACL, Trust, AEAD) │ Inferenz-Adapter │ KV-Cache(opt)│
├─────────────────────────────────────────────────────────────────────────────┤
│ R3  Engine      Katalog │ Transaktions-Fassade │ Planner/Executor │ Job-Sched.│
│                 Derivation-Cache │ Context-Engine-Orchestrierung              │
├─────────────────────────────────────────────────────────────────────────────┤
│ R2  Index/Rech. Vector │ Text │ Graph │ Rank (Fusion, Kalibrierung) │ Pack    │
│                 (reine Berechnung, ohne Laufzeit, ohne eigene Persistenz-Logik)│
├─────────────────────────────────────────────────────────────────────────────┤
│ R1  Storage     Log (WAL) │ Store (Segmente, Manifest, Compaction, Blobs)     │
│     Kernel      MVCC/Branches (Snapshots, OCC/SSI, Overlay-Sichtbarkeit)      │
├─────────────────────────────────────────────────────────────────────────────┤
│ R0  Fundament   Types │ Ports (Clock, Rng, Fs, Embedder, …) │ Codec │ SIMD │ Sys│
└─────────────────────────────────────────────────────────────────────────────┘
```

Regeln: Abhängigkeiten zeigen nur nach unten (geprüft durch `tests/layering.rs` und `cargo-deny`-Regeln). **R0–R2 sind laufzeitfrei** (kein `tokio`); der Storage-Kernel (R1) bietet eine synchrone API mit Completion-Handles und besitzt eigene I/O-Threads, damit er in jeder Laufzeit und im deterministischen Simulator läuft. `tokio` beginnt bei R3 und in R5. `unsafe` ist auf `sys`, `simd`, `codec` (Zero-Copy-Zugriff) und den Mmap-Lesepfad in `store` begrenzt.

### 4.2 Ziel-Workspace (Soll)

| Ring | Crate | Rolle | Herkunft |
|---|---|---|---|
| R0 | `contextra-types` | IDs, `Seq`, `Hlc`, Fehler (`ErrorCode`, Remediation), Limits, Konfigurationstypen | behält, bereinigt |
| R0 | `contextra-ports` | Traits: `Clock`, `Rng`, `Fs`, `Embedder`, `Reranker`, `Llm`, `Tokenizer`, `MetricsSink`, `Judge` | behält, bereinigt |
| R0 | `contextra-codec` | On-Disk-/Wire-Formate (rkyv-Archive, Varint, Bitpacking, Magic/Version) | ersetzt `contextra-wire` |
| R0 | `contextra-simd` | Distanz-/Bitpacking-Kernel mit Runtime-Dispatch (AVX2/AVX-512/NEON/Scalar) | behält |
| R0 | `contextra-sys` | Betriebssystemzugriff: `DurableFs` (fsync, Verzeichnis-fsync, Locks, preallocate), io_uring-/pread-Backend, Mmap-Guards | `contextra-sys` + `contextra-durable-fs` |
| R1 | `contextra-log` | WAL: Format, Group Commit, Recovery, Segment-Recycling | neu (aus `store/wal`) |
| R1 | `contextra-store` | Memtable, Truth-/Index-Segmente, Manifest-Log, Compaction, Blob-Store, Block-Cache, Liveness | **Neuschnitt** |
| R1 | `contextra-mvcc` | Sequencer, Snapshots/Epochen, Branch-Lineage, OCC/SSI, Idempotenz | behält, umgebaut |
| R2 | `contextra-vector` | Flat, HNSW (RAM), Vamana (SSD), Quantisierer, gefilterte Suche | behält, bereinigt |
| R2 | `contextra-text` | Analyzer (u. a. Deutsch), FST, Postings, BM25F, Block-Max-WAND | behält, bereinigt |
| R2 | `contextra-graph` | CSR-Segmente, Traversierung, Forward-Push-PPR, Leiden | behält, stark reduziert |
| R2 | `contextra-rank` | RRF, Normalisierung, Kalibrierung, MMR | behält, bereinigt |
| R2 | `contextra-pack` | Context-Packer: Knapsack, Platzierung, Rendering, Handles | **neu** (aus Fragmenten in `engine`/`rank`) |
| R3 | `contextra-engine` | Katalog, Namespaces, Query-Planner/-Executor, Ingest-Orchestrierung, Job-Scheduler, Derivation-Cache | Neuschnitt aus `engine` + `db` |
| R4 | `contextra-ingest` | Extraktion, strukturbewusstes Chunking, Revisions-Diff, Injection-Scan | behält |
| R4 | `contextra-agent` | Sessions, Merge-Gates, Action-Log, Review-Merge | behält + `checkpoint` integriert |
| R4 | `contextra-cognition` | Konsolidierung, Zusammenfassung, Entitäten, Widerspruchs-Jobs | behält, bereinigt |
| R4 | `contextra-security` | AEAD at Rest, ACL/Capabilities, Trust/Taint, Egress-Guard | `crypto` + `privacy` stark reduziert |
| R4 | `contextra-infer-{candle,ollama,onnx}` | Adapter für `Embedder`/`Reranker`/`Llm` | behalten |
| R4 | `contextra-kvcache` | optionaler Prefix-/KV-Cache-Store für lokale LLMs | behält (optional), ohne Shredding |
| R5 | `contextra-mcp`, `contextra-server`, `contextra-cli`, `contextra` (Fassade) | Zugriffsschichten | `mcp`, `contextra` behalten; Server/CLI neu |
| Dev | `contextra-testkit`, `contextra-sim` | Testdaten, **deterministischer Simulator** (Fs/Clock/Rng/Fault) | `testkit` behält, `sim` neu |

Entfernt oder ausgelagert werden `contextra-core`, `contextra-db`, `contextra-adapt`, `contextra-router`, `contextra-checkpoint` (integriert), `contextra-audit-export`, `contextra-avv-generator`, `contextra-license` (Verpackung), `contextra-sandbox`, `contextra-py`, `contextra-wire` (ersetzt). Details und Begründung: Abschnitt 16.1.

### 4.3 Schreibpfad (Überblick)

```
Client ──► Ingest-Pipeline (extrahieren → chunken → Hash-Abgleich → Embedding aus Derivation-Cache oder Port)
       ──► Txn (Snapshot gepinnt, Staging, Schema-/Pflichtfeldprüfung)
       ──► Commit-Sequencer: OCC-Validierung → WAL-Group-Commit (fdatasync je Gruppe) ← einziger Commit-Punkt
       ──► Apply: Memtable (Records, Kanten, Delta-Vektoren, Mini-Invertindex) → neuer Snapshot sichtbar
       ──► [async] Seal: Memtable → Truth-Segment + Index-Segmente → Manifest-Eintrag → WAL-Recycling
       ──► [async] Derived Jobs: fehlende Indizes, Summaries, Entitäten, Konfliktprüfung
```

### 4.4 Lesepfad (Überblick)

```
Anfrage ──► Snapshot pinnen (Branch, Seq, now) ──► Planner (Kosten aus Manifest-Statistiken)
        ──► Filter-Bitmaps (ACL ∩ Zeit ∩ Trust ∩ Attribute ∩ Graph-Menge)  [Gid-Raum]
        ──► Operatoren je Segment parallel: Vektor │ BM25F │ Graph │ Prior    (geteilte Top-k-Schwelle)
        ──► Fusion (RRF/Norm) ──► Validierung gegen Truth (Liveness/Sichtbarkeit) ──► optional Rerank
        ──► Pack (Zoom-Knapsack, MMR, Platzierung, Handles, Warnungen) ──► Evidence Pack + Explain
```

---

## 5. Datenmodell

### 5.1 Identität und Adressräume [Entscheidung D-04]

| Typ | Breite | Eigenschaft |
|---|---|---|
| `RecordId` | 128 Bit, zeitlich sortierbar (UUIDv7-artig) | Stabile logische Identität über Versionen und Branches; extern sichtbar (in Handles gekürzt). |
| `ExternalKey` | String ≤ 512 B, je Namespace eindeutig (optional) | Anwendungsschlüssel (z. B. `sharepoint:/site/doc#chunk-12`). Abbildung `ExternalKey → RecordId` im Katalog. |
| **`Gid`** | `u64`, dicht, monoton, nie wiederverwendet | **Globaler dichter Index-Adressraum.** Wird bei Erstanlage eines Records vergeben und bleibt über alle Versionen gleich. Alle Bitmaps (Roaring64) sind über `Gid`. Abbildung `RecordId ↔ Gid` ist Teil des Truth Layers (`idmap`). |
| `RowAddr` | `(segment: u32, ord: u32)` | Physische Position einer Version innerhalb eines Segments; nur innerhalb eines Snapshots gültig. |
| `Seq` | `u64`, global monoton | Commit-Reihenfolge über alle Branches. Transaktionszeit-Achse. |
| `Hlc` | 64 Bit (Hybrid Logical Clock) | Wanduhr-konsistente Zeit je Commit, für `AS OF TIMESTAMP`. |
| `ContentHash` | BLAKE3-256 über kanonischen Inhalt mit Typpräfix | Adressiert Text, Blobs, Embeddings-Quellen. |
| `BranchId` | `u32` | Branch-Identität; `main = 0`. |

**Warum `Gid`:** Segmente sind nach `(gid, born_seq)` sortiert. Dadurch ist die Übersetzung einer `Gid`-Menge (Graph-Ergebnis, ACL, Filter) in eine segmentlokale `ord`-Bitmap eine **sortierte Verschmelzung bzw. Bereichsschnitt**, kein Hash-Join. Das ist der Preis-/Nutzen-Kern des Hybrid-Versprechens („ein Filter für ANN und BM25") und wird im Katalog der Risiken (Abschnitt 18) mit Kostenmodell geführt.

### 5.2 Record

Ein **Record** ist die Wissenseinheit. Er besteht aus einem **unveränderlichen Inhaltsteil** und **versionierten Attributen**.

```rust
struct RecordVersion {                 // Zeile im Truth-Segment (Spalten, nicht Struct-Layout)
    gid: Gid,                          // stabil
    record_id: RecordId,
    ns: NsId,                          // Namespace
    kind: Kind,                        // Document | Chunk | Entity | Fact | Episode | Summary | Artifact
    class: MemoryClass,                // Working | Episodic | Semantic | Procedural | Reference
    parent: Option<Gid>,               // Hierarchie: Dokument → Abschnitt → Chunk; Summary → Quelle
    level: u8,                         // 0 Rohchunk, 1 Abschnitt, 2 Dokument, 3 Thema
    content: Option<ContentRef>,       // inline ≤ 2 KiB, sonst Blob; Text immer als ContentHash adressiert
    content_hash: ContentHash,
    props: Props,                      // typisiert nach Namespace-Schema
    valid: Interval<Hlc>,              // Gültigkeitszeit (Realwelt), Default offen
    trust: TrustTier,                  // Abschnitt 13.3
    taint: TaintFlags,                 // z. B. INJECTION_SUSPECT, EXTERNAL
    confidence: f32,                   // [0,1], Quelle: Extraktor/Review
    support: u32,                      // Anzahl bestätigender Evidenzen (additiv beim Merge)
    prov: ProvId,                      // Abschnitt 5.4
    acl: AclRef,                       // Abschnitt 13.2
    flags: Flags,                      // TOMBSTONE | STALE | ARCHIVED | DERIVED | PINNED
    born_seq: Seq,                     // Commit, der diese Version erzeugt hat
    branch: BranchId,
}
```

**Regeln**

- **Zwei Zeilenarten** (`row_kind`), beide unveränderlich:
  - **Content-Zeile:** trägt `content_hash`, Text, Vektoren, Token-Zählungen und die Attribute zum Erzeugungszeitpunkt. Eine neue Content-Zeile für dieselbe `gid` erhält einen **Death-Eintrag** für die vorherige Content-Zeile (6.6).
  - **Attr-Zeile:** trägt **nur Attribute** (Props, `trust`, `taint`, `acl`, `valid`, `flags`, `confidence`, `support`). Sie ersetzt die vorherige Attr-Zeile (Death-Eintrag), **lässt die Content-Zeile am Leben**.
- **Sichtbare Version** einer `gid` = (neueste sichtbare Content-Zeile, neueste sichtbare Attr-Zeile, sonst die Attribute der Content-Zeile), zusammengeführt per `gid`. Reine Attributänderungen lösen **keine Neuberechnung** von Embedding, Token-Zählung oder Text-Index aus, und die Treffer der Indizes bleiben gültig. Filter (ACL, Trust, Zeit, Flags) werden im `Gid`-Raum aus den **aktuellen Attributen** gebildet und sind damit auch für Treffer aus älteren Content-Zeilen korrekt. Validierung gegen den Truth-Snapshot (INV-3): Ein Index-Treffer `(segment, ord)` ist gültig, wenn seine Content-Zeile für die `gid` die sichtbare Content-Zeile ist.
- **Retire** schreibt eine Attr-Zeile mit `TOMBSTONE` **und** einen Death-Eintrag für die Content-Zeile (ab `died_seq` unsichtbar in Suche und Lesen, davor weiterhin per Zeitreise sichtbar).
- **Veränderliche Zähler** (Zugriffe, Expand-Rate, `last_accessed`) liegen **nicht** im Record, sondern im Statistik-Sidecar (5.7).
- `kind = Fact` repräsentiert eine **Aussage** `(subject, predicate, object | value)` mit `valid`-Intervall; n-äre Relationen werden als **reifizierte Facts** mit Rollen-Kanten modelliert (kein separater Hyperkanten-Speicher, D-09).

### 5.3 Namespaces und Katalog

Ein **Namespace** ist die Einheit für Schema, Richtlinien und Zugriff (Ersatz für „Collection"). Der **Katalog** ist selbst eine versionierte Tabelle im Truth Layer (Zeitreise gilt auch für Schemata).

```toml
[namespace."wissen/handbuecher"]
class            = "reference"            # Standard-MemoryClass
analyzers        = ["de", "en"]           # Sprachen; Spracherkennung je Chunk
chunker          = "structure-v1"         # Abschnitt 11.1
embedders        = ["emb-de-1024-v2"]     # Embedding-Spaces (Abschnitt 5.6)
tokenizers       = ["o200k", "llama3"]    # Token-Zählungen werden persistiert
zoom             = ["L0:raw", "L1:summary<=200tok", "L2:gist<=24tok"]
decay            = { class = "slow", half_life_days = 730 }
retention        = { history_days = 30, branches_days = 1 }
merge            = { text = "manual", props = "field", counters = "sum", edges = "add-wins" }
durability       = "group"                # strict | group | relaxed | volatile
default_trust    = "curated"
acl_default      = "grp:alle-mitarbeiter"
functional_predicates = ["hat_verantwortlichen", "aktueller_status"]
```

Der Katalog hält außerdem: Embedding-Spaces, Analyzer-Definitionen (ID + Parameter fließen in den Index-Segmentschlüssel), Tokenizer-Registry, Decay-Profile, Prinzipale/Gruppen, Merge-Gate-Konfiguration.

**Mandantenisolation [Entscheidung D-07]:** Ein **Store** (Verzeichnis, eigener Schlüssel, eigener Log) ist die **harte** Isolationsgrenze (ein Mandant = ein Store). Innerhalb eines Stores trennen **Namespaces + ACL-Bitmaps** Abteilungen und Rollen. Es gibt keine mandantenübergreifenden Datenflüsse (bleibt aus Veto F-10 erhalten) und keinen mandantenübergreifenden Derivation-Cache (kein Seitenkanal „existiert dieser Text bei jemand anderem?").

### 5.4 Provenienz

```rust
struct Provenance {                    // append-only, im Truth Layer (Tabelle `prov`)
    id: ProvId,                        // (subject_gid, ev_seq); weitere Evidenz = ev_seq + 1
    source_uri: Option<Uri>,
    source_hash: Option<ContentHash>,  // Hash des Quelldokuments (Version)
    source_span: Option<(u64, u64)>,   // Byte-/Zeichenbereich im Quelldokument
    origin: Origin,                    // Ingest{parser_version} | LlmExtraction{model, prompt_hash}
                                       // | ToolCall{tool, input_hash, output_hash} | Human{principal}
                                       // | Derivation{producer, sources: Vec<Gid>}
    producer: ProducerId,              // Modell-/Parser-ID mit Version
    confidence: f32,
    observed_at: Hlc,
    run: Option<RunId>,                // Ingest-Lauf / Agenten-Sitzung
}
```

Provenienz ist **Pflicht** (Commit-Validierung, Abschnitt 7.3). Sie wird **nicht signiert**; Manipulationsschutz erfolgt über Zugriffskontrolle und Prüfsummen. Tool-Ausgaben tragen `input_hash`/`output_hash` für Replay (nicht-deterministische Tools bleiben nachvollziehbar).

### 5.5 Zoom-Leiter und Hierarchie

- **Hierarchie:** `Document → Section → Chunk` (über `parent`, `level`). Retrieval arbeitet auf kleinen Chunks (Präzision); der Packer expandiert bei Bedarf zum Elternabschnitt („small-to-big").
- **Zoom-Stufen je Record:** `L0` Rohtext, `L1` Zusammenfassung (≤ 200 Token), `L2` Gist/Titel (≤ 24 Token). `L2` wird **extraktiv und deterministisch** erzeugt (Titel, Überschriftenpfad, Schlüsselsatz) bereits beim Ingest. `L1` ist **abstraktiv** (LLM) und wird als Job erzeugt; fehlt sie, wählt der Packer die nächste verfügbare Stufe.
- Jede Stufe kennt `tok_count[tokenizer]` (persistiert als Spalte). Token-Budgetierung benötigt **keinen** Inhaltszugriff.

### 5.6 Embedding-Spaces und Derivation-Cache [Entscheidung D-05]

```rust
struct EmbeddingSpace {
    id: SpaceId, model: ModelId, version: u32, dim: u16,
    metric: Cosine | Dot | L2, normalize: bool,
    storage: F16 | F32,                    // Truth-Speicherung (Default F16)
    matryoshka_prefixes: Vec<u16>,         // z. B. [256, 512, 1024]; leer = keine
    index: IndexParams,                    // HNSW(M, efC) | Vamana(R, L, alpha) | Flat
    quant: None | SQ8 | PQ{m,bits} | RaBitQ{bits},
}
```

- Embeddings sind **Derivate**, geschlüsselt nach `(ContentHash, SpaceId)` in der Tabelle `emb` (Vektorblöcke im Truth-Segment, Index darüber). **Identischer Inhalt wird pro Store nur einmal eingebettet** (über Namespaces und Branches hinweg).
- **Modellwechsel:** Ein neuer Space wird angelegt; ein Hintergrundjob embeddet nach, Abfragen wählen Space explizit oder per Namespace-Policy (`active_space`). Während der Migration liefert die Abfrage `lane_coverage` (Anteil der Zeilen mit Embedding im aktiven Space) und kann auf den alten Space zurückfallen. Nach Umschalten und Aufbewahrungsfrist wird der alte Space verworfen. **Vektoren verschiedener Spaces werden nie verglichen oder gemischt.**
- **Eingebaute Vektoren:** Clients dürfen Embeddings mitliefern (Space-ID Pflicht). Dimension und Normalisierung werden validiert (NaN/∞/leer → Fehler `E_VECTOR_INVALID`).

Der **Derivation-Cache** ist allgemein: `(ContentHash, ProducerId) → Derivat` für Embeddings, Token-Zählungen, extraktive und abstraktive Zusammenfassungen, Entitätsextraktionen, Sprachdetektion. Eintrag = Truth-Tabelle `deriv` (Quelle, Produzent, Version, Zeit, Wert-/Blob-Referenz). Er ist **je Store**.

### 5.7 Statistik-Sidecar

`AccessStats { gid, reads, expands, last_access, salience }` liegt in einer separaten, **nicht versionierten** Struktur (periodischer Checkpoint, kein WAL-Eintrag pro Lesezugriff). Verlust ist unkritisch (nur Ranking-Prior). Der Sidecar speist Decay/Salience (Abschnitt 11.4) und das optionale gelernte Layout (8.9).

### 5.8 Handles

Für LLM-Kontext erhält jeder Record pro Branch einen **kurzen Handle** (4–6 Zeichen Crockford-Base32, ohne verwechselbare Zeichen, z. B. `c:k7qd`). Eindeutig und stabil je Branch; Tabelle `handle` ist Teil des Branch-Zustands. Unbekannter Handle → Fehler mit Vorschlägen („Meintest du `c:k7qd`, `c:k7qb`?", Edit-Distanz-Index). Halluzinierte IDs führen nie zu falschen Treffern.

---

## 6. Storage-Kernel

### 6.1 Verzeichnislayout

```
store/
  CURRENT                        # doppelt gepufferter Superblock (Verweis auf Manifest-Log-Tail)
  LOCK                           # exklusiver Prozess-Lock (flock/LockFileEx)
  wal/000042.wal …               # 64-MiB-Segmente, vorallokiert, ein gemeinsamer Log für alle Branches
  manifest/000007.mlog …         # append-only Manifest-Log (Strukturänderungen), CRC32C + Hash-Verkettung
  truth/seg-<id>.cseg            # Truth-Segmente (immutable)
  truth/death-<id>.cdth          # Death-Segmente (Ersetzungs-/Tombstone-Einträge, immutable)
  index/<seg-id>/<kind>-<paramhash>.cidx   # abgeleitete Index-Segmente (vec/ann, fts, csr)
  blobs/<aa>/<hash>              # große Inhalte (Quelldokumente, Artefakte), content-addressed
  catalog/…                      # Katalog-Tabellen sind normale Truth-Tabellen (kein eigenes Format)
  tmp/                           # Schreibziele vor atomarem Rename
```

Alle Dateien sind **write-once**; Ersetzung erfolgt durch neue Datei + Manifest-Eintrag. Nach dem Schreiben: `fdatasync(datei)`, `rename`, `fdatasync(verzeichnis)`. Unter Windows kapselt `DurableFs` die äquivalenten Operationen (`FlushFileBuffers`, `MoveFileEx(WRITE_THROUGH)`); die Persistenz-Semantik ist Teil der `DurableFs`-Testsuite.

### 6.2 Write-Ahead-Log (`contextra-log`)

- **Ein gemeinsamer Log** für alle Branches (Branch-ID im Record) → ein `fdatasync` pro Gruppe unabhängig von Branch-Anzahl.
- **Record:** `[len u32 | crc32c u32 | lsn u64 | type u8 | branch u32 | txn u64 | payload]`; `lsn` ist monoton und identisch mit `Seq` des Commits (ein Commit = ein Record oder eine begrenzte Folge mit Commit-Marker). Payload ist **logisch und idempotent** (Put/Retire/Link/…); Replay prüft `applied_lsn`.
- **Group Commit:** Leader/Follower. Gruppe schließt bei **≤ 200 µs oder ≥ 1 MiB**. Ein `fdatasync` je Gruppe; Ack erst danach (Klasse `Group`/`Strict`).
- **Torn Writes:** Erkennung durch Länge + CRC + monotone LSN; Recovery stoppt am ersten ungültigen Record (Präfix-Konsistenz, INV-2).
- **Keine HMAC-Kette, keine Legacy-Schlüssel.** (Die Hash-Verkettung der Löschbeweis-Ära entfällt; Integritätsschutz = CRC je Record, optional AEAD, 6.10.)
- **Segment-Recycling:** Segmente unterhalb von `checkpoint_lsn` (aus dem Manifest) werden wiederverwendet oder gelöscht.
- **Durability-Klassen** (je Namespace/Branch/Transaktion wählbar, [Entscheidung D-08]):

| Klasse | Verhalten | Einsatz |
|---|---|---|
| `Strict` | `fdatasync` je Commit (keine Gruppierung) | Finanzdaten, Kataloganpassungen |
| `Group` (Default) | Gruppen-`fdatasync`, Ack danach | `main`, kuratierte Branches |
| `Relaxed` | Ack nach `write()`, Sync alle 100 ms | Session-Branches |
| `Volatile` | nur RAM, kein WAL (wird beim Neustart verworfen) | Tests, Wegwerf-Experimente |

### 6.3 Memtable

Je Branch eine Memtable-Familie **(Records, Kanten, Vektoren, Mini-Invertindex)**:

- **Records/Kanten:** `crossbeam-skiplist`, sortiert nach `(gid, born_seq)` bzw. `(src, etype, dst, edge_id)`.
- **Vektoren:** append-only Arena je Space; Länge per Release/Acquire veröffentlicht. Bis **20 000** Vektoren **Flat-Scan (SIMD)**; darüber **Mini-HNSW im RAM** (nie persistiert; WAL deckt Verlust).
- **Mini-Invertindex:** Term → Posting-Liste (Hash-Map + Sortierung beim Lesen), BM25-Statistiken inkrementell.
- Frisch geschriebene Daten sind damit **sofort in allen drei Modalitäten durchsuchbar**, ohne auf Index-Builds zu warten (Lane-Vollständigkeit 100 % für den Memtable-Anteil).
- **Seal-Auslöser:** 64 MiB, 100 000 Zeilen, `COMMIT 'msg'` (explizit synchron) oder Druckstufe `Elevated`.

### 6.4 Truth-Segment (`.cseg`)

Ein Truth-Segment ist ein immutables, **spaltenorientiertes** Objekt mit Abschnittsverzeichnis. Spalten sind nach **Hot-Head / Cold-Tail** gruppiert, damit Ranking mit einem I/O auskommt und schwere Inhalte nur für Gewinner gelesen werden.

| Abschnitt | Inhalt | Zugriffsklasse |
|---|---|---|
| `head.keys` | `gid`, `record_id`, `row_kind`, `born_seq`, `branch`, `flags` (sortiert nach `(gid, born_seq)`) | hot, RAM-resident |
| `head.attrs` | `kind`, `class`, `level`, `parent`, `trust`, `taint`, `confidence`, `valid`, `acl`, Props mit niedriger Kardinalität (Dictionary-codiert) | hot |
| `head.tok` | `tok_count` je registriertem Tokenizer, `L2`-Gist | hot |
| `head.codes` | Quantisierte Embedding-Codes (nur wenn Space-Policy „Codes im Truth") | hot |
| `body.text` | Text in zstd-Blöcken (Level 3, trainierte Wörterbücher je Namespace-Klasse) | cold |
| `body.vec` | Rohvektoren je Space, 4-KiB-ausgerichtet, F16/F32, Zeilenreihenfolge = `ord` | cold (Rerank) |
| `body.props` | restliche Props (rkyv-Archive, Zero-Copy) | cold |
| `edges` | Kanten `(src, etype, dst, edge_id)` samt Props/valid/trust/prov | gemischt |
| `prov` | Provenienz-Batches | cold |
| `footer` | Statistiken (Zeilenzahl, Gid-Bereich, Term-/Grad-Histogramme, Min/Max je Spalte), Blockindex, Prüfsummen | hot |

- **Blockgrößen:** 16 KiB Standardblock; Prüfsumme (CRC32C) je Block; BLAKE3 über das Gesamtobjekt im Footer (für `scrub`).
- **Kompression:** zstd je Block (Text, Props); Vektoren unkomprimiert (F16) für O_DIRECT-Zero-Copy.
- **Zero-Copy:** Zeilen- und Spaltenzugriff über `rkyv`-Archive direkt auf Block-Cache-Rahmen; `bytecheck`-Validierung einmal beim Laden, danach Trust-Flag am Rahmen.

### 6.5 Index-Segmente (abgeleitet)

Index-Segmente (`.cidx`) werden **aus einem Truth-Segment** oder einer Folge von Truth-Segmenten gebaut und teilen dessen `ord`-Raum:

| Art | Inhalt | Details |
|---|---|---|
| `vec.flat` | nichts (Scan über `body.vec`) | Segmente < 50 000 Zeilen |
| `vec.hnsw` | Graph + SQ8-Codes | RAM-resident, für ≤ ~8 Mio. Zeilen je Segment |
| `vec.vamana` | Nachbarlisten in 4-KiB-Sektoren; Codes im RAM-Ladevorgang | SSD-Graph, Rerank aus `body.vec` |
| `fts` | FST-Wörterbuch, Postings (Blöcke à 128), Block-Max-Daten, optional Positionen, Längennormen | pro Analyzer-Konfiguration |
| `csr` | Vorwärts-/Rückwärts-Adjazenz, Elias-Fano-Offsets, delta-codierte Ziele, Kantentyp-Masken | Hub-Knoten mit Skip-Index |
| `bitmaps` | Roaring je Label/Low-Cardinality-Prop/ACL-Gruppe | Filter-Pushdown |

Segmentschlüssel = `(truth_segment_ids, kind, paramhash, builder_version)`. Index-Builds **dürfen nichtdeterministisch** sein (parallel, Seeds); optional deterministisch im Einzel-Thread-Modus für Reproduzierbarkeitstests. Das **Index-Manifest** (Teil des Manifest-Logs) bindet jedes Index-Segment an seine Truth-Quellen (`source_segments`) und hält das **Watermark** (6.7/8.7).

### 6.6 Liveness: Death-Segmente und Bitmaps

Jede Zeile gilt als **lebendig** zum Snapshot `S`, wenn `born_seq ≤ S` und **kein Death-Eintrag** `(gid, row_addr, died_seq ≤ S)` existiert. Death-Einträge entstehen beim Seal aus Updates und Tombstones.

- **Death-Segmente** (`.cdth`): sortierte Arrays `(segment, ord, died_seq)`, je Seal ein kleines immutables Segment; Compaction vereinigt sie.
- **LivenessMap** je Truth-Segment: Roaring32 `dead_head` = Zeilen mit Tod ≤ Head. Wird beim Publizieren eines neuen Manifests **copy-on-write** fortgeschrieben (persistente Struktur; Lesen ist wait-free).
- **Zeitreise:** `dead_at(S)` wird aus Death-Segmenten gefiltert (`died_seq ≤ S`) und je `(segment, Generation, S)` gecacht.
- **Branch-Overlay:** Zeilen, die ein Branch verändert, stehen in **dessen** Segmenten/Memtable; die Menge der berührten `Gid` (`Shadow(B)`, Roaring64) maskiert Zeilen der Vorfahren (Abschnitt 7.5).

Dies ersetzt Soft-Delete, Nachbarschafts-Reparatur und „Ghost-Pointer"-Nachweise des Ist-Standes: Tote Zeilen bleiben im ANN-Graph als **Routing-Knoten** (Ergebnisse werden maskiert) bis zur Compaction.

### 6.7 Manifest-Log und Watermarks

- **Manifest-Log** (`.mlog`): append-only Einträge `SegmentAdd`, `SegmentRetire`, `IndexAdd`, `IndexRetire`, `DeathAdd`, `BranchCreate/Update`, `CheckpointLsn`, `CatalogRoot`, `Statistics`. Jeder Eintrag: CRC32C + BLAKE3 des Vorgängers (Torn-Tail/Bit-Rot-Erkennung, **kein** Sicherheitsmerkmal).
- **Manifest-Zustand** im RAM als unveränderliches `Arc<Manifest>`; Publikation per `ArcSwap` je Branch → wait-free Leser.
- **Watermark** je Index-Segment: `source_segments` + `built_from_seq`. Ein Index deckt Zeilen **bis** `built_from_seq` ab; die Zeilen danach liegen in Memtable bzw. jüngeren Truth-Segmenten **ohne** fertigen Index und werden per Flat-Scan ergänzt (Abschnitt 8.7). Eine Abfrage meldet `lane_coverage` je Modalität.

### 6.8 Seal- und Checkpoint-Protokoll (Reihenfolge ist verbindlich)

1. Truth-Segment schreiben (`tmp/`), `fdatasync`, `rename`, Verzeichnis-`fdatasync`.
2. Death-Segment analog.
3. Index-Segmente **soweit synchron gebaut** (Standard: nur `vec.flat`/Mini-Strukturen; schwere Builds asynchron).
4. **Manifest-Eintrag** `SegmentAdd + DeathAdd (+ IndexAdd) + CheckpointLsn` anhängen, `fdatasync` → **Strukturpunkt**.
5. Superblock `CURRENT` doppelt gepuffert aktualisieren.
6. Memtable-Anteil bis `checkpoint_lsn` freigeben (Epochen-Reclaim), WAL-Segmente recyceln.

Absturz vor 4: Dateien sind verwaist und werden von der GC entsorgt, WAL-Replay stellt den Zustand wieder her. Absturz nach 4: Zustand vollständig im Manifest.

### 6.9 Compaction

- **Strategie:** größenabgestufte (Size-Tiered) Compaction der Truth-Segmente mit Faktor 4–8 und zeitlich zusammenhängenden Bereichen; getrennte Planung je **Branch-Lineage** (Overlay-Segmente werden beim Merge kompaktiert, nicht früher).
- **Auftrag:** Merge von Truth-Segmenten → Anwendung von Death-Einträgen **jenseits der Retention** (physische Entfernung) → neues Truth-Segment → Neuaufbau der Index-Segmente (HNSW: Neuaufbau; Vamana: Streaming-Merge nach FreshDiskANN-Prinzip oder Neuaufbau je Kostenmodell).
- **Segmentobergrenze für ANN:** 8 Mio. Zeilen je Index-Segment **[Ziel]**; größere Truth-Mengen werden in mehrere ANN-Segmente geteilt, damit Neuaufbaukosten und Speicherspitzen beherrschbar bleiben. Fan-out-Budget: ≤ 24 lebendige Segmente je Branch-Sicht in Steady-State; bei Überschreitung hebt der Planer die Compaction-Priorität an (Druckstufe).
- **Drosselung:** Compaction-I/O-Rate und Speicher (`max_memory`, `max_peak_memory`) sind begrenzt (Abschnitt 14); bei `Critical` pausieren Index-Neuaufbauten, Flush hat Vorrang.
- **Retention (D-06):** Eine Zeile ist **physisch entfernbar**, wenn (a) ein Death-Eintrag existiert, (b) `died_seq < retention_horizon(ns)` und (c) kein gepinnter Snapshot, Branch-Fork-Punkt oder benannter Tag sie noch referenziert. Es gibt **keinen Nachweis** der Entfernung (Nicht-Ziel).
- **Write-Amplification** ist die bewusste Kosten-Seite des LSM-Ansatzes und wird in Phase 0 gemessen (Abschnitt 17); Gegenmaßnahmen: großes Tier-Verhältnis für Bulk-Ingest, Hot-Head/Cold-Tail (Rewrite nur der betroffenen Abschnitte bei reinen Attribut-Merges, **[Ziel]**).

### 6.10 Verschlüsselung at Rest (optional, store-weit) [Entscheidung D-10]

- Aktivierbar je Store. **Ein Datenschlüssel (DEK)** je Store; der DEK wird durch einen **KEK** geschützt (Passphrase via Argon2id oder externe Schlüsseldatei/KMS-Hook über `KeyProvider`-Port).
- **AEAD** (XChaCha20-Poly1305 oder AES-256-GCM-SIV) je Block von WAL-, Truth-, Index- und Blob-Dateien; Nonce = `(file_id, block_no, generation)`; Dateikopf trägt Algorithmus und Schlüssel-ID.
- **Schlüsselrotation:** DEK-Rewrap (KEK-Wechsel) ist O(1); DEK-Wechsel geschieht lazy durch Compaction (neue Segmente mit neuem DEK).
- Hashes (`ContentHash`) werden über **Klartext** gebildet; Schlüssel fließen nie in Hashes ein.
- **Nicht enthalten:** Schlüssel je Person/Datensatz, Schlüsselvernichtung als Löschmechanismus, Widerrufsprotokolle.

### 6.11 Integrität und Prüfwerkzeuge

- CRC32C bei jedem Block-Read (Hot Path); BLAKE3 Objekt-Hash im Footer.
- **`scrub`** (Hintergrund/Befehl): liest alle Segmente, prüft Prüfsummen und Manifest-Verkettung, meldet defekte Dateien; defekte **Index**-Segmente werden automatisch neu gebaut, defekte **Truth**-Segmente führen zu Fehler `E_CORRUPT_TRUTH` mit Hinweis auf Backup-Restore.
- **`verify`:** Konsistenzprüfung Manifest ↔ Dateien ↔ Katalog ↔ `idmap`.

### 6.12 Backup, Restore, Replikationsfähigkeit

- **Backup = Menge von Dateien** zu einem konsistenten Manifest-Stand (Segmente sind immutable): inkrementell per Dateiliste (neue Segmente seit letztem Backup) + WAL-Tail; **Point-in-Time** = beliebiger `Seq` innerhalb der Retention.
- **Online-Snapshot:** `snapshot_export(seq)` pinnt Manifest und kopiert/verlinkt referenzierte Dateien (Hardlink auf demselben Volume, sonst Kopie; Fehler `E_CROSS_DEVICE` statt stiller Kopie, wenn `link_only` verlangt).
- **Replikation (nach v1):** Log-Shipping des WAL + Datei-Sync der immutablen Segmente; Follower bedienen Time-Travel-Reads. Das Format erzwingt dafür nichts Zusätzliches (Single-Writer-Log, immutable Dateien).

### 6.13 Garbage Collection

- **Roots:** Refs/Branches, Tags, aktive Snapshot-Pins (Epochen-Register), Retention-Fenster, Fork-Punkte aktiver Branches.
- **Mark/Sweep** über das Manifest (Dateien haben keinen Zeiger-Graphen wie ein Prolly-Baum; Erreichbarkeit = Manifest-Referenz). Verwaiste Dateien (Absturz vor Manifest-Eintrag) werden nach einer Karenzzeit gelöscht. Blobs: Referenzzählung über `content_hash`-Verwendung in Truth-Segmenten, Mark/Sweep beim Compaction-Lauf.
- **Pin-Lease:** Snapshots haben ein Lease (Default 300 s, verlängerbar). Abgelaufene Leases werden freigegeben; Abfragen erhalten `E_SNAPSHOT_EXPIRED` (retryable). Verhindert, dass vergessene Leser Retention und Speicher blockieren.

---

## 7. Transaktionen, MVCC und Branches (`contextra-mvcc`)

### 7.1 Zwei Ebenen der Nebenläufigkeit

1. **Innerhalb eines Branches: Snapshot Isolation (SI).** Leser sehen alle Zeilen mit `born_seq ≤ snapshot.seq` (und lebendig). Leser blockieren nie Schreiber. Ein **zentraler Commit-Sequencer** serialisiert die Commit-Reihenfolge (Leader der Gruppe), die Anwendung auf Memtables läuft je Branch parallel.
2. **Zwischen Branches: Git-Semantik.** Branches sind vollständig isolierte Welten (INV-6). Synchronisation nur über **Merge, Rebase, Cherry-pick**.

`Seq` ist **global** über alle Branches (eine Zeitachse, ein Log). Zeit-Zuordnung über `Hlc` je Commit; `AS OF TIMESTAMP` ist eine Binärsuche im monotonen Commit-Index.

### 7.2 Isolationsstufen

| Stufe | Default | Verhalten |
|---|---|---|
| `Snapshot` | **ja** | Write-Write-Konflikt je `Gid` (First-Committer-Wins). Agentenschreiben ist überwiegend append-only; SI genügt und ist billig. |
| `Serializable` | opt-in je Transaktion | Zusätzlich **Read-Set-Validierung** mit **Bereichs-/Prädikat-Tracking** (Phantomschutz): Konflikt, wenn nach `begin_seq` ein Commit eine gelesene `Gid` oder einen gelesenen Bereich (Gid-Bereich, Namespace+Prädikat, Graph-Nachbarschaft) verändert hat. Fail-closed bei `snapshot_seq < pruned_through` (`E_SNAPSHOT_TOO_OLD`). |

### 7.3 Transaktions-API und Typestate [löst Befund F5]

```rust
let tx: Txn<ReadWrite> = branch.begin(opts)?;     // pinnt Snapshot (branch, seq) und Lease
let rec = tx.get(gid)?;                           // Lesen UND Read-Set-Registrierung sind EINE Operation
tx.put(record)?; tx.link(edge)?; tx.retire(gid)?;
tx.commit(CommitOpts { idem: IdemKey, durability, message })?;
```

- **Es gibt keine API**, die Lesen ohne Registrierung oder Registrierung mit fremdem Snapshot erlaubt. `get`, `scan`, `search` einer `Txn` verwenden zwingend deren Snapshot; ein Parameter `snapshot_seq` existiert **nicht** (Fehlerklasse „`u64::MAX`-Registrierung" ist konstruktiv ausgeschlossen).
- `Snapshot<'db>` ist lifetime-gebunden: Zero-Copy-Referenzen (`&'snap Archived…`) überleben den Pin nicht.
- `Txn<ReadOnly>` vs. `Txn<ReadWrite>` per Typestate. Reine Leseabfragen (Suche) laufen auf `Snapshot` ohne Read-Set (SI-Lesen braucht kein Tracking); Suche innerhalb einer `Serializable`-Transaktion registriert den **Prädikatsbereich**, nicht jedes Ergebnis.

**Commit-Validierung (Reihenfolge):**
1. Schema, Typen, Kardinalitäten, **Pflichtfeld Provenienz**, Größenlimits (Abschnitt 14.3).
2. Idempotenzprüfung (`idem`-Tabelle, 7.8).
3. OCC: Write-Set gegen Commits seit `begin_seq` (SI) bzw. Read-Set + Prädikate (Serializable).
4. Write-Time-Konfliktprüfung (funktionale Prädikate, 11.5) → erzeugt Konflikt-Records oder schließt bitemporal.
5. Gate-Hooks des Namespace (nur bei Merge nach `main`, 7.6).
6. WAL-Group-Commit; Apply; Snapshot publizieren.

### 7.4 Branches und Sichtbarkeit

**Branch** = benannter Zeiger `(BranchId, parent, fork_seq, head_seq, policy)`. **Fork** kopiert nichts: ein neuer Eintrag im Manifest-Log plus leere Memtable-Familie. Zeit **O(1)**.

**Sichtbarkeitsregel.** Sei `Lineage(B) = [(B, S), (P1, f1), (P2, f2), …]` mit `S` = Snapshot-Seq in `B`, `f1` = Fork-Seq von `B` in `P1`, `f2` = Fork-Seq von `P1` in `P2` (`f2 < f1`). Für eine `Gid g` ist die **sichtbare Content-Zeile** (und analog die sichtbare Attr-Zeile, 5.2) die Zeile mit

1. kleinster Lineage-Tiefe (eigener Branch vor Elternteilen) und innerhalb einer Tiefe größtem `born_seq ≤` Schranke der Tiefe;
2. Ist diese Zeile ein **Tombstone**, ist `g` unsichtbar.

**Effiziente Ausführung:** Zeilen von Vorfahren werden mit der Maske `¬(dead_at(Schranke) ∪ Shadow(B))` gelesen, wobei `Shadow(B)` = Roaring64 der in `B` (oder Zwischengliedern) berührten `Gid`. Eigene Memtable-/Overlay-Segmente liefern die aktuelle Version. Eine Abfrage fragt also `Vorfahr-Segmente (maskiert) + Overlay-Segmente + Memtable` und fusioniert die Top-k-Listen.

**Segment-Explosion vermeiden:** Session-Branches halten Schreibzugriffe in der Memtable (flache Suche), versiegeln erst ab Schwelle (Default 64 000 Zeilen) zu **einem** kleinen Segment und kompaktieren beim Merge.

**Branch-Arten:** `main`; `dev/<name>` (langlebig, `Group`); `sess/<agent>/<id>` (kurzlebig, `Relaxed`, Quote + TTL); `mr/<n>` (Merge-Request, 7.6); `tag/<name>` (unveränderlicher Zeiger, pinnt Retention).

### 7.5 Diff

Diff `A → B` ist die **Menge der in `B`-Lineage seit dem Fork berührten `Gid`** (aus `Shadow` und Overlay-Segmenten) verglichen mit der Sicht von `A`; Kosten **O(Δ · log N)**. Ausgabe: Strom `(gid, old, new, Klasse)` mit `Added | Removed | Modified{felder} | Moved`. Eine **semantische Schicht** gruppiert nach Entität/Konzept, hängt Provenienz an und erzeugt eine kompakte, LLM- und menschenlesbare Zusammenfassung („+12 Fakten zu *Kubernetes*, 3 ersetzt, 1 Konflikt").

### 7.6 Drei-Wege-Merge und Merge-Gates

**Merge-Basis** = letzter gemeinsamer Fork-Punkt (lineare Lineage-Struktur; Criss-Cross tritt durch die Baumstruktur der Branches nicht auf; Rebase/Cherry-pick erzeugen keine Mehrfachbasen). **Merge-Walk** über die vereinigten Δ-Mengen beider Seiten:

| Situation | Regel |
|---|---|
| nur eine Seite hat `g` geändert | übernehmen |
| beide identisch (gleicher Inhalt + Attribute) | übernehmen |
| `Record`: disjunkte Felder | feldweise automatisch zusammenführen |
| gleiches Feld verschieden oder `content_hash` beidseitig verschieden | **Konflikt** (Embeddings folgen dem Inhalt und werden nie „semantisch gemergt") |
| `support` (Zähler) | additiv: `base + Δours + Δtheirs` |
| `trust` | strengere (niedrigere) Stufe; Hochstufung nur durch ein Gate/privilegierten Prinzipal |
| `Edge` | Mengensemantik, **Add-Wins** für Add/Add und Add/Delete (konfigurierbar je Kantentyp: `AddWins` · `RemoveWins` · `Manual`); Add/Add identisch = eine Kante |
| `emb`, `deriv`, `prov`, `idmap` | nie Konflikt (content-addressed bzw. append-only, Vereinigung) |
| `Conflict`-Records | Vereinigung, Deduplizierung über `(a,b)` |
| Katalog/Schema | Kompatibilitätsprüfung, sonst manuell |
| abgeleitete Records (`level > 0`, `Summary`) | **nicht mergen**; `STALE` markieren und neu erzeugen (Work Queue) |

**Merge-Ausführung:** Das Netto-Δ des Branches wird **als ein atomarer Commit in das Ziel geschrieben** (neue Versionen mit `merged_from`-Provenienz). Embeddings werden über `ContentHash` wiederverwendet (keine Neuberechnung); Indexe für die neuen Zeilen entstehen im regulären Seal. Hat sich das Ziel seit Beginn bewegt, wird gegen die neue Basis wiederholt (begrenzte Schleife, Default 3) oder abgelehnt. **Konflikte leben im `mr/<n>`-Branch** (INV-7). Strategien: `fast-forward` (Ziel unverändert), `merge`, `squash`, `rebase`, `cherry-pick` (ausgewählte Records inkl. Provenienz-Abschluss und zugehöriger Kanten).

**Merge-Gates** (Pipeline vor Promotion nach `main`, je Namespace konfigurierbar):

| Gate | Prüfung | Ergebnis |
|---|---|---|
| G1 Schema/Provenienz | Typen, Pflichtfelder, jede Aussage mit Quelle oder Ableitungsnachweis | Verstoß → Reject |
| G2 Vertrauen | `confidence ≥ τ`; Records von `trust < min_main_trust` nur mit Review | Reject/Review |
| G3 Konflikte | Widerspruchsscan der berührten Subjekte; funktionale Prädikate; Kardinalität; referentielle Integrität | Konflikt-Records, Review |
| G4 Blast-Radius | Δ-Größe, Anteil gelöschter Records, Rate; Ausreißer stoppen den Merge | Review |
| G5 Zitierprüfung | bei Antwort-/Fakt-Records: Citation-Binding auf existierende Chunks; optional NLI-/Entailment-`Judge` | Reject/Quarantäne |
| G6 Taint-Cap | Records mit `INJECTION_SUSPECT`/`EXTERNAL` werden auf `trust ≤ Untrusted` gedeckelt oder verworfen | Cap/Reject |
| G7 Review-Hook | Mensch/LLM-Reviewer über Port (`Reviewer`) | Approve/Reject |

Gates sind **deterministische, auditierbare Funktionen** über Diff + Katalog (außer G5/G7 mit Judge/Reviewer-Port). Ergebnis wird im `mr/<n>`-Objekt gespeichert. Ein Merge nach `main` ohne bestandene Gates ist nur für privilegierte Prinzipale möglich und wird im Commit vermerkt.

### 7.7 Rollback, Savepoints, Session-Abort

- **Savepoint** = `(seq)` innerhalb des Branches. `ROLLBACK TO sp` = Branch-Head auf `seq` zurücksetzen (Ref-Update im Manifest-Log, O(1)); spätere Commits werden als verworfener `seq`-Bereich markiert (`aborted_ranges`), für Leser unsichtbar, bei Compaction ignoriert.
- **Session-Abort** = Branch löschen (der Reflog im Manifest-Log behält ihn bis Ablauf von `branches_days`, zur Fehleranalyse und zum Replay).
- **Atomarität:** Der einzige atomare Punkt ist das Manifest-/Head-Update. Entweder ist der alte oder der neue Zustand vollständig sichtbar, auch bei Merges.

### 7.8 Idempotenz

`idem`-Tabelle: `idem_key → (seq, result_summary, expires_at)`. Schlüssel kommt vom Client (Agenten **müssen** wiederholen können). Wiederholung innerhalb der Frist (Default 24 h) liefert das gespeicherte Ergebnis (INV-5). Erzeugt von MCP-Schreibwerkzeugen automatisch aus `(session, tool_call_id)`.

### 7.9 Zeitreise (bitemporal)

- `AS OF SEQ n` / `AS OF TIMESTAMP t` (Transaktionszeit): Snapshot-Auflösung über den Commit-Index.
- `VALID AT t` (Gültigkeitszeit): Filter auf `valid`-Intervalle; Zeilen ohne Gültigkeitsangabe gelten als gültig (explizit **Fail-Open** für Zeit, aber **Fail-Closed** für ACL/Trust).
- Kombination `AS OF TX t1 VALID AT t2` beantwortet „Was glaubten wir am 1. März über den Zustand am 1. Januar?".
- **Retention-Grenze:** Zeitreise ist nur innerhalb der Retention möglich; ältere Abfragen liefern `E_BEYOND_RETENTION` mit der frühesten verfügbaren `Seq`.
- **Entitäts-Historie:** jeder Commit führt eine Touched-Gids-Zusammenfassung (Bloom + Sortierliste bei kleinen Commits), damit `HISTORY(gid)` nicht alle Segmente abtastet.

---

## 8. Index-Layer

### 8.1 Gemeinsamer Adressraum und Filter-Pushdown

Alle Index-Operatoren eines Segments arbeiten auf dessen `ord`-Raum. Filter entstehen im `Gid`-Raum als `Roaring64` und werden **pro Segment** in `Roaring32`-`ord`-Bitmaps übersetzt:

1. Quellen der Bitmap: ACL (Gruppen-Bitmaps), Zeit (`valid ∩ as_of`), Trust/Taint, Label/Prop-Bitmaps, **Graph-Frontier** (Ergebnis einer Expansion), Namespace-Zugehörigkeit, `¬dead`, `¬Shadow(B)`.
2. Übersetzung: `head.keys.gid` ist sortiert → Bereichs-Slicing/Galloping-Merge; Ergebnis je Query und Segment gecacht.
3. **Dieselbe Bitmap** schränkt ANN-Traversierung (als Prädikat), BM25-Postings (Block-Skipping) und Flat-Scan ein.

Dies ist der Kern des „ohne Duplikation hybrid"-Versprechens; die Übersetzungskosten sind sein Preis (Abschnitt 18, R-03).

### 8.2 Vektor (`contextra-vector`)

**Segmentarten:** `flat` (< 50 000 Zeilen, exakt, SIMD) · `hnsw` (RAM; Default für `lite`/`standard`) · `vamana` (SSD; `scale`). **Immutable**; Neuaufbau bei Compaction. Mutierbarer Delta-Anteil nur in der Memtable (6.3).

| Parameter | HNSW | Vamana |
|---|---|---|
| Aufbau | `M = 16`, `ef_construction = 200` | `R = 64`, `L_build = 100`, `α = 1,2` |
| Suche | `ef_search` adaptiv 16…256 | Beam-Suche (Beamweite 8), ein Sektor-Read je Hop |
| RAM | SQ8-Codes + Graph | PQ-/RaBitQ-Codes, Einstiegspunkte, oberste Ebenen |
| Rerank | Top-2k mit Vollpräzision aus `body.vec` | wie links |

- **Quantisierung:** SQ8 (Perzentile 0,5 %/99,5 %) als Default; PQ/OPQ und **RaBitQ** (1–4 Bit) für `scale`; **Matryoshka-Präfixe** für Vorfilter (kurzer Präfix → Kandidaten, voller Vektor → Rerank). Quantisierungsparameter sind Teil der Segmentidentität (kein „Rekalibrieren im laufenden Index"; Drift wird beim Neuaufbau behoben).
- **Distanz:** Cosine/Dot/L2 über `contextra-simd` mit Runtime-Dispatch; Vektoren mit NaN/∞ oder falscher Dimension werden am Commit abgelehnt.
- **Gefilterte Suche** (Selektivität σ = |S|/N, `S` = Filtermenge im Segment):

| Strategie | Bedingung | Verfahren |
|---|---|---|
| (A) exakter Pre-Filter | `|S| ≤ τ` (Default 20 000) | SIMD-Distanzen direkt über `body.vec`-Zeilen von `S`, nach `ord` sortiert gelesen |
| (B) gefilterter Graph-Beam | `0,1 ≤ σ < 0,5` | Bitmap als Prädikat im Beam; bei σ < 0,1 Zwei-Hop-Expansion (ACORN-Prinzip) |
| (C) Post-Filter | `σ ≥ 0,5` | adaptiver Over-Fetch |
| **Laufzeit-Fallback** | (B) liefert nach `ef` Pops < k gültige Treffer | Wechsel auf (A) |

- **Frühabbruch:** Alle Segmente teilen eine atomare k-te-Score-Schwelle.
- **Mehrere Spaces je Record:** je Space eigene Index-Segmente; Abfragen nennen den Space.

### 8.3 Volltext (`contextra-text`)

- **Segment:** FST-Termwörterbuch, Postings in 128er-Blöcken (Delta + SIMD-Bitpacking), Term-Frequenzen, **Block-Max-Scores** für Block-Max-WAND/MaxScore, Positionsströme optional (Phrasen/Nähe), 1-Byte-Längennormen je Feld.
- **Analyzer-Pipeline** (ID + Parameter im Segmentschlüssel): Unicode-Segmentierung (UAX #29), Lowercasing, Umlaut-/ß-Normalisierung, **Spracherkennung je Chunk**, Snowball-Stemming, **deutsches Decompounding** (DP-Zerlegung mit Fugenelementen, Wörterbuch; Domänenlisten Medizin/Recht sind produktiv zu befüllen oder zu entfernen, nicht als Stub zu führen), N-Gramme für Code/Bezeichner/Artikelnummern.
- **BM25F ist Standard:** Felder `title` (Gewicht 2,0), `heading_path` (1,5), `body` (1,0); `k1 = 1,2`, `b = 0,75` je Feld konfigurierbar. **Globale Statistiken** (`N`, `df`, `avgdl`) werden aus den Segment-Footern im Manifest aggregiert, damit Scores über Segmente und Branches **vergleichbar** sind. Frische Memtable-Anteile nutzen dieselben aggregierten Statistiken plus Delta.
- **Filter:** Bitmap überspringt Postings-Blöcke komplett; BM25 läuft „innerhalb einer Graph-Nachbarschaft" ohne Nachfilterung.

### 8.4 Graph (`contextra-graph`)

- **Truth:** sortierter Kantenbestand `(src, etype, dst, edge_id)` im Truth-Segment; Adjazenz = Bereichsscan.
- **Index:** **CSR-Segmente** je Richtung (Offsets Elias-Fano, Ziele delta + bitgepackt, Kantenzeiger für Props), Typmasken, Temporal-Pushdown (`valid ∩ as_of`) über Spalten neben dem CSR, Hub-Knoten (Grad > 10⁴) mit Skip-Index. Frische Kanten: sortiertes Delta-Overlay in der Memtable. **Knoten = `Gid`**; es gibt keine zweite Knoten-Identität.
- **Operatoren:** richtungsoptimierte BFS mit Frontier-Bitmap und Budget-Abbruch; **k-Hop** (Default max 3, max 10 000 besuchte Knoten); **Personalized PageRank per Forward-Push** (ε ≈ 1e-4, α = 0,85) als **einziger** PPR-Algorithmus; Pfad-Konfidenz `C(P) = (∏ conf(eᵢ)) · γ^(k−1)` (γ = 0,9) als Score-Faktor für abgeleitete Kanten; **Leiden-Communities** als Batch-Job (deterministisch mit Seed), Ergebnis speist Community-Summaries (Abschnitt 11.3).
- **Kanten sind erstklassig:** Typ, Props, `valid`, `trust`, `confidence`, `prov`. Kantenmengen folgen **Add-Wins-Semantik** (Qwen-Idee, CRDT-artig); Tombstone-Kanten bleiben bis zur Retention.
- **Entfallen** (Ist-Funktionen, begründet in 16.1): TL-HFD, APPRH, k-Path-Diffusion, PathRAG (nicht snapshot-fähig), Perkolation/Re-Bonding, Kantenverstärkungs-Lernen, synaptische Kanten, separater Hyperkanten-Speicher samt Kaskaden-Queue. n-äre Relationen = reifizierte Facts. Jede Wiedereinführung erfordert Benchmark-Nachweis gegen den Standard (Abschnitt 19, D-09).
- **Referentielle Integrität:** Kanten auf `Gid`, die zu `main` nicht (mehr) sichtbar sind, werden beim Lesen ausgeblendet; beim Merge meldet G3 „dangling edge". Kein kaskadierendes Löschen: Retire eines Knotens retired **nicht** automatisch seine Kanten; die Sichtbarkeit der Kante folgt der Sichtbarkeit beider Endpunkte (Lesezeit-Join über Frontier-Bitmap).

### 8.5 Entscheidung: ein Algorithmus je Aufgabe

| Aufgabe | Standard | Zurückgestellt (nur mit Nachweis) |
|---|---|---|
| Vektor-ANN | HNSW, Vamana | DiskANN-Varianten über Vamana hinaus |
| Quantisierung | SQ8, PQ, RaBitQ | INT4/FP16-Mixed-SIMD |
| Lexikalisch | BM25F + Block-Max-WAND | Learned Sparse (SPLADE) |
| Graph-Relevanz | k-Hop + Forward-Push-PPR | Hyperkanten-Diffusion, Perkolation |
| Fusion | RRF (k = 60) + gewichtete Norm-Fusion | gelernte Fusion (nur auf Feedback-Basis) |
| Rerank | Cross-Encoder (ONNX/Candle) optional | LLM-Rerank |

### 8.6 Statistiken für den Planner

Segment-Footer und Manifest liefern: Zeilenzahlen, `df` je Term (Top-N Terme exakt, Rest Histogramm), Roaring-Kardinalitäten der Filter-Bitmaps, Gradhistogramme, Min/Max je Spalte, Selektivitätsschätzer je Prop. Der Planner nutzt sie für Reihenfolge und Strategiewahl (9.2).

### 8.7 Watermarks und Index-Build-Jobs [löst Befund F1]

- Jeder Index-Build ist ein **Job** in der dauerhaften Jobtabelle (`job`, Truth-Tabelle): `(kind, source_segments, params, state, lease)`. Jobs sind **idempotent** und wiederholbar.
- **Eine Abfrage mit Snapshot `S` liest je Modalität:** `Index-Segmente (bis built_from_seq) + Flat-/Mini-Scan über alle Zeilen ohne fertigen Index (Memtable, junge Segmente)`; alle Kandidaten werden gegen den Truth-Snapshot validiert (INV-3). Damit entsteht **keine Suchlücke**, nur eine **Latenz-/Recall-Verschiebung**, die als `lane_coverage` und `degraded`-Warnung im Ergebnis steht.
- Ein Index-Build-Fehler ändert keinen Commit und keine Daten (P1). Es gibt **keine kompensierenden Transaktionen**; Fehler führen zu Job-Retry mit Backoff und Alarm nach N Fehlern.
- Beim Start: fehlende oder defekte Index-Segmente werden neu eingeplant; Abfragen laufen sofort (Flat-Fallback).

### 8.8 Index-Parameter-Evolution

Parameter (`M`, `R`, Analyzer, Space) sind Teil des Segmentschlüssels. Eine Änderung erzeugt neue Index-Segmente neben den alten (Doppelbetrieb), Umschaltung per Katalog-Commit, Aufräumen per GC. Kein Online-Umbau eines bestehenden Index.

### 8.9 Zurückgestellt: gelerntes Layout

Affinity-Re-Layout (Co-Retrieval-Clustering aus Access-Traces, MNEMOS 5.3) bleibt Backlog. **Reaktivierung**, wenn (a) ein Benchmark-Workload mit wiederkehrenden Co-Retrieval-Mustern ≥ 20 % Cold-Tail-I/O-Reduktion im Prototyp zeigt und (b) der Hot-Head/Cold-Tail-Schnitt (6.4) allein die Zielwerte nicht erreicht.

---

## 9. Query-Engine und hybrides Retrieval

### 9.1 Abfrageschnittstelle

Primär: typisierter **Rust-Builder** und **JSON-Plan** (stabil, versioniert, über gRPC/MCP/HTTP identisch). Keine eigene Textsprache in v1 [D-14].

```rust
let pack = branch.query()
    .namespaces(["wissen/handbuecher", "wissen/tickets"])
    .text("Wie skaliere ich Pods automatisch?")          // BM25F-Leg
    .vector(Space("emb-de-1024-v2"), Embed::Auto)          // Vektor-Leg (Embedding via Port/Cache)
    .expand(Graph::khop(2).via(["verwandt_mit","teil_von"]).score(Ppr::default()))
    .filter(Filter::trust_at_least(Trust::Curated).valid_at(Now).acl(principal))
    .fuse(Fuse::rrf(60).weights([1.0, 0.7, 0.4]))          // vector, bm25, graph
    .prior(Prior::decay_class("volatil").recency_half_life(days(30)))
    .rerank(Rerank::cross_encoder("bge-reranker-v2").pool(100))
    .pack(Pack::budget(3500, Tokenizer("o200k")).zoom(Adaptive).mmr(0.7)
              .placement(Placement::UShape).cache_friendly(true))
    .as_of(AsOf::Head).now(clock.now())
    .run()?;                                                // → EvidencePack
```

### 9.2 Logischer und physischer Plan

Operatoren: `Scan/Filter`, `VectorSearch`, `Bm25Search`, `GraphExpand`, `PriorScore`, `Fuse`, `Rerank`, `Validate`, `Pack`.

**Planner-Entscheidungen** (kostenbasiert, aus 8.6):
- **Reihenfolge:** selektive Filter/Graph-Frontiers zuerst (liefern Bitmap für Vektor und BM25); bei unselektiven Filtern parallel generieren und nachfusionieren.
- **Graph-Seeds:** übergebene Anker-Entitäten; sonst die Top-Treffer der Text-/Vektor-Legs (nicht „alle Text-Treffer", Seed-Menge ist begrenzt, Default 50).
- **Strategie je Segment:** A/B/C aus 8.2 nach `σ`.
- **Parallelität:** segmentweise Fan-out auf dem CPU-Pool; I/O auf dem I/O-Reaktor; begrenzt durch Semaphoren (Abschnitt 14).
- **Adaptive Pool-Größe:** der Kandidatenpool je Leg (`3·k` Default) wird per einfacher Regel an das Latenzbudget angepasst (Ziel-p95 konfigurierbar); **kein** PID-/Bandit-Apparat.

### 9.3 Fusion und Scoring

- **RRF:** `s(d) = Σᵢ wᵢ / (k + rankᵢ(d))`, `k = 60`; Signale mit Gewicht ≤ 0 oder ohne Treffer entfallen; Provenienz je Dokument: `{raw_score, rank, contribution}` je Signal (Explain).
- **`Norm`-Fusion:** robust geclippte Min-Max-Normalisierung je Leg + CombSUM; Fallback auf RRF bei degradiertem Signal.
- **Priors (multiplikativ, konfigurierbar):** Trust-Faktor, Frische/Decay (11.4), Wichtigkeit/Salience, **Pfad-Konfidenz** (Graph-Leg), Community-Boost (`same_community_as`, Default ×1,2).
- **Aktualität ist kein Ersatz für Korrektheit:** ein verifizierter alter Fakt kann höher gewichtet werden als ein neuer unbelegter Agenten-Claim (Trust-Faktor dominiert Frische im Default-Profil).
- **Rerank:** optionaler Cross-Encoder auf Pool `min(10·k, 200)`; Deadline (Default 500 ms); bei Timeout/Fehler Ergebnis ohne Rerank und Warnung (`rerank_skipped`).

### 9.4 Kalibrierung und Abstention

Die Fusion kann per **isotonischer Regression** aus Feedback (Klicks, `expand`-Rate, Reviewer-Labels) auf eine Relevanzwahrscheinlichkeit kalibriert werden (optional). Daraus entsteht das Feld `evidence_sufficiency ∈ {Sufficient, Weak, None}`: Liegt die beste kalibrierte Relevanz unter der Schwelle oder fehlt Provenienz, meldet das Evidence Pack **`None`** statt Treffer „aufzufüllen"; Agenten SOLLEN dann antworten „unbekannt". Kalibrierstand ist modell-/space-gebunden und verfällt bei Modellwechsel (Fingerprint-Vergleich).

### 9.5 Validierung gegen Truth

Jeder Kandidat wird vor der Ausgabe geprüft: sichtbar im Snapshot (Lineage-Regel), nicht `TOMBSTONE`, `ACL` erfüllt (erneut, defensiv), `valid`/`trust`-Filter erfüllt. Verstöße werden verworfen und im Explain als `validation_drop` gezählt (nicht inhaltlich offengelegt, INV-4).

### 9.6 Begrenzungen

`MAX_K = 1 000`; `DEFAULT_SCAN_LIMIT = 10 000`; `HARD_SCAN_CEILING = 100 000`; Query-Text ≤ 64 KiB; Query-Zeitbudget (Default 2 s, Deadline-Propagation in alle Operatoren; bei Ablauf Teilergebnis mit `partial: true`).

---

## 10. Context Engine (`contextra-pack`)

Die Context Engine überführt Kandidaten in ein **Evidence Pack**, das ein LLM direkt verwenden und zitieren kann.

### 10.1 Eingabe und Ausgabe

**Eingabe:** fusionierte, validierte Kandidaten mit `relevance`, `trust`, `taint`, `conflicts`, `valid`, Zoom-Stufen `(level, tok_count[tokenizer], gain)`, Embedding-Kopf (für MMR), Eltern-Verweis.

**Ausgabe `EvidencePack`:**

```rust
struct EvidencePack {
    items: Vec<PackItem>,              // geordnet gemäß Platzierung
    budget: TokenBudget { limit, used, tokenizer },
    warnings: Vec<Warning>,            // Conflict | Stale | LowTrust | Tainted | PartialIndex | ValidationDrops
    sufficiency: Sufficiency,          // aus 9.4
    prefix_digest: Option<Hash>,       // Hash der stabilen Präfixblöcke (KV-Cache-Schlüssel)
    explain: ExplainId,                // 10.6
    snapshot: SnapshotRef,             // Branch, Seq, now → Reproduzierbarkeit (INV-10)
}
struct PackItem { handle, level, text, source: SourceRef, trust, valid, prov_summary, warnings }
```

### 10.2 Zoom-Knapsack

Für Kandidaten *i* mit Stufen *ℓ*: Kosten `c[i][ℓ]` (Token), Nutzen `u[i][ℓ] = rel_i · gain(ℓ) · trust_i · fresh_i`. Wähle je Kandidat **genau eine Stufe oder „weglassen"**, Σ Kosten ≤ Budget, Σ Nutzen maximal unter Redundanzabzug. Praxis (Multiple-Choice-Knapsack):

1. Pro Kandidat obere konvexe Hülle der `(Kosten, Nutzen)`-Punkte.
2. Start: alle auf gröbster Stufe (Handle/Gist).
3. Greedy über Upgrade-Schritte nach größtem `Δ-Nutzen/Δ-Kosten` (Max-Heap) bis Budget erschöpft; auf der Hülle ist das die optimale LP-Relaxation und praktisch nahezu optimal.
4. **MMR dynamisch:** Nutzen sinkt, wenn semantisch ähnliche Items bereits gewählt (Cosinus auf SQ8-Köpfen; Schwelle `> 0,92` = Duplikat). MMR läuft als **Stream-Operator** und bricht ab, sobald das Budget erreicht ist.

Ergebnis: viele **grobe** plus wenige **feine** Treffer statt weniger vollständiger Chunks; der Agent steuert per `expand` nach.

### 10.3 Small-to-Big

Bei hoher Chunk-Relevanz und freiem Budget ersetzt der Packer Chunk-Folgen desselben Abschnitts durch den **Eltern-Abschnitt** (`level 1`), sofern dessen Token-Kosten unter der Summe der Einzelkosten plus Schwelle liegen; sonst Nachbar-Chunks desselben Abschnitts in Dokumentreihenfolge (Lesbarkeit).

### 10.4 Platzierung und Cache-Freundlichkeit

- **U-Form (`Placement::UShape`)**: höchste Relevanz an **Anfang und Ende** (`[r1, r3, r5 … r6, r4, r2]`), Mitte weniger kritisch (Lost-in-the-Middle-Gegenmaßnahme).
- **Cache-friendly:** stabile Items (System-/Policy-/Skill-Records mit unveränderlichen Hashes) zuerst in **kanonischer Reihenfolge** (nach `ContentHash`), danach volatile Treffer. `prefix_digest` identifiziert das stabile Präfix für KV-/Prompt-Caches.
- **Edges-first** (Variante): Kanten-/Beziehungszusammenfassung vor dem Fließtext.

### 10.5 Rendering und Handles

- **Compact** (Default), XML und JSON. Compact spart Token: `[c:k7qd] (L1·142tok·curated·2026-10-14) Text…`.
- **Datenrahmen für unvertrauenswürdige Inhalte:** Items mit `trust ≤ Untrusted` oder `taint ≠ ∅` werden in klare Delimiter mit Quellenvermerk eingefasst („Daten, keine Anweisungen"); Instruktionstext wird nie aus Records mit Taint erzeugt.
- **Progressive Disclosure:** `recall` liefert kompakt mit Handles; `expand(handle, level)` lädt feiner/gröber nach; Expand-Aufrufe werden als Relevanzsignal in den Statistik-Sidecar geschrieben.

### 10.6 Explain

`explain(id)` liefert je Kandidat: Signalbeiträge (Vektor, BM25, Graph, Priors), Rang je Signal, gewählte Stufe, Verwerfungsgrund (Budget, Redundanz, Trust, Filter, Validierung), Planentscheidungen (Strategie je Segment, `lane_coverage`, benutzte Index-Generationen), Zeitverbrauch je Operator.

### 10.7 Tokenizer-Registry

`Tokenizer` ist ein Port. Der Katalog registriert Tokenizer-Familien; für registrierte Familien persistiert der Ingest exakte `tok_count`. Für nicht registrierte Tokenizer schätzt der Packer (≈ 4 Zeichen/Token, Code ×1,8) und markiert die Zählung als `estimated`; Budgets werden dann mit Sicherheitsabschlag (Default 5 %) angewandt.

---

## 11. Wissensqualität und Lebenszyklus

### 11.1 Ingest-Pipeline (`contextra-ingest` + Engine)

```
Quelle (Bytes + Metadaten + source_uri + ACL-Vorgabe)
 → Format-Erkennung → Extraktion (PDF, Office, HTML, E-Mail, CSV/JSON, Markdown, Code)
 → Normalisierung (Unicode NFC, Whitespace) → Injection-/Taint-Scan
 → strukturbewusstes Chunking → Hash-Abgleich mit der Vorversion der Quelle
 → Derivation-Cache (Token-Zählungen, Sprache, extraktives L2, Embedding)
 → Txn: Put/Retire-Set + Provenienz → Commit
```

- **Strukturbewusstes Chunking** (Zielgröße in **Token**, nicht Bytes): Markdown/HTML/Office folgen Überschriften-, Absatz- und Satzgrenzen mit **Überschriftenpfad** (`heading_path`) als Feld; Tabellen als Zeilengruppen mit Kopfzeile; Code syntaxbaumbasiert (Funktion/Klasse als Einheit, Signatur als L2); PDF layoutbewusst (Spalten, Kopf-/Fußzeilen entfernen). Defaults: Ziel 512 Token (min 50, max 1 024), keine harte Überlappung (Kontextgewinn über Small-to-Big). Chunker-ID und -Version werden je Record vermerkt (reproduzierbares Re-Chunking).
- **Revisions-Diff (Kern der Inkrementalität):** Eine Quelle (`source_uri`) hat eine geordnete Chunk-Liste. Beim erneuten Ingest werden neue Chunks per `ContentHash` den alten zugeordnet:
  - gleicher Hash → **Identität bleibt** (`gid`, Embedding, Zusammenfassungen, Kanten, Statistiken);
  - neuer Hash → neue Version bzw. neuer Record; Embedding via Cache oder Port;
  - fehlend → `retire` (Tombstone-Commit, mit `Supersedes`-Spur wo ein Nachfolger erkannt wird).
  Der gesamte Revisionsvorgang ist **eine Transaktion**; Leser sehen entweder die alte oder die neue Dokumentversion.
- **Quelldokument-Ablage:** Standard speichert **extrahierten Text + Hash + URI** (ressourcenschonend). Optional (`keep_source = true`) das Original als Blob.
- **Embedding-Platzierung:** Embeddings werden **vor** dem Commit berechnet (Pipeline-Stufe, aus Cache oder Port), damit ein committeter Chunk in allen Modalitäten auffindbar ist. Bulk-Modus `embedding = Deferred` committet Chunks sofort (lexikalisch + Graph auffindbar) und füllt den Vektor-Lane per Job nach; Abfragen melden `lane_coverage` (P8).
- **Batching und Backpressure:** Embedding-Anfragen werden gebündelt (Batchgröße je Adapter, Default 32); bei Druckstufe `Elevated` verlangsamt der Ingest, bei `Critical` blockiert er mit `E_BACKPRESSURE` (retryable).
- **Near-Duplicate-Erkennung** (optional): MinHash/SimHash auf Chunk-Ebene; Treffer erzeugen `DuplicateOf`-Kanten statt Löschung (INV-9).
- **Konnektoren** (SharePoint, Confluence, Dateisysteme, IMAP …) liegen **außerhalb** des Kerns und rufen die Ingest-API; der Kern erwartet `source_uri`, `source_hash` und ACL-Zuordnung (aus dem Quellsystem übernommen).

### 11.2 Entitäten, Fakten, Entity-Resolution

- **Extraktions-Jobs** (LLM via Port, Work Queue 11.6): Entitäten, Beziehungen, Fakten mit `confidence`, `prov.origin = LlmExtraction{model, prompt_hash}`, Standard-Trust `AgentInferred`.
- **Entity-Resolution:** kanonische Entitäts-Records mit **Alias-Tabelle** (Namen, Schreibweisen, Abkürzungen); Auflösung per Blocking (Normalform, Trigramm, Embedding) + Scoring; unsichere Zuordnungen erzeugen `SameAs?`-Kandidaten im Review statt stiller Verschmelzung.
- **Claim-Binding:** Jede Aussage trägt mindestens eine Chunk-Referenz (G1/G5); ohne Beleg landet sie in der **Quarantäne-Zone** (`trust = Hypothesis`): abfragbar, aber nie Teil des Default-Grounding-Kontexts.
- **Schema-Constraints** (optional je Namespace): erlaubte Kantentypen, Kardinalitäten, funktionale Prädikate, Typen der Endpunkte.

### 11.3 Konsolidierung und Zusammenfassungen (`contextra-cognition`)

Alle Verfahren sind **Jobs über Ports**, keine Engine-interne Modellaufrufe:

| Job | Wirkung | Ergebnis |
|---|---|---|
| `Summarize{L1}` | Abstraktive Zusammenfassung je Chunk/Abschnitt | Derivat (`deriv`), `level`-Record bei Bedarf |
| `Consolidate` | Episoden (Turns, `Episodic`) → semantische Fakten (`Semantic`), Duplikate ab Cosinus 0,95 | neue Records + `DerivedFrom`-Kanten; Quellen bleiben (nur `ARCHIVED`-Markierung nach Policy) |
| `CommunitySummary` | Leiden-Communities → Themen-Zusammenfassung (GraphRAG-global) | `Summary`-Records `level = 3` |
| `Extract` | Entitäten/Fakten (11.2) | Kandidaten-Records im Branch → Gates |
| `Judge` | Widerspruchs-/Entailment-Prüfung (NLI/LLM) | `Conflict`-Records |
| `Reembed` | Space-Migration | Embeddings je Space |

Jeder Job-Output ist ein **normaler Commit** mit `derived_by = (modell, prompt_hash)` und läuft über Branch/Gates, wenn er Aussagen verändert. Lineage: `DerivedFrom`-Kanten bilden einen DAG für **inkrementelle Neuberechnung** (L0 → L1 → Entitätsprofil → Themenzusammenfassung); Änderung der Quelle (`content_hash`) markiert Derivate `STALE`, die Work Queue plant sie neu (INV-8).

### 11.4 Temporales Wissen, Decay, Archivierung [löst Befund F7]

- **Bitemporal** (7.9). Neue Aussage für ein **funktionales Prädikat** mit abweichendem Objekt und höherem Vertrauen oder neuerem `valid_from` **schließt** den alten Fakt bitemporal (`valid.until = new.valid_from`), legt `Supersedes` an und löscht nichts. Gleichrangige Widersprüche → `Conflict`-Record.
- **Decay** wirkt **nur auf das Ranking** (Prior, 9.3): `fresh(r, now) = 2^(−max(0, now − t_ref(r)) / T½(class))` mit `t_ref = max(valid.from, last_verified)`. Halbwertszeiten je `DecayProfile` (z. B. `volatil` 30 d, `normal` 180 d, `langsam` 730 d, `keine`). **`now` kommt aus dem Abfrage-Kontext** (reproduzierbar), nicht aus der Systemuhr im Index.
- **Archivierung statt Vergessen:** Records mit niedriger Salience (Zugriffe, Expand-Rate, Graph-Zentralität, Wichtigkeit) werden als **`ARCHIVED`** markiert (Commit): sie sind aus Default-Abfragen ausgeblendet, per `include_archived` auffindbar, mit `PINNED` geschützt. **Zoom-Down** (optional, Policy): L0-Inhalt in Cold-Tier-Segmente verlagern, L1/L2 bleiben hot. Endgültiges Entfernen geschieht nur über Tombstone + Retention (13.5), nie durch Decay.
- **Working-Memory-TTL:** Records der Klasse `Working` können ein `expires_at` (Zeit, nicht `Seq`) tragen; Ablauf = Tombstone-Commit durch Maintenance-Job (sichtbar, versioniert).

### 11.5 Konflikterkennung

Mehrstufig und **konfliktsichtbar beim Lesen**:

1. **Schreibzeit, deterministisch:** gleiche `(subject, predicate)` bei **funktionalen** Prädikaten mit abweichendem Objekt und überlappender Gültigkeit; Kardinalitätsverletzung; Schlüsselkollision. Kosten: ein Zeiger-/CSR-Lookup.
2. **Schreibzeit, heuristisch:** Embedding-Nachbarn (Cosinus > τ) mit gegensätzlicher Polarität (Zahlen, Negation, Datum) → Kandidaten.
3. **Asynchron, semantisch:** `Judge`-Job (NLI/LLM) über Kandidaten.
4. **Ergebnis:** `Conflict{a, b, kind, status ∈ Open|Resolved|Dismissed, evidence}`. Beim Lesen: Evidence Pack trägt Warnung `Conflict` mit beiden Seiten, Quellen und Vertrauen („zwei widersprüchliche Aussagen, 0,81 vs. 0,73"). Auflösung = `Supersedes`/Valid-Interval-Schließung oder Dismiss; Commit mit Begründung.

### 11.6 Work Queue

Dauerhafte Jobtabelle (Truth-Tabelle `job`): `Embed`, `Reembed`, `BuildIndex`, `Summarize`, `Extract`, `Consolidate`, `Judge`, `Reconcile`, `Compact`, `Scrub`. **Leasing, At-least-once**, idempotente Ergebnisse (Idempotenzschlüssel = Job-ID + Versuch). Externe Worker (Agenten) können Aufgaben über die API leasen und per Commit erledigen. Prioritäten: Index-Builds und `Embed` > `Judge` > `Summarize` > Rest; Budget je Zyklus (z. B. `max_llm_calls_per_cycle`).

---

## 12. Agenten-Runtime und Schnittstellen

### 12.1 Sitzungen (`contextra-agent`)

- **Session** = Branch `sess/<agent>/<id>` mit Policy (`ttl`, `quota_rows`, `quota_bytes`, `max_write_rate`, `durability = Relaxed`, `min_main_trust`).
- **Savepoint je Tool-Call:** jeder Tool-Aufruf wird in der Tabelle `alog` `(session, step, tool, input_hash, output_hash, seq)` protokolliert; **Tool Call Replay**: nicht-deterministische Tool-Ausgaben sind über Hashes nachvollziehbar.
- **Agentenzustand** (Plan, Working Memory, Zwischenergebnisse) liegt als Records in `agent/state` und ist damit **versioniert und forkbar**. Ein Agenten-Fork ist ein Branch-Fork (O(1)); Hardlink-Klonen und ein getrenntes Checkpoint-System entfallen.
- **Ablauf:** `begin → [recall/act/remember]* → propose → Gates (7.6) → merge | review | discard`. Fehler, Timeout oder Quotenüberschreitung → automatisches `discard` (Reflog behält).
- **Reviewable Merge:** Standardmäßig wird ein erfolgreicher Session-Branch als `mr/<n>` **vorgeschlagen**; Auto-Merge ist je Namespace und Vertrauensklasse konfigurierbar.
- **Budget-Reservierung (RAII):** Token-/Kostenbudgets einer Sitzung werden reserviert und bei Fehler erstattet.
- **Event-Quelle:** Commit-Observer (WAL-Tail) für Agenten, die auf Änderungen reagieren; begrenzter, nicht blockierender Dispatcher mit Latenzgrenze und Drop-Zählern.

### 12.2 Schnittstellen

| Schnittstelle | Zweck | Format |
|---|---|---|
| Rust-API (eingebettet) | typisierte Builder, `Txn`, `Snapshot`, Async-Fassade | Crate `contextra` |
| gRPC + Arrow Flight | Server-Betrieb; Arrow Flight für Bulk-Ingest/-Export und Vektorbatches | `prost`, Arrow |
| HTTP/JSON | einfache Integration, Health, Metrics | JSON-Plan (identisch zu gRPC) |
| **MCP** (stdio und streamable HTTP) | Agenten-Clients (Claude Desktop, Cursor, VS Code, eigene Agenten) | JSON-RPC 2.0, Nachricht ≤ 4 MiB |
| CLI (`contextra`) | Betrieb: `init`, `ingest`, `query`, `branch`, `merge`, `scrub`, `verify`, `backup`, `stats`, `upgrade` | — |

### 12.3 MCP-Werkzeugkatalog (Soll)

Intent-basiert, wenige Werkzeuge, token-sparsam; Schreibwerkzeuge **standardmäßig gesperrt** (`allow_write`), jede Sitzung schreibt in ihren Session-Branch.

| Werkzeug | Klasse | Zweck |
|---|---|---|
| `ctx_recall` | Lesen | hybride Suche + Packer (Handles, Warnungen, Sufficiency) |
| `ctx_expand` | Lesen | Handle auf Stufe `L0/L1/L2` nachladen |
| `ctx_neighbors` | Lesen | Graph-Nachbarschaft eines Handles |
| `ctx_history` / `ctx_diff` | Lesen | Versionsverlauf, Änderungszusammenfassung |
| `ctx_explain` | Lesen | Plan und Scores einer früheren Abfrage |
| `ctx_describe` | Lesen | Selbstbeschreibung: Namespaces, Schemata, Limits, Beispiele |
| `ctx_remember` | Schreiben | Record/Fakt mit Provenienz in den Session-Branch (idempotent) |
| `ctx_relate` | Schreiben | Kante/Fakt anlegen (binär und n-är) |
| `ctx_retire` | Schreiben | Record ausblenden (Tombstone; Begründung Pflicht) |
| `ctx_session` | Steuerung | `begin`, `savepoint`, `rollback`, `propose`, `discard` |

Die Klassifikation **Lesen/Schreiben/Steuerung ist deklarativ am Werkzeug** (nicht aus dem Methodennamen abgeleitet); ein Test erzwingt, dass jedes registrierte Werkzeug klassifiziert ist; **unbekannte Methoden führen zu Fehler**, nie zu einer Standardklasse (Fail-closed, behebt Befunde B-01/B-02).

### 12.4 LLM-lesbare Fehler

Jeder Fehler trägt `{code, message, retryable, remediation, did_you_mean?, context}`. Beispiele: `E_HANDLE_UNKNOWN` (mit Vorschlägen), `E_CONFLICT` (mit Konflikt-ID und empfohlenem Befehl), `E_BACKPRESSURE` (`retry_after_ms`), `E_BUDGET_TOO_SMALL` (Mindestbudget), `E_SNAPSHOT_EXPIRED`, `E_BEYOND_RETENTION`, `E_FORBIDDEN_NAMESPACE` (ohne Existenz zu verraten). Fehlertexte enthalten nie Inhalte unberechtigter Records (INV-4).

---

## 13. Sicherheit und Betrieb ohne Löschkette

### 13.1 Bedrohungsmodell (Auszug)

| Bedrohung | Maßnahme |
|---|---|
| Unberechtigter Datenzugriff über Suche (Treffer, Scores, Zähler, Fehlertexte) | ACL-Bitmap im Operator (INV-4), defensive Validierung (9.5), neutrale Fehlertexte |
| **Prompt Injection / Memory Poisoning** über Dokumente, Web, Mails, Tool-Ausgaben | Taint-/Trust-Labels, Injection-Scan beim Ingest, Datenrahmen im Pack, Tier-Cap und Gates (G6), Quarantäne |
| Fehlerhafte/halluzinierende Agenten | Session-Branches, Gates, Zitierpflicht, Idempotenz, Quoten, Review-Merge |
| Datenabfluss an Cloud-LLMs | Egress-Guard (13.4) |
| Diebstahl von Datenträgern/Backups | AEAD at Rest (6.10) |
| Korruption (Bit-Rot, Absturz) | CRC je Block, `scrub`, Manifest-Verkettung, Prefix-Konsistenz-Tests |
| Ressourcenerschöpfung (Agent-Schleifen, große Uploads) | Limits, Quoten, Backpressure, Deadline-Propagation |

### 13.2 Zugriffskontrolle

- **Prinzipale und Gruppen** im Katalog (aus dem Identitätssystem des Unternehmens importiert: OIDC-Claims → Gruppen). Jeder Record trägt `acl: AclRef` (Menge erlaubter Gruppen/Prinzipale oder Verweis auf Namespace-Default).
- **Auswertung als Bitmap:** je Gruppe eine Roaring64 über `Gid` (`bitmaps`-Index + Memtable-Anteil); der Filter eines Prinzipals = Vereinigung seiner Gruppen-Bitmaps ∩ Namespace-Freigaben. Änderungen an Gruppen-Zugehörigkeit sind Katalog-Commits (wirken sofort für neue Snapshots).
- **Capabilities** für Agenten: attenuierbare Token (Caveats: Namespaces, Branch-Muster, Operationen, Trust-Obergrenze, TTL, Token-Budget je Zeitraum); ein Hauptagent kann Subagenten eingeschränkte Token ausstellen, ohne den Server zu fragen. Fail-closed: fehlende/ungültige Capability → keine Daten.
- **Schreibrechte** sind getrennt von Leserechten; Hochstufung von `trust` ist ein privilegiertes Recht.

### 13.3 Trust und Taint

```
Trust:  Verified  >  Curated  >  ToolVerified  >  AgentInferred  >  Untrusted/Hypothesis
Taint:  EXTERNAL | INJECTION_SUSPECT | UNVERIFIED_TOOL | PII?   (Flags, kombinierbar)
```

- **Vererbung:** Abgeleitete Records erhalten das **Minimum** der Trust-Stufen und die **Vereinigung** der Taint-Flags ihrer Quellen (eine Zusammenfassung einer Webseite bleibt `Untrusted`).
- **Hochstufung** nur durch ein Gate oder einen privilegierten Prinzipal (Commit-Eintrag).
- **Injection-Scan** beim Ingest: Phrasenfilter (EN/DE), Struktur-Heuristiken (versteckter Text, Unicode-Steuerzeichen, Base64 bis Tiefe 2, NFC), Policy `Quarantine | FlagOnly | Escalate`. Ehrliche Einordnung: Kein Schutz gegen neuartige Formulierungen; der Storage liefert **verlässliche Herkunftsmetadaten**, auf denen die Agentenschicht aufsetzt.
- **PII-Tagging** (`PII?`-Flag, regelbasiert, optional) dient **Ausgabe-Policies** (Records mit PII aus Recall/Export ausschließen), nicht der Löschung.

### 13.4 Egress-Guard (Cloud-LLMs)

Vor jeder Übermittlung von Pack-Inhalten an ein **Cloud-Ziel** (Adapter mit `transport = Cloud`) prüft der Guard: Policy (`LocalOnly | CloudAllowed{namespaces, trust_min}`), Taint/Trust-Regeln, Muster für Geheimnisse (Schlüssel, Token, IBAN, Kennungen), Größen-/Raten-Limits (Bulk-Exfiltration je Session), Timeout-Fail-Closed (Default 100 ms Klassifikation). Typestate `GuardedPayload<Unsanitized → Sanitized>` verhindert das Senden ungeprüfter Nutzlasten. Lokale Adapter (Candle, Ollama, ONNX) unterliegen dem Guard nicht.

### 13.5 Löschung, Retention, Purge [Entscheidung D-06]

Dies ersetzt die gesamte Löschkette.

- **Retire (Standard).** `retire(gid)` ist ein **Tombstone-Commit**: sofort aus allen Sichten ab diesem Snapshot unsichtbar, in Zeitreise innerhalb der Retention weiter sichtbar. Dasselbe gilt für Kanten, Facts, Dokument-Revisionen (Abschnitt 11.1) und TTL-Ablauf.
- **Retention.** Je Namespace `history_days` (Default 30) und `branches_days` (Default 1); optional `history_versions`. Nach Ablauf entfernt die Compaction tote Zeilen physisch (6.9). **Es gibt keinen Nachweis**, keine Signatur und keinen Bericht über die Entfernung.
- **Purge (Administration, best effort).** `purge(filter)` = Tombstone aller passenden Records + **Retention-Override** (Horizont = jetzt, soweit keine Pins/Tags/Branches blockieren) + Compaction-Auftrag für betroffene Segmente. Zweck: Speicher freigeben und Daten aus der Historie nehmen, **ohne Garantie** und ohne Beleg; blockierende Pins/Tags werden gemeldet.
- **Konsequenzen für Derivate:** Embeddings, Zusammenfassungen und Token-Zählungen sind an `ContentHash` gebunden; sie werden vom Retention-GC entfernt, wenn **kein lebender oder retention-pflichtiger Record** den Hash mehr referenziert (Referenzzählung beim Compaction-Lauf).
- **Ausdrücklich nicht Teil des Produkts:** Löschnachweise, Ed25519-/HMAC-Beweise, Layer-Abdeckungsberichte, Crypto-Shredding, Revocation-Logs, Graph-Reparatur-Attestierungen, Lösch-Modi im KV, Ausschluss-Scopes („`ExcludedScope`").

### 13.6 Audit und Betriebsprotokolle (schlank)

Statt einer Audit-Chain: strukturierte **Betriebs- und Sicherheitsprotokolle** (`tracing`-Events, JSON-Lines) für Anmeldung/Capabilities, Schreibzugriffe (Prinzipal, Branch, Commit-Seq), Merge-Entscheidungen, Gate-Ergebnisse, Trust-Hochstufungen, Egress-Entscheidungen. Sie sind **nicht kryptografisch gesichert**; Integrität obliegt dem Log-Backend des Betreibers. Lesezugriffe werden **gesampelt** (konfigurierbar) im Access-Trace protokolliert, getrennt vom Inhaltszustand.

### 13.7 Beobachtbarkeit

`tracing` + `metrics` (Prometheus/OpenTelemetry-Exporter über `MetricsSink`-Port). Pflichtmetriken: WAL-Gruppengröße/-Latenz, Commit-Latenz je Klasse, Memtable-Größe, Segmentzahl/-Größe je Branch, Compaction-Rückstand, Write-Amplification, Block-Cache-Trefferquote je Klasse, Index-Watermark-Rückstand je Modalität, `lane_coverage`, Query-Latenz je Operator, Rerank-Skips, Backpressure-Stufe, Embedding-Queue, Job-Rückstand. `stats`-Befehl und `explain` sind Teil der API.

---

## 14. Ressourcenmodell und Leistungsarchitektur

### 14.1 Speicherbudgets (INV-11)

Ein globales **`MemoryBudget`** wird beim Start in Klassen aufgeteilt (Prozente des Gesamtbudgets, Default je Profil):

| Klasse | Inhalt | Default-Anteil |
|---|---|---|
| `fixed` | Manifeste, FST-Wörterbücher, Skip-Daten, ANN-Codes, CSR-Offsets, Bitmaps, Katalog | 25 % |
| `memtable` | Memtables aller Branches (inkl. Mini-HNSW) | 15 % |
| `block_cache` | Truth-/Index-Blöcke, getrennte Teilbudgets je Klasse: Postings · Nachbar-Sektoren · Vektorblöcke · Text · Attribute | 40 % |
| `query` | Arbeitsspeicher aktiver Abfragen, Pack, Rerank | 10 % |
| `jobs` | Compaction, Index-Builds, Ingest-Puffer | 10 % |

Der Block-Cache ist ein reiner Lese-Cache über **unveränderliche** Daten (W-TinyLFU oder CLOCK-Pro), seitenausgerichtete Rahmen (4/16 KiB), Pin-Zähler je Rahmen; **getrennte Budgets pro Klasse**, damit ein großer Scan heiße Strukturen nicht verdrängt. Mmap ist ein **optionaler** Lesepfad (Entwicklung/kleine Stores), kein Standard (unkontrollierte Eviction, SIGBUS bei I/O-Fehlern).

### 14.2 Backpressure und Druckstufen

`PressureLevel ∈ {Normal, Elevated, Critical}`, berechnet aus: Memtable-Füllstand, WAL-Warteschlange, Compaction-Rückstand (Segmentzahl über Budget), Embedding-Queue, Block-Cache-Verdrängungsrate.

| Stufe | Wirkung |
|---|---|
| `Elevated` | Ingest-Drosselung (Verzögerung je Batch), früherer Seal, Index-Builds niedriger Priorität pausieren |
| `Critical` | Neue Schreibtransaktionen warten oder erhalten `E_BACKPRESSURE`; nur Flush/Compaction/Lesen laufen; Abfrage-Concurrency reduziert |

Backpressure ist **sichtbar** (Metrik, Fehlercode mit `retry_after_ms`), nie ein stilles Verwerfen.

### 14.3 Limits (konfigurierbar, mit Fehlercodes)

| Limit | Default |
|---|---|
| Chunk-Text | ≤ 64 KiB (Ingest teilt größere Quellen) |
| Props je Record | ≤ 128 Felder, ≤ 32 KiB |
| Ops je Transaktion | ≤ 100 000; gestagte Bytes ≤ 64 MiB |
| Kanten je Knoten (aktiv) | Warnung ab 10⁴, hartes Limit 10⁶ (Hub-Schutz) |
| Aktive Snapshots | ≤ 4 096; Lease 300 s |
| Branches je Store | ≤ 100 000 (Sessions kurzlebig) |
| Query-`k` | ≤ 1 000 |
| MCP-Nachricht | ≤ 4 MiB |
| Namespaces | ≤ 4 096 |

### 14.4 Threading- und I/O-Modell

| Komponente | Ausführung |
|---|---|
| Netzwerk, Jobs, Timer | `tokio` (Multi-Thread) ab R3/R5 |
| Storage-Kernel (WAL-Writer, Sealer, Compactor) | eigene Threads mit synchroner API und Completion-Handles; kein `tokio` im Kern |
| Datei-I/O | Linux: `io_uring` (registrierte Puffer, `O_DIRECT` für Vektor-/Sektor-Reads, optional SQPOLL); andere Plattformen: Blocking-Pool mit `pread`/Overlapped |
| CPU-lastig (Distanzen, BM25, Index-Builds, Hashing, Dekompression) | eigener Compute-Pool (rayon-artig), strikt getrennt vom Async-Executor |
| Backpressure | begrenzte Kanäle + Semaphoren für I/O-Slots und Speicher |

**Lock-freie Bausteine:** `ArcSwap<Manifest>` je Branch (wait-free Reads), `crossbeam-skiplist` + Arena für Memtable, Epochen-Reklamation (`crossbeam-epoch`) für Speicherfreigabe, geshardeter Block-Cache mit atomaren Pin-Zählern, MPSC → Single-Writer je Commit-Gruppe. **Es gibt keine globale Sperre im Lese-Hotpath.** Lock-Reihenfolgen sind dokumentiert und werden mit `loom` getestet.

### 14.5 Orientierungsrechnung (10 Mio. Chunks, 1024 d) **[Ziel]**

| Tier | Inhalt | Größenordnung |
|---|---|---|
| RAM fest | Manifeste, FST, Skip-Daten, ANN-Codes (≈ 128 B/Vektor), CSR-Offsets, Bitmaps | 3–6 GB |
| RAM Block-Cache | Postings, Nachbar-Sektoren, Text-/Attributblöcke | 8–32 GB (konfigurierbar) |
| NVMe | Vektorblöcke F16 (~20 GB), Graph (~3 GB), Postings (5–10 GB), Text komprimiert (~10 GB) | 40–60 GB |

Alle Werte sind **Arbeitsannahmen**, die in Phase 0 gemessen werden.

### 14.6 Plattformen

Linux x86-64/aarch64 (Primärziel, `io_uring`), Windows Server und macOS (portabler I/O-Backend; `DurableFs`-Testsuite beweist Persistenzsemantik), Single-Binary-Daemon. Container-tauglich (Verzeichnis-Volume). Keine Abhängigkeit von nativen Bibliotheken im Kern (ONNX-Runtime nur im optionalen Adapter).

---

## 15. Rust-Engineering-Regeln

### 15.1 Typsystem als Sicherheitsnetz

- Newtypes/Phantom-Typen für `Gid`, `RecordId`, `Seq`, `Lsn`, `SegmentId`, `NsId`, `BranchId`, `ContentHash<T>`; keine nackten `u64`/`String` in öffentlichen Signaturen des Kerns.
- **Typestate:** `Txn<ReadOnly|ReadWrite>`, `Snapshot<'db>`, `GuardedPayload<Unsanitized|Sanitized>`, `Segment<Building|Sealed>`.
- `#[non_exhaustive]` auf allen öffentlichen Enums, die wachsen dürfen (`Kind`, `ErrorCode`, `Origin`).
- `#![forbid(unsafe_code)]` in: `types`, `ports`, `mvcc`, `vector`(außer SIMD-Aufrufe über `contextra-simd`), `text`, `graph`, `rank`, `pack`, `engine`, `ingest`, `agent`, `cognition`, `security`, `mcp`, `server`. `unsafe` nur in `sys`, `simd`, `codec` (Archive-Zugriff), `store` (Mmap-Lesepfad, O_DIRECT-Puffer). Jeder `unsafe`-Block trägt `// SAFETY:` mit Invariante; Baseline-Datei zählt Blöcke (nur abnehmend).

### 15.2 Determinismus und Ports

- `Clock`, `Rng`, `Fs`, `Net` (Embedder/Llm-Transport) sind **Ports**. Direkter Zugriff auf `SystemTime::now()`, `Instant::now()` (außer in der Port-Implementierung), `thread_rng()`, `std::fs` im Kernel ist verboten (Lint-Regel in `xtask`).
- Ausnahme: kryptografisches Schlüssel-/Nonce-Material über `getrandom` im `security`-Crate.
- Jede Abfrage und jede Maintenance-Entscheidung erhält `now: Hlc` und `seed` explizit.

### 15.3 Fehlerbehandlung

- Keine `unwrap()`/`expect()`/`panic!` in Produktionspfaden des Kerns (Clippy-Deny); Ausnahmen mit Begründungskommentar und Test. Panic-Grenze an FFI-/Thread-Grenzen (`catch_unwind` → Fehler).
- Fehlertyp je Crate (`thiserror`), Abbildung auf `ErrorCode` mit `remediation`/`retryable` an der Engine-Grenze. Kein Fehler wird verschluckt (`#[must_use]` auf Ergebnissen kritischer I/O-Operationen; Lint für verworfene `Result` in I/O-Pfaden).
- I/O-Fehler in WAL/Manifest sind **fatal für den Store** (Store wechselt in `ReadOnly`/`Failed`, kein Weiterschreiben auf unsicherem Stand, „fsyncgate"-Regel: nach fehlgeschlagenem `fsync` wird die Datei nicht wiederholt gesynct, sondern der Store schließt und recovered).

### 15.4 Nebenläufigkeit

- Lock-Hierarchie dokumentiert (Manifest-Publikation → Branch-Memtable → Block-Cache-Shard); **keine verschachtelten Locks im Lese-Hotpath**.
- `loom`-Tests für: Manifest-Publikation + Epochen-Reklamation, Memtable-Vektor-Arena (Release/Acquire), Block-Cache-Pin/Evict, Group-Commit Leader/Follower, Snapshot-Lease.
- Keine Blockierung des Async-Executors: CPU- und blockierende I/O-Arbeit nur über die dafür vorgesehenen Pools (`xtask`-Lint auf `block_on`/synchrones I/O in `async`-Funktionen).

### 15.5 Abhängigkeitsrichtlinie

- Neue Abhängigkeit erfordert Eintrag im Entscheidungsregister mit Begründung, Lizenz- und Supply-Chain-Prüfung (`cargo-deny`, `cargo-vet`), reproduzierbarem Build und `--locked`.
- Bevorzugt: `rkyv`, `blake3`, `crc32fast`, `crossbeam-*`, `arc-swap`, `parking_lot`, `roaring`, `fst`, `zstd`, `tokio` (R3+), `tonic`/`prost`, `arrow`/`arrow-flight`, `tracing`, `metrics`, `thiserror`, `rayon` (nur Compute-Pool), `io-uring`. Eigenentwicklung bei **Vektorindex, Postings, Segmentformat, WAL**, weil Kontrolle über Layout und Zero-Copy zentral ist.
- `usearch`/FFI-Indizes sind **keine** Standardabhängigkeit (Native-Library-Risiko, nicht Snapshot-/Liveness-kompatibel).

### 15.5.1 Serialisierung

Intern (On-Disk) **`rkyv`-Archive** mit versionierten Schemata plus eigene Blockformate; extern (API) **`prost`** (gRPC) und JSON (HTTP/MCP). FlatBuffers entfällt (zwei Serialisierungsfamilien intern sind unnötig).

### 15.6 Feature-Flags

Flags schalten **Adapter und Optionales**, nie Kernsemantik: `onnx`, `candle`, `ollama`, `kv-prefix`, `server`, `encryption` (kompiliert AEAD ein; Laufzeitaktivierung per Store-Konfiguration), `simd-avx512`, `io-uring`, `fault-injection` (nur Tests). **Kein** Feature verändert Durability, Isolation oder Sichtbarkeit.

### 15.7 Governance-Werkzeuge

`xtask` wird auf **Pflichtprüfungen** reduziert: Layering, Unsafe-Baseline, Determinismus-Lint, Panic-Inventar, Format-Magic-Katalog, Duplikat-Symbole, Lizenz/Supply-Chain, Benchmark-Trend, Fuzz-Smoke. Kampagnen-/Harness-Artefakte (`.jules`, `campaign_*`-Tests mit Hypothesen-Dateien, Veto-Deadlines) werden nach Migration von Inhalten in reguläre Tests überführt oder archiviert; sie sind kein Bestandteil des Produkts. Archive (`Archiv.zip`, 5,8 MB) gehören nicht in das Repository.

---

## 16. Migration vom Ist-Stand

### 16.1 Disposition der Crates

Grundlage: Rolle, Größe (Zeilen in `src/`, gezählt) und Befund aus 2.2. **Pro-Datei-Entscheidungen werden in Phase 1 mit dem Compiler und den Tests als Netz getroffen**; diese Tabelle legt die Richtung fest.

| Ist-Crate (Zeilen) | Disposition | Begründung |
|---|---|---|
| `contextra-store` (27 k) | **Neuschnitt** → `log` + `store`; WAL-Teile (Group Commit, Torn-Write-Erkennung, Recovery) als Vorlage, SSTable-/Compaction-Ideen portieren | Löschkette, String-Schlüsselraum, KV-Modell passen nicht zum Segment-/Gid-Modell |
| `contextra-engine` (21 k) + `contextra-db` (2 k) | **Neuschnitt** → `engine`; `db`, `core` ersatzlos | 4-Index-2PC entfällt; Planner/Executor/Katalog neu; Chunker, Extraktion, Filter, Temporal-Filter als Quellen übernehmen |
| `contextra-mvcc` (3,6 k) | **Umbau** | Sequencer, Snapshots, Epochen behalten; Branch-Lineage, Idempotenz, Typestate-Transaktion neu; SSI-Read-Set mit Prädikaten |
| `contextra-vector` (13,8 k) | **Bereinigen/Umbauen** | HNSW, SQ8, ACORN, SIMD-Dispatch, Vamana (aus `experimental-diskann` promovieren) behalten; mutierbares Soft-Delete/Reparatur/Rebuild-Guard/`partial-index-rebuild` entfernen; Immutable-Segment-API + Liveness-Maske |
| `contextra-text` (6,6 k) | **Bereinigen** | BM25F zum Default, Segmentformat mit FST/Block-Max, globale Statistiken, Analyzer-ID im Schlüssel; Domänen-Stubs füllen oder entfernen |
| `contextra-graph` (20,5 k) | **Stark reduzieren** (Ziel ≈ 6–8 k) | Behalten: CSR, BFS, Forward-Push-PPR, Leiden, Konsistenz/Widerspruch, Entity-Extraktions-Schnittstelle. Entfernen/archivieren: `path_rag`, `tl_hfd`, `apprh`, k-Path, Dense-/Shadow-PPR-Varianten, `percolation`, `edge_reinforcement*`, `hyperedge*` (Speicher/Kaskade), `session_dag` (ersetzt durch Branches), `cascade` (Löschkaskade) |
| `contextra-rank` (4,1 k) | **Bereinigen** | RRF, Norm-Fusion, isotonische Kalibrierung, MMR; DiBud/Resonance/Global-Fusion nur bei Nachweis |
| `contextra-cognition` (6,3 k) | **Bereinigen** | Konsolidierung, Aggregation/Community-Summaries, Synthese als Jobs; Löschen-nach-Decay entfernen |
| `contextra-ingest` (21 k) | **Behalten + erweitern** | Extraktoren (PDF/Office/HTML/E-Mail/CSV), strukturbewusstes Chunking, Revisions-Diff, Injection-Scan-Anbindung |
| `contextra-agent` (5,1 k) + `contextra-checkpoint` (4,2 k) | **Zusammenführen** | Sessions/Gates/Action-Log/Idempotenz; Checkpoint/Hardlink/Orphan entfallen zugunsten Branches |
| `contextra-crypto` (8,6 k) | **Radikal reduzieren** → `security` (AEAD, KDF, Key-Provider) | `deletion_proof*`, `ed25519_proof`, `kv_shredding`, `kv_cipher`, `kv_segment`, `revocation_log`, `audit_chain`, `wal_completeness`, `anti_tamper` entfernen |
| `contextra-privacy` (5,2 k) | **Bereinigen** → in `security` | Egress-Guard, Bulk-Exfiltrations-Detektor, Guarded-Payload, Entity-Recognizer (PII-Flag) behalten; `avv_generator`, `processing_registry`, `audit_trace`, `context_edit_audit`, `egress_vault` prüfen/entfernen |
| `contextra-kvcache` (4,7 k) | **Optional behalten** | Radix-Prefix-Store, KIVI, Eviction; **`segment.rs` Shredding-Anteile und Tenant-Krypto entfernen**; als Feature `kv-prefix` |
| `contextra-infer-candle/-ollama/-onnx` (5,1 k/5,3 k/2,0 k) | **Behalten** | Adapter für `Embedder`/`Reranker`/`Llm`-Ports |
| `contextra-mcp` (7,4 k) | **Umbauen** | neuer Werkzeugkatalog (12.3), deklarative Klassifikation; `proof_key*`, `forget`/Löschwerkzeuge ersetzen durch `ctx_retire` |
| `contextra-adapt` (5,0 k), `contextra-router` (6,9 k) | **Aus dem Kern entfernen** (`contrib/research/`, nicht im Workspace) | Bandit/PID/Lyapunov/Shadow-Gates ohne Bedarfsnachweis. Behalten: einfache Latenz-adaptive Pool-Regel (in `engine`), Kaskadenrouting nur falls Produktbedarf belegt (Reaktivierungskriterium D-13) |
| `contextra-audit-export`, `contextra-avv-generator` | **Entfernen** | Compliance-Artefakte (Nicht-Ziel) |
| `contextra-license` (0,5 k) | **Aus dem Kern lösen** | Verpackungsthema (16.3) |
| `contextra-sandbox` (2,1 k), `contextra-py` (2,4 k) | **Auslagern** (eigenes Repo/optional) | WASM- und Python-FFI nicht Kernprodukt |
| `contextra-wire` (2,6 k) | **Ersetzen** durch `codec` | siehe 15.5.1 |
| `contextra-simd`, `contextra-sys`, `contextra-durable-fs`, `contextra-ports`, `contextra-types`, `contextra-testkit` | **Behalten/bereinigen** (`sys` + `durable-fs` zusammenführen) | Fundament |

### 16.2 Entfernungsinventar der Löschkette (belegt per Dateisuche)

Suchbegriffe: `deletion_proof`, `DeletionProof`, `crypto_shred`, `kv_shred`, `shred`, `revocation`, `audit_chain`, `GraphRepairAttestation`, `LayerCleanupProof`, `ExcludedScope`, `KvDeleteMode`, `DeletionLayer`, Feature `deletion-proof`. Fundstellen (Zeilen mit Treffern; Auswahl der größten):

| Crate | Quelldateien (Treffer) | Test-/Fuzz-/Bench-Dateien |
|---|---|---|
| `contextra-crypto` | `deletion_proof.rs` (192), `deletion_proof_typestate.rs` (85), `kv_shredding.rs` (50), `revocation_log.rs` (49), `ed25519_proof.rs` (29), `kv_cipher.rs` (25), `crypto.rs` (13), `lib.rs` (12); außerdem `audit_chain.rs`, `kv_segment/`, `wal_completeness.rs`, `anti_tamper.rs` | über 25 Testdateien (u. a. `guarantee_deletion_irrecoverable.rs`, `deletion_proof_v3_validation.rs`, `revocation_*`, `audit_chain_tests.rs`, `campaign_j26_hypotheses.rs`), 2 Fuzz-Targets, 1 Bench |
| `contextra-store` | `kv/segment.rs` (32), `kv/delete_mode.rs` (12), `kv/mod.rs`, `lsm/config.rs`; Feature `deletion-proof`, `legacy-wal-key` | `kv_delete_mode_wiring_test.rs`, `kv_deletion_proof_registry.rs`, `deletion_proof_unregistered.rs`, `campaign_j21_hypotheses.rs` u. a. |
| `contextra-engine` | `lib.rs` (21), `collection/mod.rs`, `contextra_impl/collections.rs`, `contextra_impl/lifecycle.rs` | `no_crypto_proof_stub.rs` u. a. |
| `contextra-kvcache` | `segment.rs` (28), `lib.rs` | `e2e_lifecycle_with_engine.rs` |
| `contextra-mcp` | `proof_key.rs` (26), `proof_key_env.rs`, `server_tools.rs`, `server_dispatch.rs`, `tools_crud.rs`, `sandbox.rs`, `tests.rs` | — |
| `contextra-graph` | `cascade.rs` (Orphan-/Kaskadenlöschung), `lib.rs` | — |
| `contextra-privacy` | `avv_generator.rs`, `egress_gateway.rs`, `lib.rs` | `pii_vault_forces_crypto_shred.rs` (23), `campaign_j27_privacy_test.rs` |
| `contextra-db` | (Crate entfällt) | `deletion_proof_integration.rs` (26), `e2e_compliance_scenario.rs`, Beispiel `verify_deletion_proof_external.rs` |
| `contextra-audit-export`, `contextra-avv-generator` | Crates entfallen (Markdown-Vorlagen, BSI-Mapping) | — |
| `contextra` (Facade) | `performance_profile.rs`, `collection_profile.rs`, `builder.rs` (`DeploymentTier`, `PerformanceProfile`, Fehler `KvDeleteMismatch`, `DurabilityVectorMismatch`) | — |
| `contextra-types` | `error.rs` (Löschfehler-Varianten) | — |
| `contextra-py` | `bindings/db.rs` | — |

**Vorgehen:** (1) Funktionalität der Gegenstücke (Abschnitte 6–13) bauen; (2) Löschkette **vor** dem Neuschnitt per Compiler-gestütztem Entfernen schneiden (Feature-Flags `deletion-proof`/`legacy-wal-key` löschen, Crates streichen, Tests/Fuzz/Bench der Kette löschen); (3) Dokumentationsreste (`SPEC.md` Teile B.6, K.3/K.4/K.8, L.1, Vetos F-02, Dateikopf-Kommentare, `COMMON_LLM_ERRORS.md`) bereinigen; (4) `grep`-Gate in CI: keine Treffer für die Suchbegriffe außerhalb von `docs/history/`. Die vollständige Liste ist durch Compiler und Suche beim Entfernen verbindlich zu erfassen; die Tabelle oben dient als Planungsgrundlage.

### 16.3 Lizenzierung und Produktformen

Die Engine kennt **keine funktional gestaffelten Ringe** und keine Lizenzprüfung im Kern. Falls ein Lizenzmodell benötigt wird, wird es als **äußere Hülle** (Daemon/CLI-Distribution, z. B. signierte Lizenzdatei für Support/Features des Servers) umgesetzt; der Kern bleibt unter MIT/Apache-2.0 (Workspace-Lizenz). Entscheidung D-12.

### 16.4 Phasenplan mit Austrittskriterien

| Phase | Inhalt | Austrittskriterium (messbar) |
|---|---|---|
| **P0 Fundament & Messung** | Repo-Aufräumen (Archive, Schalen), Löschkette schneiden, **Benchmark-Harness** und **deterministischer Simulator** (`sim`: Fs/Clock/Rng/Fault), Datensätze (deutsch/englisch, BEIR/GermanQuAD-artig, interne Unternehmenskorpora) | Workspace baut ohne Löschketten-Treffer; Simulator führt WAL-Crash-Zyklen aus; Baseline-Zahlen für Ist-HNSW/BM25 vorhanden |
| **P1 Storage-Kernel** | `log`, `store`, `mvcc`: WAL + Group Commit, Memtable, Truth-Segmente, Manifest-Log, Liveness, Compaction (nur Truth), Recovery, Blob-Store, Block-Cache, optional AEAD | Crash-Simulation (≥ 10⁶ zufällige Absturzpunkte) ohne Prefix-Verletzung; Commit-p99 < 5 ms; Write-Amplification gemessen und dokumentiert |
| **P2 Index-Segmente** | Flat/HNSW/Vamana-Segmente, FTS-Segmente, CSR-Segmente, Watermarks/Jobs, gemeinsame Bitmaps/Gid-Übersetzung | Recall@10 ≥ 0,95 bei Zielparametern; gefilterte Suche A/B/C mit Fallback; Index-Verlust-Test (Löschen aller `index/`) ⇒ Neustart + Flat-Fallback + Rebuild ohne Datenverlust |
| **P3 Engine & Query** | Katalog, Namespaces, Ingest-Orchestrierung, Revisions-Diff, Planner/Executor, Fusion, Validierung, Packer (Knapsack, MMR, Platzierung, Explain) | End-to-End-Qualität (nDCG@10, Recall) ≥ Ist-System auf denselben Daten; Pack hält Budget in 100 % der Läufe; Hybrid-p99 im Zielkorridor oder dokumentiert |
| **P4 Branches & Agenten** | Overlay-Branches, Merge/Diff, Gates, Idempotenz, Sessions, MCP-Katalog | Property-Tests: Merge kommutiert wo spezifiziert; Branch-Isolation (INV-6); Session-Abort hinterlässt `main` unverändert; MCP-Klassifikationstest grün |
| **P5 Betrieb & Satelliten** | Backup/Restore, `scrub`/`verify`, Metriken, Server (gRPC/HTTP/MCP-HTTP), Inferenz-Adapter, Egress-Guard, optional KV-Prefix-Cache | Restore-Test aus Backup; 72-h-Soak ohne Speicherwachstum jenseits Budget; Egress-Guard Fail-closed-Tests |
| **P6 Härtung** | Fuzzing aller Formate, Miri/Loom-Suiten, Last-/Chaos-Tests, Dokumentation, Formatfreeze 1.0 | Alle Abnahmekriterien (Abschnitt 17.4) erfüllt |

Reihenfolge ist zwingend: **P0 vor P1** (Messung und Simulator zuerst); **P1 vor P2**; GC und Compaction sind Phase-Gates mit Simulation, nicht Nachrüstung.

### 16.5 Daten- und API-Kompatibilität

Keine Migration von Ist-Dateiformaten (v0.1.0, keine Produktivdaten). Ein **Import-Werkzeug** (`export` aus Ist → JSONL/Arrow → `ingest`) KANN bereitgestellt werden, falls Testdaten erhalten bleiben sollen. Python-/MCP-Clients passen sich an den neuen Katalog an; die Fassade `contextra` bietet Migrationshinweise im Changelog.

---

## 17. Teststrategie und Abnahme

### 17.1 Testebenen

| Ebene | Werkzeug | Gegenstand |
|---|---|---|
| Einheit/Eigenschaft | `proptest`, `quickcheck` | Roaring-Übersetzung, Sichtbarkeitsregel, Merge-Tabelle, Knapsack-Optimalität (vs. Brute-Force kleiner Instanzen), RRF, Tokenzählung |
| Nebenläufigkeit | `loom`, `shuttle` | Manifest-Publikation, Arena, Cache, Group Commit, Leases |
| Speichersicherheit | Miri (Teilmengen), `cargo-fuzz` | `unsafe`-Inseln, Parser aller Dateiformate, Extraktoren (PDF/Office/HTML/E-Mail) |
| **Deterministische Simulation** | `contextra-sim` | Zufällige Abfolgen von Commits, Seals, Compactions, Merges, Abstürzen, Fault-Injection (Short-Write, Torn-Write, fsync-Fehler, ENOSPC, Bit-Flip) gegen ein **Referenzmodell** (einfacher In-Memory-Store); Invarianten INV-1…INV-12 als Orakel |
| Differenzielle Tests | Referenzimplementierungen | ANN gegen exakte Suche; BM25 gegen Referenz (z. B. Tantivy-Scores auf identischen Statistiken als Plausibilitätscheck, nicht als Abhängigkeit); PPR gegen Dense-Power-Iteration (nur Test) |
| End-to-End-Qualität | Benchmark-Harness | nDCG@10/Recall@k/MRR je Leg und hybrid; Kontext-Qualität (Antwortgenauigkeit mit festem LLM über Evidence Packs, Token-Verbrauch) |
| Last/Soak/Chaos | Benchmark + Chaos-Skripte | Durchsatz, Latenzverteilung, Speicherstabilität, Absturz unter Last |
| Sicherheit | Gezielte Tests | ACL-Leckage (Scores/Zähler/Fehler), Capability-Attenuierung, Egress-Fail-closed, Injection-Korpus |

### 17.2 Orakel-Auswahl der Simulation (Beispiele)

- Nach beliebigem Absturz: sichtbarer Zustand = Präfix der bestätigten Commits (INV-2).
- Beliebige Indexverluste: Abfrageergebnis ⊇ Ergebnis mit Index in Flat-Qualität (Recall), nie ⊄ Truth (INV-3).
- Branch-Operationen: Sicht von `main` unverändert bis Merge (INV-6); Merge-Ergebnis = Anwendung des Δ auf `main` (Referenzmodell).
- Retention-Compaction: entfernt nie eine Zeile, die ein Pin/Tag/Fork-Punkt noch benötigt (INV-9).

### 17.3 Benchmark-Wahrheit

Es gibt **einen** Benchmark-Satz und eine Zahl pro Messung. Veröffentlichte Zahlen nennen Hardware, Datensatz, Parameter, Konfidenzintervall und ob Embedding-Inferenz enthalten ist. Messungen unterhalb 10⁶ Dokumenten dürfen nicht auf 10⁷–10⁸ extrapoliert werden (Befund B-06): P2/P3 messen mindestens bei 10⁶, 10⁷ und – für `scale` – 10⁸ Chunks (synthetische Vektoren mit realistischer Clusterstruktur plus ein echter Korpus ≥ 10⁶).

### 17.4 Abnahmekriterien für 1.0

1. Alle Invarianten INV-1…INV-12 durch Simulation/Property-Tests abgedeckt; ≥ 10⁷ Simulationsläufe ohne Verletzung in CI-Nightly.
2. Kein Treffer des Löschketten-Greps (16.2) im Repository (außer `docs/history/`).
3. Zielwerte aus 1.5 erreicht oder mit Abweichungsbegründung und Entscheidungsregistereintrag versehen.
4. Alle Dateiformate haben Fuzz-Targets mit ≥ 24 h Laufzeit ohne Befund.
5. Wiederherstellungstest: Löschen aller Index-Segmente ⇒ vollständige Wiederherstellung ohne Datenverlust; Restore aus Backup auf neuer Maschine.
6. ACL-Leckagetest-Suite grün (kein Informationsabfluss über Treffer, Zähler, Scores, Fehlertexte).
7. MCP-Konformität: jedes Werkzeug klassifiziert, Schreibwerkzeuge standardmäßig gesperrt, Fehler LLM-lesbar.
8. Dokumentation: Betriebshandbuch, Formatspezifikation, API-Referenz, Migrationsnotizen.

---

## 18. Blindfleck- und Risikoregister

| ID | Risiko / Blindfleck | Wirkung | Gegenmaßnahme | Frühindikator |
|---|---|---|---|---|
| R-01 | **Write-Amplification** des LSM + Neuaufbau von ANN-Segmenten bei Compaction | Ingest-Durchsatz, SSD-Verschleiß | Hohes Tier-Verhältnis für Bulk, Segmentobergrenze 8 Mio., Index-Streaming-Merge (Vamana), Compaction-Drosselung; Messung in P1/P2 | Write-Amp > 15 im Bulk-Test |
| R-02 | **Segment-Fan-out** (viele Segmente × Branches) erhöht Query-Latenz | p99 | Fan-out-Budget ≤ 24, Compaction-Priorität, geteilte Top-k-Schwelle, Session-Branches in Memtable halten | Segmentzahl je Sicht |
| R-03 | **Gid → ord-Übersetzung** bei großen Filtermengen (ACL, Frontier) | CPU/Latenz | Sortierte Spalte + Range-Slicing, Cache je Query/Segment, ACL-Gruppen-Bitmaps vorberechnet, Obergrenzen für Frontier; Messung P2 | Übersetzungszeit > 20 % der Query |
| R-04 | **Flat-Fallback-Kosten** bei großem Rückstand ungebauter Indizes | Latenz bei Bulk-Ingest | Watermark-Metrik, Index-Build-Priorität, `lane_coverage`-Warnung, Ingest-Drosselung | Rückstand > N Segmente |
| R-05 | **Embedding-Modellwechsel** (Neuberechnung großer Bestände) | Zeit, Kosten, Mischzustand | Space-Konzept, Doppelbetrieb, `lane_coverage`, Priorisierung nach Salience, Kostenabschätzung vorab | Migrationsdauer-Schätzung |
| R-06 | **Mutable Statistik/Decay-Parameter** erschweren Reproduzierbarkeit | Debugging | `now` im Query-Kontext, Sidecar nicht ergebnisrelevant außer als Prior mit gespeichertem Stand im Explain | Nichtreproduzierbare Rankings |
| R-07 | **LLM-Jobs (Extraktion, Judge) sind nicht deterministisch/teuer** | Qualität, Kosten | Produzenten-ID + Prompt-Hash, Gates, Quarantäne, Budget je Zyklus, Review-Merge | Konfliktrate, Gate-Reject-Rate |
| R-08 | **Prompt-Injection über Dokumente** nicht verhinderbar | Sicherheit | Trust/Taint, Datenrahmen, Egress-Guard, Quarantäne; ehrliche Kommunikation der Grenzen | Injection-Korpus-Erkennungsrate |
| R-09 | **ACL-Drift** zwischen Quellsystem und Contextra | Datenleck | ACL aus Quelle bei jedem Ingest/Sync neu übernehmen, ACL-Sync-Job, Alterswarnung, Fail-closed bei fehlender ACL | ACL-Alter je Quelle |
| R-10 | **Windows-Persistenzsemantik** (Verzeichnis-fsync, Rename) | Durability | `DurableFs`-Testsuite pro Plattform, Dokumentation, CI-Matrix | Plattformtest rot |
| R-11 | **Hub-Knoten im Graph** (Entität „Firma", „Kunde") | PPR-Qualität, Latenz | Degree-Cap, Skip-Index, Hub-Dämpfung im PPR, Typ-Masken | Gradhistogramm |
| R-12 | **Branch-Retention blockiert GC** | Speicherwachstum | Branch-TTL und -Quoten, Lease-Ablauf, Warnmetriken | Speicher durch Pins |
| R-13 | **Chunking-Qualität** dominiert RAG-Qualität | Gesamtqualität | Strukturbewusster Chunker, Small-to-Big, Chunker-Versionierung, Evaluation je Dokumenttyp | nDCG je Dokumenttyp |
| R-14 | **Scope-Wucherung** (Rückkehr der Forschungsfeatures) | Wartbarkeit | Entscheidungsregister, Reaktivierungskriterien, Crate-Governance (48-h-Regel bleibt für neue Crates/Abhängigkeiten) | Neue Features ohne Benchmark |
| R-15 | **Ein Store = ein Prozess** (V-13-Prinzip) | Betrieb | Daemon-Modus, MCP/gRPC; klare Dokumentation | — |
| R-16 | **Single-Commit-Sequencer** als Engpass | Durchsatz | Gruppierung, parallele Apply je Branch, Messung; Sharding nach v1 | Commit-Warteschlange |
| R-17 | **Zeit-/Uhrenfehler** (HLC, Sprünge) | `AS OF TIMESTAMP` | Reflog monoton, `Seq` ist maßgeblich, Zeit informativ | Uhrensprung-Warnung |
| R-18 | **Retention ohne Nachweis** kann Erwartungen im Betrieb enttäuschen (Daten „noch da") | Vertrauen | Klare Doku von Retire/Retention/Purge-Semantik, `retention_status`-Auskunft (nur Betriebsinfo, kein Nachweis) | Supportfälle |

---

## 19. Entscheidungsregister

| ID | Entscheidung | Verworfene Alternativen / Begründung | Reaktivierung |
|---|---|---|---|
| **D-01** | **LSM-Segmente + MVCC (`Seq`) + Overlay-Branches** als Zustandsmodell; Git-Semantik (Fork/Diff/Merge/Zeitreise) darauf. | *Prolly-Tree/Merkle-DAG* (EngramDB, Versioned Storage): History-Independence und strukturelles Sharing, aber Pfadkopie je Schreibzugriff, Vektor-/Text-Indizes passen nicht in den Baum, Merge ist ohnehin recordweise. Für Bulk-Ingest von Unternehmensdokumenten ist LSM überlegen. | Wenn Content-Adressierung des Gesamtzustands (Audit-Anforderung) verlangt wird **und** Messung die Mehrkosten rechtfertigt. |
| **D-02** | **Keine signierten Commits, keine Merkle-Wurzel**; Integrität über Prüfsummen, Manifest-Verkettung, `scrub`. | Ed25519-Signaturen/Provenance-Signaturen (Gemini, EngramDB): Kosten und Schlüsselverwaltung ohne Produktnutzen; Nachweisanforderung ist Nicht-Ziel. | Bei expliziter Kundenanforderung als Add-on außerhalb des Kerns. |
| **D-03** | **Truth ≠ Index** mit Watermarks; kein Mehr-Store-2PC. | 4-Index-2PC mit Kompensation (Ist). | — |
| **D-04** | **`Gid` (u64, dicht)** als globaler Adressraum; Segmente nach `(gid, born_seq)` sortiert. | String-Schlüssel (Ist), `RecordId` 128 Bit als Bitmap-Schlüssel (zu breit). | — |
| **D-05** | **Derivate (Embedding, Summary, Tokens) per `(ContentHash, Producer)` gecacht**; Embedding-Spaces; Modellwechsel per Doppelbetrieb. | Embeddings als Teil des Records (mehrfache Berechnung, Merge-Konflikte). | — |
| **D-06** | **Löschung = Tombstone + Retention-Compaction; Purge best effort; kein Nachweis.** | Löschkette (Ist), Crypto-Shredding (MNEMOS): Nicht-Ziel. | — |
| **D-07** | **Store = harte Mandantengrenze**; innerhalb Namespaces + ACL-Bitmaps. | Mandanten-Präfixe im gemeinsamen Schlüsselraum (Ist). | — |
| **D-08** | **Vier Durability-Klassen** (`Strict/Group/Relaxed/Volatile`) statt `DurabilityMode`/`DeploymentTier`/`PerformanceProfile`-Kombinatorik. | Funktions-Ringe/Tiers. | — |
| **D-09** | **Ein Algorithmus je Aufgabe** (8.5); n-äre Relationen als reifizierte Facts. | 6 PPR-Varianten, PathRAG, Hyperkanten-Speicher, Perkolation (Ist). | Nur mit Benchmark-Nachweis, der den Standard schlägt. |
| **D-10** | **AEAD at Rest store-weit** (ein DEK, KEK-geschützt). | Per-Datei-/Per-Subject-Schlüssel. | Bei Mandanten-in-einem-Store-Anforderung. |
| **D-11** | **Branches ersetzen Checkpoint/Hardlink-Klonen und Session-DAG.** | Separate Checkpoint-Engine (Ist). | — |
| **D-12** | **Keine Lizenzlogik im Kern**; Lizenzierung als äußere Hülle. | `contextra-license` mit Ringen (Ist). | — |
| **D-13** | **Bandit-/PID-/Router-/Adapt-Schichten aus dem Kern entfernt.** | Forschungsnah, kein Bedarfsnachweis. | Bedarfsnachweis (zahlender Kunde/validierter Anwendungsfall) **und** Offline-Evaluation, die gegenüber einfacher Regel ≥ 10 % bessere Qualität/Kosten zeigt. |
| **D-14** | **Keine eigene Abfragesprache und keine SQL-Schicht in v1**; Rust-Builder + JSON-Plan + MCP. | GQL/Cypher-Dialekt (EngramDB), SQLite-Virtual-Tables (Versioned Storage). | Nach 1.0 bei belegter Nutzernachfrage. |
| **D-15** | **Ein Commit-Sequencer, ein WAL für alle Branches.** | WAL je Branch (mehr `fsync`s). | Falls R-16 eintritt. |
| **D-16** | **Zeitreise nur innerhalb Retention**; Zeit informativ, `Seq` maßgeblich. | Unbegrenzte Historie. | — |
| **D-17** | **Decay beeinflusst nur Ranking; Archivierung statt Löschen.** | Löschen unter Schwelle (Ist). | — |
| **D-18** | **Eigenimplementierung von WAL, Segmentformat, Vektorindex, Postings**; keine FFI-Index-Bibliotheken als Standard. | `usearch`, RocksDB, `sled`/`redb` als Kern. | — |
| **D-19** | **BM25F als Standard**, `k1 = 1,2`, `b = 0,75`; Deutsch-Analyzer produktiv. | `bm25f` optional/aus, `k1 = 1,5` (Ist). | — |
| **D-20** | **Single-Writer-Store pro Prozess**; Mehrprozess über Daemon. | Mehrprozess-Dateizugriff. | — |
| **D-21** | **Sessions schreiben nur in Session-Branches; Merge über Gates; Review-Merge als Standard.** | Direktes Schreiben nach `main`. | Privilegierter Pfad bleibt. |
| **D-22** | **Embeddings vor Commit** (Pipeline), `Deferred` nur explizit mit `lane_coverage`. | Asynchrone Embeddings als Standard (Auffindbarkeitslücke). | — |

---

## Anhang A — Dateiformate (Skizzen, normativ in `contextra-codec`)

Allgemeiner Dateikopf: `magic[4] | format_version u16 | compat_flags u16 | incompat_flags u32 | file_kind u8 | file_id u128 | created_hlc u64 | header_crc32c u32`. Magic je Art: `CWAL`, `CMLG` (Manifest-Log), `CSEG` (Truth), `CDTH` (Death), `CIDX` (Index), `CBLB` (Blob). Blöcke: `len u32 | codec u8 | crc32c u32 | payload`. Footer: Abschnittsverzeichnis `(kind, offset, len, crc)`, Statistiken, BLAKE3 über Datei. Alle Mehrbytewerte little-endian; alle Offsets 8-Byte-aligned, Vektor-/Sektorbereiche 4-KiB-aligned.

WAL-Payload-Typen: `Begin`, `PutRecord`, `PutEdge`, `Retire`, `SetAttrs`, `PutDerivation`, `CatalogOp`, `BranchOp`, `Commit{idem, hlc, branch, durability}`. Replay ist idempotent über `applied_lsn`.

## Anhang B — Fehlercodes (Auszug)

| Code | Retry | Bedeutung / Remediation |
|---|---|---|
| `E_CONFLICT` | ja (nach Neu-Lesen) | Write-Konflikt; Remediation: neu begin, Konflikt-ID prüfen |
| `E_SNAPSHOT_EXPIRED` / `E_SNAPSHOT_TOO_OLD` | ja | Lease abgelaufen / Snapshot vor Prune-Horizont |
| `E_BEYOND_RETENTION` | nein | Zeitreise außerhalb Retention; früheste `Seq` angegeben |
| `E_BACKPRESSURE` | ja | `retry_after_ms` |
| `E_HANDLE_UNKNOWN` | nein | mit `did_you_mean` |
| `E_FORBIDDEN_NAMESPACE` | nein | neutral (keine Existenzauskunft) |
| `E_VECTOR_INVALID` | nein | NaN/∞/Dimension |
| `E_BUDGET_TOO_SMALL` | nein | Mindestbudget angegeben |
| `E_CORRUPT_TRUTH` | nein | Prüfsummenfehler; Restore |
| `E_GATE_REJECTED` | nein | Gate und Grund angegeben |
| `E_IDEMPOTENT_REPLAY` | — | (kein Fehler) gespeichertes Ergebnis geliefert |
| `E_STORE_FAILED` | nein | fatale I/O-Störung; Neustart/Recovery |

## Anhang C — Beispiel: Agent mit Session-Branch

1. `ctx_session begin` → Branch `sess/planner/s-42` (O(1)); Quote 50 000 Zeilen, TTL 2 h.
2. `ctx_recall("Wie wurde der Deployment-Fehler im Oktober gelöst?", budget=6000)` → Evidence Pack mit Handles, 11 weiteren Handles via `expand`, Warnung `Conflict` (zwei Runbook-Versionen), `sufficiency = Sufficient`.
3. Agent bearbeitet, `ctx_remember(fact, prov={tool_call…}, idem=…)` je Schritt; Savepoints je Tool-Call.
4. `ctx_session propose` → Diff-Zusammenfassung („+3 Fakten, 1 ersetzt"), Gates G1–G6 laufen; G3 meldet Konflikt mit Fakt F-17.
5. Reviewer löst: neuer Fakt schließt alten bitemporal; `Merge` → atomarer Commit in `main`; Embeddings wiederverwendet; Index-Jobs ergänzen Segmente.
6. Fehlerfall: Agent bricht ab → `discard`; `main` unverändert; Reflog hält den Branch 1 Tag für Analyse.

## Anhang D — Glossar

| Begriff | Bedeutung |
|---|---|
| **Truth Layer** | Commit-Log + Truth-Segmente + Tabellen (Katalog, idmap, prov, emb, deriv, idem, alog, job); einzige Quelle der Wahrheit |
| **Index Layer** | Abgeleitete, neu aufbaubare Index-Segmente mit Watermark |
| **Gid** | Dichter globaler Record-Index für Bitmaps und Graph-Knoten |
| **Segment** | Unveränderliche Datei (Truth, Death oder Index) |
| **Death-Eintrag** | `(gid, row, died_seq)`: Version ab `died_seq` ersetzt/retired |
| **Shadow(B)** | Menge der in Branch B berührten Gids, maskiert Vorfahren |
| **Watermark** | `built_from_seq` eines Index-Segments; Rest wird per Flat/Mini ergänzt |
| **Lane** | Eine Retrieval-Modalität (Vektor, Text, Graph) |
| **Lane-Coverage** | Anteil der Zeilen mit fertigem Index je Lane |
| **Evidence Pack** | Budgetiertes, zitierfähiges Abfrageergebnis mit Warnungen und Erklärung |
| **Zoom-Stufe** | L0 Rohtext, L1 Zusammenfassung, L2 Gist |
| **Handle** | Kurzer, token-sparsamer Alias eines Records je Branch |
| **Retire** | Versionierter Tombstone (kein Löschnachweis) |
| **Retention** | Zeit-/Versionsfenster, nach dem Compaction tote Daten physisch entfernen darf |
| **Purge** | Administrativer Best-Effort-Eingriff: Tombstone + Retention-Override + Compaction |
| **Gate** | Deterministische Prüfung vor Merge nach `main` |
| **Derivat** | Aus Inhalt abgeleiteter, gecachter Wert (Embedding, Summary, Tokenzahl) |

---

*Ende der Spezifikation. Diese Datei ist der normative Soll-Zustand für Contextra; Änderungen erfolgen ausschließlich über Einträge im Entscheidungsregister (Abschnitt 19).*
