# `contextra-durable-fs` – Implementierungsspezifikation

|Feld|Wert|
|---|---|
|Zweck-Satz (D-97)|Dateisystem-Implementierung `RealVfs` mit dauerhaften Schreib- und Austauschoperationen.|
|Ring|1|
|Erlaubte Workspace-Abhängigkeiten|`types`, `ports`, `sys` (SPEC 5.3); Verbraucher: `engine`, Fassade `contextra` (nur Verdrahtung, D-88/D-95)|
|Stufe / Gates|S0; G-B (ein `Vfs`), G-F (Startprobe)|
|Arbeitspakete|WP-1.4, WP-5.2|
|SPEC-Anker|8.5, D-04, D-50, D-55, D-56, D-65, D-83, FI-F03, 14.1, 14.7, P14|

## 1. Ist-Zustand (HEAD `1454bbec0`) [V]

|Punkt|Befund|
|---|---|
|Umfang|1.677 Zeilen: `lib.rs` (507), `vfs.rs` (286), `scrub.rs` (320), `health.rs` (214), `journal.rs` (210), `fault_injector.rs` (140)|
|`Vfs`|`RealVfs` implementiert `ports::Vfs` (F-29); das zweite Trait ist entfernt. `pub use vfs::{…, Vfs}` re-exportiert das Port-Trait.|
|Hilfsfunktionen|`sync_dir`, `atomic_replace`, `durable_remove`, `scrub_and_remove`, `DurableHandle` (vergiftet sich bei I/O- oder `fsync`-Fehler, `INV-FSYNC-1`), `DirLock` (verbietet Locking auf Netzlaufwerken/FUSE), `probe_platform_durability`, `RealFs`|
|Weitere|`PendingDeleteJournal` (Journal ausstehender Datei-Löschaufträge), `AtomicHealthState`/`HealthState`, `FaultInjector`/`NoFault` (Trait für Fehlerinjektion in Produktionspfaden), `scrub_if_safe`, `ScrubPolicy`|
|Eingehende Kanten|**keine** (kein Crate nutzt `durable-fs`; F-34): `store` nutzt `std::fs` (171 Treffer), `vector` 54|
|Dokumentationskopf|nennt „Stand SPEC 4.1“ und „keinen anderen Crates“ (D-91 verbietet handgepflegte Verbrauchsangaben)|
|Abhängigkeiten|`contextra-ports`, `contextra-sys`|

## 2. Verantwortung und Abgrenzung

`durable-fs` ist die **einzige** Implementierung des Ports `Vfs` im Produktionsbaum (`RealVfs`). Aufgaben:

- Datei öffnen, positionelles Lesen und Schreiben, `sync_data`/`sync_all`/`sync_dir`, atomarer Austausch (tmp → `fsync` → `rename` → Verzeichnis-`fsync`),
- exklusiver Verzeichnis-Lock (D-50),
- Startprobe des Dateisystems (D-56),
- optionales `scrub` vor dem `unlink`, **nur** nach D-83,
- `fsync`-Fehler unverändert und **fatal je Handle** nach oben (D-55).

Nicht hier: Format- oder Schlüssellogik, Löschsaga, Journal der Löschung (liegt in `erasure`), Gesundheitsstatus des Stores (`engine`), Fehlerinjektion (liegt in `testkit::FaultVfs`).

## 3. Zielstruktur (Module) [E]

|Modul|Inhalt|
|---|---|
|`real_vfs`|`RealVfs: Vfs`, `RealFile: VfsFile` (Handle mit Zustand `Healthy`/`Poisoned`)|
|`atomic`|`write_atomic` (tmp im selben Verzeichnis, `sync_data`, `rename`, `sync_dir`)|
|`dir_lock`|`DirLock` (exklusiv, `LOCK`-Datei, `flock`-Semantik über `sys`)|
|`probe`|Startprobe: `statfs`-Typ (über `sys::fs_class`), atomares Rename, Verzeichnis-`fsync`, Atomaritätsprobe für Schreibvorgänge an ausgerichteten Offsets, Messung `fsync_p50`, Erkennung `rotational`/COW|
|`scrub`|`scrub_before_unlink` nur bei `rotational == true` und nicht-COW (D-83)|

Entfallen oder wandern: `fault_injector.rs` (nach `testkit`), `health.rs` (Zustand lebt je Handle und im Store), `journal.rs` (`PendingDeleteJournal` dupliziert Journal-Zweck, P3, prüfen), `RealFs` (zweite Fassade).

## 4. Schnittstellen [E]

Die Form folgt dem Port in Datei 02 (async `Vfs`). Zusätzlich öffentlich:

```rust
pub struct RealVfs { /* Runtime-Handle für spawn_blocking, ProbeResult */ }
impl RealVfs { pub fn new(rt: tokio::runtime::Handle) -> Self; }

pub struct FsProbe {
    pub fs: FsClass,                   // aus sys::fs_class
    pub rename_atomic: bool,
    pub dir_fsync_ok: bool,
    pub write_atomic_64: bool,         // L-4: zusätzlich Probe auf 80-Byte-Slots
    pub fsync_p50: Duration,           // Grundlage von D-30: ≤ 6 × fsync_p50 + 2 ms
    pub rotational: Option<bool>,
}
pub enum ProbeVerdict { Supported(Tier), Refused(UnsupportedFilesystem) }
```

`Tier` ∈ {`Tier1`, `Tier2`}; `Tier2` führt `FsJournalOrCow` immer im Bericht. `Refused` bricht den Start mit `Config::UnsupportedFilesystem` ab (T-PLAT-1).

## 5. Implementierungsvorgaben

|ID|Vorgabe|Quelle|Test|
|---|---|---|---|
|DF-01|`RealVfs` implementiert **das** `ports::Vfs`; im Crate existiert kein zweites `Vfs`-Trait.|D-04|T-LINT-2|
|DF-02|Atomarer Austausch: Zieldatei nie halb geschrieben sichtbar: temporäre Datei im **selben Verzeichnis** schreiben, `sync_data`, `rename`, Verzeichnis-`sync`. Jeder Schritt meldet Fehler unverändert.|FI-F01, 8.5|T-CRASH-2|
|DF-03|Ein `fsync`-Fehler (`sync_data`, `sync_all`, `sync_dir`) macht das Handle **dauerhaft fatal**: jede weitere Operation auf diesem Handle liefert `Io(Poisoned)`; es gibt keinen Wiederholungsversuch. Neuöffnen ist Aufgabe des Aufrufers (Löschpfad: `Pending` mit Neuöffnen und Prüfung).|D-55, I-10|T-FSYNC-1|
|DF-04|Blockierende Systemaufrufe laufen über `spawn_blocking`; kein `tokio::fs`, kein `block_on`. Tokio-Features nur `rt-multi-thread`, `sync`, `time`, `macros`.|D-03, D-54|T-LINT-2, T-ARCH-4|
|DF-05|Exklusiver Verzeichnis-Lock beim Öffnen; ein zweiter Prozess scheitert mit `Io`/`Config` (fester Text). Lock-Datei `LOCK` im Datenverzeichnis.|D-50|T-LOCK-1|
|DF-06|Startprobe prüft: `statfs`-Typ (Tier 1: ext4, XFS auf Linux x86_64/aarch64; Tier 2: APFS, btrfs, ZFS), atomares Rename, Verzeichnis-`fsync`, Atomarität für Schreibvorgänge an ausgerichteten Offsets (64 Byte), `fsync_p50` (Stichprobe). NFS, SMB, FUSE, overlayfs, Windows und unbekannte Typen ⇒ `UnsupportedFilesystem`.|D-56, FI-F03|T-PLAT-1|
|DF-07|`scrub` (Überschreiben vor `unlink`) nur, wenn die Probe `rotational = true` **und** nicht-COW meldet; sonst entfällt es (nicht tragend nach D-85). Kosten gehen getrennt in BM-ERASE-WA ein.|D-83|BM-ERASE-WA, T-CFG-3|
|DF-08|Dateimodi: Verzeichnisse 0700, Dateien 0600 bei Anlage; `tmp/` wird beim Start geleert (Waisen).|5.6, 14.1|T-OPS-2, T-CRASH-2|
|DF-09|`remove_file` meldet fehlende Dateien als Erfolg nur dort, wo Idempotenz gebraucht wird (`unlink` im Purge, I-15); alle anderen Fehler unverändert. Nach `unlink` folgt `sync_dir` des Elternverzeichnisses.|I-15, 11.4|T-CRASH-6|
|DF-10|Plattenreserve: `statvfs`-Information (freier Platz) über `Vfs` abrufbar, damit der Store `Limit::DiskReserve` entscheiden kann (D-65).|D-65|T-OPS-2, T-CRASH-3|
|DF-11|Keine Panics; Fehlertexte fest, ohne Pfad im `Display` (HR-4). Pfade stehen nur in `tracing`-Feldern mit Hash, nicht im Klartext.|P6, HR-4|T-LOG-1|
|DF-12|Der Verzeichnisname und alle Dateinamen enthalten nur Tags, `file_id`, `erasure_id` oder feste Namen; `RealVfs` selbst erzeugt keine Namen.|D-75, D-09|T-RAW-6|

## 6. Rückbau und Migration

|Position|Maßnahme|WP|Nachweis|
|---|---|---|---|
|`vfs.rs` Rest (zweites Trait, `RealFs`)|auf `ports::Vfs` (async) umstellen; `RealFs` entfernen|WP-1.4|T-LINT-2|
|`fault_injector.rs`, Parameter `FaultInjector`/`NoFault`|nach `testkit::FaultVfs` verlagern; Produktionscode trägt keinen Fehlerinjektor (D-84: `fault-injection` entfällt)|WP-1.4|T-CFG-1|
|`health.rs` (`AtomicHealthState`)|entfällt; Handle-Poisoned (DF-03) und Store-`Poisoned` ersetzen es|WP-1.4|T-FSYNC-1|
|`journal.rs` (`PendingDeleteJournal`)|prüfen; Löschaufträge gehören in das Löschjournal (`erasure`, 11.6). Ohne Kernkonsument entfernen.|WP-1.4|Grep|
|`scrub.rs`|auf D-83-Bedingung (`rotational` und nicht-COW) umstellen; `ScrubPolicy`-Varianten reduzieren|WP-5.2|T-CFG-3|
|Eingehende Kanten|`store`, `vector`, `text` nutzen `Vfs` aus `ports`; `engine`/`contextra` erzeugen `RealVfs` (D-88)|WP-1.4|T-ARCH-3|
|Kopfzeilen mit „Stand SPEC 4.1“, Verbraucherliste|Q-01, Q-09|WP-1.5|T-ARCH-5, T-LINT-4|

## 7. Arbeitspakete

1. WP-1.4 (rot): Contract-Tests für `Vfs` im `testkit` (Datei anlegen, `write_at`/`read_at`, atomarer Austausch, `rename`, `list_dir`, Lock). `FaultVfs`-Crash-Matrix für `write_atomic`.
2. WP-1.4 (grün): Port async umstellen, `RealVfs` auf `spawn_blocking`, Handle-Zustandsautomat (DF-03).
3. WP-5.2: Startprobe (DF-06), `scrub`-Bedingung (DF-07), freier Platz (DF-10).
4. Evidenz für G-F: Probe-Protokoll auf ext4, XFS, btrfs, ZFS, APFS; Negativfälle (tmpfs mit `overlay`, FUSE) in Containertests.

## 8. Tests und Benchmarks

|ID|Prüft|
|---|---|
|T-FSYNC-1|`fsync`-Fehler ⇒ Handle fatal, nie Wiederholung|
|T-CRASH-2, T-CRASH-6|Crash an jedem `write`/`fsync`/`rename`/`unlink` im Austausch; Neuöffnen räumt Waisen|
|T-LOCK-1|zweiter Prozess scheitert am Lock|
|T-PLAT-1|Startprobe verweigert unbekannte/nicht unterstützte Dateisysteme|
|T-OPS-2|freier Platz, Plattenreserve|
|T-CFG-3|`batch_window ≤ erase_sla/2` (Konfigurationsprüfung, nutzt `fsync_p50` aus der Probe)|
|T-RAW-6|keine Klartextnamen in Datei- und Verzeichnisnamen|
|BM-ERASE-LAT (Beitrag)|`fsync`-Kosten: 4 `fsync` im Normalpfad (D-58)|

## 9. Abnahme

Definition of Done (Index Abschnitt 8); Grep `trait Vfs` im Repo = 1 (in `ports`); kein `std::fs`/`tokio::fs` außerhalb von `durable-fs`; Handle-Zustandsautomat per Property-Test (`Healthy → Poisoned`, nie zurück) belegt.

## 10. Offene Punkte

- L-1 (async `Vfs`), L-4 (Atomaritätsprobe auf 80-Byte-Slots erweitern).
- Ob die Probe `fsync_p50` bei jedem Start misst (Dauer) oder aus einer zwischengespeicherten Messung ableitet, legt WP-5.2 fest; D-30 verlangt nur, dass die Schwelle relativ zum gemessenen Wert gilt.
