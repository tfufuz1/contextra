# contextra-durable-fs — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/SPEC.md` §5.3, INV-1/6/F-SYNC und `docs/crates/contextra-durable-fs.md`. **Ausgangslage:** VFS, Journal, Health und Scrub existieren; sechs Nicht-Test-Panikmuster bleiben und die Saga-Anker müssen gegen die Soll-Schnittstelle geprüft werden.

1. Kanonisiere `Vfs` und `write_atomic`: Temp-Datei im Zielverzeichnis, `sync_file`, atomisches Rename, Verzeichnis-Sync; jeder Zwischenschritt ist fehlbar und hinterlässt keinen als committed lesbaren Zustand.
2. Zentralisiere `DurableHandle`/`AtomicHealthState`: jedes fehlgeschlagene Flush/Fsync vergiftet irreversibel; alle späteren Writes liefern `PoisonedHandle`.
3. Ersetze Nicht-Test-`unwrap`/`expect` in Journal, Scrub und VFS durch `DurableFsError`; validiere Pfade, Längen und Plattformfälle.
4. Implementiere `FsClass`-Erkennung und sichere Ablehnung/Markierung unsicherer Mounts; schaffe die expliziten fsync-Anker 1–4 für Journal, Gate, Key-Revoke und Ledger.
5. Finalisiere ratenbegrenztes, fehlbares `scrub_file` mit Receipt und Recovery-fähigem Pending-Delete-Journal.

**Abnahme:** T-DFS1–6, Crash-Punkte vor/nach Rename, Poisoning- und Recovery-Tests sowie Format-Golden-Tests.
