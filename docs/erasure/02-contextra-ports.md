# Implementierungsspezifikation `contextra-ports` (Löschkette)

*Ring 0 · Konzept-WP 2 · Abhängig von: 01 · Blockiert: 03, 06–10 · Index: `00-INDEX-QUERSCHNITT.md`*

## 1. Zweck

Verträge zwischen Kernel (Store), Treiber (Engine) und Layern (Text, Vektor, Graph, Checkpoint). Ersetzt die Platzhalter ohne Implementierung durch einen tragfähigen, fail-closed Vertrag. Keine Logik außer reinen Wertetypen.

## 2. Ist-Zustand (✔ gelesen)

| Datei | Befund |
| --- | --- |
| `gate.rs` | `Seq(pub u64)` (Duplikat zu `types`), `KeyBuf([u8;16])`, `KeyBound`, `KeyRange`, `GateDecision{Open,Erased,Restricted}`, `RangeGate{ContainsErased,RestrictedNeverDiscard,EntirelyOpen}`. `ReadGate` hat **Default-Methoden mit fail-open**: `gate_epoch→0`, `check_raw→Open` (erwartet `&[u8;78]`), `gated_seq_raw→None`, `covers*` |
| `erasure.rs` | `LayerId([u8;16])`, `EraseTarget{KeyRange,ExactKeys}` (Byte-Bereiche), `LayerScope`, `PurgeCx{deadline_ms}`, `PurgeOutcome`, `Scanner<State>`/`Armed` (leere Marker), `ErasureLayer` mit `id/resolve/on_gate…`, Fehler als `String` |
| `barrier.rs` | `CommitBarrier`, `PurgeBarrier`, `BarrierShared/Exclusive`, `CompactionPermit`, `PurgePermit`: nur `Arc<T>`-Weiterleiter, keine Implementierung |
| `keys.rs` | `FileId(u64)`, `FileKind`, `FileKey`, `FileKeyring`, `RecordSealer`, `Envelope`, `SealCtx` ohne Implementierung |
| `ledger.rs` | `Timestamp(pub u64)` (Duplikat), `RangeSet` aus `KeyBuf`-Paaren, `LedgerEntry{commitment, range_set, gated_seq, completed_at, chain_hash, ledger_mac}`, `LedgerSink{append, read_all, prune}` mit `String`-Fehlern |
| `observer.rs` | `TargetKind` (6 Varianten), `ObservedPhase{Gated,Completed,Restricted,Lifted}`, `ErasureEvent`, `ErasureObserver::on_erased`, `ObserverError`. **Kein Implementierer** im Workspace |
| `scanner/*` | `LayerScanner{layer, scan_units, scan(..)}`, `ScanSession` (`begin`/`finish` nur `pub(crate)`), `LayerWitness` (Mint-Schutz, INV-WITNESS-MINT-1), `WitnessLayer{LsmMemtable,SsTableAllLevels,HnswIndex,WalAllSegments,CsrGraph,KvCacheSegments,EmbeddingCache,Custom}`, `Purger` (synchron, `ScanTarget{target_id:String,target_hash}`), `PurgeStats` |
| `lib.rs:185-204` | Re-Exporte aller genannten Typen |

## 3. Änderungen je Datei

| Datei | Aktion |
| --- | --- |
| `gate.rs` | **Neu geschrieben** (§4.1). `KeyBuf`, `KeyBound`, `KeyRange`, Duplikat-`Seq` entfallen |
| `erasure.rs` | **Ersetzt** (§4.2–4.6) |
| `barrier.rs`, `keys.rs` | **Löschen** (außer `RecordSealer`/`SealCtx`/`Envelope`: nach `seal.rs` verschieben und für Stufe 2 überarbeiten, WP-11; bis dahin hinter `cfg(feature = "stage2")`, Default aus) |
| `ledger.rs` | **Neu** (§4.7) |
| `observer.rs` | Beibehalten; `TargetKind`/`Timestamp` aus `types` (01 §4.1); Einsatz in §4.8 festgelegt |
| `scanner/*` | **Unverändert** außer: `Purger` und `ScanTarget` als `#[deprecated]` markieren, nach WP-6…8 entfernen. `ScanSession::begin/finish` bleiben `pub(crate)` |
| `lib.rs` | Exporte anpassen |
| `tests/` | Compile-Fail-Tests (§7) |

## 4. Verträge (normativ)

### 4.1 `ReadGate` v2 (`gate.rs`)

```rust
pub use contextra_types::Seq;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GateDecision { Open, Restricted, Erased }
impl GateDecision { pub const fn is_blocked(self) -> bool { !matches!(self, Self::Open) } }

/// Ergebnis für einen Präfix-/Bereichsscan.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RangeGate { EntirelyOpen, Partial, EntirelyBlocked }

/// Verwerfungsregel für Compaction/Flush (ersetzt `RangeGate::ContainsErased/RestrictedNeverDiscard`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DiscardRule { Keep, MayDrop }

/// Synchron, ohne I/O, ohne Panik, ohne Allokation im Open-Fall.
/// KEINE Default-Methoden: jeder Implementierer entscheidet jede Methode (fail-closed, G-6).
pub trait ReadGate: Send + Sync {
    /// `false` ⇒ alle Prüfungen liefern Open. Ein Relaxed-Load.
    fn is_active(&self) -> bool;
    /// Streng monoton; Acquire-Load.
    fn epoch(&self) -> u64;
    /// Rohschlüssel exakt so, wie `LsmStorage` ihn sieht (`t:{tenant}:` + Collection-Schlüssel).
    fn check(&self, raw_key: &[u8]) -> GateDecision;
    /// Version-bewusst: Erased{up_to} sperrt nur `version_seq <= up_to`.
    fn check_version(&self, raw_key: &[u8], version_seq: Seq) -> GateDecision;
    /// Für Scans: liegt der gesamte Präfixraum frei/gesperrt?
    fn check_prefix(&self, prefix: &[u8]) -> RangeGate;
    /// Für Compaction/Flush: darf diese Version verworfen werden?
    fn discard_rule(&self, raw_key: &[u8], version_seq: Seq) -> DiscardRule;
    /// Dokumentsicht der Indizes. `collection` ohne Präfix (`"default"`, `"docs"`).
    fn check_doc(&self, tenant: contextra_types::TenantId, collection: &str, doc: contextra_types::DocId) -> GateDecision;
    /// Sequenz, ab der der Schlüssel gesperrt ist (für Write-Abweisung und Diagnose).
    fn gated_seq(&self, raw_key: &[u8]) -> Option<Seq>;
}
```

**Semantik.** `Restricted` ⇒ nicht lesbar, `discard_rule = Keep`. `Erased` ⇒ nicht lesbar, `MayDrop` (nur für Versionen im Scope des Auftrags). Nicht ladbarer Zustand: Implementierung liefert für **alles** `Restricted` (I-7). Hilfs-Implementierung `AlwaysOpenGate` und `AlwaysRestrictedGate` im Crate `contextra-testkit`, **nicht** in `ports`.

### 4.2 Scope-Typen (`erasure.rs`)

```rust
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum KeyPattern { Prefix(Vec<u8>), Exact(Vec<u8>) }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verdict { In, Out }

/// Longest-Match-Wins; Exact schlägt Prefix; kein Treffer = nicht im Scope (Index 3.1).
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct KeyScope { rules: Vec<(KeyPattern, Verdict)> }
impl KeyScope {
    pub const MAX_RULES: usize = 4096;
    pub const MAX_BYTES: usize = 1 << 20;                    // Summe der Musterbytes
    pub fn new(rules: Vec<(KeyPattern, Verdict)>) -> Result<Self, ErasureError>; // sortiert, dedupliziert, prüft Limits
    pub fn contains(&self, raw_key: &[u8]) -> bool;
    pub fn rules(&self) -> &[(KeyPattern, Verdict)];
    pub fn could_overlap_range(&self, min: &[u8], max: &[u8]) -> bool; // Hülle: SSTable-Min/Max
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DocScope {
    Set { tenant: TenantId, collection: String, docs: BTreeSet<DocId> },
    Collection { tenant: TenantId, collection: String },
    Tenant { tenant: TenantId },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ResolvedScope {
    pub id: ErasureId,
    pub target_kind: TargetKind,
    pub keys: KeyScope,
    pub docs: Vec<DocScope>,
    pub plan_hash: [u8; 32],         // BLAKE3 über kanonische Serialisierung (§5)
    pub unprovenanced: u64,          // Altbestand ohne derived_from (08)
}
// Debug von ResolvedScope und Mustern: nur Zählungen, nie Bytes.
```

`could_overlap_range` ist **konservativ** (Falsch-Positive erlaubt, Falsch-Negative verboten): liefert `true`, wenn ein `In`-Muster `p` existiert mit `p`-Präfixraum ∩ `[min,max] ≠ ∅` oder `Exact` in `[min,max]`.

### 4.3 Budget, Fortschritt, Kontext

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IoClass { Interactive, Erase, Compaction }

pub trait IoBudget: Send + Sync {
    /// Wartet kooperativ (Clock-basiert), bis `bytes` zugeteilt sind. Cancel-safe.
    fn acquire<'a>(&'a self, class: IoClass, bytes: u64) -> BoxFuture<'a, Result<(), ErasureError>>;
    /// Druckkopplung: Critical ⇒ Erase/Compaction pausieren, Interactive nie.
    fn pressure(&self) -> PressureHint;
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PressureHint { Normal, Elevated, Critical }

/// Durables Fortschrittsprotokoll (Write-Ahead). Implementiert vom Kernel.
pub trait ProgressSink: Send + Sync {
    fn record<'a>(&'a self, layer: &'a WitnessLayer, p: &'a LayerProgress) -> BoxFuture<'a, Result<(), ErasureError>>;
}

pub struct PurgeCx<'a> {
    pub erasure_id: ErasureId,
    pub clock: &'a dyn Clock,
    pub cancel: &'a CancellationToken,         // tokio_util (bereits Workspace-Abhängigkeit, **[?]** in `ports/Cargo.toml` prüfen)
    pub budget: &'a dyn IoBudget,
    pub cursor: Option<Cursor>,
    pub reader: &'a dyn PrivilegedRead,
    pub progress: &'a dyn ProgressSink,
    pub max_units: usize,                      // Obergrenze je Aufruf (Tick)
}
```

Hängt `ports` heute nicht an `tokio_util`, wird `CancellationToken` durch einen eigenen `Cancel`-Trait (`fn is_cancelled(&self)->bool; fn cancelled(&self)->BoxFuture<'_,()>`) ersetzt, damit keine Kante entsteht. Entscheidung in WP-0 nach Blick in `ports/Cargo.toml`.

### 4.4 Privilegierter Lesezugriff

```rust
/// Umgeht das Gate bewusst, ausschließlich für Purger des laufenden Auftrags.
pub trait PrivilegedRead: Send + Sync {
    fn get<'a>(&'a self, raw_key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>, ErasureError>>;
    /// Seitenweise (≤ 1024 Schlüssel), Cursor undurchsichtig.
    fn scan_prefix<'a>(&'a self, prefix: &'a [u8], after: Option<&'a [u8]>, limit: usize)
        -> BoxFuture<'a, Result<Vec<(Bytes, Bytes)>, ErasureError>>;
}
```

Der Kernel prüft bei **jedem** Aufruf, dass der Schlüssel `ResolvedScope.keys.contains` erfüllt, sonst `ErasureError::Layer{kind: ScopeViolation}` (I-8). Nach `Sealed` liefert jeder Aufruf `Err`.

### 4.5 `ErasureLayer`

```rust
pub struct PurgeOutcome {
    pub progress: LayerProgress,        // cursor=None ⇒ fertig
    pub units_committed: u64,           // NUR durch Commit belegte Arbeit
    pub bytes_freed: u64,
}
pub struct LayerError { pub kind: LayerErrorKind, pub layer: WitnessLayer, pub detail: &'static str }

pub trait ErasureLayer: Send + Sync {
    fn layer(&self) -> WitnessLayer;
    /// RAM-Maskierung und Cache-Räumung. Idempotent, schnell (< 10 ms), keine Datei-I/O.
    fn on_gate<'a>(&'a self, s: &'a ResolvedScope) -> BoxFuture<'a, Result<(), LayerError>>;
    /// Wiederaufnehmbar über `cx.cursor`. Teilfehler ⇒ Err. Kein block_on.
    fn purge<'a>(&'a self, s: &'a ResolvedScope, cx: &'a PurgeCx<'a>) -> BoxFuture<'a, Result<PurgeOutcome, LayerError>>;
    fn scanner(&self) -> &dyn LayerScanner;
}
```

Vertragsregeln (Konformanzsuite C1–C7 in 13): idempotent; Wiederaufnahme an jedem Cursor; berührt nur `ResolvedScope`; Witness-Ehrlichkeit; Fehlerinjektion ⇒ `Err`; cancel-safe; respektiert Gate (liest nie am Gate vorbei außer über `cx.reader`).

### 4.6 `ErasureKernel` (Port zum Store)

Die Methoden aus Konzept 4.3 mit konkreten Typen:

```rust
pub struct EraseRequest { pub id: ErasureId, pub target: EraseTarget, pub reason: ErasureReason, pub policy: ErasurePolicy }
pub struct AcceptReceipt { pub id: ErasureId, pub existing: bool }
pub struct GateReceipt   { pub id: ErasureId, pub gated_seq: Seq, pub epoch: u64 }
pub struct SealReceipt   { pub id: ErasureId, pub guarantee: Guarantee, pub exceptions: Vec<ErasureException>, pub ledger_index: Option<u64> }
pub struct JobView       { pub id: ErasureId, pub state: ErasureState, pub scope: Arc<ResolvedScope>, pub layers_done: Vec<WitnessLayer>, pub attempt: u32 }

pub trait ErasureKernel: Send + Sync {
    fn accept<'a>(&'a self, req: &'a EraseRequest, plan: &'a ResolvedScope) -> BoxFuture<'a, Result<AcceptReceipt, ErasureError>>;
    fn publish_gate<'a>(&'a self, id: ErasureId) -> BoxFuture<'a, Result<GateReceipt, ErasureError>>;
    fn purge_store<'a>(&'a self, id: ErasureId, cx: &'a PurgeCx<'a>) -> BoxFuture<'a, Result<PurgeOutcome, ErasureError>>;
    fn record_layer<'a>(&'a self, id: ErasureId, layer: &'a WitnessLayer, p: &'a LayerProgress) -> BoxFuture<'a, Result<(), ErasureError>>;
    fn seal<'a>(&'a self, id: ErasureId, w: &'a [LayerWitness]) -> BoxFuture<'a, Result<SealReceipt, ErasureError>>;
    fn complete<'a>(&'a self, id: ErasureId) -> BoxFuture<'a, Result<(), ErasureError>>;   // Gate-Lebensende (I-9)
    fn fail<'a>(&'a self, id: ErasureId, layer: Option<&'a WitnessLayer>, kind: LayerErrorKind) -> BoxFuture<'a, Result<ErasureState, ErasureError>>;
    fn open_jobs(&self) -> Vec<JobView>;
    fn status(&self, id: ErasureId) -> Option<ErasureStatus>;
    fn privileged_reader(&self, id: ErasureId) -> Result<Arc<dyn PrivilegedRead>, ErasureError>;
    fn gate(&self) -> Arc<dyn ReadGate>;
    fn store_scanners(&self) -> Vec<Arc<dyn LayerScanner>>;   // Memtable, SSTables, WAL
}
```

Zusicherungen: `accept` ist idempotent über `(id, plan_hash)`; abweichender Hash ⇒ `IdempotencyConflict`. `publish_gate` kehrt erst nach durablem `Gated`-Eintrag und Veröffentlichung zurück. `seal` ist nur zulässig, wenn **alle** erwarteten Schichten (`JobView.layers` aus dem Plan) einen Witness liefern; fehlt einer, wird er als `NotMeasured` ausgewiesen und die Garantie sinkt (I-6). `complete` ist nur nach `Sealed` und nur bei `remaining == 0` über alle Witnesses zulässig; sonst `InvalidTransition`.

### 4.7 `LedgerSink` v2 (`ledger.rs`)

```rust
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LedgerEntry {
    pub index: u64,
    pub erasure_id_hmac: [u8; 32],
    pub guarantee: Guarantee,
    pub layers_root: [u8; 32],       // BLAKE3 über Schicht-Digests in fester Reihenfolge
    pub exceptions: Vec<ErasureException>,
    pub plan_hash: [u8; 32],
    pub policy_version: u32,
    pub completed_at: Timestamp,
    pub prev_hash: [u8; 32],
    pub entry_mac: [u8; 32],         // HMAC über alle Felder + prev_hash
}
pub trait LedgerSink: Send + Sync {
    fn append<'a>(&'a self, e: LedgerDraft) -> BoxFuture<'a, Result<LedgerEntry, ErasureError>>; // idempotent über (erasure_id_hmac, plan_hash)
    fn head<'a>(&'a self) -> BoxFuture<'a, Result<LedgerHead, ErasureError>>;
    fn verify<'a>(&'a self) -> BoxFuture<'a, Result<LedgerVerification, ErasureError>>;
}
pub struct LedgerHead { pub index: u64, pub hash: [u8; 32] }
pub trait LedgerAnchor: Send + Sync { fn publish<'a>(&'a self, h: &'a LedgerHead) -> BoxFuture<'a, Result<(), ErasureError>>; }
```

`prune` entfällt (Ledger ist append-only). `read_all` entfällt (Streaming in 03 §11). `LedgerDraft` enthält alle Felder außer `index/prev_hash/entry_mac`, die der Sink setzt.

### 4.8 `ErasureObserver` (Einsatz festgelegt)

Der vorhandene Port bleibt und wird zum **Benachrichtigungsweg für RAM-Caches** (Embedding-, KV-, Query-Cache in der Engine). Der Kernel ruft ihn mit `ObservedPhase::Gated` nach `publish_gate` und `Completed` nach `complete`. `Restricted`/`Lifted` bleiben ungenutzt in Stufe 1. Zustellung ist **nicht** Teil von G1: Observer-Fehler dürfen den Auftrag nie blockieren oder Gate/Purge verändern (nur Zähler und Warnung). Caches dürfen sich nicht allein auf den Observer verlassen, sondern tragen die Gate-Epoche im Schlüssel (10 §9).

## 5. Kanonische Serialisierung des Plans

`plan_hash = BLAKE3( b"ctx-erase-plan-v1" ‖ target.canonical_bytes() ‖ u32 n_rules ‖ jede Regel (verdict u8, kind u8, len u32 LE, bytes) in sortierter Reihenfolge ‖ docs … ‖ u64 unprovenanced )`. DocScope::Set: `tag 1, tenant u64, collection (len+bytes), n u32, DocId aufsteigend (u64 LE bzw. u128 LE)`. Golden-Test fixiert das Format.

## 6. Re-Exporte (`lib.rs`)

Entfernen: `barrier::*`, `keys::{FileKeyring,FileKey,FileKind}`, `gate::{KeyBound,KeyRange}`, `PortEraseTarget`, `ErasureTarget`, `StorageBackend`, `LayerScope`, `LayerId`. Hinzufügen: Typen aus §4. Breaking Change; Nutzerprüfung: `rg "PortEraseTarget|CommitBarrier|PurgeBarrier|FileKeyring|KeyBuf|LayerScope" crates` (Erwartung: nur `ports`-intern).

## 7. Tests

| ID | Inhalt |
| --- | --- |
| TST-PORTS-01 | Compile-Fail: `impl ReadGate for X {}` ohne alle Methoden schlägt fehl (`trybuild`, G-6) |
| TST-PORTS-02 | Compile-Fail: `ScanSession::begin/finish` von außen nicht aufrufbar (bestehender Test bleibt) |
| TST-PORTS-03 | `KeyScope`: Longest-Prefix-Wins, Exact-vor-Prefix, Komplement-Fall (`t:7:` In, `t:7:__col:` Out, `t:7:__txt:default:` In) |
| TST-PORTS-04 | `KeyScope::could_overlap_range` hat keine Falsch-Negative (proptest gegen Brute-Force über Schlüsselmengen) |
| TST-PORTS-05 | `KeyScope::new` lehnt > 4096 Regeln und > 1 MiB ab |
| TST-PORTS-06 | `plan_hash` Golden und Reihenfolgeunabhängigkeit der Regeleingabe |
| TST-PORTS-07 | `Debug` von `ResolvedScope`/`KeyScope` enthält keine Schlüsselbytes |
| TST-PORTS-08 | `types`-Gleichheit der Re-Exporte (01 TST-TYPES-11) |
| TST-PORTS-09 | Mock-`ErasureKernel`: Zustandsübergangs-Contract (Konformanz in 13) |

## 8. Definition of Done

Workspace baut ohne `KeyBuf`/Barrier/Keyring; keine neue Kante in `capabilities.toml`; Doku mit „# Fehler / # Crash“ je Methode (Repo-Konvention ✔); G-6 grün.

## 9. Offene Punkte

| Nr. | Frage | Vorschlag |
| --- | --- | --- |
| O-P1 | `CancellationToken` in `ports`? | Eigener `Cancel`-Trait, falls `tokio-util` nicht bereits Abhängigkeit (**[?]** `ports/Cargo.toml`) |
| O-P2 | `RecordSealer` jetzt oder in WP-11? | Verschieben nach `seal.rs` hinter Feature, in WP-11 ausarbeiten |
| O-P3 | `GateId`/`StoreId` aus `types` nutzen? | Nein (01 §4.1) |
