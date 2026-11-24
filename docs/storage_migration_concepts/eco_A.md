# SPEC.md — Contextra-Ökosystem (Gesamtsystem) mit Speicherkern `contextra-storage`, Revision E1

| | |
|---|---|
| Status | **Entwurf zur Freigabe (Gate G-E0).** Eigenständige Spezifikation des Gesamtsystems um den neuen Speicherkern. |
| Gegenstand | Lokales, eingebettetes Wissens- und Gedächtnissystem für KI-Assistenten und Agenten: Aufnahme, hybride Suche, Gedächtnis, Agenten-Workflows, Modellanbindung, Datenschutz und **nachweisbare Löschung**, auf **einem** Speicherkern. |
| Verhältnis zu anderen Dokumenten | **Kern-Spec** (`SPEC.md` R9.1 + §3.7): normativ für Formate, Commit, Recovery, Löschmechanismus und Kern-Abhängigkeiten. **Diese Datei:** normativ für Crate-Topologie, Schnittstellenverträge zwischen den Crates (konkret gegen die Ist-Signaturen), Space-Katalog, Datenflüsse, Migration und Systemverifikation. **SPEC-R10** ist die Bestandsanalyse (Befunde `E10-nn`); bei Widerspruch gilt diese Datei (Anhang B). Rangfolge: Kern-Invarianten (§15 Kern-Spec) vor Invarianten dieser Datei (§16) vor übrigem Text. |
| Evidenzbasis | `docs/interface_specs/*.md` und `api/*.json` (erzeugt von `tools/extract_interfaces.py` aus dem Quellcode, Snapshot `f68e2c3`, 2026-11-24), `capabilities.toml` (Schema v3), Workspace-`Cargo.toml`, `deny.toml`, ausgewählte Quelldateien. Jede Zahl in §2, §3.1 und Anhang A ist aus diesen JSON-Dateien **maschinell abgeleitet** (`tools/eco_baseline.py`, §13.4). |
| Verifikation | **Nichts wurde gebaut oder getestet** (kein `cargo` verfügbar). Die Interface-Specs sind automatisch erzeugt und können hinter dem Quellcode zurückliegen („Maßgeblich bleibt der Quellcode“). Befunde mit *(zu verifizieren)* brauchen einen reproduzierenden Test. Leistungsangaben sind Entwurfsziele. |
| Rechtlicher Vorbehalt | Technische Eigenschaften, keine Rechtsaussage zu Art. 17 DSGVO. „Gelöscht“ ist immer an eine Stufe der Kern-Spec (§9.2) gebunden. |
| Sprache | Prosa deutsch, Bezeichner englisch. |

---

## 0. Kurzfassung

1. **Ein Speicherkern, ein Löschmechanismus.** Alle dauerhaften Daten liegen in Spaces des Kerns (§6) oder in Segmenten eines registrierten `SegmentOwner`. Kein Crate hat eigene Persistenz mit eigener Löschlogik.
2. **Anschluss über zwei vorhandene Nähte:** (a) `contextra_ports::StorageEngine` (26 Methoden) über den temporären Adapter `contextra-storage-compat`; (b) das assoziierte Trait-Bündel `contextra_db::Backends` (`Storage`, `Vector`, `Text`, `Graph`). Der neue Kern wird als zusätzliches Bündel `KernelBackends` eingehängt, nicht durch Umschreiben von `contextra-db`.
3. **Löschen ist ein Ticket mit Stufen und Beleg.** Heute liefert `AgentMemory::forget` `Result<(), ContextraError>` und `contextra_delete`/`contextra_forget` sind `DatabaseWrite`-Tools ohne Receipt. Ohne Receipt gibt es keinen Erfolg (ECO-INV-1).
4. **Der Kern bleibt synchron und `forbid(unsafe_code)`**; der Kern hängt (Ist) nur an `contextra-storage-ports`. Async, HTTP, FFI und Modelle leben an den Rändern. Unsafe-Inseln bleiben `contextra-sys`, `contextra-simd`, `contextra-wire`.
5. **Fehler werden auf `ErrorClass` abgebildet** (§5.3): 42 `ContextraError`-Varianten ohne Erase-/Restriction-Zustände auf 11 Kern-Klassen.
6. **Migration über Gates, Verbraucher für Verbraucher**, ohne Dual-Writes; R10 §8 Sofortmaßnahmen sind Vorbedingung (§14.1).

---

## 1. Ziele, Nicht-Ziele, Produktverträge

### 1.1 Ziele

| ID | Ziel | Prüfbar durch |
|---|---|---|
| Z-1 | Hybride Suche (Bedeutung, exakter Begriff, Zusammenhang) ohne Voll-Neuaufbau bei Änderungen | §13 Retrieval-Suite, Index-Watermarks |
| Z-2 | Langzeitgedächtnis mit Konsolidierung und Rückkehr zu früheren Ständen (Views, Branches) | §13, Kern-Spec §8 |
| Z-3 | Modelle austauschbar (Candle, ONNX, Ollama, Cloud) hinter `EmbeddingProvider`/`LlmTextGenerator` | §10 |
| Z-4 | Vertrauliche Daten verlassen das Haus nicht ungeschützt (`EgressGateway`, Surrogate, Prompt-Guard) | §8 |
| Z-5 | Löschung nachweisbar auf definierter Stufe, über alle abgeleiteten Daten | §7.3, Canary-Test |
| Z-6 | Crash-Konsistenz über alle Crates | §13, Kern-Spec §10 |
| Z-7 | Betrieb ohne Cloud-Abhängigkeit; Open-Core ohne Format-Bindung | §15.4 |

### 1.2 Nicht-Ziele

Verteilter Betrieb (Kern-Spec NZ-20), eigenes Modelltraining, automatische PII-Erkennung im Kern (der Kern klassifiziert nicht; die deklarierende Schicht ist verantwortlich), Rechtsberatung, Bezahlschranke für Basislöschung (O-3).

### 1.3 Produktverträge

| Vertrag | Zusicherung | Einschränkung |
|---|---|---|
| P-1 Suche | Keine Treffer aus gelöschten Clustern (Gate vor Auswertung, `filter_visible` nach Index) | Index darf zurückliegen (`IndexLag`), nie Sichtbarkeit herstellen |
| P-2 Gedächtnis | Synthesen tragen Lineage (`DerivedFrom`); Quelllöschung löscht oder markiert sie | Cross-Subject nur mit Link-Admission |
| P-3 Löschung | `erase(level, scope, deadline)` → Ticket → signierter Receipt mit `Residual` | `Logical` ist keine Vernichtung |
| P-4 Wiederherstellung | Restore mit Rollback-Schutz; Gelöschtes kehrt nicht zurück | Backup-Horizont/`KeyAssurance` begrenzen die Aussage |
| P-5 Egress | Cloud-Aufrufe sehen nur Surrogate/zulässige Felder; Injection-Treffer werden quarantänisiert | signatur-/phrasenbasiert, kein Vollschutz |
| P-6 Fehler | Jeder öffentliche Fehler trägt `ErrorClass` + `hint()`; keine Schlüssel/Körper/Querytexte in Meldungen und Logs | – |

---

## 2. Ist-Aufnahme aus `docs/interface_specs`

### 2.1 Tatsächliche Abhängigkeitstiefe

Tiefe = längste Kette interner `normal`-Abhängigkeiten (aus `deps[*].kind == "normal"`, nur Workspace-Crates). Sie widerspricht der heutigen `level`-Spalte in `capabilities.toml`:

{{TABLE_DEPTH}}

`capabilities.toml` führt `contextra-ports` mit Ebene 2, `contextra-mvcc` mit Ebene 1, `contextra-adapt` mit Ebene 0, `contextra-infer-onnx` mit Ebene 0 — obwohl `infer-onnx` von `contextra-ports` und `contextra-types` abhängt. Die `level`-Spalte ist also **nicht aus dem Graphen abgeleitet** und wird in Schema v4 durch eine berechnete Tiefe plus explizite `allowed_internal` ersetzt (§3.2).

### 2.2 Befunde (F-nn, neu gegenüber R10)

| ID | Sev. | Befund | Beleg (interface_specs) |
|---|---|---|---|
| F-01 | H | **`contextra-erasure` ist ein Orphan:** 0 interne Abhängigkeiten, `used_by` nur `contextra-erasure-fuzz`; 559 öffentliche Items, 39 Traits, 119 `unwrap`, 2 `panic!`. Es hat ein **eigenes Modell** (`DeletionScope`, `DeletionLayer`, `LayerWitnessMeasurement`, `DeletionProof`, `DeletionProofVerifier`, `CustodyLevel`), das nicht auf Kern-`EraseLevel`/`TicketStage`/`Residual` passt. | `api/contextra-erasure.json` |
| F-02 | H | **`contextra-db` koppelt über `Backends`** (`type Storage: StorageEngine; type Vector: VectorIndex; type Text: TextIndex; type Graph: GraphIndex`). `contextra-vector`, `-text`, `-graph`, `-checkpoint`, `-store` stehen im Manifest **nur als `dev`**, der Default-Bündel-Code verweist auf `contextra_store::LsmStorage` und `contextra_graph::CsrGraph`. Ein produktiver Composition Root ist im Manifest nicht erkennbar *(zu verifizieren: Build ohne dev-Abhängigkeiten, Wer instanziiert `Backends` in Produktion?)*. | `api/contextra-db.json`, `crates/contextra-db/src/backends.rs` |
| F-03 | M | **Unsafe-Insel `contextra-simd` ist nicht autark:** hängt an `contextra-types` (und damit `contextra-ids`), während `contextra-sys` ausdrücklich „0 interne Abhängigkeiten“ fordert. `contextra-vector` hängt zusätzlich an `contextra-sys` und `memmap2` (Feature `mmap`). | `api/contextra-simd.json`, `api/contextra-vector.json` |
| F-04 | H | **`DocId` ist featureabhängig breit:** `u64` standardmäßig, `u128` bei `docid-128` (gesetzt in `ids`, `types`, `checkpoint`, `mvcc`, `py`, …). Cargo-Feature-Vereinigung macht die Breite workspace-global und damit zu einem **Datenformat-Schalter**. | `api/contextra-ids.json` |
| F-05 | H | **Fehlerraum deckt Löschung nicht ab:** `ContextraError` hat 42 Varianten (u. a. `WalCorruption`, `Bincode`, `Cluster`, `CrossDeviceLink`), aber keine für `Erased`, `Restricted`, `UnknownCluster`, `ErasePending`, `Lagged`, `KeyLost`. Der Kern kennt 30 `Error`-Varianten in 11 `ErrorClass`. | `api/contextra-types.json`, `api/contextra-storage-ports.json` |
| F-06 | H | **MCP-Toolkatalog** (`TOOL_REGISTRY`, 10 Tools): `contextra_search/get/collections/explain` (`DatabaseRead`), `contextra_insert/upsert/forget/delete/create_collection/drop_collection` (`DatabaseWrite`). **Kein** `contextra_erase`. Die Crate-Doc nennt nur 6 davon (Doku-Drift). | `crates/contextra-mcp/src/sandbox.rs:53ff.` |
| F-07 | H | `AgentMemory::forget(&self, id: &MemoryId) -> Result<(), ContextraError>` „delegiert direkt an `Contextra::delete`“: Erfolg ohne Beleg. | `api/contextra.json` |
| F-08 | M | **Kern-Ports unvollständig gegenüber Kern-Spec §9.4:** `KeyProvider` hat 7 Methoden (`info`, `kek`, `create_cluster_key`, `cluster_key`, `probe`, `list`, `destroy_many`), keine Tomb-API; `KeyProviderInfo` hat nur `destroy`, `keys_in_backups` (kein `assurance`); `KeyState` hat keine Varianten im Extrakt; `Clock` hat `read`, `advance_utc_floor`. `Storage` hat nur `open` und `catalog`. | `api/contextra-storage-ports.json`, `api/contextra-storage.json` |
| F-09 | M | **Zwei Receipt-Begriffe:** `contextra_storage_ports::Receipt{stamp,durable}` (Commit) und das Lösch-Receipt der Kern-Spec §9.7; `contextra-erasure` hat zusätzlich `DeletionProof`. Drei Begriffe für nachweisbare Operationen. | Ports-/Erasure-JSON |
| F-10 | M | **Verwaiste Verbraucher:** `contextra-kvcache.used_by = []`, `contextra-privacy.used_by = ['contextra-mcp']`, `contextra-checkpoint` arbeitet auf eigener `FsSeam`/`PersistentCheckpointStore` (nur `env`, `ids` intern). | `used_by`-Felder |
| F-11 | M | **Altstand hält die Krypto:** `contextra-store` hängt an `contextra-crypto` (aes-gcm-siv, argon2, bincode, rand, lru); `contextra-mcp`, `-kvcache`, `-infer-candle` ebenfalls. | `api/contextra-crypto.json` |
| F-12 | M | **Panikfreiheit nicht erfüllt:** Produktions-`unwrap`: `graph` 575, `store` 513, `db` 433, `router` 183, `erasure` 119, `checkpoint` 83, `infer-ollama` 69, `types` 62, `vector` 52, `infer-candle` 43, `mcp` 36 (Zähler aus `counts`, grob; Kern-Spec verlangt keine `unwrap/expect/panic` im Kern, `contextra-mcp` deklariert „Zero-Panic“ und hat 9 `panic!`). | `counts` je Crate |
| F-13 | N | **Kern-Pfade nicht flach:** `path` der Storage-Crates ist `crates/contextra-storage/crates/*` (R10 E10-02); `capabilities.toml` kennt die Storage-Crates nicht. | `path`, `capabilities.toml` |

### 2.3 Positive Befunde (beibehalten)

`contextra-storage-ports`: 0 `unwrap/expect/panic`, 0 `unsafe`, Parser-Abdeckung 100 %, 19 Traits mit klaren Rollen. `contextra-storage`: 0 `unwrap/expect/panic`, nur `contextra-storage-ports` intern. `contextra-rank`, `contextra-kvcache`, `contextra-mvcc`, `contextra-infer-onnx`, `contextra-simd`, `contextra-sandbox`: 0 `unwrap`. `contextra-ports` besitzt die Verträge, an denen die Konsumenten bereits hängen (`StorageEngine`, `VectorIndex`, `TextIndex`, `GraphIndex`, `Reranker`, `EmbeddingProvider`, `LlmTextGenerator`, `EgressPolicy`, `ReportSigner`, `Checkpoint`, `CheckpointCoordinator`, `ReadGate`, `Purger`, `LedgerSink`): deshalb werden sie **gemappt**, nicht ersetzt (§5).

