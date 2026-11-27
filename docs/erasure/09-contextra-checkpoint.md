# Implementierungsspezifikation `contextra-checkpoint` (Löschkette – Restore-Sperre und Taint-Politik)

*Ring 0 · Konzept-WP 9 · Abhängig von: 02, 03, 04 §4, 10 · Index: `00-INDEX.md` (K-9, K-23, K-35, K-36, A-3)*

> **Verifikationsgrenze.** Statisch, nichts kompiliert. **✔** gelesen, **[?]** Prüfauftrag.

## 1. Zweck

Ein Checkpoint pinnt eine `seq_no`, legt Metadaten als Store-Schlüssel ab und kann SSTables per Hardlink klonen. Ohne Löschkette ist er ein **Wiederauferstehungsweg**: `restore_checkpoint` rollt denselben Store auf `meta.tx_id` zurück (K-9). Ziel: (1) Restore wird nach einem Erase gesperrt, bis `Completed`; (2) Taint-Politik `Drop`/`LegalHold` (`Rewrite` später); (3) Layer `CheckpointErasure` mit Witness; (4) Hardlink-Klone verschwinden vollständig; (5) Fehler beim Entpinnen werden propagiert.

## 2. Ist-Zustand (✔)

| Aspekt | Befund |
| --- | --- |
| Abhängigkeiten | `types`, `ports`, `tokio`, `tracing`, `parking_lot` (`Cargo.toml`). Kein `store`. Der Kernel-Port (`ErasureKernel`/`ReadGate`) kommt aus `ports` (K-36) |
| Metadaten | `CheckpointMeta{name, collection_id: String, seq_no: u64, tx_id: TxId, metadata: serde_json::Value, created_at}` (`meta.rs:64`); Store-Schlüssel `{namespace}:checkpoint:{name}` (`store.rs`, K-23) |
| Store | `PersistentCheckpointStore<S: StorageEngine>`: `create_checkpoint` (`:515`), `drop_checkpoint` (`:611`), `list_checkpoints` (`:710`), `get_checkpoint` (`:744`), `restore_checkpoint` (`:756`), `create_hardlink_clone` (`:895`). `validate_identifier` (`meta.rs:32`, `pub(crate)`) |
| Restore | `last_tx_id() > meta.tx_id ⇒ Err(Transaction("Serialization barrier violation…"))`, sonst `storage.rollback_to_tx(meta.tx_id)`; danach `list_checkpoints()` (`store.rs:756-782`) |
| Drop | Delete des Metaschlüssels mit eigener TxId, **dann** `unpin_checkpoint(seq_no)`; Fehler des Entpinnens nur `warn` (`:611-640`, ✔ CHK-002) |
| Hardlinks | `hardlink_cloner.rs`: `CheckpointHardlinkCloner`, `DefaultHardlinkCloner` über **`tokio::fs`** (`hard_link`, `copy` bei Cross-Device, `let _ = fs::remove_file(&target_path)` bei Fehler, `:236`). Kein Pfad zum Entfernen **fertiger** Klone außerhalb der Orphan-Logik **[?]** |
| Ports | `ports::Checkpoint` (`restore`, `restore_named_checkpoint`, `drop_named_checkpoint`, …) |

## 3. Dateien

| Datei | Aktion |
| --- | --- |
| `src/erasure.rs` | **NEU**: `CheckpointErasure`, `TaintQuery`, `CheckpointGuard` (Restore-Sperre) |
| `src/store.rs` | Gate-Prüfung in `restore_checkpoint`, `create_checkpoint` (Epoche), `drop_checkpoint` propagiert Entpin-Fehler; `with_gate(Arc<dyn ReadGate>)` |
| `src/hardlink_cloner.rs` | `remove_clone_dir(path)` über `Vfs`-Port **oder** weiterhin `tokio::fs` mit Gate-Ausnahme (§6); `let _ = remove_file` entfernen |
| `src/meta.rs` | `metadata`-Allowlist (§7) |
| `src/lib.rs` | Exporte |
| `tests/checkpoint_erasure_conformance.rs` | **NEU** |

## 4. Restore-Sperre (K-9)

`PersistentCheckpointStore` bekommt `gate: Option<Arc<dyn ReadGate>>` (`with_gate`, Builder-Stil wie `with_clock`). **Fehlt das Gate**, verhält sich der Store wie heute (Kompatibilität, Embedded-Tests), die Engine setzt es immer (11 §3).

`restore_checkpoint(name)`:

1. Metadaten laden (wie heute). Das Laden des Metaschlüssels läuft selbst durch das Gate (der Schlüssel `{ns}:checkpoint:{name}` kann im Scope liegen, siehe §5).
2. `if gate.is_active()`: für **jeden** Eintrag aus `restore_block_list()` (§5.2) mit `blocked.covers(meta)` ⇒ `Err(ContextraError::Erased)`. Kein „not found“, damit Clients den Unterschied sehen (Index §4: Gate-Semantik).
3. Epochenprotokoll: `e1 = gate.epoch()` vor, `e2` nach `rollback_to_tx`-Vorbereitung; bei `e1 != e2` Schritt 2 wiederholen, **bevor** `rollback_to_tx` ausgeführt wird (der Rollback ist nicht rücknehmbar).
4. Danach unverändert (`rollback_to_tx`, `list_checkpoints`).

**Bis `Completed` gilt die Sperre für jeden Checkpoint, dessen `seq_no ≥ scope.min_write_seq`** (Konzept 5.8). Stufe 1 benutzt konservativ `min_write_seq = 0` (§5.1). Nach `Completed` gibt es für markierte Checkpoints nur zwei Zustände: bereits entfernt (`Drop`) oder dauerhaft gesperrt und im Ledger als `LegalHold` ausgewiesen.

Wichtig: `rollback_to_tx` stellt **den ganzen Store** zurück (nicht nur einen Scope). Der Gate gilt nach dem Rollback weiter (Gate ist RAM+Journal, nicht Teil des Rollbacks); erase-gesperrte Schlüssel bleiben nach einem Restore eines **unmarkierten** Checkpoints gesperrt (Test TST-CKPT-ERA-3).

## 5. Taint und Politik

### 5.1 `TaintQuery`

```rust
pub struct TaintQuery { pub tenant: TenantId, pub collections: BTreeSet<String>, pub min_seq: Seq }   // collections leer = alle des Mandanten
```
Der Treiber (10 §6) erzeugt sie aus dem `ResolvedScope`. **Stufe 1:** `min_seq = Seq(0)` (konservativ: alle Checkpoints der betroffenen Collection(s) gelten als markiert). Optimierung O-K1: `min_write_seq` aus der Scope-Auflösung (frühester Schreib-Seq eines Scope-Schlüssels über `PrivilegedRead::get_version_range`, **nicht** vorhanden ⇒ später).

Ein Checkpoint ist markiert, wenn `meta.collection_id ∈ query.collections` (oder `collections` leer **und** Namespace gehört zum Mandanten) **und** `meta.seq_no ≥ query.min_seq`. Mandantenzuordnung: `namespace` des Stores = `{tenant}:{collection}`-ähnlich **[?]** (K-23: Namespace-Aufbau im Engine-Wiring prüfen, `rg "PersistentCheckpointStore::" crates/contextra-engine`); solange nicht belegt, wird nur nach `collection_id` abgeglichen und für Tenant-Scope **alle** Checkpoints aller Namespaces des Mandanten aufgelistet (Engine liefert die Namespace-Liste).

### 5.2 Block-Liste

`CheckpointErasure` hält `blocked: RwLock<Arc<Vec<TaintQuery>>>` (Copy-on-Write). Aufgefüllt in `on_gate`, geleert erst in `complete` (Treiber ruft `release(id)`, nicht bei `Sealed`). Der Restore-Check in §4 liest diese Liste, **nicht** nur das Gate, weil ein Checkpoint-Restore auch Schlüssel zurückbringt, die das Gate bereits erlaubt (`seq > up_to` nach `Sealed`).

### 5.3 Politik (`CheckpointTaintPolicy`)

| Politik | Wirkung in `purge` |
| --- | --- |
| `Drop` (Default) | Für jeden markierten Checkpoint: `drop_checkpoint_for_erase(name)` (§5.4). Karenz (Konfiguration `ErasureConfig.checkpoint_grace`, Default 0) schiebt den Start hinaus, G1 gilt sofort |
| `LegalHold` | Checkpoint bleibt, Restore bleibt **dauerhaft** gesperrt, Ausnahme `ErasureException::LegalHold{checkpoint}` im Seal; Garantie ohne `purged` für diese Schicht |
| `Rewrite` | **Stufe 1: nicht implementiert.** `accept` im Treiber lehnt `Rewrite` ab (`InvalidTarget("checkpoint rewrite unsupported")`), bis der Store eine gefilterte Klon-API anbietet (O-K2, eigener Nachtrag zu 02/03) |

### 5.4 `drop_checkpoint_for_erase(name)`

Reihenfolge (jeder Schritt propagiert Fehler, **kein** `warn` als Ersatz):
1. Metaschlüssel löschen (nur zulässig, wenn der Schlüssel **nicht** im Scope liegt; sonst fällt er durch Store-Purge, dann Schritt übersprungen).
2. `unpin_checkpoint(seq_no)` ⇒ **Fehler = `Err(Retryable)`** (heute nur `warn`, `store.rs:631-638`); der Auftrag bleibt in `Purging`. Zusätzlich `SnapshotRegistry`-Pin laut 04 §4 (Pin-Alter) beachten: ein gereapter Pin ist **kein** Fehler, sondern „bereits entpinnt“ (idempotent).
3. Hardlink-Klonverzeichnis (falls vorhanden) entfernen (§6).
4. `list_checkpoints()` (Cache) aktualisieren.

## 6. Hardlink-Klone

Ein Klon legt zu jeder SSTable einen zweiten Namen an. Die Datei verschwindet erst, wenn **alle** Namen weg sind. Der Store entfernt nur **seinen** Namen (Pending-Delete, 03 §8).

- **Entfernen des Klonverzeichnisses:** `CheckpointErasure::purge` listet die Klon-Dateien (`read_dir`), entfernt jede Datei einzeln und synchronisiert das Verzeichnis, entfernt das Verzeichnis. Fehler einzelner Dateien ⇒ `Err(Retryable)`; die Liste der noch vorhandenen Dateien steht im Cursor (`[ver][work_hash][done u64]`, Arbeitsliste = sortierte Dateinamen).
- **I/O-Weg:** G-4 verbietet `std::fs`/`tokio::fs` in den **neuen** Dateien. Deshalb bekommt `CheckpointErasure` einen `Arc<dyn Vfs>` und entfernt über `vfs.remove_file` + `vfs.sync_dir`. `DefaultHardlinkCloner` (bestehend, `tokio::fs`) bleibt unberührt (Ausnahme der G-4-Liste); die Doku in `hardlink_cloner.rs` erhält „# Crash“-Abschnitt.
- **Witness:** Der Store-Scanner (SSTable/WAL) durchsucht **auch** Klonverzeichnisse (13 §6: Scanner-Wurzelliste enthält `checkpoint_dirs`). Ein Klon mit Scope-Daten ist `remaining > 0`.
- **Cross-Device-Kopien** (`is_cross_device_error` ⇒ `copy`): sind eigene Bytes und werden wie Dateien entfernt.

## 7. Metadaten

`CheckpointMeta.metadata` ist freies JSON (B-17). Stufe 1: Allowlist-Prüfung beim Anlegen (`metadata` darf nur Schlüssel enthalten, die in `CheckpointMetadataPolicy.allowed_keys` stehen, Default: `model_version`, `prompt_id`, `label`) **oder** Versiegelung (Stufe 2, `RecordSealer`). Nicht erlaubter Schlüssel ⇒ `Err(InvalidInput)`. Altbestand: Beim Erase wird `metadata` im markierten Checkpoint mit dem Checkpoint verworfen (Politik Drop), bei `LegalHold` nicht angefasst. Prüfauftrag O-K3: `rg "metadata" crates/contextra-engine crates/contextra-db | rg -i checkpoint` für Produktivnutzung (Konzept 12.1 Nr. 7).

## 8. Layer

```rust
pub struct CheckpointErasure<S: StorageEngine> {
    store: Arc<PersistentCheckpointStore<S>>,   // je Namespace; Engine registriert alle
    vfs: Arc<dyn Vfs>,
    clone_root: PathBuf,                        // Wurzel der Hardlink-Klone
    blocked: RwLock<Arc<Vec<TaintQuery>>>,
}
impl<S: StorageEngine + 'static> ErasureLayer for CheckpointErasure<S> {
    fn layer(&self) -> WitnessLayer { WitnessLayer::checkpoints() }   // A-3, Custom("Checkpoints")
    // on_gate: blocked erweitern (G1, sofort), Pin-Reaping NICHT hier
    // purge:   Politik anwenden (§5.3), je Checkpoint ein Schritt, Cursor = done u64 + work_hash
    // scanner: Metadaten- und Klon-Scan (Scope-Nadeln), Positivkontrolle über Testklon
}
```

`expected_layers` enthält `Checkpoints` nur, wenn der Namespace des Scopes mindestens einen Checkpoint hat **oder** der Store ein Klonverzeichnis kennt; sonst **nicht registriert** (leere registrierte Schicht meldet `remaining = 0`, Index §4).

## 9. Gate-Ende (I-9)

`complete` ist erst zulässig, wenn keine markierten Checkpoints mehr existieren **oder** alle markierten per `LegalHold` ausgewiesen sind (dann bleibt der Gate-Eintrag `Erased{up_to}` als Intervall dauerhaft stehen, 03 §6.5). Der Treiber fragt `CheckpointErasure::remaining_tainted()`.

## 10. Tests

| ID | Inhalt |
| --- | --- |
| TST-CKPT-ERA-1…7 | Konformanz C1–C7 |
| TST-CKPT-ERA-2 | Restore eines markierten Checkpoints nach `on_gate` ⇒ `Erased`; vorher funktionierte er |
| TST-CKPT-ERA-3 | Restore eines **unmarkierten** Checkpoints (anderer Namespace) bringt gesperrte Schlüssel nicht zurück |
| TST-CKPT-ERA-4 | `Drop`: Checkpoint, Pin, Klonverzeichnis weg; zweiter Lauf No-op |
| TST-CKPT-ERA-5 | `unpin_checkpoint` schlägt fehl ⇒ `Err(Retryable)`, kein `warn`-Erfolg (Mock-Storage) |
| TST-CKPT-ERA-6 | `LegalHold`: Restore bleibt gesperrt, Seal enthält `LegalHold{checkpoint}`, Garantie ohne `purged` |
| TST-CKPT-ERA-7 | Hardlink: zwei Namen, Store entfernt einen ⇒ Witness meldet `remaining > 0` bis der Klon entfernt ist; danach 0 |
| TST-CKPT-ERA-8 | `Rewrite` wird im `accept` abgelehnt |
| TST-CKPT-ERA-9 | Epochenwechsel zwischen Check und Rollback führt zur erneuten Prüfung (Loom-Modell, 13 §8) |
| TST-CKPT-ERA-10 | Metadaten-Allowlist |

## 11. Arbeitspakete

| WP | Inhalt |
| --- | --- |
| 9.1 | `with_gate`, Restore-Sperre, Block-Liste |
| 9.2 | `CheckpointErasure` (`on_gate`, `purge` Drop/LegalHold), Entpin-Propagation |
| 9.3 | Klonverzeichnis-Entfernung über `Vfs`, Scanner-Wurzeln |
| 9.4 | Metadaten-Allowlist |
| 9.5 | (später) `Rewrite` |

## 12. Definition of Done

Konformanz grün; Restore-Sperre getestet; kein `let _ =` in neuem Code; `warn`-only-Pfad beim Entpinnen ersetzt; Bestandstests (`store/tests.rs`, `guard/tests.rs`, `orphan/tests.rs`) grün.

## 13. Offene Punkte

| Nr. | Frage | Vorschlag |
| --- | --- | --- |
| O-K1 | `min_write_seq` | später über Versionsbereich-Lesen im Kernel |
| O-K2 | `Rewrite` | eigener Nachtrag |
| O-K3 | Produktive Nutzung von `metadata` | siehe §7 |
| O-K4 | Namespace-Aufbau je Mandant | siehe §5.1 |

## Revision 2 (28.11.2026, Audit gegen HEAD `e12873de6`; Details in 14)

**Stand.** `PersistentCheckpointStore::with_gate` und `ensure_restore_allowed` (`store.rs:463-465, 799-810`) sind umgesetzt ✔, sie prüfen aber nur den Meta-Schlüssel `{namespace}:checkpoint:{name}` gegen **aktive** Gates und bei Epochenwechsel (`restore_checkpoint`, `store.rs:764, 784`). **Nicht umgesetzt:** `TaintQuery`, Politik (`Drop`/`Rewrite`/`LegalHold`), `drop_checkpoint_for_erase`, Layer `CheckpointErasure`, Zeuge `Checkpoints`.

**K-54 (S1).** Nach `Completed` ist der Gate-Eintrag entfernt (I-9). Ein älterer Checkpoint wird dann nicht mehr blockiert; die Barriere `last_tx > meta.tx_id` (`store.rs:773`) greift nicht, weil Erase keine `TxId` erzeugt **[?]** (`TST-CKPT-RESTORE-3` entscheidet). Folge: Wiederherstellen stellt gelöschte Daten wieder her.

**Verbindliche Ergänzungen (A-5, 14 §3.9).**

1. **Erasure-Floor:** `restore_checkpoint(name)` ⇒ `Erased`, wenn ein abgeschlossener Auftrag mit `gated_seq > meta.seq_no` existiert und der Checkpoint nicht `Rewritten` ist. Der Floor stammt aus dem Journal-Summary (über Rollover erhalten) und wird beim Start neu aufgebaut (CP-23).
2. **Pflicht-Layer:** `CheckpointErasure` ist in Regulated Teil von `required_layers`; `Sealed` verlangt `Checkpoints.remaining == 0`.
3. **Reihenfolge:** Checkpoint-Bereinigung **nach** `purge_store` (Index §4), vor Verifikation. Hardlink-Klone: Löschen eines Klons löscht nicht die Datei, solange ein weiterer Link existiert; der Zeuge zählt Dateien mit Link-Zähler > 1 als Rest, bis der Quell-Checkpoint selbst bereinigt ist.
4. **Restore ⇄ Layer-Purge:** exklusive Sperre, solange ein offener Auftrag den Scope berührt (K-9), plus Loom-Modell (c) aus 14 §5.
5. **Tests:** `TST-CKPT-RESTORE-3` (Floor), `TST-CKPT-HLINK-1` (Link-Zähler), `TST-CKPT-CRASH-1` (CP-29).
