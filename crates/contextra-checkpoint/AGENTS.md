# AGENTS.md — `contextra-checkpoint`

> **Ring 1** · Klasse `[K]` · Reifegrad `stable` · Unsafe: nein · Rolle: Benannte Checkpoints, RAII-`CheckpointGuard`, Orphan-Reaping, Manifest-Verifikation (BLAKE3).
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-checkpoint --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types`, `mvcc` |
| Features | keine |
| Lints | `#![forbid(unsafe_code)]`, `deny(dead_code)` |

## 2. Abhängigkeiten & Schulden

- Normalkanten sauber (`ports`, `types`). Dev-Kanten beschränkt auf `contextra-testkit`, `criterion`, `proptest`, `serde_json`, `tempfile` (M6: `store` und `mvcc` vollständig entfernt).
- `capabilities.toml` nennt noch `contextra-core`: entfernen.
- Direkte Datei-I/O: `orphan.rs` unterstützt nun `Vfs` (`persist_vfs`, `load_vfs`).
- Zugriff auf den Store nur über `StorageEngine`-Port, nie über `contextra_store`.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `guard.rs` | `CheckpointGuard<S>`: commit/rollback, Drop registriert Orphan |
| `store.rs` | `PersistentCheckpointStore`: create/restore/drop, `allocate_tx`, Snapshot-Pinning |
| `manifest.rs`, `meta.rs` | signiertes/geprüftes Manifest, Metadaten |
| `orphan.rs` | Reaper für verwaiste Checkpoints |
| `hardlink_cloner.rs` | Hardlink-Klone für Snapshots |

## 4. Verträge & Invarianten

1. **Guard wird konsumiert:** `commit()` oder `rollback()`; `Drop` kann nicht async, deshalb Orphan-Registrierung und späterer Reaper (`recover_orphaned_checkpoints`).
2. **Pinning:** gepinnte Checkpoints verhindern Version-GC im Store; beim Löschen unpinnen.
3. **Restore bringt Gelöschtes nicht zurück:** Restore läuft hinter dem `RestoreGuard` von `contextra-erasure` (INV-NO-RESURRECTION-1, INV-RESTORE-HORIZON-1). `checkpoint` selbst kennt keine Löschlogik.
4. Zeit/IDs über `Clock`/`IdGen` (Tests `checkpoint_clock_determinism`, `id_gen_determinism_test`).
5. Manifest mit Prüfsumme; manipuliertes Manifest ⇒ Ablehnung (`manifest_fault_injection.rs`).

## 5. Ports

**Implementiert:** `CheckpointCoordinator`/`Checkpoint`. **Konsumiert:** `StorageEngine`, `Clock`, `IdGen`, `Vfs` (Soll).

## 6. Nebenläufigkeit & Performance

- Offene Guards halten Schreibpuffer und blockieren das MVCC-Watermark: kurz halten, nie über lange `.await`-Ketten.
- Concurrency-Tests: `cache_concurrency_pinning.rs`, `concurrency.rs`.

## 7. Anti-Patterns

- Guard vor langem `.await` anlegen.
- `SystemTime` als `TxId`.
- `contextra_store::checkpoint` importieren (existiert intern, nicht öffentlich).

## 8. Tests & Messung

```bash
cargo test -p contextra-checkpoint --locked
cargo clippy -p contextra-checkpoint --all-targets -- -D warnings
cargo bench -p contextra-checkpoint
```

- Vorhanden: systematische Crash-Tests, Tenant-Isolation, Zeitreise-Korrektheit, Proptests für Guard.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 4.6 | Backup/Snapshot isolieren, Legacy-Löschmarken bereinigen, autarker Wiederanlauf-Test |
| M6 | Dev-Kante `store` entfernen |
