# Contextra – Gesamtspezifikation 4.0 (Reduktionsfassung, Kern v1)

<!-- doc-stamp: 2026-10-30 | Ersetzt: Fassung 3.1 (Baukasten-Fassung, Zielzustand v1.0) -->

| | |
|---|---|
| **Status** | Normativ für den reduzierten Scope (Kern v1). Ersetzt Fassung 3.1 vollständig. |
| **Stand** | 30.10.2026 |
| **Code-Basis** | Repository-HEAD nach Reduktionscommit `ed402e21a`, Folgecommits `068261abd` (Entfernung `xtask/`) und `2870ec929` |
| **Wiederherstellungspunkt** | Commit `3c1f144fa` (letzter vollständiger Stand vor der Reduktion) |
| **Build-Status** | **Rot.** Der Workspace lädt nicht vollständig und kompiliert nicht. Details: §16 und `docs/PLAN.md` Phase R0. |

---

## 0. Schnellstart für Menschen und LLM-Agenten

**Was Contextra in v1 ist:** Eine eingebettete, lokal betriebene Rust-Bibliothek (Single-Node) für hybride Suche aus HNSW-Vektorsuche und BM25-Textsuche, mit Mandantentrennung, verschlüsselter Persistenz (WAL und SSTables), MVCC-Transaktionen mit Serializable-Snapshot-Isolation (SSI) und einer Rust-API (`contextra`, `contextra-db`).

**Was Contextra in v1 nicht ist:** Kein Inferenz-Server, kein Agenten-Framework, kein MCP-Server, keine Python-Bindings, keine Dokumenten-Ingestion, kein Compliance-Werkzeug mit Löschbeweis. Diese Teile wurden in der Reduktion vom 30.10.2026 entfernt (§5.2).

**Lesereihenfolge:**
1. `docs/CONSTITUTION.md` und `docs/VETOES.md` (bindend)
2. Diese Datei, §2 bis §4 (Scope, Regeln, Architektur)
3. `docs/PLAN.md`, Phase R0 (was jetzt zu tun ist)
4. `AGENTS.md` bzw. `CLAUDE.md` (Arbeitsregeln für Agenten)

**Vor jeder Änderung:** Den Build-Status (§0, Tabelle oben) prüfen. Solange R0 nicht abgeschlossen ist, sind Befehle wie `cargo check --workspace` das Ziel, nicht der Ist-Zustand.

---

## 1. Geltung, Belegbasis, Konventionen

### 1.1 Rangfolge bei Widersprüchen

1. `docs/CONSTITUTION.md` und `docs/VETOES.md`
2. Diese Spezifikation (Fassung 4.0)
3. `docs/PLAN.md` (Fassung 4.0)
4. Quelltext und Manifeste (`Cargo.toml`, `crates/*/Cargo.toml`, `crates/*/src`)
5. `capabilities.toml` und `governance/*.toml` (werden in PLAN Phase R2 an den Ist-Stand angeglichen; bis dahin nachrangig)
6. `AGENTS.md`, `CLAUDE.md`, `README.md`, `docs/ARCHITECTURE.md` (abgeleitete Dokumente)

Historische Dokumente (`docs/CHANGELOG.md`, `docs/decisions/`, `docs/analysis/`, `docs/spec/`, `REDUCTION_RESIDUALS.md`, `.agents/jules/*LOG*`) sind nicht normativ. Sie tragen seit der Reduktion einen Statushinweis.

### 1.2 Kennzeichnungen

| Kennzeichen | Bedeutung |
|---|---|
| **[K]** | Kern: Bestandteil von v1 |
| **[T]** | Testwerkzeug: nur für Tests und Benchmarks, nicht im Produktpfad |
| **[A]** | Entfernt: nicht im Workspace. Wiederherstellung nur über Git-Historie (§5.2) |
| **[O]** | Offen: Zugehöriger Code ist vorhanden, der Scope aber nicht entschieden (§15) |

### 1.3 Pfadkonvention

Pfade sind relativ zum Repository-Root. Crate-Pfade lauten `crates/contextra-<name>/`. Die Crate-Namen in Code (`contextra_<name>`) entsprechen den Paketnamen mit Unterstrich.

---

## 2. Produkt und Scope

### 2.1 Zweck

Lokales Wissens- und Speichermanagement für Anwendungen ohne Cloud-Zwang. Typische Nutzung: Retrieval-Schicht für RAG-Anwendungen, die Dokumentfragmente mit Vektor- und Stichwortsuche finden und dabei Mandanten strikt trennen.

### 2.2 Funktionsumfang v1

- **Collections und Dokumente** mit Mandantenbezug. Der Mandant wird ausschließlich aus dem `RequestContext` übernommen (VETO-F10).
- **Hybride Suche**: HNSW-Vektorindex, BM25-Textindex und Fusion der Ranglisten (Reciprocal Rank Fusion, Kalibrierung über `contextra-rank`).
- **Transaktionen** mit MVCC-Snapshots und SSI-Validierung (`contextra-mvcc`).
- **Persistenz** als LSM-Baum mit Write-Ahead-Log, MemTable, SSTables, Manifest und Hintergrund-Kompaktion (`contextra-store`).
- **Verschlüsselung at rest** in jedem Build aktiv: AEAD-Blockverschlüsselung, Schlüsselableitung per HKDF, WAL-Integrität (`contextra-crypto`). Regel 8 in §3.
- **Logisches Löschen** über Tombstones (§9).
- **Eingebettete Rust-API** über die Fassade `contextra` (§11).

### 2.3 Nicht-Ziele v1

Diese Fähigkeiten sind in v1 **nicht** implementiert und dürfen nur über einen neuen ADR aufgenommen werden:

- Verteilter Betrieb: kein Sharding, kein P2P, keine verteilte Replikation (Single-Node-Doktrin)
- Echtzeit-Audio und Speech-to-Text (VETO-OP03)
- Eingebettete Inferenz: lokale Embedding- und LLM-Backends (Candle, ONNX, Ollama). Entfernt, §5.2.
- Agenten-Laufzeit, Router, Graph-Reasoning und Konsolidierung über Schlafzyklen. Entfernt, §5.2.
- MCP-Server, Python-Bindings (PyO3), WASM-Sandbox. Entfernt, §5.2.
- Dokumenten-Ingestion (PDF, DOCX, E-Mail und weitere Formate). Entfernt, §5.2.
- PII-Filter und Egress-Gateway. Entfernt, §5.2.
- KV-Cache-Verwaltung. Entfernt, §5.2.
- DiskANN als Indextyp. Entfernt (Dateien `contextra-vector/src/diskann`).
- Löschbeweis als Rechtsnachweis und Compliance-Zusagen. Siehe §9.
- Windows als Garantieplattform. Best-Effort über `acl_win32.rs`.

**Offen:** RaBitQ-Quantisierung ist im Code vorhanden (`contextra-vector/src/quantize_rabitq.rs`, Feature `experimental-rabitq`), widerspricht aber dem Nicht-Ziel „Indexerweiterungen ohne ADR“. Entscheidung D-R4 (§15).

### 2.4 Plattform und Toolchain

- Rust-Edition 2021, MSRV **1.89** (`[workspace.package] rust-version`).
- Toolchain-Pin über `rust-toolchain.toml`. Entwicklungsumgebung über `flake.nix`.
- Referenzplattformen: Linux und macOS. Windows: Best-Effort.

---

## 3. Verfassung, Vetos und Arbeitsregeln

### 3.1 Vetos (bindend, Details in `docs/VETOES.md`)

| Veto | Regel | Status |
|---|---|---|
| **VETO-F10** | Keine mandantenübergreifenden Datenflüsse. Kein Cache, Index, Embedding, Statistik oder Audit-Eintrag überschreitet Mandantengrenzen. | permanent abgelehnt |
| **VETO-F02** | HNSW-Löschung nur per Tombstone, Slot-Nullung und Rebuild. Kein Rewiring der Nachbarn. Slot-Nummern werden erst nach Abschluss des Rebuilds wiederverwendet. | bestätigt |
| **VETO-OP03** | Kein Echtzeit-Audio und kein Audio-Streaming im Kern. | bestätigt, geschlossen |
| **Single-Node-Doktrin** | Kein Sharding, kein P2P, keine verteilte Replikation. | permanent |

### 3.2 Die zwölf Regeln für jede Änderung

Diese Regeln sind normativ. Verletzungen sind Fehler, auch wenn kein automatischer Check sie erkennt.

| # | Regel | Ist-Prüfung in v1 |
|---|---|---|
| 1 | **Ring und Klasse zuerst.** Abhängigkeiten zeigen nur auf gleichen oder niedrigeren Ring (§4). | `tests/layering.rs` (muss an §4 angeglichen werden, R2) |
| 2 | **Kein Panic in Produktion.** Kein `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unreachable!`, `exit`, kein ungeprüftes Indexieren. Fehler sind typisiert. | Workspace-Clippy-Lints (§12.1) |
| 3 | **Zeit, Zufall und IDs nur über Ports** (`Clock`, `Rng`, `IdGen`). | Konvention, Prüfung über Review |
| 4 | **Persistentes läuft über das WAL.** Sicherheitszustand wird über WAL und Manifest persistiert. Eigene Sidecar-Logs gibt es in v1 nicht. | Tests in `contextra-store` |
| 5 | **I/O nur über `Vfs`** (`ports::vfs`). `fsync`-Fehler werden propagiert und nie mit `let _ =` verworfen. | Konvention, Clippy-Lint `let_underscore_must_use` (R2 zu prüfen) |
| 6 | **Unsafe nur in zwei Inseln:** `contextra-sys` und `contextra-simd`. Jeder `unsafe`-Block trägt `// SAFETY:`. Alle übrigen Crates setzen `#![forbid(unsafe_code)]`. | `tests/unsafe_islands.rs` |
| 7 | **Mandant aus dem `RequestContext`**, nie aus Literalen, Defaults oder Nutzerparametern. | Tests in `contextra/tests/tenant_isolation_facade.rs` |
| 8 | **Sicherheit ist kein Feature-Schalter.** Verschlüsselung und Integritätsprüfung sind in jedem Build aktiv. | Offen: Feature `encryption-at-rest` ist im Code verwendet, nicht deklariert (§16, B4) |
| 9 | **Messen vor Behaupten.** Löschaussagen und Leistungsangaben entstehen nur aus Messungen oder Tests. | Review, `contextra-bench` |
| 10 | **Fail-Closed.** Unbekannter Zustand, unbekannte Formatversion oder gebrochene Kette führen zur Ablehnung mit benanntem Grund. | Tests in `contextra-store` und `contextra-crypto` |
| 11 | **Eine Typfamilie je Konzept.** Doppeldefinitionen sind verboten (§16, B7). | Offen: Typ-Registry in `docs/TYPE_REGISTRY.md` |
| 12 | **Ein PR = ein Arbeitspaket** (WP aus `docs/PLAN.md`), höchstens ein Crate-Schnitt. | Review |

---

## 4. Architektur und Ringmodell

### 4.1 Ringe

Das Ringmodell ordnet Crates nach Abhängigkeitstiefe. Eine Abhängigkeit darf nur auf denselben oder einen niedrigeren Ring zeigen. Crates in Tooling-Ringen werden nur als `dev-dependency` eingebunden.

| Ring | Bedeutung | Crates |
|---|---|---|
| **Ring 0** | Fundament: Typen, Ports, Isolation, Algorithmen ohne Persistenz | `contextra-types`, `contextra-ports`, `contextra-sys` (U), `contextra-simd` (U), `contextra-mvcc`, `contextra-crypto`, `contextra-vector`, `contextra-text`, `contextra-rank` |
| **Ring 1** | Persistenz | `contextra-store` |
| **Ring 2** | Orchestrierung: Collections, Suche, Transaktionen über Indizes und Speicher | `contextra-engine` |
| **Ring 3** | Fassade für Anwendungen | `contextra-db` |
| **Ring 4** | Composition Root und öffentliche Schnittstelle | `contextra` |
| **Tooling** | Nur Tests und Benchmarks | `contextra-testkit`, `contextra-bench` |

(U) = Unsafe-Insel.

**Änderung gegenüber Fassung 3.1:** In 3.1 lag `contextra-engine` in Ring 3. Die Ist-Abhängigkeiten zeigen, dass `engine` ausschließlich unterhalb von `db` steht und keine Fassadenfunktion übernimmt. Die Neuzuordnung nach Ring 2 ist Vorschlag D-R3 (§15) und wird mit Anpassung von `tests/layering.rs` wirksam.

### 4.2 Ist-Abhängigkeiten (Workspace-intern, aus den Manifesten)

| Crate | hängt ab von |
|---|---|
| `types` | – |
| `ports` | `types` |
| `sys` | – |
| `simd` | `types` |
| `mvcc` | `ports`, `types` |
| `crypto` | `ports`, `types` |
| `text` | `ports`, `types` |
| `rank` | – |
| `vector` | `crypto`, `mvcc`, `ports`, `simd`, `sys`, `types` |
| `store` | `crypto`, `mvcc`, `ports`, `sys`, `types` |
| `engine` | `crypto`, `mvcc`, `ports`, `rank`, `store`, `sys`, `text`, `types`, `vector` |
| `db` | `crypto`, `engine`, `ports`, `rank`, `store`, `sys`, `text`, `types`, `vector` |
| `contextra` | `crypto`, `db`, `ports`, `rank`, `store`, `types` |
| `testkit` | `ports`, `types` |

**Kernbefund:** `rank` ist die einzige Ring-0-Crate ohne Workspace-Abhängigkeit. `crypto` und `mvcc` stehen auf `ports`. Ring-0-Crates dürfen nicht von `store` oder höher abhängen. Das wird eingehalten.

### 4.3 Grundregeln

- Ein Crate kennt nur die Ports, die es braucht. Konkrete Implementierungen werden über Konstruktoren eingebunden (Composition Root: `contextra`).
- Ports sind dyn-kompatibel und liegen ausschließlich in `contextra-ports`.
- Tokio darf in Ring 0 nicht vorkommen (Prüfung in `tests/layering.rs`, R2).

---

## 5. Modulkatalog

### 5.1 Verbliebene Crates

| Crate | Ring | Klasse | Zweck (laut `Cargo.toml`) | Stand |
|---|---|---|---|---|
| `contextra-types` | 0 | [K] | Kanonische Domänentypen, IDs, Budgets, Filter und Fehlertypen | Enthält Restmodul `types/saos.rs` (§16, B7) |
| `contextra-ports` | 0 | [K] | Dyn-kompatible Port-Traits für alle Subsysteme | Einige Ports bedienen nur entfernte Crates (§6) |
| `contextra-sys` | 0 | [K] (U) | Systemnahe Abstraktionen und FFI-Insel: `mmap`, `mlock`, `posix`, `durability`, `vault`, `acl_win32` | 21 `unsafe`-Stellen |
| `contextra-simd` | 0 | [K] (U) | SIMD-Distanzkerne mit Laufzeit-Dispatch | Größte `unsafe`-Fläche (rund 270 Stellen) |
| `contextra-mvcc` | 0 | [K] | MVCC: Sequenzlog, Snapshot-Registry, Transaktionspuffer, SSI | Stabil |
| `contextra-crypto` | 0 | [K] | Kryptografischer Kern: AEAD, HKDF, WAL-Verschlüsselung, Audit-Kette, Anti-Tamper | Enthält Feature-Konflikte (§16, B4) |
| `contextra-vector` | 0 | [K] | HNSW-Index mit SIMD-Distanz, SQ8-Quantisierung, Partial-Rebuild | RaBitQ [O] |
| `contextra-text` | 0 | [K] | BM25, invertierter Index, WAND, deutsche Morphologie, Domänenwörterbücher (Recht, Medizin) | Stabil |
| `contextra-rank` | 0 | [K] | Fusion, Isotone und Platt-Kalibrierung, Drift-Erkennung, Erklärbarkeit | Modul `dibud` [O] |
| `contextra-store` | 1 | [K] | LSM-Baum: WAL, MemTable, SSTable, Manifest, Kompaktion, Block-Cache | Enthält Residuen (§16, B2) |
| `contextra-engine` | 2 | [K] | Collections, Hybridsuche, Transaktionen, Chunker, Filter, Hintergrundarbeiter | Größte Residuenfläche (§16, B2) |
| `contextra-db` | 3 | [K] | Fassade für Anwendungen: `prelude` und `advanced` | Enthält Residuen (§16, B2) |
| `contextra` | 4 | [K] | Composition Root, öffentliche Crate-API (`builder`, `open`, `open_with_config`) | Build-Blocker: `contextra_core` (§16, B3) |
| `contextra-testkit` | Tooling | [T] | ManualClock, InMemory-Speicher, FaultVfs, ReferenceModel, Konformitätssuiten | Enthält Doppeldefinition (B7) |
| `contextra-bench` (`benchmarks/`) | Tooling | [T] | Benchmarks: ANN, BEIR, Pfad-RAG, Regressions-Gate, Vergleichsläufe | Enthält Residuen (B2) |

### 5.2 Entfernte Crates [A]

Die Reduktion hat die folgenden Crates aus dem Workspace entfernt. Sie sind nicht mehr Teil des Repositorys, aber über Git wiederherstellbar.

| Gruppe | Crates |
|---|---|
| **B** (wiederherstellbar) | `contextra-infer-ollama`, `contextra-privacy`, `contextra-ingest`, `contextra-mcp`, `contextra-wire`, `contextra-checkpoint`, `contextra-audit-export`, `contextra-avv-generator` |
| **C** (wiederherstellbar) | `contextra-adapt`, `contextra-agent`, `contextra-cognition`, `contextra-graph`, `contextra-infer-candle`, `contextra-infer-onnx`, `contextra-kvcache`, `contextra-router`, `contextra-sandbox`, `contextra-py`, `contextra-license` |
| **Kern-Altlast** | `contextra-core` (Kompatibilitätsfassade, nicht wiederhergestellt) |
| **Teilweise entfernt** | `contextra-crypto`: `deletion_proof`, `kv_segment`, `kv_shredding`, `revocation_log`, `ed25519_proof`, `wal_completeness`. `contextra-store`: `scanner`. `contextra-engine`: `extraction`, `decay_controller`, `contextra_impl/erasure`. `contextra-db`: `multistep`, `homeostat`, Beispiel `verify_deletion_proof_external`. `contextra-vector`: `diskann` |

**Restore-Befehl** (Beispiel für ein Crate):

```bash
git checkout 3c1f144fa -- crates/contextra-graph
```

Hinweis: Der im Residuen-Bericht genannte Tag `pre-reduction-1.0` existiert im Repository nicht. Maßgeblich ist der Commit `3c1f144fa` (D-R6).

---

## 6. Ports (Vertragsschicht)

`contextra-ports` ist die einzige Stelle, an der Verträge zwischen Subsystemen definiert werden. Die Ports sind dyn-kompatibel.

| Port (Modul) | Status | Hinweis |
|---|---|---|
| `storage`, `kv`, `kv_bridge_port` | [K] bzw. [O] | `kv_bridge_port` [O]: Prüfen, ob Konsumenten existieren |
| `vector_index`, `text_index` | [K] | Von `vector`, `text`, `engine` genutzt |
| `embedding`, `reranker`, `query_rewriter` | [K] bzw. [O] | Implementierungen entfernt; Trait und Konsumenten prüfen (R2) |
| `clock`, `rng`, `id_gen` | [K] | Determinismus-Ports (Regel 3) |
| `vfs` | [K] | I/O-Port (Regel 5) |
| `lifecycle`, `metrics`, `observability` | [K] | Betrieb und Telemetrie |
| `contract_tests` | [T] | Konformitätstests für Port-Implementierungen |
| `graph`, `graph_index` | [O] | Graph ist entfernt. Port nur behalten, wenn ein Konsument in v1 existiert |
| `attention`, `plugin`, `license`, `checkpoint`, `scanner` | [O] | Bedienen entfernte Crates. Kandidaten für Entfernung (R2) |

**Zielzustand:** Nach Phase R2 enthält `contextra-ports` nur Ports mit mindestens einem Konsumenten in v1.

---

## 7. Isolierte Prüfbarkeit

- Jedes verbliebene Crate hat eigene Tests unter `crates/<name>/tests/`. Die Testabdeckung variiert stark (vgl. `contextra-store` mit rund 230 Dateien gegenüber `contextra-sys` mit rund 10).
- `contextra-testkit` liefert deterministische Ersatzimplementierungen: `ManualClock`, `InMemoryStorageEngine`, `InMemoryVectorIndex`, `InMemoryTextIndex`, `InMemoryGraphIndex`, `FaultVfs`, `ReferenceModel`. Der Graph-Index ist Relikt aus der entfernten Graph-Crate (B7, B2).
- Workspace-Tests unter `tests/`: `layering.rs` (Ringregeln) und `unsafe_islands.rs` (Unsafe-Verbot außerhalb der Inseln).
- Beispiele: `crates/contextra-db/examples/` (`quickstart`, `hybrid_search`, `collections`) und `examples/` im Root.

---

## 8. Persistenz und Recovery

### 8.1 Datenpfad

Schreibpfad: Client → `TxBuffer` (`mvcc`) → WAL (`store::wal`) → MemTable (`store::memtable`) → SSTable (`store::sstable`) → Kompaktion (`store::compaction`).

### 8.2 Bausteine

- **WAL**: Append-only, HMAC-Verkettung, CRC32, Flusher-Actor, Replay, Grenzprüfung. Verschlüsselung über `crypto::wal_crypto`.
- **MemTable**: Skip-Liste mit Sequenznummern, Tombstones.
- **SSTable**: Blockweise, mit Bloom-Filter, Index, CRC32 und Kompression. Block-Cache mit LRU als Standard, SIEVE als Option (Feature `sieve-cache`).
- **Manifest**: Transaktionale Verwaltung der SSTable-Generationen.
- **Kompaktion**: Hintergrund-Merge (Tiered oder Leveled).
- **Mandantenkodierung**: `tenant_codec` präfixiert Schlüssel mit Mandantenbezug.
- **Systemdruck**: `system_pressure` steuert Flush und Drosselung.

### 8.3 Recovery-Garantien

- Beim Öffnen wird eine Reparatur ausgeführt (`repair_on_open`, Engine-Invariante). Ergebnis ist ein konsistenter Zustand oder ein benannter Fehler (Regel 10).
- Jeder `sync_all()`- und `sync_data()`-Aufruf propagiert Fehler mit `?`. `let _ = …sync_all()` ist verboten.
- Atomare Umbenennung und Integritätsschlüssel werden in `store::util` gekapselt.

### 8.4 Kryptografie

- Schlüsselableitung über HKDF (`crypto::kdf`). **Offen:** Die Fassung 3.1 verlangte für bestimmte Pfade Argon2id (INV-KDF-1). Entscheidung in D-R9 (§15).
- Blockverschlüsselung über AEAD (`crypto::kv_cipher`). Alle Schlüssel werden beim Verwerfen gelöscht (zeroized).
- Audit-Kette (`crypto::audit_chain`) und Anti-Tamper (`crypto::anti_tamper`) sind vorhanden. Ob sie Teil von v1 sind, ist offen (D-R9).

---

## 9. Löschung (reduzierter Umfang)

### 9.1 Was v1 nicht enthält

Die Löschkette aus Fassung 3.1, §9 ist **aus dem Scope v1 genommen**. Entfernt sind unter anderem: Layer-Scanner, Schlüsselvernichtung als Ausführungsstufe, `ErasedSet` (Lesesperre), Löschsaga, Backup-Guard, Empfängerregister (`RecipientLedger`), signierter Löschbericht V5 (`DeletionProof`) und die Auflistung der Löschschichten.

Zugehörige Dokumente (ADR-0001 bis ADR-0008, Referenzabgleich vom 29.10.2026) sind **historisch** und nicht mehr bindend. Eine Wiederaufnahme erfordert einen neuen ADR mit eigenem Scope (D-R5).

### 9.2 Was v1 bietet

- **Logisches Löschen** über Tombstones in MemTable und SSTable. Der Tombstone verdeckt ältere Versionen für Leser.
- **Physisches Entfernen** erfolgt bei der Kompaktion. Der Zeitpunkt ist nicht zugesichert.
- **Mandantenbezogenes Löschen** über den Mandantenschlüssel-Präfix (`tenant_codec`).
- **HNSW-Löschung** nach VETO-F02 (Tombstone, Slot-Nullung, Rebuild).

### 9.3 Aussagen, die nicht getroffen werden dürfen

- Keine Aussage, dass Daten „sicher gelöscht“, „vernichtet“ oder „DSGVO-konform gelöscht“ sind.
- Keine Aussage über Wirksamkeit gegenüber Flash-Speicher, Sicherungen, Swap oder Kopien bei Dritten.
- Der Löschbefund ist technisch, keine Rechtsauskunft und keine Zertifizierung.

---

## 10. Retrieval und Hybridsuche

### 10.1 Vektorsuche (`contextra-vector`)

- HNSW-Index mit Partial-Rebuild (Feature `partial-index-rebuild`, im Code vielfach referenziert).
- SIMD-Distanzkerne mit Laufzeit-Dispatch aus `contextra-simd`.
- Quantisierung: SQ8 (`quantize`). RaBitQ (`quantize_rabitq`, Feature `experimental-rabitq`) ist [O] (D-R4).
- Gefilterte Suche über das Modul `acorn`.
- Löschung nach VETO-F02.

### 10.2 Textsuche (`contextra-text`)

- BM25 mit Feldgewichtung (Feature `bm25f`).
- Invertierter Index mit WAND-Top-K.
- Deutsche Morphologie: Stemming-Trie, Stoppwörter, Komposita-Zerlegung.
- Domänenwörterbücher `legal_de` und `medical_de`.

### 10.3 Fusion und Kalibrierung (`contextra-rank`)

- Fusion mehrerer Signale (RRF, Top-K).
- Kalibrierung: Isotone Regression und Platt-Skalierung.
- Drift-Erkennung und Erklärbarkeit (`explain`).

### 10.4 Orchestrierung (`contextra-engine`)

- Hybrid-Query-Builder (`collection::query_builder`).
- Chunker (`chunker`) mit Abschnitts-, Fenster- und Blockstrategien.
- Zeitlicher Filter (`temporal_filter`) und Import/Export.

### 10.5 Qualitätsziele

Die Messung erfolgt in `contextra-bench` (ANN-Benchmarks, BEIR, Pfad-RAG, Regressions-Gate). Numerische Ziele werden nicht in dieser Spezifikation festgelegt, sondern aus den Baselines in `benchmarks/results/` abgeleitet. Regel 9 gilt.

---

## 11. Öffentliche API

### 11.1 Fassade `contextra` (Ring 4)

- Einstieg: `builder(dimension)`, `open(path)`, `open_with_config(path, config)`.
- Re-Exporte: `ContextraBuilder`, `Collection`, `CollectionConfig`, `Contextra`, `ContextraConfig`, `SearchResult`, `DriftStatusProvider`, `EmbeddingBackend`, `TextEmbeddingEngine`, `LlmQueryRewriter`, `AgentMemory`, `Memory`, `MemoryId`, `ContextraError`, `DocId`, `ScoredDocument`, `DistanceMetric`.
- Modul `rank` als Re-Export von `contextra-rank`.
- Invarianten: `#![forbid(unsafe_code)]`, höchstens 20 öffentliche Funktionen (Regel), keine direkten Abhängigkeiten auf Ring 0/1 außer über `db`.

**Befund:** `contextra/src/lib.rs` importiert `contextra_core` und enthält ein Feature-Modul `candle`. Beides ist nach der Reduktion nicht mehr vorhanden (§16, B3). Der Zielzustand ist nach Phase R0 hergestellt.

### 11.2 Fassade `contextra-db` (Ring 3)

- `prelude`: stabile Kern-API für Einfügen, Suchen, Löschen (empfohlen).
- `advanced`: erweiterte Subsystem-Schnittstellen. Die Beschreibung im Modul-Kopf verweist noch auf `adapt`, `cognition` und `graph`. Diese Verweise sind zu entfernen (D-R9).

### 11.3 Stabilität

- Änderungen an `prelude` oder den Re-Exporten der Fassade brauchen einen Eintrag in `docs/CHANGELOG.md`.
- Der Pre-1.0-Status (Workspace-Version 0.1.0) erlaubt Breaking Changes, sie müssen aber dokumentiert werden.

---

## 12. Qualitätssicherung

### 12.1 Workspace-Lints

In `Cargo.toml` unter `[workspace.lints]` gilt für alle Mitglieder:

- Rust: `unsafe_code = "deny"`, `unsafe_op_in_unsafe_fn = "deny"`, `unexpected_cfgs = "deny"`.
- Clippy (deny): `undocumented_unsafe_blocks`, `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `indexing_slicing`, `exit`, `dbg_macro`, `await_holding_lock`, `await_holding_refcell_ref`.
- Clippy (warn): `cast_possible_truncation`, `cast_sign_loss`.

**Konsequenz:** `unexpected_cfgs = "deny"` macht jeden Verweis auf ein nicht deklariertes Feature zu einem Fehler (§16, B4).

### 12.2 Pflichtbefehle (Ziel nach R0)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Der frühere Befehlssatz `cargo xtask …` ist nicht mehr verfügbar (§16, B1, D-R2).

### 12.3 Gates

- Der Gate-Katalog `governance/gates.toml` bleibt als Quelle erhalten. Der Runner (`xtask`) wurde mit Commit `068261abd` entfernt. Gates sind damit derzeit **nicht ausführbar**.
- Die 34 Workflows in `.github/workflows/` referenzieren überwiegend `cargo xtask` oder entfernte Komponenten (Python, MCP, PyPI). Bereinigung in R2.
- Ratchet-Baselines (`governance/ratchet.toml`) dürfen nur sinken (REQ-QA-005). Nach der Reduktion sind die Zahlen neu zu setzen, mit Vermerk im Changelog (D-R6).

---

## 13. Betrieb, Lieferkette, Release

- Lieferkette: `deny.toml` (cargo-deny), `supply-chain/` (cargo-vet mit `audits.toml`, `imports.lock`), `.cargo/audit.toml` (cargo-audit).
- Benchmarks sind getrennt vom Produktpfad und laufen über `contextra-bench`.
- **Versionierung:** Der Workspace steht auf `0.1.0`. Die Fassung 3.1 sprach von einem Zielzustand v1.0. Die Release-Version ist Entscheidung D-R7.
- **Relikte im Release-Pfad:** `publish-pypi.yml`, `build-wheels.yml` und `release-mcp-binary.yml` beziehen sich auf entfernte Komponenten (Python-Wheels, MCP-Binary). Sie sind zu entfernen (R2).

---

## 14. Datenlebenszyklus (Kurzfassung)

- Alle Daten liegen im lokalen Datenverzeichnis des Betreibers. Es gibt keinen Cloud-Zwang und keine Telemetrie-Pflicht.
- Persistente Daten sind verschlüsselt (§8.4).
- Löschung ist logisch (Tombstone) und wird physisch bei der Kompaktion umgesetzt. Die Zusicherungen aus §9.3 gelten.
- Externe Sicherungen, Datenträger-Firmware, Swap, Core-Dumps und Kopien bei Dritten liegen außerhalb des Einflussbereichs von Contextra.

---

## 15. Entscheidungen dieser Fassung (ADR-D-R)

| ID | Thema | Entscheidung | Status |
|---|---|---|---|
| **D-R1** | Scope v1 | Kern v1 umfasst die Crates in §5.1. Alles in §5.2 ist entfernt. | Entschieden (Reduktion) |
| **D-R2** | Gate-Runner | Option (a): minimalen `xtask` neu aufbauen, der `governance/gates.toml` auswertet. Option (b): Gates als direkte `cargo`-Befehle in `justfile` und CI. Empfehlung: (b) für v1, (a) nach Release. | **Offen** |
| **D-R3** | Ring von `engine` | Wechsel von Ring 3 auf Ring 2 (§4.1). | Vorschlag, Bestätigung nötig |
| **D-R4** | RaBitQ | Freigabe als `experimental` mit Test- und Messnachweis oder Entfernung. | **Offen** |
| **D-R5** | Löschkette | Ausgesetzt. Wiederaufnahme nur über neuen ADR mit eigenem Scope. | Entschieden |
| **D-R6** | Restore-Verfahren | Maßgeblich ist Commit `3c1f144fa`, nicht der Tag `pre-reduction-1.0`. Ratchet-Baselines werden nach R0 neu gesetzt. | Entschieden |
| **D-R7** | Release-Version | `0.1.0` (Pre-1.0) oder `1.0.0` nach Abschluss von Phase R4. | **Offen** |
| **D-R8** | Nicht-Ziele | Python-Bindings, MCP, Ingestion, Inferenz und Agenten-Laufzeit sind v1-Nicht-Ziele. Re-Entry nur über eigenen Plan und ADR. | Entschieden |
| **D-R9** | Kryptografie und API-Reste | (i) HKDF oder Argon2id für Passwortpfade. (ii) Ob `audit_chain` und `anti_tamper` in v1 bleiben. (iii) Bereinigung der `advanced`-Fassade. | **Offen** |

---

## 16. Befunde und offene Punkte (Stand 30.10.2026)

Die Befunde beruhen auf statischer Prüfung des Repositorys (keine lokale Kompilierung, da in der Prüfumgebung kein Rust-Toolchain verfügbar war). Sie sind vor Phase R0 durch `cargo check --workspace` zu bestätigen.

| ID | Befund | Ort | Auswirkung |
|---|---|---|---|
| **B1** | `Cargo.toml` listet `xtask` als Workspace-Mitglied, das Verzeichnis fehlt (entfernt mit `068261abd`). | `Cargo.toml` | Workspace kann nicht geladen werden |
| **B2** | Quelltext referenziert entfernte Crates (`contextra_graph`, `contextra_adapt`, `contextra_wire`, `contextra_cognition`, `contextra_infer_*`, `contextra_privacy`, `contextra_mcp`, `contextra_checkpoint`). Repositoryweit 51 Rust-Dateien betroffen; die Crate-Zuordnung ergibt 50 Treffer: `engine` 29, `db` 12, `types` 3, `rank` 2, `contextra` 2, `store` 1, `bench` 1. Zusätzlich 44 Dateien mit entfernten Bezeichnern (`DiskAnn`, `DeletionProof`, `Homeostat`, `MultiStep`, `erasure`, `decay_controller`). | `crates/*/src`, `benchmarks/` | Unaufgelöste Pfade beim Kompilieren |
| **B3** | `contextra_core` wird in 12 Dateien verwendet. Die Crate ist entfernt und nicht mehr Abhängigkeit der Fassade. | `crates/contextra/src/lib.rs` u. a. | Fassade kompiliert nicht |
| **B4** | Mindestens 14 Feature-Namen werden in `cfg(feature = …)` verwendet, aber in keinem `Cargo.toml` deklariert. Beispiele: `entity-extraction` (20 Stellen), `encryption-at-rest` (11), `wal-integrity` (7), `sandbox`, `onnx`, `onnx-bench`, `candle`, `reranking`, `experimental-diskann`, `graph-connectivity-health`, `adaptive-candidate-pool-sizing`, `coherence-bonus-fusion`. Mit `unexpected_cfgs = "deny"` sind das Fehler. `encryption-at-rest` berührt Regel 8. | Mehrere Crates | Kompilierfehler; Sicherheitspfad teils als Feature behandelt |
| **B5** | `capabilities.toml` (98 Verweise auf entfernte Crates) und `governance/gates.toml` beschreiben den Vorzustand. Der Gate-Runner fehlt (B1). | Root | Gate-Status nicht prüfbar |
| **B6** | Die Dokumentation (Fassung 3.1, README, AGENTS, CLAUDE, ARCHITECTURE, TYPE_REGISTRY) beschreibt den Vorzustand. Dieser Stand wird in Phase R3 bereinigt. | `docs/`, Root | Falsche Erwartungen für Agenten und Menschen |
| **B7** | Doppeldefinitionen öffentlicher Typen: `BoundedTopK` (`text/src/wand.rs`, `rank/src/fusion/topk.rs`), `HybridQueryBuilder` (`engine`, `types/src/types/saos.rs`), `InMemoryStorageEngine` (`testkit`, `store/src/engine/memory_only.rs`), `Signal` (`types/src/types/saos.rs`, `rank/src/fusion/types.rs`). `types/saos.rs` ist ein Restmodul aus dem entfernten Agenten-Kontext. | siehe Liste | Verstoß gegen Regel 11 |
| **B8** | RaBitQ-Code vorhanden, Nicht-Ziel laut §2.3. | `contextra-vector` | Spezifikation und Code widersprechen sich (D-R4) |
| **B9** | 34 Workflows, davon überwiegend mit `cargo xtask` oder entfernten Komponenten. | `.github/workflows/` | CI rot oder nicht aussagekräftig |
| **B10** | Der Restore-Tag `pre-reduction-1.0` aus dem Residuen-Bericht existiert nicht. | Git | Wiederherstellung schlägt fehl (D-R6) |
| **B11** | `capabilities.toml`, `tests/layering.rs` und `docs/ARCHITECTURE.md` führen Ring-Zuordnungen nach Fassung 3.1, die sich mit §4 nicht decken (u. a. `engine` in Ring 3). | `capabilities.toml`, `tests/layering.rs`, `docs/` | Widersprüchliche Vorgaben für Tests |

---

## 17. Entfallene Spezifikationsteile (Übersicht)

Die folgenden Abschnitte der Fassung 3.1 sind entfallen. Ihr Wortlaut ist über den Commit `3c1f144fa` (Datei `docs/SPEC.md`) nachvollziehbar.

- §10 (Bausteine für lokale LLM-Anwendungen F3–F5, Inferenz, Privacy, MCP, Python, Sandbox)
- §11 (Kompositionsprofile für KMU, Baukasten-Profile)
- §9 (Löschkette) außer den Abschnitten §9.2 und §9.3 dieser Fassung
- Die Gate-Matrix aus §12 und §17.3 in ihrer Fassung 3.1 (ersetzt durch §12 dieser Fassung und D-R2)

---

## Anhang A. Änderungsverlauf

| Fassung | Datum | Änderung |
|---|---|---|
| 3.1 | 28.10.2026 | Baukasten-Fassung, Zielzustand v1.0 |
| 4.0 | 30.10.2026 | Reduktionsfassung. Scope auf Kern v1 reduziert, Ringmodell an Ist-Abhängigkeiten angepasst, Löschkette aus v1 genommen, Befunde B1–B11 und Entscheidungen D-R1–D-R9 aufgenommen. |

## Anhang B. Vorgehen bei Widersprüchen zum Code

Ist eine Aussage dieser Spezifikation nicht mit dem Code vereinbar, gilt: (1) Der Widerspruch wird als Befund in §16 aufgenommen. (2) Die Entscheidung trifft der Maintainer über einen ADR-D-Eintrag in §15. (3) Die Spezifikation wird entsprechend angepasst. Der Code wird nicht stillschweigend an eine veraltete Spezifikation angepasst und umgekehrt.
