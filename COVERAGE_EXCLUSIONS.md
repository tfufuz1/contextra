# Coverage Exclusions: contextra-sys

Folgende Code-Pfade sind plattformbedingt nicht auf allen Zielsystemen gleichzeitig abdeckbar
und werden in der Code-Coverage von `contextra-sys` dokumentiert:

1. **`crates/contextra-sys/src/durability.rs`:**
   - macOS-spezifischer Pfad (`#[cfg(target_os = "macos")]` mit `libc::fcntl(fd, libc::F_FULLFSYNC, 0)`):
     Wird ausschließlich auf macOS-Builds ausgeführt und ist auf Linux CI nicht testbar.
   - Generischer POSIX-Fallback (`#[cfg(all(unix, not(any(target_os = "linux", target_os = "macos"))))]`):
     Wird auf Linux/macOS nicht durchlaufen.

2. **`crates/contextra-sys/src/fs_probe.rs`:**
   - macOS-spezifischer Pfad (`#[cfg(target_os = "macos")]` zur APFS-Erkennung):
     Wird auf Linux CI nicht erreicht.
   - Seltene Dateisystem-Branches in Linux `statfs` (z. B. XFS, Btrfs, ZFS, NFS), wenn CI auf OverlayFS/Ext4 läuft.

3. **`crates/contextra-sys/src/hardening.rs`:**
   - Nicht-Linux Unix-Fallbacks (`#[cfg(not(target_os = "linux"))]`).
