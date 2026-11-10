# Öffentliche API: `contextra-sys`

Vollständige Übersicht aller öffentlichen Typen, Traits, Structs und Funktionen in `contextra-sys`.

## `src/acl_win32.rs`

| Signatur | Beschreibung |
|---|---|
| `pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()> ` | Beschränkt die Zugriffsrechte der angegebenen Datei unter Unix auf den Besitzer (`0600` / `-rw-------`). |
| `pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()> ` | Verifiziert unter Unix, dass eine Datei strikt auf den Besitzer beschränkt ist (`0600`) und dem aktuellen Prozess gehört. |
| `pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()> ` | Beschränkt unter Windows die Zugriffsrechte der angegebenen Datei auf den Prozessbesitzer mittels Win32-DACL. |
| `pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()> ` | Verifiziert unter Windows, dass die Datei über eine geschützte DACL verfügt, die ausschließlich dem aktuellen Prozessbesitzer Zugriff gewährt. |
| `pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()> ` | Best-Effort-Fallback für Zielarchitekturen, die weder Unix noch Windows sind (No-Op). |
| `pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()> ` | Best-Effort-Fallback für Zielarchitekturen, die weder Unix noch Windows sind (No-Op). |

## `src/durability.rs`

| Signatur | Beschreibung |
|---|---|
| `pub fn set_fail_fsync(fail: bool, errno: i32) ` |  |
| `pub fn should_fail_fsync() -> Option<i32> ` |  |
| `pub fn durable_flush_file(file: &File, metadata_changed: bool) -> IoResult<()> ` | Flasht Dateiinhalt und optional Metadaten auf den physischen Datenträger gemäß den Garantien der Zielplattform. |
| `pub fn durable_flush_dir(path: &Path) -> IoResult<()> ` | Flasht den Verzeichniseintrag (übergeordnetes Verzeichnis) einer neu erstellten oder umbenannten Datei. |
| `pub fn write_all_durable(file: &mut File, mut buf: &[u8]) -> IoResult<()> ` | Schreibt die gesamten Bytes aus `buf` zuverlässig in `file`, unter robuster Behandlung partieller Schreibvorgänge und Signal-Unterbrechungen. |
| `pub struct DurableFile ` |  |
| `pub fn new(file: File) -> Self ` | Kapselt eine geöffnete [`File`] in einem gesunden, unvergifteten Zustand. |
| `pub fn is_poisoned(&self) -> bool ` | Gibt `true` zurück, wenn das Datei-Handle infolge eines früheren Flush-Fehlers vergiftet ist. |
| `pub fn get_ref(&self) -> Result<&File> ` | Gibt eine Referenz auf die innere [`File`] zurück, sofern das Handle nicht vergiftet ist. |
| `pub fn write_all(&mut self, buf: &[u8]) -> Result<()> ` | Schreibt Daten dauerhaft in die Datei. Schlägt sofort fehl, wenn das Handle vergiftet ist. |
| `pub fn sync_all(&mut self) -> Result<()> ` | Flasht Dateiinhalt und Metadaten auf das physische Speichermedium. |
| `pub fn sync_data(&mut self) -> Result<()> ` | Flasht den Dateiinhalt (ohne Metadaten, z. B. via `fdatasync` unter Linux). |

## `src/error.rs`

| Signatur | Beschreibung |
|---|---|
| `pub enum Error ` |  |
| `pub type Result<T> = std::result::Result<T, Error>` | Spezifischer Result-Typ für `contextra-sys`. |

## `src/lib.rs`

| Signatur | Beschreibung |
|---|---|
| `pub mod acl_win32` |  |
| `pub mod durability` |  |
| `pub mod error` |  |
| `pub mod mlock` |  |
| `pub mod mmap` |  |
| `pub mod posix` |  |
| `pub mod vault` |  |
| `pub mod mem ` | Baukasten-Zielmodul `mem`: Kapselt Memory-Locking, Vault-Speicher und Zeroization. |
| `pub use crate::mlock::` |  |
| `pub use crate::vault::{LockedRegions, VaultBuffer}` |  |
| `pub mod fsmeta ` | Baukasten-Zielmodul `fsmeta`: Plattformspezifische Datei-ACLs und Eigentümerprüfungen. |
| `pub use crate::acl_win32::{set_restrictive_file_acl, verify_file_acl_owner_only}` |  |
| `pub mod map ` | Baukasten-Zielmodul `map`: Sichere Speicherabbildung (INV-IO-1 / INV-ENCRYPT-REST-2). |
| `pub use crate::mmap::{mmap_readonly, SealedFile, SealedMap, SealedReadOnly, Unsealed}` |  |
| `pub use acl_win32::{set_restrictive_file_acl, verify_file_acl_owner_only}` |  |
| `pub use durability::{durable_flush_dir, durable_flush_file, write_all_durable, DurableFile}` |  |
| `pub use error::{Error, Result}` |  |
| `pub use mlock::` |  |
| `pub use mmap::{mmap_readonly, SealedFile, SealedMap, SealedReadOnly, Unsealed}` |  |
| `pub use posix::{ensure_fd_limit, reopen_and_dup2, set_fd_limit}` |  |
| `pub use vault::{LockedRegions, VaultBuffer}` |  |

## `src/mlock.rs`

| Signatur | Beschreibung |
|---|---|
| `pub fn set_fail_mlock(fail: bool, errno: i32) ` |  |
| `pub fn should_fail_mlock() -> Option<i32> ` |  |
| `pub enum MemLockPolicy ` |  |
| `pub fn apply_dont_dump(ptr: *const u8, len: usize) -> Result<()> ` | Schließt eine Speicherregion per `madvise(MADV_DONTDUMP)` aus Core-Dumps aus (Unix). |
| `pub fn mem_lock_typed(ptr: *const u8, len: usize, policy: MemLockPolicy) -> Result<bool> ` | Sperrt eine Speicherregion im RAM gemäß Richtlinie und liefert ein typisiertes [`Result<bool>`]. |
| `pub fn mem_lock_with_policy(ptr: *const u8, len: usize, policy: MemLockPolicy) -> IoResult<bool> ` | Sperrt unter POSIX/Windows eine Speicherregion im RAM, um das Auslagern (Swapping) zu verhindern. |
| `pub fn mem_unlock(addr: usize, len: usize) ` | Entsperrt eine zuvor gesperrte Speicherregion im RAM. |
| `pub fn mem_lock(ptr: *const u8, len: usize) -> bool ` | Sperrt eine Speicherregion im RAM mit der Standard-Richtlinie [`MemLockPolicy::Tolerant`]. |

## `src/mmap.rs`

| Signatur | Beschreibung |
|---|---|
| `pub fn mmap_readonly(file: &File) -> io::Result<memmap2::Mmap> ` | Erstellt eine unveränderliche Nur-Lese-Speicherabbildung (Read-Only Memory Map) für eine geöffnete Datei. |
| `pub struct SealedReadOnly` |  |
| `pub struct Unsealed` |  |
| `pub struct SealedFile<State = SealedReadOnly> ` |  |
| `pub fn new_verified(file: File, expected_len: u64) -> io::Result<Self> ` | Erstellt eine versiegelte Dateireferenz nach erfolgreicher Integritäts- und Prüfsummenverifikation. |
| `pub fn as_file(&self) -> &File ` | Gibt eine Referenz auf die zugrundeliegende [`File`] zurück. |
| `pub fn verified_len(&self) -> u64 ` | Gibt die verifizierte Dateilänge in Bytes zurück. |
| `pub fn map_readonly(&self) -> io::Result<SealedMap> ` | Mappt die versiegelte Datei über die typgesicherte [`SealedMap`]-Abstraktion. |
| `pub struct SealedMap ` |  |
| `pub fn map_sealed(file: &SealedFile<SealedReadOnly>) -> io::Result<Self> ` | Mappt eine nachweislich versiegelte Datei in den virtuellen Adressraum. |
| `pub fn get_subslice(&self, offset: usize, len: usize) -> Result<&[u8]> ` | Liest einen Teilbereich der Speicherabbildung mit sicherer Grenzenprüfung. |
| `pub fn as_slice(&self) -> &[u8] ` | Gibt den gesamten Speicherinhalt als Byte-Slice zurück. |
| `pub fn len(&self) -> usize ` | Gibt die Länge der Speicherabbildung in Bytes zurück. |
| `pub fn is_empty(&self) -> bool ` | Prüft, ob die Speicherabbildung leer ist (0 Bytes). |

## `src/posix.rs`

| Signatur | Beschreibung |
|---|---|
| `pub fn reopen_and_dup2(path: &Path, target_fd: i32, flags: i32) -> std::io::Result<()> ` |  |
| `pub fn reopen_and_dup2(_path: &Path, _target_fd: i32, _flags: i32) -> std::io::Result<()> ` |  |
| `pub fn ensure_fd_limit(min_fds: u64) -> bool ` | Stellt sicher, dass das weiche Limit für offene Dateideskriptoren (`RLIMIT_NOFILE`) mindestens `min_fds` beträgt. |
| `pub fn set_fd_limit(limit: u64) -> bool ` | Setzt das weiche und harte Dateideskriptorlimit (`RLIMIT_NOFILE`) des aktuellen Prozesses exakt auf `limit`. |

## `src/vault.rs`

| Signatur | Beschreibung |
|---|---|
| `pub struct LockedRegions ` |  |
| `pub fn new() -> Self ` | Erstellt eine neue, leere Nachverfolgungsinstanz für gesperrte Speicherregionen. |
| `pub fn lock_slice(&mut self, slice: &[u8], attempt_mlock: bool) -> bool ` | Versucht einen Byte-Slice im RAM zu sperren, falls `attempt_mlock` wahr ist, unter Verwendung der Standardrichtlinie [`MemLockPolicy::Tolerant`]. |
| `pub fn lock_slice_with_policy(` | Versucht einen Byte-Slice im RAM gemäß der angegebenen [`MemLockPolicy`] zu sperren, falls `attempt_mlock == true`. |
| `pub fn zeroize_slice(slice: &mut [u8]) ` | Überschreibt einen veränderbaren Byte-Slice sicher mit Nullen unter Verwendung von flüchtigen Schreibzugriffen (`write_volatile`) |
| `pub fn unlock_all(&mut self) ` | Entsperrt alle registrierten Speicherregionen und leert die interne Nachverfolgungsliste. |
| `pub fn len(&self) -> usize ` | Gibt die Anzahl der aktuell nachverfolgten, gesperrten Regionen zurück. |
| `pub fn is_empty(&self) -> bool ` | Gibt `true` zurück, wenn aktuell keine gesperrten Speicherregionen verwaltet werden. |
| `pub struct VaultBuffer ` |  |
| `pub fn allocate(size: usize) -> Result<Self> ` | Allokiert einen neuen Puffer der Größe `size` mit Nullen, verankert ihn im RAM (`MemLockPolicy::FailClosed`) |
| `pub fn allocate_with_policy(size: usize, policy: MemLockPolicy) -> Result<Self> ` | Allokiert einen neuen Puffer der Größe `size` gemäß der übergebenen Sperr-Richtlinie. |
| `pub fn from_slice(source: &[u8]) -> Result<Self> ` | Allokiert einen Puffer und initialisiert ihn sofort mit einer Kopie der übergebenen sensiblen Daten. |
| `pub fn is_locked(&self) -> bool ` | Gibt `true` zurück, wenn der Speicherbereich aktiv im physischen RAM verankert ist. |
| `pub fn as_slice(&self) -> &[u8] ` | Liefert die unveränderliche Sicht auf den Pufferinhalt. |
| `pub fn as_mut_slice(&mut self) -> &mut [u8] ` | Liefert die veränderbare Sicht auf den Pufferinhalt. |
| `pub fn len(&self) -> usize ` | Gibt die Kapazität/Länge des Puffers in Bytes zurück. |
| `pub fn is_empty(&self) -> bool ` | Prüft, ob der Puffer 0 Bytes lang ist. |
