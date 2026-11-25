# SYS-3 — Öffentliche Kernfassade (`contextra-storage`)

Ersetzt die in R14 verstreuten API-Skizzen. Abhängigkeiten: SYS-1 (Typen, Ports, Fehler), SYS-2 (Datenmodell), SYS-4 (Zahlen). Gate: G-W (Schreib-API), G-R (Lese-API), G-E (Löschen). Diese Datei legt **Signaturen und Semantik** fest; Abläufe stehen in den Zyklen (CYC-*).

## 1. Grundsätze

1. Die gesamte Fassade ist **synchron** und blockierend (`INV-PORT-1`). Die Async-Hülle liegt in `engine::kernel` (`spawn_blocking`). Jede Methode nennt unten ihre **Blockierklasse**: `B0` kein Warten außer kurzer Rang-Sperren, `B1` kann auf Credit/Provider/Platte warten, `B2` kann bis zu einer Frist warten (nur `Wait`-Pfade).
2. Alle Typen sind `Send + Sync` (außer `Cursor`, `Send`). `Storage` ist ein `Arc`-Handle; Klonen ist billig.
3. Es gibt keinen globalen Zustand. `open` ohne explizit übergebene Ports ⇒ `InvalidInput` (DEC-43).
4. Jede Methode liefert `Result<_, Error>`. Fehlerklassen und Retry-Semantik: SYS-1 §6. Die Zuordnung Methode → mögliche Fehler steht in §9.
5. Kein Rückruf in Aufrufercode unter Sperren. `Observer`, `SegmentOwner`, `Consumer` werden nie unter Rängen gerufen (SYS-1 §5).
6. Ids und Zeitwerte: `Epoch`/`Stamp` aus SYS-1 §2; `Receipt` trägt immer `instance_id`. Es gibt keine API, die Stamps verschiedener Instanzen vergleicht.
7. Öffentliche Aufrufer sehen keine Spaces `< 256` (`ReservedSpace`).

## 2. Lebenszyklus

```rust
pub struct Ports {                         // alle Pflicht außer den mit ? markierten
    pub fs: Arc<dyn Fs>, pub clock: Arc<dyn Clock>, pub entropy: Arc<dyn Entropy>,
    pub anchor: Arc<dyn Anchor>, pub keys: Arc<dyn KeyProvider>,
    pub cipher: Arc<dyn Cipher>, pub mac: Arc<dyn Mac>, pub kdf: Arc<dyn Kdf>, pub checksum: Arc<dyn Checksum>,
    pub metrics: Arc<dyn Metrics>, pub observer: Arc<dyn Observer>, pub budget: Arc<dyn MemoryBudget>,
    pub authority: Option<Arc<dyn Authority>>,   // Pflicht, wenn LiftPolicy::FourEyes
    pub host_probe: Option<Arc<dyn HostProbe>>, pub cold: Option<Arc<dyn ColdTier>>,
}
pub struct OpenConfig {
    pub root: PathBuf, pub ledger_dir: Option<PathBuf>,       // None ⇒ <root>/ERASE-LEDGER (nur Development oder mit Ack)
    pub mode: OpenMode,                                       // ReadWrite | ReadOnly
    pub profile: ComplianceProfile,                           // SYS-5 §4
    pub anchor_policy: AnchorPolicy,                          // Required (Standard bei Encrypted/Authenticated) | Optional
    pub acks: Acknowledgments,                                // acknowledged_weak_keys, acknowledged_weak, ledger_same_device_ack
    pub params: Params,                                       // SYS-4 §2, Art O/P
}
impl Storage {
    pub fn create(cfg: &OpenConfig, ports: &Ports, fmt: FormatParams) -> Result<InstanceId, Error>;   // schreibt FORMAT (Art F), B1
    pub fn open(cfg: OpenConfig, ports: Ports) -> Result<(Storage, OpenReport), Error>;               // CYC-6 §3, B1
    pub fn close(self, mode: CloseMode) -> Result<CloseReport, Error>;                                // Clean | Fast, CYC-6 §5
    pub fn tune(&self, t: Tuning) -> Result<(), Error>;                                               // nur Art R, B0
    pub fn state(&self) -> InstanceState;                       // Normal | DiskFull | Poisoned | ReadOnly(Reason) | LockedKeys | RestoreQuarantine
    pub fn instance_id(&self) -> InstanceId;
    pub fn backup_mark(&self) -> Result<BackupMark, Error>;     // B0
}
pub struct OpenReport { pub state: InstanceState, pub recovered_stamp: Epoch, pub tickets_resumed: u32,
                        pub quarantined: Vec<QuarantineReason>, pub warnings: Vec<OpenWarning> }
```

`OpenReport.state ≠ Normal` heißt: nur die in §9 genannten Methoden laufen. `LockedKeys`, `RestoreQuarantine`, `ReadOnly` liefern für Schreib- und Lesemethoden den Fehler aus der Tabelle §9; Löschaufträge werden in `LockedKeys` und `Poisoned` als Intent aufgenommen (CYC-3 §9).

## 3. Spaces und Cluster

```rust
impl Storage { pub fn spaces(&self) -> Spaces<'_>; pub fn clusters(&self) -> Clusters<'_>; }
impl Spaces<'_> {
    pub fn create(&self, name: SpaceName, opts: SpaceOptions) -> Result<SpaceId, Error>;   // Matrix SYS-2 §2 vor E/A; B1 (Catalog-Edit)
    pub fn get(&self, name: &SpaceName) -> Result<Option<SpaceInfo>, Error>;                // B0
    pub fn list(&self) -> Vec<SpaceInfo>;                                                   // ohne Kernspaces, B0
}
impl Clusters<'_> {
    pub fn register(&self, spec: ClusterSpec) -> Result<ClusterId, Error>;                  // Entropy, Schlüssel zuerst, dann Edit; B1
    pub fn register_batch(&self, specs: Vec<ClusterSpec>) -> Result<Vec<ClusterId>, Error>; // ≤ 256, idempotent je BatchId
    pub fn state(&self, c: ClusterId) -> Result<ClusterState, Error>;                       // Live|Retired|Shredded|Purged|Unknown→Err
    pub fn link(&self, derived: ClusterId, sources: &[ClusterId]) -> Result<Receipt, Error>;           // Control-Frame, CYC-3 §7
    pub fn restrict(&self, c: ClusterId, r: RestrictionSpec) -> Result<RestrictionId, Error>;           // Control-Frame
    pub fn lift(&self, c: ClusterId, id: RestrictionId, auth: LiftAuth) -> Result<Receipt, Error>;      // CYC-3 §7
    pub fn restrictions(&self, c: ClusterId) -> Result<Vec<Restriction>, Error>;
    pub fn hold(&self, stamp: Epoch, ttl: Duration) -> Result<HoldId, Error>;  pub fn release(&self, h: HoldId);
}
pub struct ClusterSpec { pub protection: Protection /* Sealed|Plain */, pub tenant_tag: TenantTag,
    pub due: Option<UtcNanos>, pub erase_locality: EraseLocality /* PackedBounded{max_file_bytes} | ClusterLocal */,
    pub expected_bytes: u64, pub data_class: DataClass, pub batch: Option<BatchId> }
```

`ClusterSpec` ist unveränderlich nach `register`. `tenant_tag` ist 16 B opak, zufällig, geheim, nie aus einer Mandanten-ID abgeleitet (AT-10); die Fassade prüft nur Länge ≠ 0-Bytes-Tag (`InvalidInput`) und kann Zufälligkeit nicht prüfen (Verantwortung der Engine, SYS-5 §3).

## 4. Schreiben

```rust
pub struct Change { pub ops: Vec<Op>, pub guards: Vec<Guard> }
pub enum Op {
    Put { space: SpaceId, cluster: ClusterId, branch: BranchId, key: Bytes, header: Bytes, body: Bytes, weights: SmallVec<[u32; 4]> },
    Delete { space: SpaceId, cluster: ClusterId, branch: BranchId, key: Bytes },
    RangeDelete { space: SpaceId, cluster: ClusterId, branch: BranchId, start: Bytes, end: Bytes },   // end exklusiv
}
pub enum Guard {                                      // Ziel + Bedingung
    Key   { space: SpaceId, cluster: ClusterId, branch: BranchId, key: Bytes, cond: KeyCond },
    Range { space: SpaceId, cluster: ClusterId, branch: BranchId, start: Bytes, end: Bytes, since: Epoch },
}
pub enum KeyCond { Absent, Version(Epoch), Unchanged { since: Epoch } }
pub struct ApplyOptions { pub durability: Durability, pub idem: Option<IdemToken>, pub allow_mixed: bool, pub view_hint: Option<Epoch> }
impl Storage { pub fn apply(&self, change: Change, opts: ApplyOptions) -> Result<Receipt, Error>; }   // B1
pub struct Receipt { pub instance_id: InstanceId, pub stamp: Epoch, pub durable: bool, pub status: ApplyStatus /* Applied | AlreadyApplied */ }
```

Semantik:
1. Ein Change ist atomar über alle Ops (`INV-ATOM-1`), mit Ausnahme von Volatile-Zielen (`INV-VOL-1`: `allow_mixed` nötig, `durable` nur, wenn alle Ziele Durable sind).
2. `durability = Relaxed` ist verboten mit `idem` und für jeden Change, der Cluster eines Löschpfads berührt (`InvalidInput`). Der Nutzer liefert nie einen Stamp.
3. Guards werden nach CYC-1 §7 zweistufig ausgewertet. `view_hint` ist die View, gegen die Stufe A prüft; fehlt sie, pinnt `apply` selbst `V1`.
4. Reihenfolge der Ops im Change ist ohne Bedeutung außer bei gleichem Schlüssel: spätere Op überdeckt frühere (letzter Eintrag gewinnt, derselbe Stamp). `Put` und `Delete` desselben Schlüssels in einem Change sind erlaubt.
5. Leere Changes (`ops.is_empty()`) ⇒ `InvalidInput`. Ein Change ohne Ops, aber mit Guards, existiert nicht (ein Guard-Test ist ein Lesen).
6. `Receipt.durable = true` ⇔ `sync_data` der Gruppe war erfolgreich (immer außer `Relaxed`).

## 5. Lesen

```rust
impl Storage {
    pub fn view(&self) -> View;                           // visible_stamp, B0, kann nicht fehlschlagen
    pub fn view_durable(&self) -> View;                   // nur durable_stamp, B0
    pub fn view_at(&self, at: Epoch) -> Result<View, Error>;   // BelowFloor{floor}, B0
}
impl View {
    pub fn stamp(&self) -> Epoch;  pub fn instance_id(&self) -> InstanceId;
    pub fn on_branch(&self, b: BranchId) -> Result<View, Error>;
    pub fn get(&self, space: SpaceId, cluster: ClusterId, key: &[u8]) -> Result<Option<Entry>, Error>;       // B1 (Platte)
    pub fn get_many(&self, reqs: &[GetReq]) -> Vec<Result<Option<Entry>, Error>>;
    pub fn cursor(&self, space: SpaceId, cluster: ClusterId, range: KeyRange, dir: Direction) -> Result<Cursor, Error>;
    pub fn page(&self, space: SpaceId, cluster: ClusterId, range: KeyRange, dir: Direction, budget: PageBudget, resume: Option<ResumeToken>) -> Result<Page, Error>;
    pub fn filter_visible(&self, refs: &[RecordRef], check: VisCheck) -> Result<Vec<bool>, Error>;      // INV-VIS-1
    pub fn renew(&self) -> Result<(), Error>;             // bis view_lease_max
    pub fn release(self);                                 // idempotent; Drop ruft es
}
pub struct Entry { pub key: Bytes, pub stamp: Epoch, pub header: Bytes, pub weights: SmallVec<[u32; 4]>, /* body lazy */ }
impl Entry { pub fn body(&self) -> Result<Bytes, Error>; }          // Gate-Check vor und nach Entsiegeln
pub struct PageBudget { pub weight_slot: Option<u8>, pub max_weight: u64, pub max_bytes: u32, pub max_entries: u32 }   // max_bytes ≤ [max_page_bytes]
pub struct Page { pub entries: Vec<Entry>, pub resume: Option<ResumeToken>, pub weight_used: u64, pub skipped_blocks: u32 }
pub enum VisCheck { Cluster, DocHead /* CYC-7 §4 */ }
```

Semantik:
1. Gate-Check vor Entsiegeln, Reihenfolge der Fehler: `UnknownCluster` → `Erased{at}` (jede Sicht, auch ≥ S) → `Restricted` (`BLOCK_READ`) → `Revoked`/`SnapshotTooOld`. Details CYC-2 §2.
2. `get` und `cursor` lesen `header` und `weights` ohne Körper-I/O; `body()` lädt den Blob oder Datenblock und liefert `Erased`, `KeyUnavailable`, `KeyLost`, nie leere Daten.
3. `page`: ein Eintrag, der das Budget sprengt, beendet die Seite, außer sie wäre leer (Fortschrittsgarantie, CYC-2 §3). `ResumeToken` ist AEAD-versiegelt (Prozessschlüssel, nicht über Neustarts gültig) und nach Löschung `Err(Erased)`.
4. `view_at(at)` mit `at < history_floor` ⇒ `BelowFloor`; mit `at > visible` ⇒ `InvalidInput`.
5. Sichten halten Lease (`[view_lease]`); Ablauf ⇒ `SnapshotTooOld`. Eine `View` darf zwischen Threads wandern, ihr `Cursor` nicht gleichzeitig benutzt werden.
6. Keine clusterübergreifenden Scans (`INV-LAYER`-Linie des Kerns). Der Inspector (§8) iteriert Cluster.

## 6. Löschen

```rust
impl Storage { pub fn eraser(&self) -> Eraser<'_>;  pub fn space_drop(&self, req: SpaceDropRequest) -> Result<Ticket, Error>; }
pub struct EraseRequest { pub request_id: EraseRequestId, pub level: EraseLevel, pub purge_deadline: Duration,
                          pub view_policy: ViewPolicy /* Wait(Duration)|Revoke */, pub origin: EraseOrigin }
impl Eraser<'_> {
    pub fn cluster(&self, c: ClusterId, req: EraseRequest) -> Result<Ticket, Error>;           // B1 bis Gate (Ok ⇒ Gate durabel oder IntentRecorded)
    pub fn clusters_batch(&self, cs: &[ClusterId], req: EraseRequest) -> Result<Vec<Ticket>, Error>;   // zerlegt in Tickets ≤ [max_gate_clusters]
    pub fn range(&self, space: SpaceId, c: ClusterId, r: KeyRange, req: EraseRequest) -> Result<Ticket, Error>;
    pub fn branch(&self, c: ClusterId, b: BranchId, req: EraseRequest) -> Result<Ticket, Error>;
    pub fn pack_expired(&self, k: PackKey, req: EraseRequest) -> Result<PackExpiredReport, Error>;
    pub fn escalate(&self, t: Ticket, to: EraseLevel) -> Result<(), Error>;
    pub fn open_tickets(&self) -> Vec<TicketInfo>;                                             // B0, überlebt Aufruferabsturz
    pub fn ticket(&self, t: Ticket) -> Result<TicketStatus, Error>;
    pub fn wait(&self, t: Ticket, until: Stage, timeout: Duration) -> Result<Stage, Error>;    // B2
    pub fn receipt(&self, t: Ticket) -> Result<ErasureReceipt, Error>;                         // nur ab Done/Terminal, sonst NotReady
    pub fn verify(&self, t: Ticket) -> Result<VerifyReport, Error>;                            // strukturell, ohne Schlüssel
}
pub fn verify_receipt(r: &ErasureReceipt, ledger_slice: &LedgerSlice, anchor: &AnchorAttestation, mac: &dyn Mac) -> Verdict;   // offline, nur Mac-Port
```

Semantik: `Ok(Ticket)` heißt: der Intent ist durabel und der Kern ist verpflichtet. Bei Redo-Gate `stage ≥ Gated`; bei vergiftetem Redo `IntentRecorded` mit `ErasePending` für Leser (CYC-3 §9). `request_id` macht Retry idempotent (gleicher Scope ⇒ gleiches Ticket, abweichender Scope ⇒ `IdemConflict`). Zulässige Ziele je Protection und Operation: CYC-3 §3 (Matrix). `view_policy` gilt für das Warten auf Reader beim Unlink; die logische Garantie gilt ab Gate.

## 7. Branches, Checkpoint, Export, Feed

```rust
impl Storage { pub fn branches(&self) -> Branches<'_>; pub fn checkpoints(&self) -> Checkpoints<'_>; pub fn feed(&self) -> Feeds<'_>; }
impl Branches<'_> {
    pub fn fork(&self, c: ClusterId, parent: BranchId, at: ForkPoint /* Head|At(Epoch) */, o: BranchOptions) -> Result<BranchId, Error>;
    pub fn drop(&self, c: ClusterId, b: BranchId) -> Result<Receipt, Error>;                    // nur leaf
    pub fn diff(&self, a: &View, b: &View, c: ClusterId, range: KeyRange) -> Result<DiffIter, Error>;
    pub fn branch_diff(&self, c: ClusterId, b: BranchId) -> Result<DiffIter, Error>;
    pub fn merge(&self, c: ClusterId, source: BranchId, target: BranchId, mode: MergeMode, idem: IdemToken) -> Result<MergeOutcome, Error>;
    pub fn info(&self, c: ClusterId, b: BranchId) -> Result<BranchInfo, Error>;
}
impl Checkpoints<'_> {
    pub fn create(&self, name: &CheckpointName) -> Result<CheckpointInfo, Error>;               // Flush-Barriere, CYC-6 §6
    pub fn restore(cfg: RestoreConfig, ports: Ports) -> Result<(Storage, RestoreReport), Error>; // statisch, Quarantäne, CYC-6 §6
    pub fn list(&self) -> Vec<CheckpointInfo>;  pub fn drop(&self, name: &CheckpointName) -> Result<(), Error>;
}
impl Storage { pub fn export_cluster(&self, v: &View, c: ClusterId) -> Result<ExportStream, Error>; }   // Strom (branch,key,header,body), Gate vor jedem Block
pub trait ConsumerFeed { fn poll(&mut self, max: usize, wait: Duration) -> Result<Vec<FeedItem>, Error>; fn ack(&mut self, pos: Epoch) -> Result<(), Error>; }
impl Feeds<'_> { pub fn open(&self, id: ConsumerId, from: Option<Epoch>) -> Result<Box<dyn ConsumerFeed>, Error>; }   // CYC-7 §4
```

## 8. Inspector

`Storage::inspector(cap: InspectCap) -> Inspector`. Liefert aggregierte oder opake Werte (Zählungen, Größen, Zustandsverteilungen, `Inspector::clusters()` als Iterator opaker Handles), entsiegelt nie, nennt nie Klartext-Cluster-IDs in Fehlertexten, ist bei `Strict` nur mit `InspectCap::Diagnostic` zulässig. Eine Capability ist ein nicht klonbarer Token, den nur `engine` ausstellt.

## 9. Fehlerzuordnung (normativ, `facade_error_matrix`)

| Methode | Fehler (zusätzlich zu `Poisoned`, `InvalidInput`, `Limit`) |
|---|---|
| `apply` | `Conflict`, `Backpressure`, `DiskFull`, `ClusterRetired`, `UnknownCluster`, `Restricted{BLOCK_WRITE}`, `BranchClosed`, `ReservedSpace`, `ChangeTooLarge`, `IdemConflict`, `IdemCapacity`, `Indeterminate`, `KeyUnavailable`, `ReadOnly`, `LockedKeys` |
| `view_at`, `get`, `page`, `body` | `BelowFloor`, `SnapshotTooOld`, `Revoked`, `Erased`, `UnknownCluster`, `Restricted{BLOCK_READ}`, `ErasePending`, `KeyUnavailable`, `KeyLost`, `Corrupt` |
| `eraser().*` | `Restricted{BLOCK_ERASE}`, `Unsupported` (Level), `EraseFanoutExceeded`, `DeadlineBeforeKeyDestroy`, `ProviderAssuranceInsufficient`, `LedgerUnavailable`, `LedgerSameDevice`, `AnchorUnavailable`, `IdemConflict`, `UnknownCluster` |
| `clusters().register` | `Limit{ClusterMetaBudget|PackGroups|ClusterLocalAdmission}`, `KeyUnavailable`, `Exhausted` |
| `branches().*` | `BranchClosed`, `BranchHasChildren`, `InvalidForkPoint`, `BelowFloor`, `MergeTooLarge`, `Conflicts` (als Wert, nicht Fehler), `Exhausted` |
| `Checkpoints::restore` | `BackupBeyondHorizon`, `RestoreQuarantine`, `Rollback`, `KeyLost`, `LedgerUnavailable` |
| `open` | `Rollback`, `LockedKeys`, `Corrupt`, `Unsupported`, `MigrationNeedsCleanClose`, `FdLimitTooLow`, `ProfileViolation`, `LedgerUnavailable`, `LedgerSameDevice`, `AnchorUnavailable` |

**Zustandsabhängigkeit.** `ReadOnly` verweigert alle Mutationen (`ReadOnly{reason}`), erlaubt Lesen und `export_cluster`. `LockedKeys`/`RestoreQuarantine`: kein Lesen (`KeyUnavailable`/`RestoreQuarantine`), nur `eraser()`-Aufnahme und Diagnose. `Poisoned`: kein Schreiben, Lesen bleibt für nicht betroffene Dateien möglich; Löschaufnahme per Intent. `DiskFull`: kein `apply` (`DiskFull`), alles andere läuft.

## 10. Entscheidungen und geschlossene Lücken

DEC-43 (keine Standard-Ports), DEC-46 (Catalog-Spiegel), DEC-50 (`Storage::create` getrennt von `open`), DEC-51 (leerer Change ist `InvalidInput`), DEC-55 (Fassade synchron, Blockierklassen B0–B2); GAP-19, GAP-20 (SYS-7 §3).

## 11. Abnahme

`facade_error_matrix` (jede Zeile §9 positiv/negativ), `facade_state_matrix` (Zustände × Methoden), `apply_empty_rejected`, `relaxed_with_token_rejected`, `view_error_order`, `page_progress_guarantee`, `no_callbacks_under_lock`, `ports_required` (DEC-43). Gates: G-W für §4, G-R für §5, G-E für §6, G-V für `branches`.

**Offene Punkte:** keine.
