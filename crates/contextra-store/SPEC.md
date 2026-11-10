# contextra-store — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-store/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

LSM-Speicher: WAL, Memtable, SSTable, Manifest, Recovery. Strikt durabilitätskritisch (INV-6). *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | storage |
| Ebene (Kern-Closure) | 1* |
| Kern-Closure | ids, env, sys, mvcc, crypto |
| Default-Features | — |
| Fehlertyp (R3) | `StoreError` (neu) |
| Brücke (hinter Feature `workspace`, §1.1) | `engine/`, `scanner/`, `From` |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 18015 Zeilen in 83 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | types, ports, sys, mvcc, crypto |
| Interne Dev-Abhängigkeiten (Ist) | testkit |
| Features (Ist, Anzahl) | 8 |
| Default-Features (Ist) | keine |
| `cfg(feature = …)`-Stellen in src/ | 63 |
| `#[test]`-Vorkommen (grep, src + tests) | 110 |
| zero-panic: Produktionsbefunde, bereinigt (Tool-Gesamt; davon in Testdateien) | 1 (Tool 50; 49 in Testdateien) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 0 / 0 / False |
| Determinismus: Produktionsbefunde, bereinigt (Tool-Gesamt) | 14 (Tool 17) |
| Port-M3-Leaks (port_contract_audit) | 0 |
| Error-Enums (Ist) | WalParseError, DurabilityConfigError |

**Tool-Caveat:** `zero_panic_analyzer.py` klassifiziert `#[cfg(test)]`-Module (`tests.rs`) als Produktion. Im Repo betrifft das 449 von 478 Treffern. Die Zahlen oben sind deshalb bereinigt; die Rohzahl steht in Klammern. Die Korrektur am Tool ist eigener Schritt (S-00).

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/cargo_dependency_graph.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel nicht vorsieht:** `types`, `ports`
- **Ziel-Kanten, die im Ist fehlen:** `ids`, `env`
- **Default-Features:** Ist = Ziel (leer).

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Verifiziert: `FeatureRing` und `validate_against_features` in `src/lsm/config.rs:73–92` (SPEC nennt 68–92, Zeilenversatz). `kv/` existiert, ist aber nicht in `lib.rs` eingebunden (bestätigt). `io_seam` ist in `lib.rs:99` deklariert.

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

(1) Eigener `StoreError`; Abbildung der heute genutzten Varianten (`Storage`, `Io`, `WalCorruption`, `ChecksumMismatch`, `Transaction*`, `Conflict`, `WalTruncationDetected`, `CrossDeviceLink`) nach §5.3. (2) `compat/`: `engine/` (370), `scanner/` (1.400), `From<StoreError>`; `StorageEngine`-Impl nur dort. (3) `lsm/config.rs:68–92` (`validate_against_features`, `FeatureRing`) entfernen — **erst nach** S-50 (Lizenz-Ports); `validate_durability` bleibt. (4) `kv/` (56 Z., **nicht** in `lib.rs` eingebunden [V]) nach `archive/`. (5) `tenant_codec` → `ids::TenantId`; `Clock`/`MetricsSink`/`IdGen` → `env`. (6) `io_seam`/`fsync` unverändert (B08), `Vfs`-Bindung per Brücke. (7) ID-Breite im Manifest (§5.2) **zuletzt**. Kern-Closure: ids, env, sys, mvcc, crypto. INV-6 gilt strikt: WAL-, Flush- und Recovery-Code wird nur an Typ-/Importstellen berührt.

### 6.2 Brückenmuster [S]

Die Brücke liegt ausschließlich in `src/compat/` hinter dem Feature `workspace`. Verbindliche Muster: docs/SPEC.md §4.3 (a–d). R2 gilt: `contextra_types`, `contextra_ports`, `contextra_testkit` dürfen im Kern nirgends vorkommen, auch nicht in Doc-Kommentaren oder `#[cfg(test)]`.

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

### 6.4 ID-Breite [S]

Persistierte Formate mit IDs tragen `id_width_bits: u16` (64|128). Öffnen mit abweichender Breite ⇒ `…::IdWidthMismatch { stored, build }`, kein Panic, keine Teilmutation (docs/SPEC.md §5.2, B03).

### 6.5 Fehlerabbildung [S]

Eigener Fehlertyp: `StoreError` (neu) (thiserror, `#[non_exhaustive]`). Jede heute genutzte `ContextraError`-Variante hat genau eine lokale Entsprechung; `Display` bleibt bitgleich; keine Sammelvariante `Other(String)` ohne Quelle. Tabelle je Baustein: `docs/decoupling/errors/<crate>.md` (aus S-xx erzeugt). Quelle: §5.3.

## 7. Schnittstellen und Persistenz

### 7.1 Persistierte Formate (§5.1 Golden-Korpus) [S]

| Format | Quelle / Kennung |
|---|---|
| Manifest | `store/manifest/core.rs`: `MFMN`, Version 1 |
| SSTable | `store/sstable/block_cache.rs`: `MFSX`, Legacy `MFST` (Footer, Blöcke, Bloom) |
| WAL | `store/wal/*` inkl. HMAC-Kette (`wal_crypto`) |
| Rollback-Intent | `store/lsm/recovery.rs`: `MFRLBK\0\0` |

Jeder Eintrag hat einen Erzeugertest (`#[ignore]`, Script `tools/golden_regen.sh`) und einen Prüftest, der immer läuft. Abweichung ⇒ Schritt abbrechen.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

**zero-panic (Produktion, bereinigt: 1; die ersten 15):** Tool-Gesamt 50, davon 49 in Testdateien (`tests.rs` unter `#[cfg(test)]`, Tool-Fehlklassifikation).

- `crates/contextra-store/src/memtable.rs:49` — assert — `const _: () = assert!(TOMBSTONE_BIT == 1u64 << 63);`

**Determinismus (Produktion, bereinigt: 14; die ersten 15):** Tool-Gesamt 17.

- `crates/contextra-store/src/manifest/core.rs:302` — random_hashmap — `let mut valid_map = std::collections::HashMap::new();`
- `crates/contextra-store/src/manifest/core.rs:356` — random_hashset — `) -> std::collections::HashSet<PathBuf> {`
- `crates/contextra-store/src/manifest/core.rs:358` — random_hashset — `let valid_set: std::collections::HashSet<PathBuf> =`
- `crates/contextra-store/src/manifest/core.rs:360` — random_hashset — `let mut dead_set = std::collections::HashSet::new();`
- `crates/contextra-store/src/wal/flusher.rs:184` — instant_now — `let deadline = tokio::time::Instant::now()`
- `crates/contextra-store/src/lsm/engine.rs:18` — random_hashmap — `use std::collections::HashMap;`
- `crates/contextra-store/src/lsm/engine.rs:69` — random_hashmap — `pub(super) intent_locks: std::sync::Mutex<HashMap<Vec<u8>, TxId>>,`
- `crates/contextra-store/src/lsm/recovery.rs:258` — random_hashset — `Option<std::collections::HashSet<std::path::PathBuf>>,`
- `crates/contextra-store/src/lsm/recovery.rs:259` — random_hashset — `Option<std::collections::HashSet<std::path::PathBuf>>,`
- `crates/contextra-store/src/lsm/recovery.rs:278` — random_hashmap — `let mut pending_tx_map: std::collections::HashMap<u64, Vec<PendingTxOp>> =`
- `crates/contextra-store/src/lsm/recovery.rs:279` — random_hashmap — `std::collections::HashMap::new();`
- `crates/contextra-store/src/lsm/recovery.rs:282` — random_hashset — `let mut replayed_hmac_set = std::collections::HashSet::new();`
- `crates/contextra-store/src/lsm/recovery.rs:629` — random_hashmap — `intent_locks: std::sync::Mutex::new(std::collections::HashMap::new()),`
- `crates/contextra-store/src/lsm/recovery.rs:751` — random_hashset — `let txs_to_rollback: std::collections::HashSet<u64> = memtable`

**Fehler-Enums (Ist):**

- `crates/contextra-store/src/wal/encode.rs:18` — WalParseError
- `crates/contextra-store/src/lsm/config.rs:25` — DurabilityConfigError

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B01 — Drei Dokumente, drei Architekturen**
> **Entscheidung:** - Es gibt kein `modules/`-Verzeichnis. Die Brücke ist ein **Modul** `compat/` im Crate (kein Extra-Crate). - `ids`/`env` sind für alle Bausteine erlaubte E0-Abhängigkeiten. Die Seam-Definitionen `Clock`, `Rng`, `MetricsSink`, `IdGen` der Schwester-Specs entfallen zugunsten `env`; `DocKey`/`ExternalId`-Typen zugunsten `ids`. `docid-128` gehört **ausschließlich** `ids` (B03). - Die Mikro-Crate-Zerlegung der Schwester-Specs ist **Folgeprojekt F-1** und nicht Teil dieser Spezifikation (§9.3).

**B03 — `docid-128` ändert die Typbreite und ist nicht additiv**
> **Entscheidung:** 1. `ids` besitzt `docid-128` allein. Jeder Crate mit Abhängigkeit auf `ids` leitet `docid-128 = ["contextra-ids/docid-128"]` weiter; leere/lokale Features entfallen. E0-Crates ohne `ids` (`ingest`, `erasure`) bekommen **kein** `docid-128`; ihre Konvertierung von `ids::DocId` ist `TryFrom` (fehlschlagend bei Überlauf), nie `as`. 2. `ids` führt `pub const ID_WIDTH_BITS: u32` (64 oder 128) und die Compile-Time-Prüfung `size_of`/`align_of` (8/8 bzw. 16/16). 3. Jedes persistierte Format, das IDs trägt, MUSS die Breite beim Öffnen prüfen (§5.2). Fehlt das Feld heute, wird die Formatversion erhöht und v(alt) als „Breite unbekannt“ nur mit expliziter Konfiguration geöffnet. 4. CI-Matrix: `cargo hack check --workspace --features docid-128` zusätzlich zum Default.

**B04 — Entkopplungsstand von `text` ist falsch eingeschätzt; Zählungen weichen ab**
> **Entscheidung:** Zahlen sind Orientierung. Abnahme erfolgt ausschließlich über die Null-Prüfung G-L2 (§8.2) und den Build ohne Features, nie über Zählwerte. Aufwandsklasse `text`: mittel.

**B07 — Eine Verzeichnis-Regruppierung bricht messbar viel**
> **Entscheidung (D-01):** Das Verzeichnislayout bleibt **flach** (`crates/contextra-*`). Die Gruppierung steht als Metadatum `group` in `capabilities.toml`. Die physische Regruppierung ist Nicht-Ziel (§9.3); sie liefert keinen Entnahmegewinn und kostet die oben genannten Brüche.

**B08 — `Vfs` kann nicht verschoben werden, ohne den Fehlertyp zu ändern**
> **Entscheidung:** `Vfs`, `VfsFile`, `VfsOpenOptions`, `VerifiedVia` bleiben in `ports`. `durable-fs` bleibt eigenständig (E0, Tier experimental, kein Verwender). Die Plan-Schritte „`io_seam`/`fsync.rs` nach `durable-fs`“ und „`sys-flush`“ entfallen (Nicht-Ziel). Verbraucher mit Dateisystembedarf (`store`, `checkpoint`) halten eine **crate-lokale** schmale Naht (siehe §6), die Brücke bindet an `ports::Vfs`.

**B11 — Nebenläufigkeits-Abdeckung fehlt genau dort, wo umgebaut wird**
> **Entscheidung:** S-01 nimmt `mvcc`, `kvcache`, `checkpoint`, `ids`, `env` in die Pfadfilter von `tsan.yml`/`loom-gate.yml`, `mvcc`+`kvcache` in die Crate-Liste der Mutation-Tests und `rust-ci.yml` auf. Enthält `crates/contextra-mvcc/tests` kein Loom-Modell (`rg -l loom crates/contextra-mvcc` leer), ist vor S-30 ein Modell für `SequenceLog` (Publish/Visibility, zwei Schreiber, ein Leser) zu schreiben; sonst ist S-30 nicht abnehmbar.

**B13 — Dev-Kanten auf `testkit` und Entnahmenachweis**
> **Entscheidung:** Dev-Kanten zu `testkit` sind erlaubt, stehen aber im Manifest `dev_allowed` (§8.3). Der Isolationsnachweis (G-L4) prüft `--edges normal` und baut zusätzlich `--no-dev-deps`; Tests laufen im Export nur mit `--with-dev` (nimmt dann `testkit`-Closure mit). `env` (Feature `test-utils`) übernimmt `ManualClock` und die Clock/Rng-Konformitätssuite, damit E1-Crates dafür nicht auf `testkit` angewiesen sind.

**B16 — Einzelentscheidungen, die der Entwurf offen ließ**

## 10. Entscheidungen und Risiken [S]

**Risiken (§9.2):**

| Format-/Recovery-Drift im `store` | S-37 | Crash-/Recovery-Suite oder GF rot ⇒ Schritt verwerfen, nicht reparieren |

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra-store --no-default-features --locked
cargo hack check -p contextra-store --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-store --modes no-default,default,each-feature
```

