# Loom Concurrency Coverage Audit (Oktober 2026)

**Stand:** 2026-10-08
**Auftrag:** J-05 / Task-Karte `T-2026-0205`
**Zweck:** Bestandsaufnahme aller `#[cfg(loom)]`-Dateien im Repository, deren CI-Verdrahtung in `.github/workflows/loom-gate.yml`, `justfile` und `xtask/src/loom_run.rs` sowie deren lokale Ausführungsergebnisse.

---

## 1. Inventar aller `#[cfg(loom)]`-Dateien

| Crate | Datei | In `loom-gate.yml` | In `justfile` | In `xtask/src/loom_run.rs` | Lokales Ergebnis |
|---|---|---|---|---|---|
| `contextra-store` | `crates/contextra-store/tests/loom_commit_flush_visibility.rs` | nein | nein | ja | **grün** (1 passed in 0.01s) |
| `contextra-store` | `crates/contextra-store/tests/loom_group_commit.rs` | ja | nein | ja | **grün** (1 passed in 0.02s) |
| `contextra-store` | `crates/contextra-store/tests/loom_group_commit_handoff.rs` | ja | ja (`just loom-store`) | ja | **grün** (2 passed in 0.08s) |
| `contextra-store` | `crates/contextra-store/src/wal/mod.rs` | nein (Inlined conditional) | nein | nein (Inlined file) | **grün** (wird durch store-Loom-Tests mitgeprüft) |
| `contextra-crypto` | `crates/contextra-crypto/tests/loom_eviction_worker_shutdown_race.rs` | nein | nein | ja | **grün** (2 passed in 75.79s) |
| `contextra-crypto` | `crates/contextra-crypto/tests/loom_kv_deferred_zeroize.rs` | ja | nein | ja | **grün** (2 passed in 0.67s) |
| `contextra-mvcc` | `crates/contextra-mvcc/tests/loom_floor.rs` | nein | nein | ja | **grün** (1 passed in 0.00s) |
| `contextra-mvcc` | `crates/contextra-mvcc/tests/loom_snapshot_registry.rs` | nein | nein | ja | **grün** (2 passed in 0.04s) |
| `contextra-vector` | `crates/contextra-vector/tests/loom_quantizer_race_test.rs` | ja | nein | ja (ausgeschlossen) | **rot** (Tokio Runtime Panic `!enabled` unter Loom) |
| `contextra-db` | `crates/contextra-db/tests/loom_relate_n_ary.rs` | nein | nein | ja (ausgeschlossen) | **nicht ausgeführt** (Kompilierfehler in `contextra-engine` unter `cfg(loom)`) |
| `contextra-checkpoint` | `crates/contextra-checkpoint/src/hardlink_cloner.rs` | nein (Inlined conditional) | nein | nein (Inlined file) | **grün** (keine eigenen Loom-Permutationen) |
| `contextra-engine` | `crates/contextra-engine/src/collection/kv_lock.rs` | nein (Inlined conditional) | nein | nein (Inlined file) | **nicht ausgeführt** (Kompilierfehler in `contextra-engine` unter `cfg(loom)`) |

> **Hinweis zu `contextra-mvcc`**: Die beiden MVCC-Dateien `loom_floor.rs` und `loom_snapshot_registry.rs` liegen außerhalb des Bearbeitungsscopes von Auftrag J-05. Sie wurden lediglich auditaktiv mitgeprüft und laufen lokal grün.

---

## 2. Vorschlag CI-Verdrahtung (Empfehlung zur Umsetzung in separaten PRs)

*(Hinweis: `.github/workflows/loom-gate.yml`, `justfile` und `xtask/` sind schreibgeschützt und wurden im Rahmen dieses Auftrags nicht geändert.)*

1. **Erweiterung der CI-Matrix in `.github/workflows/loom-gate.yml`**:
   - In Job `loom-store`: Hinzufügen des Schritts für `loom_commit_flush_visibility`:
     ```yaml
     - name: Run Loom Test — contextra-store (loom_commit_flush_visibility)
       run: RUSTFLAGS="--cfg loom" cargo test -p contextra-store --test loom_commit_flush_visibility --release
     ```
   - In Job `loom-crypto`: Hinzufügen des Schritts für `loom_eviction_worker_shutdown_race`:
     ```yaml
     - name: Run Loom Test — contextra-crypto (loom_eviction_worker_shutdown_race)
       run: RUSTFLAGS="--cfg loom" cargo test -p contextra-crypto --test loom_eviction_worker_shutdown_race --release
     ```
   - Neuer Job `loom-mvcc`: Aufnahme von `loom_floor` und `loom_snapshot_registry`:
     ```yaml
     loom-mvcc:
       runs-on: ubuntu-24.04
       steps:
         - uses: actions/checkout@v4
         - name: Run Loom Test — contextra-mvcc (loom_floor)
           run: RUSTFLAGS="--cfg loom" cargo test -p contextra-mvcc --test loom_floor --release
         - name: Run Loom Test — contextra-mvcc (loom_snapshot_registry)
           run: RUSTFLAGS="--cfg loom" cargo test -p contextra-mvcc --test loom_snapshot_registry --release
     ```

2. **Erweiterung von `justfile`**:
   - Ergänzung der Bequemlichkeitstargets `loom-store` und `loom-crypto`:
     ```just
     loom-crypto:
         RUSTFLAGS="--cfg loom" cargo test -p contextra-crypto --test loom_eviction_worker_shutdown_race --release
         RUSTFLAGS="--cfg loom" cargo test -p contextra-crypto --test loom_kv_deferred_zeroize --release
     ```

3. **Behebung von Sonderfällen in `xtask/src/loom_run.rs`**:
   - `loom_quantizer_race_test.rs`: Das Modell verwendet im Testaufbau `tokio::runtime::Builder::new_current_thread().enable_all()`. Unter `RUSTFLAGS="--cfg loom"` wird Tokio durch Loom gemockt, weshalb `enable_all()` mit `assertion failed: !enabled` abbricht. Hier sollte im Test-Setup `loom::tokio::runtime::Runtime` bzw. ein Loom-kompatibler Mock-Executor genutzt werden.
   - `loom_relate_n_ary.rs`: Crate `contextra-engine` kapselt Teile der `Collection`-Struktur mit `#[cfg(not(loom))]`, wodurch abhängige Datenstrukturen unter `cfg(loom)` nicht kompilieren. Eine Säuberung der `#[cfg(loom)]`-Guards in `contextra-engine/src/collection/mod.rs` wird empfohlen.
