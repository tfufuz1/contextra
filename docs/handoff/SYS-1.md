# Übergabebericht: Mission `SYS-1` · Systemabstraktionen: POSIX, Durability, mlock, Vault, mmap (Unsafe-Insel)

## 1. Zusammenfassung & Zielerreichung
Das Crate `contextra-sys` (Ring 0, Klasse A Baustein, Unsafe-Insel) wurde vollständig gemäß SPEC §10.1 und den Architekturinvarianten gehärtet:
- **Typisiertes Fehlermodell (`error.rs`):** Crate-weites `Error` (`thiserror`, `#[non_exhaustive]`) mit strukturierten OS-Fehlern (`SyncFailed(i32)`, `MemoryLockFailed`, `PermissionDenied`, `InvalidInput`, `Io`, `Unsupported`, `HandlePoisoned`) und `Result<T>`-Typalias.
- **INV-SYS-FSYNC-1 (Durability & Handle-Poisoning):** `DurableFile` kapselt Datei-Handles und erzwingt unwiderrufliches Handle-Poisoning bei fehlgeschlagenen Flushes. `EINTR` ist der einzige wiederholte Syscall; jeder sonstige Fehler vergiftet das Handle dauerhaft.
- **INV-SYS-MEMLOCK-1 (VaultBuffer & Zeroization):** `VaultBuffer` allozierte Speicherpuffer pinnen Seiten im RAM (`mlock`), schließen Core-Dumps aus (`MADV_DONTDUMP`), führen vor der Freigabe beim Drop ein nicht-wegoptimierbares Zeroize (`write_volatile` + `compiler_fence(SeqCst)`) aus und rufen erst danach `munlock` auf. Bei fehlgeschlagenem `mlock` schlägt die Allokation fail-closed fehl (`Error::MemoryLockFailed`).
- **Mmap-Härtung (INV-IO-1):** `SealedMap::get_subslice` mit sicher berechneten Grenzen (`checked_add`) zur Verhinderung von Integer-Überläufen.
- **Fault-Injection-Tests:** Eigene Injektions-Hooks unter `#[cfg(test)]` belegen sowohl `INV-SYS-FSYNC-1` (`test_inv_sys_fsync_1_poisoning`) als auch `INV-SYS-MEMLOCK-1` (`test_inv_sys_memlock_1_vault_protection`).

## 2. Invarianten & Status
- **`INV-SYS-FSYNC-1`**: Erfüllt. Schlägt fsync fehl, wird `Error::SyncFailed(errno)` zurückgegeben und das Handle dauerhaft vergiftet. Belegt durch `test_inv_sys_fsync_1_poisoning`.
- **`INV-SYS-MEMLOCK-1`**: Erfüllt. Speicherseiten werden via `mlock` gesperrt, `MADV_DONTDUMP` gesetzt, Drop führt volatile Zeroize + Memory Barrier vor `munlock` aus. Belegt durch `test_inv_sys_memlock_1_vault_protection`.
- **`INV-IO-1`**: Erfüllt. `SealedFile` / `SealedMap` erzwingen Längen- und Integritätsprüfung vor Mmap.
- **`INV-SAFE-FACADE`**: Erfüllt. Sämtliche öffentlichen Schnittstellen sind 100 % safe Rust; alle internen `unsafe`-Blöcke tragen begründete `// SAFETY:`-Dokumentationen.

## 3. Funktionsinventar-Diff (Substanzerhalt E-05/E-06)
- **Entfernte Signaturen:** Keine Funktionalität gelöscht. Bei `durable_flush_file`, `durable_flush_dir`, `write_all_durable` und `mem_lock_with_policy` wurde der Rückgabetyp im Quelltext explizit als `std::io::Result` (`IoResult`) ausgeschrieben, um Verwechslungen mit dem neuen `crate::Result` zu vermeiden. Die API-Signatur bleibt identisch.
- **Hinzugefügte Funktionen (Auswahl):**
  - `DurableFile::new`, `DurableFile::sync_all`, `DurableFile::sync_data`, `DurableFile::write_all`, `DurableFile::get_ref`, `DurableFile::is_poisoned`
  - `VaultBuffer::allocate`, `VaultBuffer::allocate_with_policy`, `VaultBuffer::from_slice`, `VaultBuffer::as_slice`, `VaultBuffer::as_mut_slice`, `VaultBuffer::is_locked`
  - `apply_dont_dump`
  - `mem_lock_typed`
  - `SealedMap::get_subslice`
  - `Error` Varianten und `From`-Konvertierungen

## 4. Verifikations-Gates & Diagnostics
- `cargo fmt --check -p contextra-sys`: Grün.
- `cargo clippy -p contextra-sys --all-targets -- -D warnings`: Grün (0 Warnungen).
- `cargo clippy -p contextra-sys --all-targets --no-default-features -- -D warnings`: Grün.
- `cargo test -p contextra-sys`: Grün (33 Unit-Tests, 8 Integration-Tests, 1 Doc-Test).
- `cargo bench -p contextra-sys --no-run`: Grün.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p contextra-sys --no-deps`: Grün.
- `zero_panic_analyzer.py --crate contextra-sys --fail-on-prod`: 0 Verstöße.
- `unsafe_boundary_auditor.py --crate contextra-sys`: 0 unbegründete Blöcke.
- `export_crate.py contextra-sys --out /tmp/export-contextra-sys`: Gate G3 bestanden.
