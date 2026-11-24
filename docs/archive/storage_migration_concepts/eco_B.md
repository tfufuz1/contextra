---

## 3. Zielarchitektur

### 3.1 Gruppen und Soll-Abhängigkeiten

Gruppen (aus `capabilities.toml`, erweitert): `foundation`, `contracts`, `storage` (**neu**), `compliance`, `retrieval`, `llm`, `data`, `safety`, `agentic`, `integration`, `apps`, `testing`. Ebenen werden nicht mehr gepflegt, sondern **aus `allowed_internal` berechnet** (Tiefe). Die Tabelle stellt für jedes Crate die Ist-Kanten (`normal`, aus den JSON-Dateien) den Soll-Kanten gegenüber; Soll ist die neue `allowed_internal` (Feature-Kanten in Klammern).

{{TABLE_SOLL}}

Lesehilfe: **Entfällt** = Kante wird entfernt; **Neu** = Kante kommt hinzu. Zyklen sind im Soll ausgeschlossen (geprüft in `tools/eco_baseline.py --check-dag`).

### 3.2 Regeln (erzwungen durch `tests/layering.rs`, `tests/unsafe_islands.rs`, `capabilities.toml` v4)

| ID | Regel |
|---|---|
| A-1 | `allowed_internal` ist die einzige erlaubte Menge interner `normal`-Abhängigkeiten; `layering.rs` vergleicht sie mit `cargo metadata` (normal, nicht dev). |
| A-2 | Der Kern `contextra-storage` hängt intern **nur** an `contextra-storage-ports` (Ist-Zustand, wird zur Regel; strenger als R10 A-1). |
| A-3 | Oberhalb des Kerns spricht jedes Crate Storage nur über `contextra-storage-api`; Domänenports (`contextra-ports`) bleiben die Verträge zwischen Wissens-Crates. |
| A-4 | `contextra-erasure` und `-erase-verify` hängen an `contextra-storage-api` (+ `contextra-ports` für `ReportSigner`), nie am Kern-Crate. |
| A-5 | Altstand (`store`, `mvcc`, `durable-fs`, `vault`, `crypto`) ist nur für `storage-compat`, Migrationswerkzeug und `contextra-db` (Übergang) erlaubt; Entfernung nach G-E6. |
| A-6 | Unsafe-Inseln: `contextra-sys`, `contextra-simd`, `contextra-wire`; alle drei **ohne interne Abhängigkeiten** (behebt F-03: `simd` löst `contextra-types`). Alle anderen Crates `#![forbid(unsafe_code)]`; `unsafe_code = "deny"` im Workspace bleibt. |
| A-7 | Kein Produktiv-Crate hat eine `dev`-only-Kante, die ein Produktiv-Bündel braucht (F-02): Der Composition Root (§3.3) hängt **normal** an den Backends. |
| A-8 | `tokio`/`reqwest` nur in `contextra`, `contextra-mcp`, `contextra-py`, `contextra-infer-*`, `contextra-agent`, `contextra-bench`; `contextra-storage-api` und tiefer sind synchron. Die `TRANSITION E29`-Einträge in `deny.toml` (bis 2027-04-30) laufen aus, nicht verlängern. |
| A-9 | Storage-nahe Ports in `contextra-ports` (`vfs`, `keys`, `erasure`, `ledger`, `gate`, `barrier`, `scanner`) werden `#[deprecated]`; Mapping in §5.5. |
| A-10 | Jedes neue Crate: Eintrag in `capabilities.toml` vor dem ersten Commit. |

### 3.3 Composition Root

Heute: `contextra-db` ist generisch über `Backends`; `contextra::AgentMemory::new(...)` und `Contextra::open(path)` / `open_with_config` / `open_authorized` konstruieren. Ziel:

1. `contextra-db` definiert weiterhin `trait Backends`. Es kommt **ein zweites Bündel** hinzu: `KernelBackends { Storage = StorageApiEngine, Vector = KernelVector, Text = KernelText, Graph = KernelGraph }`, wobei `StorageApiEngine` die Implementierung von `StorageEngine` aus `contextra-storage-compat` ist.
2. Das Bündel wird **im Crate `contextra`** (Fassade, Feature `storage-kernel`) zusammengesetzt, weil dort Produktivabhängigkeiten auf `contextra-storage-api`, `-vector`, `-text`, `-graph` erlaubt sind. `contextra-db` selbst bekommt keine normalen Kanten auf Index-Crates (bleibt entkoppelt, wie `Backends` es vorsieht).
3. Features: `storage-kernel` (neu, Default **aus** bis G-E5), Default danach **an**; `legacy-store` (Altstand, bis G-E6). Beide gleichzeitig aktiv ⇒ Compile-Fehler (keine Dual-Writes).
4. `open_authorized` (heute) wird die einzige Stelle, die `AdminCap` erzeugen darf (E10-14): der Token wird dort aus der Embedding-Autorisierung abgeleitet und nicht klonbar weitergereicht.

---

## 4. Crate-Katalog (final)

Entscheidung: **BEHALTEN**, **UMBAU**, **ERSETZEN**, **NEU**, **ENTFERNEN** (nach Cutover). Baseline (unwrap/panic) steht in Anhang A.

### 4.1 Grundlagen und Verträge

| Crate | Entscheidung | Maßnahmen (konkret) |
|---|---|---|
| `contextra-ids` | UMBAU | `DocId`-Breite als Datenformat festschreiben: persistierte Keys codieren **immer 16 Bytes BE** (u64 wird nullerweitert); Feature `docid-128` nur noch für die In-Memory-Typbreite, nie für Formate (F-04). `TenantId` bleibt `u64` und wird nie als Kern-`ClusterId` benutzt. |
| `contextra-types` | UMBAU | `ContextraError` um Lösch-/Kernzustände erweitern (§5.3), `fn class() -> ErrorClass` ergänzen; `tombstone.rs` mit `TombId` abgleichen; `bincode`-Nutzung (Abhängigkeit) schrittweise entfernen. |
| `contextra-env` | UMBAU | `Clock::now_unix_nanos` fallibel (E10-74); `ManualClock`/`SeededRng` bleiben für Tests; Löschfristen nutzen den Storage-`Clock` (`read()`, `advance_utc_floor`). |
| `contextra-ports` | UMBAU | Domänenports bleiben (`EmbeddingProvider`, `TextGenerator`, `LlmTextGenerator`, `LlmTextGeneratorStreaming`, `Reranker`, `QueryRewriter`, `VectorIndex`, `TextIndex`, `GraphIndex`, `HybridSearchProvider`, `StorageEngine`, `Checkpoint`, `CheckpointCoordinator`, `EgressPolicy`, `ReportSigner`, `ContextPreparer`, `GroundingValidator`, `DriftStatusProvider`, `PluginManifest`, `LicenseGate`); Deprecation der Storage-nahen Teile nach §5.5. |
| `contextra-sys` | BEHALTEN | Windows-Pfad (`windows-sys`), `HostProbe`-Adapter. |
| `contextra-simd` | UMBAU | `contextra-types`-Abhängigkeit entfernen (autark, F-03). |
| `contextra-wire` | BEHALTEN | Receipt-/Ticket-Frames ergänzen; **signiert werden nur die kanonischen Receipt-Bytes**, nicht FlatBuffers-Nachrichten. |

### 4.2 Speicher

| Crate | Entscheidung | Maßnahmen |
|---|---|---|
| `contextra-storage-ports` | UMBAU | `KeyProvider` um Tomb-API und `KeysInBackups::{No,Bounded,Yes}` ergänzen; `KeyProviderInfo` um `assurance` (F-08); `Error` um `SnapshotTooOld`, `ErasePending`, `LedgerUnavailable`, `LedgerSameDevice`, `BackupBeyondHorizon`, `ProviderAssuranceInsufficient`, `Wiped` ergänzen; Clock fail-closed (`ClockReading.skew`). |
| `contextra-storage` | UMBAU | Kern gemäß Kern-Spec; `Storage` erhält `apply`, `view`, `page`/`cursor`, `feed` (heute nur `open`, `catalog`; Module `engine::{cursor,feed,page,idem,registry,…}` existieren). |
| `contextra-storage-api` | **NEU** | Fassade und Composition Root des Speicherbereichs (§5.1). |
| `contextra-storage-fs` | UMBAU | `RealFs` (`std` + `rustix`), `FaultFs`, `MemFs` (Kern-Spec §3.7.3). |
| `contextra-crypto-ref` | BEHALTEN (Rolle erweitert) | Standard-Adapter (Kern-Spec SD-71). |
| `contextra-keys-file` / `-cohort` | UMBAU | auf neuen `KeyProvider`, Argon2id Pflicht. |
| `contextra-keys-kms` | NEU (optional) | KMS/HSM-Adapter. |
| `contextra-storage-compat` | **NEU (temporär)** | `StorageEngine`-Impl (§5.2); `contextra-storage-bridge` geht auf. |
| `contextra-store` | BEHALTEN → ENTFERNEN | Altstand; keine neuen Features. |
| `contextra-mvcc` | UMBAU → ENTFERNEN | `TxBuffer`/`SequenceLog` nur noch Übergang; `contextra-vector` löst die Kante (§4.4). |
| `contextra-durable-fs` | ENTFERNEN nach G-E2 | keine produktiven Verbraucher im Ist-Graphen (`used_by` prüfen). |
| `contextra-vault` (excluded) | UMBAU → ENTFERNEN | RAM-Staging → `Volatile`-Space (`agent_scratch`, §6). |
| `contextra-crypto` | UMBAU (Legacy) | nur Migrationslesepfad; `emergency_wipe` fail-closed, `seal_record_with_nonce` nicht öffentlich, Argon2id Pflicht, Zufallsnonce (E10-20…23). |

### 4.3 Löschschicht

| Crate | Entscheidung | Maßnahmen |
|---|---|---|
| `contextra-erasure` | **ERSETZEN (Orchestrator)** | Neues Modell nach §5.5; aus 559 öffentlichen Items bleiben nur Saga, Bericht, Signatur (`ReportSigner`, `DeletionProofVerifier` → `verify_receipt`), Risiko, Retention, Restriction-Workflow. Entfernen: `erased_set`, `crypto_local`, `ledger/*`, `journal/*`, `vault/*`, `kv_shredding`, `SecretKey/Dek/UnitKey/AncestorSalts` (Schlüssel gehören hinter `KeyProvider`), `GLOBAL_GATE_SET`, doppelte Typen. Tier `frozen` → `beta` erst mit G-E3. |
| `contextra-erase-verify` | **NEU** | aus `measure*`/`LayerWitnessMeasurement`; `MeasureKind`, Orakel über Ports. |

### 4.4 Wissen

| Crate | Entscheidung | Maßnahmen (konkret) |
|---|---|---|
| `contextra-db` | UMBAU | `Backends`-Bündel `KernelBackends` (§3.3); öffentliche Typen frei von Store-Typen; `Collection::delete`/`delete_op` liefern Ticket/Receipt-Info; `insert_with_ttl`/`reap_expired_documents`/`trigger_expiry_cleanup` laufen über `pack_expired` (§7.4); `vault_sys.inc`/`volatile_vault.rs` → `Volatile`-Space; `snapshot_seq`/`get_at_snapshot` → `View` mit Lease. Normale Kante auf `contextra-mvcc`/`-vault` entfällt. |
| `contextra-vector` | UMBAU | `contextra-mvcc` und `contextra-sys` (Feature `mmap`) entfallen; `pending.wal`/`tombstone.wal` → Owner-Segmente (`vec_idx`, `Rebuildable`); `mmap` nur für Nicht-Personal-Rebuildables, sonst `read_at` über Storage-Api; Gate-Filter statt Tombstone-Sidecar; `VectorIndex::is_rebuild_required`/`trigger_rebuild_async` bleiben. |
| `contextra-text` | UMBAU | `TextIndex` bleibt; Persistenz als Owner `text_idx`; `derive_doc_id` mit keyed MAC (INV-PRIV-1); Purge über `rewrite_without`. |
| `contextra-graph` | UMBAU | `GraphIndex`-Persistenz als `graph_ent` (`ClusterSections`) und `graph_csr` (`Rebuildable`); Startup-Vollscan → `ConsumerFeed` mit Watermark; Doc-Purge durabel (E10-55); `scc`/`arc-swap`/`bincode` prüfen (Kern-Spec §3.7.6). |
| `contextra-rank` | BEHALTEN + Gate-Stelle | Eingänge bereits `filter_visible`-gefiltert (dokumentieren); `DocId` nur über fixe Breite. |
| `contextra-checkpoint` | UMBAU | Eigene `FsSeam`/`PersistentCheckpointStore`/`CheckpointHardlinkCloner` (83 `unwrap`) entfallen; `Checkpoint`/`CheckpointCoordinator`-Ports werden von `storage-compat` über den Kern-Checkpoint bedient; `orphan.rs` entfällt (Kern kennt Graveyard/Pins). |
| `contextra-kvcache` | UMBAU | v1 **nur RAM** (O-4); `used_by = []` ⇒ Aufnahme in den Produktivpfad erst mit Verbraucher; Persistenz-Tier später als Owner. |
| `contextra-privacy` | BEHALTEN + Anbindung | `SurrogateVault`/`EgressVault` → Space `privacy_vault` (§6), Surrogat 128 Bit (E10-40); Verbraucher bisher nur `contextra-mcp`. |
| `contextra-prompt-guard` | BEHALTEN | – |
| `contextra-infer-{onnx,ollama,candle}` | BEHALTEN | keine Persistenz; `contextra-crypto`-Kante von `infer-candle` über `kv-bridge` → nur über Features. |

### 4.5 Anwendung, Fassade, Ränder

| Crate | Entscheidung | Maßnahmen |
|---|---|---|
| `contextra-cognition` | UMBAU | Synthesen als `DerivedFrom`-Cluster (`memory_derived`); `async-trait`, `blake3` prüfen. |
| `contextra-router` | UMBAU (klein) | 183 `unwrap`; Kalibrierung optional als `router_cal`. |
| `contextra-agent` | UMBAU | Normale Kanten auf `db`, `checkpoint`, `router`; Checkpoints im Cluster der Person (`agent_ckpt`); Volatile Scratch. |
| `contextra-sandbox`, `-adapt`, `-ingest` | BEHALTEN | `ingest`: Klassifikation `Personal` als Parser-Ausgabe. |
| `contextra` | UMBAU | Composition Root (§3.3); `AgentMemory` (§9.2). |
| `contextra-engine` | BEHALTEN → ENTFERNEN | Re-Export, hat 11 interne `allowed_internal`-Kanten (Ist) → nur `db`. |
| `contextra-mcp` | UMBAU | Toolkatalog nach §9.1. |
| `contextra-py` | UMBAU | Bindings §9.3. |
| `contextra-testkit` | UMBAU | `FaultVfs`/`ManualClock`/`InMemoryStorageEngine` → `storage-fs::FaultFs`, `env`-Sim; `erasure-conformance`-Feature auf Kern-Orakel. |
| `contextra-bench` | UMBAU | Kante auf `contextra-store` → `storage-api`. |

---

## 5. Schnittstellenverträge

### 5.1 `contextra-storage-api` (Fassade)

Die Fassade re-exportiert aus `contextra-storage-ports` die Ist-Typen und ergänzt, was der Kern heute nicht hat:

| Typ / Operation | Quelle | Stand (Ist) |
|---|---|---|
| `Change { ops, guards, idem, durability, allow_mixed, caller }` | ports `change.rs` | vorhanden |
| `Op::{Put, Delete, RangeDelete}`, `Guard::{Key, Range}`, `Precondition::{Absent, Version, Unchanged}` | ports | vorhanden |
| `Receipt { stamp, durable }`, `Durability::{Durable, Relaxed}`, `Origin::{User, Maintenance, Replication, Migration}`, `CallerMeta { origin, caller_seq }` | ports | vorhanden |
| `ClusterId`, `SpaceId`, `Stamp`, `BranchId`, `IdemToken`, `EraseRequestId`, `Ticket`, `TombId`, `InstanceId`, `DurablePosition`, `DataClass::{NonPersonal, Personal}`, `Protection::{Sealed, Plain}`, `ClusterLifecycle::{Live, Retired, Shredded, Purged}` | ports `ids.rs`/`change.rs` | vorhanden |
| `SpaceOptions`, `SpaceSpec`, `SealPolicy`, `HistoryPolicy`, `PackPolicy`, `SpaceClass` | kern `spaces` | vorhanden (nicht in Ports: Fassade re-exportiert) |
| `EraseLevel`, `EraseScope`, `EraseTicket`, `TicketStage`, `TicketStatus`, `Residual`, `LiftPolicy`, `RestrictionSpec` | kern `erase::*` | vorhanden, aber **im Kern-Crate**: für Verbraucher in die Fassade (nicht in Ports) heben |
| `Storage::apply(Change) -> Result<Receipt>` | Kern-Spec §6 | **fehlt** (nur `open`, `catalog`) |
| `Storage::view(ViewAt) -> View`, `View::{get, page, filter_visible}`, `Cursor`, `Page` | Kern-Spec §7 | **fehlt** (`engine::views::VersionView`, `engine::cursor/page` Module vorhanden, Inhalt prüfen) |
| `Storage::feed(consumer) -> Feed` (`ConsumerFeed::poll`) | ports `traits::engine` | Trait vorhanden, Impl prüfen |
| `Erasure::{request, status, receipt, restrict, lift}` | Kern-Spec §9 | **fehlt** (Protokoll ER0–ER7 nicht umgesetzt, E10-05) |
| `ErrorClass { Transient, DiskFull, Poisoned, Indeterminate, Corrupt, Unsupported, Key, Usage, Conflict, Rollback, Internal }` | ports `error.rs` | vorhanden; `hint()` prüfen |

Vertrag: Verbraucher importieren **nur** aus `contextra-storage-api`; Vollscans nur über `Cursor`/`Page`; jede Index-/Extension-Ausgabe läuft durch `View::filter_visible` (INV-VIS-1).

### 5.2 Abbildung `StorageEngine` → Kern (`contextra-storage-compat`)

`contextra_ports::StorageEngine` hat 26 Methoden und ist das Rückgrat von `contextra-db`. Der Adapter bildet sie wie folgt ab; Zeilen mit **⚠** brauchen eine Semantikklärung vor G-E4.

| `StorageEngine`-Methode | Kern-Abbildung |
|---|---|
| `get`, `get_at_seq` | `View::get` auf `ViewAt::Durable` bzw. `At(Stamp)`; `seq` ≙ `Stamp` |
| `get_tracked`, `get_at_seq_tracked`, `scan_prefix_tracked`, `supports_ssi_tracking` | Lesemenge wird als `Guard::Key`/`Guard::Range` mit `Precondition::Version`/`Unchanged` in den späteren `Change` übernommen (Stufe-A-Guards, Kern-Spec §6.3); `supports_ssi_tracking = true` |
| `put`, `put_batch`, `delete`, `delete_many` | im Tx-Puffer des Adapters gesammelt → ein `Change` mit `Op::Put`/`Op::Delete` |
| `delete_prefix` | `Op::RangeDelete` |
| `put_if_absent` | `Op::Put` + `Guard::Key{Precondition::Absent}` |
| `commit` | `Storage::apply(Change)`; liefert `Receipt{stamp}`; `Indeterminate` wird **nicht** blind wiederholt, sondern über `IdemToken` aufgelöst |
| `rollback` | Tx-Puffer verwerfen (kein Kern-Aufruf) |
| `rollback_to_tx` | **⚠** Savepoint im Tx-Puffer (Kürzen auf Marke); Semantik gegen `contextra-store` und `contextra-mvcc::TxBuffer` testen |
| `flush` | wartet auf Syncer-Gruppenquittung (`Durability::Durable`); bei `Relaxed` Barriere |
| `stats`, `last_seq_no` | Kern-Metriken; `Stamp` des letzten durablen Commits |
| `last_tx_id` | **⚠** Tx-IDs sind Adapter-Zähler (`TxId` in `contextra-ids`), nicht Kern-Stamps; Zuordnung `TxId → Stamp` im Adapter, nach Neustart aus `CallerMeta.caller_seq` rekonstruierbar |
| `pin_checkpoint`, `unpin_checkpoint` | View-Lease (`SnapshotTooOld` bei Ablauf); Kern-Checkpoint nur über `Checkpoint`-Port |
| `scan_prefix`, `scan_prefix_bounded`, `scan_prefix_at`, `scan`, `scan_bounded` | `View::page`/`Cursor` mit Budget; **kein** `Vec` für unbeschränkte Scans (R10 §5.2) |

Konformanz: `contextra-ports` liefert `verify_storage_engine_contract` und `assert_storage_engine_contract`; beide laufen unverändert gegen `contextra-storage-compat` über Altstand **und** Kern (Gate G-E1/G-E4).

### 5.3 Fehlerabbildung `ContextraError` → `ErrorClass`

`ContextraError` bleibt der Domänenfehler (42 Varianten). Es kommt eine Variante `Storage(StorageFailure{class, hint})` hinzu sowie typisierte Lösch-/Kernzustände; jede Variante erhält `fn class(&self) -> ErrorClass`. Abbildung der **vorhandenen** Varianten:

| ErrorClass | `ContextraError`-Varianten |
|---|---|
| `Transient` | `Timeout`, `TransactionTimeout`, `CommitTimeout`, `Io` (EINTR-artig) |
| `DiskFull` | `Io` (ENOSPC), `MemoryBudgetExceeded` nur wenn Plattenreserve gemeint ist (sonst `Usage`) |
| `Poisoned`/`Indeterminate` | `Storage`, `Io` (EIO/Fsync), `Transaction` bei begonnenem Sync |
| `Corrupt` | `WalCorruption`, `ChecksumMismatch`, `WalTruncationDetected`, `OrphanedVectorReference`, `HnswConnectivityDegraded`, `GraphRepairFailed`, `Index`, `Text` (bei Lesefehler persistierter Daten); Index-Varianten lösen Rebuild aus, nie Datenverlust-Alarm |
| `Unsupported` | `CapabilityUnsupported`, `SnapshotUnsupportedForSignal`, `Plugin` |
| `Key` | `Crypto` |
| `Usage` | `InvalidInput`, `InvalidSequenceNumber`, `PolicyViolation`, `LimitExceeded`, `PinBudgetExceeded`, `EmbeddingDimensionMismatch`, `ParseError`, `Json`, `Serialization`, `Bincode` (an der API-Grenze), `KvDeleteModeConfig`, `KvQuantization`, `DurabilityConfig`, `CheckpointNotFound`, `NotFound` |
| `Conflict` | `Conflict` (`is_occ_conflict`), `StaleRead` |
| `Rollback` | – (neu: Anker höher als Catalog/Ledger) |
| `Internal` | `Internal`, `Cluster`, `CrossDeviceLink` (Link-Pfad entfällt mit `hardlink_cloner`) |
| Domäne (unverändert, keine Kernklasse) | `Sandbox`, `SandboxTimeout`, `MemoryLimitExceeded`, `ModelLoad` |

**Neu aufzunehmen** (heute ohne Variante): `Erased`, `Restricted`, `UnknownCluster`, `ErasePending`, `Lagged`, `KeyLost`, `KeyUnavailable{retry_after}`, `SnapshotTooOld`, `BackupBeyondHorizon`, `ProviderAssuranceInsufficient`, `Wiped`. Verbraucher matchen auf `class()`/Variante, nie auf Text (R10 §5.7). Die Kern-`Error` (30 Varianten) hat `Erased`, `Restricted`, `UnknownCluster`, `KeyLost`, `Lagged`, `Shredded`, `ClusterRetired` bereits; `Shredded` ist **nie** ein Lesefehler (Kern-Spec §3.5 F-6) und wird im Adapter auf `Erased` abgebildet.

### 5.4 Identitäten und Schlüssel

| Identität | Typ (Ist) | Regel |
|---|---|---|
| `TenantId` | `u64` (`contextra-ids`) | Domänen-Mandant. Nie als `ClusterId` verwendet. |
| `ClusterId` | zufällig (`contextra-storage-ports`, `Redacted<…>`) | vom Kern vergeben, nie aufrufervergeben, Debug redigiert. |
| Mapping Person/Mandant → Cluster | – | ausschließlich `IdentityDirectoryOwner` (Kern-Spec §12.7): Key `HMAC(K_dir_tenant, canonical_person_id)`, `EntryDEK` je Person, Reverse-Lookup über `K_reverse`. |
| `CollectionId`, `DocId`, `EntityId`, `TxId` | `u64`; `DocId` optional `u128` | **Format:** `DocId` immer 16 Byte BE im Key (F-04). Schlüssel enthalten nie Klartext-Personenmerkmale, nie Querytext. |
| `Stamp` | Kern | `Seq` (`contextra-ids`) ≙ `Stamp` nur im Adapter; Uhr nie für Stamps. |

Key-Layout je Space legt der Eigentümer-Crate fest und friert es mit seinem Formatversionsfeld ein; die Länge ist durch `DEFAULT_MAX_KEY_BYTES`/`MAX_KEY_BYTES` des Kerns begrenzt.

### 5.5 Löschvertrag: altes Modell → neues Modell

**Zustandsmodell des Kerns (Ist):** `TicketStage`: `IntentRecorded → Gated → KeyDestroying → KeysDestroyed → Purging → Purged → Done`; `TicketStatus`: `Pending`, `Blocked`, `Incomplete`, `Complete`, `CompleteWithResidual`, `Failed`; `Residual`: `LogicalOnly`, `BoundedBackupKeys`, `WeakKeyAssurance`, `DeclaredHostOrDeviceExposure`; Ledger-Ereignisse: `EraseIntent`, `UnlinkProgress`, `BranchEraseDone`, `ReceiptIssued`. Kern-Spec §9.2 nennt die Stufen; `EraseLevel`-Varianten stehen im Kern-Crate (im Extrakt ohne Variantenliste, gegen Quelle prüfen).

**Abbildung der Ist-Typen:**

| Ist (`contextra-erasure` / `contextra-ports`) | Neu |
|---|---|
| `DeletionScope`, `ExcludedScope`, `SubjectRef`, `CollectionRef`, `TenantRef`, `DocKey` | `EraseScope` (Cluster-/Branch-/Fristbezug); Auflösung `SubjectRef → ClusterId` nur über `IdentityDirectoryOwner` |
| `DeletionLayer`, `UncoveredLayer`, `LayerWitnessMeasurement`, `LayerCleanupProof` | `SegmentOwner` + `EraseAttestation`; Messung in `contextra-erase-verify` (`MeasureKind`) |
| `DeletionProof`, `DeletionProofVerifier`, `VerificationResult`, `SignatureVersion` | `ErasureReceipt` (kanonische Bytes, Kern-Spec §9.7) + `verify_receipt`; Signatur über Port `ReportSigner::{signer_id, sign_report}` (bleibt in `contextra-ports`), Ed25519 `verify_strict` |
| `CustodyLevel`, `SigningKeyHandle`, `DeletionProofKeyPair` | `KeyAssurance`/`KeysInBackups` (Provider); Signaturschlüssel hinter `ReportSigner`, nie im Orchestrator-Crate |
| `SecretKey`, `Dek`, `UnitKey`, `AncestorSalts`, `HkdfError` | entfallen; Schlüssel nur `KeyProvider`/`Kdf` |
| Port `ErasureLayer { id, resolve, on_gate, purge, witness, key_coverage }` | `SegmentOwner { kind, manifest, on_gate, rewrite_without, verify_absent, watermark }`: `id→kind`, `purge→rewrite_without`, `witness→verify_absent`, `key_coverage→manifest.key scope`, `resolve→Orchestrator` |
| Port `ReadGate { check, check_bound, covers, gated_seq, gate_epoch, … }` | Kern-`GateIndex` + `View` (kein Port mehr nötig) |
| Ports `Purger`, `PurgeBarrier`, `CommitBarrier`, `LayerScanner` | Kern-Purge (Kern-Spec §9.5.1), Barrieren kerninterne |
| Port `LedgerSink { append, read_all, prune }` | Kern-`LedgerWriter` (genau ein Schreiber) |
| Ports `ErasureObserver::on_erased` | `ConsumerFeed` (`Erased`-Platzhalter) + `Observer::event(ObserverEvent)` |
| Ports `KeyMaterialSource`, `RecordSealer`, `FileKeyring` | `Entropy`, `Cipher`, `KeyProvider` |
| `StorageBackend`, `ErasureTarget` (in `contextra-erasure` und `-ports`) | entfallen |

**Ablauf über Crates** (R10 §5.3, konkretisiert): `contextra-mcp`/`contextra` → `contextra-erasure::request` → prüft `Restriction` über Fassade → `EraseIntent` ins Ledger (`IntentRecorded`) → Kern-Gate (`Gated`; ab hier `Erased` in `View`) → `KeyProvider.destroy_many` (`KeyDestroying`/`KeysDestroyed`, `Absent` abwarten) → Kern-Purge (`Purging`/`Purged`) → alle Owner `verify_absent` → `erase-verify` misst (`MeasureKind`) → Kern liefert kanonische Bytes, `ReportSigner` signiert → `ReceiptIssued`, `Done`.

**Verbot:** Kein Verbraucher löscht über `Collection::delete`, `StorageEngine::delete*` oder Dateioperationen mit Erfolgsrückgabe, sobald `storage-kernel` aktiv ist (Lint `no-direct-delete`, Testfall §13.2 T-3).

### 5.6 Zeit

Löschfristen, Hold-TTL, Idempotenz-Ablauf: ausschließlich Storage-`Clock` (`ClockReading{monotonic_nanos, utc_millis, durable_utc_millis, skew}`; bei `ClockSkew::Uncertain` fail-closed). `contextra-env::Clock` nur für Anzeige/Metriken. `Collection::insert_with_ttl`, `reap_expired_documents`, `trigger_expiry_cleanup`, `update_document_importance`/Decay (`decay_controller.rs`) laufen auf dem Storage-Clock, wenn sie Löschung auslösen.

### 5.7 Krypto-Grenzen

Kern und Erasure nutzen nur `Cipher`, `Mac`, `Kdf`, `Checksum`, `Entropy`, `KeyProvider`. `contextra-crypto` bleibt Legacy (§4.2). `contextra-privacy` nutzt einen **eigenen** Clusterschlüssel der betroffenen Person (O-6).

### 5.8 Dauerhaftigkeitsmodi der Fassade

`contextra::DurabilityMode { Full, WalNoHmac, MemoryOnly }` wird abgebildet:

| `DurabilityMode` | Kern |
|---|---|
| `Full` | `Durability::Durable`, `IntegrityMode::Encrypted` (Personal) bzw. `Authenticated` |
| `WalNoHmac` | `Authenticated` nur für `NonPersonal`; für `Personal` **Fehler** `DurabilityConfig` (Plain nie bei Personal, INV-SEAL-1) |
| `MemoryOnly` | Volatile-Spaces; keine Receipts mit `durable=true` |

---

## 6. Space- und Segmentkatalog

Jeder Space wird mit `SpaceOptions` deklariert (Kern-Spec §4.5); `DataClass` ist Pflicht und wird an externen Grenzen nie still defaultet. Spalte „Owner“: wer `SegmentOwner` ist bzw. wer schreibt.

| Space / Segment | Eigentümer-Crate | Klasse (`SegmentClass`) | `DataClass` | Seal | Hinweise |
|---|---|---|---|---|---|
| `docs` | `contextra-db` | `ClusterSections` | Personal (wenn Collection personenbezogen) | `Cluster` | Chunks/Körper; `HistoryPolicy::KeepUntilFloor` |
| `docs_meta` | `contextra-db` | `ClusterSections` | wie `docs` | `Cluster` | TTL, Importance, Links (`link_memories`) |
| `kv_app` | `contextra-db` | `ClusterSections` | **vom Aufrufer** (`put_kv`) | `Cluster`/`ByMode` | kein Default |
| `identity_dir` | Identity-Owner (in `contextra-erasure`/Storage-Api) | `MixedPerSubjectKey` | Personal | EntryDEK | Kern-Spec §12.7 |
| `vec_idx` | `contextra-vector` | `Rebuildable` | folgt Quelle | `Cluster` | ersetzt `pending.wal`, `tombstone.wal`; Lineage Pflicht |
| `text_idx` | `contextra-text` | `Rebuildable` | folgt Quelle | `Cluster` | Postings; `derive_doc_id` keyed |
| `graph_ent` | `contextra-graph` | `ClusterSections` | Personal | `Cluster` | Entitäten/Kanten, `DerivedFrom` |
| `graph_csr` | `contextra-graph` | `Rebuildable` | folgt Quelle | `Cluster` | CSR/Communities |
| `memory` | `contextra-cognition` | `ClusterSections` | Personal | `Cluster` | Episoden |
| `memory_derived` | `contextra-cognition` | `ClusterSections` | Personal | `Cluster` | Synthesen, `DerivedFrom` + Link |
| `agent_ckpt` | `contextra-agent` | `ClusterSections` | Personal | `Cluster` | im Cluster der Person |
| `agent_scratch` | `contextra-agent` | – (Volatile) | Personal | – | nur RAM, kein Receipt |
| `privacy_vault` | `contextra-privacy` | `ClusterSections` | Personal | `Cluster` | Surrogat-Zuordnung |
| `router_cal` | `contextra-router` | `NoPersonalData` | NonPersonal | `ByMode` | braucht Provenienz-Evidence |
| `kvcache_t2` | `contextra-kvcache` | – | – | – | v1 nicht persistent (O-4) |
| `sys_*` | Kern | intern | NonPersonal | – | Catalog, Ledger, Registry |

Regel: Neuer Space ⇒ Eintrag hier, `capabilities.toml` und Löschtest (T-3). Ein Owner ohne `rewrite_without` darf in `clustered`-Spaces keinen Block anlegen (Kern-Spec §12.2).

---

## 7. Datenflüsse

### 7.1 Schreiben (Aufnahme)

1. `contextra-ingest` extrahiert (Format-Features `fmt-*`) und chunkt, setzt Klassifikation (`Personal`).
2. `contextra-db::Collection::insert`/`insert_many`/`upsert` löst Mandant → Cluster über `identity_dir` auf, erzeugt Embeddings über `EmbeddingProvider::embed_batch` (außerhalb jeder Kernsperre).
3. Ein `Change` schreibt `docs` und `docs_meta` atomar (Mehr-Space), mit `IdemToken` (Idempotenz statt blindem Retry bei `Indeterminate`).
4. Indexe: Übergang (bis G-E5) **write-through** wie heute (`VectorIndex::insert`, `TextIndex::insert`, `GraphIndex::add_entity/add_edge`, `commit`); Ziel: Indexe konsumieren `ConsumerFeed::poll` mit Watermark (`IndexLag`), `Lagged` erzwingt Rebuild.
5. `Receipt{stamp,durable}` geht zurück; `Collection::insert` meldet Erfolg erst nach `durable` (bei `Durability::Durable`).

### 7.2 Suchen

`Contextra::hybrid_search*` → `VectorIndex::search_filtered`, `TextIndex::search`, `GraphIndex::traverse`/`personalized_page_rank` auf **einem** `View` (gleicher Stamp für alle Signale) → `contextra-rank` Fusion → `View::filter_visible` (Gate/Restriction) → optional `Reranker::rerank` → Antwort. Jeder Treffer trägt Provenienz (`contextra_explain`). `Erased`/`Restricted` erscheinen nie mit Inhalt.

### 7.3 Löschen

Sequenz nach §5.5. Pflichtbeobachtung: **Canary-Test T-3** schreibt einen eindeutigen Marker über alle Pfade (Dokument, Embedding, Textindex, Graph, Gedächtnis-Synthese, Checkpoint, Surrogat, Cache), löscht, und prüft Lesbarkeit in Fassade **und** in jedem Konsumenten, danach Restdatenmessung (`MeasureKind`).

### 7.4 Fristen und Packgruppen

`PackPolicy::ByTenantAndExpiry{bucket, shards}` für TTL-Daten; `Eraser::pack_expired` erzeugt Tickets je ≤ 4096 Cluster. `reap_expired_documents`/`trigger_expiry_cleanup` in `contextra-db` rufen diesen Pfad, nicht `delete`.

### 7.5 Konsolidierung (Gedächtnis)

`contextra-cognition` liest Quellcluster, erzeugt Synthese mit `DerivedFrom`-Lineage und Link auf alle Quellen; **Cross-Subject** verlangt Link-Admission, sonst Ablehnung. Löschung einer Quelle: Synthese wird mit dem Gate erfasst oder (bei gemeinsamer Löschdomäne) neu berechnet.

### 7.6 Checkpoint und Restore

`Checkpoint::take_snapshot`/`restore` und `CheckpointCoordinator::{create,restore,drop,list}_named_checkpoint` laufen über den Kern-Checkpoint (Kern-Spec §10.3) mit Rollback-Schutz (`Anchor`); keine Hardlink-Klone. Restore nach Löschung nur innerhalb des Backup-Horizonts (`BackupBeyondHorizon` sonst).

---

## 8. Sicherheit und Datenschutz

| ID | Regel |
|---|---|
| S-1 | `AdminCap` wird nur in `Contextra::open_authorized` aus der Embedding-Autorisierung erzeugt; `from_authorized_context` nicht `pub` (E10-14; `trybuild`-Test). |
| S-2 | Alle MCP-Eingaben untrusted: `MAX_RPC_BYTES`, `MAX_SEARCH_QUERY_BYTES`, `MAX_JSON_DEPTH`, `MAX_BATCH_SIZE` (vorhanden) bleiben; Kollektionsnamen über `validate_collection_name`. |
| S-3 | Tool-Kategorien (`ToolCategory`): `DatabaseRead`, `DatabaseWrite`, `CodeExecution`, `CloudEgress` + **neu `DatabaseErase`**; Schreibfreigabe (`is_write_allowed_by_env`) gilt auch für Erase; Erase verlangt zusätzlich `level` und `request_id`. |
| S-4 | Cloud-Egress nur über `EgressGateway`/`DefaultEgressGuard` (`EgressPolicy::evaluate`), Antworten über `CloudResponseRehydrator`; Surrogat-Vault in `privacy_vault` (Personen-Cluster). `BulkExfiltrationDetector` bleibt. |
| S-5 | `PromptInjectionGuard` vor jedem Tool-Ergebnis, das in ein Modell fließt; Treffer → `QuarantinePolicy`, Audit über `SecurityAuditLogger` ohne Klartext. |
| S-6 | Sandbox (`McpSandbox`, wasmtime): Ein-/Ausgaben flüchtig; persistierte Ergebnisse nur als Record im Cluster des Auftraggebers. |
| S-7 | Logging: keine ClusterId, Schlüssel, Körper, Querytexte (Kern: `Redacted<T>`); Metrikkatalog geschlossen, low-cardinality. |
| S-8 | Sicherheitsbefunde R10 §8 sind Vorbedingung (§14.1). |

---

## 9. Externe Schnittstellen

### 9.1 MCP-Toolkatalog (Soll)

| Tool | Kategorie | Änderung |
|---|---|---|
| `contextra_search`, `contextra_get`, `contextra_collections`, `contextra_explain` | `DatabaseRead` | unverändert; Ergebnisse nur gefiltert (`filter_visible`) |
| `contextra_insert`, `contextra_upsert` | `DatabaseWrite` | `data_class` Pflicht bei neuen Collections |
| `contextra_create_collection`, `contextra_drop_collection` | `DatabaseWrite` | `drop_collection` wird Erase-Ticket (kein stilles Löschen) |
| `contextra_delete` | `DatabaseWrite` | Antwort enthält `level: "Logical"`, `residual: ["LogicalOnly"]`, `receipt: null` |
| `contextra_forget` | `DatabaseWrite` | `deprecated: true`, Alias auf `contextra_erase` mit `level = Logical` |
| `contextra_erase` | **`DatabaseErase`** (neu) | `level`, `request_id`, `deadline`; Antwort: `ticket` oder `receipt` |
| `contextra_erase_status` | `DatabaseRead` (neu) | Ticketstatus (`TicketStage`/`TicketStatus`/`Residual`) |
| `contextra_verify_receipt` | `DatabaseRead` (neu) | `verify_receipt` offline |

### 9.2 Rust-Fassade `AgentMemory`

Ist: `new`, `new_arc`, `new_with_id_gen`, `with_id_gen`, `engine`, `remember`, `recall`, `forget(&MemoryId) -> Result<(), ContextraError>`, `relate`, `relate_n_ary`. Soll: `remember`/`recall`/`relate*` unverändert; `forget` **deprecated** und gibt `EraseOutcome::{Receipt(ErasureReceipt), Pending(Ticket), LogicalOnly}` zurück; neu `erase(&MemoryId, EraseLevel, deadline) -> Result<EraseOutcome, ContextraError>`. Ohne Receipt/Ticket kein `Ok`.

### 9.3 Python (`contextra-py`)

Neu: `erase`, `erase_status`, `receipt`, `verify_receipt`; keine Store-/Kern-Typen in der API; Fehler als Python-Ausnahmen mit `ErrorClass` und `hint`. `pyo3` auf eine Version ohne die in `deny.toml` ignorierten Advisories heben (Kern-Spec §3.7.6).

### 9.4 Wire

`contextra-wire` (FlatBuffers) transportiert Tickets/Receipt-Referenzen; **signiert** wird nie die FlatBuffers-Repräsentation, nur die kanonischen Receipt-Bytes.

---

## 10. Inferenz und Steuerung

Ports in `contextra-ports`: `EmbeddingProvider {provider_name, embed, embedding_dim, embed_batch}`, `TextEmbeddingEngine`, `TextGenerator`, `LlmTextGenerator {generate, generate_with_context}`, `LlmTextGeneratorStreaming`, `Reranker {rerank, record_implicit_feedback, rerank_deadline_ms}`, `QueryRewriter`. Implementierungen: `contextra-infer-onnx` (ort, Feature `onnx`/`reranking`), `-candle` (GGUF, Feature `candle`), `-ollama` (REST, Feature `client`). Regeln: (1) keine Persistenz in `infer-*`; (2) Modell-Fingerprint gehört in jeden abgeleiteten Index-/Cache-Schlüssel (Embeddings ändern sich mit dem Modell → `vec_idx` trägt `model_id`, Wechsel ⇒ Rebuild); (3) Netzaufrufe laufen asynchron am Rand und nie unter Kernsperren; (4) `contextra-router` + `contextra-adapt` entscheiden Modellwahl, Kalibrierung ist `NonPersonal`.

