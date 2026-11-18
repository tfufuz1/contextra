# contextra-sys — Crate-Spezifikation (OS-Primitive & Unsafe-Insel)

**Ebene:** E0 (Autarkes Fundament — Kern-Closure leer)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-sys`  
**Referenz:** `docs/SPEC.md` §2, §4

---

## 1. Zweck & Systemrolle

`contextra-sys` ist eine der beiden explizit genehmigten **Unsafe-Inseln** im Contextra-Workspace (neben `contextra-simd`):
- **Betriebssystemnahe Speichersteuerung:** Kapselung von POSIX-Systemaufrufen wie `mlock`, `munlock`, `madvise` zur Verhinderung des Auslagerns von Schlüsselspeichern in den Swap.
- **Direkte Datei-I/O-Flags:** Plattformspezifische Deskriptor-Flags (`O_DIRECT`, `O_DSYNC`, `posix_fadvise`).
- **Kapselung von Unsafe:** Sämtliche `unsafe`-Aufrufe von `libc` oder System-FFI sind strikt hinter sicheren, typsicheren Rust-Schnittstellen gekapselt. Außerhalb von `contextra-sys` und `contextra-simd` ist `unsafe` im gesamten Workspace verboten (`#![forbid(unsafe_code)]`).

---

## 2. Komponenten & Sichere Wrapper

### 2.1 RAM-Sperrung (`mlock` Wrapper)
Verhindert, dass flüchtiger Arbeitsspeicher, der kryptographische Schlüssel enthält, auf unverschlüsselte Swap-Partitionen geschrieben wird:

```rust
pub struct LockedMemory<T> {
    data: T,
}

impl<T> LockedMemory<T> {
    pub fn new(data: T) -> Result<Self, SysError> {
        // SAFETY: mlock wird mit gültiger Adresse und exakter Größe des Typs aufgerufen.
        unsafe {
            let ptr = &data as *const T as *const libc::c_void;
            let size = std::mem::size_of::<T>();
            if libc::mlock(ptr, size) != 0 {
                return Err(SysError::from_last_os_error());
            }
        }
        Ok(LockedMemory { data })
    }
}

impl<T> Drop for LockedMemory<T> {
    fn drop(&mut self) {
        // SAFETY: munlock gibt den zuvor erfolgreich gesperrten Adressbereich wieder frei.
        unsafe {
            let ptr = &self.data as *const T as *const libc::c_void;
            let size = std::mem::size_of::<T>();
            libc::munlock(ptr, size);
        }
    }
}
```

### 2.2 I/O-Beratung (`madvise` / `fadvise`)
Optimiert sequentielles Lesen bei SSTable-Kompaktionen oder verbietet Read-Ahead für Sparse-Lookups:
```rust
pub fn advise_sequential(fd: std::os::unix::io::RawFd, offset: u64, len: u64) -> Result<(), SysError>;
pub fn advise_random(fd: std::os::unix::io::RawFd, offset: u64, len: u64) -> Result<(), SysError>;
```

---

## 3. Sicherheitsregeln für Unsafe (INV-11)

1. Jeder einzelne `unsafe`-Block MUSS zwingend durch einen vorangestellten `// SAFETY:`-Kommentar begründet werden, der die Einhaltung aller Vorbedingungen explizit darlegt.
2. Keine ungeprüften Zeiger-Dereferenzierungen.
3. Rückgabewerte des Betriebssystems müssen immer auf Fehlerbedingungen (`errno`) geprüft und in typsichere `SysError`-Ergebnisse übersetzt werden.
4. Es werden keine ungesicherten Zeiger oder rohen Dateideskriptoren an Aufrufer nach außen exportiert.
