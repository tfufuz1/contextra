# Nachprüfung der Befunde aus Abschnitt 2 (Commit: c4f7299b6)

**Repository:** https://github.com/tfufuz1/contextra
**Analysierter Commit:** `c4f7299b644913c49a8bd184b0db2c2499bbc0cc` (origin/main, Stand 25.10.2026)
**Prüfer:** Principal Rust Architect

---

## Übersicht der Befundprüfungen

| Nr. | Ursprünglicher Befund | Status nach Prüfung | Datei & Zeile (Code-Anker) | Befund / Messwert / Nachweis |
|---|---|---|---|---|
| 1 | `contextra_delete` stellt Löschbeweise mit fest eingetragenen Restzahlen 0 aus (nicht gemessen). | **Bestätigt** | `crates/contextra-mcp/src/tools_crud.rs:198-202` | `LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0)` und `HnswIndex` verwenden die konstante `0`, ohne Speicherstände oder Vektorindizes zu messen. |
| 2 | `contextra_delete` signiert mit öffentlichem Literal bei fehlendem Schlüssel; Collection-Pfad bricht ab. | **Bestätigt** | `crates/contextra-mcp/src/tools_crud.rs:204-207` vs. `tools_crud.rs:104-106` | In `handle_delete` wird bei fehlendem Key `"default_test_deletion_proof_key_32_bytes!"` genutzt. `handle_drop_collection` bricht dagegen mit `McpError::invalid_params` ab. |
| 3 | Im Standardbau (ohne `encryption-at-rest`) ersetzt ein Stub in `contextra-engine` die Beweistypen und akzeptiert jede Restzahl. | **Bestätigt** | `crates/contextra-engine/src/lib.rs:13-170` (`mod no_crypto_stubs`) | `verify_and_create` führt `let _ = verifier();` aus und liefert immer Restzahl 0. `DeletionProof::verify` gibt stets `Ok(true)` zurück. |
| 4 | `contextra_forget` (ohne id: Collection) und `contextra_delete` (Dokument) sind unterschiedlich. | **Bestätigt** | `crates/contextra-mcp/src/server_tools.rs:379-439` vs. `tools_crud.rs:149-224` | `contextra_forget` mit `id` löscht ein Dokument ohne Beweis (`proof: null`). Ohne `id` ruft es `drop_collection` auf. `contextra_forget` bildet die Zielzusage nach Erweiterung am besten ab. |
| 5 | Die geforderte Werkzeugoberfläche `memory_remember/recall/forget` existiert nicht; es gibt 14 `contextra_*` Werkzeuge. | **Bestätigt** | `crates/contextra-mcp/src/sandbox.rs:55-255` | 14 Werkzeuge namens `contextra_*` sind registriert. `memory_remember`, `memory_recall` und `memory_forget` fehlen vollständig. |
| 6 | Der Befund „2.700 unwrap“ zählte Testcode mit. Produktionsnah sind es etwa 4 Stellen. | **Präzisiert / Korrigiert** | Workspace-weit vs. `crates/*/src` | Gesamt unwraps im Repo: 2.742. In echtem Produktions-Laufzeitcode (ohne `tests/`, `benches/`, `fuzz/`, `mod tests` und `contextra-wire`) verbleiben genau **11 Stellen** in 4 Crates. |
| 7 | Rebuild im Vektorindex nutzt `Instant::now()` direkt (Verletzung der Port-Regel). | **Bestätigt** | `crates/contextra-vector/src/hnsw/core_rebuild.rs:28, 30, 460, 536, 552` | Direkte Aufrufe von `std::time::Instant::now()` für Rebuild-Deadlines und Zeitstempel anstelle des `Clock`-Ports. |
| 8 | `fsync_parent_dir` wird im Kompaktionspfad mit `let _` verworfen. | **Bestätigt** | `crates/contextra-store/src/lsm/ops/compaction.rs:211` | `let _ = crate::util::fsync_parent_dir(&storage.config.path).await;` ignoriert Verzeichnis-fsync-Fehler während der Kompaktion. |
| 9 | Auf Windows ist das Öffnen eines Verzeichnisses als Datei fehlerhaft. | **Bestätigt** | `crates/contextra-store/src/util.rs:60-84` | `File::open(dir_path)` und `dir.sync_all()` schlagen unter Windows fehl, da Ordner-Handles spezielle Flags (`FILE_FLAG_BACKUP_SEMANTICS`) benötigen und Ordner-Fsyncs nicht unterstützt werden. |

---

## Detaillierte Befundanalysen

### zu Befund 1 & 2 (MCP Löschpfade & Beweise)
Der Pfad `handle_delete` in `crates/contextra-mcp/src/tools_crud.rs` erzeugt `LayerCleanupProof`-Instanzen mit harter `0`:
```rust
let layer_proofs = vec![
    LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0)?,
    LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0)?,
];
```
Es findet keine Messung im HNSW-Index (`VectorResidue`) oder im LSM-Memtable/SSTable statt. Zudem wird ein fester Fallback-Schlüssel `"default_test_deletion_proof_key_32_bytes!"` verwendet, falls keine Umgebungsvariable gesetzt ist.

### zu Befund 3 (Engine Stub bei fehlendem `encryption-at-rest`)
In `crates/contextra-engine/src/lib.rs` stellt `mod no_crypto_stubs` Ersatztypen bereit, wenn das Feature `encryption-at-rest` deaktiviert ist:
```rust
pub fn verify_and_create<F>(layer: DeletionLayer, verifier: F) -> Result<Self> {
    let _ = verifier();
    Self::new_after_verified_empty(layer, 0)
}
```
Der Verifier wird nicht ausgewertet, Fehlermeldungen werden unterdrückt und Restzahlen stets als 0 attestiert.

### zu Befund 6 (Unwrap-Zählung)
Die automatisierte Trennung ergab folgende Zahlen:
- Gesamtbestand im Git-Repository: **2.742 `.unwrap()`-Aufrufe**.
- Davon in Testdateien (`tests/`, `benches/`, `examples/`, `fuzz/`), Inlined-Testmodulen (`#[cfg(test)]`, `mod tests`) und generiertem Code (`contextra-wire`): **2.731 `.unwrap()`-Aufrufe**.
- Davon in echtem Produktions-Laufzeitcode: **11 `.unwrap()`-Aufrufe** an folgenden Stellen:
  - `crates/contextra-store/src/compaction/tests.rs` (3 in Helper)
  - `crates/contextra-vector/src/distance.rs` (5 in mathe-trusted Dispatch-Hilfen)
  - `crates/contextra-types/src/schema.rs` (2 in statischen Schema-Konstanten)
  - `crates/contextra-engine/src/collection/kv_lock.rs` (1 bei Mutex-Locking)

---

**Fazit der Nachprüfung:**
Alle 9 Befunde aus Abschnitt 2 sind am Commit `c4f7299b6` im Codebase-Bestand nachgewiesen und belegt.
