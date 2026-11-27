Module: vfs (RealFs und Abstraktion), journal (Append-only mit Prüfsummen), health (AtomicHealthState, fail-closed), scrub, fault_injector, platform, error (DurableFsError).

Harte Invarianten:
- INV-FSYNC-1: Schlägt ein Flush fehl, wird das Handle vergiftet. Kein Schreibzugriff auf demselben Deskriptor danach.
- INV-VFS-ATOMIC-1: Atomare Ersetzung läuft über temporäre Datei, fsync und Rename. Die Reihenfolge darf nicht verändert werden, ohne Crash-Tests zu bestehen.
- Der Health-Zustandsautomat bleibt fail-closed. Übergänge müssen explizit sein.

Deine Stärken: Crash-Matrizen, Fault-Injection, Property-Tests für Journal-Recovery, Verifikation von Verzeichnis-Sync.

Pflichten:
- Jede Änderung an der Schreib- oder Rename-Logik bekommt einen Crash-Test über den `fault_injector`.
- Das Manifest führt `beta`, README `stable`. Belege den Reifegrad mit Testabdeckung oder korrigiere die Angabe.
- Keine Abhängigkeit zu höheren Crates.

Verifikation: `cargo test -p contextra-durable-fs --locked`, `cargo test -p contextra-durable-fs --features inspect`, Benchmarks in `benches/fs_bench.rs`.

Optimierungsfokus: Batching von fsync, Vermeidung unnötiger Syncs, Journal-Kompaktierung ohne Verlust der Recovery-Garantie.
