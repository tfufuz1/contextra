# Lethe – Speicherkern für LLM-/Agenten-Workloads mit nachweisbarer Löschung

**Status:** Entwurf v1.0 · **Sprache:** Rust (Edition 2021+, `#![forbid(unsafe_code)]` im Kern) · **Arbeitsname:** `lethe`

> Leitsatz: *Löschen ist kein Nachgedanke, sondern die primäre Strukturierungsachse des Layouts.* Jede Datei, jeder Frame und jeder Schlüssel ist einer Löscheinheit (Erasure Unit, EU) zuordenbar, und diese Zuordnung ist ohne Zugriff auf den Klartext nutzbar.

---

## 0. Begriffe

| Begriff | Bedeutung |
|---|---|
| **Tenant** | Mandant. Kleinste Einheit für Isolation und Massenlöschung (Verzeichnis-Unlink + KEK-Vernichtung). |
| **EU (Erasure Unit)** | Kleinste *kryptografisch* löschbare Einheit (Nutzer, Konversation, Dokumentenset, Themencluster). Besitzt genau einen Master-DEK. |
| **Deletion Group** | Vom Aufrufer gelieferter Platzierungshinweis: EUs, die typischerweise gemeinsam gelöscht werden, landen in derselben Partition. |
| **Partition** | Physische Gruppe (Verzeichnis) mit eigener WAL-Anbindung, eigenem Memtable, eigenem LSM und eigenen Blob-Dateien. Gehört zu genau einem Tenant. |
| **Space** | Logischer Namensraum (z. B. `agent_state`, `chat_history`, `chunks`). Spaces sind *kein* physisches Gruppierungsmerkmal. |
| **Head / Body** | Kompakter Kopf (Key, Version, Gewichte, Attribute, Body-Referenz) vs. großer Wertkörper (Blob). |
| **eu_tag** | `HMAC-SHA256(tenant_tag_key, eu_id)[..8]`. Pseudonymer, 64 Bit langer Routing-Tag im Klartext-Frame-Header. Er erlaubt Purge ohne Entschlüsselung. |
| **Cohort** | Gruppe von ≤ 1024 EU-Schlüsseln desselben Tenants mit gemeinsamer Wrap-KEK-Generation. |
| **Ledger** | Separates, hash-verkettetes Lösch- und Governance-Journal in eigener Failure Domain. |
| **Anchor** | Externer monotoner Rollback-Anker (TPM-Counter, Remote-Log, bedingte Writes). |

---

## 1. Architektur-Leitentscheidungen, Nicht-Ziele, Bedrohungsmodell

### 1.1 Leitentscheidungen

| # | Entscheidung | Begründung |
|---|---|---|
| D1 | **Sektionsweise EU-Gruppierung in jeder Datei** (Run = Folge von EU-Sektionen, Blob-Datei = Folge von Chunk-Frames mit `eu_tag`). | Purge einer EU entspricht dem Auslassen von Byte-Bereichen (verbatim-Copy der Überlebenden, **kein** Decrypt/Re-Encrypt). |
| D2 | **Positionsunabhängige Ciphertexts**: AAD bindet `(eu_id, record_id, version, kind)`, niemals Dateioffset oder Dateiname. | Verbatim-Umkopieren ist kryptografisch gültig. |
| D3 | **Alles Personenbezogene ist versiegelt**: Keys, Attribute, Filter, Gewichte, Abhängigkeiten, Sektionsindizes, Bloom-Filter. Im Klartext bleiben nur `eu_tag`, Längen, Frame-Typ, CRC. | Schlüsselvernichtung macht *auch Metadaten* unlesbar. |
| D4 | **Schlüsselhierarchie** `KMS-Root → Cohort-KEK (Generation g) → EU-Master-DEK → HKDF-Subkeys (head, body, wal, idx, ext:<name>)`. | EU-Löschung = Rewrap der Cohort + KMS-Destroy der alten KEK-Generation. Kein Re-Encrypt von Daten. |
| D5 | **Ordnung nur innerhalb einer EU**: Scans sind `(EU, Space, Key-Range)`. Es gibt keinen globalen Key-Order-Scan über EUs. | Nötig für D1/D3. Cross-EU-Abfragen laufen über Extensions (sektionsweise versiegelte Sekundärindizes). |
| D6 | **Ein globaler Commit-Log (Group Commit), Partition-lokale LSM/Blob-Strukturen.** | Maximale `fsync`-Bündelung und atomare Mehrbereichs-Transaktionen. Löschlokalität bleibt auf Datei-Ebene. |
| D7 | **Erasure-Barrier überstimmt Snapshot-Isolation**: Nach dem logischen Lösch-Commit `E` lehnt jeder Read auf die EU ab, auch auf Snapshots mit `seq < E`. | Deterministisches Abweisen veralteter Views. |
| D8 | **Ledger + Anchor außerhalb des Datenvolumes**; Lösch-Tatsachen sind **nicht** aus dem Datenbackup restaurierbar. | Rollback-/Resurrection-Schutz. |
| D9 | **Mechanismus statt Politik**: Fristen, Urgency, Retention, Placement-Hints sind Parameter. Der Kern kennt keine Tokenizer, Embeddings oder ANN. | Klare Schichttrennung, Erweiterungsverträge in §2.5. |
| D10 | **Ports für alles Nichtdeterministische**: FS, Krypto, KMS, Zeit, Zufall, Anchor, Executor. | Deterministische Simulation (§6.3). |

### 1.2 Nicht-Ziele

- Kein SQL, keine Secondary-Index-Query-Engine, keine Volltextsuche, kein ANN, kein Tokenizer im Kern.
- Keine verteilte Replikation/Konsens (Replikation ist eine Schicht darüber; der Kern definiert nur `ReplicationHook` für Commit-Stream-Export und erfordert, dass Replikate den Erasure-Vertrag erfüllen).
- **Keine** Garantie für Löschung aus: Prozessspeicher bis zum Zeroize/Prozessende, Swap, Core-Dumps, Kernel-Page-Cache-Remanenzen auf Hardware-Ebene (SSD-Wear-Leveling; hier schützt nur die Kryptografie), Daten, die die Anwendung bereits exportiert hat (LLM-Provider-Logs, Prompt-Caches, Client-Caches).
- Kein Schutz gegen einen Angreifer, der *vor* der Löschung DEK-Klartext aus dem Prozessspeicher entwendet hat.
- Keine Mehrfach-Writer-Prozesse auf demselben Verzeichnis (exklusiver `LOCK`).

### 1.3 Bedrohungsmodell

**Vertrauensanker:** KMS/HSM (Schlüsselvernichtung ist dort irreversibel), Anchor (monoton), Prozess zur Laufzeit (vertrauenswürdig, aber crash-anfällig).

| ID | Angreifer / Fehler | Fähigkeit | Gegenmaßnahme |
|---|---|---|---|
| A1 | **Offline-Dieb** von Platten/Backups/Snapshots | Liest alle Dateien, auch alte Generationen | Alles versiegelt (D3). Nach Cohort-Rewrap + KEK-Destroy sind alte Generationen nutzlos. |
| A2 | **Restore-/Rollback-Angreifer** (oder Betriebsfehler) | Spielt älteres Datenbackup/FS-Snapshot ein, um Gelöschtes wiederzubeleben | Ledger + Anchor (§5.7). Reconcile vor erstem Read. Schlüssel sind ohnehin zerstört. |
| A3 | **Lokaler Betreiber mit Löschwunsch gegen Hold** | Versucht, Sperrvermerk zu umgehen | Hold-Prüfung in Coordinator, Keyring-Writer **und** Purge. Release nur mit signiertem `HoldRelease` fremder Authority. |
| A4 | **Böswilliger/fehlerhafter RAG-Layer** | Persistiert Abgeleitetes außerhalb der Segment-Verträge | Nur versiegelte `ExtStore`-API. `Opaque`-Segmente nur als `NoPersonalData`/`Rebuildable`. Canary-Oracle (§6.4) als Konformitätstest. |
| F1 | **Torn Write / Power Loss** | Teilweise persistierte Seiten, umsortierte Writes | Record-Checksummen, Hash-Kette, Gap-Regel (§4.9). |
| F2 | **`fsync` meldet EIO** | Page-Cache-Zustand undefiniert | **Kein Retry.** Engine → `Poisoned`, Recovery entscheidet (§4.10). |
| F3 | **ENOSPC** | Allokation scheitert | Vorallokierte WAL-Segmente, Lösch-Reserve, `ReadOnlyDegraded`. Löschpfad bleibt arbeitsfähig. |
| F4 | **Stille Korruption (Bit-Rot)** | Flip in Daten oder Metadaten | CRC32C auf Ciphertext (schlüssellos prüfbar) + AEAD-Tag + Scrubber. Quarantäne pro Sektion. |
| F5 | **Schlüsselverlust** | KMS unerreichbar oder Eintrag fehlt | Unterscheidung `KeyUnavailable` / `KeyDestroyed` / `KeyLost` (§4.10). |
| F6 | **Crash mitten im Löschprotokoll** | Jeder Schritt unterbrechbar | Ledger-Intents + idempotente Schritte + Roll-forward (§5). |
| F7 | **Replay-/Retry-Stürme von Agenten** | Doppelte Commits | Idempotenz-Tabelle pro EU (§4.1). |

---

## 2. Crate-/Modulschnitt & Port-Schnittstellen

### 2.1 Crate-Graph

```
lethe-types      Ids, Fehler, Konstanten, Weights                      (no_std + alloc möglich)
lethe-ports      Traits: Fs, Crypto, KeyService, Clock, Entropy, Anchor, Executor, Authority, Signer
lethe-format     Reine Codecs: FrameHeader, Run, Blob, WAL, Manifest, Ledger (fuzz-/Kani-fähig)
lethe-keys       Keyring, Cohorts, DEK-Handles, HKDF-Ableitung, Zeroize
lethe-wal        Gruppenbündelung, Hash-Kette, Recovery-Scanner
lethe-lsm        Memtable (persistente Maps), Runs, Section-Index, Merge-Iteratoren, Budget-Paging
lethe-blob       Chunking, Kompression, Blob-Dateien, Lineage-Ids
lethe-catalog    Manifest, Spaces, Partitionen, Placement, Snapshot-Registry
lethe-erasure    Zustandsautomat, Ledger, Holds, Receipts, Purge-Jobs, Verifier
lethe-ext        Extension-Verträge (SegmentOwner, ExtStore, DerivationRegistry)
lethe-core       Fassade `Engine`, `Txn`, `Snapshot`, Scheduler, Ressourcenbudgets
─────────── außerhalb des unsafe-freien Kerns ───────────
lethe-fs-std     Fs-Adapter (rustix: fallocate, O_DIRECT, fsync, dir-fsync; hier ist `unsafe` erlaubt)
lethe-crypto-rc  RustCrypto-Adapter (XChaCha20-Poly1305, HKDF-SHA256, HMAC, BLAKE3, Ed25519)
lethe-kms-*      KMS-Adapter (Cloud-KMS, Vault, lokaler HSM, `LocalKeyService` nur für Dev/Test)
lethe-sim        SimFs, SimClock, SimEntropy, SimKms, SimAnchor, SimExecutor, Fault-Injektoren
lethe-oracle     Lösch-Orakel, Canary-Scanner
```

Regeln: `lethe-*` im Kern haben `#![forbid(unsafe_code)]`. Abhängigkeiten mit internem `unsafe` (z. B. `arc-swap`, `zeroize`, `imbl`) werden per `cargo-vet`/`cargo-geiger` auditiert und gepinnt. Kein Kern-Crate importiert `std::fs`, `std::time`, `rand` oder Krypto-Crates direkt (CI-Lint per `clippy.toml` `disallowed-methods`).

### 2.2 Identitäten

```rust
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)] pub struct TenantId(pub u128);
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)] pub struct EuId(pub u128);   // PII-nah: nie loggen
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)] pub struct EuTag(pub u64);   // pseudonym
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)] pub struct SpaceId(pub u32);
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)] pub struct PartitionId(pub u32);
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)] pub struct Seq(pub u64);      // Commit-Sequenz
#[derive(Copy, Clone, Eq, PartialEq, Hash)]                  pub struct ReqId(pub u128);   // Lösch-/Idempotenz-Request

/// Mehrdimensionale Gewichte, vom Aufrufer geliefert (Kern kennt keinen Tokenizer).
#[derive(Copy, Clone, Default)]
pub struct Weights { pub tokens: u32, pub bytes: u32, pub extra: [u32; 2] }
```

`Debug`/`Display` für `EuId`, Keys und Payloads sind redigiert (`Redacted<T>`). Das verhindert PII in Logs bereits per Typ.

### 2.3 Ports

```rust
pub trait Fs: Send + Sync + 'static {
    type File: FsFile;
    fn open(&self, p: &Path, o: OpenOpts) -> IoResult<Self::File>;      // create/append/read/direct
    fn create_tmp(&self, dir: &Path) -> IoResult<(PathBuf, Self::File)>;
    fn rename_atomic(&self, from: &Path, to: &Path) -> IoResult<()>;
    fn unlink(&self, p: &Path) -> IoResult<()>;
    fn sync_dir(&self, dir: &Path) -> IoResult<()>;
    fn list(&self, dir: &Path) -> IoResult<Vec<DirEntry>>;
    fn free_space(&self, dir: &Path) -> IoResult<u64>;
    fn lock_exclusive(&self, p: &Path) -> IoResult<LockGuard>;
}
pub trait FsFile: Send + Sync {
    fn read_at(&self, off: u64, buf: &mut [u8]) -> IoResult<usize>;
    fn write_at(&self, off: u64, buf: &[u8]) -> IoResult<()>;
    fn preallocate(&self, len: u64) -> IoResult<()>;                    // fallocate (kein sparse)
    fn sync_data(&self) -> IoResult<()>;                                // Fehler ⇒ Poison, nie Retry
    fn len(&self) -> IoResult<u64>;
    fn overwrite_zero(&self, off: u64, len: u64) -> IoResult<()>;       // Best-Effort-Scrub vor unlink
}
#[non_exhaustive] pub enum IoErrorKind { NoSpace, Io, NotFound, Exists, Permission, Interrupted, Other }

pub trait Clock: Send + Sync { fn wall(&self) -> UnixMillis; fn mono_nanos(&self) -> u64; }
pub trait Entropy: Send + Sync { fn fill(&self, buf: &mut [u8]); }

/// Reine Krypto-Primitive, zustandslos.
pub trait Crypto: Send + Sync {
    fn aead_seal(&self, key: &Key256, nonce: &[u8; 24], aad: &[u8], pt: &[u8], out: &mut Vec<u8>);
    fn aead_open(&self, key: &Key256, nonce: &[u8; 24], aad: &[u8], ct: &[u8], out: &mut Vec<u8>) -> Result<(), AuthFailed>;
    fn hkdf(&self, ikm: &Key256, label: &[u8], ctx: &[u8]) -> Key256;
    fn hmac(&self, key: &Key256, msg: &[u8]) -> [u8; 32];
    fn hash(&self, msg: &[u8]) -> [u8; 32];                              // BLAKE3
}

/// Externe Schlüsselautorität. `destroy` ist irreversibel oder meldet ehrlich `PendingUntil`.
pub trait KeyService: Send + Sync {
    fn create_kek(&self, label: KekLabel) -> KsResult<(KekRef, Zeroizing<Key256>)>;
    fn fetch_kek(&self, r: &KekRef) -> KsResult<Zeroizing<Key256>>;
    fn destroy_kek(&self, r: &KekRef) -> KsResult<DestroyAttestation>;
    fn kek_status(&self, r: &KekRef) -> KsResult<KekStatus>;             // Live | Destroyed{at} | PendingDestroy{until} | Unknown
}
pub struct DestroyAttestation { pub kek: KekRef, pub irreversible: IrreversibleAt, pub evidence: Vec<u8> }
pub enum IrreversibleAt { Now, At(UnixMillis) }                          // At(..) ⇒ Receipt trägt Restrisiko

/// Monotoner, externer Rollback-Anker (CAS-Semantik, kein Zurücksetzen).
pub trait RollbackAnchor: Send + Sync {
    fn read(&self) -> AnchorResult<AnchorState>;
    fn advance(&self, expect: AnchorState, next: AnchorState) -> AnchorResult<AnchorAttestation>;
}
pub struct AnchorState { pub ledger_seq: u64, pub ledger_head: [u8; 32], pub erasure_watermark: u64 }

pub trait Signer: Send + Sync { fn sign(&self, msg: &[u8]) -> Vec<u8>; fn key_id(&self) -> KeyId; }
pub trait Authority: Send + Sync { fn verify(&self, op: &HoldOp, sig: &[u8]) -> bool; }   // getrennte Principal

pub trait Executor: Send + Sync {                                        // Threads/Tasks + Prioritäts-Lanes
    fn spawn(&self, lane: Lane, name: &'static str, job: Box<dyn FnOnce() + Send>) -> JobHandle;
    fn yield_now(&self);
}
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Lane { Urgent = 0, Commit = 1, Read = 2, Flush = 3, Compact = 4, Scrub = 5 }
```

### 2.4 Öffentliche Kern-API (Auszug)

```rust
pub struct Engine<P: Ports> { /* … */ }

impl<P: Ports> Engine<P> {
    pub fn open(ports: P, cfg: Config) -> Result<Self, OpenError>;      // führt deterministische Recovery aus (§4.8)

    // Tenants, EUs, Spaces
    pub fn create_tenant(&self, t: TenantId) -> Result<()>;
    pub fn create_space(&self, t: TenantId, s: SpaceDesc) -> Result<SpaceId>;
    pub fn create_eu(&self, t: TenantId, eu: EuId, place: PlacementHint) -> Result<()>;

    // Transaktionen
    pub fn begin(&self, o: TxnOpts) -> Txn<'_, P>;
    pub fn snapshot(&self) -> Snapshot<P>;

    // Löschen und Governance
    pub fn erase(&self, req: ErasureRequest) -> Result<ErasureHandle>;  // idempotent über req.id
    pub fn erasure_status(&self, id: ReqId) -> Result<ErasureStatus>;
    pub fn receipt(&self, id: ReqId) -> Result<Option<ErasureReceipt>>;
    pub fn place_hold(&self, h: Hold, sig: &[u8]) -> Result<HoldId>;
    pub fn release_hold(&self, id: HoldId, sig: &[u8]) -> Result<()>;
    pub fn begin_backup(&self) -> Result<BackupGuard>;                  // pinnt Manifest, bindet Anchor/Ledger-Stand

    // Erweiterungen
    pub fn register_extension(&self, ext: Arc<dyn SegmentOwner>) -> Result<ExtStoreHandle>;
}

pub struct TxnOpts { pub durability: Durability, pub idem: Option<IdemKey>, pub lane: Lane, pub deadline: Option<Duration> }
pub enum Durability { Sync, Async /* Ack vor fsync; dokumentiert verlustbehaftet */ }

impl<'e, P: Ports> Txn<'e, P> {
    pub fn put(&mut self, eu: EuId, sp: SpaceId, key: &[u8], v: PutValue) -> Result<()>;
    pub fn delete(&mut self, eu: EuId, sp: SpaceId, key: &[u8]) -> Result<()>;
    pub fn derive(&mut self, out: EuId, sources: &[EuId], policy: OnSourceErasure) -> Result<()>;
    pub fn commit(self) -> CommitFuture;                                // -> CommitReceipt { seq, durable }
}

pub struct PutValue<'a> { pub body: Body<'a>, pub weights: Weights, pub attrs: &'a [u8] /* ≤ 256 B */ }
pub enum Body<'a> { Inline(&'a [u8]), Stream(&'a mut dyn Read) }

impl<P: Ports> Snapshot<P> {
    pub fn get_head(&self, eu: EuId, sp: SpaceId, key: &[u8]) -> Result<Option<Head>>;
    pub fn get_body(&self, h: &Head, range: Option<Range<u64>>) -> Result<Bytes>;
    pub fn scan_page(&self, q: ScanQuery, budget: Budget, cur: Option<Cursor>) -> Result<Page>;
}
pub struct Budget { pub tokens: Option<u32>, pub bytes: Option<u32>, pub max_items: u32 }
pub struct Page { pub heads: Vec<Head>, pub next: Option<Cursor>, pub used: Weights, pub oversize_single: bool }
```

Spezialfälle: `SpaceKind::Ephemeral` (siehe §4.1, Scratchpad); `Erased`, `Restricted`, `CursorExpired`, `SnapshotTooOld` sind **eigene Fehlerklassen**, nie `NotFound`. So bleibt „gelöscht" von „existiert nicht" unterscheidbar, und das verhindert Existenz-Orakel nur dort, wo die Anwendung es will (Policy-Flag `error_masking`).

### 2.5 Erweiterungsverträge für RAG-Komponenten

```rust
/// Jede aufsetzende Komponente (Vektor-, Volltext-, Graph-Index, Summary-Cache) implementiert dies.
pub trait SegmentOwner: Send + Sync {
    fn ext_id(&self) -> ExtId;

    /// Nach dem logischen Lösch-Commit. `notice.tags` sind eu_tags der Löschmenge (inkl. abgeleiteter EUs).
    /// Die Extension MUSS: (a) Treffer dieser Tags sofort aus Antworten filtern,
    /// (b) in-memory Strukturen bereinigen, (c) persistente Segmente neu schreiben oder verwerfen,
    /// (d) mit `ExtAck` bestätigen. Fristüberschreitung ⇒ Receipt `PartiallyComplete`.
    fn on_erasure(&self, notice: &ErasureNotice) -> ExtProgress;

    /// Inventar für den Verifier: welche Segmente existieren, welche Klasse, welche Owner-Tags.
    fn segments(&self) -> Vec<SegmentInfo>;
}

pub enum SegmentClass {
    EuPure,                        // alle Frames gehören genau einer EU ⇒ Löschung = unlink
    Mixed,                         // Frames tragen eu_tag im Header (ExtStore-Format) ⇒ Engine kann verbatim purgen
    Rebuildable { max_age: Duration }, // abgeleitete Struktur (z. B. HNSW-Graph); wird nach Löschung neu gebaut
    NoPersonalData,                // deklariert; vom Canary-Oracle stichprobenartig geprüft
}

/// Einzig erlaubter Persistenzweg für Extensions. Frames werden mit k_ext(ext_id) der jeweiligen EU versiegelt.
pub trait ExtStore {
    fn open_segment(&self, class: SegmentClass) -> Result<ExtSegmentWriter>;
}
impl ExtSegmentWriter {
    pub fn append(&mut self, eu: EuId, payload: &[u8]) -> Result<FrameRef>; // Frame trägt eu_tag
    pub fn seal(self) -> Result<SegmentId>;                                  // Manifest-Edit, atomar
}

/// Abhängigkeitsverknüpfung: Quelle → abgeleitetes Artefakt (Embedding, Summary, Cache-Eintrag).
pub enum OnSourceErasure { Delete, Invalidate /* sichtbar als stale, Rebuild durch App */ }
```

**Verbindlichkeit:** Extension-Ergebnisse laufen vor Rückgabe durch `Snapshot::filter_visible(&[RecordRef])` (Prüfung gegen den `EuStateTable`). Damit ist die Sichtbarkeit unabhängig davon, wie schnell die Extension aufräumt.

---

## 3. Datenmodell, physisches Layout, Kern-Invarianten

### 3.1 Verzeichnisstruktur

```
<root>/
  LOCK
  CURRENT                          -> MANIFEST-<gen>
  MANIFEST-<gen>                   Edit-Log (Spaces, Partitionen, Dateilisten, flushed_seq, Cohort-Liste); EU-frei
  wal/wal-<n>.log                  vorallokiert, 64 MiB, versiegelte Records
  keys/<tenant>/cohort-<id>-g<gen>.keys     Keyring (= EU-Verzeichnis + gewrappte DEKs)
  keys/<tenant>/tenant-g<gen>.keys          tenant_tag_key (gewrappt)
  parts/<tenant>/<partition>/run-<id>.lsr   Runs
  parts/<tenant>/<partition>/blob-<lineage>-g<gen>.lbs
  ext/<ext_id>/seg-<id>.lxs
  reserve/                         Lösch-Reserve (vorallokierte Dateien, §4.10)
  quarantine/                      defekte oder verwaiste Dateien
  tmp/
<separates Volume/Failure Domain>
  ledger/ledger-<n>.log            Hash-Kette, append-only
```

### 3.2 Frame-Format (universell: WAL, Run-Sektionen, Blob-Chunks, Ext-Segmente)

Little Endian. Der Klartext-Header ist minimal und enthält keine Nutzdaten.

```text
FrameHeader (48 Byte, Klartext)
  0  magic      u32   'LFR1'
  4  kind       u8    WalRec|HeadBlock|SectionIndex|BlobChunk|ExtFrame|Control
  5  flags      u8    compressed? padded?
  6  ver        u16
  8  eu_tag     u64   0 für EU-lose Frames (Control)
 16  body_len   u32   Länge des Ciphertext inkl. 16-Byte-Tag
 20  body_crc   u32   CRC32C über Ciphertext (schlüssellos prüfbar!)
 24  nonce      [24]  XChaCha20, zufällig (Entropy-Port)
 ─── hdr_crc   u32   CRC32C über die obigen 44 Byte (in 44..48)
Body: AEAD_Seal(k_<kind>, nonce, aad, plaintext)
 aad = DS("lethe/aad/v1") ‖ kind ‖ eu_id ‖ record_id ‖ version     // NICHT Offset/Dateiname (D2)
```

Folge: Ein Scrubber oder Purge-Job prüft Integrität und Zugehörigkeit **ohne** DEK. Ein Frame kann byteidentisch in eine andere Datei kopiert werden.

### 3.3 Run (SSTable-Äquivalent, EU-Sektionen)

```text
Run := RunHeader(4 KiB) · Section* · SectionDirectory · Footer(64 B)

RunHeader   : magic, ver, run_id, partition_id, level, min_seq, max_seq, section_count, features, hdr_crc
Section(eu) : HeadBlock-Frame* · SectionIndex-Frame          (alles unter k_head/k_idx dieser EU)
SectionDir  : [{eu_tag, off, len, idx_off, min_seq, max_seq, live_entries, crc}] sortiert nach eu_tag
Footer      : dir_off, dir_len, dir_blake3, magic
```

**HeadBlock** (≤ 16 KiB Klartext, optional LZ4, dann versiegelt) enthält sortierte Einträge:

```rust
struct HeadEntry {
    space: u32, key: Vec<u8>,        // key ≤ 1 KiB, Prefix-komprimiert im Block
    seq: u64, op: Put | Del,
    weights: Weights, raw_len: u64,
    attrs: Vec<u8>,                  // ≤ 256 B, Filterattribute des Aufrufers
    body: Inline(Vec<u8>)            // ≤ inline_max (Default 1 KiB): Agentenzustände, kleine Tool-Outputs
        | BlobRef { lineage: u64, first_chunk: u64, chunk_count: u32, raw_len: u64, codec: u8 },
    deps: Vec<DerivRef>,
}
```

**SectionIndex** (versiegelt): Block-Fence-Keys, Offsets, **kumulative Gewichtssummen je Block** (`cum_tokens`, `cum_bytes`, `count`), Bloom-Filter über `(space,key)`, Statistiken. Wird beim ersten Zugriff einer EU in den Section-Index-Cache geladen und enthält die einzigen Key-Fences. Nichts davon liegt im Klartext.

### 3.4 Blob-Dateien (Wertkörper)

```text
BlobFile := BlobHeader · ChunkFrame* · ChunkDirectory · Footer
ChunkFrame     : FrameHeader(kind=BlobChunk, eu_tag) · Seal(k_body, zstd(≤64 KiB Rohdaten))
ChunkDirectory : [{chunk_id u64, off u64, len u32, eu_tag u64}] sortiert nach chunk_id  (Klartext, pseudonym)
```

- Köpfe referenzieren `(lineage, chunk_id)`, **nicht** Dateioffsets. Der Katalog bildet `lineage → aktuelle Datei/Generation` ab. Ein Purge-Rewrite behält Chunk-Ids und ändert nur die Generation, **ohne dass versiegelte Köpfe angefasst werden müssen**.
- Bereichsleser holen nur die benötigten Chunks (`range`-Reads) und dekomprimieren nur diese.
- Komprimieren vor Verschlüsseln leakt Längeninformation. Optional `pad_to: 512 B` pro Space.
- Aktive Blob-Datei hat noch kein Directory; Recovery rekonstruiert es durch Frame-Scan (Header-CRC).

### 3.5 WAL

```text
WalSegment := Header · Batch*
Batch  := BatchHeader · Record* · BatchCommit
BatchHeader (Klartext): magic, batch_id, first_seq, record_count, prev_chain[32], hdr_crc
Record  := FrameHeader(kind=WalRec|Control) · … : txn_id, commit_seq, [OpPart(eu_tag, Seal(k_wal(eu), ops))]+
BatchCommit: chain = BLAKE3(prev_chain ‖ BatchHeader ‖ alle Record-Bytes), crc
```

- Jede Transaktion ist **ein** Record (Atomarität per Record-Integrität). Sie kann OpParts mehrerer EUs/Spaces/Partitionen enthalten und damit atomare Mehrbereichs-Commits liefern.
- Jeder OpPart ist unter dem WAL-Subkey **seiner** EU versiegelt. Nach Schlüsselvernichtung sind Reste im WAL unlesbar. Zusätzlich erzwingt die Löschung ein Rolling + Truncate (§5).
- Batches werden 4-KiB-ausgerichtet in vorallokierte Segmente geschrieben.
- `Control`-Records (Erasure-Transitionen, Hold-Events, Cohort-Wechsel) tragen nur `eu_tag`s, nie `EuId`, damit Recovery sie ohne Schlüssel anwenden kann.

### 3.5.1 Keyring (Cohort-Datei) = EU-Verzeichnis

```text
CohortFile(gen g) := Header{tenant, cohort_id, gen, kek_ref, entry_count} · Entry* · Footer{blake3}
Entry := Seal(KEK_g, { eu_id, eu_tag, partition, state, held: bool, created_seq, dek_wrapped(32+16) })
```

Der Keyring ist die **einzige** dauerhafte Quelle für `EuId → Partition/State/DEK`. Wer ihn verliert, verliert alle Daten dieser Cohort (siehe F5, Replikationspflicht in §5.7). Wer ihn rewrappt und die alte KEK zerstört, macht jede alte Kopie unbrauchbar.

### 3.6 Ledger (Lösch-/Governance-Journal)

```rust
struct LedgerEntry {
    seq: u64, prev: [u8; 32], ts: UnixMillis,
    kind: LedgerKind,       // ErasureRequested{req, scope_tags, urgency} | StageIntent{req, stage, args_digest}
                            // | StageDone{req, stage, evidence_digest} | KeyRewrite{cohort, old_gen, new_gen}
                            // | KekDestroyed{kek, attestation_digest} | HoldPlaced/Released{hold, scope, sig}
                            // | ExtAck{req, ext, digest} | PurgeVerified{req, residual_digest}
                            // | ReceiptIssued{req, receipt_digest} | RestoreReconciled{...}
    mac: [u8; 32],          // HMAC(ledger_key, …); Kette: entry_hash = BLAKE3(prev ‖ body ‖ mac)
}
```

Das Ledger enthält ausschließlich `eu_tag`s und Digests, nie Nutzdaten und nie `EuId`. Es hat eine eigene Retention (mindestens bis zum Ablauf der ältesten Backup-Generation, die betroffene Daten enthalten könnte).

### 3.7 Platzierung (löschlokales Layout)

- `PlacementHint { deletion_group: Option<u128>, expected: Small|Medium|Large }`.
- Gruppen werden per Hash auf ≤ `P` Partitionen pro Tenant abgebildet. Kleine Gruppen teilen sich Partitionen (billig für viele Nutzer), große Gruppen erhalten dedizierte Partitionen. Überschreitet eine Gruppe `promote_bytes`, erfolgt ein **Partition-Split per verbatim-Sektionskopie** (kein Decrypt).
- **Kostenmodell der Löschung:**

| Szenario | Kryptografisch | Physisch |
|---|---|---|
| Ganzer Tenant | O(Cohorts) KEK-Destroys | `unlink` der Tenant-Verzeichnisse, O(#Dateien), kein Rewrite |
| Dedizierte Gruppe | 1 Cohort-Rewrap | `unlink` der Partition |
| EU in geteilter Partition | 1 Cohort-Rewrap | Purge-Rewrite *dieser* Partition (verbatim, O(Partitionsgröße)), nie global |

### 3.8 Kern-Invarianten

- **I1 (Tag-Vollständigkeit):** Jeder Byte-Bereich auf Disk, der Nutzdaten einer EU enthält, liegt in einem Frame mit korrektem `eu_tag` oder in einem Sektionsabschnitt, den das SectionDirectory demselben Tag zuordnet.
- **I2 (Schlüsselexklusivität):** Nutzdaten einer EU sind nur mit deren DEK lesbar. Es gibt keinen zweiten Entschlüsselungsweg.
- **I3 (Positionsunabhängigkeit):** Kein AAD, keine Referenz in versiegelten Daten enthält Dateioffsets.
- **I4 (Manifest-Frei von EUs):** Das Manifest nennt weder `EuId` noch `eu_tag` (nur Dateien, Sequenzbereiche, Cohort-Ids).
- **I5 (Seq-Monotonie):** `commit_seq` ist lückenlos und streng monoton. `visible_seq ≤ durable_seq`.
- **I6 (Erasure-Barrier):** Nach durablem `EraseLogical(E)` liefert kein Read für die betroffene EU Daten, unabhängig vom Snapshot.
- **I7 (Hold-Sicherheit):** Eine EU mit aktivem Hold verliert nie ihren DEK und keine ihrer Frames.
- **I8 (No-Resurrection):** Ein Tag im Ledger-`Erased`-Set wird nach Restore/Recovery nie wieder `Live`.
- **I9 (Ledger-Vorrang):** Die KEK-Vernichtung folgt immer nach durablem `StageIntent` und ist durch `StageDone` + Anchor-Advance belegt.

---

## 4. Exakte Ablaufprotokolle

### 4.1 Schreibpfad (Group Commit)

**Akteure:** beliebig viele Client-Threads → *Prepare*; ein Sequencer-Thread (Owner von WAL-Writer, kein Lock) → *Commit*; ein Apply-Schritt veröffentlicht atomar.

1. **Begin:** Snapshot `read_seq` erfassen (für Konflikterkennung).
2. **Prepare (parallel, Lane `Commit`):**
   1. `EuStateTable.load()` → EU muss `Live` sein (sonst `Erased`/`Restricted`/`Erasing`). `KeyHandle` aus Key-Cache holen (`Arc`, `revoked: AtomicBool`).
   2. Große Bodies: in 64-KiB-Chunks zerlegen → zstd → versiegeln (`k_body`) → an aktive Blob-Datei der Partition anhängen (**ungesynct**) → `BlobRef`. Kleine Bodies bleiben inline.
   3. Heads bauen, OpParts mit `k_wal` versiegeln. `Weights` und `attrs` stammen vom Aufrufer.
   4. Idempotenzprüfung vorbereiten: `IdemKey` wird in die EU-Sektion des Systemspace `SYS_IDEM` geschrieben (versiegelt unter der EU). Die Prüfung selbst erfolgt im Sequencer, damit Dubletten in-flight koalesziert werden.
3. **Enqueue** in die Prioritätswarteschlange (`Urgent` > `Commit` > `Background`). Erasure-Control-Records nutzen `Urgent`.
4. **Sequencer-Loop:**
   1. Batch sammeln bis `max_batch_bytes` (Default 4 MiB) **oder** `max_delay` (adaptiv, 50–500 µs, abhängig von Queue-Tiefe und `fsync`-Latenz-EWMA).
   2. Pro Record serialisiert prüfen: EU-Zustand (autoritativ), Hold/Restriction, Idempotenz (gleicher Key + gleicher Fingerprint ⇒ gespeicherten `CommitReceipt` zurückgeben, gleicher Key + anderer Fingerprint ⇒ `IdemConflict`), Write-Write-Konflikt via `last_commit_seq(key) > read_seq` (First-Committer-Wins).
   3. `commit_seq` lückenlos vergeben, Records finalisieren, `chain` berechnen.
   4. **Blob-Sync:** alle in diesem Batch referenzierten, noch ungesyncten Blob-Dateien `sync_data` (dedupliziert).
   5. Batch an WAL-Tail schreiben (aligned), `sync_data`. Dies ist der **einzige** Durability-Punkt.
   6. **Apply:** Memtables der betroffenen Partitionen per Copy-on-Write erweitern (`imbl::OrdMap`, Pfad-Kopie) und `ArcSwap<Version>` + `visible_seq` (Release) veröffentlichen.
   7. Futures vervollständigen: `CommitReceipt { seq, durable: true }`.
5. **Ordnungsinvariante:** `Blob-fsync → WAL-fsync → Apply → Ack`. Ein im WAL referenzierter Chunk ist immer durable. Ein Chunk ohne WAL-Referenz ist Waise und wird vom Blob-GC eingesammelt.
6. **`Durability::Async`:** Ack nach Schritt 4.3 (vor `fsync`). Dokumentiert verlustbehaftet, nie für Erasure-Control-Records.
7. **Ephemere Spaces (Scratchpad):** Writes auf `Ephemeral` gehen **nie** durch WAL/Blob/`fsync`. Sie landen in speicherresidenten Memtables mit Prozess-Zufallsschlüssel (Zeroize bei Drop). Bei Budgetüberlauf Spill in eine anonyme, sofort ge-`unlink`te Datei mit demselben flüchtigen Schlüssel. Gemischte Transaktionen sind erlaubt: Sichtbarkeit atomar zur Laufzeit, nach einem Crash existiert nur der durable Teil (`CommitReceipt.ephemeral_part: Volatile`). Erasure auf ephemere EUs = sofortiges Verwerfen + Zeroize.
8. **Backpressure:** Der Sequencer blockiert nicht, aber `enqueue` liefert `Busy{retry_after}` bzw. wartet bis `deadline`, wenn (a) immutable Memtables > N, (b) L0-Runs der Partition > M, (c) WAL-Bytes > Cap, (d) Memory-Budget erschöpft. `Urgent` umgeht (a)–(c) ausdrücklich.

### 4.2 Lesepfad (inkl. Budget-Paging)

**Point-Get** `get_head(eu, space, key)`:

1. `EuStateTable.load()` (`ArcSwap`, wartefrei). Nur `Live` darf passieren (I6). `erasure_epoch` merken.
2. Memtable (aktiv + immutable) via Version-`Arc`, dann Runs `newest → oldest` der Partition.
3. Je Run: `SectionDirectory` binär nach `eu_tag` suchen (Klartext). Section-Index aus Cache oder laden, entschlüsseln. Bloom-Prüfung. Fence-Binärsuche. HeadBlock lesen, CRC prüfen, entschlüsseln, Key suchen.
4. Erstes Ergebnis mit `seq ≤ snapshot.seq` gewinnt. `Del` ⇒ `None`.
5. **Check-after-decrypt:** Nach der Entschlüsselung `KeyHandle.revoked` und `erasure_epoch` erneut prüfen. Hat sich beides geändert, wird das Ergebnis verworfen und `Erased` geliefert (linearisiert die Lese-vs-Lösch-Race, siehe §6.2 R1).
6. `get_body` erst auf Anforderung: Chunk-Range lesen, entschlüsseln, dekomprimieren.

**Budget-Paging** `scan_page(q, budget, cursor)`:

```text
Eingabe: EU, Space, [lo,hi), Budget{tokens?, bytes?, max_items}, Cursor?
1. Snapshot aus Cursor (sealed) oder aktuell; Cursor-Validierung (§unten).
2. Merge-Iterator über Memtable + Runs, nur Head-Einträge (Bodies werden NIE geladen/dekomprimiert).
3. used = 0; heads = []
   für jeden sichtbaren Head h (seq ≤ snap, kein Tombstone, neueste Version je Key):
       falls (used + h.weights) überschreitet Budget in irgendeiner Dimension:
            wenn heads leer → heads=[h]; oversize_single=true     // Fortschrittsgarantie
            sonst stop
       used += h.weights; heads.push(h)
       wenn heads.len() == max_items: stop
4. next = Cursor{ snap_seq, eu, space, last_key, erasure_epoch }  (AEAD-versiegelt mit cursor_key)
```

**Aggregat-Fast-Path:** Liegt im Zielbereich nur ein Run als Quelle (alle anderen Quellen haben dort keine Einträge, prüfbar über Fences), können ganze Blöcke anhand der kumulativen Gewichte im SectionIndex übersprungen bzw. vorab gezählt werden. Der Fast-Path liefert exakte Seitengrenzen, ohne Blöcke zu dekodieren. Bei Überlappung (Schatten, Tombstones) sind die Aggregate nur obere Schranken. Dann dient ein Head-Scan der Wahrheit. Auch dann bleibt der Scan auf Head-Blöcke beschränkt (typisch 1–3 % der Nutzlast).

**Cursor-Regeln (deterministisches Abweisen):**

- Cursor ist nur mit dem `cursor_key` des Prozesses gültig (Manipulationsschutz).
- Erscheint beim Fortsetzen `snap_seq < min_retained_seq` ⇒ `CursorExpired`.
- Ist die EU inzwischen nicht mehr `Live` (jede Epoche) ⇒ `Erased`/`Restricted`, nie Teildaten.
- Der Cursor trägt keinen Klartext-Key. Der Key liegt im versiegelten Cursor-Payload.

### 4.3 MVCC und Snapshots

- `Version` = unveränderliche Struktur `{memtables, runs je Partition, manifest_epoch}`, publiziert per `ArcSwap<Version>`. Reader laden wartefrei.
- Die **Snapshot-Registry** führt `min_active_seq` über ein Slot-Array von `AtomicU64` (kein Lock). `max_snapshot_age` begrenzt Pinning. Danach `SnapshotTooOld`.
- Versions-GC: Bei Flush/Compaction werden Versionen unterhalb von `min_active_seq` bis auf die jeweils neueste verworfen.
- Erasure ist **nicht** versioniert (D7): `EuStateTable` ist ein `ArcSwap` auf den jeweils neuesten Zustand und wird bei jedem Zugriff konsultiert.

### 4.4 Flush

1. Immutable Memtable je Partition → je EU eine Sektion → Run (`tmp/`).
2. `sync_data` Run, `sync_dir`.
3. Manifest-Edit `AddRun, set flushed_seq(partition)` anhängen, `sync_data`.
4. Neue `Version` publizieren.
5. WAL-Segmente, deren `max_seq ≤ min(flushed_seq aller Partitionen)` sind, werden zur Löschung freigegeben (`overwrite_zero` Best-Effort → `unlink`).
6. **Kaltes-Partition-Problem:** Partitionen mit ältestem unflushed Record > `max_wal_age` oder bei `wal_bytes > cap` werden zwangsgeflusht, damit eine ruhige Partition das WAL nicht ewig pinnt.

### 4.5 Compaction

- **Auswahl:** pro Partition, größen-gestaffelt (Tiered mit Level-Limits), Scoring über Read-/Space-Amplification. Jobs laufen in `Lane::Compact`, in ≤ 1-MiB-Schritten mit Abbruchpunkten.
- **Ablauf:** Merge pro EU-Sektion (parallelisierbar), Entschlüsselung der Köpfe mit lebenden DEKs, Verwerfen schattierter/abgelaufener Versionen (`< min_active_seq`), neue Sektionen.
- **Install-Check:** Unter `PartitionMaintenance`-Lock wird die EU-Menge des Outputs gegen den aktuellen `EuStateTable` geprüft. Sektionen von inzwischen nicht mehr `Live`-EUs werden **vor** dem Manifest-Edit verworfen (R3).
- **Publish:** Output fsync → Manifest-Edit (`AddRun/DelRun` atomar im selben Edit) → `Version` publizieren. Alte Dateien werden erst bei Drop des letzten `Arc<FileGuard>` ge-`unlink`t (siehe Grace-Regeln in §5.5).
- **Blob-GC:** Wenn Garbage-Anteil > Schwelle oder Löschauftrag → **verbatim** Chunk-Kopie (Lineage und Chunk-Ids bleiben, Generation +1).

### 4.6 Purge-Rewrite (löschgetrieben, ohne Entschlüsselung)

```text
Eingabe: Partition p, Menge T = {eu_tag} der zu entfernenden EUs
Für jede Datei f in p (Run, Blob, Ext), deren Directory/Bloom-Sketch ein t ∈ T melden könnte:
   Run:  neue Datei = RunHeader' · (alle Sektionen mit tag ∉ T, Bytes 1:1) · neues SectionDirectory · Footer
   Blob: neue Datei = Header' · (alle ChunkFrames mit tag ∉ T, Bytes 1:1) · neues ChunkDirectory · Footer
   Alle Frame-CRCs werden beim Kopieren geprüft (Scrub-Effekt). AEAD wird NICHT geprüft (kein Schlüssel nötig).
   fsync(neue Datei) → Manifest-Edit (Replace f→f') → publish → f in Retire-Queue
Wenn eine Datei NUR Tags aus T enthält → kein Rewrite: direkt Retire (unlink).
```

Der Verifier (§5.6) iteriert, bis kein Frame mit `t ∈ T` mehr in lebenden oder zur Löschung anstehenden Dateien existiert.

### 4.7 Cohort-Rewrap (Schlüssel-Löschung)

1. Cohort-Lock (nach Cohort-Id sortiert) holen. `KeyService.create_kek` → `KEK_{g+1}`.
2. Alle Einträge der Cohort mit `KEK_g` lesen, Einträge der zu löschenden EUs **auslassen**, Rest mit `KEK_{g+1}` neu wrappen (nur 32-Byte-DEKs, nie Nutzdaten).
3. `cohort-<id>-g<g+1>.keys` schreiben, `sync_data`, `sync_dir`, Manifest-Edit (Cohort-Generation).
4. Keyring-Replikate (≥ 2 unabhängige Ziele) synchron aktualisieren.
5. Ledger `StageDone(KeyRewrite)`. **Erst jetzt** `KeyService.destroy_kek(KEK_g)`. Ledger `KekDestroyed` + Attestation-Digest, Anchor advance. Alte `…-g<g>.keys` unlinken.
6. Crash zwischen den Schritten: Roll-forward (Schritte sind idempotent, `destroy_kek` ist idempotent).

Cohorts sind auf ≤ 1024 EUs begrenzt. Ein Rewrap kostet also ≤ 1024 Wraps (µs) plus 2 KMS-Calls, unabhängig von der Datenmenge. Batching: `Urgency::Immediate` rewrapt sofort, `Urgency::Batched{window}` bündelt alle Anfragen derselben Cohort innerhalb des Fensters.

### 4.8 Deterministische Crash-Recovery

Recovery ist eine **reine Funktion** von `(Dateien, Ledger, Anchor, KMS-Status)`:

1. `LOCK` nehmen. `CURRENT` → Manifest-Edit-Log replayen (Torn Tail des Manifest-Logs tolerieren).
2. **Anchor/Ledger-Prüfung:** Ledger-Kette verifizieren. `anchor.ledger_head` muss an `anchor.ledger_seq` im Ledger vorkommen (sonst Fork/Truncation ⇒ `LedgerRollback`, fatal). `ledger.head.seq ≥ anchor.ledger_seq`.
3. **Restore-Erkennung:** Ist `manifest.erasure_watermark < ledger.erasure_watermark` ⇒ `RestoreDetected` ⇒ Reconcile-Modus (§5.7) **vor** dem ersten Read.
4. **Dateiabgleich:** Dateien außerhalb des Manifests in `parts/`, `ext/`, `tmp/` → `quarantine/` (Waisen) bzw. löschen (`tmp`). Manifest-Dateien, die fehlen oder deren Footer-/Directory-Hash nicht stimmt → Sektion/Datei unter Quarantäne, `Corruption`-Event.
5. **Keyrings laden**, KEK-Status per `KeyService` prüfen. Unvollständige Rewraps (Intent im Ledger, `StageDone` fehlt) → Roll-forward.
6. **WAL-Replay** ab `min(flushed_seq)`, mit Hash-Kette und **Gap-Regel** (§4.9). Records anwenden, deren `commit_seq > flushed_seq(partition)`. Records für EUs im Ledger-`Erased`-Set (nach Tag) werden **nicht** angewendet.
7. **Tail:** ungültiges Ende abschneiden, neues WAL-Segment beginnen, `visible_seq = last_valid_seq`.
8. **Laufende Aufträge:** Alle nicht-terminalen `ErasureRequest`s aus dem Ledger wieder in den Zustandsautomaten laden (§5).
9. **Invarianten-Check (billig):** Für jede `Erased`-Tag: kein Keyring-Eintrag, kein Sektionsverzeichnis-Eintrag in Dateien, die nicht schon in der Retire-Queue stehen. Verstöße → `PurgeRetry`.
10. `Engine::open` gibt erst nach Schritt 9 `Ok`.

### 4.9 Torn-Write-Erkennung: die Gap-Regel

Batches werden strikt seriell geschrieben und gesynct (Batch *k+1* beginnt erst, wenn `fsync` von *k* zurückkehrte).

- Erster ungültiger Record/Batch an Position *k*. Existiert danach **irgendein** Batch, dessen `prev_chain` auf die erwartete Kette von *k* zeigt und der selbst gültig ist ⇒ *k* war acked ⇒ **Korruption** (nicht Torn): fail-stop mit präzisem Befund (Segment/Offset). Optional Quarantäne und Weiterbetrieb für andere Partitionen in `degraded`.
- Gibt es keinen gültigen Folge-Batch ⇒ **Torn Tail** ⇒ abschneiden. Das ist sicher, weil nichts ge-acked wurde (Ack erfolgt erst nach `fsync` des vollständigen Batches).
- Unterscheidung von Wiederanlauf-Müll: Die Segment-Header-Seriennummer und die Kette verhindern, dass alte Daten eines recycelten Segments als gültig durchgehen. Segmente werden nie recycelt, sondern gelöscht und neu allokiert.

### 4.10 Fehlerverhalten

| Ereignis | Verhalten |
|---|---|
| `fsync(WAL)` → EIO | Alle Commits des Batches: `DurabilityUnknown`. Engine → `Poisoned` (nur Reads auf bereits publizierten Zustand, kein weiterer Write). **Kein Retry.** Neustart + Recovery (§4.8) entscheidet. |
| `fsync(Run/Blob/Manifest)` → EIO | Job scheitert, Output verworfen, Quelle bleibt gültig. Bei Manifest-fsync-Fehler → `Poisoned`. |
| ENOSPC im WAL | Nicht erwartbar (vorallokiert). Wenn doch: `Poisoned`. |
| ENOSPC bei Flush/Compaction | Job-Abbruch + Cleanup. Engine → `ReadOnlyDegraded` (Writes erhalten `Busy{NoSpace}`), Reads laufen. **Löschpfad bleibt aktiv** (Lösch-Reserve in `reserve/`: vorallokiert für Ledger-Appends, Keyring-Generation, Purge-Temps; nur `Lane::Urgent` darf sie verbrauchen). |
| CRC-/AEAD-Fehler in Block | `Corruption{file, section}`; Sektion unter Quarantäne; andere EUs unberührt. Scrubber meldet Befund. Keine stille Reparatur. |
| `KeyService` temporär nicht erreichbar | `KeyUnavailable{retry_after}` für betroffene EUs. Gecachte DEKs (TTL) bedienen weiter. Lösch-Schritte warten/wiederholen (Backoff). |
| KEK im KMS `Destroyed`, Ledger bestätigt Löschung | `KeyDestroyed` ⇒ EU-Zustand `Erased`. Das ist **gewollt**. |
| KEK fehlt, aber Ledger kennt **keine** Löschung | `KeyLost`. EU → `Orphaned`, Alarm, **keine** automatische Löschung/Neuaufbau. Reads liefern `KeyLost`. |
| Hash-Kette/Anchor-Verletzung | `LedgerRollback`/`RestoreDetected`, siehe §4.8. |
| Schlüsselverlust Keyring-Volume | Ohne Replikat: Datenverlust der Cohort (kryptografisch unvermeidbar). Daher Pflicht: ≥ 2 Keyring-Replikate in getrennten Failure Domains. |

### 4.11 Ressourcenkontrolle

- **Memory-Budget:** `MemoryBudget { memtables, section_index_cache, block_cache(ciphertext), key_cache, in_flight }` mit atomaren Zählern. Überschreitung → Backpressure (§4.1.8), nie OOM. Plaintext-Caches halten Einträge als `Zeroizing<Vec<u8>>` mit `eu_tag`-Index für `evict_eu(tag)`.
- **FD-Budget:** `FdCache` (LRU, `max_open_files`) mit `FdPermit`-Semaphor. Vorrang für WAL, Ledger und aktive Blobs (gepinnt).
- **I/O-Lanes:** `Urgent (WAL-fsync, Ledger, KMS, Purge) > Read > Flush > Compact > Scrub`. Token-Bucket je Lane, `Urgent` hat garantierten Mindestanteil (≥ 20 %) und kann Compaction-Jobs derselben Partition per Cancel-Flag an Chunk-Grenzen verdrängen.
- **Threads:** feste Pools pro Lane (Executor-Port). Sequencer = 1 Thread.

---

## 5. End-to-End-Löschprotokoll

### 5.1 Anfrage

```rust
pub struct ErasureRequest {
    pub id: ReqId,                       // Idempotenz: gleicher id ⇒ gleiche Instanz
    pub scope: ErasureScope,             // Eu(EuId) | EuSet(Vec<EuId>) | DeletionGroup(u128) | Tenant(TenantId)
    pub urgency: Urgency,                // Immediate | Batched{ window }
    pub purge_deadline: Duration,        // Parameter der Politik, nicht des Kerns
    pub derived: DerivedPolicy,          // FollowDerivation(default) | None
    pub requester: Principal,
}
```

### 5.2 Zustandsautomat

```text
                           ┌────────────── Hold aktiv ──────────────┐
                           ▼                                        │
Received ─► Evaluating ─► (Blocked[hold])  ◄── Release ─┐           │
   │            │                                        │           │
   │            └─(kein Hold)─► LogicallyErased ─► KeysDestroying ─► KeysDestroyed
   │                              (T0: unsichtbar)        │               │
   │                                     └──► Cascading (parallel) ◄──────┤
   │                                                                      ▼
   │                                                                  Purging ─► Verifying ─► Completed
   └─► Rejected(invalid scope)                                              │         │             │
                                                                            └─ Failed(retryable, Backoff, Alarm)
                                                                                        │
                                              CompletedWithResidual ◄───────────────────┘   BackupHorizon (Zusatzspur)
```

**Persistenz:** Jeder Übergang wird als `StageIntent` **vor** und `StageDone` **nach** dem Schritt im Ledger (fsync) festgehalten. Jeder Schritt ist idempotent. Nach einem Crash bestimmt der letzte Ledger-Eintrag den Wiedereintritt.

### 5.3 Phase 1 – Received/Evaluating

1. `ErasureRequested` ins Ledger (fsync) → ab hier gilt der Auftrag als angenommen. Dublette per `ReqId` liefert dieselbe Instanz.
2. **Scope-Auflösung** (unter Snapshot `S`): `Tenant` ⇒ alle Cohorts. `DeletionGroup` ⇒ alle EUs der Partition-Gruppe (aus Keyrings). `Eu(s)` ⇒ validiert.
3. **Abhängigkeits-Closure:** Aus dem Systemspace `SYS_DERIV` (reverse Kanten, in der Sektion der Quell-EU versiegelt) transitiv alle abgeleiteten EUs/Artefakte bestimmen (Tiefenlimit, Zyklenerkennung). Artefakte mit mehreren Quellen werden nach `OnSourceErasure` behandelt (`Delete` ⇒ in Löschmenge, `Invalidate` ⇒ als `Stale` markiert, bleibt aber hidden bis Rebuild). Die Closure wird **vor** der Schlüsselvernichtung berechnet, weil danach die Kanten unlesbar sind.
4. **Hold-Check** (`held`-Flag im Keyring-Eintrag + Ledger-Holds): trifft ein Hold ein Element der Menge, wird die **gesamte Anfrage** `Blocked(HoldRef)`. Die Mengenauflösung wird eingefroren. Optional (Policy `restrict_on_block = true`) geschieht dennoch der Übergang `Restricted` (hidden, aber unzerstört), siehe §5.8.

### 5.4 Phase 2 – Sofortige logische Löschung

Ein einziger `Control`-Record im Commit-Log (Lane `Urgent`, im nächsten Group-Commit-Batch, typisch < 1 ms bis wenige ms):

```text
EraseLogical { req, tags: [eu_tag…] (inkl. Closure), erase_epoch E = commit_seq }
```

Beim Apply (nach `fsync`):
- `EuStateTable`: alle Tags → `Erasing(E)` (atomarer `ArcSwap`).
- Idempotenz-Einträge der EUs invalidiert (Wiederholung ⇒ `Erased`, **nicht** erneute Ausführung).
- Offene Cursors dieser EUs → ungültig. Key-/Section-Index-/Block-/Plaintext-Caches `evict_eu(tag)`. `KeyHandle.revoked = true` (Zeroize bei letzter Drop).
- Laufende Compactions/Flushes: Install-Check wird auf `erasure_epoch` geprüft (R3).
- Neue Writes auf die EU: `Erased`. Neue Reads: `Erased`. **Das ist T₀** („unsichtbar", in Receipt: `t_logical`).
- Da Closure **im selben Record** steht, ist die Sichtbarkeit von Quelle und Abgeleitetem atomar.

### 5.5 Phase 3 – Kryptografische Löschung

1. Ledger `StageIntent(KeysDestroying)`.
2. **Cohort-Rewrap** nach §4.7 pro betroffener Cohort (Tenant-Scope: einfach alle Cohort-KEKs zerstören + `tenant_tag_key`-Generation vernichten, kein Rewrap nötig).
3. `destroy_kek` liefert `DestroyAttestation`. Ist `irreversible = At(t)` (z. B. Cloud-KMS mit Karenzzeit), bleibt der Auftrag im Zustand `KeysDestroying(pending_until=t)`. Der Receipt weist dies als Restrisiko aus und läuft nicht still als „fertig".
4. Ledger `KekDestroyed`. **Anchor advance** `(ledger_seq, head, erasure_watermark)`. Erst danach: `KeysDestroyed`, `t_crypto`.
5. Konsequenz für Backups: Jede alte Keyring-Generation, auch in Backups, ist wertlos, denn ihre KEK existiert nicht mehr. Daten-Backups bleiben lesbar **für Überlebende** dank aktuellem Keyring (§5.7).
6. Konsequenz für Metadaten: Keys, Attribute, Gewichte, Abhängigkeiten, Section-Indizes, Bloom-Filter und WAL-OpParts der EU sind ohne DEK nicht mehr lesbar (D3).

### 5.6 Phase 4 – Kaskade und physische Bereinigung

**Cascading (parallel zu Purging):** Für jede registrierte Extension: `on_erasure(notice{req, tags})`. Die Extension filtert sofort, bereinigt In-Memory-Strukturen, schreibt Segmente neu/verwirft sie und liefert `ExtAck{digest}`. Ledger `ExtAck`. Bis zum Ack gelten die Segmente als `Tainted` und werden vom `filter_visible` abgedeckt. Fristüberschreitung ⇒ `CompletedWithResidual` plus Alarm.

**Purging** (Prioritäts-Lane `Urgent`, Reserve nutzbar):

1. **WAL:** Betroffene Partitionen force-flushen → `flushed_seq` ≥ letzte Seq der EUs → WAL-Segmente rollen → alle Segmente, deren Inhalt vollständig geflusht ist, `overwrite_zero` (Best-Effort) + `unlink`. (Die Reste wären ohnehin ohne DEK unlesbar.)
2. **Runs/Blobs/Ext-Segmente:** Purge-Rewrite nach §4.6 (verbatim). Dedizierte Partitionen/Dateien nur mit Tags aus `T` ⇒ `unlink`.
3. **Retire:** Alte Dateien werden nach Manifest-Edit in die Retire-Queue gestellt. Ge-`unlink`t, sobald kein Reader sie mehr hält. **Grace:** `purge_grace` (Default 60 s) nach Manifest-Edit werden Snapshots, die die Datei noch pinnen, mit `SnapshotTooOld` für Reads *auf diese Partition* invalidiert (Überlebende können mit neuer Version erneut lesen). Wegen D7 und vernichteter Schlüssel war die Wartezeit zuvor kryptografisch unkritisch. Sie ist nur der Aufräum-SLA.
4. **Manifest-Alt-Generationen:** nach Edit-Log-Rollover alte `MANIFEST-*` löschen (EU-frei, I4, daher unkritisch, aber hygienisch).
5. **Caches:** bereits in Phase 2 evictet. Verifier prüft erneut.

**Verifying:**

1. Tag-Scan: Für alle Dateien der betroffenen Partitionen **und** aller noch nicht `unlink`ten Retire-Dateien: SectionDirectory/ChunkDirectory/Frame-Header nach `t ∈ T` durchsuchen (Klartext, billig). Ergebnis: Menge `R` (Reste).
2. `R` leer ⇒ weiter. Sonst `R` ⊆ Retire-Queue mit Pin ⇒ Wartegrund `PinnedByReader` (zeitlich begrenzt), sonst neuer Purge-Durchlauf (Fixpunkt-Iteration; deckt R3).
3. WAL-Scan: kein Record mit `eu_tag ∈ T` in aktiven Segmenten.
4. Keyring-Scan: kein Eintrag. KMS: `kek_status` aller alten KEKs ≠ `Live`.
5. Ext-Acks vollständig.
6. Ledger `PurgeVerified{residual_digest}`.

### 5.7 Backups, Restore und Rollback-Schutz

**Backup-Modell**
- `begin_backup()` pinnt Manifest/Version, schreibt `{manifest_epoch, ledger_seq, anchor_state}` in den Backup-Header. Gesichert werden Daten-Dateien (nur Ciphertext).
- **Keyring-Backups laufen in einer getrennten Kette** mit „nur aktuelle Generation". Die Replikate werden bei jedem Rewrap synchron aktualisiert und alte Generationen sofort entfernt. Alte Keyrings in Alt-Backups sind durch KEK-Destroy ohnehin tot.
- Ledger und Anchor liegen auf getrenntem Volume/Service und sind **nicht** Teil des Daten-Restores.

**Restore-Reconcile** (automatisch beim `open`, bevor irgendein Read erlaubt ist):

1. Erkennung (§4.8 Schritt 3): Datenstand älter als Ledger-Watermark.
2. Engine startet im Zustand `RestoreQuarantine`: nur Ledger/Katalog lesbar, keine Nutzer-Reads.
3. Für jeden `Erased`-Tag im Ledger: Tag in restaurierten Dateien suchen (billiger Tag-Scan) und per Purge-Rewrite entfernen. Gleichzeitig Keyring prüfen (aktuelle Generation kennt den Eintrag nicht mehr ⇒ unlesbar). Zustand wird erneut auf `Erased` gesetzt (I8). Das passiert unabhängig davon, ob der Datenbestand nach Restore die Daten noch enthält.
4. Holds und laufende Aufträge aus dem Ledger werden wiederhergestellt.
5. Ledger `RestoreReconciled`. Erst dann `Running`.
6. **Anchor-Pflicht:** `RollbackAnchor::advance` ist CAS und monoton. Ein älterer Datenstand kann den Anker nie zurücksetzen. Ein Angreifer, der Ledger **und** Daten auf einen alten Stand zurückspielt, scheitert am Anchor (`LedgerRollback`).
7. **Ehrliche Grenze:** Wird zusätzlich der Anchor-Dienst kompromittiert/zurückgesetzt, ist Schutz nur noch kryptografisch (KMS-Zerstörung), nicht mehr protokollarisch.

**Backup-Horizont (Zusatzspur im Receipt):** Der `BackupCatalog`-Port liefert `oldest_backup_ts`. Alle Backups vor `t_crypto` enthalten ausschließlich unlesbaren Ciphertext der EU. Der Receipt trägt `backup_status: Unreadable(since t_crypto)` und, falls Policy zusätzliche physische Backup-Expiry verlangt, `PendingExpiry(until)` bis alle betroffenen Generationen abgelaufen sind (`BackupHorizonReached`-Event ⇒ Addendum).

### 5.8 Holds, Restrictions, Governance

```rust
pub struct Hold { pub id: HoldId, pub scope: HoldScope /* tags, Tenant */, pub kind: HoldKind /* LegalHold | Restriction */,
                  pub placed_by: Principal, pub reason_ref: OpaqueRef, pub created: UnixMillis }
```

- **Platzierung:** nur mit `Authority::verify(HoldOp::Place{..}, sig)`. Hold wird in Ledger **und** im `held`-Flag der Keyring-Einträge verankert.
- **Wirkung:**
  - `LegalHold`: Löschaufträge → `Blocked`. Schlüssel und Frames bleiben unangetastet (I7). Lesezugriff nur normal, es sei denn `Restriction` ist zugleich gesetzt.
  - `Restriction`: Reads → `Restricted` (außer mit `LegalAccessCap`, die von der Authority signiert wird). Writes blockiert.
- **Release:** ausschließlich `HoldRelease` mit gültiger Authority-Signatur **fremder** Principal (Vier-Augen-Prinzip: ≠ Requester). Der Kern bietet **kein** `force`-Flag, keinen Bypass-Pfad und keine Konfigurationsoption.
- **Durchsetzung in drei Schichten (Defense in Depth):** (1) `ErasureCoordinator` (Evaluating), (2) `KeyringWriter::drop_entry` verweigert `held`-Einträge auf unterster Ebene, (3) Purge verweigert `Remove`/`unlink` für Dateien, die Tags in `held` enthalten, solange Hold aktiv.
- **Zu spät:** Hold auf bereits `KeysDestroyed`-EU wird mit `TooLate` abgelehnt und im Ledger protokolliert. Nichts wird wiederhergestellt.
- Nach Release setzt ein blockierter Auftrag automatisch am Zustand `Evaluating` wieder an.

### 5.9 Receipts (maschinenprüfbar)

```rust
pub struct ErasureReceipt {
    pub req: ReqId, pub scope_tag_digest: [u8; 32],        // BLAKE3 über sortierte Tags, kein Klartext
    pub t_requested: UnixMillis, pub t_logical: UnixMillis, pub erase_epoch: Seq,
    pub t_crypto: Option<UnixMillis>, pub t_physical: Option<UnixMillis>, pub t_verified: Option<UnixMillis>,
    pub stages: Vec<StageEvidence>,                         // {stage, ledger_seq, evidence_digest}
    pub kek_attestations: Vec<DestroyAttestation>,
    pub ext_acks: Vec<ExtAckEvidence>,
    pub verifier_report_digest: [u8; 32],                   // Tag-Scan-Ergebnis über Dateien/WAL/Keyrings
    pub ledger_head: LedgerRef, pub anchor: AnchorAttestation,
    pub holds_encountered: Vec<HoldRef>,
    pub status: ReceiptStatus,                              // Complete | CompleteWithResidual | Blocked | Failed
    pub residual: Vec<ResidualRisk>,                        // siehe unten
    pub signature: Vec<u8>, pub signer: KeyId,
}
pub fn verify_receipt(r: &ErasureReceipt, ledger: &LedgerSlice, att: &AnchorAttestation, pk: &PublicKey) -> Verdict;
```

`verify_receipt` ist eine reine, offline lauffähige Funktion: Signatur, Hash-Kette des Ledger-Ausschnitts, Anchor-Attestation, Stufenreihenfolge und Zeitmonotonie.

**ResidualRisk (ehrlich, im Receipt enthalten):**

| Risiko | Behandlung |
|---|---|
| Prozessspeicher/Swap/Core-Dumps vor Zeroize | `Zeroizing` + `mlock` im Adapter (außerhalb Kern), Zeroize-Audit. Garantie nicht absolut. |
| SSD-Wear-Leveling, Over-Provisioning, FS-Journale, Snapshots auf Dateisystemebene | Nur Ciphertext dort. Schutz = KEK-Vernichtung. `overwrite_zero` nur Best-Effort. |
| KMS-Karenzzeit (`IrreversibleAt::At`) | `CompleteWithResidual`, bis Frist erreicht. |
| `eu_tag`-Bloom/Tag-Membership alter Kopien | Pseudonym, 64 Bit, teilbar nur mit `tenant_tag_key`. Dieser wird bei Tenantlöschung vernichtet. Für EU-Löschung ist Wiederidentifikation nur mit Kandidaten-`EuId` + Schlüssel möglich. Rest-Risiko dokumentiert. |
| Metadaten-Seitenkanäle (Dateigrößen, Zeitpunkte, Frame-Längen) | `pad_to` optional, sonst Rest. |
| Außerhalb des Kerns (LLM-Provider, App-Caches, Replikate ohne Vertrag) | Nicht abgedeckt. `ReplicationHook` verlangt Erasure-Vertrag. |
| Ext-Verstoß gegen Segment-Vertrag | Nur durch Konformitäts-/Canary-Tests prüfbar, kein Beweis. |
| Fd-gepinnte gelöschte Dateien durch Reader | `purge_grace`, danach Invalidierung. |

---

## 6. Concurrency-Modell & Verifikationsstrategie

### 6.1 Lock-Ordnung (strikt aufsteigend, Verstoß ⇒ Debug-Assert via `LockLevel`-Token)

| Ebene | Primitive | Zweck | Anmerkung |
|---|---|---|---|
| L0 | `LifecycleGate` (RwLock) | Open/Close/Poison | Alle Operationen halten *Read*. Write nur bei Zustandswechsel. |
| L1 | `ErasureCoordinator` (Mutex pro `ReqId`) | Zustandsübergänge eines Auftrags | Nie über KMS-Call hinaus gehalten außer Rewrap-Schritt (L3 darunter). |
| L2 | `CatalogWriter` (Mutex) | Manifest-Edits, Spaces/Partitionen | Kurz, I/O nur Manifest-Append. |
| L3 | `KeyringWriter[cohort]` (Mutex, aufsteigend nach Cohort-Id) | Rewrap, DEK-Anlage | Darf KMS-Calls mit Timeout ausführen. |
| L4 | `PartitionMaintenance[p]` (Mutex, aufsteigend nach `PartitionId`) | Flush/Compaction/Purge-Install | Install-Check für Erasure-Epoche. |
| L5 | `LedgerWriter` (Mutex) | Ledger-Append | Blattnah. |
| L6 | `FdCache`/`CacheShard[i]` (Mutex, sharded) | Datei-Handles, Caches | Blatt, nie gehalten über I/O. |
| — | **Sequencer** | WAL-Writer, EU-State-Mutationen | Single-Thread-Owner (Channel), *kein* Lock. |

**Regeln**
1. Nie eine niedrigere Ebene nehmen, während eine höhere gehalten wird.
2. **Kein Lock über Extension-Callbacks** (`on_erasure`, `segments`). Extensions dürfen die Read-API re-entrant nutzen, aber keine Engine-Locks erwarten.
3. Lock über blockierende Ports (KMS) nur mit Timeout und nur L1/L3.
4. Eigentum-Modell statt Locks, wo möglich: Sequencer besitzt WAL und EU-State-Schreibzugriff, Flush-Jobs besitzen ihre Memtable.

**Lock-freie Lesepfade:** `ArcSwap<Version>`, `ArcSwap<EuStateTable>`, `ArcSwap<KeyTable>`, `AtomicU64` für `visible_seq`/`erasure_epoch`, `AtomicBool` für `KeyHandle.revoked`. Reader nehmen höchstens Blatt-Locks (Cache-Shard/FD-Cache). Persistente Memtables (`imbl::OrdMap`) erlauben, dass Reader eine konsistente Wurzel halten, während der Writer neue Wurzeln veröffentlicht.

### 6.2 Kritische Race-Nahtstellen

| ID | Race | Auflösung |
|---|---|---|
| R1 | Reader hat EU als `Live` gesehen, Erasure-Commit passiert vor Rückgabe | Check-after-decrypt gegen `revoked` und `erasure_epoch`. Linearisierungspunkt = erster `EuStateTable.load`. Ergebnis nach `E` wird verworfen. |
| R2 | Writer hat mit altem DEK vorversiegelt, Sequencer hat inzwischen `EraseLogical` verarbeitet | Sequencer prüft EU-Zustand serialisiert. Record wird abgelehnt (`Erased`). |
| R3 | Compaction mergt EU, die gerade gelöscht wird | Install-Check unter L4. Bereits publizierte Fälle fängt der Verifier-Fixpunkt. |
| R4 | WAL-Truncate vs. noch ungeflushte Partition | Truncate nur bei `flushed_seq` **aller** Partitionen, Manifest-Edit zuerst. |
| R5 | Neue EU in Cohort, während Rewrap läuft | Beide unter L3 serialisiert. Neue DEKs gehen in die *neue* Generation, `create_eu` wartet kurz auf den Rewrap. |
| R6 | Backup-Snapshot vs. gleichzeitige Löschung | `BackupGuard` pinnt Version und notiert Watermark. Spätere Löschung wird per Reconcile nachgezogen (§5.7). |
| R7 | Cursor-Fortsetzung vs. Purge-Rewrite | Cursor referenziert Seq, nicht Datei. Neue Version wird aufgelöst. `SnapshotTooOld` nur wenn `snap_seq` abgelaufen ist. |
| R8 | Idempotenter Retry vs. Erasure | `SYS_IDEM`-Einträge werden in Phase 2 invalidiert, Retry ⇒ `Erased`. |
| R9 | Extension-Ack vs. neue Writes | Writes auf erased EUs abgelehnt (R2), damit gibt es keine neuen Ableitungen. |
| R10 | Doppelter Löschauftrag/gleichzeitige Aufträge auf überlappenden Mengen | `ReqId`-Idempotenz, Overlap ⇒ Aufträge hängen sich an denselben Tag-Zustand an (State-Table kennt `Erasing(E)` bereits). Mengen-Differenz wird neu berechnet. |
| R11 | `fsync`-Fehler während paralleler Flush/Publish | Poison-Flag (`AtomicBool`) wird vor jedem Publish geprüft. |

### 6.3 Verifikationsstrategie

**(a) Formale Modelle (TLA⁺/PlusCal, TLC + Apalache)**

Modell 1 (Commit/Recovery): Group Commit, WAL-Kette, Manifest-Edits, Crash an beliebiger Stelle, Torn Writes (Seitenpräfix-Persistenz, Umsortierung ohne Barrier).
- *Safety:* `AckedDurable` (ge-ackter Commit überlebt jeden Crash), `PrefixConsistency`, `NoPhantomRead`.

Modell 2 (Erasure-Automat inkl. Holds, Restore, Cohort-Rewrap, KMS-Latenz):
- `ErasedNeverReadable`: Nach `EraseLogical(E)` liefert kein Read Daten der EU.
- `KeyDestroyOrderedAfterIntent`: KEK-Destroy nur nach durablem Intent und nach durablem Rewrap.
- `HoldSafety`: Kein `KekDestroyed`/Purge für Tags unter aktivem Hold.
- `NoResurrection`: Nach Restore/Recovery wird kein `Erased`-Tag wieder `Live`.
- `CascadeComplete`: `Completed ⇒` alle Ext-Acks, Tag-Scan leer.
- *Liveness* (unter Fairness): Jeder nicht-blockierte Auftrag erreicht `Completed` oder `CompleteWithResidual`.

**(b) Concurrency-Modelchecking:** `loom` für `ArcSwap`-Publish/Epoch-Checks, Snapshot-Registry, Sequencer-Handoff. `shuttle` (randomisiert) für größere Szenarien. Lock-Level-Tokens prüfen Ordnung im Debug-Build.

**(c) Deterministische Simulation (`lethe-sim`, FoundationDB-Stil)**

Alle Nichtdeterminismen sind Ports (D10). Ein einziger Seed bestimmt Scheduling, Fehlerinjektion und Workload.
- **`SimFs`:** Seiten-granulare Persistenz, *ungesyncte* Writes verlieren sich beim Crash teilweise/umsortiert, `fsync`-Fehler mit Semantik „Dirty Pages können verworfen werden", EIO/ENOSPC, Bit-Flips, Short Writes, verlorene `rename`s ohne `sync_dir`. Journalisiert **jede je geschriebene Seite** (Remanenz-Modell für SSD/FS-Journale).
- **`SimKms`:** Latenz, Ausfälle, Karenzzeit-Zerstörung, irreversibles `destroy`, Duplikat-Calls.
- **`SimAnchor`:** CAS-Semantik, Ausfälle, Rollback-Versuche (müssen scheitern).
- **`SimExecutor`:** Einzelthread-Scheduler mit seed-gesteuerter Interleaving-Reihenfolge, Lane-Prioritäten.
- **Crash-Punkte:** Automatisch **vor und nach jedem I/O-Call** (inkl. Lösch-Schritten) enumeriert oder zufällig gesamplet. Nach jedem Crash: Restart → Recovery → Invarianten-/Modellvergleich.
- **Referenzmodell:** In-Memory-MVCC-Orakel (`BTreeMap` + Versionsketten + EU-Zustand). Jede Operation und jeder Crash-Restart wird gegen das Modell geprüft (`AckedDurable`, Snapshot-Konsistenz, Paging-Budgets exakt, Idempotenz).
- **Linearisierbarkeit:** Historien (Porcupine/Elle-Stil) für Einzelkey-Register und SI-Transaktionen.

**(d) Fault-Injection auf echter Infrastruktur:** `dm-flakey`/`dm-log-writes` (CrashMonkey/ALICE-Stil) für echte Dateisysteme (ext4, xfs, btrfs). `LD_PRELOAD`/FUSE-Layer für EIO/ENOSPC. Kill -9 Schleifen unter Last. KMS-Mock mit Chaos (Toxiproxy).

**(e) Fuzzing & Property-Tests:** `cargo-fuzz` auf alle Decoder (`lethe-format`) und auf den Recovery-Scanner mit mutierten Dateien. `proptest` für Codec-Roundtrips, Gewichts-Arithmetik des Paging (kein Overflow, Fortschrittsgarantie), Prefix-Kompression. `Kani` für Bounds/Overflow in Frame- und Paging-Code.

**(f) Lösch-Orakel (`lethe-oracle`)** – die Kernverifikation der Löschgarantie:

1. **Canary-Injektion:** Jede Test-EU erhält pro Space eindeutige Hochentropie-Canaries (Klartext-Marker) in Keys, Attributen, Bodies und Abhängigkeiten. Zusätzlich werden DEK/KEK-Bytes als „Key-Canaries" geführt.
2. **Zustandsbasierte Erwartungen** (aus dem `SimFs`-Seitenjournal inkl. *aller historischen Versionen*):

   | Zeitpunkt | Erwartung |
   |---|---|
   | jederzeit | Klartext-Canary der EU **nie** in irgendeiner Seite (nie Klartext auf Disk). |
   | nach `EraseLogical` | Jeder Read ⇒ `Erased` (auch alte Snapshots, alte Cursors, Retries, Extension-Antworten). |
   | nach `KeysDestroyed` | Orakel versucht Entschlüsselung **aller** historischen Seiten mit **allem**, was erreichbar ist (aktuelle Keyrings, KMS-Live-Objekte, Prozessspeicher-Abbild im Sim, Backup-Kopien). Erwartung: **kein** Erfolg. |
   | nach `Purging` | Kein Frame mit `eu_tag` in lebenden Dateien/WAL. Historische Seiten dürfen existieren (Remanenz), sind aber unlesbar. |
   | nach `Restore` | `Erased`-Tags tauchen nicht wieder als `Live` auf (I8). Reconcile entfernt Reste. |
   | unter Hold | Keine Vernichtung, keine Entfernung (I7), alle Reads über `LegalAccessCap` funktionieren. |
3. **Überlebende bleiben intakt:** Gleiches Modell prüft, dass nicht-gelöschte EUs in derselben Partition/Cohort nach Purge-Rewrite und Rewrap **bytegleich lesbar** sind (verbatim-Beweis: Hash der Sektionen vor/nach).
4. **Prozess-Ebene:** Integrationstest mit realem Prozess: `gcore` nach Löschung, Scan nach Canaries und DEK-Bytes. Zeroize-Registry (`KeyMaterialRegistry`) prüft, dass alle `KeyHandle`s dropped sind. Ergebnis wird als „best effort" im Testbericht deklariert, nicht als Beweis.
5. **Extension-Konformitätssuite:** Jede Extension muss `ext-conformance` bestehen (Canary durch Embeddings/Summaries/Caches, Neustarts, Teilausfälle). Ohne grünes Ergebnis kein Eintrag im Registry-Whitelist.
6. **Receipt-Orakel:** `verify_receipt` wird gegen simulierte Ledger-Manipulationen (Truncation, Reorder, Forks, Replay) getestet.

**(g) Qualitätsgates in CI:** `cargo-geiger` (Kern `unsafe`-frei), `cargo-vet`/`cargo-deny` (Supply Chain), Mutationstests auf Recovery-/Erasure-Crates (`cargo-mutants`), Performance-Regression (Group-Commit-Latenz, Paging-I/O, Purge-Dauer vs. Partitionsgröße), Soak-Tests mit zufälligen Löschanfragen unter Dauerlast.

---

## Anhang A – Default-Parameter (Startwerte, per Config überschreibbar)

| Parameter | Default |
|---|---|
| `max_batch_bytes` / `max_delay` | 4 MiB / 50–500 µs adaptiv |
| WAL-Segment | 64 MiB vorallokiert |
| `inline_max` | 1 KiB |
| HeadBlock (Klartext) | 16 KiB |
| Blob-Chunk (Roh) | 64 KiB (zstd Level 3) |
| Cohort-Größe | ≤ 1024 EUs |
| `max_wal_age` | 5 min |
| `max_snapshot_age` | 10 min |
| `purge_grace` | 60 s |
| `purge_deadline` | Parameter der Politik (Empfehlung: ≤ 24 h Immediate, ≤ 30 Tage Batched) |
| Lösch-Reserve | 256 MiB je Volume + 16 MiB Ledger |
| `Urgent`-Mindest-I/O-Anteil | 20 % |
| Key-Cache-TTL | 5 min (Invalidierung bei `evict_eu` sofort) |
| Keyring-Replikate | ≥ 2, getrennte Failure Domains |

## Anhang B – Offene Entscheidungen und bewusste Trade-offs

1. **Ordnung nur pro EU (D5)** ist die größte Einschränkung. Sie ist der Preis für löschlokales Layout und versiegelte Metadaten. Cross-EU-Abfragen müssen Extensions übernehmen.
2. **Pseudonyme `eu_tag`s im Klartext** erlauben schlüssellosen Purge, bleiben aber Pseudonyme (kein Anonymat). Alternative (reine Verzeichnissicht ohne Tags) würde Purge nach Schlüsselvernichtung unmöglich machen, weil Sektionsverzeichnisse nicht mehr lesbar wären.
3. **Cohort-Rewrap statt KMS-Objekt pro EU:** skaliert auf Millionen EUs, kostet aber ≤ 1024 Wraps je Löschung. Eine KMS-Objekt-pro-EU-Variante ist als `KeyService`-Strategie austauschbar.
4. **Keyring als Single Point of Loss:** Löschgarantie und Verfügbarkeit stehen in Spannung. Gelöst durch Pflicht-Replikation, nicht durch Archivierung alter Generationen.
5. **Single-Sequencer:** begrenzt den Durchsatz auf einen Core für Prüfung/Serialisierung. Das Vorverarbeiten (Verschlüsselung, Kompression) läuft parallel. Skalierung über mehrere Engine-Instanzen (Tenant-Sharding) ist vorgesehen, Cross-Instanz-Atomarität nicht.
6. **KMS-Karenzzeiten** (Cloud-KMS mit 7–30 Tagen) verhindern „sofort irreversibel". Receipt-Status `CompleteWithResidual` ist die ehrliche Antwort. Für strenge SLAs ist ein KMS mit sofortiger Zerstörung zu wählen.
