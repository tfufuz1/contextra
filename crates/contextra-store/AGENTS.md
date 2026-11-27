Das Crate ist das größte im Workspace und hat die meisten Tests (138) und Benchmarks (6).

Module (Auswahl): wal, memtable, sstable, compaction, manifest, lsm, erasure, fsync, io_budget, key_source, system_pressure, tenant_codec, scanner, io_seam.

Harte Regeln:
- Persistenzformate sind heilig. SSTable Format V4 und das WAL-Format dürfen nur mit expliziter Migration und Kompatibilitätstests geändert werden. Feature `legacy-wal-key` zeigt eine bestehende Altkompatibilität.
- Crash-Konsistenz: Jede Änderung am Schreibpfad (WAL, Flush, Manifest-Commit) braucht einen Crash- oder Fault-Injection-Test (`fault-injection`-Feature, `scrub_*`-Muster aus durable-fs).
- Das Manifest wird atomar committet. Kein Zustand ohne gültigen Manifest-Stand.
- HMAC-Kette im WAL darf nicht gelockert werden.
- Konkurrenz: Keine Sperren über `.await`-Punkte. Lock-Scopes minimal halten.

Verifikation: `cargo test -p contextra-store --locked`, `cargo test -p contextra-store --features fault-injection`, `cargo bench -p contextra-store` (sechs Benchmarks als Regressionsbasis).

Optimierungsfokus: Compaction-Strategie nach Lastprofil, Block-Cache-Treffer (Feature `block-cache-v2`), Bloom-Filter-Größen nach gemessener False-Positive-Rate, I/O-Budget-Steuerung unter Systemdruck.
