# Öffentliche API: `contextra-durable-fs`

Vollständige Übersicht aller öffentlichen Typen, Traits, Structs und Funktionen in `contextra-durable-fs`.

## `src/error.rs`

| Signatur | Beschreibung |
|---|---|
| `pub enum DurableFsError ` |  |
| `pub type Error = DurableFsError` | Kanonischer Alias `Error` gemäß Klasse-A-Vertrag (B4). |
| `pub fn kind(&self) -> std::io::ErrorKind ` |  |
| `pub type Result<T> = std::result::Result<T, DurableFsError>` | Einheitlicher Result-Typ für `contextra-durable-fs`. |

## `src/fault_injector.rs`

| Signatur | Beschreibung |
|---|---|
| `pub trait FaultInjector: Send + Sync ` | Schnittstelle zur Injektion von I/O-Fehlern in Persistenzpfade. |
| `pub struct NoFault` |  |
| `pub enum InjectedFault ` |  |
| `pub struct FaultInjectingVfs<V: Vfs> ` |  |
| `pub fn new(inner: V) -> Self ` | Kapselt ein bestehendes [`Vfs`] in einen Injektor-Dekorator. |
| `pub fn inject_write_disk_full(&self) ` | Konfiguriert, dass der nächste Schreibaufruf mit `ENOSPC` fehlschlägt. |
| `pub fn inject_fsync_failure(&self) ` | Konfiguriert, dass der nächste `sync_file`-Aufruf mit EIO fehlschlägt. |
| `pub fn inject_rename_failure(&self) ` | Konfiguriert, dass das nächste `rename` fehlschlägt. |
| `pub fn inject_dir_fsync_failure(&self) ` | Konfiguriert, dass der nächste Verzeichnis-Sync fehlschlägt. |
| `pub fn simulate_crash(&self) -> Result<()> ` | Simuliert einen plötzlichen Prozessabsturz (Crash-Punkt-Simulation): |

## `src/health.rs`

| Signatur | Beschreibung |
|---|---|
| `pub enum HealthState ` |  |
| `pub const fn new() -> Self ` |  |
| `pub fn transition(self, event: HealthEvent) -> Self ` |  |
| `pub fn allows_writes(self) -> bool ` |  |
| `pub fn is_poisoned(self) -> bool ` |  |
| `pub enum HealthEvent ` |  |
| `pub struct AtomicHealthState ` |  |
| `pub fn new() -> Self ` |  |
| `pub fn load(&self) -> HealthState ` |  |
| `pub fn record_event(&self, event: HealthEvent) -> HealthState ` | Führt einen atomaren Zustandsübergang anhand eines [`HealthEvent`] aus. |

## `src/journal.rs`

| Signatur | Beschreibung |
|---|---|
| `pub enum JournalSyncPolicy ` |  |
| `pub enum RecordRecoveryPolicy ` |  |
| `pub const JOURNAL_MAGIC: [u8; 4] = [0x43, 0x54, 0x58, 0x4A]` | Magic Bytes für den Header eines Journal-Records: `[0x43, 0x54, 0x58, 0x4A]` ("CTXJ"). |
| `pub const RECORD_HEADER_SIZE: usize = 20` | Header-Größe für jeden Journal-Record: |
| `pub fn calculate_crc32(data: &[u8]) -> u32 ` |  |
| `pub struct JournalRecord ` |  |
| `pub seq_no: u64,` | Monoton steigende Sequenznummer des Records. |
| `pub payload: Vec<u8>,` | Nutzdaten des Records. |
| `pub struct AppendOnlyJournal ` |  |
| `pub fn open(path: impl Into<PathBuf>) -> Result<Self> ` | Öffnet oder erstellt ein Journal mit Standard-Sync-Politik ([`JournalSyncPolicy::Always`]). |
| `pub fn open_without_recovery(` | Öffnet ein Journal ohne automatische Initial-Recovery (nützlich für manuelle Recovery). |
| `pub fn open_with_policy(` | Öffnet oder erstellt ein Journal mit benutzerdefinierter Sync-Politik und Health-State. |
| `pub fn path(&self) -> &std::path::Path ` |  |
| `pub fn append(&mut self, payload: &[u8]) -> Result<u64> ` | Hängt einen neuen Payload-Record an das Journal an. |
| `pub fn flush(&mut self) -> Result<()> ` | Flusht ausstehende Journal-Schreibvorgänge synchron auf stabilen Speicher (`INV-FSYNC-1`). |
| `pub fn recover(&mut self, recovery_policy: RecordRecoveryPolicy) -> Result<Vec<JournalRecord>> ` | Führt die Recovery des Journals durch und liest gültige Records ein. |
| `pub fn health_state(&self) -> HealthState ` |  |
| `pub struct PendingDeleteJournal ` |  |
| `pub fn open(journal_path: impl Into<PathBuf>) -> Result<Self> ` | Öffnet oder erstellt ein `PendingDeleteJournal` am angegebenen Pfad mit neuem Health-State. |
| `pub fn open_with_health(` | Öffnet oder erstellt ein `PendingDeleteJournal` mit einer geteilten [`AtomicHealthState`]-Instanz. |
| `pub fn health_state(&self) -> HealthState ` |  |
| `pub fn record(&mut self, path: &std::path::Path) -> Result<()> ` | Fügt einen neuen Dateipfad dauerhaft zum Journal hinzu. |
| `pub fn pending(&self) -> &[PathBuf] ` |  |
| `pub fn process<F>(&mut self, mut remove_fn: F) -> Result<usize>` | Verarbeitet alle ausstehenden Einträge unter Verwendung der Entfernungsfunktion `remove_fn`. |

## `src/lib.rs`

| Signatur | Beschreibung |
|---|---|
| `pub mod error` | Fehlertypen und Result-Definitionen für Dateisystemoperationen. |
| `pub mod fault_injector` | Fehlerinjektor und VFS-Dekorator zur Simulation von Ausfällen und Crash-Punkten. |
| `pub mod health` | Fail-Closed-Zustandsverfolgung und Poisoning-Zustandsautomat (§18.4). |
| `pub mod journal` | Append-only-Journal mit Prüfsummen und Pending-Delete-Journal für Crash-Resilienz. |
| `pub mod platform` | Plattformspezifische synchrone Flush-Primitive für Dateien und Verzeichnisse. |
| `pub mod scrub` | Sicheres Überschreiben (Scrubbing) und Integritätsprüfung mit Ratenbegrenzung. |
| `pub mod vfs` | Virtuelles Dateisystem (VFS) mit atomaren Ersetzungen und synchroner Schnittstelle. |
| `pub use error::{DurableFsError, Error, Result}` |  |
| `pub use fault_injector::{FaultInjector, NoFault}` |  |
| `pub use health::{AtomicHealthState, HealthEvent, HealthState}` |  |
| `pub use journal::` |  |
| `pub use platform::FlushKind` |  |
| `pub use scrub::` |  |
| `pub use vfs::{RealFs, StdVfs, Vfs}` |  |
| `pub enum FsClass ` |  |
| `pub const fn is_supported_local(&self) -> bool ` |  |
| `pub fn flush_file(file: &File, kind: FlushKind) -> Result<()> ` | Flusht eine geöffnete Datei auf die Festplatte. |
| `pub fn flush_dir(dir: &Path) -> Result<()> ` | Flusht Verzeichniseinträge auf stabile Speichermedien. |
| `pub fn fs_class(path: &Path) -> Result<FsClass> ` | Ermittelt die Dateisystemklasse für den angegebenen Pfad (§23.4). |
| `pub fn rename_durable(from: &Path, to: &Path) -> Result<()> ` | Benennt eine Datei atomar um und flusht das übergeordnete Verzeichnis. |
| `pub struct PoisonedHandle` |  |
| `pub struct DurableHandle<F: FaultInjector = NoFault> ` |  |
| `pub fn new(file: File) -> Self ` | Kapselt eine geöffnete [`File`] in ein neues [`DurableHandle`] mit standardmäßigem [`NoFault`]. |
| `pub fn new_with_injector(file: File, injector: F) -> Self ` | Kapselt eine geöffnete [`File`] mit einem benutzerdefinierten [`FaultInjector`]. |
| `pub fn new_with_path_and_injector(file: File, path: PathBuf, injector: F) -> Self ` | Kapselt eine geöffnete [`File`] mit zugehörigem Pfad und benutzerdefiniertem [`FaultInjector`]. |
| `pub fn is_poisoned(&self) -> bool ` |  |
| `pub fn flush(&mut self, kind: FlushKind) -> Result<()> ` | Flusht Dateiinhalte synchron auf die Festplatte und führt Fehlerinjektions-Hooks aus. |
| `pub fn get_ref(&self) -> Option<&File> ` |  |
| `pub fn get_mut(&mut self) -> Option<&mut File> ` | Liefert eine veränderliche Referenz auf die zugrundeliegende Datei, oder `None`, falls das Handle vergiftet ist. |
| `pub fn poison(&mut self) ` | Vergiftet dieses Handle explizit manuell. |
| `pub enum LockMode ` |  |
| `pub struct DirLock ` |  |
| `pub fn acquire(dir: &Path, mode: LockMode) -> Result<Self> ` | Erwirbt eine Sperre auf `dir` im angegebenen [`LockMode`]. |
| `pub fn mode(&self) -> LockMode ` |  |
| `pub fn sync_dir(dir: &Path) -> Result<()> ` | Synchronisiert die Verzeichnismetadaten eines Ordners mit Standard-Injektor [`NoFault`]. |
| `pub fn sync_dir_with_injector<F: FaultInjector>(dir: &Path, injector: &F) -> Result<()> ` | Synchronisiert die Verzeichnismetadaten eines Ordners mit benutzerdefiniertem [`FaultInjector`]. |
| `pub fn atomic_replace(path: &Path, bytes: &[u8]) -> Result<()> ` | Ersetzt den Inhalt der Datei unter `path` atomar mit `bytes` unter Verwendung des Standardinjektors [`NoFault`]. |
| `pub fn atomic_replace_with_injector<F: FaultInjector>(` | Ersetzt den Inhalt der Datei unter `path` atomar mit `bytes` unter Verwendung eines [`FaultInjector`] (`INV-VFS-ATOMIC-1`). |
| `pub fn durable_remove(path: &Path) -> Result<()> ` | Entfernt eine Datei dauerhaft und flusht das Elternverzeichnis mit Standardinjektor [`NoFault`]. |
| `pub fn durable_remove_with_injector<F: FaultInjector>(path: &Path, injector: &F) -> Result<()> ` | Entfernt eine Datei dauerhaft und flusht das Elternverzeichnis mit benutzerdefiniertem [`FaultInjector`]. |
| `pub fn scrub_and_remove(path: &Path) -> Result<ScrubReceipt> ` | Überschreibt eine Datei vollständig mit Nullen und entfernt sie dauerhaft (mit [`NoFault`]). |
| `pub fn scrub_and_remove_with_injector<F: FaultInjector>(` | Überschreibt eine Datei mit Nullen und entfernt sie dauerhaft unter Verwendung eines [`FaultInjector`]. |

## `src/platform.rs`

| Signatur | Beschreibung |
|---|---|
| `pub enum FlushKind ` |  |
| `pub fn sync_file(file: &File, kind: FlushKind) -> io::Result<()> ` | Flusht eine geöffnete Datei synchron und crash-sicher auf das Speichermedium. |
| `pub fn sync_dir_path(dir: &Path) -> io::Result<()> ` | Flusht Verzeichniseinträge synchron auf stabile Speichermedien. |

## `src/scrub.rs`

| Signatur | Beschreibung |
|---|---|
| `pub struct ScrubReceipt ` |  |
| `pub bytes_scrubbed: u64,` | Gesamtanzahl der vor dem Löschen mit Nullen überschriebenen Bytes. |
| `pub enum ScrubPolicy ` |  |
| `pub enum ScrubRefusal ` |  |
| `pub enum ScrubOutcome ` |  |
| `pub struct ScrubProgress ` |  |
| `pub bytes_processed: u64,` | Bisher verarbeitete Bytes. |
| `pub total_bytes: u64,` | Gesamtgröße der zu prüfenden Datei / des Segments in Bytes. |
| `pub fn scrub_file_with_rate_limit<F>(` | Prüft eine Datei segmentweise auf Prüfsummenintegrität mit konfigurierbarer Ratenbegrenzung. |
| `pub fn scrub_if_safe(` | Bereinigt und löscht eine Datei sicher, sofern alle Sicherheitsvorbedingungen erfüllt sind. |
| `pub fn scrub_if_safe_with_injector<F: FaultInjector>(` | Bereinigt und löscht eine Datei sicher unter Verwendung eines benutzerdefinierten [`FaultInjector`]. |

## `src/vfs.rs`

| Signatur | Beschreibung |
|---|---|
| `pub trait Vfs: Send + Sync ` | Synchroner Trait für virtuelle Dateisystemoperationen . |
| `pub struct StdVfs` |  |
| `pub type RealFs = StdVfs` | Alias für bestehende Workspace-Kompatibilität (`RealFs`). |
