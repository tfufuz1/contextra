# Implementierungsplan: contextra-storage

**Ebene:** L2 · **Ziel-Zustand:** Speicherkern nach SPEC §5 (Schreibpfad), §6 (Lesepfad), §8 (Löschung), §9 (Recovery). Durabilität, Replay und Löschnachweis sind verdrahtet, nicht nur vorhanden.
**Basis:** SPEC R14 §2.1, §3–§10, §14, §15; R15 §2.1 (B-01…B-13), DEC-41…DEC-51, INV-FAC-1…5. Die Fassade `lib.rs` (1419 Zeilen) wurde vollständig gelesen; die Module in `engine/`, `erase/`, `recovery/` nur in Struktur.

## 1. Ist-Befund (gemessen und gelesen)

- 164 Quelldateien, 16 824 Zeilen, 110 `#[test]`. Der Verbotsgrep auf Produktionscode ist sauber (0 Treffer). Das Crate ist also strukturell vorbildlich, aber nicht baubar.
- **Baubarkeit:** `contextra_storage_ports::traits::` (42 Stellen) und `contextra_storage_ports::limits::` (2 Stellen) verweisen auf ein Crate, das es nicht gibt. `pub use contextra_storage_ports as ports;` in `lib.rs:3`. Der Import-Fehler ist die erste Blockade.
- **B-01 Durabilität:** `apply` (Z. 447–657) fügt in `Memtable` ein und publiziert eine `Version` mit `durable_stamp = stamp`. Es gibt keinen Aufruf von `RedoWriter`, `sync_data` oder `Committer` im Pfad. `Receipt.durable` ist `true`, ohne dass etwas geschrieben wurde.
- **B-02 Replay:** `recovery/replay.rs` hat keinen Aufrufer außerhalb seiner Datei. `open_instance` (`recovery/open.rs`) übergibt keine Redo-Frames.
- **B-03 Löschung:** `erase_cluster` (Z. 849–1000) ruft `maint::purge` nicht auf. `complete_purge` bekommt `[0u8; 32]`. `finalize` bekommt `CompletionEvidence { anchor_advanced: true, verified: true, owners_attested: true, graveyard_empty: true }` als Literale.
- **B-04 Verschlüsselung:** `flush_cluster` (Z. ≈ 745–790) und `get` (Z. ≈ 1155) nutzen `SealMode::Plain`, `key: None`, `PlainNoopCipher`. `PlainNoopCipher` (Z. 97–122) ist identische Abbildung. Sealed-Cluster sind Klartext.
- **B-05 Authentisierung:** `CoreMac` (Z. 126–141) ist CRC32C mit Schlüsselvorsatz, achtfach expandiert. Lineare Prüfsumme, keine MAC.
- **B-06 Ledger-Schlüssel:** `ledger_key` wird bei jedem `open` aus `Entropy` gezogen (Z. 227–229).
- **B-07 RAM-Kontrollpfade:** `restrict_cluster` (Z. 414), `lift_restriction` (430), `link_clusters` (407), `branch_fork` (706), `branch_drop` (727) ändern nur `EngineState`. Keine Persistenz.
- **B-08 Space-Metadaten:** `create_space` (Z. 304) legt `SpaceSpec` nur in `st.spaces` ab. `CatalogOp::SpaceAdd` trägt `(space, protection)`. Nach `open` ist die Zuordnung leer.
- **B-09 Reihenfolge:** `create_cluster` (Z. 354) schreibt `ClusterRegister` in den Catalog, **bevor** `create_cluster_key` läuft (Z. 389).
- **B-10 Defaults:** `open` (Z. 200–224) nimmt `MemAnchor`, `SimClock`, `SimEntropy::new(1)`, wenn Config leer ist.
- **B-11:** `let _ = catalog.append_durable(...)` im Gate-Pfad von `erase_cluster` (Z. ≈ 930–942).
- **B-12 Mutex:** `EngineShared.state: Mutex<EngineState>` schützt Catalog-Appends, Ledger-I/O und Erase-Phasen. Kein Teil nutzt `sync::ranked`.
- **B-13:** Hartkodiert: Idem-TTL `86_400_000_000_000` (Z. 466), Guard-Fenster `65_536` (Z. 217), Governor `5_000` (Z. 213).
- `Storage::open` setzt `ledger_key` neu, `IdemRegistry::default()`, `GuardWindow::new(65_536)`: alles RAM, nicht aus Redo rekonstruiert.

## 2. Entfernen

| Ziel | Stelle | Grund |
|---|---|---|
| `contextra_storage_ports`-Alias und Importe | `lib.rs:3`, 189 Zeilen gesamt | Phantom-Crate; ersetzen durch `contextra_ports::kernel` |
| `PlainNoopCipher` aus Produktionspfad | `lib.rs:97–122`, `774`, `1155` | DEC-43, B-04. Nur hinter `cfg(any(test, feature = "testing"))` |
| `CoreMac` aus Produktionspfad | `lib.rs:126–141`, `775`, `886`, `1156` | DEC-43, B-05 |
| `MemAnchor`/`SimClock`/`SimEntropy` als Default | `lib.rs:211–224` | DEC-43, B-10 |
| `SealMode::Plain`-Pfad für Sealed | `lib.rs:745–790` | DEC-44 |
| Konstante `CompletionEvidence` | `lib.rs:≈ 980–1000` | DEC-49, B-03 |
| `[0u8; 32]` als Purge-Beleg | `lib.rs:≈ 975` | DEC-49, B-03 |
| `let _ =` auf `append_durable` | `lib.rs:≈ 930–942` | B-11 |
| `EngineState`-Felder ohne Persistenz (`restrictions`, `branches`-Mutationen) | nach Umbau: Redo-Replay ist Quelle | DEC-46 |
| Bisheriger Schreibpfad ohne Redo in `apply` | `lib.rs:447–657` | DEC-41 |

## 3. Ändern

### 3.1 Fassade `lib.rs` (1419 Zeilen) zerlegen

- `lib.rs` nur noch: Modulliste, Re-Exports, `Storage`, `StorageView` (Signaturen).
- Implementierung wandert in `facade/apply.rs`, `facade/view.rs`, `facade/erase.rs`, `facade/branch.rs`, `facade/checkpoint.rs`, `facade/space.rs`.
- Grund: Die Datei enthält Schreibpfad, Löschung, Checkpoint und Lesen gleichzeitig; das macht Review und Modelltests unmöglich.

### 3.2 `apply` (Z. 447–657)

Neuer Ablauf (DEC-41):

1. `Change` validieren (Limits, Guards). Unverändert.
2. Stufe-A-Guards beim Aufrufer, Stufe-B im RAM (DEC-14).
3. Frame bauen über `codec`, Schlüssel und AAD über `ports::kernel::Cipher`.
4. An `Committer::commit(frames, durability)` übergeben.
5. Sequencer vergibt Stamp unter Rang 1 (nur RAM).
6. Syncer schreibt Redo und ruft `sync_data`.
7. Erst nach `GroupAck` publizieren: `Version` mit `durable_stamp = stamp` nur bei `Durable`.
8. `Receipt { stamp, durable }` mit `durable = (ack.durable)`.

`Durability::Relaxed` ergibt `durable: false`, ist nie mit `IdemToken` zulässig (FIX-27).

### 3.3 `open` (Z. 200–300)

Neuer Ablauf (DEC-42, DEC-43):

1. `Config::validate()`. Fehlt `anchor`, `clock`, `entropy`, `fs`, `root` ⇒ `InvalidInput`. Kein Default außerhalb von `testing`.
2. `open_instance`: Catalog laden, Anker prüfen (Rollback, F-9).
3. Ledger öffnen, Kette verifizieren (DEC-45). Fehlschlag ⇒ `LedgerUnavailable`.
4. **Redo-Replay** ab `replay_floor` (neu verdrahten: `recovery::replay`).
5. Aus Catalog: Cluster-Tabelle, Spaces (mit `SpaceOptions`), Cluster→Space-Zuordnung (DEC-47).
6. Aus Redo: Restriction-Tabelle, Link-Graph, Branch-Tabellen, Gate-Index (DEC-46).
7. Ledger-Schlüssel aus RootKey ableiten (DEC-45); `ledger_key` nicht mehr zufällig.
8. Erst danach `Storage` zurückgeben.

### 3.4 `create_cluster` (Z. 354–395)

Reihenfolge (DEC-48): (1) Cluster-ID zufällig, (2) bei Sealed `create_cluster_keys` durabel, (3) Catalog-Edit `ClusterRegister`, (4) RAM-Tabellen. Fehler in (3) hinterlässt höchstens einen Waisenschlüssel.

### 3.5 Kontrollpfade (DEC-46)

`restrict_cluster`, `lift_restriction`, `link_clusters`, `branch_fork`, `branch_drop` werden Control-Frames:

- Frame durch Sequencer, im Redo, mit Stamp.
- Zugehöriges Ledger-Event (`RestrictionSet`, `RestrictionLift`, `Link`, `BranchFork`, `BranchDrop`).
- Erst nach Syncer sichtbar.
- Der Catalog bekommt dafür **keine** Op.

### 3.6 `create_space` (Z. 304–352)

`CatalogOp::SpaceAdd` um `name` und kanonisch kodierte `SpaceOptions` erweitern (DEC-47). **Vor G-F**, weil danach nur per Migration.

### 3.7 `erase_cluster` (Z. 849–1000)

Neuer Ablauf (DEC-49, R14 Anhang A):

1. Synchron: Intent und Gate (Frame im Redo, Ledger-Event `EraseIntent`, `GateApplied`). Rückgabe: `Ticket`.
2. Auf Wartungs-Lanes (`urgent > flush > compact > blobgc > idle`):
   - `KeyDestroy` über `KeyProvider::destroy_keys`, Ledger-Event `KeysDestroyed`.
   - `Purge` über `maint::purge::PurgeRunner`.
   - `Verify` über `Eraser::verify`; befüllt `CompletionEvidence`.
3. `finalize` nur mit gemessener Evidenz. Konstanten verboten (INV-FAC-4).
4. Fehler eines durablen Schritts: Ticket vergiftet oder `Blocked`, nie `let _ =`.

`erase_cluster` gibt bei Logical-Level sofort einen Teilbeleg zurück; `Complete` erst nach Verify.

### 3.8 Rang-1-Disziplin (DEC-50)

- `EngineState`-Mutex nur für RAM.
- Catalog-Appends, Ledger-I/O, Redo-Schreiben über Syncer bzw. eigene Threads.
- `sync::ranked` verwenden; Rang-Verletzung ist Test-Fehler.

### 3.9 Konfiguration (DEC-51)

Hartkodierte Werte aus B-13 in `Config`. In `config.rs` mit Standardwert aus §15. Validierung in `Config::validate`.

## 4. Implementieren

1. **Limits** aus §15 zentral (`limits.rs`): `MAX_GROUP_BYTES`, `REDO_SEGMENT_BYTES` (heute importiert, fehlen).
2. **Replay-Verdrahtung** in `recovery/open.rs`: Iterator über Redo-Segmente ab `replay_floor`, Anwendung über `CommitPublisher`.
3. **Schreibpfad** mit `Committer` und `RedoWriter` (existiert in `engine/committer.rs`, `store/redo/writer.rs`).
4. **Catalog-Erweiterung** `SpaceAdd` mit Optionen; Rekonstruktion beim Öffnen.
5. **Ledger-Verifikation** beim Öffnen, `LedgerWriter::recover` mit abgeleitetem Schlüssel.
6. **Erase-Automat** nach Anhang A (`Ticket`-Zustände).
7. **Purge** in `maint::purge`: Plan (`plan_purge`), Umschreiben (`rewrite_without`), Löschen (`unlink_dead_files`) — verdrahten.
8. **Anker-Fortschreibung** vor KeyDestroy und Purge (blockierend, §8.9).
9. **Checkpoint** über `contextra-checkpoint`-Traits (`SnapshotParticipant`).
10. **Tests:**
    - `crash_sweep_apply`: Prozessende an jeder `Fs`-Operation. Nach Neustart: Wert vorhanden **oder** nie quittiert (INV-FAC-1, INV-FAC-2).
    - Restart-Tests für Restriction, Link, Branch, SpaceOptions (DEC-46, DEC-47).
    - Mutationstest: Evidenz `false` ⇒ `Incomplete`.
    - Sealed-Flush in `Plain` ⇒ `Err` (DEC-44).
    - `loom`-Test: Gate × Query (`INV-RAG-4`).

## 5. Tests (vollständig)

| Test | Prüft | Gate |
|---|---|---|
| `crash_sweep_apply` | INV-FAC-1, INV-FAC-2, Z-2 | G-FAC |
| `restart_controls` | DEC-46, FIX-42 | G-FAC |
| `restart_space_options` | DEC-47, FIX-43 | G-FAC |
| `erase_evidence_mutation` | INV-FAC-4, DEC-49 | G-FAC |
| `sealed_plain_rejected` | DEC-44 | G-FAC |
| `ledger_key_derivation` | DEC-45, FIX-09 | G-FAC |
| `create_cluster_order` | DEC-48 | G-FAC |
| `no_io_under_rank1` | INV-FAC-5 | G-FAC |
| `o_files_after_done` | `O-Files` nach Done leer | G-E |
| Bestehende 110 Tests | Regression | laufend |

## 6. Abhängigkeiten nach Umbau

Erlaubt: `bytes`, `thiserror`, `zeroize`, `crossbeam-skiplist` (nur hinter Trait `MemIndex`), `siphasher` (nur Guard-Fingerabdrücke), `contextra-ports` (Kernel), `contextra-types`, `contextra-crypto` **nur über Trait-Objekte**, nicht direkt.
Verboten: `tokio`, `futures`, `rayon`, `parking_lot`, `scc`, `dashmap`, `memmap2`, `serde*`, `bincode`, `blake3`, `sha2`, `rand`, `crc32fast`, `anyhow`, `tracing`.

**Hinweis:** `crossbeam-skiplist` ist in der Verbotsliste nicht genannt, aber in §2.7 erlaubt. Kein Konflikt.

## 7. Gate und Reihenfolge

1. **P0-A:** Phantom-Import durch `contextra_ports::kernel` ersetzen; `cargo check -p contextra-storage --locked`. Ohne diesen Schritt ist kein anderer Befund prüfbar.
2. **P0-C (Fassadenwahrheit):**
   a. Replay in `open` (DEC-42).
   b. Redo in `apply` (DEC-41).
   c. Sealed-Adapter (DEC-44, benötigt `contextra-crypto`-Adapter).
   d. Ledger-Schlüssel (DEC-45).
   e. Kontrollpfade (DEC-46), Space-Optionen (DEC-47), Schlüssel-zuerst (DEC-48).
   f. Erase-Automat (DEC-49).
   g. Rang 1 (DEC-50), Config (DEC-51).
3. **G-FAC:** alle Tests aus §5 grün.
4. **P2/P5/P6/P7** nach §13.1.

**Reihenfolge innerhalb P0-C ist nicht beliebig:** (a) vor (b), weil Replay den Zustand beschreibt, den `apply` schreibt; (c) vor (d) und (f), weil Ledger und Erase Sealed-Cluster voraussetzen.

**Risiken:**
- Redo-Format ist bis G-F nicht eingefroren. Änderungen in P0-C sind noch erlaubt, danach Migration.
- `lib.rs`-Zerlegung ist groß. Vorher Verhalten mit Snapshot-Tests festhalten, damit Umstrukturierung nachweisbar folgenlos bleibt.
- Erase-Automat auf Wartungs-Lanes erhöht die Zahl der Zustände. Modelltest (TLA+, §12) vor Implementierung.
