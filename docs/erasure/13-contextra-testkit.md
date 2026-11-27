# Implementierungsspezifikation `contextra-testkit` (Löschkette – CrashVfs, Konformanz, Referenzmodell)

*Ring 0 (Test) · Konzept-WP 12 · Abhängig von: 01, 02 · Blockiert: Crash-Tests von 03, Konformanz von 06–09, E2E von 10/11 · Index: `00-INDEX.md` (§6 DoD, K-17, K-31, K-36)*

> **Verifikationsgrenze.** Statisch, nichts kompiliert. **✔** gelesen, **[?]** Prüfauftrag.

## 1. Zweck

Ohne Werkzeug keine Beweisbarkeit: (1) **`CrashVfs`** erzwingt Abstürze an jedem I/O-Punkt und unterscheidet *durable* von *flüchtig* (Voraussetzung der Crash-Matrix CP-1…CP-18); (2) **Konformanzsuite C1–C7** für jeden `ErasureLayer`; (3) **Referenzmodell** (Nach `Completed` darf nichts des Scopes in irgendeiner Schicht gefunden werden, Nachbarn unverändert); (4) Hilfs-Gates und Mocks; (5) Canary-Prüfer für `T-DEL-1`; (6) Loom-Modelle; (7) CI-Gates mit Rot-Grün-Test.

## 2. Ist-Zustand (✔)

| Aspekt | Befund |
| --- | --- |
| Abhängigkeiten | `contextra-ports` (feature `testkit`), `contextra-types`, `bytes`, `parking_lot`, `tracing` (`Cargo.toml`). Kante auf `store`/`engine` fehlt und darf nicht entstehen (K-36); Features `default = ["testkit","erasure-conformance"]` |
| Dateien | `fault_vfs.rs`, `in_memory_{graph,store,text,vector}.rs`, `manual_clock.rs`, `mocks.rs`, `reference_model.rs`, `conformance/{clock,failure,graph,macros,metrics,rng,storage,text,vector,vfs}.rs`; Tests `conformance_selftest.rs`, `in_memory_doubles.rs` |
| `FaultVfs` | In-Memory (`BTreeMap<PathBuf, Vec<u8>>`), Zähler, `FaultConfig{fail_writes_after, fail_syncs_after, fail_reads_after, fail_all}`, `trigger_crash()`; `impl Vfs` (`:239`). **Kein** `open`/`rename`, `sync_dir` No-op (03 §9, `fault_vfs.rs:307`) ⇒ für Crash-Matrix ungeeignet |
| `ReferenceModel` | `put`, `delete`, `commit`, `commit_batch`, `rollback`, `snapshot_seq`, `get_at`, `scan_prefix_at`, `scan_range_at`, `get_latest`, `scan_prefix_latest`, `snapshot_map_at` (`reference_model.rs:50-228`): reiner **KV**-Referenzstand, ohne Erase |
| Vfs-Port | `ports/src/vfs.rs`: `Vfs` (`open`, `read_at`, `read_verified`, `read`, `write_atomic`, `create_dir_all`, `remove_file`, `unlink`, `rename` (**Default `Unsupported`**, K-17), `read_dir`, `exists`, `len`, `list_dir`, `list`, `sync_dir`), `VfsFile` (`read_at`, `write_at`, `set_len`, `len`, `sync_data`, `sync_all`), `VfsOpenOptions::{read_only, write_create}` |

## 3. Dateien

| Datei | Aktion |
| --- | --- |
| `src/crash_vfs.rs` | **NEU** (§4) |
| `src/gates.rs` | **NEU**: `AlwaysOpenGate`, `AlwaysRestrictedGate`, `RecordingGate` (kein Default in `ports`, 02 §4.1) |
| `src/erasure_mocks.rs` | **NEU**: `MockErasureKernel` (Zustandsautomat-Contract), `MockLayer`, `MockProgressSink`, `MockPrivilegedRead`, `ManualIoBudget` |
| `src/conformance/erasure_layer.rs` | **NEU**: Konformanzsuite C1–C7 (§5) |
| `src/conformance/macros.rs` | Makro `assert_layer_conformance!` |
| `src/erasure_model.rs` | **NEU**: `ErasureModel` (§6) |
| `src/canary.rs` | **NEU**: `Canary`, `scan_tree` (§7) |
| `tests/crash_vfs.rs`, `tests/layer_conformance_selftest.rs`, `tests/erasure_model.rs` | **NEU** |
| `Cargo.toml` | `loom` nur als `[target.'cfg(loom)'.dev-dependencies]` (Version wie Workspace für `engine`/`mvcc` ✔ übernehmen); **keine** neue Laufzeitabhängigkeit |
| `tools/erasure_gates.py`, `tools/tests/erasure_gates/…` | **NEU** (§9) |
| `lib.rs` | Exporte hinter `#[cfg(feature = "erasure-conformance")]` |

## 4. `CrashVfs`

### 4.1 Modell

Zwei Zustände je Datei und Verzeichniseintrag:

```text
Inode { durable: Vec<u8>, volatile: Vec<u8> }                    // sync_data/sync_all: durable = volatile
Namespace { durable: BTreeMap<PathBuf, InodeId>, volatile: BTreeMap<PathBuf, InodeId> }   // sync_dir(dir): Einträge unterhalb von dir durable
```

- **Schreiben** (`write_at`, `set_len`) ändert `volatile`. **`sync_data`/`sync_all`** übernimmt `volatile → durable` der Datei. **`sync_dir(p)`** übernimmt die Namensänderungen (`create`, `rename`, `remove`) unterhalb von `p` nach `durable`.
- **`write_atomic(path, data)`**: modelliert Temp-Datei + `sync_all` + `rename` + `sync_dir` (Vertragsdoku in `ports/src/vfs.rs`); nach einem Crash gilt **entweder** alter **oder** neuer Inhalt vollständig (nie gemischt).
- **`rename`** ändert nur `volatile`-Namespace; vor `sync_dir` darf ein Crash ihn verlieren **oder** behalten (`CrashPolicy` entscheidet).
- **Hardlinks** (`hard_link` ist nicht Teil von `Vfs`; Checkpoint-Klone nutzen `tokio::fs`, 09 §6): `CrashVfs` bietet `link(&self, from, to)` als Testhilfe (zwei Namen ↔ ein `InodeId`); Entfernen eines Namens gibt den Inhalt erst frei, wenn der letzte Name weg ist. So wird 09 TST-CKPT-ERA-7 prüfbar.
- Alle Operationen sind **deterministisch** (`BTreeMap`, keine Zeit, keine `HashMap`). Zufallsentscheidungen kommen aus `CrashSeed(u64)` + interner xorshift-Folge (reine Testlogik, kein `rand`).

### 4.2 API

```rust
pub struct CrashVfs { /* Arc<Mutex<State>> */ }
impl CrashVfs {
    pub fn new(seed: CrashSeed) -> Self;
    /// Zählt jede mutierende Operation (write_at, set_len, create, rename, remove, sync_*, write_atomic).
    pub fn op_count(&self) -> u64;
    /// Plan: nach der n-ten mutierenden Operation „abstürzen“ (diese Operation ist noch ausgeführt, die folgenden liefern Err(Interrupted)).
    pub fn crash_after(&self, n: u64);
    /// Alternativ: vor der n-ten Operation.
    pub fn crash_before(&self, n: u64);
    /// Berechnet den Zustand nach Neustart gemäß Politik und gibt ein frisches, nicht abgestürztes `CrashVfs` zurück.
    pub fn restart(&self, policy: CrashPolicy) -> CrashVfs;
    pub fn fail_sync(&self, path_prefix: &Path, after: u64, errno: ErrKind);   // fsync-Fehler (Poison-Pfad, 03 §5.3)
    pub fn fail_write(&self, path_prefix: &Path, after: u64, errno: ErrKind);  // ENOSPC/EIO
    pub fn link(&self, from: &Path, to: &Path) -> io::Result<()>;
    pub fn snapshot_tree(&self) -> TreeSnapshot;       // Pfad → (durable, volatile) für Canary-Scan
}
pub enum CrashPolicy {
    /// Nur Durables überlebt (strengster Fall, Verlust jeder ungesyncten Änderung).
    DropUnsynced,
    /// Ungesyncte Datei-Schreibvorgänge überleben bis zu einem seed-gewählten Präfix (Torn Write, Byte-genau oder 512-/4096-Byte-Sektoren).
    TornTail { sector: u32 },
    /// Ungesyncte Namensänderungen werden in seed-gewählter Teilmenge angewandt (rename ohne sync_dir).
    PartialNamespace,
    /// Alles Volatile überlebt (Prozess-Kill ohne OS-Crash).
    KeepAll,
    /// Bitflip an seed-gewählter Stelle einer durablen Datei (CP-14, Medienfehler).
    BitFlip { path_prefix: PathBuf },
}
```

`impl Vfs for CrashVfs` vollständig (alle Methoden, **inklusive** `rename`, `open` mit `VfsFile`, `sync_dir`). Fehler nach dem Crashpunkt: `io::ErrorKind::Interrupted`; Code unter Test behandelt das wie jeden I/O-Fehler (kein Panik-Pfad).

### 4.3 Systematische Crash-Schleife

```rust
pub fn for_each_crash_point<F>(setup: impl Fn(&CrashVfs) -> Scenario, policies: &[CrashPolicy], check: F)
where F: Fn(&CrashVfs /* nach restart */, CrashPoint) -> Result<(), String>;
```
1. Läuft das Szenario **ohne** Crash und liest `N = op_count()`.
2. Für jedes `n ∈ 0..=N` und jede `policy`: neues `CrashVfs` mit gleichem Seed, `crash_after(n)`, Szenario ausführen (Fehler erwartet, kein Panik), `restart(policy)`, `check(...)`.
3. `check` führt **Wiederaufnahme** aus (Kernel öffnen, Treiber `run_job` bis `Completed`) und prüft: Endzustand ≙ Endzustand ohne Crash (Gate bleibt bis `Completed` gesetzt, Witness 0, Nachbarn bit-gleich). Obergrenze der Läufe pro Szenario `max_points` (Default 4 000; darüber seed-gezogene Stichprobe mit festem Seed, Meldung in der Test-Ausgabe).
4. Jede CP-Zeile der Matrix (03 §9, Konzept 8.1) erhält ein **benanntes** Szenario (`cp_01_after_requested` … `cp_18_pending_delete_gap`) und einen gezielten Haltepunkt über `CrashVfs::mark("name")`-Marker, die der Store im Testbetrieb **nicht** kennt: Szenarien finden ihren Punkt über den Op-Zähler **und** das Muster der letzten Operation (`last_ops()` liefert `Vec<OpRecord{kind,path_hash}>`). So bleibt der Store frei von Testcode (keine `cfg(test)`-Haken im Produktionspfad).

### 4.4 Grenzen (ehrlich)

`CrashVfs` modelliert **POSIX-Semantik** (durable ⇔ `fsync`/`sync_dir`), keine Controller-/Flash-Eigenschaften, keine CoW-Dateisysteme. Es beweist die **Protokollkorrektheit** der Reihenfolge Write-Ahead → Wirkung → Sync, nicht die Hardware (Konzept 2.2).

## 5. Konformanzsuite C1–C7 für `ErasureLayer`

### 5.1 Fixture-Vertrag

```rust
pub trait LayerFixture: Send + Sync {
    type Layer: ErasureLayer;
    fn new(vfs: CrashVfs) -> Self;
    fn layer(&self) -> &Self::Layer;
    /// Füllt Scope-Dokumente (`n_scope`) und Nachbarn (`n_neighbor`), deterministisch aus Seed.
    fn seed(&self, n_scope: usize, n_neighbor: usize);
    fn scope(&self) -> ResolvedScope;
    /// Alle Zustandsbytes/Objekte, die **nicht** zum Scope gehören, als kanonischer Digest (C3).
    fn neighbor_digest(&self) -> [u8; 32];
    /// Setzt ein Restobjekt des Scopes **absichtlich** nach dem Purge zurück (C4).
    fn plant_residue(&self);
    /// Fehlerinjektion in den Schreib-/Lesepfad der Schicht (C5).
    fn inject_fault(&self, f: FaultSpec);
    fn reader(&self) -> Arc<dyn PrivilegedRead>;   // Scope-prüfender Reader (Mock oder Store)
    fn gate(&self) -> Arc<dyn ReadGate>;
    fn is_visible(&self, doc: DocId) -> bool;       // Suchpfad der Schicht, C7
}
```

### 5.2 Fälle (Pflicht, je ein `#[tokio::test]`)

| Fall | Prüfung |
| --- | --- |
| **C1 Idempotenz** | `purge` zweimal bis `cursor == None`: gleicher Endzustand; zweiter Lauf hat `units_committed == 0` |
| **C2 Wiederaufnahme** | Für **jeden** Cursor, den ein vollständiger Lauf produziert: Lauf abbrechen, mit diesem Cursor fortsetzen ⇒ Endzustand gleich ununterbrochenem Lauf |
| **C3 Scope-Isolation** | `neighbor_digest()` vor und nach `purge` identisch (bit-gleich); Zugriff auf Nicht-Scope-Schlüssel über `reader` ⇒ `ScopeViolation` |
| **C4 Witness-Ehrlichkeit** | Nach `plant_residue()` meldet `scanner().scan(...)` `remaining > 0`; ohne Rest `remaining == 0`; Positivkontrolle schlägt bei defektem Scanner fehl ⇒ `NotMeasured` (I-6) |
| **C5 Fehlerinjektion** | Jeder injizierte Fehler (`Io`, `Corrupt`, `Timeout`) ⇒ `Err(LayerError)`, **nie** `Ok` mit Teilzähler; `LayerErrorKind` passend (06 §5.5) |
| **C6 Cancel-Safety** | `cancel` an jeder `await`-Stelle (Cancel-Token feuert nach k Polls, k = 0..K) ⇒ kein inkonsistenter Zustand, Wiederaufnahme liefert Endzustand von C1 |
| **C7 Gate-Respekt** | Nach `on_gate`: `is_visible(doc) == false` für alle Scope-Dokumente, Nachbarn sichtbar; Layer liest Scope-Daten **nur** über `cx.reader` (Mock-Storage zählt Direktzugriffe auf gesperrte Schlüssel = 0) |

Zusatz (nicht C-nummeriert, Pflicht): **K1** `on_gate` ist idempotent und < 10 ms (Zeit über `ManualClock` nur als Gesamtbudget, nicht wall-clock); **K2** keine `HashMap`-Iterationsreihenfolge im Ergebnis (zweimal laufen, Bytes vergleichen).

### 5.3 Makro

```rust
assert_layer_conformance!(MyFixture);   // erzeugt C1…C7 + K1/K2 als einzelne Tests
```
`conformance_selftest.rs` enthält je Fall einen **absichtlich fehlerhaften** Layer (z. B. committet Teilzähler bei Fehler, rührt Nachbarn an, meldet Witness 0 trotz Rest) und erwartet, dass die Suite **rot** wird (Mutationsnachweis der Suite selbst).

## 6. Referenzmodell (`erasure_model.rs`)

```rust
pub struct ErasureModel {          // aufbauend auf ReferenceModel (KV) + Schichten
    kv: ReferenceModel,
    docs: BTreeMap<(TenantId, String, DocId), DocRecord>,   // Körper, Embedding-Digest, Terme
    postings: BTreeMap<(TenantId, String, String /*term*/), BTreeSet<DocId>>,
    graph: BTreeMap<ObjRef, BTreeSet<(String /*collection*/, DocId)>>,   // derived_from
    checkpoints: BTreeMap<String, (u64 /*seq*/, BTreeSet<(TenantId, String)>)>,
}
impl ErasureModel {
    pub fn apply(&mut self, op: Op);                               // insert, update, delete (Retire), link, checkpoint
    pub fn erase(&mut self, target: &EraseTarget);                 // physisch aus **allen** Schichten
    pub fn expected_visible(&self, tenant: TenantId) -> BTreeSet<(String, DocId)>;
    pub fn assert_matches<S: SystemUnderTest>(&self, sut: &S) -> Result<(), ModelDiff>;
}
```
Eigenschaft (Konzept 9, „Modelltest“): Nach `Completed` darf **kein** Schlüssel des Scopes in **keiner** Schicht des SUT gefunden werden (`sut.find_any(scope)` über KV, Text, Vektor, Graph, Checkpoints), Nachbarn identisch zum Modell. **Proptest:** beliebige Operationsfolgen (≤ 60 Operationen, 3 Mandanten, 3 Collections), beliebig `erase(Document|Collection|Tenant)` eingestreut, beliebig `restart()` über `CrashVfs`, am Ende `assert_matches`. Property **Erase ∘ Erase = Erase** (idempotent) und **Retire ≠ Erase** (nach `delete` bleibt Zeitreise sichtbar; nach `erase` nicht, **auch** nicht für `get_at_seq(seq < gate)`).

## 7. Canary `T-DEL-1` (E2E in `contextra/tests/deletion_e2e.rs`)

```rust
pub struct Canary { pub value_needle: Vec<u8> /* 48 Byte eindeutig, aus Seed */, pub key_needle: Vec<u8>, pub doc_id_needles: Vec<Vec<u8>> /* LE, BE, Dezimal, Hex */ }
pub fn scan_tree(root: &Path, needles: &[&[u8]], opts: ScanOpts) -> Vec<Hit>;   // rekursiv, byteweise, Dateien beliebigen Typs
pub struct ScanOpts { pub include_mmap_regions: bool, pub exclude: Vec<PathBuf> /* nur Ledger-Anker außerhalb */ }
```

Ablauf (Index DoD 3): (1) Dokument mit `value_needle` im Text und im Embedding-Rohbytemuster schreiben (über alle Schichten: Text, Vektor, Graph-Kante, Checkpoint, KV), Flush + Compaction erzwingen, Checkpoint anlegen; (2) `erase(Document)` mit `await = Purged`; (3) `scan_tree(data_dir)` **und** `scan_tree(checkpoint_clone_dir)` **und** Ledger-Pfad: **0 Treffer** für `value_needle`; **0 Treffer** für `key_needle` nach erzwungener Journal-Verdichtung (Journal enthält nur HMAC, I-12) — vorher erlaubt (Journal hält Schlüsselbytes für Gate/Filter, 03 §5); (4) Prozess-Dump (`/proc/self/mem`-Scan) ist **nicht** Teil von CI, nur optionaler manueller Test hinter `--features heap-scan` (Konzept 2.2: Heap/Swap sind außerhalb, Betriebsvorgabe). (5) Nachbardokument bit-identisch lesbar, `erase_status == Completed`, `guarantee == G1+G3`.

Zusätzlich **Negativkontrolle** (Scanner-Eignung): Derselbe Test **ohne** `erase` findet die Nadel mindestens in WAL **oder** SSTable (`scan_tree` ≠ leer), sonst ist der Scan wertlos (Test schlägt fehl).

## 8. Loom-Modelle (K-31)

`contextra-store` wird unter `cfg(loom)` nicht gebaut. Deshalb prüft Loom **Modelle** der Protokolle in `contextra-testkit/tests/loom_erasure.rs` (`#![cfg(loom)]`):

| ID | Modell | Eigenschaft |
| --- | --- | --- |
| LOOM-R1 | Writer-Commit vs. Gate-Veröffentlichung unter `commit_mutex` (zwei Mutexe + Atomics für `gated_seq`/`epoch`) | Kein Commit mit `seq > gated_seq` auf gesperrtem Schlüssel wird akzeptiert; kein Commit zwischen „Sequenz gelesen“ und „Gate sichtbar“ |
| LOOM-R4 | Leser (Kandidaten → Hydration) vs. Gate-Publish (`e1`/`e2`) | Leser liefert nie ein Dokument, das nach `e2` gesperrt ist |
| LOOM-R7 | Boot: Gate laden vs. WAL-Replay-Thread | Replay beginnt erst nach Gate-Laden (Atomic-Fence-Modell) |
| LOOM-CKPT | Restore-Prüfung vs. Gate-Publish (09 §4 Schritt 3) | Rollback nur nach Prüfung mit gleicher Epoche |

Modelle bilden die **Reihenfolgeregeln** ab (Acquire/Release), sie ersetzen nicht den Stresstest `chaos_*` im Store.

## 9. CI-Gates (`tools/erasure_gates.py`)

Ein Python-Skript (nur Standardbibliothek), Exit-Code ≠ 0 bei Verstoß; Ausgabe `datei:zeile: Gx: Text`.

| Gate | Regel | Geltungsbereich (Pfade, Stand 00-INDEX §4) |
| --- | --- | --- |
| G-1 | kein `block_on` / `futures::executor::block_on` / `Handle::block_on` | `store/src/erasure/**`, `engine/src/erasure/**`, `engine/src/background_workers/erasure_worker.rs`, neue Layer-Dateien (`text/src/erasure.rs`, `vector/src/hnsw/erasure.rs`, `graph/src/erasure.rs`, `checkpoint/src/erasure.rs`), `crypto/src/{vault,erasure_root}.rs` |
| G-2 | kein `let _ =`, `unwrap_or_default()`, `.ok();` (ausgenommen Zeilen mit `// gate-allow: G-2 <Begründung>`; Begründung Pflicht) | dieselben Pfade plus `crypto/src/dek.rs` |
| G-3 | `#[tracing::instrument]`/`#[instrument]` ohne `skip`/`skip_all`, an Funktionen mit Parametern namens `id`, `key`, `target`, `doc_id`, `term` | Workspace `crates/*/src` |
| G-4 | kein `std::fs`/`tokio::fs` | `store/src/erasure/**`, `engine/src/erasure/**`, neue Layer-Dateien (Liste wie G-1; Ausnahme: `checkpoint/src/hardlink_cloner.rs`, nicht neu) |
| G-5 | kein `Instant::now`, `SystemTime::now`, `HashMap`, `HashSet` | neue Dateien (Liste wie G-1) |
| G-6 | `impl ReadGate for` ohne Default-Methoden (Parser zählt Methoden gegen die Trait-Definition) und `trait ReadGate` ohne `{ … }`-Defaultkörper | `ports/src/gate.rs`, `store/src/erasure/gate.rs`, `testkit/src/gates.rs` |

**Rot-Grün-Test je Gate** (`tools/tests/erasure_gates/`): `g1_red.rs` (muss Fehler liefern) und `g1_green.rs` (muss sauber sein), … `g6`; `pytest`-ähnlicher Lauf `python3 tools/erasure_gates.py --self-test` prüft, dass **jedes** Gate auf Rot-Beispiel anschlägt und auf Grün-Beispiel schweigt. CI ruft `--self-test` **vor** dem eigentlichen Lauf. Einbindung: `rust-ci.yml` (existiert ✔ laut Konzept) bekommt Schritt „Erasure-Gates“.

## 10. Mutationstests

`mutants-guarantees.toml` (existiert ✔) bekommt Einträge für die **Garantiepfade**: `store/src/erasure/gate.rs` (`check`, `check_version`, `discard_rule`), `store/src/erasure/filter.rs`, `store/src/erasure/purge_lsm.rs` (Filter in Flush/Merge/Replay), `engine/src/erasure/driver.rs` (Zustandsfolge), `ports/src/erasure.rs` (`KeyScope::contains`, `could_overlap_range`). Ziel: **Mutationsscore 100 %** auf diesen Dateien (jeder überlebende Mutant ist ein fehlender Test). Format der Datei **[?]** vor Ergänzung lesen.

## 11. Zuordnung Test ↔ Invariante (DoD 1)

| Invariante | Test |
| --- | --- |
| I-1 Reihenfolge | TST-ENG-ERA-3, `MockErasureKernel`-Contract (TST-PORTS-09) |
| I-2 Write-Ahead | Crash-Schleife mit Journal-Sync-Fehler (`fail_sync`) |
| I-3 Kein Wiederaufleben | Crash-Szenarien CP-5…CP-7, Replay/Flush/Compaction mit Erase-Filter |
| I-4 Idempotenz | C1, Property „Erase ∘ Erase“ |
| I-5 Epoche monoton | LOOM-R1/R4, Property-Test auf `GateSnapshot.epoch` |
| I-6 Witness-Ehrlichkeit | C4, TST-ENG-ERA-5 |
| I-7 Fail-closed | CP-14 (Journal mittendrin kaputt) ⇒ alles `Restricted` |
| I-8 Scope-Treue | C3 + Resolver-Tests (10 TST-ENG-ERA-2) |
| I-9 Gate-Lebensdauer | CP-11, TST-CKPT-ERA-6 |
| I-10 Lock-Ordnung | Store-Lock-Order-Tests (03), LOOM-R1 |
| I-11 Pins blockieren G1 nie | 04 Tests, TST-ENG-ERA-7 |
| I-12 Kein Klartext | Canary (`key_needle` nach Verdichtung), TST-MCP-ERA-5, G-3 |

Jede Invariante hat damit mindestens **einen** Test, der bei Verletzung rot wird; die Tabelle ist selbst ein Test (`tests/invariant_coverage.rs` liest diese Datei und prüft, dass jede genannte Testkennung als `fn` im Workspace existiert).

## 12. Arbeitspakete

| WP | Inhalt | Blockiert |
| --- | --- | --- |
| 13.1 | `CrashVfs` + `for_each_crash_point` + Selbsttests | Crash-Tests 03 (WP-3.x) |
| 13.2 | `gates.rs`, `erasure_mocks.rs` (inkl. `MockErasureKernel`) | 02 TST-PORTS-09, 10 |
| 13.3 | `conformance/erasure_layer.rs` + Makro + Selbsttest rot | 06–09 |
| 13.4 | `tools/erasure_gates.py` + Rot/Grün | alle Pakete (CI) |
| 13.5 | `ErasureModel` + Proptest | 10 |
| 13.6 | `Canary`, `scan_tree`, `T-DEL-1` | 10, 11 |
| 13.7 | Loom-Modelle | 03 §6 |
| 13.8 | Mutationseinträge, `invariant_coverage.rs` | alle |

## 13. Definition of Done

`CrashVfs` besteht Selbsttests (durable/flüchtig, `rename` ohne `sync_dir`, Torn Write, Hardlink); Suite C1–C7 erkennt alle absichtlich defekten Layer; Gates G-1…G-6 mit Rot-Grün-Test; `tests/layering.rs` grün (testkit nur `ports`/`types`); keine neue Kante in `capabilities.toml`; keine `HashMap` im neuen Code (G-5).

## 14. Offene Punkte

| Nr. | Frage | Vorschlag |
| --- | --- | --- |
| O-X1 | Format `mutants-guarantees.toml` | lesen, dann Einträge ergänzen |
| O-X2 | Loom-Version/Feature in Workspace | aus `engine`/`mvcc` übernehmen |
| O-X3 | Hardlink in `Vfs`-Port aufnehmen? | Nein in Stufe 1 (nur Testhilfe `link`); sonst Port-Änderung mit Nachtrag zu 02 |
| O-X4 | `SystemUnderTest`-Trait für `ErasureModel::assert_matches` | Implementierung im `contextra`-E2E-Test (testkit kennt keine Engine) |

## Revision 2 (28.11.2026, Audit gegen HEAD `e12873de6`; Details in 14)

**Stand.** `erasure_mocks.rs` (1 110 LOC; `MockErasureKernel` erzwingt `can_transition_to`), `erasure_model.rs` (440), `conformance/erasure_layer.rs` (566), `tests/loom_erasure.rs`, `tests/erasure_model.rs`, `tests/layer_conformance_selftest.rs` ✔.

**Ergänzungen.** (1) `MockLayer` erhält Fehlerinjektion: `fail_on_gate(n)`, `residue_after_purge(n)` (Zeuge meldet Rest), `fail_purge_retryable(n)`, `slow_purge(units)`. (2) Treibertests laufen gegen **beide** Kernel (`MockErasureKernel`, echter `StoreErasureKernel` über `CrashVfs`); das Referenzmodell prüft, dass die Übergangstabelle in beiden identisch ist (Differential-Test `TST-MODEL-DIFF-1`). (3) Konformanz: neu **C8** `CONF-LAYER-READ-1` und `CONF-LAYER-IDEM-1`; **jeder** produktive Layer (Text, HNSW, Graph, Checkpoint) führt die Suite aus, nicht nur Selbsttests. (4) `CrashVfs`-Szenarien CP-19…CP-30 (14 §5) als Tabelle `crash_matrix()`; Torn-Write, Reorder-nach-`sync_dir`, Verlust ungesyncter Renames. (5) Loom-Modelle: (a) Schreiber gegen Gate-Veröffentlichung, (b) `complete` gegen Lese-Epoche, (c) Layer-`purge` gegen `restore_checkpoint`. (6) Canary `T-DEL-1` läuft in Klartext- und verschlüsseltem WAL-Modus.
