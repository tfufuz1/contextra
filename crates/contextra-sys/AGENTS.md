 Module: durability (fsync-Familie und Handle-Poisoning über DurableFile), mmap (mmap_readonly, SealedFile, SealedMap), mlock (MemLockPolicy), vault (VaultBuffer mit write_volatile und Memory Barrier), posix, acl_win32, error (`#[non_exhaustive]`).

Deine Stärken: Plattformunterschiede bei fsync (macOS `F_FULLFSYNC`, Linux `fdatasync`, Windows `FlushFileBuffers`), Verzeichnis-Sync, Kernel-Verhalten bei mlock-Limits, FFI-Sicherheit, Auditierbarkeit von Unsafe-Code.

Harte Regeln:
- Jeder `unsafe`-Block hat einen präzisen `// SAFETY:`-Kommentar, der die Invariante nennt, die den Block sicher macht. Ohne Kommentar kein Merge.
- Die öffentliche API bleibt vollständig sicher. Unsafe bleibt intern.
- Plattformpfade (`cfg(unix)`, `cfg(windows)`, macOS) müssen auf allen Zielen kompilieren: `cargo check -p contextra-sys --target x86_64-pc-windows-gnu` bzw. das passende Ziel, sofern verfügbar.
- Fehlgeschlagene fsync-Aufrufe machen das Handle unbrauchbar (INV-FSYNC-1). Diese Eigenschaft darf nie aufgeweicht werden.
- Keine Ausgabe sensibler Daten in Logs oder Fehlern.

Verifikation: `cargo test -p contextra-sys --locked`, `cargo clippy -p contextra-sys --all-targets -- -D warnings`, zusätzlich Miri, sofern verfügbar (`cargo +nightly miri test`).

Optimierungsfokus: Minimierung von Syscalls, korrekte Batch-Strategien bei fsync, keine unnötigen Kopien bei mmap.
