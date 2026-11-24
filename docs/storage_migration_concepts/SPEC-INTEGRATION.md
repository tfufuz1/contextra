# SPEC-INTEGRATION.md — Gesamtsystem-Schnittstellenspezifikation Contextra

**`contextra-storage` als Ersatz für `contextra-store`**

| | |
|---|---|
| Revision | **I1.0 — Entwurf zur Freigabe als Gate G-INT-0** |
| Gegenstand | Außenverträge zwischen `contextra-storage` und allen übrigen Crates des Workspace: Port-Topologie, Fassaden-API, Async-Grenze, Record-/Schlüsselraum, Index-Anbindung, Engine-/Transaktionsmodell, Checkpoint/Zeitreise, Löschnachweis, Konfiguration, Fehlerabbildung, Migrations- und Verdrahtungsplan |
| Evidenzbasis | Klon von `github.com/tfufuz1/contextra`, Commit `2514565d486aa28cee8b0fba369ff6f20876c961` (2026-11-23), plus `crates/contextra-storage/SPEC.md` R9.1 (inhaltsgleich mit der hochgeladenen `SPEC.md`, per `diff` geprüft) |
| Normative Quelle für den Speicherkern | `crates/contextra-storage/SPEC.md` (R9.1). Dieses Dokument **ändert sie nicht**; wo es eine Änderung für nötig hält, steht sie als **[LÜCKE]** im Register (§16) und braucht eine Entscheidung im Storage-Review |
| Rangfolge bei Widerspruch | Storage-SPEC §15 (Invarianten) → Storage-SPEC §12.8 → übrige Storage-SPEC → dieses Dokument → Besitzer-SPs/READMEs der Verbraucher-Crates |
| Sprache | Prosa deutsch, Bezeichner englisch |
| Legende | **[SPEC]** durch die Storage-SPEC vorgegeben · **[BEFUND]** im Repo beobachtet (Pfad angegeben) · **[NEU]** Vorschlag dieses Dokuments · **[LÜCKE]** offene Entscheidung/Widerspruch · **[ENTSCHEIDUNG]** hier getroffen, begründet |

> **Vorläufigkeit.** Laut `PHASE-STATUS.md` ist kein Gate bestanden (G-0 offen, Formate nicht eingefroren, Toolchain 1.89 nicht belegt, `clippy`/`fmt`/`deny` ungeprüft). Alle Signaturen in §5–§9 sind deshalb **Vertragsentwürfe**, die gegen die Besitzer-SPs (`docs/01-ports.md` … `23-facade.md`) abzugleichen sind. Keine Leistungsaussage dieses Dokuments ist gemessen.

---

## 0. Kurzfassung

Der Altkern `contextra-store` ist ein **asynchroner, `TxId`-gesteuerter Key-Value-LSM** hinter dem Port `StorageEngine`. Darüber koordiniert `contextra-engine` vier Speicher (LSM, HNSW, BM25, CSR-Graph) per Intent-Log und kompensierenden Aktionen. Löschung ist dort eine Orchestrierung über Scanner/Purger und endet in einem `DeletionProof`.

Der neue Kern ist ein **synchroner, cluster-orientierter Speicher** mit atomaren Mehr-Space-Changes, MVCC-Sichten, clusterlokalen Branches und einem dreistufigen, nachweisbaren Löschprotokoll. Indizes sind **abgeleitete, über Watermarks angebundene Komponenten**, nicht Teil des Kerns.

Daraus folgen fünf Strukturentscheidungen, die den Rest dieses Dokuments tragen:

1. **Eine Port-Welt.** `contextra-storage-ports` ist die einzige Quelle für alle Persistenz-Ports. Das Modul `contextra-ports::persistence` und die Parallel-Abstraktionen (`durable-fs::Vfs`, `contextra-crypto::persistence`) werden zurückgebaut oder zu Adaptern (§4).
2. **Zwei Fassaden, eine Wahrheit.** Der Kern bleibt synchron. Eine **permanente** Async-Fassade (`AsyncStorage`) bedient die Tokio-Welt; ein **befristeter** Legacy-Adapter bildet `StorageEngine` für noch nicht migrierte Crates nach (§6).
3. **Eine Record-Schicht.** Neues Ring-1-Crate `contextra-records` besitzt Space-Katalog, Schlüsselcodecs, Kopfformate und die Zuordnung *Mandant/Person/Sammlung/Dokument → Cluster*. Sie ersetzt `TenantKeyCodec` (§7).
4. **Indizes werden Konsumenten.** `VectorIndex`/`TextIndex`/`GraphIndex` verlieren `insert(tx)/commit(tx)/rollback(tx)` und werden zu idempotent anwendbaren, Watermark-geführten `SegmentOwner` (§8). Das 4-Index-2PC der Engine entfällt (§9).
5. **Granularität der Löschung bestimmt Granularität der Cluster.** Kryptografische Löschung gibt es nur pro Cluster; `Eraser::range` ist für `Sealed` nur `Logical` [SPEC §9.8]. Wer Dokument-Crypto-Erasure will, muss das Dokument zum Cluster machen — mit der Obergrenze von 4096 Clustern je Löschhülle (§7.4).

**Kritischste Befunde (Details §1 und §16):** drei parallele Port-/Krypto-/FS-Abstraktionen im Workspace (B-02…B-04); Abweichungen des Ports-Crates von der eigenen SPEC (B-13: u. a. `Error::Shredded`, `Receipt` ohne `instance_id`, abweichender `KeyProvider`); `Hold` ist auf 24 h begrenzt, benannte Langzeit-Checkpoints haben daher keinen Kernvertrag (L-02); es fehlt ein `Eraser::tenant`-Pfad (L-01); `ContextraConfig::encryption_passphrase` führt unter `OpenProfile::Production` zu `ProviderAssuranceInsufficient` (§12).

---

## 1. Evidenz: Ist-Zustand

### 1.1 Befunde

| ID | Befund | Beleg |
|---|---|---|
| B-01 | Der Workspace enthält die sieben neuen Storage-Crates, `contextra-durable-fs` **und** den Altkern `contextra-store`; beide Kerne stehen in `default-members`. Der Altkern bleibt laut `PHASE-STATUS.md` bis zur P8-Ablöse unverändert. | `Cargo.toml`, `crates/contextra-storage/PHASE-STATUS.md` |
| B-02 | **Zwei Port-Welten.** `contextra-ports::persistence` (`Fs`, `File`, `Clock`, `Entropy`, `Metrics`, `Hooks`, `Pressure`, `Cipher`, `Mac`, `Kdf`, `KeyProvider { kek(), domain_key(KeyDomain), domain_key_create(), make_durable() }`, `FileKind`) und `contextra-storage-ports` (`Fs`, `Anchor`, `Clock`, `Entropy`, `Cipher`, `Mac`, `Kdf`, `Checksum`, `MemoryBudget`, `Metrics`, `Observer`, `Authority`, `ColdTier`, `Spawner`, `KeyProvider { create_cluster_key, cluster_key, probe, list, destroy_many }`). Die beiden `KeyProvider` sind **semantisch unvereinbar** (Domänenschlüssel vs. Cluster-/Epochenschlüssel). Die neuen Storage-Crates hängen ausschließlich von `contextra-storage-ports` ab. | `contextra-ports/src/persistence.rs`, `contextra-storage/crates/contextra-storage-ports/src/{keys.rs,traits/*}` |
| B-03 | **Dritte Dateisystem-Abstraktion:** `contextra-durable-fs::Vfs` ist **pfadbasiert** (`read`, `write_atomic`, `append`, `sync_file`, `sync_dir`, `remove`, `rename`, `list`) und kennt weder `read_at`/`read_ranges`, noch `preallocate`, noch `lock`. Es bringt aber brauchbare Bausteine mit: plattformspezifische Flush-Primitive, Fehlerinjektor, Poisoning-Zustandsautomat, Scrub mit Ratenbegrenzung, Pending-Delete-Journal. | `contextra-durable-fs/src/{lib.rs,vfs.rs,platform.rs,scrub.rs}` |
| B-04 | **Zweite Krypto-Welt:** `contextra-crypto` führt `KeyManager` (AES-256-GCM-SIV, HMAC-SHA256) für WAL/SSTables, `wal_crypto`, `kv_shredding`, `kv_segment`, `deletion_proof`, `audit_chain`, `ed25519_proof`, `revocation_log`. Die Storage-SPEC verlangt XChaCha20-Poly1305 über den `Cipher`-Port mit scope-spezifischem Schlüssel [SPEC §5.2]. `contextra-storage-ports` bietet `Cipher/Mac/Kdf/Checksum` und `contextra-crypto-ref` liefert die Referenz. | `contextra-crypto/src/lib.rs`, `contextra-storage/crates/contextra-crypto-ref` |
| B-05 | **Der Port `StorageEngine`** (`contextra-ports/src/storage.rs`) ist `BoxFuture`-basiert, `TxId`-gesteuert (`put(tx,…)`, `commit(tx)`, `rollback_to_tx`), kennt SSI-Tracking-Hooks (`get_tracked`, `scan_prefix_tracked`, `supports_ssi_tracking`), `put_if_absent`, `delete_prefix`, `scan_prefix_at(seq)` und Cursor-Scans mit `MAX_SCAN_MERGE_ACCUMULATOR = 100_000`. `graph_persist.rs` und `contextra-checkpoint` arbeiten generisch gegen `S: StorageEngine`. `contextra-engine`, `-cognition`, `-agent`, `-db` verwenden dagegen den **konkreten** `LsmStorage`; `contextra-engine` greift zusätzlich in Interna (`sstable::{BlockCache, SstableBuilder, SstableReader}`, `compaction::{CompactionEngine, CompactionConfig}`, `MergeOperator`, `scanner::{lsm_scanners, lsm_purgers}`, `TenantScopedStorage`). *(Zählungen aus `grep`; Produktions- und Testcode nicht getrennt.)* | `contextra-ports/src/storage.rs`, `grep contextra_store::` je Crate |
| B-06 | **Index-Ports tragen ein Transaktionsmodell:** `VectorIndex`/`TextIndex` mit `insert(tx,…)`, `delete(tx,…)`, `commit(tx)`, `rollback(tx)`, `rollback_to_tx`, `last_tx_id`. `DbTransaction` implementiert ein 4-Index-2PC mit `CommitIntent` unter `__tx_intent:<tx_id>` und kompensierenden Aktionen (`CompensateLsmAction`, `CompensateHnswAction`, …). | `contextra-ports/src/{vector_index.rs,text_index.rs}`, `contextra-engine/src/transaction/*` |
| B-07 | **Löschorchestrierung in der Engine:** `ErasureOrchestrator` ruft Purger in fester Reihenfolge Text → Vector → LSM; `LayerScanner`/`ScanSession` verlangen `scanned > 0` und `control_matches > 0`; `UNMEASURED_ERASURE_LAYERS` nennt sieben **nicht gemessene** Schichten (S1b Blob, S4 Caches, S5 Lineage, S6 Checkpoints, S7 Schlüsselmaterial, S9 Graph, S10 KV-Cache). Der Beleg ist ein `DeletionProof` aus `contextra-crypto`. | `contextra-engine/src/contextra_impl/erasure.rs`, `contextra-ports/src/scanner/*` |
| B-08 | **Mandantenmodell im Schlüssel:** `TenantKeyCodec` kodiert `t:{tenant}:{collection}:chunk|graph|doc:{id}` als ASCII; `TenantPolicy::{Optional, Required}`; `TenantId(u64)`, `CollectionId(u64)` (aus BLAKE3 des Namens abgeleitet), `DocId(u64)` bzw. `u128` per Feature `docid-128`. | `contextra-store/src/tenant_codec.rs`, `contextra-types/src/types/domain/ids.rs`, `contextra-engine/src/contextra_impl/erasure.rs` |
| B-09 | **Checkpoints:** `CheckpointCoordinator` (benannt; `create_named_checkpoint`, `restore_named_checkpoint`, `drop`, `list`) wird von `PersistentCheckpointStore` über `StorageEngine` persistiert; dazu `CheckpointGuard` (RAII-Rollback) und Hardlink-Cloner. Der Altkern hat ein `pub(crate)`-Checkpoint-Modul für Snapshot-Pinning. | `contextra-ports/src/checkpoint.rs`, `contextra-checkpoint/src/lib.rs` |
| B-10 | **MVCC-Crate:** `SequenceLog` (monotone Sequenz + Änderungsbenachrichtigung), `SnapshotRegistry`/`SnapshotGuard`, `TxBuffer`/`IndexOp` (gestaffelte Transaktionen mit Orphan-Reaping), `ReadSet`/`SsiValidator`. | `contextra-mvcc/src/lib.rs` |
| B-11 | `WalObserver`/`ObserverRegistry` (Circuit-Breaker, `CommittedBatch`) existieren im Altkern. Im gesichteten Quelltext von `engine`, `graph`, `vector`, `text` gibt es **keinen** Treffer auf `WalObserver`. | `contextra-store/src/lsm/observer.rs`, `grep` |
| B-12 | `LsmConfig`/`ContextraConfig` führen `memtable_size_limit`, `max_ram_mb`, `group_commit_window_micros`, `tx_timeout`, `encryption_passphrase: Option<String>`, `block_cache_shards`, `durability_mode ∈ {Full, WalNoHmac, MemoryOnly}`, `ssi_max_tracked_keys`, `deletion_proof_active`, `vector_delete_mode`. `KvDeleteMode ∈ {TombstoneOnly, CryptoShred}` (Default `CryptoShred`). | `contextra-store/src/lsm/config.rs`, `contextra-engine/src/lib.rs`, `contextra-store/src/kv/delete_mode.rs` |
| B-13 | **Abweichungen des Ports-Crates von der Storage-SPEC** (stimmt mit dem in SPEC §12.8 Nr. 30 genannten offenen WP-01.5 überein): (a) `Error::Shredded` existiert als **Fehlervariante**, die SPEC verbietet das (§3.5 F-6, §7 Nr. 5: „`Shredded` ist … nie ein Lesefehler“); (b) es fehlen `SnapshotTooOld`, `ErasePending`, `BackupBeyondHorizon`, `LedgerUnavailable`, `LedgerSameDevice`, `ProviderAssuranceInsufficient`, `FdLimitTooLow`/`AdmissionLimit`, `Revoked` (SPEC §3.5 F-7, §4.3.1, §11.4); (c) `Receipt { stamp, durable }` ohne `instance_id` (SPEC §6.1, §6.9 verlangt `{instance_id, stamp, durable}`); (d) `KeyProvider`: Einzel-`create_cluster_key` statt Batch ≤ 256 mit `BatchId`, `keys_in_backups: bool` statt `KeysInBackups`, kein `assurance`, keine Tomb-Epochen-Methoden, `probe` je Cluster statt je Epoche, `list` ohne `created_utc`; (e) `MAX_GROUP_BYTES = 4 MiB` gegenüber `max_group_bytes` 8 MiB (SPEC §16); (f) `DEFAULT_MAX_CLUSTERS = 4_000_000` gegenüber 16 Mio. weich (SPEC §4.6); (g) `Storage::open` liefert `Storage`, die SPEC verlangt einen `OpenReport` (§10.1 OP10); (h) `facade.rs` enthält nur Kommentare, `Memtable`/`ConsumerFeed` sind Platzhalter. | `contextra-storage-ports/src/{error.rs,change.rs,keys.rs,limits.rs,facade.rs,traits/engine.rs}`, `contextra-storage/src/lib.rs` |
| B-14 | `contextra-storage-bridge/src/lib.rs` ist ein leerer Platzhalter. `docs/18-bridge.md` fordert dort Async-Adapter, Legacy-KV-Abbildung (Space `LEGACY_KV`, Plain-Cluster je Mandant), Idempotenz-Pflicht und die Abbildung Receipt → `DeletionProof`. | `contextra-storage-bridge/src/lib.rs`, `docs/18-bridge.md` |
| B-15 | Nur acht Crates haben eine Cargo-Abhängigkeit auf `contextra-store`: `contextra`, `-py`, `-db`, `-engine`, `-graph`, `-cognition`, `-checkpoint`, `-agent`; dazu `benchmarks/contextra-bench`. `contextra-vector` hat eine **eigene** Persistenz (`crate::persistence::MmapIndex`) neben der Abhängigkeit auf `mvcc`/`crypto`. | `grep` in allen `Cargo.toml`, `contextra-vector/src/hnsw/batch.rs` |

### 1.2 Verbraucherinventar

| Crate | Kopplung an den Altkern | Konsequenz im Zielbild |
|---|---|---|
| `contextra-engine` | konkreter `LsmStorage`, Interna (B-05), `DbTransaction` 4-Index-2PC (B-06), `ErasureOrchestrator` (B-07), Collections/CRUD/Search, Hintergrund-Worker | größte Refaktorierung: Transaktionsmodell, Keyspace, Erasure (§9) |
| `contextra-db` | `LsmStorage::new`, `SystemPressure`, `CompactionConfig`; Homeostat, Multistep, Volatile-Vault | Konfiguration/Pressure → Governor (§12); Volatile-Vault → Volatile-Spaces |
| `contextra-graph` | generisch `S: StorageEngine` (`persist_entity`, `persist_edge`, `delete_edge_persistence`), Session-DAG | Persistenz → Records (§7); CSR → `SegmentOwner` (§8) |
| `contextra-cognition` | `LsmStorage` (Konsolidierung, Kompaktierung, Locks, Wartungsplaner) | Guards + Idempotenz statt Intent-Keys (§9.3) |
| `contextra-agent` | `LsmStorage` (`task:`/`audit:`/`dlq:`-Schlüssel) | Space `ctx.agent.*`, Session ≠ Cluster (§7.2) |
| `contextra-checkpoint` | generisch `StorageEngine`; Hardlink-Cloner | Neuzuschnitt (§10) |
| `contextra` (Fassade) | `DurabilityMode`, `KvDeleteMode`-Typen | Typen wandern nach `contextra-types` (§12) |
| `contextra-py`, `-bench` | nur Abhängigkeit | folgt der Fassade |
| unberührt auf Cargo-Ebene | `-vector`, `-text`, `-rank`, `-router`, `-adapt`, `-kvcache`, `-infer-*`, `-privacy`, `-ingest`, `-mcp`, `-wire`, `-sys`, `-simd`, … | indirekt betroffen über Ports (§8); `-vector` zusätzlich über seine eigene Persistenz (B-15, L-12) |

### 1.3 Schlüsselraum-Inventar des Altsystems

Per `grep` im Quelltext gefundene Schlüsselfamilien (Produktions- und Testcode gemischt; vor der Migration per Laufzeit-Scan zu bestätigen):

| Familie | Fundstellen-Muster | Natur |
|---|---|---|
| Nutzdaten | `t:{tenant}:{col}:chunk:{doc}`, `…:graph:{entity}`, `…:doc:{doc}`, `__col:{n}:`, `__docid:`, `__col_idx:`, `__txt:{…}:`, `__meta:dimension`, `collection:{…}` | kanonisch |
| Graph | `__graph:entity:`, `__graph:edge:{from}:{to}`, `__graph:community:{…}`, `__graph:hyperedge:`, `__graph:hyperedge_by_entity:`, `__graph:cascade_queue:`, `graph:entity:deleted:{…}`, `__rel:{doc}:{rel}:` | kanonisch / Arbeitsqueue |
| Agent | `task:{id}:step:{n}`, `task:{id}:final`, `audit:{id}:step:{n}`, `tool:{handler}`, `dlq:`, `__session_dag:{ns}:` | Zustand / Protokoll |
| Transaktions-/Wartungsmarker | `__tx_intent:`, `__maintenance_intent:tick`, `consolidation_intent:`, `contextra:consolidated:{k}` | **Protokollartefakte des 2PC/OCC** — entfallen (§9.3) |
| Segmente | `tenant:{}:seg:{}` | KV-Segmentverwaltung |

### 1.4 Semantikvergleich Alt → Neu

| Altes Konzept | Neues Konzept | Qualität der Abbildung |
|---|---|---|
| `TxId`, `seq_no` | `Stamp` (instanzlokal, nur mit `InstanceId` vergleichbar) | **verlustfrei**, aber `TxId` ist nicht mehr Aufrufer-vergeben |
| `put/delete(tx,…)` + `commit(tx)` | `Change { ops, guards, idem, durability }` + `apply` | verlustfrei (Staging entfällt, kein serverseitiges Orphan-Reaping) |
| SSI-`ReadSet`, `get_tracked`, `scan_prefix_tracked` | explizite `Guard::Key`/`Guard::Range` (Stufe A exakt, Stufe B im Sequencer) | **nicht gleichwertig**: Snapshot-Isolation + Guards statt SSI (NZ-3). Schreibschiefe muss durch Guard-Modellierung verhindert werden (§9.2) |
| `put_if_absent` | `Guard::Key{pre: Absent}` | verlustfrei |
| `MergeOperator` | Version-Guard + Retry (E-29) | semantische Änderung (§9.4) |
| `delete_prefix` | `Op::RangeDelete{start,end}` mit Präfix-Nachfolger als `end` | verlustfrei; **Sealed** nur logisch relevant (§7.4) |
| `scan_prefix` → `Vec` | `Cursor`/`page` mit Budget und `ResumeToken` | Verbesserung; unbeschränkte Scans entfallen |
| `scan_prefix_at(seq)`, `get_at_seq` | `view_at(stamp)` | verlustfrei bis `history_floor` |
| `rollback_to_tx` (gesamte DB) | kein In-Place-Rollback; nur Branch-`reset`/Restore | **bewusst nicht abbildbar** (§10) |
| `pin_checkpoint(seq)` | View-Lease (≤ 10 min) bzw. `Hold` (≤ 24 h) bzw. Branch | **Lücke** für Langzeit-Pins (L-02) |
| `WalObserver` | Consumer-Feed aus dem Redo | gleichwertig, *mindestens-einmal*, nur Stamps ≤ `durable_stamp` |
| `SystemPressure` | `MemoryBudget` + Governor | gleichwertig; harte Admission statt Pegel |
| async (`BoxFuture`) | synchroner Kern + Async-Fassade | Adapter nötig (§6) |
| `DeletionProof` über Scanner | `ErasureReceipt` + Owner-Atteste + `verify_receipt` | strenger, mit `MeasureKind` (§11) |

### 1.5 Grenzen dieser Analyse

Gelesen wurden: vollständig die Storage-SPEC; die Ports-Crates, `lib.rs`-Köpfe und die Schnittstellen der Crates `store`, `mvcc`, `checkpoint`, `durable-fs`, `crypto`; die Engine-Module `transaction`, `erasure` und Ausschnitte von `lib.rs`; Schlüsselfamilien per `grep`. **Nicht** im Detail gelesen: Implementierungen von `contextra-vector`/`-text`/`-graph`/`-cognition`/`-agent`, `kv_segment`/`kv_shredding`, `audit_chain`, `contextra-kvcache`, `contextra-ingest`, `contextra-audit-export`, `contextra-avv-generator`. Aussagen dazu sind als „zu verifizieren“ markiert. Vor jedem Refaktorierungs-WP in §14 ist die betroffene Schnittstelle am Code zu bestätigen.

---

## 2. Leitentscheidungen

| ID | Entscheidung | Begründung | Verworfene Alternative |
|---|---|---|---|
| ID-1 | `contextra-storage-ports` ist die **einzige** Quelle aller Persistenz-Ports. `contextra-ports::persistence` wird nach Migration von `contextra-crypto` entfernt. | Storage-SPEC §3.1 und die Cargo-Abhängigkeiten der neuen Crates legen `storage-ports` fest; zwei inkompatible `KeyProvider` sind ein Sicherheitsrisiko (falscher Provider ⇒ falsche Löschsemantik). | Beide Welten parallel pflegen; `persistence` zum Master machen (widerspricht SPEC §9.4 und Cluster-Epochen) |
| ID-2 | Der Kern bleibt **synchron**. Die Async-Fassade (`AsyncStorage`) ist **dauerhaft**; der Legacy-`StorageEngine`-Adapter ist **befristet** bis G-CUTOVER. | NZ-17/K-12; `docs/18-bridge.md` verbietet eine dauerhafte Altschicht. | Kern async machen (verliert Loom-/Simulationsfähigkeit) |
| ID-3 | Domain-Crates sprechen **nicht** direkt `Storage`, sondern `RecordStore` aus `contextra-records`. | Eine Stelle für Space-Katalog, Codecs, Scope-Auflösung und Guard-Aufzeichnung; Mandantenisolation prüfbar an einer Naht. | `Storage` direkt in jedem Crate (Wiederholung der Schlüsselkonventionen, Mandantenleck-Risiko) |
| ID-4 | **Kein Dual-Write** und kein Schreiben in den Altkern aus dem neuen Pfad. Migration ist ein einmaliger, verifizierter Forward-Export. | Storage-SPEC §14 G-CUTOVER. | Dual-Write zur Risikominderung (bricht Erasure-Semantik) |
| ID-5 | Indexe werden `SegmentOwner` + Consumer. Der Kern garantiert **keine** atomare Kopplung von Quelldaten und Indexgeneration. | Storage-SPEC §12.4.2. | 2PC über LSM+Indexe beibehalten |
| ID-6 | Mandantenisolation wird durch **Cluster-Zuordnung plus `tenant_tag`** erzwungen, nicht durch Schlüsselpräfix. Die Prüfung liegt im `ScopeResolver` (obere Schicht, NZ-16). | Cluster sind die Isolationseinheit; ein Präfix innerhalb eines Clusters würde nur Mandanten *im selben Cluster* trennen. | `t:{tenant}:`-Präfix beibehalten |
| ID-7 | Space-Namen sind **versioniert und nach `DataClass` getrennt** (`ctx.<family>.<personal\|shared>.v<n>`). | `SpaceOptions` sind unveränderlich; `DataClass::Personal` erzwingt `Sealed` + geclustert [SPEC §4.5]; `SpaceId`s sind nie wiederverwendbar. | Ein Space je Familie mit gemischter Klasse (unzulässig) |
| ID-8 | `IdemToken`s sind **zufällige, vom Aufrufer je Request vergebene** 16-Byte-Werte, nie inhaltsabgeleitet. | `INV-PRIV-1`: keine persistierten Inhalts-Hashes. | `IdemToken = H(Request)` |
| ID-9 | Die Engine bindet jede Suche an eine `view_durable()`-Sicht und lässt **jedes** Index-Ergebnis durch `View::filter_visible`, bevor es den Aufrufer erreicht. | `INV-VIS-1`. | Vertrauen auf `on_gate`-Invalidierung allein |

---

## 3. Zielarchitektur

### 3.1 Schichten

```
Ring 3  contextra (Fassade) · contextra-mcp · contextra-py · contextra-ingest · contextra-db
        │
Ring 2  contextra-engine (Collections, IngestUnit, Search, ErasureCoordinator)
        contextra-cognition · contextra-agent · contextra-router · contextra-rank · contextra-adapt
        │
Ring 1  contextra-vector · contextra-text · contextra-graph      (SegmentOwner + Query)
        contextra-checkpoint (Neuzuschnitt)
        contextra-records   [NEU]  Space-Katalog · Codecs · ScopeResolver · RecordStore · ReadSetRecorder
        contextra-storage-bridge   AsyncStorage (permanent) · LegacyKvAdapter (befristet, Feature legacy-kv)
        │
Ring 1  contextra-storage   synchroner Kern (SPEC R9.1)
        contextra-crypto-ref · contextra-keys-file · contextra-keys-cohort
        │
Ring 0  contextra-storage-ports   (alle Persistenz-Ports)
        contextra-ports           (Domain-Ports: Index, Embedding, Scanner/Witness, …)
        contextra-types · contextra-sys · contextra-wire
```

*Ringzuordnung nach `package.metadata.contextra.ring` ist in jedem neuen Crate zu setzen; `contextra-store` hat Ring 1.*

### 3.2 Crate-Topologie

| Crate | Status | Rolle im Zielbild |
|---|---|---|
| `contextra-storage-ports` | Bestand, **erweitern** | alle Persistenz-Ports (§4), Fassaden-Wertetypen (§5) |
| `contextra-storage` | Bestand, in Arbeit | Kern; Fassaden-Signaturen nach §5 |
| `contextra-crypto-ref` / `-keys-file` / `-keys-cohort` | Bestand | Referenz-/Produktiv-Provider; `keys-file` ist `FilePassphrase` (§12) |
| `contextra-storage-bridge` | Bestand (Platzhalter) | `AsyncStorage`, `LegacyKvAdapter`, `DeletionProofBridge` (§6, §11) |
| `contextra-records` | **neu** | Ring 1; hängt von `storage-ports`, `contextra-types`, optional `contextra-storage-bridge` ab |
| `contextra-ports` | Bestand, **zurückschneiden** | `persistence`-Modul entfällt; `StorageEngine` wird `#[deprecated]`, `KvBridgeStorage` bleibt (§6.4); Index-Ports werden umgebaut (§8) |
| `contextra-store` | Bestand, **einfrieren** | keine Änderung bis G-CUTOVER, danach Archivierung (Tag `legacy-store-final`, Storage-SPEC §14) |
| `contextra-durable-fs` | Bestand, **umwidmen** | liefert Plattform-Primitive und Scrub für den `RealFs`-Adapter; `Vfs` entfällt (§4.3) |
| `contextra-crypto` | Bestand, **aufteilen** | Store-nahe Teile (`KeyManager` für WAL/SSTable, `wal_crypto`, `persistence`) entfallen; Beleg-/Signatur-Teile (`deletion_proof`, `audit_chain`, `ed25519_proof`) bleiben (§11) |

### 3.3 Abhängigkeitsregeln (`layering`-Test, ergänzt Storage-SPEC §3.1)

1. Nur `contextra-storage-bridge` und `contextra-records` dürfen `contextra-storage` importieren. Alle anderen Crates sehen `contextra-records` bzw. Ports.
2. Kein Crate außerhalb der Adapter-Crates ruft `std::fs`, `SystemTime` oder `rand` auf (erweitert die `disallowed_methods`-Regel des Kerns auf Ring 1–3 für Persistenzpfade).
3. Kein Domain-Crate importiert `contextra-store` (nach der Migration CI-Verbot; bis dahin Allowlist je Crate, schrumpfend).
4. `contextra-vector`, `-text`, `-graph` dürfen **nicht** von `contextra-engine` abhängen (Zykel-Verbot bleibt) und greifen auf Persistenz nur über `SegmentIo` zu (§8.3).
5. Wer ein Segment oder Derivat persistiert, ist als `SegmentOwner` registriert (`INV-INT-1`, §15).

### 3.4 Synchron/Asynchron-Grenze

Der Kern ruft nie eine Runtime. `AsyncStorage` besitzt einen eigenen, begrenzten Blocking-Pool (`Spawner`-Port, `contextra-storage-ports/traits/spawn.rs`). Regeln:

- Jeder `apply` über die Async-Grenze trägt einen `IdemToken` (ID-8); Cancellation nach Übergabe ist mit demselben Token wiederholbar und ergibt `AlreadyApplied` oder den gespeicherten terminalen Status, nie ein zweites Apply [`docs/18-bridge.md` §2].
- Rückstau des Blocking-Pools wird als `Backpressure{retry_after}` sichtbar, nie als unbeschränkte Queue.
- Der Pool hat eine reservierte Lane für `urgent`-Arbeit (Löschaufträge), die nicht hinter normalen Writes wartet.

---

## 4. Port-Konsolidierung

### 4.1 Besitzmatrix der Ports

| Belang | Einziger Besitzer (Zielbild) | Entfällt / wird Adapter |
|---|---|---|
| Dateisystem (`Fs`, `File`, `FsLock`, `FsCaps`) | `storage-ports::traits::fs` | `ports::persistence::{Fs,File}`; `durable-fs::Vfs` (§4.3) |
| `Clock`, `Entropy` | `storage-ports::traits::anchor` (Clock mit Skew-Status) | `ports::persistence::{Clock,Entropy}`; `ports::Clock` bleibt für Domain-Code (nur Lesezeit), **nie** für Stamps/Fristen |
| `Cipher`, `Mac`, `Kdf`, `Checksum` | `storage-ports::traits::crypto` | `ports::persistence::{Cipher,Mac,Kdf}`; `contextra-crypto::persistence` |
| `KeyProvider` | `storage-ports::keys` (Zielform §4.2) | `ports::persistence::KeyProvider` |
| `Anchor`, `Authority`, `ColdTier`, `HostProbe` | `storage-ports::traits::*` | — |
| `MemoryBudget` | `storage-ports::traits::memory` | `SystemPressure`/`PressureSource` (Altkern, Ports) → Governor |
| `Metrics`/`Observer` (Storage) | `storage-ports::traits::observe` (geschlossener Katalog) | `ports::persistence::Metrics`; Brücke zu `ports::MetricsSink` (§12.3) |
| Domain-Metriken (Engine, Router, Index) | `ports::MetricsSink` | — |
| Index-/Embedding-/Scanner-Ports | `contextra-ports` (umgebaut, §8) | tx-Lebenszyklus der Index-Ports |

### 4.2 Zielform `KeyProvider` [SPEC §9.4]

Der Port wird auf die SPEC-Form gebracht (WP-01.5). Verbindlich für alle Provider (`crypto-ref`, `keys-file`, `keys-cohort`, spätere HSM/KMS-Crates):

```rust
pub trait KeyProvider: Send + Sync {
    fn info(&self) -> KeyProviderInfo;                       // destroy, keys_in_backups: KeysInBackups, assurance: KeyAssurance
    fn kek(&self, id: KekId) -> Result<Key32, KeyError>;
    fn create_cluster_keys(&self, request_id: BatchId, cs: &[ClusterId], e: &dyn Entropy)
        -> Result<Vec<KeyCreateResult>, KeyError>;           // ≤ 256, idempotent je BatchId, Teilresultate, Erfolg vor Rückgabe durabel
    fn cluster_keys(&self, req: &[(ClusterId, u32)]) -> Result<Vec<KeyLookup>, KeyError>;   // ≤ 256; Missing ≠ Shredded
    fn destroy_many(&self, cs: &[ClusterId]) -> Result<DestroyReceipt, KeyError>;           // ≤ 256, ALLE Epochen, idempotent
    fn probe(&self, c: ClusterId, epoch: u32) -> Result<KeyProbe, KeyError>;                // Present | Disabled | Absent
    fn list(&self) -> Result<Vec<KeyListEntry>, KeyError>;                                  // { cluster, created_utc }
    fn create_tomb_epoch(&self, epoch: u32) -> Result<(), KeyError>;
    fn tomb_key(&self, epoch: u32) -> Result<Option<Key32>, KeyError>;                      // None ⇒ retired
    fn retire_tomb_epoch(&self, epoch: u32) -> Result<KeyProbe, KeyError>;
}
```

`KeyError` unterscheidet **`Unavailable{retry_after}`**, **`Absent`**, **`Disabled`**, **`Lost`** [SPEC §12.8 Nr. 13]; Provider-Nichterreichbarkeit wird nie als Vernichtung gedeutet.

### 4.3 `contextra-durable-fs` → Plattform-Backend

`Vfs` ist für den Kern ungeeignet (B-03: keine Range-Reads, keine Vorallokation, kein Lock, ganzdateiorientiert). **[ENTSCHEIDUNG]** Das Crate wird zum Plattform-Backend:

| Baustein | Verwendung |
|---|---|
| `platform::{sync_file, sync_dir, FlushKind}` | Implementierung von `Fs::sync_data/sync_all/sync_dir` im `RealFs`-Adapter (inkl. Windows-Pfad, Storage-SPEC NZ-8) |
| `health::{AtomicHealthState, HealthEvent}` | Basis für das Handle-Poisoning-Modell (F-3, `INV-PIPE-2`) — nur wenn Zustände deckungsgleich sind, sonst Kern-eigen |
| `fault_injector` | Nur als Quelle von Szenarien; Kern-Tests nutzen `FaultFs` (Geister-Journal, Storage-SPEC §13.1) |
| `scrub::{scrub_if_safe, ScrubReceipt, ScrubRefusal}` | Mechanik für `EraseLevel::Scrubbed` (best-effort, CoW/SSD ausdrücklich nicht belegt) |
| `journal::PendingDeleteJournal` | Vergleichsimplementierung zum Catalog-Graveyard; **nicht** parallel betreiben (Catalog ist einzige Wahrheit, L6) |
| `vfs::{Vfs, StdVfs, RealFs}` | **entfällt** nach Migration der Verbraucher (Nutzer vorab per `grep` ermitteln, L-11) |

### 4.4 Angleichungsliste `storage-ports` ↔ SPEC (Gate G-INT-0)

Alle Punkte aus B-13 sind vor G-0 zu schließen. Zusätzlich nötig für die Integration **[NEU]**:

| Nr. | Änderung | Grund |
|---|---|---|
| P-1 | `Error::Shredded` entfernen; Zustand `ClusterLifecycle::Shredded` bleibt | SPEC §3.5 F-6, §7 Nr. 5 |
| P-2 | Fehlende Varianten aus B-13(b) ergänzen; `Error::hint()` mit stabiler, sicherer Handlung | Z-18, F-7 |
| P-3 | `Receipt { instance_id: InstanceId, stamp, durable }` | SPEC §6.1 Nr. 9, §2.3 |
| P-4 | `ApplyOutcome` einführen (§5.2) | Idempotenz-Ergebnis als Erfolgsfall |
| P-5 | `HostProbe`, `BlockPruner`, `MeasureKind`, `SegmentIo`, `SegmentOwner`, `ExtensionBuilder` als Ports aufnehmen | SPEC §3.2, §12 |
| P-6 | `Spaces`-Registrierung mit Namen (`ensure`, §5.4) | Space-IDs sind nicht konstant (§7.1) |
| P-7 | `Clusters::info()` mit `tenant_tag`, `protection`, Lifecycle (§5.3) | `ScopeResolver` braucht die Mandantenprüfung |
| P-8 | `MAX_GROUP_BYTES`, `DEFAULT_MAX_CLUSTERS` auf SPEC-Werte oder SPEC auf Code-Werte (Entscheidung, L-08) | B-13(e,f) |

---

## 5. Fassade `Storage` (synchron) — Vertragsentwurf

Die SPEC gibt nur Namen vor (`facade.rs` enthält ausschließlich Kommentare). Die folgenden Signaturen sind der **Vorschlag für SP-FACADE (23)**; **[SPEC]** kennzeichnet, was aus der Norm folgt.

### 5.1 Identität und Grundtypen

```rust
// bestehend in storage-ports: ClusterId([u8;16]), SpaceId(u64), BranchId(u32), InstanceId, Stamp(u64 < 2^56),
//   IdemToken, EraseRequestId, Ticket, TombId, DurablePosition{instance, stamp}, DataClass, Redacted<T>
pub struct RecordRef { pub space: SpaceId, pub cluster: ClusterId, pub branch: BranchId, pub key: Bytes, pub stamp: Stamp }  // [NEU] Eingabe für filter_visible
pub struct Receipt   { pub instance_id: InstanceId, pub stamp: Stamp, pub durable: bool }                                   // [SPEC §6.1]
```

`Stamp` ist **nur mit `InstanceId` vergleichbar** [SPEC §2.3]. Jede Persistenz außerhalb des Kerns (Index-Watermarks, Consumer-Cursor, Checkpoint-Marken, Audit-Referenzen) speichert `DurablePosition`, nie nacktes `Stamp`.

### 5.2 Schreiben

```rust
pub enum ApplyOutcome { Applied(Receipt), AlreadyApplied(Receipt) }                       // [NEU, ENTSCHEIDUNG] Idempotenz-Treffer ist ein Erfolg
pub enum ApplyError {                                                                      // [SPEC §3.5 F-2…F-8], alle mit hint()
    Conflict { contended: bool }, Backpressure { retry_after: Duration }, DiskFull, ChangeTooLarge,
    ClusterRetired, Erased { at: Stamp }, Restricted, BranchClosed, UnknownCluster, ReservedSpace,
    IdemConflict, IdemCapacity, Indeterminate { first_stamp: Stamp, last_stamp: Stamp }, Poisoned, Limit(&'static str), /* … */
}
impl Storage {
    pub fn apply(&self, change: Change) -> Result<ApplyOutcome, ApplyError>;            // [SPEC §6.1]
}
```

Vertrag (alle **[SPEC]**): kein ACK vor `sync_data`; `Durable` ⇒ sichtbar erst nach Sync; `Relaxed` nie für Löschung; `Indeterminate` ist **nur** über denselben `IdemToken` auflösbar, blindes Retry ohne Token ist verboten; ein Retry nach Cluster-Retire liefert `Erased`/`ClusterRetired`, nie ein neues Apply (§6.5). Die Fassade fügt die implizite `Absent`-Guard auf den `SYS_IDEM`-Schlüssel selbst an (K-38); Aufrufer sehen sie nicht.

### 5.3 Lesen

```rust
impl Storage {
    pub fn view(&self) -> View;                                    // visible_stamp, für Diagnose/Admin
    pub fn view_durable(&self) -> View;                            // Pflicht für Indexbau, Feed und Suche (INV-CONS-1)
    pub fn view_at(&self, s: Stamp) -> Result<View, ReadError>;    // BelowFloor, SnapshotTooOld
}
impl View {
    pub fn stamp(&self) -> DurablePosition;
    pub fn on_branch(&self, b: BranchId) -> Result<View, ReadError>;
    pub fn get(&self, sp: SpaceId, c: ClusterId, key: &[u8]) -> Result<Option<Record>, ReadError>;   // Kopf ohne Körper-E/A
    pub fn body(&self, r: &Record) -> Result<Zeroizing<Vec<u8>>, ReadError>;                          // KeyUnavailable | KeyLost | Erased
    pub fn cursor(&self, sp: SpaceId, c: ClusterId, range: KeyRange, o: CursorOpts) -> Result<Cursor, ReadError>;
    pub fn page(&self, sp: SpaceId, c: ClusterId, range: KeyRange, b: PageBudget, resume: Option<ResumeToken>)
        -> Result<Page, ReadError>;                                // Budget nie überschritten außer oversize_single [SPEC §7 Nr. 4]
    pub fn filter_visible(&self, refs: &[RecordRef]) -> Result<Vec<Visible>, ReadError>;              // INV-VIS-1
    pub fn renew(&self) -> Result<(), ReadError>;                  // bis view_lease_max
}
pub struct PageBudget { pub weight_slot: u8, pub max_weight: u64, pub max_bytes: usize /* ≤ 1 MiB */, pub max_entries: usize }
pub struct Record { pub key: Bytes, pub stamp: Stamp, pub header: Bytes, pub weights: [u32; 4], /* Körper lazy */ }
pub enum ReadError { UnknownCluster, Erased { at: Stamp }, ErasePending, Restricted, Revoked, SnapshotTooOld { stamp: Stamp, floor_hint: Stamp },
                     BelowFloor, KeyUnavailable { retry_after: Option<Duration> }, KeyLost, Corrupt { place: &'static str }, /* … */ }
```

- **Gate zuerst** [SPEC §7 Nr. 2]: Clusterzustand und Restrictions werden vor jeder Entsiegelung geprüft; `Erased` gilt für jede Sicht, auch ≥ S.
- `Shredded` ist **kein** `ReadError` (P-1).
- Verbraucher halten `View`s nur kurz (Lease 60 s, max. 10 min). Lange Scans setzen über `ResumeToken` + `view_at(V)` neu auf; `BelowFloor` und `SnapshotTooOld` sind getrennt zu behandeln.

### 5.4 Spaces, Cluster, Branches

```rust
impl Storage {
    pub fn spaces(&self) -> Spaces<'_>;
    pub fn clusters(&self) -> Clusters<'_>;
    pub fn fork(&self, c: ClusterId, parent: BranchId, at: ForkPoint, o: BranchOptions) -> Result<BranchId, BranchError>;   // [SPEC §8.4]
    pub fn branch_drop(&self, c: ClusterId, b: BranchId) -> Result<(), BranchError>;
    pub fn merge(&self, c: ClusterId, source: BranchId, target: BranchId, mode: MergeMode, idem: IdemToken)
        -> Result<MergeOutcome, BranchError>;                                   // Merged(Receipt) | Conflicts(Vec<ConflictCell>)
    pub fn diff(&self, a: &View, b: &View, c: ClusterId, range: KeyRange) -> Result<DiffIter, BranchError>;
}
impl Spaces<'_> {
    pub fn ensure(&self, name: &str, opts: SpaceOptions) -> Result<SpaceHandle, SpaceError>;   // [NEU] idempotent; gleicher Name + gleiche Optionen ⇒ gleiche SpaceId
    pub fn lookup(&self, name: &str) -> Result<Option<SpaceHandle>, SpaceError>;
}
impl Clusters<'_> {
    pub fn register(&self, spec: ClusterSpec) -> Result<ClusterId, ClusterError>;              // [SPEC §4.3]; Admission INV-ADM-1
    pub fn info(&self, c: ClusterId) -> Result<ClusterInfo, ClusterError>;                    // [NEU] state, protection, tenant_tag, due, locality, restriction_flags
    pub fn link(&self, derived: ClusterId, sources: &[ClusterId]) -> Result<(), ClusterError>;// ≤ 8 Quellen; EraseFanoutExceeded
    pub fn restrict(&self, c: ClusterId, r: RestrictionSpec) -> Result<RestrictionId, ClusterError>;
    pub fn lift(&self, c: ClusterId, id: RestrictionId, p: LiftAuth) -> Result<(), ClusterError>;
}
pub struct SpaceHandle { pub id: SpaceId, pub name: Arc<str>, pub options: SpaceOptions }
pub struct ClusterInfo { pub state: ClusterLifecycle, pub protection: Protection, pub tenant_tag: [u8; 16],
                         pub due: Option<UtcSeconds>, pub locality: EraseLocality, pub restricted: RestrictionFlags }
```

`ensure` ist nötig, weil `SpaceId`s monoton und nie wiederverwendbar sind (`INV-ID-1/2`) und außer den Systemspaces `SYS_BRANCH = 1`, `SYS_IDEM = 2` **keine konstanten Werte** haben. `ensure` mit gleichem Namen, aber abweichenden (unveränderlichen) Optionen liefert `SpaceError::OptionsMismatch`; Schemaänderung = neuer Name mit höherer Versionsnummer (ID-7).

### 5.5 Löschen

```rust
pub struct EraseRequest { pub request_id: EraseRequestId, pub level: EraseLevel, pub purge_deadline: Duration /* Pflicht, kein Default */,
                          pub view_policy: ViewPolicy }
impl Storage {
    pub fn eraser(&self) -> Eraser<'_>;
    pub fn space_drop(&self, sp: SpaceId, req: EraseRequest) -> Result<Ticket, EraseError>;     // Logical beide Seal-Modi; Physical|Scrubbed nur Plain
}
impl Eraser<'_> {
    pub fn cluster(&self, c: ClusterId, req: EraseRequest) -> Result<Ticket, EraseError>;       // Sealed ⇒ Cryptographic, Plain ⇒ Physical|Scrubbed [SPEC §9.2]
    pub fn clusters_batch(&self, cs: &[ClusterId], req: EraseRequest) -> Result<Vec<Ticket>, EraseError>;   // deterministisch in Tickets ≤ 4096
    pub fn range(&self, sp: SpaceId, c: ClusterId, r: KeyRange, req: EraseRequest) -> Result<Ticket, EraseError>;
    pub fn branch(&self, c: ClusterId, b: BranchId, req: EraseRequest) -> Result<Ticket, EraseError>;
    pub fn pack_expired(&self, k: PackKey, req: EraseRequest) -> Result<Vec<Ticket>, EraseError>;
    pub fn tenant(&self, tenant_tag: [u8; 16], req: EraseRequest) -> Result<Vec<Ticket>, EraseError>;         // [LÜCKE L-01] nicht in der SPEC
    pub fn escalate(&self, t: Ticket, level: EraseLevel) -> Result<(), EraseError>;
    pub fn open_tickets(&self) -> Result<Vec<TicketStatus>, EraseError>;
    pub fn status(&self, t: Ticket) -> Result<TicketStatus, EraseError>;
    pub fn receipt(&self, t: Ticket) -> Result<ErasureReceipt, EraseError>;                     // kanonische Bytes, keine Signatur (NZ-14)
    pub fn verify(&self, t: Ticket) -> Result<VerifyReport, EraseError>;                        // strukturell, Pflichtmessungen (a)–(d)
}
```

Vertrag: `Ok(Ticket)` bedeutet **durables Gate** (`Stage::Gated`) — danach nie wieder ein Fehler [SPEC §9.5 ER1] — oder auf einem vergifteten Handle `IntentRecorded` (§9.5.2). `purge_deadline` ist Pflichtparameter. Ein Ticket **endet** nur auf der zur Schutzklasse passenden Stufe; `Logical` allein ergibt `CompleteWithResidual{LogicalOnly}`. `Eraser::verify` und `receipt` liefern **kein** `Complete`, wenn eine Pflichtmessung nicht `Complete` ist (`INV-VER-1`).

### 5.6 Feed, Segmente, Checkpoint, Diagnose

```rust
impl Storage {
    pub fn consumers(&self) -> Consumers<'_>;      // register(name, from: DurablePosition) -> Consumer;  poll(max) -> Vec<FeedItem> | Err(Lagged); ack(DurablePosition)
    pub fn segments(&self) -> Segments<'_>;        // register(owner: Arc<dyn SegmentOwner>, manifest) -> SegmentRegistration; unregister (ledger-protokolliert)
    pub fn checkpoint(&self, dst: &Path, o: CheckpointOpts) -> Result<CheckpointManifest, CheckpointError>;
    pub fn restore(src: &Path, o: RestoreOpts) -> Result<RestoreHandle, RestoreError>;       // RestoreQuarantine bis Fencing + Reconcile
    pub fn instance_id(&self) -> InstanceId;  pub fn erase_epoch(&self) -> u64;  pub fn backup_mark(&self) -> BackupMark;
    pub fn health(&self) -> HealthReport;          // inhaltsfrei
}
pub enum FeedItem { Change { at: DurablePosition, ops: Vec<FeedOp> }, Redacted { at: DurablePosition, cluster: ClusterId },
                    Erased { at: DurablePosition, scope: ErasedScope }, Restricted { at: DurablePosition, cluster: ClusterId } }   // [SPEC §12.3]
```

`SegmentOwner` und `ExtensionBuilder`/`BlockPruner` folgen **[SPEC §12.1/12.2]** unverändert. Jeder Owner-Aufruf hat Deadline und Fencing-Generation; Callbacks laufen nie unter Kernsperren.

### 5.7 Fehlerhinweis-Vertrag

Jeder öffentliche Fehler hat stabilen Typ und `hint() -> &'static str` ohne Schlüssel/Nutzwerte (Z-18). Die Fassaden oberhalb (§13) ergänzen eine typisierte `suggested_call`.

---

## 6. Async-Fassade und Legacy-Adapter (`contextra-storage-bridge`)

### 6.1 `AsyncStorage` (permanent)

```rust
pub struct AsyncStorage { /* Arc<Storage>, Spawner, BlockingPool, Limits */ }
impl AsyncStorage {
    pub async fn open(cfg: Config, pool: PoolConfig) -> Result<(Self, OpenReport), Error>;
    pub async fn apply(&self, change: Change) -> Result<ApplyOutcome, ApplyError>;        // Change.idem ist Pflicht (sonst Usage-Fehler)
    pub async fn view_durable(&self) -> View;                                              // View ist Send, blockierende Methoden laufen im Pool
    pub fn feed(&self, name: &str, from: DurablePosition) -> impl Stream<Item = Result<FeedBatch, FeedError>> + Send;
    pub async fn eraser(&self) -> AsyncEraser;                                             // eigene urgent-Lane des Pools
    pub async fn wait_for(&self, pos: DurablePosition, deadline: Duration) -> Result<(), WaitError>;  // durable_stamp ≥ pos
}
```

| Vertrag | Regel |
|---|---|
| Cancellation | Wird der Future nach Übergabe verworfen, ist der Change möglicherweise durabel. Der Aufrufer wiederholt mit **demselben** `IdemToken`. |
| Pool | begrenzte Threads, begrenzte Queue; Überlauf ⇒ `Backpressure`; getrennte `urgent`-Lane. Kein Runtime-Aufruf im Kern. |
| Receipt-Weitergabe | Cancellation darf ein Erase-Receipt nie verschlucken; `AsyncEraser::erase_*` liefert das `Ticket` auch bei abgebrochenem Aufrufer über `open_tickets()` wieder. |
| Streams | `feed` hält keine `View`; Backpressure des Konsumenten verzögert nur das Polling, nie Redo-Retention über `max_lag_bytes` hinaus (dann `Lagged`). |

### 6.2 `LegacyKvAdapter` (befristet, Feature `legacy-kv`)

Implementiert `contextra_ports::StorageEngine` auf Basis von `AsyncStorage`, **ausschließlich** damit Crates in der Übergangszeit lauffähig bleiben, die noch nicht auf `RecordStore` umgestellt sind (voraussichtlich `graph_persist`, `cognition`, `agent`). Regeln [`docs/18-bridge.md` §2]:

| `StorageEngine` | Abbildung | Einschränkung |
|---|---|---|
| `get`, `get_at_seq` | `view_durable().get` / `view_at(seq)` | `seq` wird als `Stamp` der **aktuellen** Instanz interpretiert |
| `put/delete/commit` | Adapter-lokaler Change-Puffer je `TxId`, ein `apply` bei `commit` mit Token aus `(tx_id, adapter_epoch)` **zufällig persistiert** | `TxId` ist adapterlokal, nicht Stamp |
| `put_if_absent` | `Guard::Key{Absent}` | — |
| `delete_prefix` | `RangeDelete[prefix, succ(prefix))` | Spaces mit `Sealed` nur logisch relevant |
| `scan_prefix*` | `page` mit Obergrenze `MAX_SCAN_MERGE_ACCUMULATOR`; darüber `Err(Unsupported)` | kein unbeschränkter Vec |
| `get_tracked`, `scan_prefix_tracked` | **`Unsupported`** (`supports_ssi_tracking() = false`) | SSI nicht nachbildbar (§9.2) |
| `rollback_to_tx` | **`Unsupported`** (E-31) | §10 |
| `pin/unpin_checkpoint` | View-Lease/`Hold` (≤ 24 h) oder `Unsupported` | L-02 |
| `MergeOperator` | entfällt (E-29) | Read-Modify-Write über Version-Guard + Retry |

Schlüsselraum: Space `ctx.legacy.shared.v1` (`NonPersonal`, **Plain**-Cluster je Mandant, `tenant_tag` gesetzt). **Personenbezogene Werte werden abgewiesen** (E-32): der Adapter schreibt nie in `Personal`-Spaces. Das ist die Brandmauer gegen stille Herabstufung der Datenklasse (§4.7 SPEC).

Der Adapter wird mit **G-CUTOVER** entfernt. Er darf nicht als dauerhafte Kompatibilitätsschicht dokumentiert oder exportiert werden.

### 6.3 Reihenfolge der Umstellung

Da Adapter und `RecordStore` beide auf `AsyncStorage` ruhen, ist die Reihenfolge: `AsyncStorage` → `RecordStore` → Crate-für-Crate-Umstellung (§14). Der Legacy-Adapter entsteht nur, wenn das erste umzustellende Crate ihn braucht.

### 6.4 `KvBridgeStorage`

Der Port `KvBridgeStorage` (`get/put/commit`, Tier-2-Spillover des KV-Caches) bleibt bestehen; seine neue Implementierung ist ein `SegmentOwner` der Klasse `Rebuildable` oder `MixedPerClusterKey` (§8.4). Der KV-Cache ist laut SPEC **außerhalb** des Kerns (NZ-18); wer Prefix-Daten persistiert, legt sie als Segment mit `rewrite_without` ab (§12.6 SPEC).

---

## 7. Record-Schicht `contextra-records` (neu, Ring 1)

Besitzt: Space-Katalog, Schlüsselcodecs, Kopfformat, Scope-Auflösung, typisierten Schreib-/Lesezugriff, Guard-Aufzeichnung. **Sie führt keine Politik** (NZ-16): Mandanten-/Rechteprüfung der Aufrufer bleibt oben; die Schicht erzwingt nur, dass jede Cluster-Auflösung über den `ScopeResolver` läuft.

### 7.1 Space-Katalog

`SpaceOptions` sind unveränderlich, `DataClass::Personal` erzwingt `Sealed` + `Seal::Cluster`, und **Sealed-Cluster dürfen nur in `Seal::Cluster`-Spaces liegen, Plain-Cluster nur in `Seal::ByMode`-Spaces** [SPEC §5.2 `INV-SEAL-1`]. Je Familie existieren daher bis zu drei Varianten; die Record-Schicht wählt die Variante aus der `Protection` des aufgelösten Clusters:

| Variante | Namensschema | `data_class` | `seal` | Cluster |
|---|---|---|---|---|
| personal | `ctx.<familie>.personal.v<n>` | `Personal` | `Cluster` | nur `Sealed` |
| sealed | `ctx.<familie>.sealed.v<n>` | `NonPersonal` | `Cluster` | nur `Sealed` |
| plain | `ctx.<familie>.plain.v<n>` | `NonPersonal` | `ByMode` | nur `Plain` |

Beim `open` ruft die Schicht `Spaces::ensure` für alle benötigten Varianten (idempotent). Familien **[NEU]**:

| Familie | `clustered` | `weight_slots` | `history` | Inhalt / Bemerkung |
|---|---|---|---|---|
| `doc` | ja | 0 | `KeepUntilFloor` | Dokumentkopf (Quelle, Version, Gültigkeit, `state`) |
| `chunk` | ja | 2 | `KeepUntilFloor` | Kopf: Trust, Provenienz, Zeiten; Körper: Text. `weights[0]` = Token je Tokenizer-Familie 0, `weights[1]` = Familie 1 [SPEC §12.5] |
| `vec.<model_id>` | ja | 0 | `KeepLatest` | quantisierter Vektor im Körper; **Modellwechsel = neuer Space** [SPEC §12.6]; `value_separation_min` so, dass übliche Vektoren inline bleiben |
| `graph` | ja | 0 | `KeepUntilFloor` | Entitäten, Kanten, Hyperkanten, Community-Zuordnung als Schlüsselfamilien (§7.5) |
| `kv` | ja | 0 | `KeepLatest` | Anwender-KV (Abbildung von `KvDeleteMode`, §12.1) |
| `agent` | ja | 0 | `KeepUntilFloor` | Task-/Step-Zustand, DLQ; Schritt-Schlüssel mit `Absent`-Guard für Exactly-once |
| `audit` | ja | 0 | `AppendOnly` | clusterlokale Protokollkette (§11.4) |
| `work` | ja | 0 | `KeepLatest` | **`Volatile`**: flüchtiger Arbeitsbereich (Ablösung von `volatile_vault`); `Personal` nur im Prozessspeicher, kein Löschnachweis [SPEC §4.5] |
| `meta` | **nein** | 0 | `KeepLatest` | `NonPersonal`, **nur** `plain`: Sammlungsregister, Modellfingerprints, `dimension`, Tenant-Register; Cluster implizit `NONE` |
| `legacy` | ja | 0 | `KeepLatest` | **nur** `plain`, Altadapter (§6.2) |

Nicht als Space modelliert: Indexstrukturen (HNSW, BM25-Postings, CSR) — sie sind Segmente (§8); IdentityDirectory — Segment `MixedPerSubjectKey` (§7.6).

### 7.2 Scope-Modell: Domänenbegriff → Speicherbegriff

| Domänenbegriff | Speicherbegriff | Regel |
|---|---|---|
| **Mandant** (`TenantId`) | `tenant_tag` (16 Byte, **zufällig** bei Mandantenanlage vergeben, im `meta`-Space abgelegt) | Bestimmt zusammen mit `due` die Packgruppe. Mandant ist **kein** Cluster. Die Zuordnung `TenantId → tenant_tag` ist nicht aus der ID ableitbar. |
| **Person/Betroffene** | **Wurzelcluster**, `Sealed`, `personal`-Spaces | Muster [SPEC §12.7]; Auffinden über `IdentityDirectoryOwner` (§7.6). |
| **Fall/Arbeitskontext mehrerer Personen** | eigener Fallcluster (`Sealed`) | Verknüpfung zu Personen per `link` **nur**, wo die Löschung der Person den Fall mitziehen soll; sonst Referenz im Schlüsselraum [SPEC K-25, §12.8 Nr. 21]. |
| **Sammlung** (`CollectionId`) | Schlüsselpräfix `collection_id` **innerhalb** des besitzenden Clusters | Eigener Cluster je Sammlung nur bei `ErasureGranularity::PerCollection`. |
| **Dokument** (`DocId`) | Schlüsselpräfix `doc_id` | Eigener Cluster nur bei `ErasureGranularity::PerDocument` (§7.4). |
| **Agenten-Sitzung** | Schlüsselpräfix `sess/<id>/` oder TTL-Branch im Personencluster | **Session ≠ Cluster** [SPEC §12.6, K-35]. Cluster je Sitzung nur bei eigener Löschdomäne. |
| **Nicht-personenbezogener Korpus** | ein Cluster je `(tenant_tag, collection)` | `Plain` oder `Sealed` je Schutzbedarf; `Plain` verlangt `Physical`-Löschung (Barriere) und ist für Personenbezug unzulässig. |
| **Abgeleitetes** (Zusammenfassung, Konsolidat, Embedding-Cache) | derselbe Cluster wie die Quelle (bevorzugt) oder abgeleiteter Cluster mit `link` | Derivation-Cache nur in `Sealed`, Secret je Cluster [SPEC K-32]. |

### 7.3 `ScopeResolver` und die Mandantennaht

```rust
pub enum SubjectRef { Person(PersonKey), Case(CaseKey), Corpus { collection: CollectionId }, System }
pub struct ScopeBinding { /* privat konstruierbar */ }                  // trägt cluster, protection, tenant_tag, data_class
impl ScopeBinding { pub fn cluster(&self) -> ClusterId; pub fn protection(&self) -> Protection; pub fn tenant_tag(&self) -> &[u8; 16]; }

pub trait ScopeResolver: Send + Sync {
    fn resolve(&self, ctx: &RequestContext, s: &SubjectRef) -> Result<ScopeBinding, ScopeError>;           // NotFound ≠ Erased ≠ Restricted
    fn create(&self, ctx: &RequestContext, s: &SubjectRef, spec: ScopeSpec) -> Result<ScopeBinding, ScopeError>;
    fn enumerate(&self, ctx: &RequestContext, sel: EraseSelector) -> Result<Vec<ScopeBinding>, ScopeError>;  // für Löschaufträge
}
```

**Mandantennaht (`INV-INT-3`):** `resolve` vergleicht `Clusters::info(c).tenant_tag` mit dem Tag des `ctx.tenant_id` und liefert sonst `ScopeError::TenantMismatch`. `ScopeBinding` ist **nicht von außen konstruierbar** (Muster wie `LayerWitness` in `contextra-ports::scanner`; `compile_fail`-Doctest Pflicht). `RecordStore` akzeptiert nur `ScopeBinding`, nie eine rohe `ClusterId`. Damit ist Cross-Tenant-Zugriff strukturell an **einer** Stelle ausgeschlossen — das ersetzt `INV-TENANT-2` des `TenantKeyCodec`.

`TenantPolicy::Required` (Engine) bleibt als Policy oben; sie entscheidet, ob ein `SubjectRef::System`-Zugriff ohne Mandanten zulässig ist.

### 7.4 Löschgranularität bestimmt Clustergranularität

`erase_document` und `erase_collection` existieren im Altsystem als erste Klasse (B-07). Die neue Semantik hängt an der Cluster-Wahl, weil kryptografische Löschung nur je Cluster möglich ist:

| `ErasureGranularity` (je Sammlung, nach erstem Write unveränderlich) | Cluster | `erase_document` | `erase_collection` | Kosten/Grenzen |
|---|---|---|---|---|
| `PerSubject` (Default für Personenbezug) | Personenwurzel | `Eraser::range` über Dokumentpräfix: **`Sealed` ⇒ nur `Logical`** ⇒ Status höchstens `CompleteWithResidual{LogicalOnly}` | wie links über alle Dokumente | 1 Cluster je Person, 40 B Metadaten, MiniSection für kleine Bestände |
| `PerDocument` (opt-in) | Dokumentcluster, per `link` an die Wurzel | `Eraser::cluster` ⇒ `Cryptographic`, volle Kette ER0–ER7 | Kaskade über Wurzel | **Fanout-Grenze, siehe unten** |
| `PerCollection` | Cluster je `(tenant_tag, collection)` | wie `PerSubject` | `Eraser::cluster` | geeignet für NonPersonal-Korpora |

**Fanout-Regel `FANOUT-1` [SPEC §9.6, E-69].** Die transitive `DerivedFrom`-Hülle ist auf `max_gate_clusters = 4096` begrenzt, `max_link_depth = 8`, höchstens 8 Quellen je Cluster. Die Admission prüft für **jeden Vorfahren**, dass `|Hülle(a) ∪ Hülle(derived)| ≤ 4096`. Folge: Eine Personenwurzel kann **höchstens 4095** Dokument-/Ableitungscluster per `link` tragen. Darüber hinaus gilt:

- Dokumentcluster jenseits der Grenze werden **nicht** gelinkt, sondern im `docs_index` der Wurzel geführt.
- Personenlöschung wird dann als **orchestrierter Mehrfach-Auftrag** ausgeführt (`Eraser::clusters_batch`, Tickets ≤ 4096), **nicht atomar über alle Dokumente**. Das `SystemErasureReport` weist das als Status `Incomplete` bis zum letzten Ticket aus und nennt den Aggregatzustand.
- Konfigurationsprüfung: `PerDocument` wird beim Anlegen abgelehnt, wenn die erwartete Dokumentzahl je Wurzel (`expected_docs_per_subject`) 4000 übersteigt; sonst `Err(Limit{EraseFanout})` mit `hint()` auf `PerSubject`.

Rechenbeispiel für den Metadatenpreis: 10 Mio. Dokumentcluster ≈ 400 MB `ClusterIndex` (40 B) plus je ein MiniSection-Siegel (40 B) und eine Directory-Zeile; das liegt innerhalb der SPEC-Grenzen (16 Mio.), aber nahe am Budget (`cluster_meta_budget` = 15 % von `memory_budget_bytes`). `PerDocument` ist daher eine bewusste, budgetierte Entscheidung (`INV-ADM-1`).

### 7.5 Schlüsselcodecs

Binär, **big-endian**, ordnungserhaltend; ersetzen die ASCII-Schlüssel des Altsystems. Mandant und Cluster stehen **nicht** im Schlüssel (Routing über Cluster).

| Space-Familie | Schlüssel (Byte-Layout) |
|---|---|
| `doc` | `collection_id u64 ‖ doc_id[16]` |
| `chunk`, `vec.*` | `collection_id u64 ‖ doc_id[16] ‖ chunk_idx u32` |
| `graph` | `collection_id u64 ‖ tag u8 ‖ …` mit `tag ∈ {N entity: id u64, E edge: from u64 ‖ to u64, H hyperedge: id u64, I hyperedge_by_entity: entity u64 ‖ he u64, C community: level u8 ‖ id u64, Q cascade_queue: seq u64, X tombstone_marker: id u64}` |
| `kv` | `collection_id u64 ‖ user_key` |
| `agent` | `tag u8 ‖ id…` mit `S session: sess_id ‖ …`, `T task: task_id ‖ step u32` (final: `step = u32::MAX`), `D dlq: seq u64` |
| `meta` | `tag u8 ‖ name` |

**[ENTSCHEIDUNG] `DocId` wird immer als 16 Byte kodiert** (`u64` nullerweitert), unabhängig vom Feature `docid-128`. So ändert das Feature das On-Disk-Layout nicht; das Format wird mit G-F eingefroren. Preis: 8 Byte je Schlüssel.

Anforderungen an jeden Codec: injektiv, Roundtrip-Property-Test, Präfixnachfolger `succ(prefix)` für `RangeDelete`/Scans korrekt (inklusive Überlauf `0xFF…`), `max_key_bytes` (1 KiB) geprüft vor jeder Sperre (`INV-LIM-1`).

### 7.6 Kopfformat und Konventionen

Der Kern interpretiert den Kopf nie [SPEC §4.4]. Die Record-Schicht definiert **Kopf-Layout v1** **[NEU]**:

```
header := trust u8 | ver u8 | flags u16 | producer_id u32 | source_stamp u64
        | valid_from i64 | valid_to i64 | observed_at i64 | source_time i64 | family_tlv…
```

- Erstes Byte = `trust ∈ {system, user, tool_verified, tool_unverified, untrusted}` [SPEC §12.6]; Vererbung = Minimum über `DerivedFrom`-Quellen; Hochstufen nur privilegiert mit Audit-Eintrag.
- Gültigkeits-, Beobachtungs- und Quellzeit liegen im Kopf (K-26); Zeitprädikate dürfen Blöcke **nicht** über Kernfelder überspringen, nur über `BlockPruner` mit `PruneContract::ShadowSafe` (K-29, `INV-PRUNE-1`).
- `source_stamp` ist nur gemeinsam mit der `InstanceId` des Clusters gültig; die Record-Schicht legt je Cluster einen `meta`-Record `instance_epoch` ab und invalidiert Ableitungen nach Instanzwechsel.
- Derivation-Cache-Schlüssel: `MAC(cache_secret, model_id ‖ canonical_input)` mit **Zufallssecret je Sealed-Cluster** (K-32); `Plain`-Cluster erhalten keinen Cache.

### 7.7 `RecordStore`

```rust
pub trait RecordStore: Send + Sync + 'static {
    fn resolver(&self) -> &dyn ScopeResolver;
    fn builder(&self, ctx: &RequestContext) -> ChangeBuilder;                                       // synchron, reine Datenstruktur
    fn commit(&self, b: ChangeBuilder, idem: IdemToken) -> BoxFuture<'_, Result<ApplyOutcome, ApplyError>>;
    fn snapshot(&self, c: Consistency) -> BoxFuture<'_, Result<RecordView, ReadError>>;              // Consistency: §8.2
    fn tracked(&self, v: RecordView) -> TrackedView;                                                 // zeichnet Guards auf
    fn eraser(&self) -> &dyn AsyncEraser;
}
impl ChangeBuilder {
    pub fn put_doc(&mut self, s: &ScopeBinding, coll: CollectionId, doc: DocId, h: DocHeader) -> &mut Self;
    pub fn put_chunk(&mut self, s: &ScopeBinding, coll: CollectionId, doc: DocId, idx: u32, c: ChunkRecord) -> &mut Self;
    pub fn put_vector(&mut self, s: &ScopeBinding, model: &ModelId, coll: CollectionId, doc: DocId, idx: u32, v: QuantizedVector) -> &mut Self;
    pub fn put_graph(&mut self, s: &ScopeBinding, coll: CollectionId, op: GraphOp) -> &mut Self;
    pub fn delete_doc(&mut self, s: &ScopeBinding, coll: CollectionId, doc: DocId) -> &mut Self;  // RangeDelete über alle Familien, ein Change
    pub fn guard(&mut self, g: RecordGuard) -> &mut Self;
    pub fn build(self) -> Result<Change, BuildError>;     // prüft max_change_ops 4096, max_change_bytes 8 MiB, max_body 4 MiB, INV-SEAL-1, DataClass
}
```

`BuildError` entspricht den Vorprüfungen von §6.1 Schritt 1 und tritt **vor** jeder Sperre auf. `ChangeBuilder` setzt `Change.allow_mixed = false`; Mischchanges mit Volatile-Zielen sind nur über `work`-Hilfsfunktionen möglich und nie `durable` über einen Absturz (`INV-VOL-1`).

**`TrackedView`** ersetzt `get_tracked`/`scan_prefix_tracked`: `get` merkt `Version(stamp)` bzw. `Absent`; `page`/`cursor` merken `Guard::Range{since: view_stamp}` (Phantomschutz). `finish()` liefert `Vec<Guard>`; Überschreitung von `max_tracked_guards` (Nachfolger von `ssi_max_tracked_keys`) ergibt `Limit`, nie stilles Verwerfen.

### 7.8 `IdentityDirectoryOwner` (Modul `directory`)

Registrierter `SegmentOwner` der Klasse `MixedPerSubjectKey` [SPEC §12.7, §12.8 Nr. 9]. Schnittstelle:

```rust
pub trait IdentityDirectory: Send + Sync {
    fn lookup(&self, t: TenantTag, p: &PersonKey) -> Result<Option<ClusterId>, DirectoryError>;
    fn bind(&self, t: TenantTag, p: &PersonKey, c: ClusterId) -> Result<(), DirectoryError>;
    fn reverse(&self, t: TenantTag, c: ClusterId) -> Result<Option<DirectoryKeyHandle>, DirectoryError>;
    fn erase_subject(&self, t: TenantTag, c: ClusterId) -> Result<DirectoryAttestation, DirectoryError>;   // destroy_entry_key + rewrite_without + verify_absent
}
```

Schlüsselmaterial (`K_dir_tenant`, `K_reverse`, `K_keyid`, `EntryDEK`) liegt ausschließlich im KMS, nie im Storage oder in Backups. Die Schnittstelle zum KMS ist ein eigener Port `SubjectKeyService` **[NEU]**, den `keys-cohort`/HSM-Crates bedienen. Solange das HMAC-Label in retained Backups liegt und `K_dir_tenant` verfügbar ist, darf das Receipt nicht `Complete` behaupten (`CompleteWithResidual` mit Backup-Frist) [SPEC §12.7 Nr. 2]. Ein persistenter Kennung→Cluster-Pfad **außerhalb** dieses Owners ist verboten und verhindert systemweites `Complete` (`ext-conformance`).

---

## 8. Index-Schicht: Vector, Text, Graph als abgeleitete Segmente

### 8.1 Umbau der Domain-Ports (`contextra-ports`)

Die heutigen Ports `VectorIndex`/`TextIndex`/`GraphIndex` tragen Transaktionssemantik (B-06). Zielbild **[NEU]**: die Index-Crates bleiben von `contextra-storage` **unabhängig**; sie sehen nur Ports und den Sichtbarkeits-Orakel-Port.

```rust
pub trait VisibilityOracle: Send + Sync {                                   // implementiert von contextra-records über View::filter_visible
    fn position(&self) -> DurablePosition;
    fn filter_visible(&self, refs: &[RecordRef]) -> Result<Vec<Visible>, VisibilityError>;
}
pub struct QueryScope { pub partition: PartitionKey /* Mandant */, pub clusters: ClusterSelector /* Alle | Menge | Branch */ }

pub trait DerivedIndex: Send + Sync + 'static {
    fn kind(&self) -> IndexKind;                                            // Vector | Text | Graph
    fn watermark(&self) -> DurablePosition;                                 // lückenlos angewandt (SegmentOwner::watermark)
    fn apply(&self, batch: &FeedBatch) -> Result<ApplyReport, IndexError>;  // idempotent, mindestens-einmal, nur ≤ durable_stamp
    fn on_gate(&self, erased: &[ClusterId], at: DurablePosition, gen: u64); // synchroner, generationsgefencter Tombstone-Zustand
    fn rebuild(&self, from: &dyn RebuildSource, scope: RebuildScope) -> Result<(), IndexError>;   // nach Lagged oder Instanzwechsel
    fn stats(&self) -> IndexStats;
}
pub trait VectorSearch: DerivedIndex {
    fn search(&self, q: &VectorQuery, s: &QueryScope, v: &dyn VisibilityOracle) -> Result<Vec<Candidate>, IndexError>;
}
pub trait TextSearch: DerivedIndex {
    fn search(&self, q: &TextQuery, s: &QueryScope, v: &dyn VisibilityOracle) -> Result<Vec<Candidate>, IndexError>;
}
pub trait GraphSearch: DerivedIndex {
    fn expand(&self, q: &GraphQuery, s: &QueryScope, v: &dyn VisibilityOracle) -> Result<Vec<GraphHit>, IndexError>;
}
pub enum IndexError { IndexLag { applied: DurablePosition, wanted: DurablePosition }, Rebuilding, Capacity(&'static str), Gated, Corrupt, /* … */ }
```

Entfallen: `insert(tx,…)`, `delete(tx,…)`, `commit(tx)`, `rollback(tx)`, `rollback_to_tx`, `last_tx_id`. `search_at(seq)` wird durch den `VisibilityOracle` (Sichtposition) ersetzt. `FeedOp` trägt die Mutationen (`UpsertVector`, `DeleteVector`, `UpsertText`, `DeleteText`, `GraphDelta`, `ClusterErased`).

Weiter gültig bleiben `stats`, `len`, die Embedding-/Reranker-Ports und `CommunityResolver`.

### 8.2 Watermark- und Konsistenzprotokoll [SPEC §12.4, §12.4.2]

1. Der Konsument liest den Feed (`Consumers::poll`), wendet `FeedBatch` **idempotent** an und rückt `watermark` erst nach vollständig angewandtem, **durablem** Delta vor (`SegmentOwner::watermark` darf nie über `durable_stamp` hinausreichen).
2. Eine Abfrage bindet `view_durable()` → Position `V`. Der Index durchsucht **Basis ∪ Delta `(applied, V]`**; ist die Lücke nicht schließbar, liefert er `IndexLag` oder einen **expliziten** Fallback (Brute-Force über Delta), nie stillschweigend unvollständige Treffer.
3. Mandanten-/Cluster-Filter wirken **vor** der Exploration (`QueryScope`); der Gate-Zustand wird vor Distanzberechnung/Scoring/Graph-Expansion je Kandidat geprüft; jedes Ergebnis passiert danach `filter_visible` (`INV-VIS-1`).
4. Konsistenzstufen der Fassade **[NEU]**:

```rust
pub enum Consistency { Durable, AtLeast { pos: DurablePosition, deadline: Duration } }
```

`Durable` liest `view_durable()` und akzeptiert Indexlag als `IndexLag`. `AtLeast` wartet bis `watermark ≥ pos` (Read-your-writes nach `Receipt`) oder liefert `IndexLag` nach der Frist.
5. Instanzwechsel (Restore, Migration) ⇒ vollständiger `rebuild`; Vergleiche über `InstanceId`-Grenzen sind verboten.

### 8.3 Persistenz der Indexe nur über `SegmentIo`

Segmente liegen unter `segments/<kind>/<id>.seg`, sind im Catalog registriert (`segment_add/remove`) und werden **nur** über das versiegelte `SegmentIo` gelesen/geschrieben [SPEC §12.1]. Direkte Dateizugriffe eines Owners sind verboten. Jeder RAM-/Scratch-Byte wird vor der Allokation über `MemoryBudget` reserviert (Unterbudgets *Extension-/Indexdelta* 20 %, *In-flight Rewrite* 10 % [SPEC §16]).

Das betrifft **`contextra-vector`**: dessen eigene Persistenz (`crate::persistence::MmapIndex`, B-15) muss hinter `SegmentIo` verlegt oder für personenbezogene Partitionen abgeschaltet werden (**L-12**; nicht im Detail geprüft). `mmap` bleibt Sache der `Fs`-Implementierung (K-11, NZ-9).

### 8.4 Owner-Tabelle

| Owner | `SegmentClass` | Löschgranularität | Rewrite-Obergrenze | Attest |
|---|---|---|---|---|
| Vector (HNSW/DiskANN je Mandantenpartition) | `Rebuildable{max_age, max_rewrite_bytes}` | Element → Quellcluster+Generation (Pflichtzuordnung, §12.4.1) | je Partition; über Grenze/Deadline ⇒ `Blocked`/`Incomplete` | `EraseAttestation` mit `verify_absent` über **alle** Elemente und Nachbarlisten (Kanten auf Tombstones werden entfernt) |
| Vector-Mikroindex (clusterlokal, optional für kleine Personencluster) | `ClusterSections` | Cluster | trivial (`rewrite_without`) | Kernattest genügt |
| Text (BM25-Postings je Partition) | `Rebuildable` | Posting → Quellcluster | je Partition | `verify_absent` über Postinglisten und Termwörterbuch-Referenzen |
| Graph (CSR, Communities) | `Rebuildable` | Kante/Knoten → Quellcluster | je Partition | `verify_absent`; Communities über **mehrere Personen** sind für `Personal`-Daten unzulässig, außer als `Rebuildable` mit Lineage |
| KV-/Prompt-Cache-Segmente | `MixedPerClusterKey` oder `NoPersonalData` mit Provenienzbeleg | Cluster/Epoche | — | Attest je Epoche; `NoPersonalData` bekommt **keine** Crypto-Erasure-Zusage |
| Identity-Directory | `MixedPerSubjectKey` | Subjekt | — | `destroy_entry_key` + `rewrite_without` + `verify_absent` |
| Checkpoint-Owner (`contextra-checkpoint`) | Art `Backup` | Kopie | — | Referenzen auf Extents entfernt/attestiert, sonst Restexposition im Receipt |

Ein globaler HNSW-/CSR-Neubau ist **kein** O(Cluster)-Löschversprechen [SPEC §12.4.1]; Kosten und Fristen werden getrennt ausgewiesen. Die Empfehlung der SPEC (V1-Referenzarchitektur) gilt: Vektoren und kleine Ranking-Metadaten clusterlokal versiegelt, tenantweite ANN/FTS/Graph-Indizes als partitionierte, begrenzte `Rebuildable`-Derivate mit durablem Delta und Gate-Filter.

### 8.5 `on_gate` und Gate-Maske

`on_gate` ist eine **Beschleunigung**, nie die Korrektheitsbarriere (jede Abfrage prüft den Kern-Gate-Zustand unabhängig). Empfohlene Struktur: ein `roaring`-Bitmap der gesperrten Cluster-Handles je Partition (`roaring` ist bereits Workspace-Abhängigkeit), generationsgefenct; die Prüfung erfolgt in der Distanz-/Scoring-Schleife vor jeder Berechnung. Cluster-Handles sind index-lokal internierte `u32` zu `ClusterId`; die Internierungstabelle gehört zum Segment und wird beim `rewrite_without` bereinigt.

---

## 9. Engine, Transaktionen, Suche, Löschkoordination (`contextra-engine`)

### 9.1 `IngestUnit` ersetzt `DbTransaction`

Der Kern liefert Atomarität über alle Spaces **eines** Changes; Indexe sind nicht Teil davon (ID-5). Der 2PC mit `CommitIntent`, Staging je Index und kompensierenden Aktionen entfällt:

| Altes Element (B-06) | Neues Element |
|---|---|
| `CommitIntent::Pending/Committed/Aborted/Failed` unter `__tx_intent:` | entfällt; Durabilität = `Receipt` (`INV-COMMIT-2`) |
| `stages_completed` (LSM durable → Indexe) | entfällt; Indexe holen über Feed nach, Watermark belegt Fortschritt |
| `CompensateLsmAction` / `CompensateHnswAction` / `CompensateTextAction` / `CompensateGraphAction` | entfällt für den LSM-Teil; Index-Korrektur = idempotentes `apply`/`rebuild` |
| `TxBuffer`-Staging, Orphan-Reaping | `ChangeBuilder` (clientseitig, kein Server-Zustand) |
| `rollback(tx)` | `ChangeBuilder` verwerfen (vor `commit` nichts persistiert) oder `branch_drop` bei Branch-Arbeit |
| `CommitLedger` | `Receipt` + Idempotenz-Record (`SYS_IDEM`) |

Ablauf `IngestUnit::commit` **[NEU]**: (1) Eingabe validieren, `DataClass` je Sammlung bestimmt Space-Variante (§7.1); (2) Scope auflösen (`ScopeResolver`); (3) Dokument chunken/einbetten **außerhalb** des Commits (Rechenarbeit, kein Lock); (4) `ChangeBuilder` mit `doc`/`chunk`/`vec`/`graph`-Ops und Guards füllen; (5) `commit(idem)`; (6) `Receipt.position` an den Aufrufer, optional `Consistency::AtLeast` für Read-your-writes.

### 9.2 Guard-Katalog (ersetzt SSI-ReadSet)

Der Altkern validiert per SSI (`ReadSet`, `SsiValidator`); der Neukern bietet Snapshot-Isolation plus **explizite** Guards (NZ-3). Schreibschiefe (*write skew*) verhindert nur eine **vollständige Guard-Modellierung**. Verbindlicher Katalog [SPEC §6.3, erweitert]:

| Operation | Guards | Anmerkung |
|---|---|---|
| Dokument neu anlegen | `Key{doc, Absent}` | Eindeutigkeit der `DocId` |
| Dokument aktualisieren | `Key{doc, Version(v)}` (+ `Range` über Chunks, wenn Chunk-Anzahl sich ändert) | Phantomschutz über `chunk`-Präfix |
| Dokument löschen | `Key{doc, Version(v)}` | `RangeDelete` je Familie in **einem** Change |
| Kante/Relation anlegen | `Key{edge, Absent}` | idempotent bei Wiederholung |
| Zähler/aggregierte Werte | `Key{k, Version(v)}` + Retry | ersetzt `MergeOperator` (§9.4) |
| Agenten-Schritt (Exactly-once) | `Key{task/step, Absent}` + `IdemToken` | Tool-Zustand: `IdemToken` + `Version` |
| Konsolidierung (Sleep-Cycle) | `Unchanged{since: merge_base}` je Schlüssel, bzw. Branch-Merge | §9.3 |
| Lesen-dann-Schreiben über Bereich | `Range{start,end,since: view_stamp}` | Pflicht für jeden Bereichsscan, dessen Ergebnis ein Write verwendet |

`TrackedView` (§7.7) zeichnet die Guards für lesende Pfade automatisch auf. **Regel `GUARD-1`:** Jede Lesung, die einen Write beeinflusst, läuft über `TrackedView` oder trägt explizite Guards; ungetrackte Lesungen sind nur für rein lesende Pfade zulässig.

### 9.3 Wegfall der Protokollartefakte, Branch-Muster für Konsolidierung

Die Schlüssel `__tx_intent:*`, `__maintenance_intent:*`, `consolidation_intent:*` sind **Artefakte des 2PC/OCC** und werden nicht migriert (§14). Die Konsolidierung (`contextra-cognition`) läuft stattdessen als **spekulativer TTL-Branch** im Cluster: Fork bei `Head`, Konsolidierung auf dem Branch, `merge(source→target, FailOnConflict)` mit Idempotenz-Token; Konflikte erscheinen als `ConflictCell`-Liste (Daten der Aufrufer-Schicht), bei Fehlschlag `branch_drop`. Das erfüllt „Fehler gehen nicht verloren, letzter guter Stand wiederherstellbar“ ohne Rückrollen der Datenbank.

Grenzen, die der Konsolidierungs-Planer beachten muss: Merge nur direkt Child → Parent; Source mit lebenden Kindern nicht mergbar; `max_branch_depth` 4 (hart 8); `max_branches_per_cluster` 1024; ein gemergter Branch ist terminal.

### 9.4 `MergeOperator` entfällt

Kommutative Aktualisierungen (Zähler, Mengen-Vereinigung) werden als Read-Modify-Write mit `Version`-Guard und begrenztem Retry (`guard_retry_max` 3, dann `Conflict{contended: true}`) umgesetzt. Hot-Counter mit hoher Konfliktrate sind **kein** Kern-Anwendungsfall; sie werden als append-only Delta-Records (`counter/<id>/<stamp>`) modelliert und beim Lesen aggregiert (`page`), mit periodischer Kompaktierung durch einen Wartungs-Job.

### 9.5 Große Dokumente: Commit-Marker-Muster

Ein Change ist auf `max_change_ops` 4096 und `max_change_bytes` 8 MiB (Körper ≤ 4 MiB) begrenzt. Passt ein Dokument mit Chunks, Vektoren und Graph-Deltas nicht hinein, gilt:

1. Teil-Changes schreiben Chunks/Vektoren/Graph-Deltas mit `doc.state = Staging`.
2. Der **letzte** Change setzt `doc.state = Live` mit `Version`-Guard auf den `doc`-Record.
3. Lesepfade und Index-Konsumenten ignorieren `Staging`-Dokumente.
4. Ein Wartungs-Job entfernt `Staging`-Dokumente älter als `staging_ttl` (Crash-Reste).
5. Das Löschen eines `Staging`-Dokuments ist ein normaler `delete_doc`.

Die Atomarität gilt nur **je Change**; die Dokumentebene ist durch den Marker linearisiert.

### 9.6 Suchpfad (normativ)

1. `ScopeResolver::resolve` für jedes angefragte Subjekt (Mandantennaht, `INV-INT-3`); `Erased`/`Restricted` werden als Ergebnis **ausgewiesen**, nie als „leer“ gemeldet.
2. `RecordStore::snapshot(Consistency)` → `view_durable()` bzw. `AtLeast`.
3. Index-Abfragen mit `QueryScope` (Vorfilter) und `VisibilityOracle`; Fehler `IndexLag` → Fallback oder expliziter Fehler laut Anfrageprofil.
4. Fusion/Ranking (`contextra-rank`) auf Kandidaten **nach** `filter_visible`.
5. Hydratisierung per `get`/`page` mit Token-Budget (`weights`), Körper erst bei Bedarf (`body`).
6. Erneute Gate-Prüfung vor Ausgabe (Generation-Check), `Zeroizing`-Puffer für Körper.
7. Antwort trägt `DurablePosition` der verwendeten Sicht (Beleg für Zeitreise-/Audit-Fragen, „Was wusste der Agent bei S?“ = `view_at(S)`).

### 9.7 Löschkoordination (`ErasureCoordinator`, ersetzt `ErasureOrchestrator`)

Der Kern erledigt Gate, Schlüssel und Purge; die Engine koordiniert **Owner**, nicht mehr Scanner:

| Auftrag | Kernaufruf | Owner-Schritt | Besonderheit |
|---|---|---|---|
| Person löschen | `Eraser::cluster(root, Cryptographic)` (Kaskade über Links) | `IdentityDirectory::erase_subject` | `FANOUT-1`: bei > 4095 Dokumentclustern Zusatztickets, Status bis zum letzten Ticket `Incomplete` |
| Dokument löschen | `PerDocument`: `cluster`; sonst `range` (`Sealed` ⇒ `Logical`) | Indexe via `on_gate`/Feed `Erased` | `Logical`-Ende ⇒ `CompleteWithResidual{LogicalOnly}` |
| Sammlung löschen | `PerCollection`: `cluster`; sonst `range` über Sammlungspräfix | wie oben | — |
| Mandant verlassen | `Eraser::tenant` **(L-01)** bzw. `enumerate` + `clusters_batch` | alle Owner der Partition | deterministische Tickets ≤ 4096 je Frist |
| Frist abgelaufen | `Eraser::pack_expired(PackKey)` | wie oben | Restricted-Cluster separat im Aggregat |

Ergebnis: `SystemErasureReport { tickets, owner_attestations, status, caps }`. `Complete` verlangt jede erforderliche Kern- **und** Owner-Attestation; fehlt eine, ist der Status `Incomplete`, nie `Complete` [SPEC §12.8 Nr. 10, §9.7]. Reihenfolge der Altlogik (Text → Vector → LSM) entfällt; die neue Ordnung ist Gate → Schlüssel → Purge → Owner-Rewrite → Verify.

### 9.8 Hintergrund-Worker

| Worker (Altsystem) | Neu |
|---|---|
| Expiry-Reaper | setzt `ClusterSpec.due` und ruft `pack_expired`; Fristen aus `purge_deadline`, nie aus Wanduhr-Vergleichen (`ClockUncertain` friert ein) |
| Orphan-Worker (`InstanceOrphanRegistry`) | Kern räumt Schlüssel-Waisen nach `orphan_key_grace` 24 h; verbleibende Worker beschränken sich auf Anwendungs-Waisen (`Staging`-Dokumente, verwaiste Kanten) |
| Hyperedge-Worker, Community-Detection | Konsument des Feeds; Ergebnis als normaler `Change` mit Producer im Kopf |
| Decay-Controller („Vergessen“) | Gewichts-/Zoom-Update als `Change`; **Entfernung** nur über `Eraser` (keine stille Entfernung, SPEC §12.6) |
| Wartungsplaner (Kognition) | nutzt Governor-Signale statt `SystemPressure`; Lasten über `MemoryBudget` |

