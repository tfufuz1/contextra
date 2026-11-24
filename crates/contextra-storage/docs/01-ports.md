# SP-PORTS — `contextra-storage-ports` (Verträge, Werttypen, Fehler)

| | |
|---|---|
| Crate | `crates/contextra-storage-ports` (keine Logik, `#![forbid(unsafe_code)]`) |
| Besitzt | alle Werttypen und Traits, die ≥ 2 Komponenten teilen; das `Storage`-Fassadenprofil (Signaturen, nicht Rümpfe) |
| Konsumiert | nichts (nur `bytes`, `zeroize`, `thiserror`) |
| Spec-Bezug | R9.1 §3.2, §3.5, §4, §6, §9.4, §12, §15–16 |
| Phase | P0, Gate G-0 (API-Review-Freeze) |

## 1 Auftrag und Abgrenzung
Einzige Quelle für Typen und Traits. Jede andere Komponente importiert von hier und definiert **keine** konkurrierenden Typen.
**Darf nicht:** Logik, Serialisierung (→ SP-CODEC), I/O, Abhängigkeit auf `contextra-ports` (Altstand). Der Altstand-Crate `contextra-ports` wird **nicht** re-exportiert; er bleibt bis zum Cutover unverändert (siehe Master G-03).

## 2 Öffentliche Schnittstelle
```rust
// Identität (Debug redigiert, kein Display)
pub struct InstanceId([u8; 16]); pub struct ClusterId([u8; 16]);  pub struct SpaceId(pub u64);  pub struct BranchId(pub u32);
pub struct BatchId([u8; 16]);
pub struct EraseRequestId([u8; 16]); // durable idempotency for erase tickets; separate from 24h write IdemToken
pub type Stamp = u64;            pub const STAMP_MAX: Stamp = (1 << 56) - 1;
// Stamp wird ausschließlich zusammen mit InstanceId interpretiert und ist instanzlokal.
pub struct DurablePosition { pub instance: InstanceId, pub stamp: Stamp }
pub struct TombId([u8; 32]);     pub struct IdemToken([u8; 16]);   pub struct Ticket([u8; 16]);

pub enum Protection { Sealed, Plain }
pub enum Durability { Durable, Relaxed }
pub enum Origin { User, Maintenance, Replication, Migration }       // NEU (E-04), 1 Byte im Frame
pub struct CallerMeta { pub origin: Origin, pub caller_seq: u64 }    // aufruferlokale Sequenz, nie als globale TxId interpretieren

pub struct Change { pub ops: Vec<Op>, pub guards: Vec<Guard>, pub idem: Option<IdemToken>,
                    pub durability: Durability, pub allow_mixed: bool, pub caller: CallerMeta }
pub enum Op { Put { space: SpaceId, cluster: ClusterId, branch: BranchId, key: Bytes, header: Bytes,
                    body: Bytes, weights: [u32; 4] },
              Delete { .. }, RangeDelete { .. /* start,end exklusiv */ } }
pub enum Guard {
    Key   { space: SpaceId, cluster: ClusterId, branch: BranchId, key: Bytes, pre: Precondition },
    Range { space: SpaceId, cluster: ClusterId, branch: BranchId, start: Bytes, end: Bytes, since: Stamp }, // NEU (E-05)
}
pub enum Precondition { Absent, Version(Stamp), Unchanged { since: Stamp } }
pub struct Receipt { pub instance: InstanceId, pub stamp: Stamp, pub durable: bool }
pub enum KeyCreateResult { Created { cluster: ClusterId, epoch: u32 }, Failed { cluster: ClusterId, error: KeyError } }
```
**Traits** (Signaturen in R8 §3.2/§9.4/§12.1; Abweichungen/Ergänzungen hier):
- `Fs`/`File`: `read_at`, `read_ranges`, `append_prealloc`, `preallocate` (**Nullen schreiben**, nie sparse), `sync_data/sync_all/sync_dir`, `rename`, `lock`, `list`, `free_space`, `caps() -> FsCaps`, optionale `link/reflink/punch_hole`. `read_ranges` hat eine geordnete Default-Schleife über `read_at`; Optimierungen ändern weder Semantik noch Ergebnisreihenfolge. Jede Operation trägt einen `IoCtx` (Datei-Rolle, Zweck) für `FaultFs`-Indizierung.
- `KeyProvider`: R8 §9.4 **plus** `create_cluster_keys(&self, request_id: BatchId, clusters: &[ClusterId], entropy: &dyn Entropy) -> Result<Vec<KeyCreateResult>, KeyError>` (idempotenter Batch, max. 256 IDs; pro Cluster expliziter Erfolg/Epoch oder Fehler; jeder Erfolg ist vor Rückgabe durable). Der Batch ist kein All-or-None-Transaction-Vertrag; Teil-Erfolge werden retrybar und verwaiste Schlüssel durch `orphan_key_grace` bereinigt (E-02).
- `Anchor`, `Clock`, `Entropy`, `Cipher`, `Mac`, `Kdf`, `Checksum`, `Authority`, `Metrics` (statischer Metrikkatalog + `MetricSample::{Counter,Gauge,Observe}` mit endlichen Labels), `Observer`, `ColdTier`, `Spawner`.
- `Memtable` (Skiplist hinter Trait), `SegmentOwner`, `ExtensionBuilder`, `ConsumerFeed` (Verträge §12).
- `MemoryBudget`: gemeinsamer, thread-sicherer Admission-Port; `try_reserve(class, bytes) -> ResourcePermit`, dessen `Drop` Budget exakt freigibt. Klassen sind `Memtable`, `ClusterMeta`, `Guard`, `BlockCache`, `KeyCache`, `ExtensionDelta`, `Inflight`, `PurgeScratch`; kein Besitzer darf vor erfolgreicher Reservation allozieren.
- `Error` (Klassen F-1…F-9) mit `hint() -> &'static str` und `class() -> ErrorClass`; Texte ohne Schlüssel/Nutzdaten (Test mit Canaries).

**Fassadenprofil** `Storage` (Rümpfe verteilt per `impl Storage` auf Komponenten; Besitzer in Klammern):
`open/close` (SP-CATALOG) · `apply` (SP-COMMIT) · `view/view_at/view_durable` (SP-READ) · `clusters()`, `spaces()`, `fork/branch_drop/merge/diff` (SP-CLUSTER) · `eraser()` (SP-ERASE) · `consumers()`, `segments()` (SP-EXT) · `checkpoint()/restore()` (SP-CKPT) · `inspector()` (SP-VERIFY, nur `testing`).

## 3 Regeln
1. Jede Änderung an einem Trait nach G-0 braucht Review durch SP-PORTS-Owner und alle Konsumenten (RFC im PR).
2. Typen sind `#[non_exhaustive]` nur dort, wo R8 Erweiterung vorsieht (Op-Kinds, Fehler); Wire-Formate hängen nie an `repr` dieser Typen (→ SP-CODEC).
3. `Debug` für `ClusterId`, `Key32`, Körper ist `Redacted<T>`.
4. `SpaceId` ist `u64`, wird ausschließlich aus dem durable reservierten Allocator ausgegeben; öffentliche Storage-Pfade weisen IDs `< 256` mit `ReservedSpace` ab. Numerische IDs sind keine Autorisierung.
5. Zeitquellen: kein `SystemTime`/`Instant` in Signaturen; nur `Clock`.

## 4 Fehlerverhalten
Fehlerklassen und Reaktion exakt nach R8 §3.5. `Error::Poisoned` ist klebrig und trägt Ursache (`PoisonCause`) ohne Inhalte.

## 5 Invarianten / Tests
`INV-CLUSTER-2` (kein Konstruktor aus Kennung), `INV-LIM-1` (Limit-Konstanten), Redaktions-Test (Canary in Debug/Display/Error → 0 Treffer), Compile-fail-Doctests: `ClusterId` nicht aus `&[u8]` ableitbar außer `from_wire` (nur `pub(crate)`-Pfad via SP-CODEC-Feature `wire`).

## 6 Lücken / Errata (Beschlüsse)
- **E-02** Batch-Schlüsselanlage ist idempotent und max. 256 IDs pro Request. Pro Schlüssel ist Erfolg vor Rückgabe durable; Teil-Erfolge sind erlaubt, müssen im Ergebnis einzeln stehen und per `BatchId` wiederholbar sein. Cluster werden erst nach erfolgreichem Key-Ergebnis registriert; verwaiste Schlüssel sind recoverbar und nach Grace bereinigbar. Kein atomarer KMS-Multi-Key-Vertrag wird vorausgesetzt.
- **E-04** `CallerMeta` ist v1 optionales Audit-Metadatum: `origin u8` plus aufruferlokale `caller_seq u64`, nie TxId/Personenkennung. Es beeinflusst weder Idempotenz noch Commitsemantik. Altstand-TxIds werden nur in der Bridge übersetzt und nicht roh persistiert.
- **E-05** `Guard::Range` (Phantom-Schutz für `scan_prefix_tracked`).
- Zwei `Clock`-Traits im Altstand (`clock.rs`, `persistence.rs`): hier genau einer.

## 7 Arbeitspakete
| ID | Titel | PW | Abhängig von | Phase | Abnahme |
|---|---|---|---|---|---|
| WP-01.1 | Werttypen, IDs, `Redacted`, `Change/Op/Guard`, `Error` + `hint()` | 2.0 | — | P0 | Redaktions-/Compile-fail-Tests grün |
| WP-01.2 | Port-Traits (Fs, Clock, Entropy, Anchor, KeyProvider inkl. Batch, Cipher/Mac/Kdf/Checksum, Authority, Metrics, Observer, ColdTier, Spawner) | 2.0 | WP-01.1 | P0 | Mock-Impl je Trait kompiliert; Layering-Test |
| WP-01.3 | Erweiterungstraits + `Storage`-Fassadenprofil + API-Review-Freeze | 1.5 | WP-01.2 | P0 | zwei Reviewer, G-0 |

## 8 Abnahme
Crate baut ohne Abhängigkeit auf Altstand-Crates; `cargo doc` vollständig; `layering`-Test grün; jede in R8 §3.2 genannte Port-Fähigkeit hat genau einen Trait.

## R8.2-Nachtrag

Besitzer der geteilten R8.2-Typen (E-74, E-76…E-81; Spec R8.2 §3.2, §3.5, §9.4, §12.2, §13.2.1):

- **`KeyProvider`** wird an SPEC §9.4 angeglichen: `info() -> KeyProviderInfo { destroy: DestroyMode, keys_in_backups: KeysInBackups, assurance: KeyAssurance }`, `kek`, `create_cluster_keys(request_id, clusters, entropy)`, `cluster_keys`, `destroy_many`, `probe(cluster, epoch)`, `list`, neu `create_tomb_epoch`, `tomb_key`, `retire_tomb_epoch`. `KeysInBackups::{No, Bounded{until: Duration}, Yes}`, `KeyAssurance::{Hardware, Kms, FilePassphrase}`. Der Code in `contextra-storage-ports/src/keys.rs` führt bisher nur die vier Batch-Methoden; `info`, `kek`, `list` und die Tomb-Methoden sind vor G-0 nachzuziehen.
- **`HostProbe`** (optional): `fn probe(&self) -> HostReport` (Swap, Core-Dumps, `RLIMIT_MEMLOCK`, Dateisystemtyp, Geräte-IDs; `Unknown` ist ein gültiger Wert). Nur Adapter, kein Aufruf aus dem Kern, wo Korrektheit davon abhängt.
- **`ExtensionBuilder::block_boundary(block_uid, entries) -> Option<AggBytes>`** und **`BlockPruner::may_skip(block_uid, agg) -> bool`** samt `PruneContract::{None, ShadowSafe{reason}}` (K-29): Typen werden von RUN und EXT geteilt und stehen deshalb hier.
- **Fehlerklassen** (F-7) ergänzt: `SnapshotTooOld`, `ErasePending`, `LedgerUnavailable`, `LedgerSameDevice`, `BackupBeyondHorizon`, `ProviderAssuranceInsufficient`; jede mit stabilem `hint()` und Fehlertext ohne Schlüssel/Körper.
- **`MeasureKind { Complete, Sample{n, N}, Declared, NotMeasured, NotImplemented }`**, Receipt-Statuscaps `ResidualKind::{BoundedBackupKeys{until}, WeakKeyAssurance, HostExposure{..}, LedgerSameDevice}`.
- `BackupMark { instance_id, erase_epoch, created_at, tomb_epoch }` als Typ; `Storage::erase_epoch() -> u64` und `Storage::backup_mark()` sind Fassade (SP-FACADE), Typen hier.

| ID | Titel | PW | Phase | Abnahme |
|---|---|---|---|---|
| WP-01.5 | R8.2-Typen (`KeyProvider`-Angleichung, `HostProbe`, `BlockPruner`/`PruneContract`, `MeasureKind`, `BackupMark`, neue Fehlerklassen) | 1.0 | WP-01.2 | P0 (vor G-0) | Mock-Impl je Trait, Layering-Test, `trybuild`-Negativtests, Konformanz-Skelett `KeyProviderConformance` für Tomb-Methoden |

PW = Personenwochen, grobe Schätzung ohne Zusage (wie SPEC §14).

## R9.1-Synchronisierung

- `BranchId`-Vergabe ist monoton, clusterlokal und endet bei `u32::MAX` mit `Exhausted`; Default-Tiefe ist 4, harte Grenze 8. `Eraser::branch` gehört in `impl Eraser`, nicht in `impl Storage`.
- `KeyProvider::list` liefert `KeyListEntry { cluster, created_utc }`, damit `orphan_key_grace` berechenbar ist. `ClusterSpec` enthält vollständig `protection`, `tenant_tag`, `due` und die Admission-relevanten Felder aus SPEC §4.3.
- Die gemeinsamen Fehler-/Statusverträge unterscheiden `Erased`, `KeyUnavailable`, `KeyLost` und Clusterzustand `Shredded`; `Shredded` ist kein lesbarer Fehler. `Disabled` bestätigt keine Schlüsselvernichtung und erlaubt keinen Cryptographic-Purge.
- Ergänzt werden Pipeline-/Admission-Typen: Intake-Credits, `policy_epoch`, `GuardRetry`, `ClusterMetaBudget`, `ClusterRetired`, `EraseFanoutExceeded` und getrennte Resource-Klassen für Cluster-Metadaten, FD-/Datei-Admission und Purge-Scratch. Alle müssen vor der jeweiligen Allokation/Mutation prüfbar sein.
