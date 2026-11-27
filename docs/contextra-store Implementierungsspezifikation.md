# contextra-store: Implementierungsspezifikation

*Principal-Review des Storage-Kerns (`crates/contextra-store`, Ring 1) · Stand HEAD `9173b20eb` · 27.11.2026*

> **Verifikationsgrenze.** Rein statische Analyse. In dieser Umgebung gibt es keine Rust-Toolchain, es wurde nichts kompiliert und kein Test ausgeführt. **✔** = Code gelesen, Beleg angegeben. **\[?\]** = plausibel, aber nicht bewiesen; jeweils mit Prüfauftrag. Gelesen wurden `manifest/*`, `lsm/recovery.rs` (Orphan-/HWM-Logik), `lsm/ops/compaction.rs` (Flush), `compaction/engine.rs` (Swap/Rollover), `wal/flusher.rs` (Fehlerpfade), `sstable/builder.rs` (Fsync-Folge), `lsm/lock_order.rs`, `lib.rs`, `Cargo.toml`. **Nicht** geprüft: `memtable.rs`, `sstable/reader.rs`, `block_cache.rs`, `tenant_codec.rs`, `scanner/*`, `erasure/*` (4k Zeilen, hat eigene Spezifikation in `docs/erasure/03`), `wal/replay.rs` im Detail.

## 1. Ergebnis in Kürze

Die Crate ist insgesamt diszipliniert gebaut: `forbid(unsafe_code)`, Lock-Rangordnung, fail-closed bei fsync-Fehlern in WAL, Manifest-Append und SSTable-Builder, sehr breite Testlandschaft (Crash-, Chaos-, Loom-, Proptest-Suiten). Die kritischen Lücken liegen **nicht** in den Hot Paths, sondern im **Manifest als Wahrheitsquelle** und in der **I/O-Abstraktion**:

1. **Das Manifest-Rollover kann Daten und die WAL-Integritätsverankerung verlieren** (F1, F2). Das ist ein P0-Befund.
2. **WAL und Manifest laufen an der VFS-Abstraktion vorbei** (F3). Genau die zwei kritischsten Dateien sind per Crash-/Fault-Injection nicht prüfbar.
3. Fehlerpfade des Manifests sind nicht „fail-closed“ (F4, F5, F6).
4. Ein stilles „Orphan überspringen“ verwandelt jeden Manifest-Fehler in unsichtbaren Datenverlust (F7).

Die Spezifikation unten behebt das durch ein **Manifest, das seinen Zustand selbst besitzt** (kein Snapshot vom Aufrufer mehr), einen **einheitlichen I/O-Port** und eine **serialisierte Katalog-Mutation** (`catalog_mutex`).

## 2. Befunde

Schwere: **P0** = Datenverlust/Unrecoverability möglich, **P1** = Korrektheit/Latenz/Testbarkeit, **P2** = Wartbarkeit.

| ID | Sev | Befund | Beleg ✔ |
| --- | --- | --- | --- |
| F1 | P0 | **Rollover verliert `WalCheckpoint` und Dead-Records.** Der Aufrufer übergibt nur `ManifestEntry::Add` aus `sstables`. Nach dem ersten Rollover liefert `extract_high_water_mark` `None`; die Erkennung von WAL-Truncation (`verify_wal_chain_completeness`) wird damit beim nächsten Start ausgehebelt. `Remove`-Spuren gehen ebenfalls verloren: tote SST-Dateien nach einem Crash zwischen `Replace` und Datei-Löschung werden nicht mehr als „dead“ erkannt, sondern nur noch als „unmanifested“ gewarnt (Leck). | `compaction/engine.rs` (Block „current\_live\_entries“, `maybe_rollover`), `manifest/core.rs` `extract_high_water_mark`, `lsm/recovery.rs` HWM-Block |
| F2 | P0 | **Snapshot-Race beim Rollover.** Die Live-Liste wird unter `sstables.read()` gebildet und danach ohne Sperre an `maybe_rollover` gereicht. Der Flush schreibt `Add`+`WalCheckpoint` ins Manifest **bevor** er die SSTable in `sstables` einhängt, und die Compaction hält `flush_mutex` nicht. Läuft das Rollover in diesem Fenster, enthält das neue Manifest die frisch geflushte SSTable nicht. Der Flush löscht danach das WAL; nach Neustart wird die SSTable als „unmanifested (skipping)“ ignoriert. Erreichbarkeit des Fensters **\[?\]**, Struktur ✔. | `lsm/ops/compaction.rs` (Flush: `manifest.append_batch` vor `sstables.write()`), `compaction/engine.rs`, `lsm/recovery.rs` |
| F3 | P0/P1 | **VFS-Umgehung.** `wal/*` hat 0 Referenzen auf `vfs`/`StoreIo` und nutzt `tokio::fs` (46 Stellen); `manifest/core.rs` ebenso; `rollover.rs` nutzt `StoreIo::default()` (Real-FS) auch wenn `LsmConfig.vfs` gesetzt ist. SSTables und Erasure-Journal laufen dagegen über `Vfs`. Folge: `CrashVfs`/Fault-VFS decken WAL und Manifest nicht ab; Tests dafür sind auf globale `FAIL_*`-Statics und String-Matching angewiesen (F8). | `wal/mod.rs` `pub(crate) mod fs { pub use tokio::fs::* }`, `manifest/core.rs`, `manifest/rollover.rs` |
| F4 | P0 | **Rollover: Fehler nach `rename` hinterlässt toten Handle.** Schlägt `sync_dir` oder das Wieder-Öffnen nach dem `rename` fehl, bleibt `self.file` auf der unlinked alten Inode. Das Cleanup löscht nur die (nicht mehr existente) tmp-Datei. Folgende Appends werden gefsynct, aber in eine verwaiste Datei geschrieben und gehen beim Neustart verloren. Das Manifest wird nicht vergiftet. | `manifest/rollover.rs` (`rollover_res`-Block) |
| F5 | P0 | **Append ohne Torn-Frame-Schutz.** Nur ein `sync_all`-Fehler vergiftet. Schlägt `write_all`/`flush` nach Teilschreibung fehl (z. B. ENOSPC), bleibt ein halber Frame mitten in der Datei; der nächste erfolgreiche Append hängt dahinter. `load` meldet das als Mid-File-Korruption (`Err`), der Store lässt sich nicht mehr öffnen. Zusätzlich ist ein `append_batch` nicht atomar (mehrere Frames). | `manifest/core.rs` `append_batch`, `load` |
| F6 | P1 | **`load` maskiert I/O-Fehler als „leer“.** `try_exists(..).unwrap_or(false)` bildet z. B. `EACCES` auf „kein Manifest“ ab. Folge in Kombination mit F7: alle SSTables erscheinen als Orphans. Außerdem: Header-Erkennung bei Dateien mit 1–4 Bytes (Crash zwischen `create_new` und Header-Write) und bei vorhandener 0-Byte-Datei (`open` schreibt Header nur bei `create_new`) führt zu gemischten Formaten bzw. fehlerhaftem Parsen von `CMAN…` als Länge. | `manifest/core.rs` `open_with_dir_lock`, `load` |
| F7 | P0 | **Orphan-SST wird nur gewarnt und übersprungen**, wenn ein gültiges Manifest existiert und die Datei weder „valid“ noch „dead“ ist. Das ist ein stiller Datenverlust-Pfad für jeden Manifest-Fehler (F1, F2, F4, F6). | `lsm/recovery.rs` (Warnung „Unmanifested or orphaned SSTable file…“) |
| F8 | P1 | **Fehlerklassifikation per String-Match.** Die WAL-Poison-Ursache wird über `write_err.to_string().contains("FAIL_APPEND_PARTIAL_ONCE")` bestimmt, ohne `cfg`-Gate an der Match-Stelle. Fault-Injection über globale `static AtomicBool` (`FAIL_SYNC_ONCE`) ist nicht instanzgebunden und läuft bei parallelen Tests ineinander. | `wal/flusher.rs` |
| F9 | P1 | **Fsync unter `sstables.write()`.** Die Compaction ruft `manifest.append` (Write+fsync) innerhalb der `sstables.write()`-Sektion auf. Die Lock-Tabelle verspricht < 100 µs für diese Sperre; ein fsync dauert Millisekunden. Alle Leser blockieren währenddessen. Außerdem bleibt bei `?` nach fehlgeschlagenem Append die Output-SSTable als Datei liegen. | `compaction/engine.rs` (Swap-Block), `lsm/lock_order.rs` |
| F10 | P2 | **Doku/Code-Drift:** `lib.rs` und `erasure/mod.rs` behaupten CRC32C; Manifest und Ledger nutzen `crc32fast::hash` (IEEE CRC-32). `#![allow(unexpected_cfgs)]` in `lib.rs` hebelt das Workspace-`deny` aus; mehrere `#[allow(dead_code)]` unterlaufen `#![deny(dead_code)]` (`wal/hmac.rs`, `wal/mod.rs`, `compaction/engine.rs`). `CONTEXTRA_INTERFACE_SPECIFICATION.md` existiert zweimal (8232 Zeilen, vermutlich identisch). | `lib.rs`, `manifest/entry.rs`, Greps |
| F11 | P2 | **Verlustbehaftete Pfade im Format.** `ManifestEntry` speichert `path.to_string_lossy()` samt Verzeichnis; Recovery vergleicht nur `file_name`. Längenfelder werden mit `as u32` geschrieben (Workspace-Lint `cast_possible_truncation` ist nur `warn`). `Replace.rank` ist abgekündigt, wird aber weiter geschrieben/gelesen. | `manifest/entry.rs` |
| F12 | P2 | **Lock-Rangordnung nur dokumentiert, nicht erzwungen.** `lock_order.rs` ist Prosa plus Konstanten; Verletzungen werden nur indirekt durch Loom-Tests gefunden. Keine Rang für „Katalog-Mutation“ (Flush, Compaction, Rollback, Erase-Purge), obwohl diese seriell sein müssen (siehe F2). | `lsm/lock_order.rs` |

## 3. Zielinvarianten (normativ)

- **INV-M1 (Manifest ist vollständig):** Der durable Manifest-Zustand ist zu jedem Zeitpunkt die Reduktion aller je bestätigten Einträge. Ein Rollover verliert nichts (Live-Set, Dead-Set, Rollback-Marker, WAL-HWM).
- **INV-M2 (Besitz):** Nur `Manifest` bildet seinen Zustand. Kein Aufrufer liefert Snapshots.
- **INV-M3 (Atomarer Batch):** Ein `append_batch` ist ein Frame mit einer CRC; er ist ganz oder gar nicht sichtbar.
- **INV-M4 (Fail-closed):** Jeder I/O-Fehler nach einem Teilschreibvorgang führt entweder zu bewiesen sauberer Rücknahme (Truncate auf `committed_len`, anschließend fsync) oder zum Poisoning des Handles. Kein Retry nach fsync-Fehler.
- **INV-M5 (Keine stillen Orphans):** Unbekannte `.sst`-Dateien werden nie gelöscht und nie still ignoriert (Quarantäne oder Boot-Fehler).
- **INV-IO1 (Ein I/O-Port):** Jede Dateioperation in `contextra-store` (Produktionscode) läuft über `StoreIo`/`Vfs`. `tokio::fs` und `std::fs` sind per Lint verboten.
- **INV-C1 (Katalog serialisiert):** Alle Änderungen am SSTable-Katalog (Flush-Publish, Compaction-Swap, Rollback, Erase-Purge, Rollover) laufen unter `catalog_mutex` (neuer Rang 350).
- **INV-C2 (Kurze Reader-Sperre):** `sstables.write()` wird nur für den reinen RAM-Swap gehalten (< 100 µs), nie über I/O.

## 4. Arbeitspakete

Reihenfolge und Abhängigkeiten: **WP-S1 → WP-S2 → WP-S3 → WP-S4** (P0-Pfad), danach **S5, S6, S7**; **S8** (Tests) begleitet jedes Paket. Jedes Paket ist so geschnitten, dass es allein kompiliert und `deny(dead_code)` erfüllt.

### WP-S1: Einheitlicher I/O-Port (F3, INV-IO1)

**Ziel:** WAL und Manifest laufen über `StoreIo`.

1. **Port erweitern** (in `contextra-ports`, Trait `VfsFile`). **\[?\]** Prüfauftrag: `rg "trait VfsFile" -A40 crates/contextra-ports`. Benötigt werden mindestens: `write_at`, `append`, `read_at`, `set_len`, `sync_data`, `sync_all`, `len`. Fehlendes wird ergänzt, bestehende Methoden bleiben unverändert (kein Breaking Change im `Vfs`-Vertrag).
2. `Wal::open_with_key_manager(path, km)` erhält `io: StoreIo` und hält `Box<dyn VfsFile>` statt `tokio::fs::File`. Alle blockierenden Aufrufe laufen in `spawn_blocking` hinter **einem** Helfer `AppendFile` (siehe WP-S2), damit Journal (`erasure/journal.rs`), Manifest und WAL denselben, einmal getesteten Append-Code teilen.
3. `Manifest::open_with_dir_lock(path, dir_lock, io)`; `rollover.rs` verwendet `self.io` statt `StoreIo::default()`.
4. **Lint-Sperre:** in `clippy.toml` (store-spezifisch via `#![deny(clippy::disallowed_methods)]` in `lib.rs`) `tokio::fs::*`, `std::fs::*` auf die Verbotsliste, Ausnahme nur `cfg(test)` und `io_seam.rs::RealFs`.
5. Das `pub(crate) mod fs`-Shim in `wal/mod.rs` (inkl. Loom-Variante mit `std::fs`) entfällt. Loom-Tests nutzen `MemVfs`.

**Akzeptanz:** `rg "tokio::fs|std::fs" crates/contextra-store/src` liefert nur `io_seam.rs` und `cfg(test)`-Code. `CrashVfs` kann jeden Write/Sync/Rename von WAL und Manifest abfangen (WP-S8).

### WP-S2: Manifest v2 mit eigenem Zustand (F1, F2, F4, F5, F6, F11)

**2.1 Format v2** (Header: Magic `CMAN`, Version `2`, danach Frames):

```text
frame  := len:u32le | crc:u32le | tag:u8 | body            (crc über tag||body, CRC32C)
tag 1  Batch   { n:u32, entries:[Entry] }                  (ein Frame = atomarer Batch, INV-M3)
tag 2  Snapshot{ live:[(SstName,max_tx)], dead:[SstName],
                 rollbacks:[u64], wal_hwm:Option<[u8;32]> } (muss erster Frame nach dem Header sein)
Entry  := Add{name,max_tx} | Replace{removed:[name],added,added_max_tx}
        | Remove{name} | RollbackComplete{target_tx} | WalCheckpoint{hmac}
SstName := u16 len | utf8 bytes     (nur Dateiname, kein Verzeichnis, kein lossy)
```

Regeln: Längen werden mit `u32::try_from` geprüft (Fehler statt `as`). `Replace.rank` entfällt. CRC32C über die Crate `crc32c` (Hardware-beschleunigt); falls `deny.toml` die Crate nicht zulässt, wird stattdessen die Doku auf „CRC-32 (IEEE)“ korrigiert und `crc32fast` beibehalten (F10). Entscheidung per ADR. Der v1-Reader bleibt für die Migration (2.5).

**2.2 Zustandsbesitz (INV-M2):**

```rust
struct ManifestState {            // reine Reduktion, kein I/O, ohne Locks testbar
    live: BTreeMap<SstName, u64>, // max_tx
    dead: BTreeSet<SstName>,
    rollbacks: BTreeSet<u64>,
    wal_hwm: Option<[u8; 32]>,
}
impl ManifestState {
    fn apply(&mut self, e: &Entry) -> Result<(), ManifestError>;  // validiert: Replace.removed ⊆ live
    fn to_snapshot(&self) -> Snapshot;
}
struct Manifest { io: StoreIo, path: PathBuf, inner: tokio::sync::Mutex<Inner>, poisoned: AtomicBool, … }
struct Inner { file: AppendFile, state: ManifestState }   // Rang 800
```

`append_batch` (unter `inner`-Lock): (1) Batch gegen eine **Kopie** des Zustands validieren (`apply`), (2) Frame enkodieren, (3) `file.append_durable(frame)` (siehe 2.3), (4) erst nach Erfolg den Zustand übernehmen. Jede Zustandsabfrage (`live()`, `wal_hwm()`) liest `inner.state`; `recovery.rs` benutzt `ManifestState` statt `reconstruct_valid_sstables`/`reconstruct_dead_sstables`/`extract_high_water_mark` (diese drei Funktionen werden gelöscht).

**2.3 `AppendFile` (geteilt von Manifest, WAL, Erasure-Journal):**

```rust
struct AppendFile { f: Box<dyn VfsFile>, committed_len: u64, poisoned: bool }
async fn append_durable(&mut self, bytes: &[u8]) -> Result<(), IoFault> {
    // 1. write_all at committed_len; Fehler -> rollback()
    // 2. sync_data; Fehler -> poison (kein Retry, INV-M4)
    // 3. committed_len += len
}
fn rollback(&mut self) { /* set_len(committed_len) + sync_data; scheitert das -> poison */ }
```

`IoFault` ist ein **typisierter** Fehler (`PartialWrite{written}`, `NoSpace`, `SyncFailed`, `Poisoned`). Kein String-Matching (F8).

**2.4 Rollover (INV-M1, F2, F4):** `rollover()` nimmt den `inner`-Lock und schreibt `Snapshot` aus **eigenem** Zustand; die Signatur verliert den Parameter `live_entries`. Protokoll:

1. `tmp = MANIFEST.new.<pid>.<ctr>` mit `create_new`, Header + Snapshot-Frame schreiben, `sync_all`.
2. Fehler bis hier: tmp löschen, Original unberührt, Fehler melden (Handle bleibt gültig).
3. `rename(tmp, MANIFEST)`, `sync_dir`, neu öffnen, `inner.file` ersetzen.
4. **Jeder Fehler ab dem rename ⇒ `poisoned = true`** (Zustand der Datei auf Platte ist die neue Datei, der Handle ist es nicht). Der nächste Start liest das neue Manifest; der laufende Prozess verweigert weitere Appends.

Auslöser: `maybe_rollover` wird vom Manifest **selbst** am Ende von `append_batch` geprüft (Schwelle auf Basis von `committed_len`), nicht mehr von der Compaction.

**2.5 Laden und Heilen (F6):**

- Existenz über `io.exists(path)?` (Fehler propagieren, nie `unwrap_or(false)`).
- Header-Fall: `len == 0` oder `len < 5` mit Präfix-Übereinstimmung zum Soll-Header ⇒ „Torn Creation“, Datei wird neu initialisiert (atomar via tmp+rename). Ungleicher Magic ⇒ `Err(UnknownFormat)`; v0/v1 nur über expliziten Migrationspfad.
- Torn Tail (unvollständiger Letzter-Frame, CRC-Fehler **nur** am Dateiende): vor dem ersten Append auf `last_valid_offset` kürzen (`set_len` + `sync_data`), damit F5 nie dahinter schreibt. **\[?\]** Prüfauftrag: ob `recovery.rs` heute heilt (`rg "set_len|truncate" crates/contextra-store/src/manifest crates/contextra-store/src/lsm/recovery.rs`).
- CRC-Fehler vor dem letzten Frame ⇒ `Err(ManifestCorrupt{offset})`, kein „best effort“.

**2.6 Migration v1 → v2:** Beim Start v1 lesen, in `ManifestState` reduzieren, via Rollover-Protokoll als v2 `Snapshot` schreiben, `MANIFEST.v1.bak` bis zum nächsten erfolgreichen Flush behalten. Kein Downgrade (ADR + Release-Note). Test: `sstable_format_v3_to_v4_compat`-Muster für Manifest.

**Akzeptanz:** Regressionstests F1 (HWM überlebt Rollover), F2 (Rollover parallel zu Flush, deterministisch mit Barrier), F4 (Fault nach rename ⇒ Poison, nächster Start findet alle Einträge), F5 (ENOSPC mitten im Frame ⇒ Rollback oder Poison, Store bleibt öffnenbar), F6 (EACCES ⇒ Fehler statt leer).

### WP-S3: Orphan-Politik (F7, INV-M5)

`RecoveryPolicy::unmanifested_sst` mit Varianten `Quarantine` (Default), `Fail` (Profil Regulated), `Adopt`:

- **Quarantine:** Datei nach `<data>/quarantine/<name>.<ts>` verschieben (`rename`, `sync_dir`), Metrik `store_orphan_sst_total`, Eintrag in `BootReport.orphans`. Nie löschen.
- **Fail:** `StoreError::UnmanifestedSstable{names}` beim Öffnen.
- **Adopt (nur explizit, Reparaturwerkzeug):** Footer/CRC prüfen, `max_seq` gegen Manifest-Zustand plausibilisieren, dann per `Add` übernehmen.
- Dead-Set-Dateien dürfen weiter automatisch gelöscht werden (durch Manifest bewiesen).
- Gilt auch bei fehlendem Manifest, aber vorhandenen SSTables (heute: alle SSTs werden übernommen, dann `Add`-Batch; bleibt, wird aber im `BootReport` ausgewiesen).

**Akzeptanz:** Test, der eine SST ohne Manifest-Eintrag ablegt: Default ⇒ Quarantäne + Report; `Fail` ⇒ Fehler; Datei existiert danach in jedem Fall noch.

### WP-S4: Katalog-Serialisierung und kurze Reader-Sperre (F2, F9, INV-C1/C2)

- Neuer Mutex `catalog_mutex` (`tokio::sync::Mutex<()>`, **Rang 350**) in `lock_order.rs` (zwischen `flush_mutex` 300 und `state` 400). Er wird gehalten von: Flush-Publish (Manifest-`Add` + RAM-Swap), Compaction-Publish, Rollback, Erase-Purge-Publish. Nicht gehalten während des Aufbaus der Output-SSTable (lange I/O).
- **Compaction-Publish neu:** unter `catalog_mutex` (a) Kontiguität prüfen (jetzt race-frei, weil alle Katalogänderer serialisiert sind; der „abort and retry“-Pfad wird zum `debug_assert`), (b) `manifest.append(Replace)` **ohne** `sstables`-Sperre, (c) `sstables.write()` nur für `retain`/`insert`/`sort` (RAM), (d) Sperre loslassen, (e) Dateien über `pending_delete` entsorgen.
- **Output-Aufräumen:** `OutputGuard` (Drop entfernt die Datei per synchronem `vfs.remove_file`, `commit()` entschärft ihn). Bei Fehler vor Manifest-`Replace` ist die Output-Datei nie ein Leck.
- Rollover wird intern vom Manifest ausgelöst (WP-S2.4); die Compaction kennt kein Rollover mehr.

**Akzeptanz:** Loom-Modell `loom_catalog_publish` (Flush ∥ Compaction ∥ Rollover, Invariante: Manifest-Live-Set == RAM-Katalog nach jedem Interleaving). Messung: Haltedauer `sstables.write()` im Compaction-Test < 100 µs (Histogramm-Gate, Muster `blocking_util_pool_measurement`).

### WP-S5: WAL-Fehlerpfade typisieren (F8)

- `PoisonCause` wird aus `IoFault` abgeleitet (`PartialWrite → TornWrite`, `SyncFailed → FsyncFailed`, …). `to_string().contains(...)` entfällt.
- Fault-Injection wandert vom globalen `static` in den `Vfs` (`FaultVfs { plan: FaultPlan }`, instanzgebunden, deterministisch, mit Seed). Features `fault-injection` bleibt nur für den `FaultVfs`-Export.
- WAL nutzt `AppendFile` (WP-S2.3); **\[?\]** Prüfauftrag: heutiges Verhalten bei Teilschreibung im WAL (Poison + Truncate auf letzten gültigen Frame beim nächsten Open?). `rg "TornWrite" crates/contextra-store/src/wal`; `wal_open_heal.rs` und `wal_torn_write_fault_injection.rs` sind die vorhandenen Referenztests.
- `let _ = remove_file(tmp)` in `wal/hmac.rs` wird zu `cleanup_tmp(path).await` mit `tracing::warn` bei Fehler (heute still).

### WP-S6: Typisierung und Format-Hygiene (F11)

`SstName(String)` (validiert: kein Separator, endet auf `.sst`), `SstId(u64)` für Metriken. Alle `as u32`/`as u64`-Konversionen im Format über `try_from`; Workspace-Lint `cast_possible_truncation` im Store-Crate auf `deny` anheben. Pfade werden nur an der Grenze zu `StoreIo` zusammengesetzt (`data_dir.join(name)`), nie im Manifest gespeichert.

### WP-S7: Wartbarkeit (F10, F12)

1. `#![allow(unexpected_cfgs)]` aus `lib.rs` entfernen; verbleibende cfgs in `check-cfg` des Workspace eintragen.
2. Jedes `#[allow(dead_code)]`/`#[allow(unused)]` im Produktionscode löschen oder mit Begründung `// allow: <ADR/Issue>` versehen; CI-Grep erzwingt das.
3. **Rang-Prüfung zur Laufzeit** (nur `debug_assertions`): `RankedMutex<T>`/`RankedRwLock<T>` mit `tokio::task_local!`-Stapel der gehaltenen Ränge; `lock()` bricht per `debug_assert!` bei absteigendem Rang. Ersetzt die rein dokumentarische Tabelle.
4. Doku-Korrektur: CRC-Angabe in `lib.rs`/`erasure/mod.rs` an die tatsächliche Wahl (WP-S2.1) anpassen; Doc-Gate `doc-truth` prüft den Text.
5. Doppelte `CONTEXTRA_INTERFACE_SPECIFICATION*.md`: `diff` prüfen, auf eine Datei reduzieren.
6. Feature-Unification: CI-Job `cargo tree -e features -p contextra` darf in Release-Konfiguration weder `test-hooks`, `fault-injection` noch `memory-only-storage` enthalten (Ergänzung zum vorhandenen Test `no_test_hooks_in_release_api`).

## 5. Teststrategie (WP-S8)

- **Crash-Matrix über allen I/O-Grenzen.** Mit `CrashVfs` (existiert in `contextra-testkit`) für **WAL und Manifest**: für jede Operationsnummer N einer festen Szenario-Sequenz (Puts, Flush, Compaction, Rollover, Rollback) bricht der VFS nach der N-ten Mutationsoperation ab, danach Recovery. Prüfung: (a) jeder bestätigte Commit ist vorhanden (Präfix-Eigenschaft), (b) kein gelöschter Schlüssel taucht auf, (c) Store öffnet immer, (d) `BootReport.orphans` ist leer, außer eine SST ist bewusst ungelistet. Erweitert `chaos_power_cut.rs`/`wal_lifecycle_crash_point_matrix.rs` um Manifest-Rollover und Katalog-Publish.
- **Referenzmodell.** `proptest_model_based.rs` um die Operationen `Rollover`, `CompactNow`, `FlushNow`, `Crash+Recover` in beliebiger Reihenfolge erweitern; Orakel ist eine einfache `BTreeMap`-Historie.
- **Deterministische Regressionstests je Befund:** `manifest_rollover_keeps_wal_hwm` (F1), `manifest_rollover_concurrent_flush` (F2, Barrier statt Sleep), `manifest_rollover_post_rename_fault_poisons` (F4), `manifest_append_enospc_midframe` (F5), `manifest_load_eacces_is_error` (F6), `recovery_unmanifested_sst_quarantined` (F7).
- **Loom:** `loom_catalog_publish` (WP-S4). Muster der vorhandenen `loom_*`-Tests.
- **Fuzz:** `cargo-fuzz`-Target für `Manifest::load` und für den WAL-Frame-Decoder (Eingabe: beliebige Bytes; Ziel: kein Panic, kein OOM, entweder `Ok` mit Präfix oder `Err`).
- **Mutation:** `mutants-guarantees.toml` um die neuen Garantien (INV-M1…M5, INV-C1) ergänzen.

## 6. Rollout und Risiken

| Schritt | Inhalt | Risiko / Gegenmaßnahme |
| --- | --- | --- |
| 1 | WP-S1 (Port) und WP-S8-Grundgerüst (CrashVfs für WAL/Manifest) | Verhalten unverändert; die Crash-Matrix wird zuerst **rot** und zeigt F1/F4/F5 reproduzierbar. Erst dann Fixes. |
| 2 | WP-S2 + WP-S3 als ein PR-Paar | Formatwechsel ⇒ Migrationstest, `MANIFEST.v1.bak`, Release-Note „kein Downgrade“. |
| 3 | WP-S4 | Neuer Lock-Rang ⇒ Loom-Modell vor dem Merge. |
| 4 | WP-S5, S6, S7 | Mechanisch; je Paket kompilierbar und `dead_code`-frei. |

**Hotfix-Option vor dem großen Umbau (klein, sofort):** in `compaction/engine.rs` den Rollover-Aufruf temporär entfernen oder per Konfiguration deaktivieren (F1/F2 verschwinden, Manifest wächst nur), und in `recovery.rs` den Orphan-Zweig von „skipping“ auf Quarantäne umstellen (F7). Beides ist ohne Formatänderung möglich.

## 7. Blinde Flecken und offene Prüfaufträge

- **Nicht verifiziert:** Kompilierbarkeit und Testlauf. AGENTS.md nennt für die Toolchain den Umweg über conda-forge; dort ist sie nicht installiert. Alle Befunde sind daher Leseergebnisse. **F2** und die Erreichbarkeit des Rollover-Fensters sind der erste Kandidat für einen Reproduktionstest.
- **Plattform:** Semantik von `rename` über geöffnete Dateien und von `sync_dir` unter Windows (`contextra-sys/src/durability.rs`, `acl_win32.rs`) ist nicht geprüft; WAL/Manifest-Rollover unter Windows ist ein eigener Prüfpunkt. Verhalten auf Netzwerk-/Overlay-Dateisystemen (fsync-Garantien) ebenso.
- **Ungelesene Module** (mögliche weitere Lücken): `sstable/reader.rs` (Bounds, Bloom-FPR, Cache-Poisoning), `block_cache.rs` (Eviction unter Last), `memtable.rs` (Shard-Verteilung, Speicherzählung), `tenant_codec.rs` (900 Zeilen, Schlüsselpräfix-Isolation), `wal/replay.rs` (HMAC-Ketten-Randfälle), `erasure/*` (Journal/Gate/Purge; eigenes Prüfgerüst in `docs/erasure/03`).
- **Ressourcenfehler:** ENOSPC-Reserve für Manifest und WAL (heute nur für das Erasure-Journal vorab reserviert), FD-Erschöpfung beim Rollover, Verhalten bei `EINTR`/teilweisen `write`s im `VfsFile`-Adapter.
- **Zeit/Zufall:** `clippy.toml` verbietet `Instant::now`/`SystemTime::now` (P28). Dass Quarantäne-Zeitstempel (WP-S3) über den `Clock`-Port laufen müssen, ist zu beachten.
