# Schnittstellenspezifikation: `contextra-sys`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-sys` |
| Version | workspace |
| Pfad | `crates/contextra-sys` |
| Rolle | Plattform-nahe Funktionen |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> # Contextra-Sys: Low-Level-Systemabstraktionen und Unsafe-Insel (E0 / Ring 0)
>
> Dieses Modul bildet die hardware- und betriebssystemnahe Basis (Ring 0, Ebene E0) für das Contextra-System
> und externe Konsumenten im Baustein-Ökosystem.
> Es kapselt alle systemweiten Zugriffe, die betriebssystemspezifische FFI-Aufrufe (Foreign Function Interface,
> Schnittstelle zu betriebssystemeigenem C-Code) oder `unsafe`-Code erfordern, in sicheren, typisierten Rust-Schnittstellen.
>
> ## Baukasten-Einordnung und Architektur
> - **Ebene E0 (Kern / Unsafe-Insel):** Darf von höheren Schichten eingebunden werden, hängt selbst von keinen anderen
> Contextra-Crates ab (autarker Baustein).
> - **Abhängigkeiten (ausgehend):** Keine internen Workspace-Abhängigkeiten. Externe Crates: `libc` (POSIX),
> `windows-sys` (Win32), `memmap2`, `thiserror`, `tracing`.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `criterion` | dev | registry | nein |  |
| `memmap2` | normal | workspace | nein |  |
| `tempfile` | dev | registry | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tracing` | normal | workspace | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-bench`, `contextra-store`, `contextra-vault`, `contextra-vector`

## 3. Features

Keine Features deklariert.

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 9 |
| Quellzeilen | 2402 |
| Öffentliche Items (Modul-Ebene) | 44 |
| davon `fn` | 19 |
| davon `struct` | 9 |
| davon `enum` | 2 |
| davon `trait` | 1 |
| davon `type` | 2 |
| davon `mod` | 11 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 13 |
| `unsafe`-Vorkommen (Produktionscode) | 21 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 3 |
| `#[deprecated]` | 0 |
| `#[cfg(feature…)]`-Gates | 0 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 78 / 78 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `acl_win32` | `crates/contextra-sys/src/acl_win32.rs` | 518 |
| `contract` | `crates/contextra-sys/src/contract.rs` | 43 |
| `durability` | `crates/contextra-sys/src/durability.rs` | 487 |
| `error` | `crates/contextra-sys/src/error.rs` | 95 |
| `crate` | `crates/contextra-sys/src/lib.rs` | 104 |
| `mlock` | `crates/contextra-sys/src/mlock.rs` | 328 |
| `mmap` | `crates/contextra-sys/src/mmap.rs` | 305 |
| `posix` | `crates/contextra-sys/src/posix.rs` | 216 |
| `vault` | `crates/contextra-sys/src/vault.rs` | 306 |

## 6. Re-Exports (`pub use`)

- `pub use crate::contract::` — `crates/contextra-sys/src/lib.rs:71` (Modul `mem`)
- `pub use crate::mlock::` — `crates/contextra-sys/src/lib.rs:72` (Modul `mem`)
- `pub use crate::vault::` — `crates/contextra-sys/src/lib.rs:75` (Modul `mem`)
- `pub use crate::acl_win32::` — `crates/contextra-sys/src/lib.rs:81` (Modul `fsmeta`)
- `pub use crate::mmap::` — `crates/contextra-sys/src/lib.rs:87` (Modul `map`)
- `pub use acl_win32::` — `crates/contextra-sys/src/lib.rs:94` (Modul `crate`)
- `pub use contract::` — `crates/contextra-sys/src/lib.rs:95` (Modul `crate`)
- `pub use durability::` — `crates/contextra-sys/src/lib.rs:96` (Modul `crate`)
- `pub use error::` — `crates/contextra-sys/src/lib.rs:97` (Modul `crate`)
- `pub use mlock::` — `crates/contextra-sys/src/lib.rs:98` (Modul `crate`)
- `pub use mmap::` — `crates/contextra-sys/src/lib.rs:101` (Modul `crate`)
- `pub use posix::` — `crates/contextra-sys/src/lib.rs:102` (Modul `crate`)
- `pub use vault::` — `crates/contextra-sys/src/lib.rs:103` (Modul `crate`)

## 8. Öffentliche Schnittstelle je Item

### Modul `acl_win32`

#### `fn set_restrictive_file_acl`
- Ort: `crates/contextra-sys/src/acl_win32.rs:64`
```rust
pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()>
```
Dokumentation: Beschränkt die Zugriffsrechte der angegebenen Datei unter Unix auf den Besitzer (`0600` / `-rw-------`). # Ablauf 1. Konvertiert den oktalen Modus `0600` (nur Lese- und Schreibrechte für den Besitzer) in einen Unix-Berechtigungstyp. 2. Wendet die Berechtigungen über `std::fs::set_permissions` auf das Dateisystem an. # Vorbedingungen - Der Pfad `path` muss existieren und für den aktuellen Prozess änderbar sein. # Nachbedingungen - Nach erfolgreichem Aufruf können andere Benutzer oder Gruppen auf diesem Unix-System die Datei weder lesen noch schreiben. # Fehlerfälle Gibt einen [`std::io::Error`]

#### `fn set_restrictive_file_acl`
- Ort: `crates/contextra-sys/src/acl_win32.rs:201`
```rust
pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()>
```
Dokumentation: Beschränkt unter Windows die Zugriffsrechte der angegebenen Datei auf den Prozessbesitzer mittels Win32-DACL. # Ablauf 1. Ruft die Besitzer-SID des aktuellen Prozesses über `get_process_owner_sid_buffer()` ab. 2. Ermittelt die Länge der SID via `GetLengthSid`. 3. Allokiert Speicher für die Windows-Zugriffskontrollliste (DACL = Discretionary Access Control List) inklusive eines ACE (Access Control Entry). 4. Initialisiert die ACL mit `InitializeAcl` und fügt mit `AddAccessAllowedAce` einen Eintrag ein, der dem Besitzer Vollzugriff (`GENERIC_ALL`) gewährt. 5. Konvertiert den Dateipfad in UTF-16 

#### `fn set_restrictive_file_acl`
- Ort: `crates/contextra-sys/src/acl_win32.rs:409`
```rust
pub fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()>
```
Dokumentation: Best-Effort-Fallback für Zielarchitekturen, die weder Unix noch Windows sind (No-Op).

#### `fn verify_file_acl_owner_only`
- Ort: `crates/contextra-sys/src/acl_win32.rs:86`
```rust
pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()>
```
Dokumentation: Verifiziert unter Unix, dass eine Datei strikt auf den Besitzer beschränkt ist (`0600`) und dem aktuellen Prozess gehört. # Ablauf 1. Liest die Dateimetadaten über `std::fs::metadata(path)`. 2. Prüft die Modus-Bits: Sind Bits für Gruppe oder Andere gesetzt (`mode & 0o077 != 0`), wird sofort abgebrochen. 3. Ermittelt über den POSIX-Systemaufruf `libc::geteuid()` die effektive Benutzer-ID (UID) des aktuellen Prozesses. 4. Vergleicht die Besitzer-UID der Datei (`meta.uid()`) mit der Prozess-UID. Stimmen sie nicht überein, wird abgelehnt. # Fehlerfälle Gibt einen [`std::io::Error`] mit [`std::io::

#### `fn verify_file_acl_owner_only`
- Ort: `crates/contextra-sys/src/acl_win32.rs:284`
```rust
pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()>
```
Dokumentation: Verifiziert unter Windows, dass die Datei über eine geschützte DACL verfügt, die ausschließlich dem aktuellen Prozessbesitzer Zugriff gewährt. # Ablauf 1. Konvertiert den Pfad in nullterminiertes UTF-16. 2. Liest die DACL und den Sicherheitsdeskriptor über `GetNamedSecurityInfoW`. 3. Kapselt den Sicherheitsdeskriptor in einer RAII-Guard, um ihn mit `LocalFree` zuverlässig freizugeben. 4. Prüft über `GetSecurityDescriptorControl`, ob das Bit `SE_DACL_PROTECTED` gesetzt ist (keine vererbten Berechtigungen von übergeordneten Ordnern). 5. Prüft, ob exakt ein einzelner ACE vorhanden ist (`AceCount 

#### `fn verify_file_acl_owner_only`
- Ort: `crates/contextra-sys/src/acl_win32.rs:416`
```rust
pub fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()>
```
Dokumentation: Best-Effort-Fallback für Zielarchitekturen, die weder Unix noch Windows sind (No-Op).

### Modul `contract`

#### `struct NoopMemoryLocker`
- Ort: `crates/contextra-sys/src/contract.rs:21`
```rust
pub struct NoopMemoryLocker
```
Dokumentation: Referenzimplementierung eines Memory-Lockers ohne echte E/A / ohne Syscalls (No-op).
- Impl-Block (Trait `MemoryLocker`) `crates/contextra-sys/src/contract.rs:23`

#### `struct OsMemoryLocker`
- Ort: `crates/contextra-sys/src/contract.rs:33`
```rust
pub struct OsMemoryLocker
```
Dokumentation: Standard-Implementierung über die nativen OS-Syscalls von `contextra-sys`.
- Impl-Block (Trait `MemoryLocker`) `crates/contextra-sys/src/contract.rs:35`

#### `trait MemoryLocker`
- Ort: `crates/contextra-sys/src/contract.rs:11`
```rust
pub trait MemoryLocker: Send + Sync
```
Dokumentation: Abstraktion für Memory-Locking im Arbeitsspeicher.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn lock(&self, ptr: *const u8, len: usize, policy: MemLockPolicy) -> Result<bool>` | `crates/contextra-sys/src/contract.rs:13` | Sperrt eine Speicherregion im RAM. |
| `fn unlock(&self, addr: usize, len: usize)` | `crates/contextra-sys/src/contract.rs:16` | Entsperrt eine zuvor gesperrte Speicherregion. |

### Modul `crate`

#### `mod acl_win32`
- Ort: `crates/contextra-sys/src/lib.rs:47`
```rust
pub mod acl_win32
```

#### `mod contract`
- Ort: `crates/contextra-sys/src/lib.rs:49`
```rust
pub mod contract
```

#### `mod durability`
- Ort: `crates/contextra-sys/src/lib.rs:51`
```rust
pub mod durability
```

#### `mod error`
- Ort: `crates/contextra-sys/src/lib.rs:53`
```rust
pub mod error
```

#### `mod fsmeta`
- Ort: `crates/contextra-sys/src/lib.rs:80`
```rust
pub mod fsmeta
```

#### `mod map`
- Ort: `crates/contextra-sys/src/lib.rs:86`
```rust
pub mod map
```

#### `mod mem`
- Ort: `crates/contextra-sys/src/lib.rs:70`
```rust
pub mod mem
```

#### `mod mlock`
- Ort: `crates/contextra-sys/src/lib.rs:55`
```rust
pub mod mlock
```

#### `mod mmap`
- Ort: `crates/contextra-sys/src/lib.rs:57`
```rust
pub mod mmap
```

#### `mod posix`
- Ort: `crates/contextra-sys/src/lib.rs:59`
```rust
pub mod posix
```

#### `mod vault`
- Ort: `crates/contextra-sys/src/lib.rs:61`
```rust
pub mod vault
```

### Modul `durability`

#### `struct DurableFile`
- Ort: `crates/contextra-sys/src/durability.rs:234`
```rust
pub struct DurableFile
```
Dokumentation: Ein sicherer RAII-Wrapper um [`std::fs::File`], der die Invariante **INV-SYS-FSYNC-1** erzwingt: Sobald ein `fsync`/`fdatasync` oder Flush-Aufruf fehlschlägt, wird das Handle dauerhaft vergiftet (`is_poisoned() == true`). Alle nachfolgenden Schreib- und Synchronisationsaufrufe auf dieser Instanz schlagen sofort mit [`Error::HandlePoisoned`] fehl.
- Impl-Block `crates/contextra-sys/src/durability.rs:239`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(file: File) -> Self` | `crates/contextra-sys/src/durability.rs:241` | Kapselt eine geöffnete [`File`] in einem gesunden, unvergifteten Zustand. |
| `pub fn is_poisoned(&self) -> bool` | `crates/contextra-sys/src/durability.rs:249` | Gibt `true` zurück, wenn das Datei-Handle infolge eines früheren Flush-Fehlers vergiftet ist. |
| `pub fn get_ref(&self) -> Result<&File>` | `crates/contextra-sys/src/durability.rs:254` | Gibt eine Referenz auf die innere [`File`] zurück, sofern das Handle nicht vergiftet ist. |
| `pub fn write_all(&mut self, buf: &[u8]) -> Result<()>` | `crates/contextra-sys/src/durability.rs:263` | Schreibt Daten dauerhaft in die Datei. Schlägt sofort fehl, wenn das Handle vergiftet ist. |
| `pub fn sync_all(&mut self) -> Result<()>` | `crates/contextra-sys/src/durability.rs:271` | Flasht Dateiinhalt und Metadaten auf das physische Speichermedium. |
| `pub fn sync_data(&mut self) -> Result<()>` | `crates/contextra-sys/src/durability.rs:285` | Flasht den Dateiinhalt (ohne Metadaten, z. B. via `fdatasync` unter Linux). |

#### `fn durable_flush_dir`
- Ort: `crates/contextra-sys/src/durability.rs:179`
```rust
pub fn durable_flush_dir(path: &Path) -> IoResult<()>
```
Dokumentation: Flasht den Verzeichniseintrag (übergeordnetes Verzeichnis) einer neu erstellten oder umbenannten Datei.

#### `fn durable_flush_file`
- Ort: `crates/contextra-sys/src/durability.rs:174`
```rust
pub fn durable_flush_file(file: &File, metadata_changed: bool) -> IoResult<()>
```
Dokumentation: Flasht Dateiinhalt und optional Metadaten auf den physischen Datenträger gemäß den Eigenschaften der Zielplattform.

#### `fn write_all_durable`
- Ort: `crates/contextra-sys/src/durability.rs:205`
```rust
pub fn write_all_durable(file: &mut File, mut buf: &[u8]) -> IoResult<()>
```
Dokumentation: Schreibt die gesamten Bytes aus `buf` zuverlässig in `file`, unter robuster Behandlung partieller Schreibvorgänge und Signal-Unterbrechungen.

### Modul `error`

#### `enum Error`
- Ort: `crates/contextra-sys/src/error.rs:24`
```rust
pub enum Error
```
Dokumentation: Typisierter Systemfehler für Low-Level-OS-Operationen.
- Impl-Block (Trait `From<Error>`) `crates/contextra-sys/src/error.rs:72`

Varianten:

- `MemoryLockFailed{` — `crates/contextra-sys/src/error.rs:27`
- `SyncFailed(i32)` — `crates/contextra-sys/src/error.rs:37`
- `PermissionDenied(String)` — `crates/contextra-sys/src/error.rs:41`
- `InvalidInput(String)` — `crates/contextra-sys/src/error.rs:45`
- `Io(#[from] std::io::Error)` — `crates/contextra-sys/src/error.rs:49`
- `Unsupported(&'static str)` — `crates/contextra-sys/src/error.rs:53`
- `HandlePoisoned` — `crates/contextra-sys/src/error.rs:57`

#### `type Result`
- Ort: `crates/contextra-sys/src/error.rs:68`
```rust
pub type Result<T> = std::result::Result<T, Error>
```
Dokumentation: Spezifischer Result-Typ für `contextra-sys`.

#### `type SysError`
- Ort: `crates/contextra-sys/src/error.rs:65`
```rust
pub type SysError = Error
```
Dokumentation: Typ-Alias für das Crate-Fehlermodell gemäß Baukasten-Spezifikation (docs/SPEC.md §3.1 / R3).

### Modul `mlock`

#### `enum MemLockPolicy`
- Ort: `crates/contextra-sys/src/mlock.rs:61`
```rust
pub enum MemLockPolicy
```
Dokumentation: Konfigurierbare Richtlinie für das Sperren von Speicherregionen (`mlock` / `VirtualLock`).

Varianten:

- `FailClosed` — `crates/contextra-sys/src/mlock.rs:64`
- `Tolerant` — `crates/contextra-sys/src/mlock.rs:68`

#### `fn apply_dont_dump`
- Ort: `crates/contextra-sys/src/mlock.rs:81`
```rust
pub fn apply_dont_dump(ptr: *const u8, len: usize) -> Result<()>
```
Dokumentation: Schließt eine Speicherregion per `madvise(MADV_DONTDUMP)` aus Core-Dumps aus (Unix). Auf Nicht-Unix-Systemen ist dies ein no-op. # Argumente * `ptr`: Zeiger auf den Beginn der Speicherregion. * `len`: Länge in Bytes.

#### `fn mem_lock`
- Ort: `crates/contextra-sys/src/mlock.rs:260`
```rust
pub fn mem_lock(ptr: *const u8, len: usize) -> bool
```
Dokumentation: Sperrt eine Speicherregion im RAM mit der Standard-Richtlinie [`MemLockPolicy::Tolerant`].

#### `fn mem_lock_typed`
- Ort: `crates/contextra-sys/src/mlock.rs:123`
```rust
pub fn mem_lock_typed(ptr: *const u8, len: usize, policy: MemLockPolicy) -> Result<bool>
```
Dokumentation: Sperrt eine Speicherregion im RAM gemäß Richtlinie und liefert ein typisiertes [`Result<bool>`]. Liefert `Ok(true)`, wenn das Sperren erfolgreich war. Liefert `Ok(false)`, wenn das Sperren fehlschlug, aber `policy == MemLockPolicy::Tolerant` gewählt wurde. Liefert [`crate::Error::MemoryLockFailed`], wenn das Sperren fehlschlug und `policy == MemLockPolicy::FailClosed` gewählt wurde.

#### `fn mem_lock_with_policy`
- Ort: `crates/contextra-sys/src/mlock.rs:227`
```rust
pub fn mem_lock_with_policy(ptr: *const u8, len: usize, policy: MemLockPolicy) -> IoResult<bool>
```
Dokumentation: Sperrt unter POSIX/Windows eine Speicherregion im RAM, um das Auslagern (Swapping) zu verhindern.

#### `fn mem_unlock`
- Ort: `crates/contextra-sys/src/mlock.rs:232`
```rust
pub fn mem_unlock(addr: usize, len: usize)
```
Dokumentation: Entsperrt eine zuvor gesperrte Speicherregion im RAM.

### Modul `mmap`

#### `struct SealedFile`
- Ort: `crates/contextra-sys/src/mmap.rs:64`
```rust
pub struct SealedFile<State = SealedReadOnly>
```
Dokumentation: Ein typisierter Dateiwächter, der den Versiegelungszustand einer Datei zur Compilezeit und Laufzeit abbildet.
- Impl-Block `crates/contextra-sys/src/mmap.rs:70`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new_verified(file: File, expected_len: u64) -> io::Result<Self>` | `crates/contextra-sys/src/mmap.rs:75` | Erstellt eine versiegelte Dateireferenz nach erfolgreicher Integritäts- und Längenprüfung. # Fehler Liefert [`std::io::ErrorKind::InvalidData`], wenn die Dateil |
| `pub fn as_file(&self) -> &File` | `crates/contextra-sys/src/mmap.rs:95` | Gibt eine Referenz auf die zugrundeliegende [`File`] zurück. |
| `pub fn verified_len(&self) -> u64` | `crates/contextra-sys/src/mmap.rs:100` | Gibt die verifizierte Dateilänge in Bytes zurück. |
| `pub fn map_readonly(&self) -> io::Result<SealedMap>` | `crates/contextra-sys/src/mmap.rs:105` | Mappt die versiegelte Datei über die typgesicherte [`SealedMap`]-Abstraktion. |

#### `struct SealedMap`
- Ort: `crates/contextra-sys/src/mmap.rs:116`
```rust
pub struct SealedMap
```
Dokumentation: Sichere Nur-Lese-Speicherabbildung für versiegelte, unveränderliche Dateien (INV-IO-1).
- Impl-Block `crates/contextra-sys/src/mmap.rs:120`
- Impl-Block (Trait `std::ops::Deref`) `crates/contextra-sys/src/mmap.rs:170`
- Impl-Block (Trait `AsRef<[u8]>`) `crates/contextra-sys/src/mmap.rs:178`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn map_sealed(file: &SealedFile<SealedReadOnly>) -> io::Result<Self>` | `crates/contextra-sys/src/mmap.rs:125` | Mappt eine nachweislich versiegelte Datei in den virtuellen Adressraum. # Fehler Liefert [`std::io::ErrorKind::InvalidData`], wenn die gemappte Größe von der er |
| `pub fn get_subslice(&self, offset: usize, len: usize) -> Result<&[u8]>` | `crates/contextra-sys/src/mmap.rs:145` | Liest einen Teilbereich der Speicherabbildung mit sicherer Grenzenprüfung. # Fehler Liefert [`crate::Error::InvalidInput`], falls Bereichsgrenzen überschritten  |
| `pub fn as_slice(&self) -> &[u8]` | `crates/contextra-sys/src/mmap.rs:155` | Gibt den gesamten Speicherinhalt als Byte-Slice zurück. |
| `pub fn len(&self) -> usize` | `crates/contextra-sys/src/mmap.rs:160` | Gibt die Länge der Speicherabbildung in Bytes zurück. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-sys/src/mmap.rs:165` | Prüft, ob die Speicherabbildung leer ist (0 Bytes). |

#### `struct SealedReadOnly`
- Ort: `crates/contextra-sys/src/mmap.rs:55`
```rust
pub struct SealedReadOnly
```
Dokumentation: Typzustands-Marker: Zeigt an, dass eine Datei vollständig versiegelt, unveränderlich und prüfsummenverifiziert ist (gemäß INV-IO-1 und INV-ENCRYPT-REST-2).

#### `struct Unsealed`
- Ort: `crates/contextra-sys/src/mmap.rs:59`
```rust
pub struct Unsealed
```
Dokumentation: Typzustands-Marker: Zeigt an, dass eine Datei noch unversiegelt oder potenziell veränderlich ist.

#### `fn mmap_readonly`
- Ort: `crates/contextra-sys/src/mmap.rs:41`
```rust
pub fn mmap_readonly(file: &File) -> io::Result<memmap2::Mmap>
```
Dokumentation: Erstellt eine unveränderliche Nur-Lese-Speicherabbildung (Read-Only Memory Map) für eine geöffnete Datei. # Vorbedingungen * `file`: Gültiger, lesbarer Dateideskriptor. # Fehler Liefert [`std::io::Error`], falls das Betriebssystem das Mapping ablehnt (z. B. unzureichender virtueller Adressraum).

### Modul `posix`

#### `fn ensure_fd_limit`
- Ort: `crates/contextra-sys/src/posix.rs:84`
```rust
pub fn ensure_fd_limit(min_fds: u64) -> bool
```
Dokumentation: Stellt sicher, dass das weiche Limit für offene Dateideskriptoren (`RLIMIT_NOFILE`) mindestens `min_fds` beträgt. Versucht das Limit bis zum maximal erlaubten harten Limit (`rlim_max`) anzuheben. Liefert `true`, wenn das effektive Limit nach dem Aufruf mindestens `min_fds` beträgt.

#### `fn reopen_and_dup2`
- Ort: `crates/contextra-sys/src/posix.rs:37`
```rust
pub fn reopen_and_dup2(path: &Path, target_fd: i32, flags: i32) -> std::io::Result<()>
```
Dokumentation: Öffnet den Pfad `path` mit den angegebenen POSIX-Flags (z. B. `O_RDONLY`) und ersetzt atomar `target_fd` via `dup2`. # Argumente * `path`: Pfad zur neu zu öffnenden Datei. * `target_fd`: Ziel-Dateideskriptor (muss >= 0 sein). * `flags`: POSIX-Öffnungsflags für `libc::open`. # Fehler Liefert [`std::io::Error`], wenn der Pfad ein ungültiges Format aufweist oder Systemaufrufe fehlschlagen.

#### `fn reopen_and_dup2`
- Ort: `crates/contextra-sys/src/posix.rs:69`
```rust
pub fn reopen_and_dup2(_path: &Path, _target_fd: i32, _flags: i32) -> std::io::Result<()>
```
Dokumentation: Fallback für Nicht-Linux-Systeme, auf denen `reopen_and_dup2` nicht unterstützt wird.

#### `fn set_fd_limit`
- Ort: `crates/contextra-sys/src/posix.rs:119`
```rust
pub fn set_fd_limit(limit: u64) -> bool
```
Dokumentation: Setzt das weiche und harte Dateideskriptorlimit (`RLIMIT_NOFILE`) des aktuellen Prozesses exakt auf `limit`. Liefert `true`, wenn das Limit erfolgreich gesetzt werden konnte.

### Modul `vault`

#### `struct LockedRegions`
- Ort: `crates/contextra-sys/src/vault.rs:40`
```rust
pub struct LockedRegions
```
Dokumentation: Kapselt und verfolgt im RAM gesperrte Speicherbereiche (Adressen und Längen) für sensible Schlüsseldaten.
- Impl-Block `crates/contextra-sys/src/vault.rs:44`
- Impl-Block (Trait `Drop`) `crates/contextra-sys/src/vault.rs:113`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-sys/src/vault.rs:46` | Erstellt eine neue, leere Nachverfolgungsinstanz für gesperrte Speicherregionen. |
| `pub fn lock_slice(&mut self, slice: &[u8], attempt_mlock: bool) -> bool` | `crates/contextra-sys/src/vault.rs:53` | Versucht einen Byte-Slice im RAM zu sperren, falls `attempt_mlock` wahr ist, unter Verwendung der Standardrichtlinie [`MemLockPolicy::Tolerant`]. |
| `pub fn lock_slice_with_policy( &mut self, slice: &[u8], attempt_mlock: bool, policy: MemLockPolicy, ) -> std::io::Result<bool>` | `crates/contextra-sys/src/vault.rs:59` | Versucht einen Byte-Slice im RAM gemäß der angegebenen [`MemLockPolicy`] zu sperren, falls `attempt_mlock == true`. |
| `pub fn zeroize_slice(slice: &mut [u8])` | `crates/contextra-sys/src/vault.rs:84` | Überschreibt einen veränderbaren Byte-Slice sicher mit Nullen unter Verwendung von flüchtigen Schreibzugriffen (`write_volatile`) und einer Compiler-Fence (`Ord |
| `pub fn unlock_all(&mut self)` | `crates/contextra-sys/src/vault.rs:96` | Entsperrt alle registrierten Speicherregionen und leert die interne Nachverfolgungsliste. |
| `pub fn len(&self) -> usize` | `crates/contextra-sys/src/vault.rs:103` | Gibt die Anzahl der aktuell nachverfolgten, gesperrten Regionen zurück. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-sys/src/vault.rs:108` | Gibt `true` zurück, wenn aktuell keine gesperrten Speicherregionen verwaltet werden. |

#### `struct VaultBuffer`
- Ort: `crates/contextra-sys/src/vault.rs:125`
```rust
pub struct VaultBuffer
```
Dokumentation: Ein im RAM verankerter, geschützter Speicherpuffer für hochsensible kryptografische Schlüsseldaten (INV-SYS-MEMLOCK-1).
- Impl-Block `crates/contextra-sys/src/vault.rs:130`
- Impl-Block (Trait `std::ops::Deref`) `crates/contextra-sys/src/vault.rs:192`
- Impl-Block (Trait `std::ops::DerefMut`) `crates/contextra-sys/src/vault.rs:200`
- Impl-Block (Trait `Drop`) `crates/contextra-sys/src/vault.rs:208`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn allocate(size: usize) -> Result<Self>` | `crates/contextra-sys/src/vault.rs:136` | Allokiert einen neuen Puffer der Größe `size` mit Nullen, verankert ihn im RAM (`MemLockPolicy::FailClosed`) und schließt ihn von Core-Dumps aus. # Fehler Liefe |
| `pub fn allocate_with_policy(size: usize, policy: MemLockPolicy) -> Result<Self>` | `crates/contextra-sys/src/vault.rs:141` | Allokiert einen neuen Puffer der Größe `size` gemäß der übergebenen Sperr-Richtlinie. |
| `pub fn from_slice(source: &[u8]) -> Result<Self>` | `crates/contextra-sys/src/vault.rs:160` | Allokiert einen Puffer und initialisiert ihn sofort mit einer Kopie der übergebenen sensiblen Daten. |
| `pub fn is_locked(&self) -> bool` | `crates/contextra-sys/src/vault.rs:167` | Gibt `true` zurück, wenn der Speicherbereich aktiv im physischen RAM verankert ist. |
| `pub fn as_slice(&self) -> &[u8]` | `crates/contextra-sys/src/vault.rs:172` | Liefert die unveränderliche Sicht auf den Pufferinhalt. |
| `pub fn as_mut_slice(&mut self) -> &mut [u8]` | `crates/contextra-sys/src/vault.rs:177` | Liefert die veränderbare Sicht auf den Pufferinhalt. |
| `pub fn len(&self) -> usize` | `crates/contextra-sys/src/vault.rs:182` | Gibt die Kapazität/Länge des Puffers in Bytes zurück. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-sys/src/vault.rs:187` | Prüft, ob der Puffer 0 Bytes lang ist. |
