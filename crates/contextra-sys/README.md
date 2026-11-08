# contextra-sys

Systemnahe OS-Bindings, Durability-Flush, Memory-Locking und Memory-Mapping hinter einer sicheren Rust-Fassade.

Normativitätsklasse: **Klasse A (Isolierter Baustein, B1–B12 nach SPEC v4.0 §5.1 / §2)**
Ring: **Ring 0 (Unsafe-Insel)**
Reifegrad: **Stabil**

---

## Zweck & Rolle
`contextra-sys` stellt autarke, hardware- und betriebssystemnahe Betriebssystem-Primitive bereit:
- **Durability-Synchronisation:** Plattformgerechte Flushes (`fdatasync`, `F_FULLFSYNC`, `FlushFileBuffers`, Verzeichnis-Sync) und Handle-Poisoning via [`DurableFile`] (INV-SYS-FSYNC-1).
- **Speicherabbildung:** Sichere, typisierte Nur-Lese-Speicherabbildung (`mmap_readonly`, `SealedFile`, `SealedMap`).
- **RAM-Sperrung:** Schutz sensibler Speicherseiten vor Paging/Swapping via `mlock` / `VirtualLock` mit Richtlinien (`MemLockPolicy`) und `madvise(MADV_DONTDUMP)`.
- **Tresor-Speicher (Vault):** RAII-verwaltete Regionen (`LockedRegions`), geschützte Allokationen ([`VaultBuffer`]) und flüchtiges Nullsetzen (`write_volatile` + Compiler-Fence).
- **POSIX-Limits:** Dateideskriptor-Limits (`ensure_fd_limit`, `set_fd_limit`) und atomare Umleitung (`reopen_and_dup2`).
- **Zugriffsbeschränkungen:** Strikte Dateizugriffsrechte (`0600` und UID-Verifikation unter Unix, restriktive DACL unter Windows).
- **Fehlermodell:** Strukturierte Fehlercodes über [`Error`] (`thiserror`, `#[non_exhaustive]`) ohne Text-Parsing von Errnos.

---

## Baukasten-Export & Copy-Paste-Anleitung
Dieses Crate ist ein vollkommen autarker Baustein (Klasse A) und hat **keine internen Workspace-Abhängigkeiten** zu anderen `contextra-*`-Crates. Es kann direkt exportiert und in beliebige externe Rust-Projekte kopiert werden:

```bash
python3 tools/export_crate.py contextra-sys --out /pfad/zu/deinem/projekt/contextra-sys
```

Oder alternativ via Cargo aus einem beliebigen Verzeichnis eingebunden werden:
```toml
[dependencies]
contextra-sys = { path = "pfad/zu/contextra-sys" }
```

---

## Externe Abhängigkeiten
- `libc` (0.2, POSIX/Unix)
- `windows-sys` (0.61, Windows-Plattformfunktionen)
- `memmap2` (0.9, Speicherabbildung)
- `thiserror` (2.0, strukturierte Typisierung)
- `tracing` (0.1, optionale Logging-Fassade)

---

## Minimalbeispiel
```rust,no_run
use contextra_sys::{durable_flush_file, DurableFile, SealedFile, VaultBuffer, write_all_durable};
use std::fs::{File, OpenOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Durability mit DurableFile (INV-SYS-FSYNC-1)
    let path = std::path::Path::new("sealed_data.bin");
    {
        let file = OpenOptions::new().create(true).write(true).open(path)?;
        let mut durable = DurableFile::new(file);
        durable.write_all(b"sensible Nutzlast")?;
        durable.sync_all()?;
    }

    // 2. Sealed Memory Mapping (INV-IO-1)
    let file = File::open(path)?;
    let sealed = SealedFile::new_verified(file, 17)?;
    let map = sealed.map_readonly()?;
    assert_eq!(&map[..], b"sensible Nutzlast");

    // 3. VaultBuffer (INV-SYS-MEMLOCK-1)
    let mut secret_buf = VaultBuffer::allocate(32)?;
    secret_buf.copy_from_slice(&[0x42u8; 32]);
    drop(secret_buf); // Automatisch zeroize mit write_volatile + compiler fence vor munlock!

    Ok(())
}
```

Ein vollständiges, autark lauffähiges Beispiel findet sich unter `examples/sys_primitives.rs`.

---

## Nicht-Ziele (Out of Scope)
- **Keine Transaktionsverwaltung:** WAL-Segmentierung, ACID-Semantiken und MVCC gehören in `contextra-store` / `contextra-mvcc`.
- **Keine Kryptografie:** Schlüsselableitung, Signaturerstellung und Verschlüsselungs-Chiffren gehören in `contextra-crypto`.
- **Keine Löschorchestration:** DSGVO-Sagas und Deletion-Proofs gehören ausschließlich in `contextra-erasure`.
- **Keine asynchronen Laufzeitabhängigkeiten:** Kein Tokio, keine Thread-Pools im Crate-Kern; blockierende Operationen müssen vom Aufrufer über `spawn_blocking` gekapselt werden.
