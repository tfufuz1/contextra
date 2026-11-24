# Schnittstellenspezifikation: `contextra-durable-fs`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-durable-fs` |
| Version | workspace |
| Pfad | `crates/contextra-durable-fs` |
| Rolle | Dauerhafte Dateioperationen (alt) |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> # Dauerhafte Dateisystem-Operationen und synchrone Disk-Flushes (`contextra-durable-fs`)
>
> Autarker Baustein der Klasse A (B1–B12) für crash-sichere Dateioperationen,
> synchrones fsync-Poisoning (`INV-FSYNC-1`) und atomare Dateieretzungen (`INV-VFS-ATOMIC-1`).
>
> ## Invarianten
> - `INV-FSYNC-1`: Schlägt ein Flush/fsync fehl, wird das Handle unwiderruflich vergiftet.
> Keine nachfolgende Schreiboperation wird auf demselben Deskriptor ausgeführt.
> - `INV-VFS-ATOMIC-1`: Atomares Schreiben (`write_atomic`) erfolgt strikt über temporäre Dateien
> im selben Zielverzeichnis, gefolgt von `sync_file`, atomarem Rename und Verzeichnis-Sync.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `crc32fast` | normal | registry | ja |  |
| `criterion` | dev | workspace | nein |  |
| `proptest` | dev | registry | nein |  |
| `serde` | normal | workspace | ja |  |
| `tempfile` | dev | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tracing` | normal | workspace | ja |  |

**Verwendet von (Crate-Ebene):** –

## 3. Features

| Feature | Aktiviert |
|---|---|
| `crc` | `dep:crc32fast` |
| `default` | `std`, `crc` |
| `inspect` | – |
| `serde` | `dep:serde` |
| `std` | – |
| `tracing` | `dep:tracing` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 8 |
| Quellzeilen | 1910 |
| Öffentliche Items (Modul-Ebene) | 56 |
| davon `fn` | 18 |
| davon `struct` | 12 |
| davon `enum` | 12 |
| davon `trait` | 2 |
| davon `type` | 3 |
| davon `const` | 2 |
| davon `mod` | 7 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 7 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 8 |
| `#[deprecated]` | 0 |
| `#[cfg(feature…)]`-Gates | 1 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 95 / 95 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `error` | `crates/contextra-durable-fs/src/error.rs` | 73 |
| `fault_injector` | `crates/contextra-durable-fs/src/fault_injector.rs` | 300 |
| `health` | `crates/contextra-durable-fs/src/health.rs` | 139 |
| `journal` | `crates/contextra-durable-fs/src/journal.rs` | 436 |
| `crate` | `crates/contextra-durable-fs/src/lib.rs` | 455 |
| `platform` | `crates/contextra-durable-fs/src/platform.rs` | 48 |
| `scrub` | `crates/contextra-durable-fs/src/scrub.rs` | 318 |
| `vfs` | `crates/contextra-durable-fs/src/vfs.rs` | 141 |

## 6. Re-Exports (`pub use`)

- `pub use error::` — `crates/contextra-durable-fs/src/lib.rs:32` (Modul `crate`)
- `pub use fault_injector::` — `crates/contextra-durable-fs/src/lib.rs:33` (Modul `crate`)
- `pub use health::` — `crates/contextra-durable-fs/src/lib.rs:34` (Modul `crate`)
- `pub use journal::` — `crates/contextra-durable-fs/src/lib.rs:35` (Modul `crate`)
- `pub use platform::FlushKind` — `crates/contextra-durable-fs/src/lib.rs:38` (Modul `crate`)
- `pub use scrub::` — `crates/contextra-durable-fs/src/lib.rs:39` (Modul `crate`)
- `pub use vfs::` — `crates/contextra-durable-fs/src/lib.rs:43` (Modul `crate`)

## 8. Öffentliche Schnittstelle je Item

### Modul `crate`

#### `mod error`
- Ort: `crates/contextra-durable-fs/src/lib.rs:18`
```rust
pub mod error
```
Dokumentation: Fehlertypen und Result-Definitionen für Dateisystemoperationen.

#### `mod fault_injector`
- Ort: `crates/contextra-durable-fs/src/lib.rs:20`
```rust
pub mod fault_injector
```
Dokumentation: Fehlerinjektor und VFS-Dekorator zur Simulation von Ausfällen und Crash-Punkten.

#### `mod health`
- Ort: `crates/contextra-durable-fs/src/lib.rs:22`
```rust
pub mod health
```
Dokumentation: Fail-Closed-Zustandsverfolgung und Poisoning-Zustandsautomat (§18.4).

#### `mod journal`
- Ort: `crates/contextra-durable-fs/src/lib.rs:24`
```rust
pub mod journal
```
Dokumentation: Append-only-Journal mit Prüfsummen und Pending-Delete-Journal für Crash-Resilienz.

#### `mod platform`
- Ort: `crates/contextra-durable-fs/src/lib.rs:26`
```rust
pub mod platform
```
Dokumentation: Plattformspezifische synchrone Flush-Primitive für Dateien und Verzeichnisse.

#### `mod scrub`
- Ort: `crates/contextra-durable-fs/src/lib.rs:28`
```rust
pub mod scrub
```
Dokumentation: Sicheres Überschreiben (Scrubbing) und Integritätsprüfung mit Ratenbegrenzung.

#### `mod vfs`
- Ort: `crates/contextra-durable-fs/src/lib.rs:30`
```rust
pub mod vfs
```
Dokumentation: Virtuelles Dateisystem (VFS) mit atomaren Ersetzungen und synchroner Schnittstelle.

#### `struct DirLock`
- Ort: `crates/contextra-durable-fs/src/lib.rs:256`
```rust
pub struct DirLock
```
Dokumentation: RAII-Guard für eine Verzeichnissperre (`.dirlock`).
- Impl-Block `crates/contextra-durable-fs/src/lib.rs:262`
- Impl-Block (Trait `Drop`) `crates/contextra-durable-fs/src/lib.rs:308`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn acquire(dir: &Path, mode: LockMode) -> Result<Self>` | `crates/contextra-durable-fs/src/lib.rs:264` | Erwirbt eine Sperre auf `dir` im angegebenen [`LockMode`]. |
| `pub fn mode(&self) -> LockMode` | `crates/contextra-durable-fs/src/lib.rs:303` | Liefert den aktiven [`LockMode`] dieser Verzeichnissperre. |

#### `struct DurableHandle`
- Ort: `crates/contextra-durable-fs/src/lib.rs:129`
```rust
pub struct DurableHandle<F: FaultInjector = NoFault>
```
Dokumentation: Ein Wrapper um [`File`], der das Handle bei einem `fsync`- oder Schreibfehler dauerhaft vergiftet (`INV-FSYNC-1`).
- Impl-Block `crates/contextra-durable-fs/src/lib.rs:136`
- Impl-Block `crates/contextra-durable-fs/src/lib.rs:143`
- Impl-Block (Trait `Write`) `crates/contextra-durable-fs/src/lib.rs:220`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(file: File) -> Self` | `crates/contextra-durable-fs/src/lib.rs:138` | Kapselt eine geöffnete [`File`] in ein neues [`DurableHandle`] mit standardmäßigem [`NoFault`]. |
| `pub fn new_with_injector(file: File, injector: F) -> Self` | `crates/contextra-durable-fs/src/lib.rs:145` | Kapselt eine geöffnete [`File`] mit einem benutzerdefinierten [`FaultInjector`]. |
| `pub fn new_with_path_and_injector(file: File, path: PathBuf, injector: F) -> Self` | `crates/contextra-durable-fs/src/lib.rs:155` | Kapselt eine geöffnete [`File`] mit zugehörigem Pfad und benutzerdefiniertem [`FaultInjector`]. |
| `pub fn is_poisoned(&self) -> bool` | `crates/contextra-durable-fs/src/lib.rs:166` | Prüft, ob dieses Handle aufgrund eines vorangegangenen Flush-Fehlers vergiftet ist (`INV-FSYNC-1`). |
| `pub fn flush(&mut self, kind: FlushKind) -> Result<()>` | `crates/contextra-durable-fs/src/lib.rs:171` | Flusht Dateiinhalte synchron auf die Festplatte und führt Fehlerinjektions-Hooks aus. |
| `pub fn get_ref(&self) -> Option<&File>` | `crates/contextra-durable-fs/src/lib.rs:197` | Liefert eine unveränderliche Referenz auf die zugrundeliegende Datei, oder `None`, falls das Handle vergiftet ist. |
| `pub fn get_mut(&mut self) -> Option<&mut File>` | `crates/contextra-durable-fs/src/lib.rs:206` | Liefert eine veränderliche Referenz auf die zugrundeliegende Datei, oder `None`, falls das Handle vergiftet ist. |
| `pub fn poison(&mut self)` | `crates/contextra-durable-fs/src/lib.rs:215` | Vergiftet dieses Handle explizit manuell. |

#### `struct PoisonedHandle`
- Ort: `crates/contextra-durable-fs/src/lib.rs:125`
```rust
pub struct PoisonedHandle
```
Dokumentation: Markierungstyp zur Kennzeichnung eines durch `fsync`- oder Schreibfehler vergifteten Handles.

#### `enum FsClass`
- Ort: `crates/contextra-durable-fs/src/lib.rs:53`
```rust
pub enum FsClass
```
Dokumentation: Dateisystemklassifikation für Persistenz- und Locking-Garantien (§23.4).
- Impl-Block `crates/contextra-durable-fs/src/lib.rs:66`

Varianten:

- `LocalSupported` — `crates/contextra-durable-fs/src/lib.rs:55`
- `Network` — `crates/contextra-durable-fs/src/lib.rs:57`
- `Fuse` — `crates/contextra-durable-fs/src/lib.rs:59`
- `CloudSync` — `crates/contextra-durable-fs/src/lib.rs:61`
- `Unknown` — `crates/contextra-durable-fs/src/lib.rs:63`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn is_supported_local(&self) -> bool` | `crates/contextra-durable-fs/src/lib.rs:69` | Liefert `true`, wenn das Dateisystem vollständige lokale Persistenz und Locking unterstützt. |

#### `enum LockMode`
- Ort: `crates/contextra-durable-fs/src/lib.rs:246`
```rust
pub enum LockMode
```
Dokumentation: Sperrmodus für Verzeichnissperren gemäß Spezifikation §7.3.1.

Varianten:

- `Normal` — `crates/contextra-durable-fs/src/lib.rs:249`
- `MemoryOnly` — `crates/contextra-durable-fs/src/lib.rs:251`

#### `fn atomic_replace`
- Ort: `crates/contextra-durable-fs/src/lib.rs:330`
```rust
pub fn atomic_replace(path: &Path, bytes: &[u8]) -> Result<()>
```
Dokumentation: Ersetzt den Inhalt der Datei unter `path` atomar mit `bytes` unter Verwendung des Standardinjektors [`NoFault`].

#### `fn atomic_replace_with_injector`
- Ort: `crates/contextra-durable-fs/src/lib.rs:335`
```rust
pub fn atomic_replace_with_injector<F: FaultInjector>( path: &Path, bytes: &[u8], injector: &F, ) -> Result<()>
```
Dokumentation: Ersetzt den Inhalt der Datei unter `path` atomar mit `bytes` unter Verwendung eines [`FaultInjector`] (`INV-VFS-ATOMIC-1`).

#### `fn durable_remove`
- Ort: `crates/contextra-durable-fs/src/lib.rs:382`
```rust
pub fn durable_remove(path: &Path) -> Result<()>
```
Dokumentation: Entfernt eine Datei dauerhaft und flusht das Elternverzeichnis mit Standardinjektor [`NoFault`].

#### `fn durable_remove_with_injector`
- Ort: `crates/contextra-durable-fs/src/lib.rs:387`
```rust
pub fn durable_remove_with_injector<F: FaultInjector>(path: &Path, injector: &F) -> Result<()>
```
Dokumentation: Entfernt eine Datei dauerhaft und flusht das Elternverzeichnis mit benutzerdefiniertem [`FaultInjector`].

#### `fn flush_dir`
- Ort: `crates/contextra-durable-fs/src/lib.rs:80`
```rust
pub fn flush_dir(dir: &Path) -> Result<()>
```
Dokumentation: Flusht Verzeichniseinträge auf stabile Speichermedien.

#### `fn flush_file`
- Ort: `crates/contextra-durable-fs/src/lib.rs:75`
```rust
pub fn flush_file(file: &File, kind: FlushKind) -> Result<()>
```
Dokumentation: Flusht eine geöffnete Datei auf die Festplatte.

#### `fn fs_class`
- Ort: `crates/contextra-durable-fs/src/lib.rs:98`
```rust
pub fn fs_class(path: &Path) -> Result<FsClass>
```
Dokumentation: Ermittelt die Dateisystemklasse für den angegebenen Pfad (§23.4).

#### `fn rename_durable`
- Ort: `crates/contextra-durable-fs/src/lib.rs:113`
```rust
pub fn rename_durable(from: &Path, to: &Path) -> Result<()>
```
Dokumentation: Benennt eine Datei atomar um und flusht das übergeordnete Verzeichnis.

#### `fn scrub_and_remove`
- Ort: `crates/contextra-durable-fs/src/lib.rs:399`
```rust
pub fn scrub_and_remove(path: &Path) -> Result<ScrubReceipt>
```
Dokumentation: Überschreibt eine Datei vollständig mit Nullen und entfernt sie dauerhaft (mit [`NoFault`]).

#### `fn scrub_and_remove_with_injector`
- Ort: `crates/contextra-durable-fs/src/lib.rs:404`
```rust
pub fn scrub_and_remove_with_injector<F: FaultInjector>( path: &Path, injector: &F, ) -> Result<ScrubReceipt>
```
Dokumentation: Überschreibt eine Datei mit Nullen und entfernt sie dauerhaft unter Verwendung eines [`FaultInjector`].

#### `fn sync_dir`
- Ort: `crates/contextra-durable-fs/src/lib.rs:319`
```rust
pub fn sync_dir(dir: &Path) -> Result<()>
```
Dokumentation: Synchronisiert die Verzeichnismetadaten eines Ordners mit Standard-Injektor [`NoFault`].

#### `fn sync_dir_with_injector`
- Ort: `crates/contextra-durable-fs/src/lib.rs:324`
```rust
pub fn sync_dir_with_injector<F: FaultInjector>(dir: &Path, injector: &F) -> Result<()>
```
Dokumentation: Synchronisiert die Verzeichnismetadaten eines Ordners mit benutzerdefiniertem [`FaultInjector`].

### Modul `error`

#### `enum DurableFsError`
- Ort: `crates/contextra-durable-fs/src/error.rs:11`
```rust
pub enum DurableFsError
```
Dokumentation: Primärer Fehlertyp für alle dauerhaften Dateisystemoperationen (`contextra-durable-fs`).
- Impl-Block `crates/contextra-durable-fs/src/error.rs:47`

Varianten:

- `Io(#[from] std::io::Error)` — `crates/contextra-durable-fs/src/error.rs:14`
- `NotFound{` — `crates/contextra-durable-fs/src/error.rs:18`
- `PoisonedHandle` — `crates/contextra-durable-fs/src/error.rs:25`
- `InvalidArgument(String)` — `crates/contextra-durable-fs/src/error.rs:29`
- `Corruption(String)` — `crates/contextra-durable-fs/src/error.rs:33`
- `ScrubRefused(String)` — `crates/contextra-durable-fs/src/error.rs:37`
- `UnsupportedFilesystem(String)` — `crates/contextra-durable-fs/src/error.rs:41`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn kind(&self) -> std::io::ErrorKind` | `crates/contextra-durable-fs/src/error.rs:50` | Gibt den entsprechenden [`std::io::ErrorKind`] für Kompatibilität mit IO-Operationen zurück. |

#### `type Error`
- Ort: `crates/contextra-durable-fs/src/error.rs:45`
```rust
pub type Error = DurableFsError
```
Dokumentation: Kanonischer Alias `Error` gemäß Klasse-A-Vertrag (B4).

#### `type Result`
- Ort: `crates/contextra-durable-fs/src/error.rs:73`
```rust
pub type Result<T> = std::result::Result<T, DurableFsError>
```
Dokumentation: Einheitlicher Result-Typ für `contextra-durable-fs`.

### Modul `fault_injector`

#### `struct FaultInjectingVfs`
- Ort: `crates/contextra-durable-fs/src/fault_injector.rs:95`
```rust
pub struct FaultInjectingVfs<V: Vfs>
```
Dokumentation: VFS-Dekorator zur präzisen Fehlerinjektion und Crash-Punkt-Simulation.
- Impl-Block `crates/contextra-durable-fs/src/fault_injector.rs:105`
- Impl-Block (Trait `Vfs`) `crates/contextra-durable-fs/src/fault_injector.rs:166`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(inner: V) -> Self` | `crates/contextra-durable-fs/src/fault_injector.rs:107` | Kapselt ein bestehendes [`Vfs`] in einen Injektor-Dekorator. |
| `pub fn inject_write_disk_full(&self)` | `crates/contextra-durable-fs/src/fault_injector.rs:120` | Konfiguriert, dass der nächste Schreibaufruf mit `ENOSPC` fehlschlägt. |
| `pub fn inject_fsync_failure(&self)` | `crates/contextra-durable-fs/src/fault_injector.rs:125` | Konfiguriert, dass der nächste `sync_file`-Aufruf mit EIO fehlschlägt. |
| `pub fn inject_rename_failure(&self)` | `crates/contextra-durable-fs/src/fault_injector.rs:130` | Konfiguriert, dass das nächste `rename` fehlschlägt. |
| `pub fn inject_dir_fsync_failure(&self)` | `crates/contextra-durable-fs/src/fault_injector.rs:135` | Konfiguriert, dass der nächste Verzeichnis-Sync fehlschlägt. |
| `pub fn simulate_crash(&self) -> Result<()>` | `crates/contextra-durable-fs/src/fault_injector.rs:141` | Simuliert einen plötzlichen Prozessabsturz (Crash-Punkt-Simulation): Alle noch nicht synchronisierten Datenänderungen gehen unwiderruflich verloren! |

#### `struct NoFault`
- Ort: `crates/contextra-durable-fs/src/fault_injector.rs:34`
```rust
pub struct NoFault
```
Dokumentation: Null-Kosten-Standardinjektor für Produktivumgebungen.
- Impl-Block (Trait `FaultInjector`) `crates/contextra-durable-fs/src/fault_injector.rs:36`

#### `enum InjectedFault`
- Ort: `crates/contextra-durable-fs/src/fault_injector.rs:82`
```rust
pub enum InjectedFault
```
Dokumentation: Skriptbarer Fehlerzustand für Tests und Chaos-Simulation.

Varianten:

- `DiskFull` — `crates/contextra-durable-fs/src/fault_injector.rs:84`
- `IoError` — `crates/contextra-durable-fs/src/fault_injector.rs:86`
- `FsyncFailure` — `crates/contextra-durable-fs/src/fault_injector.rs:88`
- `RenameFailure` — `crates/contextra-durable-fs/src/fault_injector.rs:90`

#### `trait FaultInjector`
- Ort: `crates/contextra-durable-fs/src/fault_injector.rs:10`
```rust
pub trait FaultInjector: Send + Sync
```
Dokumentation: Schnittstelle zur Injektion von I/O-Fehlern in Persistenzpfade.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn on_write(&self, _path: Option<&Path>, _bytes: usize) -> io::Result<()>` | `crates/contextra-durable-fs/src/fault_injector.rs:12` | Hook vor oder während eines Schreibvorgangs. |
| `fn on_fsync(&self, _path: Option<&Path>) -> io::Result<()>` | `crates/contextra-durable-fs/src/fault_injector.rs:17` | Hook vor dem Hardware-Flush (`fsync`) einer Datei. |
| `fn on_rename(&self, _from: &Path, _to: &Path) -> io::Result<()>` | `crates/contextra-durable-fs/src/fault_injector.rs:22` | Hook vor einer atomaren Umbenennung (`rename`). |
| `fn on_dir_fsync(&self, _dir: &Path) -> io::Result<()>` | `crates/contextra-durable-fs/src/fault_injector.rs:27` | Hook vor dem Hardware-Flush (`fsync`) eines Verzeichnisses. |

### Modul `health`

#### `struct AtomicHealthState`
- Ort: `crates/contextra-durable-fs/src/health.rs:86`
```rust
pub struct AtomicHealthState
```
Dokumentation: Threadsicherer, atomarer Wrapper für den aktuellen [`HealthState`].
- Impl-Block (Trait `Default`) `crates/contextra-durable-fs/src/health.rs:90`
- Impl-Block `crates/contextra-durable-fs/src/health.rs:96`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-durable-fs/src/health.rs:99` | Initialisiert eine neue atomare Zustandsverwaltung im Startzustand `Healthy`. |
| `pub fn load(&self) -> HealthState` | `crates/contextra-durable-fs/src/health.rs:107` | Liest den aktuellen [`HealthState`] mit `Acquire`-Speicherordnung aus. |
| `pub fn record_event(&self, event: HealthEvent) -> HealthState` | `crates/contextra-durable-fs/src/health.rs:117` | Führt einen atomaren Zustandsübergang anhand eines [`HealthEvent`] aus. |

#### `enum HealthEvent`
- Ort: `crates/contextra-durable-fs/src/health.rs:73`
```rust
pub enum HealthEvent
```
Dokumentation: Ereignisse und Auslöser für Statusübergänge im Speichersystem.

Varianten:

- `FsyncFailure` — `crates/contextra-durable-fs/src/health.rs:75`
- `Enospc` — `crates/contextra-durable-fs/src/health.rs:77`
- `ReadChecksumWarning` — `crates/contextra-durable-fs/src/health.rs:79`
- `Recovered` — `crates/contextra-durable-fs/src/health.rs:81`

#### `enum HealthState`
- Ort: `crates/contextra-durable-fs/src/health.rs:18`
```rust
pub enum HealthState
```
Dokumentation: Einstufung des Systemzustands für das Speichersubstrat (§18.4).
- Impl-Block `crates/contextra-durable-fs/src/health.rs:30`

Varianten:

- `Healthy= 0` — `crates/contextra-durable-fs/src/health.rs:21`
- `Degraded= 1` — `crates/contextra-durable-fs/src/health.rs:23`
- `ReadOnly= 2` — `crates/contextra-durable-fs/src/health.rs:25`
- `Poisoned= 3` — `crates/contextra-durable-fs/src/health.rs:27`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new() -> Self` | `crates/contextra-durable-fs/src/health.rs:33` | Erzeugt einen neuen [`HealthState`] im Ausgangszustand `Healthy`. |
| `pub fn transition(self, event: HealthEvent) -> Self` | `crates/contextra-durable-fs/src/health.rs:39` | Berechnet den Folgezustand basierend auf einem eingehenden [`HealthEvent`]. |
| `pub fn allows_writes(self) -> bool` | `crates/contextra-durable-fs/src/health.rs:60` | Gibt an, ob im aktuellen Zustand mutierende Schreiboperationen zulässig sind. |
| `pub fn is_poisoned(self) -> bool` | `crates/contextra-durable-fs/src/health.rs:66` | Prüft, ob das Speichersubstrat dauerhaft vergiftet (`Poisoned`) ist. |

### Modul `journal`

#### `struct AppendOnlyJournal`
- Ort: `crates/contextra-durable-fs/src/journal.rs:71`
```rust
pub struct AppendOnlyJournal
```
Dokumentation: Universelles Append-only Journal mit Prüfsummen und Sequenznummern.
- Impl-Block `crates/contextra-durable-fs/src/journal.rs:80`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn open(path: impl Into<PathBuf>) -> Result<Self>` | `crates/contextra-durable-fs/src/journal.rs:82` | Öffnet oder erstellt ein Journal mit Standard-Sync-Politik ([`JournalSyncPolicy::Always`]). |
| `pub fn open_without_recovery( path: impl Into<PathBuf>, policy: JournalSyncPolicy, health: Arc<AtomicHealthState>, ) -> Result<Self>` | `crates/contextra-durable-fs/src/journal.rs:91` | Öffnet ein Journal ohne automatische Initial-Recovery (nützlich für manuelle Recovery). |
| `pub fn open_with_policy( path: impl Into<PathBuf>, policy: JournalSyncPolicy, health: Arc<AtomicHealthState>, ) -> Result<Self>` | `crates/contextra-durable-fs/src/journal.rs:116` | Öffnet oder erstellt ein Journal mit benutzerdefinierter Sync-Politik und Health-State. |
| `pub fn path(&self) -> &std::path::Path` | `crates/contextra-durable-fs/src/journal.rs:131` | Gibt den Pfad des Journals zurück. |
| `pub fn append(&mut self, payload: &[u8]) -> Result<u64>` | `crates/contextra-durable-fs/src/journal.rs:136` | Hängt einen neuen Payload-Record an das Journal an. |
| `pub fn flush(&mut self) -> Result<()>` | `crates/contextra-durable-fs/src/journal.rs:179` | Flusht ausstehende Journal-Schreibvorgänge synchron auf stabilen Speicher (`INV-FSYNC-1`). |
| `pub fn recover(&mut self, recovery_policy: RecordRecoveryPolicy) -> Result<Vec<JournalRecord>>` | `crates/contextra-durable-fs/src/journal.rs:194` | Führt die Recovery des Journals durch und liest gültige Records ein. |
| `pub fn health_state(&self) -> HealthState` | `crates/contextra-durable-fs/src/journal.rs:300` | Gibt den aktuellen Gesundheitszustand zurück. |

#### `struct JournalRecord`
- Ort: `crates/contextra-durable-fs/src/journal.rs:62`
```rust
pub struct JournalRecord
```
Dokumentation: Ein gelesener Journal-Eintrag.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `seq_no` | `u64` | `crates/contextra-durable-fs/src/journal.rs:64` |
| `payload` | `Vec<u8>` | `crates/contextra-durable-fs/src/journal.rs:66` |

#### `struct PendingDeleteJournal`
- Ort: `crates/contextra-durable-fs/src/journal.rs:307`
```rust
pub struct PendingDeleteJournal
```
Dokumentation: Journal zur dauerhaften Aufzeichnung von Entfernungsaufträgen über Prozessabstürze hinweg.
- Impl-Block `crates/contextra-durable-fs/src/journal.rs:313`
- Impl-Block (Trait `Drop`) `crates/contextra-durable-fs/src/journal.rs:428`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn open(journal_path: impl Into<PathBuf>) -> Result<Self>` | `crates/contextra-durable-fs/src/journal.rs:315` | Öffnet oder erstellt ein `PendingDeleteJournal` am angegebenen Pfad mit neuem Health-State. |
| `pub fn open_with_health( journal_path: impl Into<PathBuf>, health: Arc<AtomicHealthState>, ) -> Result<Self>` | `crates/contextra-durable-fs/src/journal.rs:320` | Öffnet oder erstellt ein `PendingDeleteJournal` mit einer geteilten [`AtomicHealthState`]-Instanz. |
| `pub fn health_state(&self) -> HealthState` | `crates/contextra-durable-fs/src/journal.rs:346` | Fragt den aktuellen Gesundheitsstatus ([`HealthState`]) des Journals ab. |
| `pub fn record(&mut self, path: &std::path::Path) -> Result<()>` | `crates/contextra-durable-fs/src/journal.rs:351` | Fügt einen neuen Dateipfad dauerhaft zum Journal hinzu. |
| `pub fn pending(&self) -> &[PathBuf]` | `crates/contextra-durable-fs/src/journal.rs:358` | Liefert eine unveränderliche Sicht auf alle aktuell ausstehenden Pfade. |
| `pub fn process<F>(&mut self, mut remove_fn: F) -> Result<usize> where F: FnMut(&std::path::Path) -> Result<()>,` | `crates/contextra-durable-fs/src/journal.rs:363` | Verarbeitet alle ausstehenden Einträge unter Verwendung der Entfernungsfunktion `remove_fn`. |

#### `enum JournalSyncPolicy`
- Ort: `crates/contextra-durable-fs/src/journal.rs:12`
```rust
pub enum JournalSyncPolicy
```
Dokumentation: Synchronisations-Politik für das Append-only-Journal.

Varianten:

- `Always` — `crates/contextra-durable-fs/src/journal.rs:14`
- `GroupCommit{` — `crates/contextra-durable-fs/src/journal.rs:16`
- `Manual` — `crates/contextra-durable-fs/src/journal.rs:23`

#### `enum RecordRecoveryPolicy`
- Ort: `crates/contextra-durable-fs/src/journal.rs:28`
```rust
pub enum RecordRecoveryPolicy
```
Dokumentation: Recovery-Verhalten bei beschädigtem Journal-Ende (Torn-Write / Bit-Rot / Zero-Fill).

Varianten:

- `FailClosed` — `crates/contextra-durable-fs/src/journal.rs:31`
- `TruncateCorruptTail` — `crates/contextra-durable-fs/src/journal.rs:33`

#### `fn calculate_crc32`
- Ort: `crates/contextra-durable-fs/src/journal.rs:45`
```rust
pub fn calculate_crc32(data: &[u8]) -> u32
```
Dokumentation: Hilfsfunktion zur Berechnung einer CRC32-Prüfsumme.

#### `const JOURNAL_MAGIC`
- Ort: `crates/contextra-durable-fs/src/journal.rs:37`
```rust
pub const JOURNAL_MAGIC: [u8
```
Dokumentation: Magic Bytes für den Header eines Journal-Records: `[0x43, 0x54, 0x58, 0x4A]` ("CTXJ").

#### `const RECORD_HEADER_SIZE`
- Ort: `crates/contextra-durable-fs/src/journal.rs:41`
```rust
pub const RECORD_HEADER_SIZE: usize = 20
```
Dokumentation: Header-Größe für jeden Journal-Record: 4 Bytes Magic + 8 Bytes SeqNo + 4 Bytes Payload-Länge + 4 Bytes CRC32 = 20 Bytes.

### Modul `platform`

#### `enum FlushKind`
- Ort: `crates/contextra-durable-fs/src/platform.rs:12`
```rust
pub enum FlushKind
```
Dokumentation: Spezifikation des Flush-Umfangs für geöffnete Dateihandles.

Varianten:

- `DataOnly` — `crates/contextra-durable-fs/src/platform.rs:14`
- `DataAndMetadata` — `crates/contextra-durable-fs/src/platform.rs:16`

#### `fn sync_dir_path`
- Ort: `crates/contextra-durable-fs/src/platform.rs:37`
```rust
pub fn sync_dir_path(dir: &Path) -> io::Result<()>
```
Dokumentation: Flusht Verzeichniseinträge synchron auf stabile Speichermedien. Unter Unix wird das Verzeichnis als Datei geöffnet und `sync_all` ausgeführt. Unter Windows werden Verzeichnis-Metadaten von NTFS automatisch journalisiert. # Errors Liefert einen [`io::Error`], falls das Verzeichnis nicht geöffnet oder gesynct werden kann.

#### `fn sync_file`
- Ort: `crates/contextra-durable-fs/src/platform.rs:23`
```rust
pub fn sync_file(file: &File, kind: FlushKind) -> io::Result<()>
```
Dokumentation: Flusht eine geöffnete Datei synchron und crash-sicher auf das Speichermedium. # Errors Liefert einen [`io::Error`], falls der Betriebssystem-Flush fehlschlägt.

### Modul `scrub`

#### `struct ScrubProgress`
- Ort: `crates/contextra-durable-fs/src/scrub.rs:65`
```rust
pub struct ScrubProgress
```
Dokumentation: Fortschrittsbericht beim aufrufergesteuerten Scrubbing.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `bytes_processed` | `u64` | `crates/contextra-durable-fs/src/scrub.rs:67` |
| `total_bytes` | `u64` | `crates/contextra-durable-fs/src/scrub.rs:69` |

#### `struct ScrubReceipt`
- Ort: `crates/contextra-durable-fs/src/scrub.rs:21`
```rust
pub struct ScrubReceipt
```
Dokumentation: Quittung nach erfolgreichem Überschreiben und dauerhaftem Entfernen einer Datei.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `bytes_scrubbed` | `u64` | `crates/contextra-durable-fs/src/scrub.rs:23` |

#### `enum ScrubOutcome`
- Ort: `crates/contextra-durable-fs/src/scrub.rs:54`
```rust
pub enum ScrubOutcome
```
Dokumentation: Ergebnis einer Bereinigungsoperation mittels [`scrub_if_safe`].

Varianten:

- `Scrubbed(ScrubReceipt)` — `crates/contextra-durable-fs/src/scrub.rs:56`
- `RemovedUnscrubbed` — `crates/contextra-durable-fs/src/scrub.rs:58`
- `Refused(ScrubRefusal)` — `crates/contextra-durable-fs/src/scrub.rs:60`

#### `enum ScrubPolicy`
- Ort: `crates/contextra-durable-fs/src/scrub.rs:28`
```rust
pub enum ScrubPolicy
```
Dokumentation: Richtlinie zur Steuerung des Überschreibverhaltens vor dem Löschen.

Varianten:

- `Disabled` — `crates/contextra-durable-fs/src/scrub.rs:30`
- `OverwriteZeros` — `crates/contextra-durable-fs/src/scrub.rs:32`

#### `enum ScrubRefusal`
- Ort: `crates/contextra-durable-fs/src/scrub.rs:37`
```rust
pub enum ScrubRefusal
```
Dokumentation: Gründe für eine Verweigerung des Nullen-Überschreibens aus Sicherheits- oder Plattformgründen.

Varianten:

- `Disabled` — `crates/contextra-durable-fs/src/scrub.rs:39`
- `MultipleHardlinks` — `crates/contextra-durable-fs/src/scrub.rs:41`
- `CopyOnWriteFilesystem` — `crates/contextra-durable-fs/src/scrub.rs:43`
- `ReaderAlive` — `crates/contextra-durable-fs/src/scrub.rs:45`
- `NotObsolete` — `crates/contextra-durable-fs/src/scrub.rs:47`
- `Unsupported` — `crates/contextra-durable-fs/src/scrub.rs:49`

#### `fn scrub_file_with_rate_limit`
- Ort: `crates/contextra-durable-fs/src/scrub.rs:75`
```rust
pub fn scrub_file_with_rate_limit<F>( path: &Path, bytes_per_sec: u64, mut on_progress: F, ) -> Result<u64> where F: FnMut(ScrubProgress),
```
Dokumentation: Prüft eine Datei segmentweise auf Prüfsummenintegrität mit konfigurierbarer Ratenbegrenzung. Wird vollständig vom aufrufenden Thread getrieben (kein Hintergrund-Thread).

#### `fn scrub_if_safe`
- Ort: `crates/contextra-durable-fs/src/scrub.rs:133`
```rust
pub fn scrub_if_safe( path: &Path, policy: ScrubPolicy, reader_alive: bool, not_obsolete: bool, ) -> Result<ScrubOutcome>
```
Dokumentation: Bereinigt und löscht eine Datei sicher, sofern alle Sicherheitsvorbedingungen erfüllt sind.

#### `fn scrub_if_safe_with_injector`
- Ort: `crates/contextra-durable-fs/src/scrub.rs:143`
```rust
pub fn scrub_if_safe_with_injector<F: FaultInjector>( path: &Path, policy: ScrubPolicy, reader_alive: bool, not_obsolete: bool, injector: &F, ) -> Result<ScrubOutcome>
```
Dokumentation: Bereinigt und löscht eine Datei sicher unter Verwendung eines benutzerdefinierten [`FaultInjector`].

### Modul `vfs`

#### `struct StdVfs`
- Ort: `crates/contextra-durable-fs/src/vfs.rs:57`
```rust
pub struct StdVfs
```
Dokumentation: Standard-Implementierung von [`Vfs`] auf Basis von `std::fs`.
- Impl-Block (Trait `Vfs`) `crates/contextra-durable-fs/src/vfs.rs:62`

#### `trait Vfs`
- Ort: `crates/contextra-durable-fs/src/vfs.rs:12`
```rust
pub trait Vfs: Send + Sync
```
Dokumentation: Synchroner Trait für virtuelle Dateisystemoperationen (Klasse A).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn read(&self, path: &Path) -> Result<Vec<u8>>` | `crates/contextra-durable-fs/src/vfs.rs:14` | Liest den gesamten Inhalt einer Datei. |
| `fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()>` | `crates/contextra-durable-fs/src/vfs.rs:17` | Schreibt Daten atomar und crash-sicher in eine Datei (`INV-VFS-ATOMIC-1`). |
| `fn append(&self, path: &Path, data: &[u8]) -> Result<()>` | `crates/contextra-durable-fs/src/vfs.rs:20` | Hängt Daten an eine Datei an und flusht synchron. |
| `fn sync_file(&self, path: &Path) -> Result<()>` | `crates/contextra-durable-fs/src/vfs.rs:23` | Synchronisiert eine Datei auf stabilen Speicher. |
| `fn sync_dir(&self, path: &Path) -> Result<()>` | `crates/contextra-durable-fs/src/vfs.rs:26` | Synchronisiert die Metadaten eines Verzeichnisses auf stabilen Speicher. |
| `fn remove(&self, path: &Path) -> Result<()>` | `crates/contextra-durable-fs/src/vfs.rs:29` | Entfernt eine Datei dauerhaft und flusht das übergeordnete Verzeichnis. |
| `fn rename(&self, from: &Path, to: &Path) -> Result<()>` | `crates/contextra-durable-fs/src/vfs.rs:32` | Benennt eine Datei atomar um und flusht das übergeordnete Verzeichnis. |
| `fn list(&self, path: &Path) -> Result<Vec<String>>` | `crates/contextra-durable-fs/src/vfs.rs:35` | Listet alle Dateinamen in einem Verzeichnis lexikografisch sortiert auf. |
| `fn exists(&self, path: &Path) -> bool` | `crates/contextra-durable-fs/src/vfs.rs:38` | Prüft, ob ein Pfad existiert. |
| `fn len(&self, path: &Path) -> Result<u64>` | `crates/contextra-durable-fs/src/vfs.rs:43` | Gibt die Größe einer Datei in Bytes zurück. |
| `fn create_dir_all(&self, path: &Path) -> Result<()>` | `crates/contextra-durable-fs/src/vfs.rs:50` | Erstellt ein Verzeichnis inklusive aller Zwischenverzeichnisse. |

#### `type RealFs`
- Ort: `crates/contextra-durable-fs/src/vfs.rs:60`
```rust
pub type RealFs = StdVfs
```
Dokumentation: Alias für bestehende Workspace-Kompatibilität (`RealFs`).

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl From<DurableFsError> for std::io::Error` — `crates/contextra-durable-fs/src/error.rs:63` (0 pub Methoden)
- `impl<T: FaultInjector + ?Sized> FaultInjector for &T` — `crates/contextra-durable-fs/src/fault_injector.rs:58` (0 pub Methoden)
