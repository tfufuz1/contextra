# Lethe – Finale Spezifikation v1.0

**Speicherkern mit nachweisbarer Löschung für LLM-/Agenten-/RAG-Workloads**
**Status:** Final (Merge) · **Datum:** 23.11.2026 · **Sprache:** Rust (Edition 2021+, `#![forbid(unsafe_code)]` im Kern)
**Zusammengeführte Quellen:** (A) `lethe-storage-engine-spec.md`, (B) „Erasure-Native Storage Core – Architekturentwurf", (C) `MNEMOS_CONCEPT.md`

> **Leitsatz:** Löschen ist die primäre Strukturierungsachse des Layouts. Löschdomäne, Schlüsseldomäne und Platzierungseinheit sind identisch (die **Erasure Unit, EU**). Alles, was LLM-/RAG-spezifisch ist (Token-Budgets, Zoom, Recall, Branches, Trust), baut auf dieser Achse auf und darf sie nie verletzen.

---

## 0. Ergebnis in Kürze

1. **Kern = Lethe (A) + Scrub-/Ticket-/Notbetriebsmodell aus B.** Ein löschfreundlicher LSM-/MVCC-Speicher mit EU-reinen Sektionen, positionsunabhängiger AEAD-Versiegelung, Ledger + Rollback-Anker.
2. **Mnemos (C) wird zur Schicht *über* dem Kern**, nicht daneben: Cells, Zoom, Token-Budget-Recall, Branches, Handles, Trust, Work Queue, MCP. Was in C mit der Löschgarantie kollidierte (globales Content-Addressing, Prolly-Tree-/Merkle-DAG als Primärzustand, globales gelerntes Layout, Index im Commit-Wurzelobjekt), wird in §1 explizit ersetzt.
3. **Sub-EU-Löschung ist nie kryptografisch.** Der Kern behauptet Nachweisbarkeit nur auf EU-Ebene. Alles darunter (Tombstones, Zoom-Down, Retention) ist logisch plus Compaction und wird so ausgewiesen.
4. Der Kern kennt weiterhin keine Tokenizer, Embeddings, ANN oder Personen (D9). Er liefert Mechanismus; Politik liegt oben.

---

## 1. Konfliktauflösung (zentrales Ergebnis)

Jede Zeile nennt, was die Quellen sagen, die Entscheidung und den Grund.

| # | Thema | A (Lethe) | B (Erasure Core) | C (Mnemos) | **Entscheidung** | Begründung |
|---|---|---|---|---|---|---|
| K1 | Name der Löscheinheit | EU | Cluster | Cell/Namespace | **EU** (Alias „Cluster" in B-Dokumenten). Cell ist eine Datenmodell-Idee der Ctx-Schicht, keine Löscheinheit. | Eine Löscheinheit muss genau einen Schlüssel besitzen. Cells wären zu fein. |
| K2 | Primärzustand | LSM + MVCC, lineare `Seq` | LSM + MVCC, `Stamp` | Prolly-Tree + Commit-DAG, content-addressed | **LSM + MVCC mit linearer `Seq`.** Hash-Verkettung nur im WAL und im Ledger. | Globales Content-Addressing und strukturelles Sharing über EUs hinweg widersprechen EU-Reinheit und Schlüsselvernichtung. Merkle-Audit wird durch Ledger + WAL-Kette ersetzt. |
| K3 | Branches/Forks | – | – | O(1)-Fork per Ref | **Branch = EU-interner Overlay** (`branch_id` im Schlüsseltupel, `SYS_BRANCH`-Tabelle, `RetentionPin` auf `fork_seq`). Kein Branch kreuzt EUs. | Forks bleiben O(1) und werden mit der EU gelöscht. Kein geteilter Zustand zwischen EUs. |
| K4 | Schlüsselgranularität | Cohort-KEK → EU-DEK | Schlüssel pro Cluster+Epoche (KMS pro Cluster oder hierarchisch) | Per-Subject-Keys | **Cohort-KEK (≤ 1024 EUs) → EU-DEK je Epoche.** Cohort-Größe 1 ist der Sonderfall „KMS-Objekt pro EU". Per-Subject = „Subject ist eine EU bzw. Deletion Group". | Skaliert auf Millionen EUs, Löschung kostet ≤ 1024 Wraps + 2 KMS-Calls. B und C sind Spezialfälle. |
| K5 | Schlüsselepochen | Ein Master-DEK | Epochen für Rotation | – | **Epochen aus B**, Epochenliste im Keyring-Eintrag. Erase vernichtet immer alle Epochen. | Rotation ohne Neuverschlüsselung ist nützlich, ändert die Löschlogik nicht. |
| K6 | Klartext-Identität im Frame | `eu_tag` = HMAC-Pseudonym | `ClusterId` (zufällig) im Sektionsheader/Directory | Hash des Klartexts als Chunk-ID | **`EuId` zufällig (128 Bit, nie aus Personendaten abgeleitet), im Klartext nur `eu_tag`** (HMAC mit `tenant_tag_key`, pro Tenant **kollisionsfrei** vergeben). | Schlüsselloser Purge braucht Klartext-Routing. `EuId` selbst bleibt versiegelt. |
| K7 | Physische Löschung | Verbatim-Rewrite der Überlebenden, O(Partitionsgröße) | In-place-Scrub, O(EU-Größe), braucht keinen Platz | GC/Compaction | **Primär In-place-Scrub (B)**, **Purge-Rewrite (A)** als Fallback (Dateisysteme ohne verlässliches Überschreiben, Platzrückgewinnung, Partition-Split). | Scrub ist unter ENOSPC lauffähig und proportional zur EU. Rewrite bleibt als portable Rückfalloption. |
| K8 | Commit-Serialisierung | Single-Sequencer-Thread, kein Lock | Flat-Combining-Leader unter `commit_lock` | Sequencer-Actor pro Branch | **Single-Sequencer-Thread (A).** | Deterministisch simulierbar, kein Lock über WAL-Append. Krypto und KMS laufen in `prepare`. |
| K9 | Frame-Header | 48 B, rechnerisch inkonsistent (siehe Anhang B) | `SealFrame` mit `epoch`, `local_ord` | – | **64-Byte-Header** mit `eu_tag`, `epoch`, `local_ord`, `body_crc`, `nonce`, `hdr_crc` (§5.2). | Beide Feldmengen werden gebraucht. |
| K10 | AAD | enthält `version` | enthält `local_ord`, `store_uuid` | – | **AAD ohne `Seq`**, mit `store_uuid ‖ kind ‖ eu_id ‖ epoch ‖ space ‖ local_ord ‖ record_id`. | `Seq` wird erst im Sequencer vergeben, versiegelt wird aber in `prepare` (siehe Anhang B). |
| K11 | Run-Layout | HeadBlock-Frames + SectionIndex | Meta/Idx/Hdr/Body-Frames pro Block | Extent mit Hot-Head/Cold-Tail | **Sektion = Meta · Idx\* · Hdr\* · Body\*** (spaltenweise zusammenhängend: Hot-Bereich vorn, Cold-Bereich hinten). | Realisiert Hot-Head/Cold-Tail (C) innerhalb der EU-Sektion; ein zusammenhängender Read für Ranking/Paging. |
| K12 | Sektionsverzeichnis | CRC + `dir_blake3` | `dead`-Flag, ID genullt | – | **Pro Eintrag CRC32C, kein Gesamt-Hash**, `dead`-Flag, Tag wird beim Scrub genullt. | Ein Gesamt-Hash verträgt keinen In-place-Scrub. |
| K13 | Scrub-Fortschritt | Fixpunkt-Verifier | `ScrubPending{run, cluster}` im Katalog | – | **Kein EU-Bezug im Manifest.** Fortschritt steht im Ledger (`StageIntent/PurgeProgress`, nur Datei-IDs + Tag-Digest). Recovery macht Tag-Scan-Fixpunkt. | I4 (Manifest ist EU-frei) bleibt gültig. |
| K14 | Abhängigkeiten/Kaskade | `SYS_DERIV` sealed, `OnSourceErasure` | `LinkTable` im Katalog, `DerivedFrom`/`Reference` | `derived_from`-Lineage | **Sealed `SYS_DERIV` in der Quell-EU** (Reverse-Kanten) + `kind ∈ {DerivedFrom, Reference}` und `on_source_erasure ∈ {Delete, Invalidate}`; Azyklizität beim Anlegen (B). Closure wird **vor** Schlüsselvernichtung berechnet. | Ein Klartext-Link-Katalog würde den Beziehungsgraphen der EUs leaken. |
| K15 | Extension-Vertrag | `SegmentOwner::on_erasure`, `ExtStore`, `SegmentClass` | `DerivedSegment::{on_gate, purge, verify_absent, watermark}`, `ChangeFeed` | Indizes im `RootSet` | **Vereinigung:** `SegmentOwner` mit `on_gate` (sofort, ohne I/O), `on_erasure` (mit Deadline), `verify_absent`, `watermark`, `segments`. Persistenz nur über sealed `ExtStore`. | Sofortsichtbarkeit (B) + Fristenmodell und Klassen (A). |
| K16 | Indizes und Atomarität | Extensions, nicht im Kern | Extensions | Vektor/FTS/Graph im Commit-Wurzelobjekt | **Indizes sind Extensions** (Crates `lethe-idx-*`). Segmente werden per Manifest-Edit sichtbar; jede Antwort läuft durch `filter_visible`; ein In-Memory-Delta-Index sichert „kein Index-Lag". | Ein zentrales Wurzelobjekt über alle EUs verträgt keine EU-Löschung ohne Neuschreiben. |
| K17 | Hold/Restriction | Authority-signiert, Vier-Augen, 3 Durchsetzungsschichten | `restrict`/`lift` ohne Signatur, nur `Err(Restricted)` | – | **A.** B-Modi werden abgebildet: `NoErase` = `LegalHold`, `NoErase+NoRead` = `Restriction`. Löschauftrag auf gehaltene EU wird **angenommen und `Blocked`** (kein verlorener Auftrag). `until` ist nur `review_at`, kein Auto-Release. | Stärkere Zusicherung. Ein abgelehnter Löschauftrag wäre ein Compliance-Risiko. |
| K18 | Erase-Zustandsautomat | 12 Zustände | `Gated/KeysDestroyed/Purged/Completed` | – | **A-Automat**, nach außen mit **Garantiestufen G0/G1/G2 (B)** und Ticket-Stufen. | Intern genau, extern lesbar. |
| K19 | Ledger-Identität | `eu_tag`s + Digests | `tomb_id = HMAC(ledger_key, ClusterId)` | – | **Beides:** `tomb_id` (No-Resurrection, `EuId`-Wiederverwendung abweisen) und `eu_tag` (Restore-Reconcile per Tag-Scan). | Restore braucht den Tag, Neuanlage-Schutz braucht die `tomb_id`. |
| K20 | Anchor-Zustand | `{ledger_seq, ledger_head, erasure_watermark}` | `{counter, ledger_head}` + `strict/best_effort` | – | **A-Zustand + `strict/best_effort`-Policy aus B.** | Watermark erlaubt Restore-Erkennung ohne Ledger-Vollscan. |
| K21 | Backup/Checkpoint | `begin_backup`, getrennte Keyring-Kette | `checkpoint/restore`, Ledger muss Präfix sein | Git-artiger Sync | **Beides verschmolzen** (§9.8). Harte Links werden mitgescrubbt (Löschung gewinnt). | Ergänzen sich. |
| K22 | Snapshot-Lebensdauer | `max_snapshot_age` 10 min | `View`-Lease 60 s | Zeitreise, Branch-Basen | **Lease 60 s (verlängerbar), harte Obergrenze 10 min.** Langlebige Basen (Branches, Zeitreise) sind `RetentionPin` mit TTL, kein Reader-Snapshot. | Reader-Pins und Retention-Pins haben verschiedene Kosten. |
| K23 | Flüchtiger Speicher | `Ephemeral` Space mit Spill-Datei | `Volatile` Region | `volatile` Durability | **`SpaceKind::Ephemeral` (A)**, Gate = sofortiges Verwerfen + Zeroize. | Identisch gemeint. |
| K24 | Learned Layout | Platzierung per Deletion Group | – | Co-Retrieval-Re-Layout, global | **Nur innerhalb einer Deletion Group/Partition**; Access-Trace ist EU-gesiegelt oder nur im RAM, wird beim Erase verworfen. Nie Co-Location über Deletion Groups. | Sonst entsteht Löschamplifikation und ein PII-Seitenkanal. |
| K25 | Dedup/Derivation Cache | – | – | tenantweit, hashbasiert | **Pro EU** (`SYS_DCACHE`, sealed), Schlüssel `HMAC(eu_dedup_key, content_hash ‖ producer)`. Kein EU-übergreifendes Dedup. | Löschbarkeit und Seitenkanäle. |
| K26 | Zoom-Down/Retention | – | – | gestuftes Vergessen inkl. „crypto-shred Original" | **Logisch + Compaction.** Kryptografisch nur, wenn das Original in einer eigenen EU liegt (Referenz-Link). | Granularitätsgrenze (§4.4). |
| K27 | Handles (Kurz-IDs) | – | – | pro Branch eindeutig | **Pro EU eindeutig, monoton vergeben** (`SYS_ALIAS`), nicht pro Branch. | Merges können nie kollidieren. |
| K28 | Zugriffskontrolle | Authority/Signer (Holds) | – | Macaroon-Capabilities | **Beides:** Authority für Governance (Hold, `LegalAccessCap`), Capabilities für Datenzugriff; `LegalAccessCap` ist eine Capability, signiert von der Authority. | Getrennte Verantwortungen, ein Token-Format. |
| K29 | Defaults | WAL 64 MiB, inline 1 KiB | WAL 16 MiB, Blob-Schwelle 16 KiB | Extent 256 KiB–1 MiB | **WAL 64 MiB, `inline_max` 4 KiB (per P3-Benchmark kalibrieren), Idx-Block 16 KiB** (§15). | Startwerte, begründet in §15. |
| K30 | Mehrere Tokenizer | `Weights{tokens,bytes,extra[2]}` | `weight: u32`, `lanes: [i64;4]` | Zählung pro Tokenizer-Familie | **`Weights{tokens,bytes,extra[2]}` + `lanes:[i64;4]`.** `extra[0..1]` = weitere Tokenizer-Familien (Zuordnung in der Space-Policy), `lanes` = Bitemporalität und Zeitfilter. | Deckt alle drei ab ohne Tokenizer im Kern. |

---

## 2. Architektur

### 2.1 Schichten

```
┌────────────────────────────────────────────────────────────────────────────┐
│ L6  lethe-mcp / lethe-server / bindings    MCP, gRPC/HTTP, Python/TS          │  außerhalb Kern
│ L5  lethe-ctx      Cells, Zoom, recall/window/expand/explain, Zoom-Knapsack,  │  Schicht „Mnemos"
│                    Handles, Trust, Work Queue, Retention, Derivation Cache    │
│ L4  lethe-idx-*    Vektor, FTS/BM25, Graph (Extensions, SegmentOwner)         │
│ L3  lethe-branch   Branch/Fork/Merge/ConflictCells (über Kern-API)            │
├────────────────────────────────────────────────────────────────────────────┤
│ L2  lethe-core     Engine: Txn, Snapshot, Scheduler, Budgets                  │  unsafe-frei
│     lethe-erasure  Automat, Ledger, Holds, Receipts, Purge, Verifier          │
│     lethe-ext      SegmentOwner, ExtStore, ChangeFeed, DerivationRegistry     │
│     lethe-catalog  Manifest, Spaces, Partitionen, Placement, Snapshot-Registry│
│     lethe-lsm · lethe-blob · lethe-wal · lethe-keys · lethe-format            │
│ L1  lethe-ports    Fs, Crypto, KeyService, Clock, Entropy, Anchor, Executor,  │
│                    Authority, Signer, BackupCatalog                           │
│ L0  lethe-types    Ids, Fehler, Konstanten, Weights                           │
├────────────────────────────────────────────────────────────────────────────┤
│ Adapter (unsafe erlaubt): lethe-fs-std, lethe-crypto-rc, lethe-kms-*,          │
│ lethe-anchor-*, lethe-sim, lethe-oracle, lethe-testkit                         │
└────────────────────────────────────────────────────────────────────────────┘
```

**Regeln (CI-durchgesetzt):** Pfeile nur nach oben. Kern-Crates tragen `#![forbid(unsafe_code)]`; Dependencies mit internem `unsafe` (`arc-swap`, `zeroize`, `imbl`) werden per `cargo-vet`/`cargo-geiger` gepinnt. `std::fs`, `std::time`, `rand` und Krypto-Crates sind außerhalb der Adapter per `clippy::disallowed_methods` verboten. Kein `mmap` (jeder Zugriff geht über `Fs::read_at`, damit `SimFs` ihn sieht). `lethe-format` ist rein (kein I/O, keine Uhr), `miri`- und Fuzz-tauglich. Keine Payload-Bytes, keine `EuId`s in Logs, Metriken oder Fehlertexten (`Redacted<T>`).

Die Engine ist synchron (feste `std::thread`-Pools pro Lane über den `Executor`-Port). Async-Fassaden (tokio) liegen erst in L6.

### 2.2 Ports

```rust
pub trait Fs: Send + Sync + 'static {
    type File: FsFile;
    fn open(&self, p: &Path, o: OpenOpts) -> IoResult<Self::File>;
    fn create_tmp(&self, dir: &Path) -> IoResult<(PathBuf, Self::File)>;
    fn rename_atomic(&self, from: &Path, to: &Path) -> IoResult<()>;
    fn unlink(&self, p: &Path) -> IoResult<()>;
    fn sync_dir(&self, dir: &Path) -> IoResult<()>;
    fn list(&self, dir: &Path) -> IoResult<Vec<DirEntry>>;
    fn free_space(&self, dir: &Path) -> IoResult<u64>;
    fn lock_exclusive(&self, p: &Path) -> IoResult<LockGuard>;
    fn capabilities(&self) -> FsCaps;                 // inplace_overwrite_reliable, reflink, direct_io
}
pub trait FsFile: Send + Sync {
    fn read_at(&self, off: u64, buf: &mut [u8]) -> IoResult<usize>;
    fn write_at(&self, off: u64, buf: &[u8]) -> IoResult<()>;
    fn append(&self, buf: &[u8]) -> IoResult<u64>;
    fn preallocate(&self, len: u64) -> IoResult<()>;   // fallocate, kein sparse
    fn sync_data(&self) -> IoResult<()>;               // Fehler => Poison, nie Retry
    fn len(&self) -> IoResult<u64>;
    fn overwrite_zero(&self, off: u64, len: u64) -> IoResult<()>;  // Scrub-Primitive
}

pub trait Crypto: Send + Sync {                         // zustandslos
    fn aead_seal(&self, k: &Key256, nonce: &[u8; 24], aad: &[u8], pt: &[u8], out: &mut Vec<u8>);
    fn aead_open(&self, k: &Key256, nonce: &[u8; 24], aad: &[u8], ct: &[u8], out: &mut Vec<u8>) -> Result<(), AuthFailed>;
    fn hkdf(&self, ikm: &Key256, label: &[u8], ctx: &[u8]) -> Key256;
    fn hmac(&self, k: &Key256, msg: &[u8]) -> [u8; 32];  // HMAC-SHA256
    fn hash(&self, msg: &[u8]) -> [u8; 32];              // BLAKE3
}
pub trait KeyService: Send + Sync {
    fn create_kek(&self, label: KekLabel) -> KsResult<(KekRef, Zeroizing<Key256>)>;
    fn fetch_kek(&self, r: &KekRef) -> KsResult<Zeroizing<Key256>>;
    fn destroy_kek(&self, r: &KekRef) -> KsResult<DestroyAttestation>;   // idempotent
    fn kek_status(&self, r: &KekRef) -> KsResult<KekStatus>;              // Live|Destroyed{at}|PendingDestroy{until}|Unknown
}
pub struct DestroyAttestation { pub kek: KekRef, pub strength: DestroyStrength, pub evidence: Vec<u8> }
pub enum DestroyStrength { LocalWipe, KmsScheduled { until: UnixMillis }, KmsDestroyed }

pub trait RollbackAnchor: Send + Sync {
    fn read(&self) -> AnchorResult<AnchorState>;
    fn advance(&self, expect: AnchorState, next: AnchorState) -> AnchorResult<AnchorAttestation>; // CAS, monoton
}
pub struct AnchorState { pub ledger_seq: u64, pub ledger_head: [u8; 32], pub erasure_watermark: u64 }

pub trait Authority: Send + Sync { fn verify(&self, op: &HoldOp, sig: &[u8]) -> bool; }
pub trait Signer: Send + Sync { fn sign(&self, msg: &[u8]) -> Vec<u8>; fn key_id(&self) -> KeyId; }
pub trait Clock: Send + Sync { fn wall(&self) -> UnixMillis; fn mono_nanos(&self) -> u64; }
pub trait Entropy: Send + Sync { fn fill(&self, buf: &mut [u8]); }
pub trait BackupCatalog: Send + Sync { fn oldest_backup_ts(&self) -> Option<UnixMillis>; }
pub trait Executor: Send + Sync {
    fn spawn(&self, lane: Lane, name: &'static str, job: Box<dyn FnOnce() + Send>) -> JobHandle;
    fn yield_now(&self);
}
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Lane { Urgent = 0, Commit = 1, Read = 2, Flush = 3, Compact = 4, Scrub = 5 }
```

Ein `MemLock`-Adapter (`mlock`, `MADV_DONTDUMP`) existiert nur im Adapter-Crate.

### 2.3 Identitäten

```rust
pub struct TenantId(pub u128);
pub struct EuId(pub u128);       // zufällig, nie ableitbar, nie loggen, nie wiederverwenden
pub struct EuTag(pub u64);       // HMAC(tenant_tag_key, eu_id)[..8], pro Tenant eindeutig
pub struct TombId(pub [u8; 32]); // HMAC(ledger_key, eu_id)
pub struct SpaceId(pub u32);
pub struct BranchId(pub u32);    // 0 = main
pub struct PartitionId(pub u32);
pub struct Seq(pub u64);         // lückenlos, streng monoton
pub struct ReqId(pub u128);

pub struct Weights { pub tokens: u32, pub bytes: u32, pub extra: [u32; 2] }
pub type Lanes = [i64; 4];       // Bitemporalität/Zeitfilter, Block-Min/Max im Meta-Index
```

**Tag-Eindeutigkeit (I21):** `create_eu` leitet `eu_tag` per HMAC mit Zähler `c` ab (`HMAC(tag_key, eu_id ‖ c)`) und erhöht `c`, bis der Tag im Keyring des Tenants unbelegt ist. Das ist nötig, weil Purge per Tag arbeitet: Eine Tag-Kollision würde Frames einer überlebenden EU löschen.

---

## 3. Begriffe

| Begriff | Bedeutung |
|---|---|
| **Tenant** | Mandant: Isolations- und Massenlöschgrenze. |
| **EU** | Kleinste kryptografisch löschbare Einheit (Nutzer, Konversation, Dokumentenset). Besitzt Schlüssel je Epoche, einen Zustand und beliebig viele Spaces/Branches. |
| **Deletion Group** | Platzierungshinweis: EUs, die gemeinsam gelöscht werden, teilen eine Partition. |
| **Partition** | Verzeichnis mit eigenem Memtable, LSM, Blob-Dateien. Gehört zu genau einem Tenant. |
| **Space** | Logischer Namensraum innerhalb einer EU (`agent_state`, `episodes`, `chunks`, Systemspaces). Kein physisches Gruppierungsmerkmal. |
| **Branch** | EU-interner Overlay mit Elternzeiger und `fork_seq`. |
| **Cohort** | ≤ 1024 EUs desselben Tenants mit gemeinsamer Wrap-KEK-Generation. |
| **Ledger** | Hash-verkettetes Governance-Journal in eigener Failure Domain. |
| **Anchor** | Externer monotoner Rollback-Anker. |
| **Cell** | Konzept der Ctx-Schicht: logische Wissenseinheit, abgebildet auf Entries in mehreren Spaces (Inhalt, Zoomstufen, Metadaten). |

---

## 4. Garantien, Bedrohungsmodell, Nicht-Ziele

### 4.1 Garantiestufen

| Stufe | Mechanismus | Wirkt ab | Garantie | Benannte Restexposition |
|---|---|---|---|---|
| **G0 Logisch** | Durabler `EraseGate`-Control-Record, atomarer Publish des `EuStateTable` | `t_logical` | Kein Read, Cursor, Feed, Extension-Antwort liefert Inhalte der EU. Pro EU: voller Snapshot oder `Erased`, nie Mischsicht. | Ciphertext liegt noch auf Platte, Klartext ggf. im RAM bis Zeroize. |
| **G1 Kryptografisch** | Cohort-Rewrap + KEK-Destroy + Ledger + Anchor | `t_crypto` | Ciphertext, Metadaten, Indizes, WAL-Teile der EU sind ohne Schlüssel unlesbar, solange kein weiterer Schlüsselträger existiert. | KMS-Karenzzeit (`KmsScheduled`), Schlüsselkopien im Prozessspeicher, Escrow. |
| **G2 Physisch** | In-place-Scrub bzw. Rewrite, WAL-Ablöse, Extension-Bereinigung, Verifier | `t_physical`/`t_verified` | Kein Frame, keine Sektion, kein Tag, keine Schlüsselmaterial-Bytes der EU in den Dateien des Stores (Löschorakel). | Medium (SSD-Wear-Leveling, COW-Snapshots, Page-Cache, Swap, Backups). Das Medium schützt nur G1. |

Das Ticket meldet nie mehr als die erreichte Stufe. `Completed` bedeutet G0+G1+G2 verifiziert; `CompleteWithResidual` benennt jedes Restrisiko.

### 4.2 Bedrohungsmodell

Vertrauensanker: KMS/HSM (Vernichtung irreversibel), Anchor (monoton), Prozess zur Laufzeit (vertrauenswürdig, crash-anfällig).

| ID | Angreifer/Fehler | Gegenmaßnahme |
|---|---|---|
| A1 | Offline-Dieb (Platten, Backups, alte Generationen) | Alles Personenbezogene versiegelt (I2). Nach Rewrap + KEK-Destroy sind alte Generationen wertlos. |
| A2 | Restore-/Rollback-Angreifer | Ledger + Anchor, Reconcile vor erstem Read (§9.8). |
| A3 | Betreiber mit Löschwunsch gegen Hold | Hold-Prüfung in Coordinator, Keyring-Writer und Purge; Release nur signiert von fremder Authority. |
| A4 | Fehlerhafter RAG-Layer | Nur sealed `ExtStore`; `Opaque`-Segmente nur als `NoPersonalData`/`Rebuildable`; Canary-Oracle. |
| A5 | Prompt-Injection über gespeicherte Inhalte | Trust-Labels mit Minimum-Propagation (§10.6). Der Storage verhindert keine Injection, liefert aber verlässliche Herkunft. |
| F1 | Torn Write / Power Loss | Record-Checksummen, Hash-Kette, Gap-Regel (§8.7). |
| F2 | `fsync` meldet EIO | Kein Retry, Engine → `Poisoned`, Recovery entscheidet. |
| F3 | ENOSPC | Vorallokiertes WAL, Lösch-Reserve, `ReadOnlyDegraded`, Scrub braucht keinen neuen Platz. |
| F4 | Bit-Rot | CRC32C auf Ciphertext (schlüssellos) + AEAD-Tag + Scrubber, Quarantäne pro Sektion. |
| F5 | Schlüsselverlust | `KeyUnavailable` / `KeyDestroyed` / `KeyLost` strikt getrennt. |
| F6 | Crash im Löschprotokoll | Ledger-Intents, idempotente Schritte, Roll-forward. |
| F7 | Retry-Stürme von Agenten | Idempotenz-Tabelle pro EU. |

### 4.3 Nicht-Ziele

Kein SQL, keine Query-Engine, kein ANN/Tokenizer/Embedding im Kern. Keine Replikation/Konsens im Kern (nur `ReplicationHook` mit Erasure-Vertrag). Keine Garantie für: Prozessspeicher bis Zeroize, Swap, Core-Dumps, Medium-Remanenz, bereits exportierte Daten (LLM-Provider-Logs, Prompt-/KV-Caches, Client-Caches, Fine-Tuning-Daten, Modellgewichte). Kein Schutz gegen Angreifer, der vor der Löschung DEK-Klartext entwendet hat. Keine Mehrprozess-Writer (exklusiver `LOCK`). Kein Tiering auf Objektspeicher und kein KV-Cache-Store in v1 (§14, Phase 9; beides müsste EU-rein und versiegelt sein).

### 4.4 Granularitätsgrenze (bindend)

Die kleinste kryptografisch nachweisbar löschbare Einheit ist die **EU**. Ein `delete`, Zoom-Down oder Retention-Schritt *innerhalb* einer EU ist logisch (Tombstone) und physisch erst nach Compaction. Bis dahin ist der alte Ciphertext mit dem lebenden EU-Schlüssel lesbar. Wer für ein Datum eine nachweisbare Einzelfall-Löschung braucht, muss es in eine eigene EU legen (Platzierung per `PlacementHint`, Verknüpfung per `Reference`-Link). Die Ctx-Schicht warnt in `describe()`, wenn eine Namespace-Policy `erasable = per_entry` verlangt, und lehnt die Policy ohne eigene EU ab.

---

## 5. Kryptografie

### 5.1 Schlüsselhierarchie

```
KMS-Root
  └─ Cohort-KEK (Generation g)                       ≤ 1024 EUs je Cohort
       └─ EU-DEK[epoch]  (256 Bit, zufällig)
            └─ HKDF-Expand("lethe/v1/" ‖ kind) ─► k_wal | k_meta | k_idx | k_hdr | k_body | k_blob | k_ext:<ext_id>
Store-weit, getrennt: ledger_key, bloom_secret, cursor_key (Prozess, flüchtig)
Tenant-weit: tenant_tag_key (gewrappt, Generation je Tenant)
```

- **Epochen** ändern nur, womit *neue* Frames versiegelt werden. Bestehende Sektionen behalten ihre Epoche (Frame-Header + AAD). Compaction darf Epochen mischen und mit der aktuellen neu versiegeln. Erase vernichtet stets alle Epochen, ein Teil-Scope existiert nur für Betreiber-Rotation, nie für Löschtickets.
- **KEK-Rotation** (Provider) wrappt nur die 32-Byte-DEKs neu.
- **Cohort-Größe 1** ergibt exakt das B-Modell „ein KMS-Objekt pro Cluster". Die `KeyService`-Strategie ist austauschbar.
- **Cohort-Zuweisung:** EUs derselben Deletion Group kommen bevorzugt in dieselbe Cohort (bündelt Rewraps).

### 5.2 Frame (universell: WAL, Run-Sektionen, Blob-Chunks, Ext-Frames)

Little Endian, 64 Byte Klartext-Header, dann Ciphertext.

```text
Off  Len  Feld
  0   4   magic      'LFR1'
  4   1   kind       WalRec|Control|MetaFrame|IdxFrame|HdrFrame|BodyFrame|BlobChunk|ExtFrame
  5   1   flags      compressed, padded
  6   2   ver
  8   8   eu_tag     0 bei EU-losen Frames (Control)
 16   4   epoch      DEK-Epoche
 20   4   local_ord  Ordnungszahl innerhalb der Sektion/Datei (Blocknummer, Chunknummer)
 24   4   body_len   Länge des Ciphertext inkl. 16 B Tag
 28   4   body_crc   CRC32C über Ciphertext (schlüssellos prüfbar)
 32  24   nonce      XChaCha20, 192 Bit, aus Entropy-Port
 56   4   reserved   0
 60   4   hdr_crc    CRC32C über Bytes 0..60
Body: AEAD_Seal(k_<kind>, nonce, aad, plaintext)
aad = "lethe/aad/v1" ‖ store_uuid ‖ kind ‖ eu_id ‖ epoch ‖ space ‖ local_ord ‖ record_id
```

**AAD-Regeln:** `record_id` ist ein record-lokaler Bezeichner (z. B. Blocknummer, `chunk_id`, `txn_id`), **nie** `Seq`, Dateiname, Datei-ID oder absoluter Offset (I3). Dadurch gilt: Ein Frame lässt sich byteidentisch in eine andere Datei kopieren, der Scrubber prüft Integrität und Zugehörigkeit ohne DEK, und die Versiegelung kann in `prepare` (vor der Seq-Vergabe) erfolgen.

**Nonce:** 192 Bit zufällig je Frame, kein Zählerzustand. Kollisionsfrei bis 2⁶⁴ Frames je Schlüssel. `DetEntropy` erzeugt in Tests reproduzierbare, eindeutige Nonces; die Suite prüft Eindeutigkeit explizit (I10).

**Kompression/Padding:** Komprimieren vor Verschlüsseln leakt Längen. Optional `pad_to` je Space (Bucket-Größen).

---

## 6. On-Disk-Formate

### 6.1 Verzeichnis

```
<root>/
  LOCK
  CURRENT                           → MANIFEST-<gen>   (atomar per rename + sync_dir)
  MANIFEST-<gen>                    Snapshot + Edit-Log (Dateilisten, flushed_seq, Cohort-Liste, SegmentRegistry); EU-frei
  wal/wal-<n>.log                   vorallokiert, 64 MiB
  keys/<tenant>/cohort-<id>-g<gen>.keys
  keys/<tenant>/tenant-g<gen>.keys
  parts/<tenant>/<partition>/run-<id>.lsr
  parts/<tenant>/<partition>/blob-<lineage>-g<gen>.lbs
  ext/<ext_id>/seg-<id>.lxs
  reserve/                          Notreserve (WAL-Reserve-Segment, Ledger, Keyring-Gen, Purge-Temps), 256 MiB + 16 MiB Ledger
  quarantine/  tmp/
<separates Volume / eigene Failure Domain>
  ledger/ledger-<n>.log
```

Die Dateizahl wächst mit dem Datenvolumen und der Partitionszahl, nicht mit der EU-Zahl.

### 6.2 WAL

```text
WalSegment := Header(4 KiB: magic, store_uuid, seg_no, base_lsn, crc) · Batch*
Batch  := BatchHeader · Record* · BatchCommit
BatchHeader (Klartext): magic, batch_id, first_seq, record_count, prev_chain[32], hdr_crc
Record := Frame(kind=WalRec|Control) mit: txn_id, commit_seq, [OpPart(eu_tag, epoch, Seal(k_wal(eu), ops))]+
BatchCommit: chain = BLAKE3(prev_chain ‖ BatchHeader ‖ alle Record-Bytes), crc
```

- Eine Transaktion ist **ein** Record, mit OpParts mehrerer EUs/Spaces/Partitionen (atomarer Mehrbereichs-Commit, I11).
- Jeder OpPart ist unter dem WAL-Subkey **seiner** EU versiegelt. Nach Schlüsselvernichtung sind WAL-Reste unlesbar; zusätzlich Rolling + Truncate (§9.5).
- Batches 4-KiB-ausgerichtet in vorallokierten Segmenten, nie recycelt (Segmente werden gelöscht und neu angelegt).
- `Control`-Records tragen nur `eu_tag`s: `EraseGate{req, tags[], E}`, `HoldEvent`, `CohortSwitch`, `CatalogMark{flushed_lsn}`, `BranchOp`, `Pin`. Recovery wendet sie ohne Schlüssel an.
- `commit_seq` steht nur in der Record-Hülle, nicht in der AAD.

### 6.3 Run (`.lsr`)

```text
Run := RunHeader(4 KiB: magic, ver, run_id, partition_id, level, min_seq, max_seq, section_count, features, crc)
       · Section* · Directory · Footer(64 B)

Section(eu) := SecHeader(64 B, Klartext: magic, eu_tag, frame_count, hot_len, sec_len, crc)
               · MetaFrame · IdxFrame* · HdrFrame* · BodyFrame*          ← section-relativ adressiert, aligned (section_align)
               [Hot-Bereich = SecHeader+MetaFrame+IdxFrame*+HdrFrame*+(optional Body-Prefix) ; Cold-Bereich = BodyFrame*]

Directory := Entry*   sortiert nach eu_tag
Entry     := { eu_tag, epoch_min, off, len, hot_len, min_seq, max_seq, live_entries, flags(dead), entry_crc }
Footer    := dir_off, dir_len, entry_count, magic, footer_crc        (KEIN Gesamt-Hash über das Directory, K12)
```

**MetaFrame** (versiegelt, `k_meta`): Block-Index mit je Block `{first_key, last_key, frame_offsets, entry_count, cum_tokens, cum_bytes, weight_sum[4], lane_min[4], lane_max[4]}`, **Bloom über `(space, branch, key)` mit EU-versiegeltem Salt**, Statistiken, Range-Marker.

**IdxFrame** (je Block, `k_idx`, Ziel 16 KiB Klartext): Einträge `{space, branch, key (≤ 1 KiB, prefix-komprimiert), seq, op(Put|Del), flags(tombstone, blobref, trust u3), weights, lanes[4], raw_len, hdr_ref, body_ref}`.
**HdrFrame** (`k_hdr`): konkatenierte opake Header/Filterattribute (≤ 1 KiB je Eintrag, Default-Cap 256 B).
**BodyFrame** (`k_body`): Inline-Körper (≤ `inline_max`) oder `BlobRef{lineage, first_chunk, chunk_count, raw_len, codec}`.

Ranking, Budget-Paging, Lane-Pruning und Window-Berechnung lesen nur Meta + Idx (+ Hdr bei Prädikaten). Bodies werden erst auf Anforderung gelesen. Das ist Hot-Head/Cold-Tail auf Sektionsebene.

**Run-weiter Cluster-Bloom** (optional): keyed SipHash über `eu_tag` mit `bloom_secret`, damit er keine Tags preisgibt.

**Directory-Suche** ist Binärsuche nach `eu_tag` im Klartext. Das Directory und der `eu_tag` sind das einzige EU-bezogene Klartext-Artefakt einer Run-Datei.

### 6.4 Blob (`.lbs`)

```text
BlobFile := BlobHeader(4 KiB) · ChunkFrame* · ChunkDirectory · Footer
ChunkFrame     := Frame(kind=BlobChunk, eu_tag, local_ord=chunk_no) · Seal(k_blob, zstd(≤ 64 KiB Rohdaten))
ChunkDirectory := [{chunk_id u64, off u64, len u32, eu_tag u64, flags(dead), entry_crc}] sortiert nach chunk_id
```

- Die aktive Blob-Datei nimmt Frames mehrerer EUs verschränkt auf (append-only). Die Directory entsteht beim Sealen. Recovery rekonstruiert sie bei der aktiven Datei per Frame-Scan (Header-CRC).
- Köpfe referenzieren `(lineage, chunk_id)`, nicht Offsets. Der Katalog bildet `lineage → Datei/Generation` ab. Rewrite/GC behält Chunk-Ids und erhöht nur die Generation, **ohne versiegelte Köpfe anzufassen**.
- Bereichsleser holen nur benötigte Chunks.
- Blob-GC (Garbage > Schwelle): verbatim Chunk-Kopie, optional Re-Gruppierung nach EU.

### 6.5 Manifest

Snapshot + Edit-Log (CRC-gesichert, Torn Tail tolerant). Edits: `AddRun/RemoveRun/ReplaceRun`, `AddBlob/RemoveBlob`, `SetFlushedSeq(partition)`, `AddSpace`, `AddPartition`, `CohortGen`, `SegmentRegistry`. **Das Manifest enthält weder `EuId` noch `eu_tag` noch Link-/Scrub-Information (I4).** Zustand und Beziehungen einer EU liegen im Keyring, in sealed Systemspaces und im Ledger.

### 6.6 Keyring (Cohort-Datei) = EU-Verzeichnis

```text
CohortFile(gen g) := Header{tenant, cohort_id, gen, kek_ref, entry_count} · Entry* · Footer{blake3}
Entry := Seal(KEK_g, { eu_id, eu_tag, partition, state, held, created_seq,
                       epochs: [{epoch, dek_wrapped(32+16)}], tag_counter })
```

Der Keyring ist die **einzige** dauerhafte Quelle für `EuId → Partition/State/Epochen/DEKs`. Rewrap lässt gelöschte EUs weg. Pflicht: ≥ 2 Replikate in getrennten Failure Domains, synchron aktualisiert, **nur aktuelle Generation**.

### 6.7 Ledger

```rust
struct LedgerEntry {
    seq: u64, prev: [u8; 32], ts: UnixMillis,
    kind: LedgerKind,   // ErasureRequested{req, tomb_ids, eu_tags, urgency} | StageIntent{req, stage, args_digest}
                        // | StageDone{req, stage, evidence_digest} | PurgeProgress{req, file_ids, tag_digest}
                        // | KeyRewrite{cohort, old_gen, new_gen} | KekDestroyed{kek, attestation_digest}
                        // | HoldPlaced/Released{hold, scope, sig} | ExtAck{req, ext, digest}
                        // | PurgeVerified{req, residual_digest} | ReceiptIssued{req, receipt_digest}
                        // | RestoreReconciled{..} | AnchorPending{req}
    mac: [u8; 32],      // HMAC(ledger_key, entry_hash); entry_hash = BLAKE3(prev ‖ body)
}
```

Das Ledger enthält nur `tomb_id`s, `eu_tag`s und Digests: nie Nutzdaten, nie `EuId`. Ein signierter Tomb-Set-Checkpoint kann die Ledger-Länge begrenzen (in die Hashkette eingebunden). Retention mindestens bis zum Ablauf der ältesten Backup-Generation, die betroffene Daten enthalten könnte.

### 6.8 Platzierung und Kostenmodell

`PlacementHint { deletion_group: Option<u128>, expected: Small|Medium|Large }`. Gruppen werden per Hash auf ≤ `P` Partitionen je Tenant abgebildet; kleine Gruppen teilen Partitionen, große erhalten dedizierte. Überschreitet eine Gruppe `promote_bytes`, erfolgt ein **Partition-Split per verbatim Sektionskopie**.

| Szenario | Kryptografisch | Physisch (Standard) |
|---|---|---|
| Ganzer Tenant | O(Cohorts) KEK-Destroys + `tenant_tag_key` | `unlink` der Tenant-Verzeichnisse |
| Dedizierte Gruppe | 1 Cohort-Rewrap | `unlink` der Partition |
| EU in geteilter Partition | 1 Cohort-Rewrap | In-place-Scrub nur dieser EU-Sektionen, O(EU-Größe) |

---

## 7. Datenmodell

### 7.1 Kern-Entry

Der Kern kennt nur: `(eu, space, branch, key: Vec<u8>) → Entry { seq, op, flags, weights, lanes[4], header ≤ 1 KiB, body }`. `header` und `body` sind opake Bytes.

Systemspaces (je EU, alle versiegelt): `SYS_IDEM`, `SYS_DERIV`, `SYS_BRANCH`, `SYS_ALIAS`, `SYS_CONFLICT`, `SYS_DCACHE`, `SYS_ACCESS`.

`SpaceKind`: `Durable` | `Ephemeral` (Scratchpad, nie WAL/fsync, flüchtiger Prozessschlüssel, Spill in sofort ge-`unlink`te Datei, Zeroize bei Drop).

### 7.2 Gewichte, Lanes, Bitemporalität

- `weights.tokens` = Token-Anzahl der primären Tokenizer-Familie des Space; `extra[0..1]` = weitere Familien (Space-Policy ordnet zu); `bytes` = Rohgröße.
- `lanes[0] = valid_from`, `lanes[1] = valid_to` (Gültigkeitszeit); `lanes[2..3]` frei. Die Transaktionszeit ist die `Seq` (plus HLC im Commit-Record). `lane_min/max` je Block prunen ganze Blöcke ohne Header zu lesen.
- Zeitreise: `as_of(seq | timestamp)` innerhalb der Retention (`min_retained_seq`) oder per `RetentionPin`.

### 7.3 Branches (EU-interner Overlay)

- `fork(eu, parent_branch) -> BranchId`: legt `SYS_BRANCH{id, parent, fork_seq, ttl, intent}` an und setzt einen `RetentionPin{eu, fork_seq, ttl}`. Kosten: ein kleiner Commit, keine Datenkopie (Ziel < 50 µs für die In-Memory-Seite, Zielwert).
- Lesen auf Branch `b` unter Snapshot `S`: Kette `b → parent (≤ fork_seq) → …`, erster Treffer gewinnt, `Del` verdeckt.
- Der `RetentionPin` hält Versionen unterhalb `fork_seq` gegen MVCC-GC. Speculative Branches haben TTL und fallen automatisch weg (Pin-Release, Tombstone-Range für ihre Einträge).
- **Merge** (Ctx-Schicht, Dreiwege Base/Ours/Theirs je Attribut): Strategien `lww | union | sum | max | min | diff3 | custom(wasm) | manual`. `manual` erzeugt `ConflictCell` in `SYS_CONFLICT` (versioniert, abfragbar, per `resolve(id, choice | merged)` lösbar). Das Ergebnis ist ein normaler Commit auf dem Zielbranch.
- Aggregat-Fast-Path (§8.3) gilt nur auf einem einzigen Branch ohne Überlappung. Sonst Head-Scan.
- Ein Branch kreuzt nie die EU-Grenze (I22).

### 7.4 Abhängigkeiten und Kaskade

```rust
pub enum LinkKind { DerivedFrom, Reference }
pub enum OnSourceErasure { Delete, Invalidate }
// Txn::link(parent_eu, child_eu, LinkKind, OnSourceErasure)  – Teil des Commits, atomar mit den ersten Daten des Kindes
```

- Reverse-Kanten liegen sealed in `SYS_DERIV` der **Quell**-EU, Vorwärtskanten in der Kind-EU. Azyklizität (Erreichbarkeitsprüfung, Tiefenlimit) beim Anlegen.
- `Txn::derive(out_eu, sources, policy)` setzt `trust = min(sources)`.
- Ein Kind mit mehreren Eltern wird bei Löschung **eines** `DerivedFrom/Delete`-Elternteils mitgelöscht (konservativ). `Invalidate` markiert `Stale` und verbleibt versteckt bis zum Rebuild. `Reference` kaskadiert nicht.

### 7.5 Öffentliche Kern-API (Auszug)

```rust
impl<P: Ports> Engine<P> {
    pub fn open(ports: P, cfg: Config) -> Result<Self, OpenError>;     // deterministische Recovery (§8.8)
    pub fn create_tenant(&self, t: TenantId) -> Result<()>;
    pub fn create_space(&self, t: TenantId, s: SpaceDesc) -> Result<SpaceId>;
    pub fn create_eu(&self, t: TenantId, eu: EuId, place: PlacementHint) -> Result<()>;
    pub fn begin(&self, o: TxnOpts) -> Txn<'_, P>;
    pub fn snapshot(&self) -> Snapshot<P>;                             // Lease 60 s, max 10 min
    pub fn pin_seq(&self, eu: EuId, seq: Seq, ttl: Duration) -> Result<PinId>;
    pub fn erase(&self, req: ErasureRequest) -> Result<ErasureHandle>; // idempotent über req.id
    pub fn erasure_status(&self, id: ReqId) -> Result<ErasureStatus>;
    pub fn receipt(&self, id: ReqId) -> Result<Option<ErasureReceipt>>;
    pub fn place_hold(&self, h: Hold, sig: &[u8]) -> Result<HoldId>;
    pub fn release_hold(&self, id: HoldId, sig: &[u8]) -> Result<()>;
    pub fn begin_backup(&self) -> Result<BackupGuard>;
    pub fn register_extension(&self, ext: Arc<dyn SegmentOwner>) -> Result<ExtStoreHandle>;
    pub fn change_feed(&self) -> ChangeFeed;
}
pub struct TxnOpts { pub durability: Durability, pub idem: Option<IdemKey>, pub lane: Lane, pub deadline: Option<Duration> }
pub enum Durability { Sync, Async /* Ack vor fsync; nie für Control-Records */ }  // Replicated: Phase 9 via ReplicationHook
pub struct IdemKey { pub run: u128, pub step: u64, pub key: u128 }
pub enum Guard { UnchangedSince { eu: EuId, seq: Seq }, KeyUnchangedSince { eu: EuId, sp: SpaceId, key: Vec<u8>, seq: Seq } }

impl<'e, P: Ports> Txn<'e, P> {
    pub fn put(&mut self, eu: EuId, br: BranchId, sp: SpaceId, key: &[u8], v: PutValue) -> Result<()>;
    pub fn delete(&mut self, eu: EuId, br: BranchId, sp: SpaceId, key: &[u8]) -> Result<()>;
    pub fn link(&mut self, parent: EuId, child: EuId, k: LinkKind, on: OnSourceErasure) -> Result<()>;
    pub fn derive(&mut self, out: EuId, sources: &[EuId], policy: OnSourceErasure) -> Result<()>;
    pub fn guard(&mut self, g: Guard) -> &mut Self;
    pub fn commit(self) -> CommitFuture;                               // -> CommitReceipt { seq, durable, ephemeral_part }
}
pub struct PutValue<'a> { pub body: Body<'a>, pub weights: Weights, pub lanes: Lanes, pub header: &'a [u8], pub trust: Trust }

impl<P: Ports> Snapshot<P> {
    pub fn get_head(&self, eu: EuId, br: BranchId, sp: SpaceId, key: &[u8]) -> Result<Option<Head>>;
    pub fn get_body(&self, h: &Head, range: Option<Range<u64>>) -> Result<Bytes>;
    pub fn scan_page(&self, q: ScanQuery, budget: Budget, cur: Option<Cursor>) -> Result<Page>;
    pub fn filter_visible(&self, refs: &[RecordRef]) -> Vec<RecordRef>;
}
pub struct ScanQuery { pub eu: EuId, pub branch: BranchId, pub space: SpaceId, pub range: KeyRange, pub dir: Dir,
                       pub lane_filter: Option<LaneRange>, pub header_pred: Option<HeaderPred> }
pub struct Budget { pub tokens: Option<u32>, pub bytes: Option<u32>, pub max_items: u32 }
pub struct Page { pub heads: Vec<Head>, pub next: Option<Cursor>, pub used: Weights, pub oversize_single: bool }
```

**Fehlerklassen** `Erased`, `Restricted`, `CursorExpired`, `SnapshotTooOld`, `KeyUnavailable`, `KeyLost`, `Busy{retry_after|NoSpace}`, `IdemConflict`, `DurabilityUnknown` sind eigene Klassen, nie `NotFound`. Die Policy `error_masking` kann „gelöscht" von „existiert nicht" bewusst ununterscheidbar machen.

---

## 8. Ablaufprotokolle

### 8.1 Schreibpfad (Group Commit)

1. **Begin:** `read_seq` erfassen.
2. **Prepare (parallel, Lane `Commit`):**
   1. `EuStateTable.load()`: EU muss `Live` sein. `KeyHandle` (`Arc`, `revoked: AtomicBool`) aus dem Key-Cache.
   2. Große Bodies: 64-KiB-Chunks → zstd → versiegeln (`k_blob`) → an aktive Blob-Datei der Partition anhängen (ungesynct) → `BlobRef`.
   3. OpParts mit `k_wal` versiegeln (AAD ohne `Seq`). `weights`, `lanes`, `header`, `trust` stammen vom Aufrufer.
   4. Guard-Vorprüfung gegen `head_seq` und `HotVersionMap` (konservativ, nur Speicher). `IdemKey` wird für `SYS_IDEM` vorbereitet; kalte Treffer werden hier geprüft, nie im Sequencer.
3. **Enqueue** (`Urgent` > `Commit` > `Background`). Erasure-Control nutzt `Urgent`.
4. **Sequencer-Loop** (ein Thread, kein Lock):
   1. Batch bis `max_batch_bytes` (4 MiB) oder `max_delay` (adaptiv 50–500 µs).
   2. Serialisiert pro Record: EU-Zustand (autoritativ, R2), Hold/Restriction, Idempotenz (gleicher Key + gleicher Fingerprint ⇒ gespeicherter `CommitReceipt`, anderer Fingerprint ⇒ `IdemConflict`), Write-Write-Konflikt (First-Committer-Wins über `last_commit_seq(eu, branch, key) > read_seq`), Guards.
   3. `commit_seq` lückenlos vergeben, `chain` berechnen.
   4. **Blob-Sync** aller referenzierten, ungesyncten Blob-Dateien (dedupliziert).
   5. Batch an WAL-Tail, `sync_data`: **einziger Durability-Punkt.**
   6. **Apply:** Memtable per Copy-on-Write (`imbl::OrdMap`), `ArcSwap<Version>` publizieren, dann `visible_seq` (Release).
   7. Futures: `CommitReceipt { seq, durable: true }`.
5. **Ordnung:** `Blob-fsync → WAL-fsync → Apply → Ack`. Ein referenzierter Chunk ist stets durable; Chunks ohne WAL-Referenz sind Waisen für den Blob-GC.
6. **`Durability::Async`:** Ack nach 4.3, dokumentiert verlustbehaftet, nie für Control-Records.
7. **Ephemeral Spaces:** nie WAL/Blob/fsync. Gemischte Transaktionen erlaubt (`ephemeral_part: Volatile`); nach Crash existiert nur der durable Teil. Erase = sofort verwerfen + Zeroize.
8. **Backpressure:** `Busy{retry_after}` bei (a) immutable Memtables > N, (b) L0-Runs > M, (c) WAL-Bytes > Cap, (d) Speicherbudget. `Urgent` umgeht (a)–(c).
9. **Idempotenz:** Retry nach Erase liefert `Erased` (Einträge werden in Phase 2 invalidiert, R8).

### 8.2 Lesepfad

**Point-Get:**
1. `EuStateTable.load()`; nur `Live` passiert (I6). `erasure_epoch` merken.
2. Memtable (aktiv + immutable), dann Runs `newest → oldest` der Partition; Branch-Kette auflösen.
3. Je Run: Directory-Binärsuche nach `eu_tag`, Meta aus Cache oder laden, Bloom, Fence-Suche, Idx lesen (CRC → entschlüsseln), Key suchen.
4. Erstes Ergebnis mit `seq ≤ snapshot.seq` gewinnt; `Del` ⇒ `None`.
5. **Check-after-decrypt:** `KeyHandle.revoked` und `erasure_epoch` erneut prüfen; bei Änderung Ergebnis verwerfen, `Erased` liefern (R1).
6. `get_body` nur auf Anforderung (Chunk-Range, Dekompression nur dieser).

**Cursor:** AEAD-versiegelt mit `cursor_key` (Prozess), enthält `{snap_seq, eu, branch, space, last_key, erasure_epoch}`, keinen Klartext-Key. `snap_seq < min_retained_seq` ⇒ `CursorExpired`. EU nicht mehr `Live` ⇒ `Erased`/`Restricted`, nie Teildaten. Ein Prozessneustart macht Cursor ungültig.

### 8.3 Budget-Paging

```text
Eingabe: ScanQuery, Budget{tokens?, bytes?, max_items}, Cursor?
1. Snapshot aus Cursor oder aktuell; Cursor validieren.
2. Merge-Iterator (Richtung Forward/Backward) über Memtable + Runs, nur Meta/Idx(+Hdr bei header_pred); Bodies NIE.
   Blöcke mit lane_min/max außerhalb lane_filter werden übersprungen.
3. Für jeden sichtbaren Head h (seq ≤ snap, kein Tombstone, neueste Version je Key):
     überschreitet used + h.weights das Budget in irgendeiner Dimension:
        heads leer ⇒ heads=[h], oversize_single=true (Fortschrittsgarantie); sonst stop
     used += h.weights; heads.push(h); bei max_items stop
4. next = AEAD-versiegelter Cursor.
5. Check-after-read: EuStateTable ein zweites Mal prüfen (R1).
```

**Aggregat-Fast-Path:** Liefert im Zielbereich nur *ein* Run Einträge (Fences prüfen, kein Schatten, keine Tombstones, ein Branch), bestimmen `cum_tokens/cum_bytes/count` im Meta exakte Seitengrenzen ohne Idx zu dekodieren. `window(anchor, budget)` ist dann O(log n) Meta-Reads. Bei Überlappung sind Aggregate nur obere Schranken, der Head-Scan liefert die Wahrheit.

### 8.4 MVCC und Snapshots

`Version = {memtables, runs je Partition, manifest_epoch}`, unveränderlich, per `ArcSwap` publiziert. Die Snapshot-Registry führt `min_active_seq` über ein `AtomicU64`-Slot-Array (lock-frei). Lease 60 s verlängerbar, hart 10 min (`SnapshotTooOld`). `RetentionPin`s (Branches, Zeitreise) werden separat in `min_retained_seq` eingerechnet. Versions-GC bei Flush/Compaction unterhalb des Minimums bis auf die jeweils neueste Version. **Erasure ist nicht versioniert (D7):** der `EuStateTable` wird bei jedem Zugriff konsultiert.

### 8.5 Flush

1. Immutable Memtable je Partition → je EU eine Sektion → Run (`tmp/`). EUs, die nicht `Live` sind, werden übersprungen.
2. `sync_data` Run, `sync_dir`.
3. Unter `L2`+`L4`: `erase_epoch` mit dem Wert vom Start vergleichen. Hat sich ein Gate eingeschoben, werden betroffene Sektionen vor dem Publish gescrubbt oder der Run verworfen (R3).
4. Manifest-Edit `AddRun, SetFlushedSeq`, `sync`, neue `Version` publizieren.
5. WAL-Segmente mit `max_seq ≤ min(flushed_seq)` freigeben (`overwrite_zero` Best-Effort → `unlink`), soweit kein offener Löschauftrag sie hält.
6. **Kaltes-Partition-Problem:** Partitionen mit ältestem ungeflushtem Record > `max_wal_age` oder bei `wal_bytes > cap` werden zwangsgeflusht.

### 8.6 Compaction

- **Auswahl:** pro Partition, größen-gestaffelt, Lane `Compact`, ≤ 1-MiB-Schritte mit Abbruchpunkten.
- **Pfad 1 (Rohkopie):** Liegt eine EU-Sektion nur in einem Eingabe-Run und enthält keine abgelaufenen Versionen: verbatim kopieren (kein Entschlüsseln).
- **Pfad 2 (Merge):** nur diese EU-Sektionen mit lebenden DEKs entschlüsseln, MVCC-GC unterhalb `min_retained_seq`, mit frischer Nonce und aktueller Epoche neu versiegeln.
- **Install-Check** unter `PartitionMaintenance`: Output-EUs gegen `EuStateTable` prüfen, nicht-`Live` verwerfen (R3). Lese-/CRC-Fehler in Sektionen gegateter EUs sind kein Korruptionsbefund (sie werden gerade gescrubbt).
- **Publish:** fsync → Manifest-Edit (`AddRun/RemoveRun` atomar) → Version. Alte Dateien werden bei Drop des letzten `Arc<FileGuard>` gelöscht.
- Dead-Anteil über Schwelle löst Rewrite zur Platzrückgewinnung aus (nicht Compliance-relevant).

### 8.7 Torn-Write-Erkennung (Gap-Regel)

Batches werden seriell geschrieben und gesynct. Erster ungültiger Batch an Position *k*: Existiert danach **irgendein** gültiger Batch, dessen `prev_chain` auf die erwartete Kette von *k* zeigt, war *k* acked ⇒ **Korruption**, Fail-Stop mit Befund (Segment/Offset), optional `degraded` für andere Partitionen. Sonst Torn Tail ⇒ abschneiden (nichts wurde acked). Segment-Seriennummer + Kette verhindern, dass Altdaten als gültig durchgehen.

### 8.8 Deterministische Recovery (reine Funktion von Dateien, Ledger, Anchor, KMS)

1. `LOCK`, `CURRENT`, Manifest replayen (Torn Tail tolerieren).
2. **Anchor/Ledger:** Kette verifizieren. `anchor.ledger_head` muss an `anchor.ledger_seq` im Ledger vorkommen (sonst `LedgerRollback`, fatal). `ledger.seq ≥ anchor.ledger_seq`.
3. **Restore-Erkennung:** `manifest.erasure_watermark < anchor.erasure_watermark` ⇒ `RestoreDetected` ⇒ Reconcile vor dem ersten Read (§9.8).
4. **Dateiabgleich:** Dateien außerhalb des Manifests → `quarantine/` (Waisen) bzw. löschen (`tmp`). Fehlende/defekte Manifest-Dateien → Quarantäne + `Corruption`-Event. **Sektionen, deren `eu_tag` im Ledger-Erased-Set steht, sind nie Korruption**, sondern Scrub-Kandidaten.
5. Keyrings laden, KEK-Status per `KeyService` prüfen. Unvollständige Rewraps (Intent ohne `StageDone`) ⇒ Roll-forward.
6. `EuStateTable` aufbauen: Keyring + Ledger (Tomb-Set) + WAL-Control-Records.
7. **WAL-Replay** ab `min(flushed_seq)` mit Hashkette und Gap-Regel; nur Records mit `commit_seq > flushed_seq(partition)`; **Records von Erased-Tags nicht anwenden**; `EraseGate`-Records anwenden, bevor irgendein Read möglich ist (R10).
8. Tail abschneiden, neues Segment, `visible_seq = last_valid_seq`.
9. Offene `ErasureRequest`s aus dem Ledger laden (§9), halbe Compactions anhand der Edit-Gruppen auflösen.
10. **Invarianten-Check:** Für jeden Erased-Tag: kein Keyring-Eintrag, kein lebender Directory-Eintrag in nicht-retirten Dateien. Verstöße ⇒ Purge-Fixpunkt-Lauf.
11. `open` liefert erst jetzt `Ok`. Recovery ist idempotent: `recover(recover(S)) = recover(S)`.

### 8.9 Fehlerverhalten

| Ereignis | Verhalten |
|---|---|
| `fsync(WAL)` EIO | Batch: `DurabilityUnknown`. Engine `Poisoned` (Reads auf publizierten Zustand, keine Writes). **Kein Retry.** Neustart entscheidet. |
| `fsync(Run/Blob)` EIO | Job scheitert, Output verworfen, Quelle bleibt gültig. |
| `fsync(Manifest)` EIO | `Poisoned`. |
| ENOSPC WAL | Nicht erwartet (vorallokiert); sonst `Poisoned`. |
| ENOSPC Flush/Compaction | Abbruch + Cleanup, `ReadOnlyDegraded` (Writes `Busy{NoSpace}`), Reads laufen. **Löschpfad bleibt aktiv** über `reserve/`; nur `Urgent` darf sie verbrauchen. Scrub schreibt in bestehende Bereiche, Rewrite wird ausgesetzt. Verlassen: Platz über Schwelle, Reserve neu anlegen. |
| CRC/AEAD-Fehler | `Corruption{file, section}`, Sektion unter Quarantäne, andere EUs unberührt. Keine stille Reparatur. |
| KMS temporär weg | `KeyUnavailable{retry_after}`; gecachte DEKs (TTL) bedienen weiter; Lösch-Schritte mit Backoff. |
| KEK `Destroyed`, Ledger bestätigt | `KeyDestroyed` ⇒ `Erased` (gewollt). |
| KEK fehlt, Ledger kennt keine Löschung | `KeyLost`, EU → `Orphaned`, Alarm, **keine** automatische Löschung/Neuaufbau. |
| Hash-Kette/Anchor verletzt | `LedgerRollback`/`RestoreDetected`. |
| Keyring-Volume verloren | Ohne Replikat Datenverlust der Cohort (kryptografisch unvermeidbar). Daher Replikationspflicht. |
| Anchor nicht erreichbar | Policy `strict`: Löschauftrag bleibt `AnchorPending`, `Completed` wird nicht gemeldet. `best_effort`: lokal fortfahren, `AnchorPending` im Ticket sichtbar. |

### 8.10 Ressourcenkontrolle

- **Speicherbudget:** `{memtables, section_meta_cache, block_cache(ciphertext), key_cache, in_flight}` mit atomaren Zählern; Überschreitung ⇒ Backpressure, nie OOM. Plaintext-Caches halten `Zeroizing<Vec<u8>>` mit `eu_tag`-Index für `evict_eu(tag)`.
- **FD-Budget:** `FdCache` (LRU, `FdPermit`); WAL, Ledger, aktive Blobs gepinnt.
- **I/O-Lanes:** `Urgent (WAL-fsync, Ledger, KMS, Purge, Scrub-nach-Gate) > Read > Flush > Compact > Scrub(Hintergrund)`. Token-Bucket je Lane, `Urgent` ≥ 20 % garantiert und verdrängt Compaction per Cancel-Flag an Chunk-Grenzen.
- **Threads:** feste Pools je Lane, Sequencer = 1 Thread.

---

## 9. Löschprotokoll Ende-zu-Ende

### 9.1 Anfrage

```rust
pub struct ErasureRequest {
    pub id: ReqId,                       // Idempotenz: gleiche id ⇒ gleiche Instanz
    pub scope: ErasureScope,             // Eu | EuSet | DeletionGroup | Tenant
    pub urgency: Urgency,                // Immediate | Batched{window}
    pub purge_deadline: Duration,        // Politik-Parameter
    pub derived: DerivedPolicy,          // FollowDerivation (Default) | None
    pub requester: Principal,
}
```

### 9.2 Zustandsautomat

```text
Received ─► Evaluating ─(Hold)─► Blocked ──Release──┐
   │            │ (kein Hold)                       ▼
   │            └────────────► LogicallyErased ─► KeysDestroying ─► KeysDestroyed
   │                             (G0, t_logical)        │                │
   │                                  └────► Cascading (parallel) ◄──────┤
   └─► Rejected(invalid scope)                                           ▼
                                                              Purging ─► Verifying ─► Completed
                                                                 │            │
                                                          Failed(retryable)   └─► CompleteWithResidual
EU-Zustände: Live | Restricted | Erasing(E) | Erased | Orphaned
```

Jeder Übergang: `StageIntent` **vor**, `StageDone` **nach** dem Schritt (Ledger, fsync). Jeder Schritt ist idempotent. Nach Crash bestimmt der letzte Ledger-Eintrag den Wiedereintritt.

### 9.3 Phase 1 – Received/Evaluating

1. `ErasureRequested` ins Ledger (fsync): ab hier ist der Auftrag angenommen.
2. Scope-Auflösung (Snapshot `S`): `Tenant` ⇒ alle Cohorts; `DeletionGroup` ⇒ EUs der Gruppe (Keyrings); `Eu(s)` ⇒ validiert.
3. **Abhängigkeits-Closure** aus `SYS_DERIV` (transitiv, Tiefenlimit, Zyklenerkennung) **vor** der Schlüsselvernichtung, da danach die Kanten unlesbar sind.
4. **Hold-Check** (`held` im Keyring + Ledger-Holds): trifft ein Hold ein Element, wird die gesamte Anfrage `Blocked`, die Mengenauflösung eingefroren. Policy `restrict_on_block` versteckt die EUs trotzdem (`Restricted`, unzerstört).

### 9.4 Phase 2 – Logische Löschung (G0)

Ein `Control`-Record (`Urgent`, nächster Batch, typisch < 1 ms bis wenige ms): `EraseGate { req, tags[] (inkl. Closure), E = commit_seq }`. Beim Apply nach `fsync`:

- `EuStateTable`: alle Tags → `Erasing(E)` (atomarer `ArcSwap`). **Das ist `t_logical`.**
- Idempotenz-Einträge invalidiert, offene Cursors ungültig, Key-/Meta-/Block-/Plaintext-Caches `evict_eu(tag)`, `KeyHandle.revoked = true`, Ephemeral-Daten verworfen und genullt.
- Laufende Compactions/Flushes prüfen `erasure_epoch` beim Install.
- Neue Writes/Reads auf die EU: `Erased`. `ChangeFeed` liefert für diese EUs nur noch `Redacted`/`Erased`.
- Quelle und Abgeleitetes werden im selben Record gegatet: Sichtbarkeit atomar (I18).

### 9.5 Phase 3 – Kryptografische Löschung (G1)

1. Ledger `StageIntent(KeysDestroying)`.
2. **Cohort-Rewrap** (§9.5.1) je betroffener Cohort. Tenant-Scope: alle Cohort-KEKs und die `tenant_tag_key`-Generation vernichten, kein Rewrap.
3. `destroy_kek` liefert `DestroyAttestation`. `KmsScheduled{until}` ⇒ Auftrag bleibt `KeysDestroying(pending_until)`; der Receipt weist das als Restrisiko aus.
4. Ledger `KekDestroyed`, **Anchor-Advance** `(ledger_seq, head, erasure_watermark)`. Erst danach `KeysDestroyed`, `t_crypto`. Die Mindeststärke (`LocalWipe | KmsScheduled | KmsDestroyed`) ist konfigurierbar; der Nachweis nennt die tatsächlich erreichte Stärke wörtlich.
5. Alte Keyring-Generationen, auch in Backups, sind wertlos (KEK existiert nicht mehr). Metadaten, Indizes, Section-Meta, Blooms, WAL-OpParts der EU sind ohne DEK unlesbar (I2).

**9.5.1 Cohort-Rewrap**
1. Cohort-Lock (aufsteigend nach Cohort-Id). `create_kek` ⇒ `KEK_{g+1}`.
2. Alle Einträge mit `KEK_g` lesen, Einträge der zu löschenden EUs **auslassen**, Rest mit `KEK_{g+1}` wrappen (nur 32-Byte-DEKs).
3. `cohort-<id>-g<g+1>.keys` schreiben, `sync_data`, `sync_dir`, Manifest-Edit.
4. ≥ 2 Keyring-Replikate synchron aktualisieren.
5. Ledger `StageDone(KeyRewrite)`. **Erst jetzt** `destroy_kek(KEK_g)`, Ledger `KekDestroyed`, Anchor-Advance, alte `…-g<g>.keys` unlinken.
6. Crash zwischen Schritten: Roll-forward (idempotent).

Kosten ≤ 1024 Wraps + 2 KMS-Calls, unabhängig von der Datenmenge. `Urgency::Batched{window}` bündelt Anfragen derselben Cohort.

### 9.6 Phase 4 – Kaskade und physische Bereinigung (G2)

**Cascading (parallel):** Jede registrierte Extension erhält `on_gate(tags, E)` (sofort, ohne I/O) und `on_erasure(notice)` (mit Deadline). Bis zum `ExtAck{digest}` sind ihre Segmente `Tainted` und durch `filter_visible` abgedeckt. Fristüberschreitung ⇒ `CompleteWithResidual` + Alarm.

**Purging (Lane `Urgent`, Reserve nutzbar):**

1. **WAL:** Betroffene Partitionen force-flushen (`flushed_seq` ≥ letzte Seq der EUs), Segmente rollen, vollständig geflushte Segmente `overwrite_zero` + `unlink`.
2. **Runs/Blobs/Ext-Segmente, Standardpfad In-place-Scrub:**
   1. Dateien über Directory/Bloom-Sketch identifizieren, `PurgeProgress{file_ids, tag_digest}` ins Ledger.
   2. Sektionsbytes (bzw. Blob-Frames der Tags) mit Nullen überschreiben, `sync_data`.
   3. Directory-Eintrag auf `dead` setzen, `eu_tag` nullen, `entry_crc` neu, `sync_data`.
   4. Dateien, die **nur** Tags der Löschmenge enthalten, werden direkt retired (`unlink`).
   Reihenfolge ist crash-sicher: Ein Crash zwischen 2 und 3 hinterlässt eine genullte Sektion mit lebendem Eintrag, die der Fixpunkt-Verifier erneut scrubbt (idempotent). Kosten: O(EU-Größe), kein neuer Platz, keine Fremddaten berührt.
3. **Fallback Purge-Rewrite (verbatim, kein Decrypt):** Wenn `FsCaps.inplace_overwrite_reliable == false`, bei Platzrückgewinnung oder Partition-Split: neue Datei = Header' · alle Sektionen/Frames mit Tag ∉ T (Bytes 1:1) · neues Directory · Footer; Frame-CRCs werden beim Kopieren geprüft (Scrub-Effekt), AEAD nicht (kein Schlüssel nötig); fsync → Manifest-Edit (`ReplaceRun`) → publish → Alt-Datei in Retire-Queue. `purge_grace` (60 s) gilt nur für diesen Pfad: pinnende Snapshots erhalten `SnapshotTooOld` für Reads auf diese Partition (Überlebende lesen mit neuer Version erneut).
4. **Katalog:** nach Edit-Log-Rollover alte `MANIFEST-*` löschen (EU-frei, hygienisch).
5. **Caches:** bereits in Phase 2 evictet, Verifier prüft erneut.

**Verifying (Fixpunkt):**
1. Tag-Scan aller Dateien der betroffenen Partitionen und aller Retire-Dateien (Directory/ChunkDirectory/Frame-Header, Klartext, billig) ⇒ Rest-Menge `R`.
2. `R` leer ⇒ weiter; sonst `PinnedByReader` (zeitlich begrenzt) oder neuer Scrub/Rewrite-Durchlauf.
3. WAL-Scan: kein Record mit Tag ∈ T in aktiven Segmenten.
4. Keyring-Scan: kein Eintrag; KMS `kek_status` aller alten KEKs ≠ `Live`.
5. Ext-Acks vollständig, `verify_absent` je Segment.
6. Ledger `PurgeVerified{residual_digest}`, dann `Completed`, `Anchor::advance`.

### 9.7 Holds und Restrictions

```rust
pub struct Hold { pub id: HoldId, pub scope: HoldScope, pub kind: HoldKind /* LegalHold | Restriction */,
                  pub placed_by: Principal, pub reason_ref: OpaqueRef, pub created: UnixMillis, pub review_at: Option<UnixMillis> }
```

- **Platzierung** nur mit `Authority::verify(HoldOp::Place, sig)`; verankert im Ledger **und** im `held`-Flag des Keyring-Eintrags.
- `LegalHold` (= B `NoErase`): Löschaufträge ⇒ `Blocked`; Schlüssel und Frames bleiben unangetastet (I7); Reads normal.
- `Restriction` (= B `NoErase+NoRead`): Reads ⇒ `Restricted` (außer mit `LegalAccessCap`, von der Authority signiert), Writes blockiert.
- **Release** nur mit gültiger Authority-Signatur einer **fremden** Principal (≠ Requester, Vier-Augen). Kein `force`, kein Bypass, keine Konfigurationsoption. `review_at` löst nur ein Ereignis aus, nie einen Auto-Release.
- **Durchsetzung in drei Schichten:** (1) `ErasureCoordinator`, (2) `KeyringWriter::drop_entry` verweigert `held`-Einträge, (3) Scrub/Rewrite/`unlink` verweigern Dateibereiche mit `held`-Tags.
- Hold auf bereits `KeysDestroyed`-EU: `TooLate`, im Ledger protokolliert, nichts wird wiederhergestellt. Nach Release setzt ein blockierter Auftrag automatisch bei `Evaluating` an.

### 9.8 Backup, Checkpoint, Restore, Rollback-Schutz

**Backup/Checkpoint:** `begin_backup()` flusht, pinnt Manifest/Version und schreibt `{manifest_epoch, ledger_seq, anchor_state}` in den Header. Gesichert werden nur Datei-Bytes (Ciphertext). **Keyring-Backups laufen in getrennter Kette „nur aktuelle Generation"**, nie im Daten-Checkpoint. Ledger und Anchor liegen außerhalb des Daten-Restores. Harte Links werden vom In-place-Scrub mitgeschwärzt (gewollt: Löschung gewinnt); Reflink-/Kopie-Checkpoints bleiben unverändert und sind nur Ciphertext ohne Schlüssel.

**Restore-Reconcile** (automatisch beim `open`, vor jedem Read):
1. Erkennung (§8.8 Schritt 3). Zustand `RestoreQuarantine`: nur Ledger/Katalog lesbar.
2. Das Ledger des Checkpoints muss ein **Präfix** des aktuellen Ledgers sein, `Anchor::read` muss konsistent sein, sonst `RollbackDetected`. Ein Checkpoint, dessen Ledger-Stand neuer ist als der Anker, wird abgelehnt.
3. Für jeden Erased-Tag im Ledger: Tag-Scan der restaurierten Dateien, Scrub/Rewrite, Zustand erneut `Erased` (I8). Die Keyring-Generation kennt die EU nicht mehr, der Ciphertext ist ohnehin unlesbar.
4. Holds und offene Aufträge aus dem Ledger wiederherstellen.
5. Ledger `RestoreReconciled`, erst dann `Running`.
6. **Anchor-Pflicht:** `advance` ist CAS und monoton; wer Ledger **und** Daten zurückspielt, scheitert am Anchor (`LedgerRollback`).
7. **Ehrliche Grenze:** Wird zusätzlich der Anchor-Dienst zurückgesetzt, bleibt nur kryptografischer Schutz (KMS-Zerstörung).

**Backup-Horizont (Receipt-Zusatzspur):** `BackupCatalog` liefert `oldest_backup_ts`. Alle Backups vor `t_crypto` enthalten nur unlesbaren Ciphertext: `backup_status: Unreadable(since t_crypto)`, bei Policy mit physischer Expiry `PendingExpiry(until)` bis `BackupHorizonReached` ⇒ Addendum.

### 9.9 Receipts

```rust
pub struct ErasureReceipt {
    pub req: ReqId, pub scope_tag_digest: [u8; 32],                 // BLAKE3 über sortierte Tags
    pub t_requested: UnixMillis, pub t_logical: UnixMillis, pub erase_epoch: Seq,
    pub t_crypto: Option<UnixMillis>, pub t_physical: Option<UnixMillis>, pub t_verified: Option<UnixMillis>,
    pub guarantee: GuaranteeLevel,                                  // G0 | G1 | G2
    pub key_strength: DestroyStrength,                              // tatsächlich erreicht
    pub stages: Vec<StageEvidence>, pub kek_attestations: Vec<DestroyAttestation>, pub ext_acks: Vec<ExtAckEvidence>,
    pub verifier_report_digest: [u8; 32], pub ledger_head: LedgerRef, pub anchor: AnchorAttestation,
    pub holds_encountered: Vec<HoldRef>,
    pub status: ReceiptStatus,                                      // Complete | CompleteWithResidual | Blocked | Failed | AnchorPending
    pub residual: Vec<ResidualRisk>, pub backup_status: BackupStatus,
    pub signature: Vec<u8>, pub signer: KeyId,
}
pub fn verify_receipt(r: &ErasureReceipt, ledger: &LedgerSlice, att: &AnchorAttestation, pk: &PublicKey) -> Verdict; // rein, offline
```

`verify_receipt` prüft Signatur, Hashkette des Ledger-Ausschnitts, Anchor-Attestation, Stufenreihenfolge und Zeitmonotonie. Das Ticket führt `Gated → KeysScheduled → KeysDestroyed → Purged → Completed` einzeln abfragbar (`ticket.wait(stage, deadline)`).

---

## 10. Erweiterungen und RAG-Schicht

### 10.1 Extension-Vertrag

```rust
pub trait SegmentOwner: Send + Sync {
    fn ext_id(&self) -> ExtId;
    /// Sofort nach durablem Gate, ohne I/O, nicht blockierend: Treffer dieser Tags ab jetzt unterdrücken.
    fn on_gate(&self, tags: &[EuTag], at: Seq);
    /// Physisch bereinigen (In-Memory, Postings, ANN-Knoten, Segmente neu schreiben/verwerfen) bis deadline.
    fn on_erasure(&self, notice: &ErasureNotice) -> ExtProgress;       // Done{digest} | Pending{retry_after}
    fn verify_absent(&self, tag: EuTag) -> bool;                       // für Orakel/Verifier
    fn watermark(&self) -> Seq;                                        // bis wohin der ChangeFeed verarbeitet ist
    fn segments(&self) -> Vec<SegmentInfo>;
}
pub enum SegmentClass {
    EuPure,                           // Löschung = unlink
    Mixed,                            // Frames tragen eu_tag (ExtStore-Format) ⇒ Engine kann scrubben/purgen
    Rebuildable { max_age: Duration },// abgeleitet (z. B. HNSW-Graph), wird nach Löschung neu gebaut; Ticket trägt die Frist
    NoPersonalData,                   // deklariert, vom Canary-Oracle stichprobenartig geprüft
}
pub trait ExtStore { fn open_segment(&self, class: SegmentClass) -> Result<ExtSegmentWriter>; }
impl ExtSegmentWriter {
    pub fn append(&mut self, eu: EuId, payload: &[u8]) -> Result<FrameRef>;   // versiegelt mit k_ext(ext_id)
    pub fn seal(self) -> Result<SegmentId>;                                   // Manifest-Edit, atomar
}
```

- Einziger erlaubter Persistenzweg ist `ExtStore`. Segmente stehen im `SegmentRegistry` des Manifests. Abmelden nur über eine ledger-protokollierte Operation. Das Ticket ist erst `Completed`, wenn jedes registrierte Segment `Done` meldet.
- **Verbindlichkeit:** Jedes Extension-Ergebnis läuft vor Rückgabe durch `Snapshot::filter_visible`.
- **ChangeFeed:** `poll(since, max)` liefert `Put | Delete | Redacted | Erased` in Seq-Reihenfolge, nie Inhalte gegateter EUs. Die `watermark`s der Extensions halten WAL und Runs nicht länger als nötig fest.
- Extensions speichern bevorzugt **als EU-Sektionen im Kern** (eigene Spaces), dann erledigt der Kern die Löschung.

### 10.2 Indexschicht (`lethe-idx-*`, Extensions)

- **Vektor:** Segmente pro EU (`EuPure`) oder gemischt (`Mixed`), Disk-ANN-artig mit quantisierten Codes im Hot-Bereich und Re-Ranking aus dem Cold-Bereich; **In-Memory-Delta-Flat-Index** (SIMD, bis ca. 50k Vektoren) deckt noch nicht gesealte Writes ab (kein Index-Lag); Löschungen über versionierte Deletion Vectors. Mehrere Embedder/Versionen je Namespace, Modellwechsel = neue Lane + Re-Embedding über die Work Queue. `unsafe` nur im SIMD-Distanzkernel-Crate, isoliert.
- **FTS:** segmentbasierter invertierter Index, BM25, Positionen, eigener Suchtokenizer (unabhängig von LLM-Tokenizern).
- **Graph:** Adjazenz als Space-Schlüssel `(src, edge_type, dst)` + Rückrichtung innerhalb der EU; begrenzte k-Hop-Expansion mit Fan-out-Budget, PPR-Approximation. Kanten **zwischen** EUs sind `Reference`-Links der Ctx-Schicht und kaskadieren nicht.
- Alle Indizes arbeiten pro EU bzw. Deletion Group; **Cross-EU-Suche** ist eine Fan-out-/Merge-Operation der Ctx-Schicht über eine explizite EU-Menge (z. B. Nutzer-EU + geteilte Organisations-EU), nie ein globaler Scan (D5).

### 10.3 Ctx-Schicht: Cell-Modell

Eine **Cell** wird auf Entries abgebildet:

| Cell-Teil | Ablage |
|---|---|
| Rohinhalt (L0) | Space `<ns>`, Key = `CellKey` (UUIDv7/ULID), Body, `weights.tokens`, `lanes` (valid_from/to) |
| Zoomstufen L1/L2 | Space `<ns>.z1`, `<ns>.z2` mit gleichem Key, jeweils eigener Token-Gewichtung und `header = producer/derived_from` |
| Attribute/Tags/Kanten-Kurzliste | `header` (≤ 1 KiB) bzw. Graph-Space |
| Trust | `flags.trust` (3 Bit) |
| Token-IDs (optional) | Space `<ns>.tok`, nur mit `boundary_safe`-Flag direkt konkatenierbar |
| Embeddings | Vektor-Extension |
| Zugriffsstatistik | `SYS_ACCESS` (sealed, Sidecar, ändert nie Cell-Versionen) |

Chunking (strukturbewusst: Markdown, Code per Syntaxbaum, JSON/Tabellen) und Tokenizer-Familien sind Namespace-Policy der Ctx-Schicht. Die Chunking-Version wird pro Cell vermerkt (reproduzierbare Re-Chunk-Jobs).

**Handles:** kurze Crockford-Base32-Aliase, **pro EU eindeutig und monoton** vergeben (`SYS_ALIAS`), validiert bei jedem Zugriff, `did_you_mean` über einen Edit-Distanz-Index.

### 10.4 Context Engine

| Operation | Zweck |
|---|---|
| `recall(q, budget, policy)` | beste Kontextmenge unter Token-Budget über eine EU-Menge |
| `window(seq, anchor, budget)` | zusammenhängendes Fenster, O(log n) über `cum_tokens` |
| `expand(handle, level)` | feinere/gröbere Stufe nachladen |
| `explain(recall_id)` | Signalbeiträge, gewählte Stufe, Verwerfungsgründe |
| `neighbors(handle, edge_types, fanout)` | Graph-Nachbarschaft |
| `diff(a, b)` | lesbare Änderungszusammenfassung (Branches/Zeitpunkte) |

**Pipeline:** Kandidaten (Vektor-Head-Codes, BM25, Graph, Recency; Ranking nur auf Hot-Bereich) → Score-Fusion (RRF/gewichtet, Trust/Recency multiplikativ) → **Zoom-Knapsack** → Ordnung → Rendering.

**Zoom-Knapsack (Multiple-Choice-Knapsack):** Wähle je Kandidat *i* genau eine Stufe ℓ (oder „weglassen") mit Kosten `c[i][ℓ]` (Token) und Nutzen `u[i][ℓ] = rel_i · gain(ℓ) · trust_i · fresh_i`, Summe der Kosten ≤ Budget, Redundanzabzug (MMR) dynamisch. Praxisverfahren: obere konvexe Hülle je Kandidat, Start auf gröbster Stufe, Greedy über Upgrade-Schritte mit größtem Δ-Nutzen/Δ-Kosten (Max-Heap), bis das Budget erschöpft ist (optimal für die LP-Relaxation). Ergebnis: viele grobe Treffer + wenige feine statt weniger voller Chunks.

**Cache-freundliche Ordnung (optional):** stabile Cells (System, Skills, Policies) zuerst in kanonischer Reihenfolge, Volatiles danach; die Antwort trägt einen **Prefix-Digest**. Das Digest-Material enthält keine EU-übergreifenden Identitäten.

**Progressive Disclosure:** `recall` liefert kompakt mit Handles → Agent entscheidet → `expand`. Expand-Aufrufe sind ein Relevanzsignal im EU-gesiegelten Access-Trace.

### 10.5 Derivation Cache, Lineage, Work Queue

- **Derivation Cache** pro EU (`SYS_DCACHE`): Schlüssel `HMAC(eu_dedup_key, source_content_hash ‖ producer_id)`; Embeddings, Token-Zählungen und extraktive Zoomstufen werden nie doppelt berechnet; Branches derselben EU teilen ihn. **Kein EU-übergreifendes Dedup.**
- **Staleness per Konstruktion:** Jede abgeleitete Facette trägt `source`-Hash. Weicht er ab, ist sie veraltet; je Policy aus Recall ausgeschlossen oder mit Warnung geliefert; Neuberechnung geht in die Work Queue.
- **Work Queue:** Die Engine ruft kein Modell auf (D9). Tasks `Embed | Summarize | Consolidate | Dedupe | Reconcile` werden per `lease(worker, kinds, limit, ttl)` (at-least-once) an externe Worker vergeben; Ergebnisse sind normale Commits mit `derived_by = (modell, prompt_hash)`. Task-Payloads sind EU-versiegelt; Tasks erloschener EUs werden verworfen.
- **Retention/Zoom-Down (logisch, §4.4):** Hot → Warm (nur Tiering, Phase 9), Zoom-Down (L0 per Tombstone entfernen, L1/L2 behalten), Tombstone-Spur. Salience-Score aus Recency, Zugriff, Wichtigkeit, Graph-Zentralität, Expand-Rate; „pinned" Cells sind geschützt. **Das ist kein Erasure.**

### 10.6 Trust und Capabilities

- **Trust-Stufen:** `System > Principal(User) > Tool-Verified > Tool-Unverified > External-Untrusted`. `derive` setzt `trust = min(sources)` (I23). Hochstufen nur durch privilegierte Principals (Ledger-/Audit-Eintrag). `recall` kann `min_trust` filtern oder untrusted Inhalte markiert und eingerahmt ausgeben.
- **Capabilities:** Macaroon-artige, attenuierbare Tokens (Caveats: Tenant, EU-Menge, Spaces, Branches, Operationen, Trust-Obergrenze, TTL, Token-Budget je Zeitraum). Hauptagenten können Subagenten eingeschränkte Tokens ausstellen. Capabilities begrenzen nie die Löschpflicht: `erase` ist eine eigene Operation mit Principal-Prüfung.
- Der Access-Trace ist **EU-gesiegelt** (oder nur im RAM keyed nach `eu_tag`) und wird beim Gate verworfen (K24).

### 10.7 MCP und Fehler für Modelle

Wenige, mächtige Tools: `mem_recall`, `mem_expand`, `mem_write`, `mem_fork/merge/discard`, `mem_history/diff`, `mem_describe` (budgetiert, z. B. 800 Token). Tool-Schemas werden aus Rust-Typen generiert (`schemars`). Fehler folgen einem LLM-lesbaren Schema `{error, message, retryable, did_you_mean?, hint?, suggested_call?}`; `Erased`/`Restricted` werden gemäß `error_masking` ausgegeben.

### 10.8 Optional: Prefix-/KV-Cache-Store (Phase 9)

Nur als Extension, EU-scoped und versiegelt (KV-Blöcke sind aus personenbezogenem Prompt abgeleitet): Schlüssel `(model_id, tokenizer_id, Hash-Kette über Token-Blöcke)` **innerhalb der EU**, kostenbasierte Eviction (`Token × Modellkosten`, Frequenz, Alter), Invalidierung per Hash-Kette. Geteilte stabile Präfixe (System-Prompts) liegen in einer eigenen `NoPersonalData`-EU.

---

## 11. Concurrency

### 11.1 Lock-Ordnung (strikt aufsteigend, Verstoß ⇒ Debug-Assert via `LockLevel`-Token)

| Ebene | Primitive | Zweck |
|---|---|---|
| L0 | `LifecycleGate` (RwLock) | Open/Close/Poison; alle halten *Read* |
| L1 | `ErasureCoordinator` (Mutex je `ReqId`) | Zustandsübergänge; nie über KMS hinaus außer Rewrap (L3 darunter) |
| L2 | `CatalogWriter` (Mutex) | Manifest-Edits; kurz, nur Manifest-Append |
| L3 | `KeyringWriter[cohort]` (Mutex, aufsteigend nach Cohort-Id) | Rewrap, DEK-Anlage; KMS-Calls nur mit Timeout |
| L4 | `PartitionMaintenance[p]` (Mutex, aufsteigend nach `PartitionId`) | Flush/Compaction/Scrub-Install, Install-Check |
| L5 | `LedgerWriter` (Mutex) | Ledger-Append, blattnah |
| L6 | `FdCache`/`CacheShard[i]` (Mutex, sharded) | Blatt, nie über I/O gehalten |
| — | **Sequencer** | WAL-Writer, `EuStateTable`-Mutation: Single-Thread-Owner (Channel), *kein* Lock |

**Regeln:** (1) nie niedrigere Ebene nehmen, solange eine höhere gehalten wird; (2) **kein Lock über Extension-Callbacks** (Extensions dürfen die Read-API re-entrant nutzen); (3) blockierende Ports nur mit Timeout und nur unter L1/L3; (4) Eigentumsmodell statt Locks wo möglich (Sequencer besitzt WAL und EU-State, Flush-Jobs ihre Memtable).

**Lock-freie Lesepfade:** `ArcSwap<Version>`, `ArcSwap<EuStateTable>`, `ArcSwap<KeyTable>`, `AtomicU64` für `visible_seq`/`erasure_epoch`, `AtomicBool` für `KeyHandle.revoked`, Slot-Array für die Snapshot-Registry. Publish-Reihenfolge: `Version`, dann `visible_seq` (Release); Leser laden `visible_seq` (Acquire), dann `Version`.

### 11.2 Kritische Races

| ID | Race | Auflösung |
|---|---|---|
| R1 | Read sieht `Live`, Gate kommt vor Rückgabe | Check-after-decrypt/-read gegen `revoked` und `erasure_epoch`; Linearisierungspunkt = erster `EuStateTable.load`. |
| R2 | Writer hat mit altem DEK vorversiegelt, Gate inzwischen verarbeitet | Sequencer prüft EU-Zustand serialisiert ⇒ `Erased`. |
| R3 | Flush/Compaction mergt EU, die gerade gegatet wird | Install-Check unter L4 + `erasure_epoch`; bereits publizierte Fälle fängt der Verifier-Fixpunkt. |
| R4 | WAL-Truncate vs. ungeflushte Partition | Truncate nur bei `flushed_seq` **aller** Partitionen, Manifest-Edit zuerst. |
| R5 | Neue EU in Cohort während Rewrap | Beide unter L3; neue DEKs in die *neue* Generation. |
| R6 | Backup vs. Löschung | `BackupGuard` pinnt Version + Watermark; Reconcile zieht nach. |
| R7 | Cursor vs. Purge/Scrub | Cursor referenziert `Seq`, nicht Datei; `SnapshotTooOld` nur bei abgelaufenem `snap_seq`. |
| R8 | Idempotenter Retry vs. Erase | `SYS_IDEM`-Einträge in Phase 2 invalidiert ⇒ `Erased`. |
| R9 | Extension-Ack vs. neue Writes | Writes auf Erased-EUs abgelehnt (R2), keine neuen Ableitungen. |
| R10 | Doppelte/überlappende Aufträge, Gate-Crash vor Publish | `ReqId`-Idempotenz, Überlappung hängt sich an `Erasing(E)`; Recovery wendet `EraseGate` an, bevor ein Read möglich ist. |
| R11 | `fsync`-Fehler während Flush/Publish | `Poison`-Flag vor jedem Publish geprüft. |
| R12 | `rotate` während Commit | `KeyHandle` trägt Epoche; Sequencer akzeptiert Epochen ≤ aktuelle. |
| R13 | In-place-Scrub vs. paralleler Compaction-Read | Reads gegateter EUs scheitern erwartbar und werden übersprungen; Install-Check verwirft Output. |
| R14 | Checkpoint vs. Scrub | Kopien sind eigenständig; harte Links werden mitgescrubbt (dokumentiert). |

---

## 12. Invarianten (konsolidiert)

| ID | Aussage |
|---|---|
| I1 | **Reinheit/Tag-Vollständigkeit:** Jeder Frame/Sektion/Blob-Chunk gehört zu genau einer EU und trägt deren korrekten `eu_tag` (bzw. liegt in einem Sektionsabschnitt, den das Directory dem Tag zuordnet). |
| I2 | **Schlüsselabschluss:** Jedes persistente Byte aus Nutzdaten (Keys, Header, Bodies, Gewichte, Lanes, Indizes, Blooms, Abhängigkeiten, Idem-Tokens, Access-Trace) liegt in einem Frame unter dem Schlüssel seiner EU. Klartext bleibt nur: `eu_tag`, Längen, Frame-Typ, CRC, Dateistruktur. Es gibt keinen zweiten Entschlüsselungsweg. |
| I3 | **Positionsunabhängigkeit:** Keine AAD und keine versiegelte Referenz enthält Datei-ID, Dateiname, Offset oder `Seq`. |
| I4 | **Manifest-Freiheit:** Das Manifest nennt weder `EuId` noch `eu_tag` noch EU-Beziehungen. |
| I5 | **Seq-Monotonie:** `commit_seq` lückenlos und streng monoton (auch über Neustarts); `visible_seq ≤ durable_seq`. |
| I6 | **Erasure-Barrier:** Nach durablem `EraseGate(E)` liefert kein Read/Cursor/Feed/Extension-Treffer Daten der EU, unabhängig vom Snapshot; pro EU nie Mischsicht. |
| I7 | **Hold-Sicherheit:** Eine EU mit aktivem Hold verliert nie DEK oder Frames. |
| I8 | **Keine Wiederbelebung:** Ein Tomb im Ledger wird nach jedem Recovery/Restore nie wieder `Live`; `EuId` nie wiederverwendet. |
| I9 | **Ledger-Vorrang:** Gate < `StageIntent` < Rewrap < `destroy_kek` < Attestation im Ledger < Anchor-Advance < `KeysDestroyed`. |
| I10 | **Nonce-Eindeutigkeit:** Kein (Schlüssel, Nonce)-Paar wird zweimal verwendet. |
| I11 | **Atomarität:** Ein Commit ist nach jedem Crash über alle Spaces/EUs/Partitionen vollständig oder gar nicht sichtbar. |
| I12 | **Determinismus:** Recovery ist eine Funktion von (Dateien, Ledger, Anchor, KMS) und idempotent. |
| I13 | **Katalog:** Eine referenzierte Datei existiert und ist vollständig gesynct; Löschen erst nach dem referenzierenden Edit. |
| I14 | **Purge-Vollständigkeit:** Nach `Completed` findet das Orakel in Store-Dateien keine Klartext-, Ciphertext-, Tag- oder Schlüssel-Bytes der EU (außer ausgewiesenem Residual). |
| I15 | **Löschlokalität:** Gelesene und geschriebene Bytes eines `erase` ≤ c · EU-Größe + konstante Metadaten, unabhängig von der Storegröße (Rewrite-Fallback ausgenommen und gekennzeichnet). |
| I16 | **DiskFull-Liveness:** Gate, Krypto und Scrub bleiben bei ENOSPC ausführbar. |
| I17 | **Lock-Disziplin:** Ordnung L0<…<L6; kein Lock über Extension-Callbacks. |
| I18 | **Kaskadenabschluss:** Die Löschmenge ist der transitive `DerivedFrom/Delete`-Abschluss zum Gate-Zeitpunkt, in **einem** Record. |
| I19 | **Rollback-Schutz:** `open` scheitert, wenn Manifest oder Ledger hinter dem Anchor liegen. |
| I20 | **Zeroization:** Schlüssel und Klartext-Puffer sind `Zeroizing`; beim Gate sofort verworfen. |
| I21 | **Tag-Eindeutigkeit:** `eu_tag` ist pro Tenant eindeutig. |
| I22 | **Branch-Reinheit:** Ein Branch und alle seine Einträge gehören zu genau einer EU. |
| I23 | **Trust-Monotonie:** `trust(derived) ≤ min(trust(sources))`. |
| I24 | **Keine Existenz-Lecks nach Erase:** Cursor, Handles, Idem-Einträge und Caches einer gegateten EU liefern nie Inhalt. |

---

## 13. Verifikation

**(a) Formale Modelle (TLA⁺, TLC + Apalache).**
Modell 1 (Commit/Recovery): Group Commit, WAL-Kette, Manifest, Crash an jeder Stelle, Torn Writes. *Safety:* `AckedDurable`, `PrefixConsistency`, `NoPhantomRead`.
Modell 2 (Erasure): Gate, Rewrap, Scrub, Holds, Restore, KMS-Latenz. *Safety:* `ErasedNeverReadable`, `KeyDestroyOrderedAfterIntent` (I9), `HoldSafety`, `NoResurrection`, `CascadeComplete`. *Liveness* unter Fairness: Jeder nicht-blockierte Auftrag erreicht `Completed` oder `CompleteWithResidual`.

**(b) Concurrency:** `loom` (ArcSwap-Publish, Epoch-Checks, Snapshot-Registry, Sequencer-Handoff), `shuttle` (R1–R14 als benannte Szenario-Tests mit erzwungenem Interleaving über die ganze Engine), Lock-Level-Tokens im Debug-Build.

**(c) Deterministische Simulation (`lethe-sim`, ein Seed bestimmt alles).**
- `SimFs`: seiten-/sektorgranulare Persistenz, ungesyncte Writes gehen beim Crash teilweise/umsortiert verloren, EIO/ENOSPC, Short Writes, Bit-Flips, verlorene `rename`s ohne `sync_dir`, **Geister-Journal aller je geschriebenen Seiten** (Remanenzmodell), Byte-Zähler je Operation/Datei (Lokalitätstest I15).
- `SimKms` (Latenz, Ausfall, Karenzzeit, irreversibles `destroy`), `SimAnchor` (CAS, Ausfall, Rollback-Versuche müssen scheitern), `SimExecutor` (seed-gesteuertes Interleaving).
- **Crash-Sweep:** Crash vor und nach jeder I/O-Operation (inkl. Scrub-, Rewrap- und Ledger-Schritte), mehrere Torn-Varianten; danach Recovery, Invarianten, Modellvergleich, zweites Recovery (Idempotenz).
- **Referenzmodell:** `BTreeMap`-MVCC-Orakel inkl. Branches und EU-Zustand; geprüft werden `AckedDurable`, Snapshot-Konsistenz, exakte Paging-Budgets, Idempotenz, Lane-Pruning = Vollscan, Forward = Backward, Rohkopie-Compaction ≡ Merge-Compaction.
- Linearisierbarkeits-Historien (Porcupine/Elle-Stil).

**(d) Echte Infrastruktur:** `dm-flakey`/`dm-log-writes`, `LD_PRELOAD`/FUSE für EIO/ENOSPC, `kill -9`-Schleifen unter Last, KMS-Mock mit Chaos.

**(e) Fuzzing/Property:** `cargo-fuzz` auf alle Decoder und den Recovery-Scanner; Bit-Flip-Differenztest über jedes Byte eines gültigen Frames/Runs/Ledgers (Erwartung: `Corrupt`/Auth-Fehler, nie stille falsche Daten); `proptest` für Codec-Roundtrips, Gewichts-Arithmetik, Fortschrittsgarantie; `Kani` für Bounds/Overflow; `miri` auf `lethe-format`.

**(f) Löschorakel (`lethe-oracle`).** Canaries (Klartext-Marker in Keys, Header, Bodies, Lanes, Abhängigkeiten, Idem-Tokens) plus Key-Canaries (DEK/KEK) plus 16-Byte-Ciphertext-Stichproben je Frame:

| Zeitpunkt | Erwartung |
|---|---|
| jederzeit | Klartext-Canary nie in irgendeiner Seite (auch nicht im Geister-Journal). |
| nach Gate | Jeder Read ⇒ `Erased` (alte Snapshots, Cursors, Retries, Extension-Antworten, ChangeFeed). |
| nach `KeysDestroyed` | Orakel entschlüsselt alle historischen Seiten mit allem Erreichbaren (Keyrings, KMS-Live-Objekte, Sim-Speicherabbild, Backups): kein Erfolg. |
| nach `Completed` | Keine Tags/Canaries/Schlüssel in Store-Dateien, WAL, Katalog-Generationen, Ledger (nur `tomb_id`/Tags), `.tmp`; Geister-Journal darf nur Ciphertext enthalten (ausgewiesen als G1-Schutz); `verify_absent` je Segment `true`. |
| nach Restore | Erased-Tags nicht wieder `Live`; Reconcile entfernt Reste. |
| unter Hold | Keine Vernichtung/Entfernung; `LegalAccessCap`-Reads funktionieren. |
| Überlebende | Nach Scrub/Rewrite/Rewrap bytegleich lesbar (Sektions-Hash vorher/nachher). |

Das Orakel läuft auch *zwischen* den Phasen, nach Crash und Fortsetzung. Zusätzlich: Prozess-Ebene (`gcore` nach Löschung, `KeyMaterialRegistry`, als „best effort" deklariert), Extension-Konformitätssuite (`ext-conformance`, Voraussetzung für den Whitelist-Eintrag), Receipt-Orakel gegen simulierte Ledger-Manipulation (Truncation, Reorder, Fork, Replay).

**(g) Retrieval-Qualität (Ctx-Schicht, CI-Gate).** **Utility@Budget** als Leitmetrik: Trace-Replay aufgezeichneter Agentenläufe (nur synthetische oder EU-gelöschte Daten) gegen Baselines (Top-k fester Chunkgröße; KV-Store + externe Vektor-DB). Metriken: Recall@Budget, Nutzen pro Token, Task-Erfolg bei gleichem Budget, Expand-Rate, Kosten pro Aufgabe; A/B Zoom-Knapsack vs. Top-k und cache-freundliche vs. relevanzbasierte Ordnung.

**(h) CI-Gates:** `cargo-geiger` (Kern `unsafe`-frei), `cargo-vet`/`cargo-deny`, `cargo-mutants` auf Recovery-/Erasure-/Guard-Code (überlebende Mutanten = Testlücke), Fuzz-Smoke (5 min je Ziel), Crash-Sweep (festes Budget pro PR, nächtlich großer Seed-Raum), p99-Benchmarks (Commit, Erase-Gate, Paging-I/O, Scrub-Dauer vs. EU-Größe), 72-h-Soak mit zufälligen Löschanfragen unter Dauerlast.

---

## 14. Roadmap mit vertikalen Durchstichen

Jede Phase verbreitert einen bereits vollständigen Pfad (Schreiben, Lesen, Löschen, Orakel, Crash-Sweep) und darf Orakel- und Crash-Tests der Vorphase nicht brechen. Der Crash-Sweep wird in jeder Phase um die neuen Fs-Operationen erweitert, nie nachgezogen. Dauer: Planungsannahme für 3–4 erfahrene Rust-Entwickler plus 1 Retrieval-Ingenieur, keine Zusage.

| Phase | Durchstich | Exit-Gate | Dauer |
|---|---|---|---|
| **P0 Fundament** | Workspace, Lints, Ports, `MemFs/SimFs` (Grundmodell), Frame-Format + Crypto-Adapter, Orakel-Skelett, CI-Gates; Format-Spezifikation v0 einfrieren; Trace-Recorder für Agentenläufe | Bit-Flip- und Fuzz-Smoke grün; Orakel findet eingestreute Canaries | 3 W |
| **P1 Erase-Walking-Skeleton** | 1 Space, `put`, WAL + Memtable, `scan_page` forward, `erase` (Gate + `LocalKeyService`, Cohort-Größe 1, Ledger ohne Anker), Recovery aus WAL | Crash-Sweep grün; O-Files findet nach `Completed` nichts, obwohl Daten nur in WAL/Memtable lagen | 4 W |
| **P2 Runs, Katalog, Scrub** | Flush, EU-reine Sektionen (Meta/Idx/Hdr/Body), Manifest + `CURRENT`, In-place-Scrub, WAL-Ablöse, vollständige Recovery | O-Files nach Erase für Daten in Runs grün; Crash-Sweep inkl. Flush/Scrub; Modelltest grün | 5 W |
| **P3 Agenten-Primitive** | Gewichte, Lanes, Backward-Paging, Blobs, `Guard`/`IdemKey`, `Ephemeral`, Snapshot-Lease, Group Commit, Aggregat-Fast-Path, `inline_max`-Kalibrierung | Paging-Propertytests; Commit-Benchmark (p99); Erase unter Last | 5 W |
| **P4 Compaction + Notbetrieb** | Leveling, Rohkopie, Merge, MVCC-GC, Branch-Overlay + `RetentionPin`, `DiskFull`-Modus + Reserve | Lokalitätstest I15 bei ×1/×10/×100; ENOSPC-Sweep: Erase bleibt ausführbar | 5 W |
| **P5 Anchor, Cohorts, Restore** | Cohort-Keyrings + Rewrap, Ledger-Hashkette + Anchor-Adapter (Datei-Referenz), Checkpoint/Restore/Reconcile, KMS-Adapter (`KmsScheduled/Destroyed`), Receipts + `verify_receipt`. **Formatfreeze** (ab hier `format_version` + Migrationspflicht) | Restore alter Images erzeugt keine lebenden gelöschten EUs; `RollbackDetected`-Tests; Receipt-Orakel grün | 5 W |
| **P6 Kaskaden, Holds, Extensions** | Links/`SYS_DERIV`, `SegmentOwner`/`ExtStore`/`ChangeFeed`, Referenz-Extension (Mini-ANN + Mini-BM25), Holds (Authority, Vier-Augen), Purge-Rewrite-Fallback, Partition-Split | Erase eines Wurzel-EU räumt alle abgeleiteten Strukturen im selben Gate; O-Segments grün; Feed nie mit Inhalt nach Gate | 5 W |
| **P7 Ctx-Schicht + Indizes** | Cells/Zoom-Spaces, Handles, Segment-Vektorindex + FTS + Graph, Hybrid-Fusion, Zoom-Knapsack, `window`/`expand`/`explain`, Branch-Merge + ConflictCells, Trust, Derivation Cache, Work Queue, Capabilities, MCP | Utility@Budget signifikant über Top-k-Baseline (Replay); Extension-Konformität grün; Go/No-Go | 12 W |
| **P8 Härtung** | `loom/shuttle` R1–R14, Fuzz-Kampagnen, Mutationstests, 72-h-Soak, TLA⁺-Modelle final, maschinenlesbarer Erasure-Report | Jede Invariante I1–I24 hat mindestens einen automatisierten Test; keine offenen Korruptionsbefunde | 4 W |
| **P9 Optional** | `Replicated`-Durability/`ReplicationHook`, Objektspeicher-Tiering (nur versiegelte, EU-reine Dateien, Rewrite-Löschpfad), Prefix-/KV-Store, Co-Retrieval-Layout innerhalb Deletion Groups, WASM-Merge | je Feature eigenes Gate; keine Regression der Orakel | – |

**Prioritäten:** Erst jede Invariante testbar machen, dann optimieren (Cache-/Blockgrößen frühestens ab P3 mit Messwerten). Format-Änderungen sind bis P5 erlaubt.

---

## 15. Default-Parameter (Startwerte, per Config überschreibbar)

| Parameter | Default | Anmerkung |
|---|---|---|
| `max_batch_bytes` / `max_delay` | 4 MiB / 50–500 µs adaptiv | bei geringer Last sofort |
| WAL-Segment | 64 MiB vorallokiert | A/B-Konflikt: weniger Fds |
| `inline_max` (Body) | 4 KiB | zwischen A (1 KiB) und B (16 KiB); in P3 per Benchmark festlegen |
| Idx-Block (Klartext) | 16 KiB | |
| Blob-Chunk (roh) | 64 KiB, zstd Level 3 | |
| `section_align` | 512 B (4 KiB optional) | sektorgenaues Scrubben |
| `header` max / Default-Cap | 1 KiB / 256 B | |
| Key max | 1 KiB | |
| Cohort-Größe | ≤ 1024 EUs | 1 = KMS-Objekt pro EU |
| `max_wal_age` | 5 min | |
| Snapshot-Lease / `max_snapshot_age` | 60 s / 10 min | |
| `purge_grace` (nur Rewrite-Pfad) | 60 s | |
| `purge_deadline` | Politik (Empfehlung ≤ 24 h Immediate, ≤ 30 Tage Batched) | |
| Lösch-Reserve | 256 MiB je Volume + 16 MiB Ledger | |
| `Urgent`-Mindest-I/O-Anteil | 20 % | |
| Key-Cache-TTL | 5 min (`evict_eu` sofort) | |
| Keyring-Replikate | ≥ 2, getrennte Failure Domains | |
| KMS-Mindeststärke | konfigurierbar, Default `KmsDestroyed` für strenge SLAs | |
| Anchor-Policy | `strict` | |
| Exploration Re-Layout (P9) | 5 % | |

**Design-Ziele (Hypothesen, zu messen):** Fork < 50 µs; Commit (1–100 Entries, NVMe, `Sync`) p99 < 2 ms; Erase-Gate-Latenz typ. < 5 ms; `window()` über 10⁶ Einträge < 1 ms ohne Bodyzugriff; Recall (1 M Cells, warm, 8k-Budget, hybrid) p50 < 5 ms / p99 < 25 ms; Recovery < 5 s bei 1 GiB WAL. Alle Werte sind Zielgrößen, keine Messungen.

---

## 16. Restrisiken und offene Entscheidungen

### 16.1 Restexpositionen (in jedem Receipt ausgewiesen)

| Bereich | Restrisiko | Maßnahme/Zuständigkeit |
|---|---|---|
| Prozessspeicher, Swap, Core-Dumps | Klartext bis Zeroize | `Zeroizing`, Cache-Drop beim Gate, `MemLock`/`DONTDUMP` im Adapter; Betreiber: Swap verschlüsseln, Dumps aus |
| SSD-Wear-Leveling, FS-Journale, COW-/Volume-Snapshots | Überschriebene Blöcke überleben | Dort liegt nur Ciphertext; tragend ist G1. `overwrite_zero` ist Best-Effort |
| Page-Cache | kurze Remanenz | nur Ciphertext; optional `O_DIRECT`-Adapter |
| KMS-Karenzzeit | Schlüssel bis Fristende wiederherstellbar | `KmsScheduled` ⇒ `CompleteWithResidual`; für strenge SLAs KMS mit sofortiger Zerstörung |
| Schlüsselkopien (Escrow, HSM-Backups) | Betreiberentscheidung | in Destroy-Policy einrechnen |
| `eu_tag` alter Kopien | Pseudonym, 64 Bit | teilbar nur mit `tenant_tag_key` (bei Tenant-Löschung vernichtet); Re-Identifikation nur mit Kandidaten-`EuId` + Schlüssel |
| Längen-/Zeit-Seitenkanäle | Dateigrößen, Frame-Längen | `pad_to` optional |
| Backups | Ciphertext alter Generationen | unlesbar ab `t_crypto`; physische Expiry per Policy |
| Obere Schichten | LLM-Provider, Prompt-/KV-Caches, App-Caches, Fine-Tuning, Modellgewichte | **nicht** im Kern lösbar; nur über `Link`/`SegmentOwner` angebundene Daten werden mitgelöscht |
| Extension-Vertragsbruch | nicht beweisbar | Konformitäts-/Canary-Tests, keine Garantie |
| Fd-gepinnte gelöschte Dateien | Reader hält Datei | `purge_grace`, danach Invalidierung |
| Sub-EU-Löschung | nur logisch (§4.4) | eigene EU verwenden |

### 16.2 Offene Entscheidungen (mit Vorschlag)

| Entscheidung | Optionen | Vorschlag |
|---|---|---|
| AEAD | XChaCha20-Poly1305 vs. AES-256-GCM-SIV | XChaCha20-Poly1305 (zufällige 192-Bit-Nonce) |
| Ordnung nur pro EU (D5) | – | bleibt; Cross-EU-Fan-out in Ctx-Schicht |
| Anchor-Backend zuerst | Datei, TPM-NV, Object-Lock/Transparency-Log | Datei-Referenz in P5, produktiv je Betreiber eine externe Option |
| Single-Sequencer-Grenze | ein Core für Prüfung/Serialisierung | Skalierung über Tenant-Sharding mehrerer Engine-Instanzen, keine Cross-Instanz-Atomarität |
| Eigener FTS vs. Tantivy | Segmentformat muss `ExtStore`-Frames sealen | Eigenbau; Tantivy höchstens als Übergang in P7 |
| WASM-Merge | Sandbox-Overhead, Determinismus (Fuel, keine Zeit/Zufall) | P9 |
| `ArcSwap`/`imbl` | Dependency vs. Eigenbau | Dependency, solange Audit akzeptiert |
| Learned Layout | gewinnt es genug gegenüber Deletion-Group-Layout? | erst nach Messung (P9), nur innerhalb Deletion Groups |
| Deployment | embedded vs. Server | Embedded-Kern + dünner Server-Wrapper ab P7 |

---

## Anhang A – Rückverfolgbarkeit

| Bereich | Hauptquelle | Ergänzt durch |
|---|---|---|
| Schlüsselhierarchie, Cohorts, Rewrap, Verbatim-Purge | A | B (Epochen, Stärkestufen) |
| Garantiestufen G0–G2, Scrub, DiskFull-Notbetrieb, Checkpoint-Semantik, Phasenplan, FaultFs-Byte-Zähler | B | A |
| Zustandsautomat, Holds (Authority), Receipts, Gap-Regel, Lock-Ordnung, TLA⁺-Modelle | A | B |
| Cells, Zoom, Token-Summen, Knapsack, Branches/Merge, Handles, Trust, Work Queue, MCP, Utility@Budget | C | A/B (Einbettung in EU-Modell) |
| Bitemporalität, Idempotenz, Capabilities, Retention | C | A/B |

## Anhang B – Korrekturen an den Quelldokumenten

Beim Zusammenführen wurden folgende Fehler bzw. Inkonsistenzen gefunden und behoben:

1. **Frame-Header (A):** Die Feldliste ergibt bereits 48 Byte, `hdr_crc` sollte aber in 44..48 liegen und „die obigen 44 Byte" schützen, sodass der Nonce überlappt. Neu: 64-Byte-Header mit lückenloser Feldtabelle (§5.2).
2. **AAD mit `version` (A):** Wird in `prepare` vor der Seq-Vergabe versiegelt, kann die AAD nicht die Commit-Seq enthalten. Neu: `record_id`/`local_ord` statt `Seq`.
3. **`eu_tag`-Kollisionen (A):** 64-Bit-Tags kollidieren bei ~10⁹ EUs mit nennenswerter Wahrscheinlichkeit; ein tagbasierter Purge würde Daten einer anderen EU löschen. Neu: eindeutige Vergabe mit Zähler (I21).
4. **`ScrubPending` und `LinkTable` im Katalog (B):** Verletzen die Opazität des Manifests (EU-IDs und EU-Beziehungsgraph im Klartext). Neu: Fortschritt im Ledger, Links sealed in `SYS_DERIV`.
5. **Directory-Gesamt-Hash (A) vs. In-place-Scrub (B):** Ein Hash über das Directory verträgt keine nachträgliche Markierung. Neu: Eintrags-CRC.
6. **`ClusterId` im Sektionsheader/Directory (B):** Klartext-Identität. Neu: nur `eu_tag`.
7. **Lift ohne Signatur (B) vs. Vier-Augen-Release (A):** B widerspricht dem Bedrohungsfall A3. Neu: A.
8. **Abgelehnter Löschauftrag bei Restriction (B):** Würde einen Löschauftrag verlieren. Neu: angenommen und `Blocked`.
9. **Hash-adressierte Per-Subject-Daten (C):** Dedup und Klartext-Hash kollidieren mit Löschbarkeit und Seitenkanälen. Neu: Derivation Cache pro EU, kein EU-übergreifendes Dedup.
10. **Access-Trace und Co-Retrieval-Graph (C):** Wären ein PII-Seitenkanal, der Löschung überlebt. Neu: EU-gesiegelt/RAM-only, beim Gate verworfen.
11. **Zoom-Down als „Crypto-Shredding des Originals" (C):** Innerhalb einer EU nicht möglich. Neu: Granularitätsgrenze (§4.4).
12. **Reader-Snapshots vs. Branch-Basen/Zeitreise:** Die Snapshot-Altersgrenze (A) vertrüge keine langlebigen Branches. Neu: `RetentionPin` getrennt von Reader-Leases.
13. **Cursor-Schlüssel pro Prozess (A):** Gilt nach Neustart als `CursorExpired`; jetzt explizit dokumentiert.
