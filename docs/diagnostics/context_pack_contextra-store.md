# Agentic Context Pack: `contextra-store`

> **Werkzeug:** `agentic_context_pack.py`
> **Erstellungszeit:** `2026-11-07T23:15:52.934647+00:00`
> **Status/Befunde:** `1 Befunde erfasst`

## Beschreibung
Curated, minified context pack for coding agents assigned to `contextra-store`. Optimized for LLM window density and context engineering.

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `crate_name` | `contextra-store` |
| `ring` | `Unknown` |
| `class` | `Standard` |

## 1. Architektur-Zuweisung
- **Crate:** `contextra-store`
- **Ring:** `None`
- **Rolle/Klasse:** `None`
- **Pfad:** `crates/contextra-store`

## 2. Crate-Spezifische AGENTS.md Verträge
# AGENTS.md — `contextra-store`

> **Ring 1** · Klasse `[K]` · Reifegrad `beta` · Unsafe: nein · Rolle: LSM-Storage: WAL, Memtable, SSTable, Manifest, Kompaktion, Block-Cache. Implementiert `StorageEngine`.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-store --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `crypto`, `mvcc`, `ports`, `sys`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types`, `mvcc`, `crypto`, `sys`, `durable-fs` |
| Features | `block-cache-v2`, `sieve-cache`, `docid-128`, `fault-injection`, `memory-only-storage`, `legacy-wal-key`, `test-hooks` |
| Lints | `#![forbid(unsafe_code)]`, `deny(dead_code)` |

## 2. Abhängigkeiten & Schulden

- **Dev-Kanten auf höhere/gleiche Schichten** (M6): `graph`, `sandbox`, `vector` (+ Selbstreferenz). Nach `testkit`/`tests/` verlagern.
- **Kein Dep auf `durable-fs`**; direkte Datei-I/O (~77 `fs`-Treffer, ~176 `tokio::`-Treffer): Umstellung auf `Vfs` (D4, WP 4.4).
- **Löschreste:** `kv/segment.rs`, `kv/delete_mode.rs` (Shredding-Bezug, 28 Treffer), `compaction/engine.rs`, `compaction/adaptive.rs` (WP 3.2).
- Implementierte Ports sind vorhanden: `scanner/` (`LsmPurger`, `MemtableScanner`, `SstableScanner`, `WalScanner`).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `wal/` (`encode`, `flusher`, `hmac`, `io`, `replay`) | Append-only WAL, HMAC-Kette, Flusher-Actor, Replay |
| `lsm/` (`engine`, `commit`, `flush`, `group_commit`, `guard`, `recovery`, `scan`, `ops/*`, `validate`) | Orchestrierung, Commit, Recovery |
| `memtable.rs` | Memtable mit Sequenznummern/Tombstones |
| `sstable/` (`builder`, `reader`, `block_cache`, `bloom`, `stream`, …) | Sortierte Segmente, Block-Cache, Bloom |
| `manifest/` | transaktionales Manifest |
| `compaction/` (`engine`, `adaptive`, `config`, `merge_operator`, `retention`) | Level-Merge, `MergeOperator` |
| `scanner/` (`memtable`, `sstable`, `wal`, `purge`) | **Port-Implementierungen** `LayerScanner`/`Purger` |
| `kv/` | KV-Segmente, Löschmodus (Schuld) |
| `fsync.rs`, `util.rs`, `system_pressure.rs`, `tenant_codec.rs`, `engine/` | Sync-Stelle, Hilfen, Druck, Tenant-Präfix, In-Memory-Engine |

## 4. Verträge & Invarianten

1. **`fsync`-Fehler propagieren** (`?`), nie `let _ =`; nach Fehler Handle vergiften (INV-FSYNC-1). `fsync.rs` ist die einzige Stelle für Datei-/Verzeichnis-Sync.
2. **Atomares Schreiben:** tmp-Datei → `fsync` → `rename` → Parent-`fsync`.
3. **Tombstone-Disziplin:** Bit 63 maskieren (`seq & !TOMBSTONE_BIT`) vor jedem `max_seq`-Vergleich (verifiziert u. a. in `compaction/adaptive.rs`).
4. **Single-Load-Regel:** `last_committed_tx` in `get_at_seq`/`scan_prefix_at` **einmal** laden (Snapshot-Isolation). **Flush-before-Visible:** `last_committed_tx` vor dem Sichtbarmachen der SSTable aktualisieren (ADR-043, Altdokument: am Code in `lsm/flush.rs` prüfen).
5. **WAL-HMAC-Schlüssel** nur über `load_or_create_integrity_key` (`wal/hmac.rs`, `pub(crate)`), nie hartcodiert; `legacy-wal-key` nur für Migration.
6. **Tenant-Präfix in jedem Schlüssel** (INV-TENANT-1); Schlüssel/Dateinamen nur opake IDs (INV-KEYSPACE-1).
7. **Neu verboten:** DSGVO-/Proof-/Saga-/Shredding-Logik (M7). Purge-Kompaktion nur über `Purger` (`compact_range(scope)`).
8. Jede Datei: Magic + Formatversion (INV-FORMAT-1); Prüfsummen erkennen Bit-Rot (kein Korrekturversprechen).

## 5. Ports

**Implementiert:** `StorageEngine`, `Purger`, `LayerScanner`, `BlockCache`, `MergeOperator`. **Konsumiert:** `Clock`, `Vfs` (Soll), `MetricsSink`.

## 6. Nebenläufigkeit & Performance

- **Sperrenordnung** (Altdokument, vor Änderung verifizieren): `write_lock` → `memtable` → `sstables` → `snapshot_registry`. Nie umgekehrt, nie `parking_lot`-Guard über `.await` (`clippy::await_holding_lock`).
- Schreibpfad über Group-Commit und Flusher-Actor (Message-Passing statt geteiltem Zustand). `loom`: `just loom-store`.
- Lesepfad Ziel: allokationsfrei (`with_value`, `PinnedBlock`, WP 4.1). Blockierende Datei-I/O in `async` nur via `spawn_blocking`.
- Komplexität: Punktlesen `O(log n)`-Ebenen + Bloom; Compaction-Schreibverstärkung messen (`tests/amplification_benchmark.rs`).

## 7. Anti-Patterns

- `let _ = file.sync_all()`.
- `bincode::deserialize(..).unwrap_or_default()` auf WAL-Daten.
- `std::fs` direkt statt `Vfs`.
- Neue Abhängigkeit auf `vector`/`text`/`graph` (Peer-Import verboten).
- Löschlogik/Proof-Typen im Store.

## 8. Tests & Messung

```bash
cargo test -p contextra-store --locked
cargo clippy -p contextra-store --all-targets -- -D warnings
just loom-store
cargo bench -p contextra-store            # wal, memtable, sstable, block_cache, lsm
cargo mutants --config mutants-guarantees.toml -p contextra-store -f "crates/contextra-store/src/wal/**"
```

- Vorhanden: Chaos (`chaos_*`), Campaign-/Property-Tests, Fuzz (`fuzz_wal_replay`, `wal_roundtrip`, `fuzz_manifest_load`, `wal_mutation_chaos`), Benchmarks inkl. Vergleich `redb`.
- Ratchet: `tokio::`/`fs`-Treffer dürfen nicht steigen.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 3.2 | Löschketten-/Shredding-Relikte entfernen, nur `Purger`/`LayerScanner` behalten |
| 4.1 | Compaction als Level-Merge, `PinnedBlock`, `with_value` |
| 4.4 / D4 | `Vfs` statt `fs`/`tokio::fs`; Dep auf `durable-fs` |
| M6 | Dev-Kanten `graph`, `sandbox`, `vector` entfernen |


---


# Kommentar-Syntax (verbindlich)

| Form | Zweck |
|---|---|
| `//!` | Modul-/Crate-Doku am Dateianfang (vor jedem Item, nach `#![...]`-Attributen der Datei; nie mitten in der Datei) |
| `///` | Rustdoc für das **direkt folgende** Item (Struct, Enum, Variante, Feld, Trait, Methode, Funktion, Konstante). Nie vor `let`, Ausdrücken, Match-Armen oder Makroaufrufen; dort `//` verwenden. |
| `// NOTE:` | Warum etwas so gebaut ist (nicht was) |
| `// TODO:` / `// FIXME:` / `// STUB:` | Offene Arbeit / bekanntes Risiko / Platzhalter; immer mit konkreter Aussage |
| `// SAFETY:` | Pflicht direkt über jedem unsafe-Konstrukt (Invariante, Herstellungsort, Folge bei Verletzung) |
| `// ANCHOR[contextra-store:<BEREICH>-<NNN>]` | Sprungmarke; vor dem Item/Abschnitt, auf den sie zeigt |
| `// FILE-CONTEXT` + `// ZWECK:` / `// INVARIANTEN:` / `// HOTSPOTS: <von>-<bis>` / `// STAND: <Datum>` | Repo-Konvention für den Dateikopf. Vorhandene Blöcke ergänzen; `HOTSPOTS`-Zeilenbereiche nach dem Kommentieren auf die neuen Zeilennummern aktualisieren. `STAND` auf das heutige Datum setzen, wenn du den Block änderst. |

Weitere Tags führst du nicht ein. Beim Kommentieren markierter Altlasten gilt: `// NOTE: Deprecated Debt DEBT-NN – <Beschreibung>`.

## ANCHOR-Ids (Kollisionsschutz)

Im Repo existieren ca. 3.900 Anchors; Ids sind workspace-weit eindeutig zu halten. Format: `ANCHOR[contextra-store:<BEREICH>-<NNN>]` mit `<BEREICH>` in Großbuchstaben/Ziffern/Bindestrichen, `<NNN>` dreistellig. Vorgehen:

1. Vor jeder neuen Id: `grep -rnF "ANCHOR[contextra-store:<BEREICH>-<NNN>]" crates benchmarks` muss leer sein.
2. Nummeriere ab dem Maximum des Bereichs weiter. Höchste bestehende Nummern in dieser Crate:

| Bereich | Höchste Nr. | Nächste freie Id |
|---|---:|---|
| `WAL-CORE` | 2 | `ANCHOR[contextra-store:WAL-CORE-003]` |
| `COMP-ENG` | 1 | `ANCHOR[contextra-store:COMP-ENG-002]` |
| `KV-SEG` | 1 | `ANCHOR[contextra-store:KV-SEG-002]` |
| `LSM-ENG` | 1 | `ANCHOR[contextra-store:LSM-ENG-002]` |
| `MAN-CORE` | 1 | `ANCHOR[contextra-store:MAN-CORE-002]` |
| `SST-BLD` | 1 | `ANCHOR[contextra-store:SST-BLD-002]` |
| `SST-RDR` | 1 | `ANCHOR[contextra-store:SST-RDR-002]` |
| `ROOT-UTIL` | 1 | `ANCHOR[contextra-store:ROOT-UTIL-002]` |

3. Neue Bereiche nach dem Modul benennen (z. B. `WAL`, `PUT`, `GATE`); fange bei `001` an, nach Prüfung 1. Setze Anchors sparsam: pro Modul, pro öffentlichem Einstiegspunkt und pro nicht-trivialem Ablauf, nicht pro Zeile.
4. Bereits vorhandene doppelte Ids anderer Stellen nicht „reparieren“; nur im Report melden.

Für Abläufe in Funktionen nutze nummerierte Schritte:

```rust
// ANCHOR[contextra-store:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge, Schlüsselformat).
// 2. Snapshot der Transaktion nehmen.
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen; ein Fehler wird zurückgegeben (Fail-Closed).
// 4. Erst danach im Speicher übernehmen.
```

# Was zu dokumentieren ist

Gehe **jede** Rust-Datei unter `crates/contextra-store` durch (`find crates/contextra-store -name "*.rs"`, ohne die generierten Dateien oben) einschließlich `tests/`, `benches/`, `examples/` und `build.rs`:

- **Modul** (`//!`): Zweck, Ring-Zugehörigkeit, Nutzer, wichtigste Typen, Invarianten, Verweis auf Nachbarmodule.
- **Öffentliche Items:** Zweck in einem Satz, dann bei Bedarf `# Errors` (welcher Fehler wann), `# Panics` nur wenn der Code abbrechen kann (sollte nach Zero-Panic-Regel nicht vorkommen: dann `FIXME`), `# Safety` bei `unsafe fn`, Vorbedingungen, Nachbedingungen, Nebenwirkungen (I/O, Zeit, Zufall, IDs). Zeit, Zufall und IDs laufen nur über `Clock`/`Rng`/`IdGen`; steht es im Code, steht es im Kommentar.
- **Nicht-triviale private Funktionen:** Warum sie existieren und was sie garantieren.
- **Nebenläufigkeit:** Sperren, atomare Operationen, Reihenfolge, Warum kein Deadlock.
- **Persistenz und Fehler:** Wo Daten WAL oder Dateisystem-Port passieren; `fsync`-Fehler werden propagiert.
- **Tests:** Ein Satz je Testfunktion oder -block: welche Invariante oder welches Verhalten bewiesen wird.
- **Architektur:** Welche Ports die Crate nutzt/anbietet, welche Abhängigkeitsrichtung gilt (DAG, `may_depend_on`), was bewusst *nicht* hier liegt.

## Rustdoc-Fallen (jeder Punkt bricht sonst `cargo clippy -D warnings` oder `cargo doc`)

- Kein `///` vor Nicht-Items (`unused_doc_comments`); keine Leerzeile zwischen `///` und Item (`empty_line_after_doc_comments`).
- Fortsetzungszeilen in Listen und Zitaten korrekt einrücken (`doc_lazy_continuation`). **Vier Leerzeichen Einrückung in Doc-Text erzeugen einen Doctest**; Code immer in Fences mit Sprache. Beispiele, die nicht kompilieren oder laufen sollen: `ignore` oder `text`, begründet.
- Intra-Doc-Links `[`Typ`]` nur auf existierende, erreichbare Items; spitze Klammern im Text (`Vec<u8>`) in Backticks setzen, sonst „unclosed HTML tag“.
- `// SAFETY:` ohne Leerzeile und ohne dazwischenliegende Zeilen direkt über dem unsafe-Ausdruck.
- Doc-Beispiele müssen gegen die tatsächliche Signatur kompilieren; im Zweifel `no_run`/`ignore` mit Grund.

## 3. Bekannte Technische Schulden (DEPRECATED_DEBT.md)
- - **Betroffene Crates:** `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-crypto`, `contextra-db`, `contextra-mcp`, `contextra-engine`.
- - **Betroffene Crates:** `contextra-engine`, `contextra-db`, `contextra-mcp`, `contextra-store`.
- | **DEBT-01** | Phase 3 & 4 | `contextra-erasure` | 0 Löschketten-/Nachweisimporte in `store`, `vector`, `text`, `crypto`. |

## 4. Öffentliche Schnittstellen & Signaturen (lib.rs)
```rust
pub mod compaction;
pub mod engine;
pub mod fsync;
pub mod kv;
pub mod lsm;
pub mod manifest;
pub mod memtable;
pub mod scanner;
pub mod sstable;
pub mod system_pressure;
pub mod tenant_codec;
pub(crate) mod util;
pub mod wal;
pub use compaction::{
pub use lsm::{
pub use manifest::{Manifest, ManifestEntry};
pub use system_pressure::{PressureLevel, SystemPressure, SystemPressureMonitor};
pub use tenant_codec::{TenantKeyCodec, TenantScopedStorage};
```

## Detaillierte Befunde
| Crate | Datei | Zeile | Art / Beschreibung |
|---|---|---|---|
| `contextra-store` | `crates/contextra-store/Cargo.toml` | `1` | Generated focused context pack for contextra-store |

---
*Automatisch generiert durch `agentic_context_pack.py` für Agentic Engineering & Context Auditing.*