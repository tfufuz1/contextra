# Contextra – Implementierungsspezifikation IST → SOLL

Stand 26.10.2026 · Basis-Commit `a547a42` (Merge PR #5302) · Ziel: Gesamtspezifikation 2.2/2.3 (Zielzustand v1.0) · Rolle: Principal Rust Engineer · **Fassung 2: Phasen 0–2 am Klon geprüft (Abschn. 0a), Phasen 3–9 ausgearbeitet (Abschn. 8–19).**

> **Zweck.** Dieses Dokument plant die Umsetzung vom heutigen, überladenen Repository-Stand zum Zielzustand der Gesamtspezifikation (im Folgenden **Spec**). Es ist eine Arbeitsunterlage im Sinne von Spec §18.7: Es führt Status gegen einen benannten Commit, ändert aber keine Soll-Aussagen. Die Löschkette wird vollständig neu gestaltet.

---

## 0. Belegbasis und Grenzen

| Quelle | Stufe |
| --- | --- |
| Spec 2.2/2.3 (2.621 Zeilen), für Kap. 4–7, 12, 13, 17 gezielt gelesen | gelesen |
| Scan-Pack und Gate-Bericht (Commit `a547a42`, 1.426 Rust-Dateien im Index) | Heuristik, liefert Prüfaufträge |
| Repository `tfufuz1/contextra`, HEAD `a547a42` = Scan-Commit; Klon vollständig (4.591 Commits, 88 seit `0304f9a`) | gelesen |
| Manifeste aller Crates per Skript ausgewertet (normal/dev, optional) | gemessen am Manifest |
| Quellcode-Stichproben: `engine/src/lib.rs`, `engine/src/contextra_impl/collections.rs`, `mcp/src/tools_crud.rs`, `types/src/witness.rs`, `crypto/src/{deletion_proof,crypto,kdf,wal_crypto}.rs`, `store/src/{fsync.rs,lsm/recovery.rs,lsm/engine.rs}`, `vector/src/hnsw/arena.rs`, `capabilities.toml` | gelesen |

**Nicht geprüft (ausdrücklich):** Kompilierung, Testsuite, Clippy, Miri, CI-Stand. Im Arbeitsumfeld steht keine Rust-Toolchain bereit (`cargo` fehlt, `static.rust-lang.org` ist gesperrt). **Jede Zeilenangabe gilt für `a547a42` und stammt aus Stichproben oder aus dem Scan.** Wo etwas nur aus dem Scan stammt, steht „Scan“. Wo ich etwas nicht gesehen habe, steht „ungeprüft“. Das erste Arbeitspaket (IMP-0.1) beschafft die fehlende Build-Baseline; alle Aufwandsaussagen gelten unter dem Vorbehalt, dass der Baum kompiliert.

---

## 0a. Korrekturen aus der Prüfung am Klon

Die Fassung 1 wurde am Klon `a547a42` gegengeprüft. Die folgenden Punkte ändern Maßnahmen oder Zahlen. Jede Korrektur ist mit dem Prüfbefehl belegt, damit sie in WP 0.2 reproduzierbar bleibt.

| ID | Fassung 1 | Befund am Klon | Konsequenz |
| --- | --- | --- | --- |
| K-01 | „Historie auf 293 Commits nachgeladen“; „rund 290 Commits seitdem“ | `git rev-list --count HEAD` = 4.591; `git rev-list --count 0304f9a..HEAD` = 88 | Diff-Umfang seit dem Rollback ist klein (88 Commits). Die Zahl 293 wird nirgends mehr verwendet. |
| K-02 | `gates.toml`, `ratchet.toml`, `protected-paths.toml`, `determinism-allow.toml` als neu anlegen | Alle vier existieren (409, 12, 102, 23 Zeilen). `ratchet.toml` hat ein anderes Schema. Zusätzlich vorhanden: `constitution.toml`, `verdict-required.toml`, `unsafe-safety-baseline.toml`, `determinism-baseline.toml`. Fehlend: `security_ports.toml`, `key_purposes.toml`, `clippy.toml`. | WP 0.4 erweitert und migriert. Neu angelegt werden nur die drei fehlenden Dateien. |
| K-03 | Spec „2.616 Zeilen“ | `wc -l` = 2.621 | nur Zählkorrektur |
| K-04 | `checkpoint/src/hardlink_cloner.rs:35,50,54` und `store/src/wal/mod.rs:145,148` als Stubs, die Erfolg melden (Gate `no-silent-noop`, [H]) | Die Stellen liegen in `#[cfg(loom)]`-Shims (`hardlink_cloner.rs:28–32`; `wal/mod.rs` Gating in Zeilen 20–25). Der Produktionspfad nutzt `tokio::fs` (`hardlink_cloner.rs:28–29`). Die Stellen sind **kein** Produktionsfehler, sondern Test-Shims im Produktionsmodul. | WP 0.2 ordnet das Gate neu (Loom-Shims als Ausnahme mit Begründung oder Verschieben in ein `cfg(loom)`-Testmodul). Der Eintrag B-18 bleibt als Gate-Befund, wird aber nicht als Laufzeitfehler gewertet. |
| K-05 | B-23: Stub im Default-Build von `engine` „ungeprüft“ | `engine/Cargo.toml`: `default = []`, `encryption-at-rest` nicht aktiv. Der Stub (`lib.rs:13,16`) ist also im Default-Build aktiv. Offen bleibt, ob `contextra`/`db`/`mcp` das Feature über Abhängigkeiten einschalten. | Maßnahme unverändert (WP 1.1). `cargo tree -e features` in WP 0.1 klärt nur die Aktivierungsfrage. |
| K-06 | WP 3.4: `testkit` um `FaultVfs`, `ManualClock` erweitern | `crates/contextra-testkit/src/fault_vfs.rs` und `manual_clock.rs` existieren. Ob `FaultVfs` ein Port-Trait implementiert, ist ungeprüft (das Trait `Vfs` existiert im Repo nicht). | WP 3.4 wird zur Anbindung an die neuen Ports, nicht zum Neubau. |
| K-07 | Determinismus: 44 Verstöße (Gate) bzw. 49 (Spec) | Gate-Aufschlüsselung: mvcc 14, engine 7, store 7, kvcache 3, ports 3, router 3, checkpoint 2, sandbox 2, adapt 1, db 1, privacy 1 = 44. Davon in Archiv-Crates (kvcache, router, sandbox, adapt): 9. In aktiven Crates: 35. | Die Zählung muss vor WP 3.5 auf ein Werkzeug festgelegt werden (WP 0.2). WP 3.5 bearbeitet nur die 35 aktiven Stellen. |
| K-08 | `async-io`: 18 Verstöße | Im Scan-Auszug (15 Zeilen) liegen 3 Stellen in `infer-onnx` und 4 in `vector/src/diskann/build.rs`, beide Archiv-Ziele; 3 weitere Stellen stehen nur in `--full`. Nach Archivierung verbleiben 8 aktive Stellen im Auszug plus 3 nicht sichtbare, also insgesamt 11. | Die Zahl sinkt durch WP 2.1 um 7, nicht durch Umbau. |
| K-09 | `KmsProvider` ist nur zu verschieben (WP 3.2) | `crypto/src/wal_crypto.rs:18–21` hat nur `get_key(&self) -> Result<Vec<u8>>`. Die Methoden der Spec (erzeugen, ableiten, wrappen, entwrappen, vernichten) fehlen. | WP 3.2 ist ein **Neuentwurf der API**, nicht nur eine Verschiebung. |
| K-10 | `kv_shredding` nur in `store/src/kv/segment.rs` genutzt | Zusätzlich in `crypto/src/audit_chain.rs:20`, `crypto/src/kv_cipher.rs:95`, `crypto/src/revocation_log.rs:323` (Test). `store/src/kv/` enthält `delete_mode.rs`, `mod.rs`, `segment.rs`. | WP 5.6 umfasst den Umbau dieser Abhängigkeiten, nicht nur die Streichung. |
| K-11 | `contextra_core` ist eine mechanische Ersetzung | 286 Dateien in `crates/`, `xtask/` und `benchmarks/` referenzieren `contextra_core`. | WP 2.3 braucht ein Migrationsskript mit Prüfung je Crate, nicht nur eine Suche und Ersetzung. |
| K-12 | Budgets sind in §3 eingehalten | `types` Ist 2.691 (Scan) bzw. 2.611 (Spec) gegen Budget 2.700: Reserve 9 bzw. 89 Zeilen. `ports` Ist 1.408 bzw. 1.526 gegen 1.500. `sys` Ist 428 gegen 450. | Die Löschtypen (WP 3.1), die Durability-Primitive (WP 3.3) und die Ports (WP 3.2) sprengen die Budgets. Entscheidung E-B1 (Abschn. 17). |
| K-13 | Lint-Satz vorhanden | `Cargo.toml:175–178`: `unexpected_cfgs = "warn"`. `[workspace.lints.clippy]` (ab Zeile 180) enthält kein `await_holding_lock`. | WP 0.4: `unexpected_cfgs = "deny"` und `await_holding_lock = "deny"` ergänzen. |
| K-14 | Gate-Name `ring-layering` | Gesamtspezifikation §17.3 führt `ring-layering` / `ring-dag`. Das Scan-Pack verwendet `ring-layering`. | Gate-Namen in `governance/gates.toml` vereinheitlichen (ein Name, ein Alias). |
| K-15 | WAL-Formate: `MFW3` | `wal/encode.rs:55–58`: `MFW2` (V2, batch-verschlüsselt) und `MFW3` existieren. Beide sind im Format-Register zu führen. | WP 4.3 erfasst `MFW2` als Altformat. |
| K-16 | D-08 (MFST entfernen) und `contextra migrate` in WP 8.4 | Die Entfernung von `MFST` braucht den Migrationspfad, der laut Plan erst in Phase 8 entsteht. | **Ordnungsproblem.** Der Migrationskern (offline, Dry-Run, verifiziertes Backup) wird in WP 4.3 vorgezogen. Die CLI bleibt in WP 8.4. |
| K-17 | Golden-Files (WP 4.3) und Layout-Wechsel (WP 5.3) | Golden-Files müssen aus dem **Ist-Format** entstehen, bevor das Record-Layout geändert wird. | **Ordnungsproblem.** WP 4.3 muss vor WP 5.3 abgeschlossen sein (hart, nicht nur „parallel“). |
| K-18 | `LayerWitness` in `types` (Spec §12.7.4) | Der Konstruktor soll nur über `ScanSession` erreichbar sein, und `ScanSession` braucht die Scanner-Trait-Methoden aus `ports`. Ein Typ in `types` kann diese Konstruktion nicht sicher kapseln. | Entscheidung E-W1: Die Struktur `LayerWitness` liegt in `ports::scan`. Die Grundtypen (`ScanMethod`, `WitnessKind`, `NotMeasuredReason`, Schichtnamen) bleiben in `types`. |
| K-19 | `LayerCleanupProof`-Platzierung im Stub | `engine/src/lib.rs` re-exportiert im Stub `LayerCleanupProof` (ab Zeile 44 im Scan-Auszug). Die echte Definition steht in `crypto/src/deletion_proof.rs:167`. | Bestätigt B-06. Die Typfamilie wird in WP 3.1 auf eine Quelle reduziert. |
| K-20 | `mcp`-Tools: „15 Werkzeuge `contextra_*`“ | Gesamtspezifikation §13.2 verlangt genau drei Werkzeuge `memory_remember`, `memory_recall`, `memory_forget`. Der Scan zählt die Ist-Werkzeuge nicht namentlich. | WP 8.3 führt die Ist-Liste mit Entsprechung oder Entfall auf (Tabelle dort). |

---

## 1. Kernbefunde des IST-Zustands

Die Befunde sind nach Wirkung auf die Löschkette und die Architektur geordnet. „WP“ verweist auf Abschnitt 4.

| ID | Befund | Beleg | Folge |
| --- | --- | --- | --- |
| B-01 | **Die Historie enthält einen Rollback.** Commit `0304f9a` („Zurückstellung auf kompilierbaren Zustand vom 5.10.26“) hat 2.507 Dateien geändert und u. a. entfernt: `ports/src/erasure.rs`, `store/src/lsm/purge/{orchestrator,intent,resume,plan,dispose,failpoint,wiring}.rs`, `store/src/lsm/ops/purge.rs`, `crypto/src/deletion_proof/*` (inkl. `proof_v4.rs`), `deletion_witness.rs`, `keystore.rs`. Der Scan listet diese Dateien noch als Hotspots der letzten 14 Tage; im HEAD existieren sie nicht. | `git show --stat 0304f9a`; `ls crates/contextra-ports/src` | Die Löschkette ist im HEAD **grüne Wiese**. Der Altstand liegt unter `0304f9a^` (`f09fe07`) und dient nur als Referenz für Testideen. Er wird **nicht** übernommen (siehe D-01). |
| B-02 | Der Sicherheitsstub `no_crypto_stubs` steckt in der Engine: `DeletionProof::verify` liefert `Ok(true)`, `LayerCleanupProof::verify_and_create` verwirft das Prüfergebnis (`let _ = verifier()`), `TenantIsolatedKvStore` hat leere Hooks. | `engine/src/lib.rs:12–211` (hinter `cfg(not(feature = "encryption-at-rest"))`) | Verstoß gegen A1/INV-WIRING-2. WP 1.1. |
| B-03 | Es gibt keine Messung am Speicher. `drop_collection` löscht, liest danach per `scan_prefix` (Lesepfad) und reicht die Zahl als `usize` in den (als deprecated markierten) Konstruktor `LayerCleanupProof::new_after_verified_empty`. Nur zwei Schichten, beide mit Aufrufer-Zahlen. | `engine/src/contextra_impl/collections.rs:276–385`; `crypto/src/deletion_proof.rs:183–228` | Verstoß gegen REQ-DEL-008. Ersatz durch Scanner (WP 6.x). |
| B-04 | Dummy-Beweise und Fallbacks im MCP: Mandant fällt auf `TenantId::SYSTEM`, Transaktion auf `TxId::new(1)`, fehlender Schlüssel auf das Literal `default_test_deletion_proof_key_32_bytes!`; `handle_delete` übergibt die Konstante `0` als Restzahl. | `mcp/src/tools_crud.rs:127,193,206,210`; `mcp/src/server_tools.rs:449` | U-22/U-23/U-37 bestätigt. WP 1.2. |
| B-05 | Der Witness ist fälschbar und verliert Information: `LayerWitness::issue_from_scanner` ist `pub` und nur `#[doc(hidden)]`; `layer()` liefert `&str`; `DeletionProof::issue` bildet unbekannte Layer-Strings **still auf `LsmMemtable`** ab, ignoriert `_custody`, setzt den Scope hart auf `Tenant` und übergibt `Vec::new()` als gelöschte Schlüssel. | `types/src/witness.rs:56–67`; `crypto/src/deletion_proof.rs:353–466` | U-40/U-42 bestätigt. WP 3.1, 6.8, 7.4. |
| B-06 | Mehrere Typfamilien für Löschung: `DeletionLayer` (crypto, 7 Varianten; engine-Stub, 3 Varianten), `DeletionProof` (crypto, engine-Stub, `graph/src/cascade.rs:84`), `ExcludedScope`, `DeletionScope`, `TenantIsolatedKvStore` (crypto, engine, kvcache), `KeyManager` (crypto und `store/src/wal/mod.rs:247`). | Scan §10 | Kantenregel 5 verletzt. WP 1.1, 2.1, 3.1. |
| B-07 | **Ebene B ist unmöglich:** Es gibt genau einen Store-weiten Schlüssel. `KeyManager::try_new(passphrase, salt)` leitet per HKDF direkt aus der Passphrase ab (INV-KDF-1 verletzt); das Salt ist zwar zufällig und liegt in der Datei `SALT`, die Argon2id-Variante `try_new_with_kdf` (mit `KdfHeader`) existiert, wird im Store-Pfad aber nicht benutzt. Es gibt weder Keystore `MFKS`, noch KeyTable, noch DocDek/ArtKey/KEK. | `crypto/src/crypto.rs:71–134`; `store/src/lsm/recovery.rs:170–200`; `store/src/lsm/engine.rs:365–385` | Neubau der Schlüsselhierarchie (WP 5.x). |
| B-08 | Die Ports der Löschkette fehlen vollständig: `Vfs`, `DurableLog`, `ErasureJournal`, `LayerScanner`, `Purger`, `EffaceableKeySlot`, `ErasedSetMirror` ergeben 0 Treffer. `KmsProvider` ist ein Trait mit `get_key() -> Result<Vec<u8>>` und **ohne jede Implementierung** (UNWIRED). | `crypto/src/wal_crypto.rs:18`; Scan §7 | WP 3.2, 5.1. |
| B-09 | Die Sidecar-Logs sind nicht produktiv: `AuditChain` und `RevocationLog` werden außerhalb von `crypto` nur in Tests verwendet. `kv_shredding::KeyRegistry` ist das alte Gruppen-Shred-Modell (`DEFAULT_SHRED_KEY_GROUP_SIZE`). Es wird von `store/src/kv/segment.rs` **und** innerhalb von `crypto` (`audit_chain.rs:20`, `kv_cipher.rs:95`, `revocation_log.rs:323`) benutzt (K-10). | `grep` über `crates/` | Neubau nach §7.9; Altmodell streichen (WP 5.6). |
| B-10 | Durabilität ist plattformblind: `fsync.rs` nutzt `tokio`-Dateien; **unter Windows liefert es `Unsupported`**, `F_FULLFSYNC`/`FlushFileBuffers` kommen **nirgends** vor. Dateizugriff läuft direkt über `std::fs`/`tokio::fs` (Scan `async-io`: 18 Verstöße). | `store/src/fsync.rs`; Gate `async-io` | INV-DURABILITY-PRIM-1 offen. WP 3.3, 4.1. |
| B-11 | Formate: WAL `MFW2` und `MFW3` (`wal/encode.rs:55–58`), SSTable `MFSX` plus Legacy `MFST` (`sstable/block_cache.rs:340–341`, Lesepfade in `reader.rs:328–423`), Manifest `MFMN` (`manifest/core.rs:17`). Für Keystore, Sidecars, KeyTable, Bericht gibt es keine Formate. | Stichprobe | WP 4.3. |
| B-12 | HNSW verwendet Arena-Slots über eine `free_list` wieder; in `vector/src/hnsw` finden sich keine `zeroize`-Treffer. | `vector/src/hnsw/arena.rs:60,121–134,296–300` | INV-HNSW-SLOT-1 verletzt; Slot-Nullung fehlt. WP 6.2. |
| B-13 | Der Textindex liegt als Präfix-Schlüssel (`__txt:<collection>:`) im `StorageEngine`, nicht als segmentierte Artefakte unter einem ArtKey. Ob Terme gelöschter Dokumente aus dem Wörterbuch verschwinden, ist **ungeprüft**. | `engine/.../collections.rs:296–304`; `text/src/inverted/*` | WP 6.3. |
| B-14 | Kern hängt hart am Rand (alle Kanten nicht optional): `engine → graph, adapt, kvcache, infer-candle, rank`; `db → cognition, graph, adapt, crypto`; `mcp → rank, adapt, crypto, privacy, license`; `contextra → core, license, store, db`; `vector → crypto, core`; `simd → core`; `store → core`, Dev-Kanten `store → graph, sandbox, vector`; `checkpoint → core`. | Manifest-Auswertung | INV-CORE-ISOLATION-1 verletzt. WP 2.x. |
| B-15 | Sicherheit ist ein Feature: `encryption-at-rest` mit 14 `cfg`-Stellen in `store` und 13 in `engine`; `contextra` hat `default = ["fast", "candle"]`; 24 tote Features; `unexpected_cfgs = warn`. | Scan; `crates/contextra/Cargo.toml` | A1, §4.5. WP 1.1, 2.4. |
| B-16 | MCP-Oberfläche: 15 Werkzeuge `contextra_*` (u. a. `insert`, `search`, `forget`, `delete`, `drop_collection`, `cloud_query`, `relate`, `consolidate`), kein `memory_*`; Konfiguration über `std::env::var` (`mcp/src/config.rs`, `proof_key_env.rs`). | `mcp/src/server_dispatch.rs` | U-32. WP 8.3. |
| B-17 | Governance-Artefakte fehlen: `governance/security_ports.toml`, `governance/key_purposes.toml`, `docs/spec/SPEC.md`, Root-`CHANGELOG.md`; `capabilities.toml` (Schema 2) kennt kein Feld `class`. Mehrere konkurrierende Spezifikationen liegen in `docs/` (z. B. `CONTEXTRA_SPEZIFIKATION.md`, `GESAMTSPEZIFIKATION.md`). | Scan §15; `docs/` | A9. WP 0.3, 0.4. |
| B-18 | Gate-Lage (Scan): 7 blockierende Gates rot (`ring-layering` 2 Fehler + 66 Warnungen, `feature-consistency` 24, `test-surface` 18, `determinism` 44, `raw-carrier` 2, `unsafe-audit` 5, `async-io` 18); `deletion-honesty` ist grün, **weil es die Lücken nicht sieht** (0 Pending-Stellen). | `contextra_scan_gates.txt` | WP 9.1. |
| B-19 | Checkpoints sind Hardlink-Klone (`checkpoint/src/hardlink_cloner.rs`) ohne `ErasedSet`, ohne Spur-D/K-Trennung; `checkpoint` hängt an `contextra-core`. | Dateiliste; Manifest | WP 7.3. |
| B-20 | Schlüsselerzeugung nutzt `rand::thread_rng` an fünf Stellen (`store/src/wal/hmac.rs:246,251,367,456`, `store/src/lsm/recovery.rs:31`). Für Schlüssel/Salts erlaubt (P28-Ausnahme), muss aber in `determinism-allow.toml` stehen; die Nutzung für Zähler/Nonces an `hmac.rs:251,367,456` ist **zu prüfen**. | Scan `crypto-rng` | WP 3.5. |

---

## 2. Entscheidungen (Auflösung bisheriger Architektur)

Die folgenden Entscheidungen sind für die Umsetzung bindend. Wo sie eine frühere Entscheidung ersetzen, steht das dabei. Jede ist ADR-pflichtig (Spec §18.6) und wird in WP 0.3 als ADR angelegt. Von den ADR-Dateien habe ich nur die Titel gesehen; die genannten Ersetzungen sind deshalb **Kandidaten**, die beim Anlegen der ADRs gegen den Inhalt zu prüfen sind.

| ID | Entscheidung | Ersetzt / betrifft | Begründung |
| --- | --- | --- | --- |
| D-01 | **Die Löschkette wird nach Spec §12 neu gebaut. Der Altstand aus `0304f9a^` wird nicht per Cherry-Pick übernommen.** Wiederverwendet werden nur Testideen, Fixtures und Fehlerfälle. | ENTSCHEIDUNGEN.csv (E1–E8, U-36) | Der Altstand folgt dem Modell vor V5/Epoch Seal/DocDek; ein Zurückholen schleppt V4, Gruppen-Shredding und Aufrufer-Zahlen zurück. |
| D-02 | **Eine Typfamilie je Konzept.** Löschtypen existieren nur in `types` (Daten), `ports` (Traits) und `crypto` (Bericht). Alle anderen Crates re-exportieren. | Doppeldefinitionen B-06 | Kantenregel 5, A2 |
| D-03 | **Verschlüsselung ist immer aktiv.** Das Feature `encryption-at-rest` und `cfg(not(feature))`-Ersatzcode entfallen. K0 (keine Verschlüsselung) ist eine Laufzeit-Betriebsart nur für `EdgeMinimal`/Entwicklung, von `doctor` rot gemeldet. | `encryption-at-rest` (B-15) | A1, INV-ENCRYPT-REST-1 |
| D-04 | **Archivierung per Git-Tag, nicht per Feature-Schalter.** Tag `archiv/v0`, danach Entfernen aus dem Workspace. Keine Zwischenlösung „hinter Feature“. | ADR-020 (Cognitive OS), ADR-026/027 (PPR, Community), ADR-042/045/046 (Agent/Router) als Kandidaten | §4.2, §18.5 |
| D-05 | **`contextra-core` entfällt** (nur Migrationsfassade, REQ-ARCH-013). Nutzer wechseln auf `types`/`ports`. | B-14, B-17 | Reduziert Kanten und Doppelpfade |
| D-06 | **Schlüsselhierarchie nach §12.2.2 ersetzt den Einzelschlüssel.** `KeyManager` bleibt als interner Träger von `K_eng`; Passphrasen laufen ausschließlich über Argon2id mit `KdfHeader`. | `try_new(passphrase, salt)` (HKDF) | INV-KDF-1 |
| D-07 | **WAL/SSTable/Blob-Records erhalten ein neues Record-Layout** (DocDek-Umschlag, Padmé, Header unter `K_eng`). Magic bleibt, die Formatversion steigt. Vor 1.0 sind inkompatible Wechsel erlaubt, wenn `contextra migrate` sie abdeckt (REQ-FMT-7). | `MFW2`, `MFST`-Legacy | Ein Migrationspfad statt Dauer-Kompatibilität |
| D-08 | **SSTable-Legacy `MFST` wird entfernt** (WP 2.4-Entscheidung der Spec). Bestand läuft über `migrate`. | `sstable/reader.rs:423` | weniger Lesepfade, kleinere Angriffsfläche |
| D-09 | **Async bleibt Ratchet.** Scanner und Purger der Ring-0-Crates (`vector`, `text`) sind synchron; `store` ruft sie über `spawn_blocking`. Kein neuer `async fn` in Ring 0. | E15/E29 | Spec §6 |
| D-10 | **K2/K3 nur als Port.** `EffaceableKeySlot` liefert für K2/K3 `CapabilityUnsupported`. | E27 (teilweise) | §12.2.3 |
| D-11 | **Windows ist keine Garantieplattform.** `FlushFileBuffers`/`ReplaceFile` werden in `sys` als Best-Effort-Primitive vorgesehen; Berichte tragen dort `platform_guarantee = false`. | O-18 | INV-PLATFORM-1 |
| D-12 | **Test-Zugänge verschwinden aus der Produktions-API.** Alle `*_for_test`-Methoden (`store/src/lsm/engine.rs:119–279`, `wal/hmac.rs:187`, `contextra-py`-Auslöser) liegen hinter `cfg(test)` oder dem Feature `test-hooks`. | Scan `test-surface` (18) | INV-SCAN-1 |
| D-13 | **Der Löschpfad hat zwei Messquellen je Pflichtschicht:** Scanner der Schicht und Black-Box-Probe des Orchestrators. Eine Aufrufer-Zahl ist nirgends Messung. | `new_after_verified_empty(layer, usize)` | §12.7.1, REQ-DEL-008 |
| D-14 | **`MetricsSink` bekommt eine echte Senke** (strukturiertes Log ohne Nutzinhalte) im Composition Root; `TestMetricsSink` verlässt den Produktionscode. | Scan: NULL\_ONLY | §5.2 |

---

## 3. Ziel-Crate-Matrix

Zeilen (Code-LOC) stammen aus dem Scan (`LOC Produktion`); sie weichen von den Spec-Zahlen ab, weil die Zählwerkzeuge verschieden sind (§1.3). Das Gate `loc-budget` pinnt später ein Werkzeug (WP 9.1).

### 3.1 Kern

| Crate | Ring | IST (LOC) | SOLL (Budget) | Wichtigste Aktionen |
| --- | --- | --- | --- | --- |
| `contextra-types` | 0 | 2.691 | 2.700 | Löschtypen (zehn Schichten), `ErasureId`, Referenz-Hash-Typen, `ReachedClass`, `Assumption`, `RequestContext` mit `CancelToken`/`Deadline`; `LayerWitness` privat |
| `contextra-ports` | 0 | 1.408 | 1.500 | neue Sicherheitsports (WP 3.2); Ports archivierter Crates raus (`graph`, `graph_index`, `attention`, `kv_bridge_port`, `plugin`, `TextGenerator`, `query_rewriter` bleibt Rand) |
| `contextra-sys` | 0 | 428 | 450 | Durability-Primitive, `mlock`, `madvise`, Dateisperre; SAFETY-Lücken (3 Stellen) |
| `contextra-simd` | 0 | 1.113 | 1.200 | Abhängigkeit von `contextra-core` entfernen; Kernel `pub(crate)`; SAFETY in den Benches; `allow(undocumented_unsafe_blocks)` entfernen |
| `contextra-wire` | 0 | 1.194 | 200 + generiert | `allow(unwrap_used, expect_used, panic, undocumented_unsafe_blocks)` auf generierte Datei beschränken; Gate trennt generiert/handgeschrieben |
| `contextra-mvcc` | 0 | 1.585 | 1.600 | Zeitaufrufe (14) über `Clock`; SSI-Nachweis (T-SSI-1) |
| `contextra-crypto` | 0 | 3.105 | 3.200 | Keystore/KMS K0/K1, KeyTable-Logik, Padmé, Bericht V5, Sidecar-Kettenlogik; **I/O-frei**; `kv_shredding`/`kv_segment`/`kv_cipher` raus |
| `contextra-vector` | 0 | 8.218 | 6.800 | Abhängigkeiten auf `crypto`/`core` entfernen; `diskann` und `experimental-*` archivieren; Slot-Nullung, Generationswechsel, `VectorScanner`; 52 `async fn` nach Ratchet |
| `contextra-text` | 0 | 2.329 | 2.400 | Panic in `tokenizer.rs:30`; ArtKey-Segmente, `TextScanner`; 43 `async fn` nach Ratchet |
| `contextra-rank` | 0 | 2.972 | 1.750 | nur `fusion/` behalten; Rest (Kalibrierung, DiBuD, Drift) ins Archiv |
| `contextra-store` | 1 | 13.096 | 13.400 | `Vfs`-Impl, `DurableLog`-Impl, Sidecars, Record-Umschlag, Purge-Kompaktion, `LsmScanner`; `core`-Abhängigkeit weg; Dev-Aufwärtskanten (`graph`, `sandbox`, `vector`) verlagern |
| `contextra-checkpoint` | 1 | 1.630 | 1.700 | Spur D/K, Restore-Guard, `CheckpointScanner`; `core` weg (`hardlink_cloner.rs` hat einen `cfg(loom)`-Shim, K-04) |
| `contextra-engine` | 3 | 10.100 | 8.500 Ziel / 10.200 Grenze | Entkopplung von `graph`, `adapt`, `kvcache`, `infer-candle`, `sandbox`; Saga, Lesesperre, Boot, Reaper-Ort entscheiden |
| `contextra-db` | 3 | 585 | 600 | Entkopplung von `cognition`, `graph`, `adapt`; keine Logik |
| `contextra-mcp` | 4 | 3.745 | 3.400 | drei Werkzeuge, Pipeline, `confirm`, IPC; Abhängigkeiten auf `rank`, `adapt`, `privacy`, `license`, `crypto` entfernen |
| `contextra` | 4 | 729 | 800 | Composition Root, Boot B0–B9, Default `embedded`, Rand nur optional |

### 3.2 Archiv (Tag `archiv/v0`, aus dem Workspace)

`contextra-graph` (8.708), `contextra-cognition` (3.191), `contextra-adapt` (2.193), `contextra-kvcache` (2.434), `contextra-router` (1.465), `contextra-agent` (2.413), `contextra-infer-onnx` (952), `contextra-infer-ollama` (1.661), `contextra-core` (106), `vector/diskann`, `rank` ohne `fusion/`. Zusammen rund 25.000 Zeilen Produktionscode, dazu die zugehörigen Tests (Scan: 138.864 Testzeilen im Workspace).

### 3.3 Rand und Experimentell (optional, im Release-Default aus)

`infer-candle` (Embedding-Backend), `privacy`, `sandbox`, `py`, `license`, `audit-export`, `avv-generator`. Diese Crates bleiben im Workspace, werden von der Fassade nur über `optional = true` und Features eingebunden und liegen außerhalb von `default-members`.

### 3.4 Werkzeug

`testkit` (erweitern: Fake-Verwahrer und Port-Anbindung; `FaultVfs` und `ManualClock` existieren bereits, K-06), `xtask` (29.706 Zeilen, **Schutzpfad**; Inventar vor jeder Kürzung), `xtask-heavy`, `bench` (4.303 Zeilen, enthält `unsafe`).

---

## 4. Phasenplan und Arbeitspakete (Überblick)

Regeln: Ein Arbeitspaket (WP) ist ein Pull-Request mit höchstens einem Crate-Schnitt (REQ-CHG-2). Jedes WP beginnt mit einem fehlschlagenden Test (§17.2). Nach jedem Schnitt laufen Build, Löschtest und Zeilenzählung. Die Nummern `IMP-x.y` sind dieses Dokument; die Spalte „Spec“ nennt die Entsprechung.

| WP | Inhalt | Crates | Hängt ab von | Spec |
| --- | --- | --- | --- | --- |
| **Phase 0 – Baseline und Netz** |  |  |  |  |
| 0.1 | Build-/Test-/Clippy-Baseline mit Evidenzdatei | alle | – | §1.2, EP13 |
| 0.2 | Gate-Läufe `--full` neu messen und archivieren | xtask | 0.1 | WP 0.2 |
| 0.3 | Spec nach `docs/spec/SPEC.md`; konkurrierende Dokumente archivieren; Vetos bestätigen; ADRs D-01…D-14 | docs, governance | – | WP 0.3, U-30/31 |
| 0.4 | Governance-Skelett: `gates.toml`, `ratchet.toml`, `protected-paths.toml`, `security_ports.toml`, `key_purposes.toml`, `determinism-allow.toml`, `clippy.toml`, Workspace-Lints | governance, xtask | 0.1 | L-2, U-29 |
| **Phase 1 – Stubs und Sofortschäden** |  |  |  |  |
| 1.1 | Engine-Stub und Feature `encryption-at-rest` löschen | engine, store | 0.1 | L-12, REQ-PRF-1 |
| 1.2 | MCP: Literal-Schlüssel, Mandanten-/Tx-Fallbacks, Dummy-Beweise fail-closed | mcp | 0.1 | L-12, REQ-PRF-4, FGT-7 |
| 1.3 | Panics und Test-Hooks aus der Produktions-API | store, text, mcp, py | 0.1 | P7, INV-SCAN-1 |
| **Phase 2 – Archivierung und Entkopplung** |  |  |  |  |
| 2.1 | Archiv-Tag, Entfernen der Archiv-Crates | Workspace | 1.x | §18.5 |
| 2.2 | `engine` entkoppeln | engine | 2.1 | §4.4 |
| 2.3 | `db`, `mcp`, `contextra` entkoppeln; `contextra-core` entfernen; Default `embedded` | db, mcp, contextra, Core-Nutzer | 2.2 | §4.5, D-05 |
| 2.4 | Rand-Crates optional; Brücken über Ports | contextra, Rand | 2.3 | §4.7 |
| 2.5 | `capabilities.toml` auf Klassen und strikte `may_depend_on` | governance, xtask | 2.3 | WP 3.6 |
| 2.6 | Unsafe-Inseln bereinigen | simd, sys, wire, Tests | 0.4 | §4.6 |
| **Phase 3 – Fundament** |  |  |  |  |
| 3.1 | Löschtypen in `types` | types | 2.3 | L-1 |
| 3.2 | Sicherheitsports in `ports` | ports | 3.1 | §5, L-2 |
| 3.3 | Durability-/Härtungsprimitive in `sys` | sys | 3.2 | §7.7, §12.18 |
| 3.4 | `testkit`: `FaultVfs`, `ManualClock`, Fake-Verwahrer | testkit | 3.2 | §17.4 |
| 3.5 | Determinismus durchsetzen (44 Zeitaufrufe, RNG-Stellen) | mvcc, engine, store, ports u. a. | 3.2 | P28 |
| **Phase 4 – Store** |  |  |  |  |
| 4.1 | `Vfs`-Implementierung, alle Dateizugriffe, `async-io` | store, checkpoint | 3.3, 3.4 | §7.7 |
| 4.2 | `DurableLog` und Sidecar-Rahmen | store | 4.1 | §7.9 |
| 4.3 | Format-Register, Versionierung, Golden-Files | store u. a. | 4.1 | §7.11 |
| 4.4 | Betriebszustände, Notreserve, Poison | store | 4.1 | §7.8, §7.10 |
| **Phase 5 – Schlüssel** |  |  |  |  |
| 5.1 | `KmsProvider` K0/K1, Keystore `MFKS`, Argon2id | crypto, store | 3.2, 4.1 | INV-KDF-1 |
| 5.2 | KeyTable `MFKT`, `KeyCache`, Hierarchie | crypto, store | 5.1, 4.2 | L-4 |
| 5.3 | Record-Umschlag, Padmé, Header-Verschlüsselung | store, crypto | 5.2, 4.3 | §12.2.5/6 |
| 5.4 | Destroy-Protokoll D1–D6 | crypto, store, engine | 5.3 | L-4 |
| 5.5 | Epoch Seal | crypto, engine | 5.4 | L-5 |
| 5.6 | Sidecars produktiv (`KeyRegistry`, `RevocationLog`, `AuditChain`) | crypto, store | 4.2 | §7.9 |
| **Phase 6 – Scanner und Purger** |  |  |  |  |
| 6.1 | `LsmScanner`/Purger (Memtable, WAL, SST, Blob), Purge-Kompaktion | store | 5.3 | L-3 |
| 6.2 | `VectorScanner`, Slot-Nullung, Generationswechsel | vector | 3.2 | §9.3, §12.16 |
| 6.3 | `TextScanner`, ArtKey-Segmente | text | 3.2, 5.2 | §10.1 |
| 6.4 | `CacheScanner` | store, engine | 3.2 | REQ-DEL-N7 |
| 6.5 | `CheckpointScanner`, Policy | checkpoint | 5.3 | §12.5.4 |
| 6.6 | `KeyScanner`, Nadelscan, Inventar | crypto, store | 5.4 | §12.7.3 |
| 6.7 | Lineage-/Sidecar-Scanner | engine, crypto | 5.6 | INV-DERIVED-1 |
| 6.8 | `ScanSession`, `ControlToken`, Mint-Härtung | types, ports | 3.1 | L-3 |
| **Phase 7 – Orchestrator** |  |  |  |  |
| 7.1 | `ErasedSet` und Lesesperre über alle Lesepfade | engine, store | 4.2 | §12.9 |
| 7.2 | Saga, Journal, Typestate | engine | 6.x, 7.1 | L-8 |
| 7.3 | Backup Spur D/K, Restore-Guard, Mirror, Wiederauferstehung | checkpoint, engine | 5.4, 7.1 | L-6, L-7 |
| 7.4 | Bericht V5 und Siegel | crypto, engine | 7.2 | L-9 |
| 7.5 | Claim-Generator und Satzbibliothek | crypto, xtask | 7.4 | L-10 |
| 7.6 | Fristen, Mixed-Policy, `RecipientLedger`, `LegacyUnkeyed`, Restmetadaten-Test | engine, crypto | 7.2 | L-11, L-13 |
| **Phase 8 – Fassade, Kernhärtung, Schnittstellen** |  |  |  |  |
| 8.1 | Composition Root, Boot B0–B9, Selbsttest | contextra, engine | 7.x | §13.4/5 |
| 8.2 | `remember`/`recall`, Embedding-Identität, Index je Tenant | engine, db | 2.x | §11 |
| 8.3 | MCP: drei Werkzeuge, Pipeline, Forget-Guard, `confirm`, IPC | mcp | 8.1 | §13 |
| 8.4 | `doctor` und CLI | mcp, contextra | 8.1 | §23 |
| 8.5 | SSI-Nachweis, loom-Inventar, Fuzz-Matrix | mvcc, engine | 2.x | T-SSI-1 |
| **Phase 9 – Abnahme** |  |  |  |  |
| 9.1 | Gate-Satz vollständig (neue Gates, Selbsttests) | xtask | 7.x | §17.3 |
| 9.2 | AC-Matrix, Crash-Matrix, Canary-Läufe | testkit, tests | 9.1 | §12.12, §17.9 |
| 9.3 | Benchmarks und fremde Baselines; `budgets.toml` | bench | 8.x | §16 |
| 9.4 | Release-Kriterien und Review-Vorbereitung | – | 9.x | §18.3 |

**Kritischer Pfad der Löschkette:** 0.1 → 1.1 → 2.1–2.3 → 3.1 → 3.2 → 4.1 → 5.1 → 5.2 → 5.3 → 5.4 → 6.x → 7.2 → 7.4 → 8.1 → 9.2. Alles andere (Gates, Doku, Unsafe-Inseln, Determinismus) läuft parallel, solange es keinen gemeinsamen Crate-Schnitt belegt.

---

## 4a. Nachtrag zu den Befunden und Reihenfolge-Korrektur

| ID | Befund | Beleg | Folge |
| --- | --- | --- | --- |
| B-21 | `Collection::delete` gibt den Key-Lock **sofort wieder frei** (`drop(_guard)` vor `begin_transaction`). Damit ist REQ-DEL-012 (Lock über die gesamte Transaktion) und AC-13 (`put`/`delete` derselben ID) strukturell verletzt. | `engine/src/collection/crud/delete.rs:8–11` | WP 7.1/7.2; loom-Test |
| B-22 | Es gibt **keine Blob-Segmente**: 0 Treffer für `MFBL`, `blob_threshold`, `BlobSegment` in `store`, `engine`, `types`. Die Schicht `BlobSegments` (S1b) ist ein Neubau. | `grep` | WP 5.3 |
| B-23 | **Teilweise bestätigt (K-05):** `contextra-engine` hat `default = []`; `encryption-at-rest` ist dort nicht aktiv, daher ist der Stub aus B-02 im Default-Build von `engine` aktiv (`lib.rs:13,16`). **Ungeprüft:** ob `contextra`, `db` oder `mcp` das Feature über Abhängigkeiten einschalten; WP 0.1 klärt es mit `cargo tree -e features`. | `engine/Cargo.toml`, `db/Cargo.toml` | Bedeutung von B-02 |

**Reihenfolge-Korrektur zu §4.** WP 2.1 entfernt die Archiv-Crates nicht sofort, weil `engine`, `db` und `mcp` noch von ihnen abhängen. Die Folge lautet: **2.1** (Tag, Kanten-Inventar, Crates aus `default-members`) → **2.2** → **2.3** → **2.1b** (physisches Entfernen, sobald kein Dependent mehr existiert).

---

## 5. Phase 0 – Baseline und Netz

### WP 0.1 – Build-, Test- und Clippy-Baseline

**Ist.** Niemand hat auf `a547a42` Kompilierung oder Tests belegt (Spec §1.2). Der Rollback `0304f9a` nennt einen „kompilierbaren Zustand vom 5.10.26“; seitdem liegen 88 Commits (K-01).

**Änderung.** Keine Codeabsicht. Es entsteht `evidence/baseline-a547a42/` mit je einer Datei pro Befehl (Befehl, Toolchain, Commit, Exit-Code, Dauer, Anzahl ausgeführter/übersprungener Tests, Hash des Rohprotokolls; INV-EVIDENCE-1):

- `cargo fetch --locked`, `cargo check --workspace --all-targets --locked` (Toolchain laut `rust-toolchain.toml`: 1.89.0)
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked --no-run`, danach `cargo test` je Kern-Crate einzeln (`types`, `ports`, `crypto`, `mvcc`, `store`, `vector`, `text`, `checkpoint`, `engine`, `db`, `mcp`, `contextra`)
- `cargo tree -e features -p contextra-engine -p contextra-db -p contextra-mcp -p contextra` (klärt B-23)
- `cargo xtask` mit den Befehlen aus `xtask/registry.toml`, die laut Scan die Gates speisen

**Verzweigung.** Ist der Baum rot, entsteht ein eigener Pull-Request „Kompilierbarkeit herstellen“ ausschließlich mit Compile-Fixes (kein Feature, kein Refactoring). Ohne grüne Baseline beginnt keine weitere Phase.

**Akzeptanz.** Alle Evidenzdateien liegen im Repo; jede rote Stelle hat einen Eintrag in `evidence/baseline-a547a42/OPEN.md` mit Crate und Ursache.

### WP 0.2 – Heuristik-Befunde neu messen

**Änderung.** Die Gates laufen mit `--full` (die Scan-Ausgabe kürzt 53 bzw. 79 Einträge) und werden unter `evidence/gates/a547a42/` archiviert. Daraus entsteht `docs/spec/IST-BEFUNDE.md` (generiert). Es schließt oder bestätigt mindestens: U-06 (`supports_ssi_tracking` konstant `true`, `store/src/lsm/ops.rs:53`), U-24/U-25 (laut Spec behoben; `determinism` meldet dennoch Zeitaufrufe in `store`, `ports`, `mvcc`), U-26 (`text/src/tokenizer.rs:30`), U-27 (Spec nennt `hmac.rs:194`, der Scan nennt `wal/mod.rs:92`), U-34 (35 Workspace-Mitglieder).

**Akzeptanz.** Jeder U-Befund trägt Commit, Befehl und Ergebnis. Die zwei Fehler des Gates `ring-layering` sind namentlich bekannt (sichtbar ist nur `infer-ollama → infer-onnx [dev]`).

### WP 0.3 – Spec ablegen, Dokumente konsolidieren, ADRs

**Ist.** Es liegen mehrere konkurrierende Spezifikationen vor (`docs/CONTEXTRA_SPEZIFIKATION.md`, `GESAMTSPEZIFIKATION.md`, `FEATURES_SPECIFICATION.md`, `CONTEXTRA_IMPLEMENTIERUNGSPLAN.md`, `CONTEXTRA_META_PLAN.md`, `SOURCE_OF_TRUTH.md`) sowie Root-Berichte (`PLAN.md`, `JULES_PROMPTS.md`, `FINDINGS_REPORT.md`, `BEFUNDE_NACHPRUEFUNG.md`, `ZEILENRECHNUNG.md`, `FRAGEN_FACHBERATUNG.md`). Mehrere `xtask`-Gates (`check-doc-references`, `check-agents-integrity`, `check-toc-integrity`, `doc-truth`) lesen Dokumente; ob sie diese konkreten Dateien referenzieren, ist **ungeprüft**.

**Änderung.**

1. Spec nach `docs/spec/SPEC.md` (U-30), Stempel mit Commit (R-18).
2. Zuerst `xtask`-Referenzen auf die Kandidaten per `rg` ermitteln, dann im selben PR Gate-Konfiguration und Dateien anpassen (Schutzpfad: ADR plus Freigabe nach `protected-paths.toml`). Danach die Kandidaten in den Tag `archiv/v0` verschieben.
3. ADRs zu D-01…D-14 in `docs/decisions/` mit Begründung, Alternativen, Freigabe. Beim Anlegen werden die Kandidaten ADR-020, ADR-023, ADR-026/027, ADR-042/045/046 auf „superseded“ geprüft.
4. `VETOES.md`: VETO-F02 und VETO-OP03 stehen auf `conditionally_accepted` mit Frist 2026-10-07 (abgelaufen, U-31). Beide werden im selben PR nach Prüfung auf „bestätigt“ gesetzt. Der Scan meldet für F02 sechs Code-Treffer (`vector/src/hnsw/core_rebuild.rs:614–618`, `hnsw/types.rs:411–412`); diese Stellen sind auf Rewiring zu prüfen (WP 6.2).
5. `ENTSCHEIDUNGEN.csv` (E1–E8) wird durch einen Verweis auf Spec §20 ersetzt (U-36).
6. `AGENTS.md`, `ARCHITECTURE.md`, `CONSTITUTION.md`, `VETOES.md`, `docs/TYPE_REGISTRY.md`, `docs/decisions/README.md` erhalten Commit-Stempel (Gate `doc-stamp`, 7 Warnungen).

**Akzeptanz.** `doc-stamp` grün; genau eine normative Quelle; ADR-Index vollständig.

### WP 0.4 – Governance-Skelett

**Änderung (neue und zu erweiternde Dateien; K-02).** `gates.toml`, `ratchet.toml`, `protected-paths.toml` und `determinism-allow.toml` **existieren bereits** und werden erweitert bzw. migriert. `ratchet.toml` hat heute ein anderes Schema (`unwrap_legacy_grep`, `[unsafe_blocks]`) und wird auf die Baselines aus §6 und §17 umgestellt. Neu anzulegen sind `security_ports.toml`, `key_purposes.toml` und `clippy.toml`.

| Datei | Inhalt |
| --- | --- |
| `governance/security_ports.toml` | je Sicherheitsport: `name`, `crate`, `trait_path`, `class`, `prod_impl`, `wiring_test`. Kern-Einträge: `KmsProvider`, `Vfs`, `DurableLog`, `ErasureJournal`, `LayerScanner`, `Purger`, `GroupRevocationListener`, `EffaceableKeySlot`, `ErasedSetMirror`; Rand: `ModuleVerifier`, `FilterModule`, `EntityRecognizer`, `InjectionDetector`, `EgressChannel` |
| `governance/key_purposes.toml` | Zeilen aus Spec §12.3: `eng`, `wal`, `sc/keyreg`, `sc/revoke`, `sc/audit`, `sc/journal`, `sc/erased`, `keystore-root`, `kek`, `docdek`, `artkey`, `kref`, `report-sign`, `vault`; je Label `contextra/v1/<zweck>` und Wirkung der Vernichtung |
| `governance/determinism-allow.toml` | Ausnahmen für System-RNG (Schlüssel, Salt, Nonce, DocDek, KEK, `K_ref`); **keine** Zeit-Einträge |
| `governance/ratchet.toml` | Baselines, die nur sinken: Panic-Aufrufe 2, `async fn` je Ring-0-Crate (`vector` 52, `text` 43, `rank` 1), SAFETY-Lücken 49, sensitive `allow` 6, Zeitaufrufe 44, tote Features 24, Mutation-Score je Kern-Crate |
| `governance/gates.toml`, `protected-paths.toml` | eine Zeile je Gate (Name, Phase, Pflichtstatus, Risikoklasse, Besitzer, Fixture); geschützte Pfade nach Spec §18.2 |
| `clippy.toml` (Workspace-Wurzel) | `disallowed-methods`: `std::time::Instant::now`, `std::time::SystemTime::now`, `rand::thread_rng`, `std::env::var`; `disallowed-types`: `std::collections::HashMap`/`HashSet` mit `RandomState` in Kernlogik (Ersatz: Typalias mit festem Seed) |

Workspace-`Cargo.toml`: Lint `clippy::await_holding_lock = deny` ergänzen (Scan: fehlt), `unexpected_cfgs = deny` (heute `warn`), `lints.workspace = true` in den 8 Crates, die es nicht tragen (sieben `*-fuzz`-Crates, `xtask`).

**Test (rot zuerst).** Gate-Selbsttests (`gate-selftests`) mit synthetischem Repo je neuer Datei: fehlende Datei → Gate rot (heute `security-ports`: „FEHLT“ wird zu `FAIL`, nicht `WARN`).

**Akzeptanz.** `security-ports` und `key-purposes` laufen und melden die heute fehlenden Implementierungen als rot (das ist gewollt: der Fortschritt von Phase 3 bis 5 macht sie grün).

---

## 6. Phase 1 – Stubs und Sofortschäden

### WP 1.1 – Engine-Stub und Feature `encryption-at-rest` löschen (L-12)

**Ist.** `engine/src/lib.rs:12–211`: Re-Export aus `crypto` unter dem Feature, sonst der Stub mit `verify → Ok(true)` (B-02). 14 `cfg`-Stellen in `store`, 13 in `engine`. `store/Cargo.toml`: `default = ["wal-integrity"]`, `encryption-at-rest`, `deletion-proof` (0 Verwendungen).

**Änderung.**

1. `engine/src/lib.rs`: den Stub-Block und den `cfg`-Re-Export löschen; `pub use contextra_crypto::deletion_proof::{...}` unbedingt. `contextra-crypto` wird nicht-optionale Abhängigkeit von `engine` (`Cargo.toml`: `optional` entfernen).
2. Alle 27 `cfg(feature = "encryption-at-rest")`-Stellen: der Zweig mit Feature wird unbedingt, `cfg(not(...))`-Zweige entfallen.
3. Features entfernen: `engine/encryption-at-rest`, `store/encryption-at-rest`, `store/deletion-proof`, `store/wal-integrity` (wird unbedingt; REQ-DUR-009: kein Modus ohne Integritätskette). Verweise in `db`, `contextra`, `mcp`, `py` und Workspace-Skripten (`justfile`, `.github/workflows`) bereinigen.
4. `TenantIsolatedKvStore` im Engine-Stub entfällt mit (Duplikat zu `crypto`/`kvcache`; wird mit WP 2.2 endgültig getilgt).

**Test (rot zuerst).** Neues Gate `no-security-stub` (Spec §17.3): synthetisches Repo mit `cfg(not(feature))`-Ersatz für einen `crypto`-Typ → rot; auf dem Repo vor WP 1.1 → rot, danach grün. Zusätzlich ein Test `engine/tests/proof_verify_not_constant.rs`: `DeletionProof::verify` mit falschem Schlüssel liefert `Err` oder `Ok(false)`.

**Akzeptanz.** `rg "no_crypto_stubs|encryption-at-rest" crates xtask .github justfile` ergibt 0 Treffer; `cargo check --workspace` grün.

### WP 1.2 – MCP: Literal-Schlüssel, Fallbacks, Dummy-Beweise (L-12)

**Ist.** B-04: `tools_crud.rs:127,193` (Tenant), `:206` (Literal), `:210` (Tx); `handle_delete` baut `LayerCleanupProof` mit der Konstante `0`; `proof_key_env.rs` liest `CONTEXTRA_DELETION_PROOF_KEY`/`CONTEXTRA_PROOF_KEY` aus der Umgebung.

**Änderung (Zwischenzustand, bis WP 7.4/8.3).**

1. `default_test_deletion_proof_key_32_bytes!` und der gesamte `unwrap_or_else`-Zweig entfallen. `proof_key.rs` und `proof_key_env.rs` werden gelöscht; der Berichtsschlüssel kommt später ausschließlich aus dem `KmsProvider` (INV-PROOF-KEY-1).
2. Mandant: `TenantId::try_new(1).unwrap_or(SYSTEM)` entfällt. Der Mandant wird aus dem aufrufenden Prinzipal über die Konfiguration abgeleitet; fehlt die Zuordnung, ist die Antwort `Denied`. Dasselbe in `server_tools.rs:449`.
3. Transaktion: `allocate_tx().unwrap_or(TxId::new(1))` wird `allocate_tx()?` mit Fehlerabbildung.
4. `contextra_delete` und `contextra_drop_collection` führen **nur die logische Löschung** aus und liefern `report: null`, `class: null`, `claims: []` und eine nicht leere Liste `excluded`; kein `DeletionProof` ohne Saga (INV-NO-DUMMY-PROOF-1). `contextra_forget` verhält sich gleich.

**Test (rot zuerst).** `mcp/tests/no_dummy_proof.rs`: Aufruf ohne Prinzipal → `Denied`; Aufruf mit Prinzipal → Antwort ohne `proof`; statische Prüfung `rg "default_test_deletion_proof_key|new_after_verified_empty"` in Produktionscode = 0 (Gate `no-literal-secrets`, REQ-KEY-010).

**Akzeptanz.** AC-26 teilweise (keine Dummy-Berichte); U-22, U-23, U-37 geschlossen.

### WP 1.3 – Panics und Test-Hooks aus der Produktions-API

**Änderung.**

- `store/src/wal/mod.rs:92` (`unwrap`) und `text/src/tokenizer.rs:30` (`panic!`, plus `#[allow(panic)]` in Zeile 20): typisierte Fehler; der Tokenizer-Konstruktor gibt `Result` zurück, der Aufrufer propagiert.
- Alle `*_for_test`-Zugänge (`store/src/lsm/engine.rs:119,125,131,254,259,264,269,274,279`, `store/src/wal/hmac.rs:187` `integrity_key_for_test`, `contextra-py` `_trigger_panic_for_test` in `collection.rs:52`, `db.rs:64`, `functions.rs:114`, `lib.rs:60`) wandern hinter `cfg(test)` bzw. ein Feature `test-hooks`, das in keinem Default- oder Release-Satz liegt. `integrity_key_for_test` gibt Schlüsselmaterial im Klartext zurück und verletzt INV-SCAN-1 unmittelbar.
- Tests, die die Hooks brauchen, aktivieren `test-hooks` über ein `dev-dependencies`-Selbstmuster (wie `crypto` es mit `test-utils` bereits tut).

**Test (rot zuerst).** Gate `public-api` (`cargo public-api`-Snapshot): verbotene Signaturmuster `*_for_test`, `-> [u8; 32]` aus Schlüsselmodulen.

**Akzeptanz.** Gates `test-surface` und `panic-surface` grün.

---

## 7. Phase 2 – Archivierung und Entkopplung

### WP 2.1 – Archiv-Tag und Kanten-Inventar

**Änderung.**

1. Maintainer setzen den Tag `archiv/v0` auf den Stand nach WP 1.x und pushen ihn.
2. `docs/spec/MIGRATION-EDGES.md` (generiert aus `cargo metadata`): je Archiv-Crate alle Dependents mit Datei und Symbol (`rg`-Treffer). Das ist die Abarbeitungsliste für 2.2/2.3.
3. Archiv-Crates werden aus `default-members` genommen (heute z. B. `graph`, `adapt`, `kvcache`, `cognition`, `agent`, `router` enthalten).

**Akzeptanz.** Tag existiert; Inventar vollständig; Build unverändert grün.

### WP 2.2 – `engine` entkoppeln

**Ist.** `engine` hängt hart an `graph`, `adapt`, `kvcache`, `infer-candle`, `rank` (und optional `sandbox`). Betroffene Module (Verzeichnisnamen gelesen, Inhalte nicht): `kv_cache_integration/`, `extraction/`, `background_workers/`, `collection/search/hybrid/`, `collection/maintenance.rs`, `collection/crud/{links,auto_extraction}.rs`, in `lib.rs` `CommunityDetectionConfig` (:347), `register_consolidation_launcher` (:302), `kv_eviction_attention_source` (:574).

**Änderung (je Kante ein Commit, Build grün nach jedem).**

1. **`kvcache`:** `kv_cache_integration/` löschen; `Collection::with_kv_store`/`set_kv_store`/`kv_store`, `start_kv_cache_eviction`, `TenantIsolatedKvStore`-Reste entfernen; Layer `KvCache` ist künftig `FeatureDisabled` (WP 6.4).
2. **`graph`:** `links.rs`, `stage_graph_entity_delete`, `check_and_trigger_community_detection`, `with_community_detection_trigger_threshold`, `CommunityDetectionConfig` und die PPR-Pfade im Hybrid-Planner entfernen. Der Planner lässt die Modalität aus und trägt `modalities_unavailable` im Ergebnis (§4.7). Das Port `GraphIndex` wird aus `ports` entfernt (WP 3.2).
3. **`adapt`:** `AdaptiveController`-Nutzung durch feste Startwerte aus §15 ersetzen (`efSearch` 64, `exact_scan_max` 20.000, Overfetch 3/6/12).
4. **`infer-candle`:** `engine` nutzt nur noch den Port `EmbeddingProvider`; das Backend injiziert die Fassade (WP 8.1). Pflicht-Abhängigkeit entfällt.
5. **`sandbox`:** nur noch Port `FilterModule`/`ModuleVerifier`; Feature `sandbox` am Crate entfällt.
6. **`rank`:** bleibt (nur `fusion/`); `engine` enthält keine zweite RRF-Implementierung (Kantenregel 10). Ob `engine` heute RRF selbst implementiert, ist **ungeprüft**; falls ja, wird sie in `rank/fusion` zusammengeführt.
7. **Reaper:** `reap_by_thermostat` und `trigger_reaper` (`collection/maintenance.rs:139,655`, beide laut Scan ohne Aufrufer) werden gelöscht. Ein thermostatgesteuertes Löschen widerspricht INV-RETENTION-1. Der Retention-Reaper nach §11.6 entsteht neu als `engine/src/reaper.rs`, `MemoryType` (Default `Semantic`) liegt in `types` (Entscheidung zur offenen Zuordnung aus §11.10).

**Test (rot zuerst).** `engine/tests/core_without_rand.rs` und das CI-Kommando `cargo build -p contextra --no-default-features --features embedded`; Gate `product-hull` zeigt nach 2.3 die Hülle (Ziel: 16 Kern-Crates).

**Akzeptanz.** `cargo metadata`: `engine` hat keine Kante zu `graph`, `adapt`, `kvcache`, `infer-*`, `sandbox`; Zeilen ≤ 10.200 (Zielbudget 8.500 nach Phase 7).

### WP 2.3 – `db`, `mcp`, `contextra` entkoppeln; `contextra-core` entfernen

**Änderung.**

- `db`: Kanten zu `cognition`, `graph`, `adapt` entfernen; `crypto`, `store`, `vector`, `text`, `checkpoint` nur, soweit §4.4 sie erlaubt (`db` darf nur `types`, `ports`, `engine`; die übrigen sind Stretch und werden in 8.2 durch Ports ersetzt – bis dahin als deklarierte Ausnahme in `capabilities.toml`). Features `volatile-vault`, `edge-reinforcement-learning`, `coherence-bonus-fusion`, `adaptive-candidate-pool-sizing`, `background-maintenance`, `bench` entfallen (tot laut Scan).
- `mcp`: Kanten zu `rank`, `adapt`, `privacy`, `license`, `crypto`, `infer-*` entfernen. Werkzeuge, die archivierte Crates brauchen (`contextra_relate`, `contextra_relate_n_ary`, `contextra_consolidate`, `contextra_cloud_query`, `contextra_plugin_status`), entfallen jetzt; der Rest wird in WP 8.3 durch die drei Werkzeuge ersetzt.
- `contextra-core`: `use contextra_core::{ContextraError, Result}` (u. a. `store/src/fsync.rs`) durch `contextra_types::{ContextraError, Result}` ersetzen (mechanisch per Skript, dann `cargo check`); Abhängigkeit in `checkpoint`, `engine` (dev), `py`, `simd`, `store`, `vector`, `contextra` entfernen; Crate löschen (D-05).
- `contextra`: Features `fast`, `docid-128`, `sovereign` (tot bzw. ohne Wirkung laut Scan) entfernen; `default = ["embedded"]`; `candle`, `privacy`, `sandbox`, `py`, `compliance` (= `privacy` + `audit-export` + `avv-generator`), `license` als `optional = true`; `license` und `infer-candle` sind **keine** Pflichtkanten mehr.

**Test (rot zuerst).** Gate `core-isolation` (INV-CORE-ISOLATION-1): jede Kante `[K] → [R]/[E]/[A]` ist rot; die aufgelösten Default-Features der Kern-Crates sind eingeschlossen.

**Akzeptanz.** `contextra-core` existiert nicht mehr; `product-hull` meldet ≤ 16 Kern-Crates ohne Tooling.

### WP 2.1b – Physisches Entfernen

Nach 2.3 gibt es keinen Dependent der Archiv-Crates mehr. Im selben PR (Schutzpfade beachten): Verzeichnisse löschen; `members`/`default-members` in `Cargo.toml`; `capabilities.toml`; `justfile` (47 Ziele), `.github/workflows` (32 Workflows), `xtask/registry.toml` (110 Befehle), `docs/TYPE_REGISTRY.md`, `mutants-guarantees.toml`, `deny.toml`; `Cargo.lock` neu erzeugen. Dev-Kanten `store → graph, sandbox` entfallen oder wandern in `tests/` des höheren Rings.

**Akzeptanz.** `cargo metadata`-Mitglieder: 16 Kern-Crates, 7 Rand/Experimentell, 4 Werkzeuge plus Fuzz-Crates; Gate `doc-truth` grün.

### WP 2.4 – Rand-Crates optional, Brücken

`infer-candle`, `privacy`, `sandbox`, `py`, `license`, `audit-export`, `avv-generator` bleiben, hängen aber nur an der Fassade (Features), nur von `types`/`ports` ab (Ring 2: Fundament) und sind aus `default-members` heraus. `privacy` (Ring 3) bindet die Ports `EgressChannel`, `EntityRecognizer`, `InjectionDetector`; ohne das Feature ist Egress gesperrt (INV-WIRING-2). Das Doppelt-Definierte (`AvvContext` in `avv-generator` und `privacy`, `EgressGuard` in `mcp` und `privacy`, `TenantId` in `privacy/src/processing_registry.rs:33`) wird bereinigt: Typen aus `types`, kein zweiter `TenantId`.

### WP 2.5 – `capabilities.toml`

**Änderung.** Schema 3: neues Feld `class` (`kern`/`rand`/`experimentell`) je Crate; `may_depend_on` strikt nach §4.4 (heute 66 Warnungen „nicht deklariert“, z. B. `mvcc → ports`, `mcp → types/ports/wire`, `checkpoint → types/ports`); `maturity` wird nicht behauptet, sondern aus der Evidenzstufe berechnet (§17.6; vorläufig alle Kern-Crates `experimental`, weil Sicherheitsports nur Null-/Testimplementierungen haben). Das Gate `ring-layering` behandelt Warnungen als Fehler.

**Akzeptanz.** `ring-layering` grün, 0 Warnungen.

### WP 2.6 – Unsafe-Inseln bereinigen

**Ist (Scan).** 14 `unsafe`-Stellen außerhalb der Inseln (nur Tests/Benches: `bench/benches/{competitive_kv,locomo,long_mem_eval}.rs`, `text/tests/alloc_profiler.rs`, `store/tests/recovery_fd_exhaustion_regression.rs:69`, `graph/tests/ppr_alloc_test.rs` entfällt mit dem Archiv); 49 Insel-Stellen ohne `// SAFETY:` (u. a. `simd/benches/simd_bench.rs:38,62,86`, `sys/src/{acl_win32.rs:77,mmap.rs:14,posix.rs:20}` und der Hauptteil in `wire/src/contextra_generated.rs`); sensitive `#[allow]` in `text/tokenizer.rs:20`, `simd/lib.rs:9`, `wire/lib.rs:12–15`.

**Änderung.**

1. SAFETY-Kommentare in `sys` und `simd` (inkl. Benches); `allow(undocumented_unsafe_blocks)` in `simd` entfernen.
2. `wire`: `contextra_generated.rs` erhält `@generated`-Kopf mit Generator und Version; die `allow(...)` gelten nur noch auf Dateiebene dieser Datei. Gate `unsafe-inventory` weist generiert und handgeschrieben getrennt aus (Spec §4.6 Punkt 6). `check-flatbuffers-drift` prüft die Reproduzierbarkeit (`flatc` fehlt in der Analyseumgebung).
3. Test-Allokatoren (`text`, `store`): lokales, begründetes `#[allow(unsafe_code)]` im Testtarget; Bench-`unsafe` entfernen oder in `sys` kapseln (was die drei Bench-Stellen tun, ist **ungeprüft**).
4. `#![forbid(unsafe_code)]` in jedem Nicht-Insel-Crate; Workspace `unsafe_code = "deny"`. Dateien dürfen nicht anhand ihres Namens (`test_*.rs`) von der Prüfung ausgenommen werden.

**Akzeptanz.** `unsafe-audit` grün; Miri-Lauf gegen die skalare Referenz für `simd` (Ziel Phase 9).

---

## 8. Phase 3 – Fundament

Phase 3 schafft die Typen, Ports und Primitive, auf denen Phasen 4–7 aufbauen. Sie ändert noch kein Persistenzformat. Ihr Ergebnis ist kompilierbar, ohne Produktionsverhalten zu ändern, außer dort, wo Phase 1 es bereits verlangt hat.

### WP 3.1 – Löschtypen in `types`

**Ist.** Löschtypen liegen dupliziert vor: `DeletionLayer` in `crypto/src/deletion_proof.rs:238` (sieben Varianten) und im Stub `engine/src/lib.rs` (drei Varianten), `DeletionProof` in drei Crates (B-06, Scan §10). `types/src/witness.rs:56–67` enthält `issue_from_scanner` als `pub` mit `#[doc(hidden)]` und `layer() -> &str` (B-05). Der Budgetstand ist knapp: Ist 2.691 (Scan) gegen Budget 2.700 (K-12).

**Änderung.**

1. Neues Modul `crates/contextra-types/src/erasure/` mit `ids.rs` (`ErasureId`, `SubjectRefHash`, `UnitRefHash`, `ReferenceEpoch`), `layer.rs` (`DeletionLayer` mit zehn Varianten aus §12.4; `Plane::{A, B}`), `class.rs` (`ReachedClass`: L, P, C*, C1, C2, C3, M), `assumption.rs` (Codes aus §12.5.2 als Enum), `scan_kind.rs` (`ScanMethod`, `WitnessKind`, `NotMeasuredReason`), `error.rs` (`ErasureError`, `ScanError`, `PurgeError`, mit Abbildung in `ContextraError`, A6).
2. `RequestContext` in `types/src/context.rs` mit `CancelToken` und `Deadline` (INV-CANCEL-1, §6 Regel 5).
3. Die Abbildung der sieben Alt-Varianten (`LsmMemtable`, `SsTableAllLevels`, `HnswIndex`, `WalAllSegments`, `CsrGraph`, `KvCacheSegments`, `EmbeddingCache`) auf die zehn Schichten steht in §21 der Gesamtspezifikation (U-41). Sie ist Teil dieses WP. Sie wird als Tabelle in `docs/spec/FORMATS.md` und als Testdatei festgehalten (`types/tests/layer_mapping.rs`).
4. `types/src/witness.rs` wird auf die Grundtypen reduziert. Die Struktur `LayerWitness` wandert nach `ports::scan` (Entscheidung E-W1, §17).
5. Alle Löschtypen außerhalb von `types` werden durch Re-Exporte ersetzt (D-02, Kantenregel 5). Die Duplikate in `crypto` und im Stub werden in WP 1.1 und WP 3.1 entfernt, nicht erst in WP 9.1.

**Test (rot zuerst).** `types/tests/layer_mapping.rs` (jede Alt-Variante hat genau eine Zielschicht; Plane stimmt mit §12.4 überein). Gate `deletion-types-once` (REQ-DEL-027) muss vor der Änderung rot sein und nach ihr grün.

**Akzeptanz.** `rg "pub struct DeletionLayer|pub enum DeletionLayer" crates` liefert genau einen Treffer. `types` bleibt im Budget oder der Budgetzuwachs ist per ADR beschlossen (E-B1).

### WP 3.2 – Sicherheitsports in `ports`

**Ist.** `KmsProvider` in `crypto/src/wal_crypto.rs:18` hat nur `get_key() -> Result<Vec<u8>>` und ist ohne Implementierung (UNWIRED, K-09). `Vfs`, `DurableLog`, `ErasureJournal`, `LayerScanner`, `Purger`, `EffaceableKeySlot`, `ErasedSetMirror`, `GroupRevocationListener` sind im Repo nicht als Trait vorhanden (Suche über `ports` und `types`: 0 Treffer, außer `GroupRevocationListener`, dessen Stand ungeprüft ist). `ports/src/lib.rs` enthält die Module `attention`, `graph`, `graph_index`, `kv_bridge_port`, `plugin`, `query_rewriter` u. a.; die Archiv-Ports werden in WP 2.3 entfernt.

**Änderung.** Neue Module in `crates/contextra-ports/src/erasure/`:

1. `kms.rs`: `KmsProvider` mit `custody() -> Custody`, `create_root(&self) -> Result<RootHandle>`, `derive(&self, root, purpose: KeyPurpose, ad) -> Result<Zeroizing<[u8;32]>>`, `sign_report(...)` für den Berichtsschlüssel (INV-PROOF-KEY-1). Der alte Trait wird als Adapter für Tests gehalten, bis WP 5.1 ihn ersetzt.
2. `vfs.rs`: `Vfs` (synchron, §6 Regel 2 und D-09). Methoden: `open(path, mode)`, `read_at`, `write_all_at`, `sync_file(FileSyncKind)`, `sync_dir(path)`, `rename_atomic(from, to)`, `remove(path)`, `lock_dir_exclusive(path) -> DirLock`, `metadata(path)`. Die Methodenauswahl folgt §7.7.
3. `durable_log.rs`: `DurableLog` mit `append(&self, rec: &[u8]) -> Result<Seq>` (fsync je Eintrag), `replay(&self, sink)`, `truncate_checked(seq)`.
4. `erasure_journal.rs`: `ErasureJournal` mit `append(&self, entry: JournalEntry) -> Result<()>` (fsync vor Wirkung, §7.9 Regel 3) und `replay(&self) -> Result<Vec<JournalEntry>>`.
5. `scanner.rs`: `LayerScanner` (`layer`, `begin_scan`, `scan_units`, `finish` als Default-Methode, §12.7.4) und `Purger` (§5.3). Die Typen `ScanSession`, `ControlToken` und `LayerWitness` liegen in `ports::scan` (E-W1). Die Konstruktoren sind `pub(crate)`. `begin_scan` ist eine Default-Methode des Traits, sodass Implementierer `ScanSession` ausschließlich über den Trait erhalten.
6. `effaceable.rs`: `EffaceableKeySlot` (§12.2.3). K2 und K3 liefern `CapabilityUnsupported` (D-10).
7. `erased_mirror.rs`: `ErasedSetMirror` (§12.17.3).
8. `recipient_ledger.rs`: `RecipientLedger` (kein Sicherheitsport, §5.2).
9. `governance/security_ports.toml` (neu, siehe WP 0.4): je Port `name`, `crate`, `trait_path`, `class`, `prod_impl`, `wiring_test`.

**Test (rot zuerst).** `ports/tests/erasure_surface.rs` (Kompilierungstest, der jeden Trait mit einem Dummy implementiert und die Signaturen festschreibt). Gate `security-ports` läuft mit der neuen Datei und muss die fehlenden produktiven Implementierungen als rot melden (WP 0.4). `cargo public-api`-Snapshot für `contextra-ports` wird neu erzeugt (REQ-PORT-4).

**Akzeptanz.** Alle Sicherheitsports sind in `security_ports.toml` geführt. Jeder hat eine Stelle im Gate, die auf die fehlende Produktionsimplementierung zeigt. Der Stand wird in `ports` budgetiert (E-B1).

### WP 3.3 – Durability-Primitive in `sys`

**Ist.** `store/src/fsync.rs` ist die einzige Sync-Stelle. Unter Windows liefert sie `ErrorKind::Unsupported` (Zeilen 22–26, 66–70; B-10). `F_FULLFSYNC`, `FlushFileBuffers` und `MoveFileEx(MOVEFILE_WRITE_THROUGH)` kommen im Repo nicht vor. `sys/src/` enthält `acl_win32.rs`, `mmap.rs`, `posix.rs` und weitere Module (Scan: `SAFETY`-Lücken in `acl_win32.rs:77`, `mmap.rs:14`, `posix.rs:20`).

**Änderung.**

1. `sys/src/durability/mod.rs` mit der Plattformtabelle aus §7.7 als `cfg`-Zweige: `posix_linux.rs` (`fdatasync`, `fsync` auf Metadaten-Änderung, Verzeichnis-`fsync`), `posix_macos.rs` (`fcntl(F_FULLFSYNC)`), `win32.rs` (`FlushFileBuffers`, `MoveFileExW` mit `MOVEFILE_WRITE_THROUGH`, `ReplaceFileW`).
2. Jede Datei dieses Moduls trägt `// SAFETY:` an jedem `unsafe`-Block (WP 2.6, §4.6).
3. Entscheidung D-11: Windows ist keine Garantieplattform. Die Primitive melden `platform_guarantee = false` über einen Typ `DurabilityGuarantee`, den der Bericht übernimmt. Der Fehler `Unsupported` verschwindet aus `store`.
4. `sys/src/mem_lock.rs` für `mlock` mit wahrheitsgemäßem Ergebnis (§12.18). `doctor` weist einen Fehlschlag aus.
5. Das Budget von `sys` steigt von 450 auf den tatsächlichen Wert; der Zuwachs braucht einen ADR (E-B1).

**Test (rot zuerst).** `sys/tests/durability_contract.rs`: auf Linux prüft ein Test, dass `sync_dir` nach `rename_atomic` aufgerufen wird (über einen Beobachter-Wrapper). Ein Test auf Windows prüft `platform_guarantee == false`. Die Plattformmatrix (`platform-matrix`) läuft mindestens unter Linux und macOS in CI.

**Akzeptanz.** `store` enthält keinen `std::fs::File::sync_all`-Aufruf mehr außerhalb von `Vfs`-Implementierungen (Gate `async-io` und `rg`). `sys` hat `unsafe`-Blöcke nur mit `SAFETY`.

### WP 3.4 – `testkit`: Anbindung an die Ports

**Ist.** `crates/contextra-testkit/src/fault_vfs.rs` und `manual_clock.rs` existieren (K-06). Ob `FaultVfs` das künftige `Vfs`-Trait implementiert, ist ungeprüft; das Trait fehlt noch.

**Änderung.**

1. `FaultVfs` implementiert `Vfs` aus WP 3.2 und erlaubt Fehler je Operation (`fsync`, `rename`, `write`, `ENOSPC`) und je Pfadmuster. Die Fehlerinjektion ist deterministisch (Zähler, kein Zufall).
2. `ManualClock` implementiert `Clock`.
3. Neu: `FakeKmsProvider` und `FakeEffaceableSlot` (K1-Verhalten mit festem Testmaterial). Beide liegen nur in `testkit` (Kantenregel 6) und werden über `dev-dependencies` eingebunden.
4. Mock-Typen im Produktionscode werden entfernt: `MockLlmGenerator` (`mcp/src/config.rs:384`), `MockEmbedder` (Kantenregel 6, REQ-PORT-3).

**Test (rot zuerst).** Gate `no-prod-mock` (neu): jeder Typ mit `Mock`, `Fake`, `Stub` oder `Null` im Namen außerhalb von `testkit` und `#[cfg(test)]` ist rot. Ausnahmen über `NullMetricsSink`, das nach REQ-PORT-3 als Null-Objekt zulässig bleibt, werden namentlich in `security_ports.toml` geführt.

**Akzeptanz.** Jeder Crash-Test der Phasen 5–7 nutzt `FaultVfs` statt Dateisystem-Tricks.

### WP 3.5 – Determinismus durchsetzen

**Ist (K-07).** 44 Zeitaufrufe nach Gate, davon 35 in aktiven Crates: `mvcc` 14 (u. a. `tx_buffer.rs:302,426,473` mit `Instant::now()`), `engine` 7, `store` 7, `ports` 3, `checkpoint` 2, `db` 1, `privacy` 1. Die übrigen 9 liegen in Archiv-Crates (`kvcache` 3, `router` 3, `sandbox` 2, `adapt` 1) und entfallen mit WP 2.1b. Zufall: `rand::thread_rng` in `store/src/wal/hmac.rs:246, 251, 367, 456` und `store/src/lsm/recovery.rs:31` (B-20). `determinism-allow.toml` erlaubt den Zugriff pauschal für `hmac.rs`, also auch für die Stellen 251, 367 und 456, die vermutlich Zähler oder Nonces erzeugen und keine Schlüssel.

**Änderung.**

1. `clippy.toml` im Workspace-Wurzelverzeichnis (neu, K-02): `disallowed-methods` für `std::time::Instant::now`, `std::time::SystemTime::now`, `rand::thread_rng`, `std::env::var`; `disallowed-types` für `HashMap`/`HashSet` mit `RandomState` in Kernlogik.
2. Zeitaufrufe in `mvcc`, `engine`, `store`, `checkpoint`, `db`, `privacy` über den `Clock`-Port. Die Konstruktoren nehmen `Arc<dyn Clock>` entgegen, der Composition Root injiziert die Systemuhr.
3. Die Systemuhr-Implementierung (`SystemClock`) bleibt als einzige Stelle erlaubt, in `determinism-allow.toml` mit Begründung und Pfad eingetragen. Ihr Standort wird in WP 8.1 festgelegt.
4. `hmac.rs:251,367,456`: Prüfung, ob die Werte Zähler, IDs oder Nonces sind. Zähler und IDs gehen über `IdGen`. Nonces bleiben beim System-RNG, werden aber über eine eigene Funktion `fresh_nonce()` mit Eintrag in `determinism-allow.toml` mit Zweck „Nonce“ erzeugt. Die Erlaubnisliste wird so eng wie möglich gefasst (Pfad und Funktionsname, nicht nur Datei).
5. `ports` (drei Zeitaufrufe): Es wird geprüft, ob diese Aufrufe in einer Trait-Implementierung stehen oder in Typdefinitionen. Standardimplementierungen zählen als Verstoß, es sei denn, sie sind die Systemuhr aus Punkt 3.

**Test (rot zuerst).** Gate `determinism` läuft mit `clippy.toml` und dem neuen Zähler (WP 0.2 legt die Zählregel fest). Jede Zählung wird in `governance/ratchet.toml` mit Baseline 44 bzw. 35 nach Archivierung eingetragen. Die Baseline darf nur sinken.

**Akzeptanz.** `rg "Instant::now|SystemTime::now|thread_rng" crates --type rust -l` zeigt nur noch die Erlaubnisliste. Ein Test prüft, dass eine Löschung mit gefälschter Uhr (`ManualClock`) reproduzierbare Zeitstempel im Bericht erzeugt.

---

## 9. Phase 4 – Store

Phase 4 führt die Datei-I/O über `Vfs` und `DurableLog` und stellt das Format-Register auf. Ihr Ergebnis ist die Voraussetzung für Phase 5, insbesondere für die Golden-Files (K-17).

### WP 4.1 – `Vfs`-Implementierung und Ende der Blockierstellen

**Ist.** Der Scan meldet 18 `async-io`-Verstöße (K-08). Im Auszug sind folgende Stellen für aktive Crates relevant: `store/src/wal/mod.rs:32,40,65` (`std::fs::read`, `write`, `set_permissions` in `async fn`), `store/src/wal/replay.rs:144,204` (`File::open` in `async fn`), `store/src/sstable/reader.rs:675` (`read_block_at_file` als `async fn` mit `Arc<std::fs::File>`), `vector/src/hnsw/core_rebuild.rs:70` (`std::thread::sleep` in `wait_for_rebuild_with_timeout_and_clock`), `mcp/src/bin/contextra-mcp-server.rs:62` (`std::fs::read` beim Start). Sieben weitere Stellen des Auszugs liegen in `infer-onnx` (3) und `vector/src/diskann/build.rs` (4), beides Archiv. Drei Stellen stehen nur in `--full`.

**Änderung.**

1. `store/src/fsync.rs` wird durch `store/src/vfs/std_vfs.rs` ersetzt: `StdVfs` implementiert `Vfs` über `sys::durability` (WP 3.3). Die Datei `fsync.rs` entfällt; alle Aufrufer nutzen `Vfs`.
2. `store/src/wal/mod.rs`: die Wrapper `read`, `write`, `set_permissions` werden synchron über `Vfs`. Der Aufruf aus `async`-Code läuft über `spawn_blocking` an der Grenze `StorageWrite` (§6 Regel 2).
3. `store/src/wal/replay.rs`: die `async`-Funktionen werden synchron; der Replay läuft im Blocking-Pool.
4. `store/src/sstable/reader.rs:675`: synchrone Funktion. Lesepfade sind nach §6 synchron (`StorageRead`).
5. `vector/src/hnsw/core_rebuild.rs:70`: `wait_for_rebuild_with_timeout_and_clock` wird synchron und nutzt den `Clock`-Port für die Wartezeit; `thread::sleep` wird durch eine Schleife mit `Clock::monotonic_nanos` und einem kurzen `Condvar`-Wait ersetzt.
6. `mcp/src/bin/contextra-mcp-server.rs:62`: Konfigurationsdatei über `tokio::fs` oder `spawn_blocking`.
7. `async fn` in Ring 0 (`vector` 52, `text` 43, D-09): kein Zuwachs. Der Abbau erfolgt nach der Entscheidung des Spikes S-ASYNC (Abschn. 17).

**Test (rot zuerst).** Gate `async-io` wird von der Zeilenheuristik auf eine AST-Prüfung umgestellt (Crate `syn`, REQ-ASYNC-2). Synthetisches Repo: `async fn` mit `std::fs::read` → rot. Der Test `store/tests/no_blocking_in_async.rs` wird vor der Änderung rot.

**Akzeptanz.** `async-io` hat 0 Verstöße in aktiven Crates. Das Gate `ratchet` führt die `async fn`-Zahl in `store` als Obergrenze.

### WP 4.2 – `DurableLog` und Sidecar-Rahmen

**Ist.** Sidecar-Logs existieren nur als Module in `crypto` (`audit_chain.rs`, `revocation_log.rs`), ohne Persistenz im Produktionspfad (B-09). Der Rahmen mit Magic, Version, Länge, Prüfsumme und HMAC-Kette fehlt.

**Änderung.**

1. `store/src/sidecar/frame.rs`: Rahmen mit Magic (`MFKR`, `MFRV`, `MFAU`, `MFEJ`, `MFES`), Formatversion (u16), Flags, Länge, Prüfsumme und HMAC-Kettenglied (§7.9 Regel 1). Der Schlüssel `K_sc/*` kommt über `KmsProvider::derive` (WP 5.1).
2. `store/src/sidecar/log.rs`: `DurableLog`-Implementierung mit `fsync` je Eintrag über `Vfs` (Fehler propagieren, REQ-FGT-4).
3. `store/src/sidecar/manifest_mark.rs`: Manifest-Marke (Name, letzte Sequenz, Kettenkopf), aktualisiert bei Checkpoint und Rotation (§7.9 Regel 1). Erkennt Tail-Truncation und Ersetzen durch ältere Version.
4. Recovery-Regeln aus §7.9 Regel 4: beschädigtes Sidecar in der Kettenmitte verweigert den Start; Löschen eines Sidecars bei vorhandenen Daten gilt als Manipulation.

**Test (rot zuerst).** `store/tests/sidecar_chain.rs`: Tail-Abschneiden wird erkannt; Korruption in der Mitte bricht fail-closed ab; Ersetzen durch ältere Marke wird erkannt. Fuzz-Ziel `sidecar_decode` (xtask-heavy, §17.1).

**Akzeptanz.** Alle fünf Sidecar-Magics haben einen Lese- und Schreibpfad mit Golden-File.

### WP 4.3 – Format-Register, Versionierung, Golden-Files, Migrationskern

**Ist.** Die Formate sind in `docs/spec/FORMATS.md` nicht registriert (die Datei fehlt). Im Code liegen vor: WAL `MFW2` und `MFW3` (`wal/encode.rs:55–58`; K-15), SSTable `MFSX` und Legacy `MFST` (`sstable/block_cache.rs:340–341`, Lesepfad `reader.rs:328–423`), Manifest `MFMN` (`manifest/core.rs:17`). Der Gate `format-corpus` existiert, die Registerprüfung `format-registry` ist ungeprüft (§17.3).

**Änderung.**

1. `docs/spec/FORMATS.md` (neu) führt jedes Artefakt aus §7.11 mit Magic, Version, Prüfsumme und Ist-Befund. Die Tabelle wird aus dem Code erzeugt, nicht von Hand gepflegt, soweit möglich.
2. Golden-Files: aus dem **Ist-Format** werden je Format und Version Fixture-Dateien erzeugt und in `crates/contextra-store/tests/golden/` abgelegt (REQ-FMT-3). Dies muss vor WP 5.3 geschehen (K-17).
3. `contextra migrate`-Kern als Bibliotheksfunktion in `store/src/migrate/` (offline, `--dry-run`, verifiziertes Backup vor jeder Änderung; REQ-FMT-6). Die CLI selbst folgt in WP 8.4.
4. Entfernen von `MFST` (D-08): Lesepfad `reader.rs:328–423` wird durch den Migrationspfad ersetzt. Vor der Entfernung muss ein Test nachweisen, dass ein `MFST`-Golden-File durch `migrate` in `MFSX` überführt wird.
5. `MFW2` wird ebenfalls über `migrate` in das aktuelle WAL-Format überführt. Der direkte Lesepfad bleibt bis zum Abschluss der Migration erhalten.
6. Downgrade-Schutz (INV-FORMAT-1): neuere Versionen als unterstützt verweigern das Öffnen. Die Manifest-Prüfung dafür liegt in `manifest/core.rs`.

**Test (rot zuerst).** `store/tests/format_corpus_roundtrip.rs` liest jedes Golden-File mit dem aktuellen Code. `store/tests/migrate_mfst.rs` migriert ein `MFST`-Golden-File und vergleicht die Schlüssel-Wert-Menge. Gate `format-registry` (neu) prüft, dass jede Datei im Datenverzeichnis einen Eintrag hat (§7.11).

**Akzeptanz.** `docs/spec/FORMATS.md` existiert, `MFST`-Lesepfad ist entfernt, Golden-Files für `MFW2`, `MFW3`, `MFSX`, `MFMN` liegen im Repo und werden grün gelesen.

### WP 4.4 – Betriebszustände, Notreserve, Poison

**Ist.** Es gibt keinen Zustandsautomaten mit `Healthy`, `Degraded`, `ReadOnly`, `Poisoned` (§7.8). Der Notreserve-Mechanismus fehlt. `wal/mod.rs:92` enthält einen `unwrap` (U-27; Scan nennt `wal/mod.rs:92`, Spec nennt `hmac.rs:194`, beides ist im Code zu prüfen).

**Änderung.**

1. `store/src/health.rs`: Enum `Health` mit den vier Zuständen. Übergänge monoton, außer Neustart und `Degraded → Healthy` (§7.8 Regel 1). Jeder Übergang wird auditiert, ohne Nutzinhalte (Regel 2).
2. `store/src/reserve.rs`: Notreserve (`emergency_reserve_bytes`, Startwert 64 MiB, mindestens zwei WAL-Segmente plus Journal), bei Start vorab reserviert (§7.10).
3. Poison-Lock: typisierter Fehler mit definiertem Übergang (§7.8 Regel 3). `wal/mod.rs:92` und `text/tokenizer.rs:30` (WP 1.3) werden entsprechend umgestellt.
4. Löschpfad-Regeln aus §7.10 (welche Handles im Zustand `Poisoned` oder `ReadOnly` für Löschen erlaubt sind) werden als Funktion `erasure_allowed(health, handle)` implementiert.

**Test (rot zuerst).** `store/tests/health_transitions.rs`: jede erlaubte und jede verbotene Übergangskante; `ENOSPC` bei Rotation führt zu `ReadOnly` und erlaubt den Tombstone-Batch über die Notreserve; `fsync`-Fehler am WAL führt zu `Poisoned`.

**Akzeptanz.** `rg "unwrap\(\)|expect\(" crates/contextra-store/src --type rust` zeigt keine Produktionsstelle im WAL-Pfad.

---

## 10. Phase 5 – Schlüssel

Phase 5 ersetzt den Einzelschlüssel durch die Hierarchie aus §12.2.2. Sie ist der größte Formatwechsel. Bestandsdaten laufen über `migrate` (D-07).

### WP 5.1 – `KmsProvider` K0/K1, Keystore `MFKS`, Argon2id

**Ist.** `KeyManager::try_new(passphrase, salt)` in `crypto/src/crypto.rs:71–134` leitet den Schlüssel per HKDF direkt aus der Passphrase ab (INV-KDF-1 verletzt, B-07). `try_new_with_kdf` mit `KdfHeader` existiert, wird im Store-Pfad aber nicht benutzt (`store/src/lsm/recovery.rs:170–200`, `store/src/lsm/engine.rs:365–385`). Die Datei `SALT` ist zufällig.

**Änderung.**

1. `crypto/src/kms/k1.rs`: `KmsProvider` für K1. Argon2id mit m ≥ 64 MiB, t ≥ 3 und mindestens 250 ms Laufzeit. Die Parameter werden bei der Einrichtung kalibriert und im `KdfHeader` (Kennung `MFKD`) gespeichert (§12.2.3).
2. `crypto/src/kms/k0.rs`: Betriebsart K0 (keine Verschlüsselung). Sie ist nur über einen expliziten Konfigurationsschalter `DevPlaintext` aktivierbar. `doctor` meldet sie rot (D-03).
3. `store/src/keystore/file.rs`: Keystore `MFKS` mit gewrappten Wurzelschlüsseln und `KdfHeader`. Atomares Ersetzen über `Vfs` (REQ-FMT-5).
4. Entfernen des HKDF-Pfads aus Passphrase (`KeyManager::try_new`) aus Produktionscode. Eine Testvariante bleibt hinter `cfg(test)`.
5. B2 (Boot): fehlt der `KmsProvider`, verweigert der Start (REQ-PORT-2, §13.5).
6. Testprofile: Argon2id mit 250 ms macht Tests langsam. Ein Testprofil mit geringen Parametern gibt es nur hinter `cfg(test)` oder `test-hooks` (K-06, Kantenregel 6), nie in Release-Builds.

**Test (rot zuerst).** `crypto/tests/kdf_argon2_only.rs`: ein Aufruf mit Passphrase ohne `KdfHeader` wird verweigert. `store/tests/keystore_mfks_roundtrip.rs`: Schreiben, Lesen, Ersetzen bei Fehlern (`rename` schlägt fehl → alter Keystore bleibt gültig).

**Akzeptanz.** `rg "try_new\(" crates --type rust` zeigt keinen Produktionsaufruf mit Passphrase und HKDF. Gate `key-purposes` läuft mit `governance/key_purposes.toml`.

### WP 5.2 – KeyTable `MFKT`, `KeyCache`, Hierarchie

**Änderung.**

1. `crypto/src/keytable/`: reine Logik (Wrap, Re-Wrap, Generationen), I/O-frei und lock-frei (§4.2). Die Datei selbst liegt in `store/src/keytable/file.rs` mit Ersetzen-Semantik (§12.2.9).
2. Hierarchie aus §12.2.2: `KEK[coll, e]` gewrappt unter der Wurzel; `DocDek[einheit]`, `ArtKey[artefakt]`, `K_ref[tenant, ref_epoche]` gewrappt unter `KEK`; `K_eng` und `K_sc/*` per HKDF aus der Wurzel bzw. aus `K_eng`.
3. `crypto/src/keycache.rs`: `KeyCache` mit `Zeroizing<[u8;32]>`, Invalidierung und Nullung bei D2 (H-04, §12.2.5 Regel 6).
4. `governance/key_purposes.toml` (neu, K-02): je Zweck ein Label `contextra/v1/<zweck>` (`eng`, `wal`, `sc/keyreg`, `sc/revoke`, `sc/audit`, `sc/journal`, `sc/erased`, `keystore-root`, `kek`, `docdek`, `artkey`, `kref`, `report-sign`, `vault`) und die Wirkung der Vernichtung.
5. `AD` für Wrap: `key_id ‖ kek_epoch` (§12.2.6).

**Test (rot zuerst).** Eigenschaftstests: Wrap/Unwrap-Rundlauf; falsches `AD` schlägt fehl; `KeyCache`-Invalidierung nullt den Speicher (über einen Drop-Beobachter mit `zeroize`). Gate `key-purposes`: jedes Label einmalig, jeder Code-Zweck im Register.

**Akzeptanz.** Jeder Schlüssel hat genau einen Zweck (INV-KEYPURPOSE-1).

### WP 5.3 – Record-Umschlag, Padmé, Header-Verschlüsselung, Blob-Segmente

**Ist.** WAL-Records laufen über `wal_crypto.rs` (ein DEK aus `KmsProvider::get_key`, K-09). SSTable-Blöcke sind AEAD unter dem Engine-Schlüssel. Blob-Segmente fehlen völlig: 0 Treffer für `MFBL`, `blob_threshold` und `BlobSegment` (B-22). Die Schicht `BlobSegments` (S1b) ist ein Neubau.

**Änderung.**

1. Record-Layout (D-07): je Record AEAD AES-256-GCM-SIV mit zufälliger 96-Bit-Nonce, Tag 128 Bit, `AD = scope_id ‖ doc_key_id ‖ record_version` (§12.2.6). Formatversion wird erhöht; Magic bleibt.
2. `doc_key_id` und die Sequenzangabe stehen im unter `K_eng` verschlüsselten Blockbereich, nicht im Klartext (REQ-DEL-N15).
3. `crypto/src/padme.rs`: Padmé-Auffüllung als reine Funktion. Mindestlänge `min_record_bytes` = 256. Eigenschaftstest: Überhang höchstens 12 %.
4. `store/src/blob/`: `segment.rs` (Magic `MFBL`, Kopf mit Digest), `writer.rs`, `pointer.rs`. Schwelle `blob_threshold` = 64 KiB. Reihenfolge: Segment durabel flushen, WAL-Commit mit Zeiger und BLAKE3-Digest, dann Ack (§7.1). Verwaiste Segmente räumt Recovery über das Manifest ab.
5. SSTable-Werte je Record unter dem DocDek der Einheit; äußere Blockschicht unter `K_eng` (§12.2.5 Regel 2).
6. `LegacyUnkeyed` (§12.2.5 Regel 7): Bestand ohne DocDek erreicht höchstens Klasse P, bis `migrate --rekey-units` läuft.
7. `migrate` erweitert um `--rekey-units` (WP 4.3-Kern).

**Test (rot zuerst).** `store/tests/layout_v4_golden.rs`: Golden-File der vorherigen Version wird durch `migrate` überführt und danach gelesen. `store/tests/destroy_leaves_others_readable.rs` (LA-4): Vernichtung eines DocDek lässt Records anderer Einheiten im selben Block lesbar. `crypto/tests/padme_bound.rs`: Überhang ≤ 12 % über einen großen Eingabebereich.

**Akzeptanz.** Der Wechsel ist durch `migrate` abgedeckt (REQ-FMT-7). Ein Testdatenbestand aus Fassung 1 kann mit Dry-Run geprüft und migriert werden.

**Risiko.** Dies ist der größte Umbau der Phase. Er betrifft WAL, SSTable und den neuen Blob-Pfad gleichzeitig. Die Reihenfolge wird als Pflicht festgelegt: erst Blob-Segmente mit altem Umschlag (ohne Verschlüsselung pro Einheit), dann Umschlag. So bleiben die Schritte einzeln testbar.

### WP 5.4 – Destroy-Protokoll D1–D6

**Änderung.**

1. `crypto/src/destroy/plan.rs`: Zustandsmaschine D1 bis D6 (§12.2.7) als reine Logik. Jeder Schritt liefert eine Aktion für den Aufrufer.
2. `crypto/src/destroy/attest.rs`: `DestroyAttestation` mit Custody, Verfahren (Zeroization), Zeitpunkt aus `Clock`, Beleg der Implementierung. Nur K1 liefert einen lokalen Beleg (Selbstauskunft).
3. `store` führt die Dateischritte aus: KeyTable-Ersetzen (D3), Entfernen alter Generationen (D4) über `Vfs::remove` und `sync_dir`.
4. `ErasureJournal`-Intent vor jedem Schritt (§7.9 Regel 3). D3-Fehler: Einheit bleibt `Frozen`, kein „vernichtet“-Bericht.
5. Notreserve für D3 bei `ENOSPC` (§7.10).

**Test (rot zuerst).** `engine/tests/destroy_crash_matrix.rs`: Absturzinjektion an jedem Schritt D1 bis D6 mit `FaultVfs`. Recovery wiederholt D2 bis D6 idempotent. Nach D4 findet der Nadelscan (WP 6.6) keinen Wrap-Blob im Ist-Format.

**Akzeptanz.** Ein Bericht trägt `key_destroyed` erst nach D6 (§12.2.7 Regel 1).

### WP 5.5 – Epoch Seal

**Änderung.**

1. `crypto/src/epoch.rs`: Planung des Re-Wraps (Liste aller lebenden DocDeks, ArtKeys und `K_ref` unter `KEK[e+1]`). Reine Funktion.
2. `store`: atomares Neuschreiben der KeyTable nach dem Re-Wrap (REQ-FMT-5). Vernichtung von `KEK[e]` nach dem Destroy-Protokoll.
3. Auslöser: `kek_epoch_len` (30 Tage), `seal-now`, automatisch nach `seal_after_deletion_max` (7 Tage; im Regulated-Tier höchstens 72 Stunden, H-07).
4. Crash-Sicherheit: `KEK[e]` und `KEK[e+1]` nebeneinander; Fortsetzung in B7.
5. K2 und K3: `EffaceableKeySlot::rotate` und `destroy` liefern `CapabilityUnsupported` (D-10).
6. Klasse-Wirkung (INV-EPOCH-SEAL-1): bis zum Seal ist die Klasse `C*` mit `R-KEKEPOCH until`.

**Test (rot zuerst).** `engine/tests/epoch_seal_crash.rs`: Absturz mitten im Re-Wrap; nach Neustart ist der Seal vollständig und idempotent. `crypto/tests/epoch_plan.rs`: die Planung enthält jeden lebenden Schlüssel genau einmal.

**Akzeptanz.** Ein Bericht mit `C1` ist erst nach einem Seal möglich, der die Löschung umfasst.

### WP 5.6 – Sidecars produktiv, Altmodell streichen

**Ist.** `AuditChain` (`crypto/src/audit_chain.rs`) und `RevocationLog` (`crypto/src/revocation_log.rs`) werden außerhalb von `crypto` nur in Tests genutzt (B-09). Beide hängen an `kv_shredding::KeyRegistry` (K-10). `store/src/kv/` mit `segment.rs` und `delete_mode.rs` nutzt das alte Modell.

**Änderung.**

1. `KeyRegistry` neu als Sidecar `MFKR` (`store/src/sidecar/keyreg.rs`, auf WP 4.2 aufbauend): Ereignisse `created`, `wrapped`, `rotated`, `destroyed` mit Referenz-Hash, ohne Schlüsselmaterial und ohne Wrap-Blob (REQ-DEL-N5).
2. `AuditChain` (`MFAU`) und `RevocationLog` (`MFRV`) auf den Sidecar-Rahmen umstellen. Kettenschlüssel `K_sc/audit` und `K_sc/revoke` aus `KmsProvider::derive`.
3. `RevocationLog` wird bei **jeder** Autorisierung konsultiert, fail-closed (§12.6). Verdrahtung in der MCP-Pipeline Stufe 3 (WP 8.3) und in der Engine.
4. Streichung: `crypto/src/kv_shredding.rs`, `kv_segment` und `kv_cipher` (Scan-Auszug: `kv_segment/segment.rs:18,35`, `kv_segment/store.rs:17,108`). `store/src/kv/` entfällt, sofern es außerhalb des Archivs nicht gebraucht wird (E-K1, Abschn. 17).
5. Verweise in `contextra-crypto/src/lib.rs:31,56` werden entfernt.

**Test (rot zuerst).** `crypto/tests/revocation_fail_closed.rs`: Autorisierung mit Eintrag im RevocationLog wird verweigert. Gate `wired` (INV-WIRING-2): der RevocationLog hat einen produktiven Aufrufer.

**Akzeptanz.** `rg "kv_shredding|kv_segment|kv_cipher" crates` liefert 0 Treffer außerhalb des Archivs.

---

## 11. Phase 6 – Scanner und Purger

Jede Schicht mit Nutzdaten bekommt einen Scanner (Messung) und einen Purger (Bereinigung). Ein Scanner-Ergebnis mit `remaining > 0` erzeugt einen Bericht ohne höheren Klassenanspruch (REQ-FGT-5). Kein Scanner darf eine Konstante liefern.

### WP 6.1 – `LsmScanner`, Purger, Purge-Kompaktion

**Ist.** Der Scanner fehlt. `compaction/engine.rs:712–720` verwirft Tombstones nur bei Vollkompaktion mit `raw_seq < min_snapshot_seq` (§7.4). Eine bereichsweise Kompaktion `compact_range(scope)` existiert nicht.

**Änderung.**

1. `store/src/scan/lsm.rs`: `LsmScanner` über Memtable (Bytes), WAL-Segmente, SSTables und Blob-Segmente (S1 und S1b, §12.4). Zwei Sichten: Rohbyte-Suche über alle Dateien des Datenverzeichnisses und dekodierte Sicht mit allen aktiven Schlüsseln (§12.7.2).
2. `store/src/compaction/range.rs`: `compact_range(scope)` für betroffene SSTables. Tombstones dürfen nur bei Vollkompaktion oder nach Snapshot-Horizont fallen.
3. WAL-Scrub: Segmente ohne Scope neu schreiben, alte entfernen, Verzeichnis-`fsync`.
4. Memtable-Beleg erst nach `force_flush` (§12.4, S1).
5. Positivkontrolle (AC-1): jeder Lauf findet einen absichtlich nicht gelöschten Kontroll-Datensatz; ein Lauf ohne Kontrolle ist ungültig (§12.7.1 Regel 6).

**Test (rot zuerst).** `store/tests/lsm_scan_needle.rs`: ein kanonischer Kanarienwert wird eingefügt, gelöscht, bereinigt; danach 0 Treffer in allen Dateien (S-E1: 2.000 → 0). `store/tests/compact_range_scope.rs`: Kompaktion verändert keine Einheit außerhalb des Scopes.

**Akzeptanz.** `LsmScanner` liefert `ResidualReport` nur über `ScanSession` (WP 6.8).

### WP 6.2 – `VectorScanner`, Slot-Nullung, Generationswechsel

**Ist.** `vector/src/hnsw/arena.rs:60` (`free_list`), `:121–134` (Wiederverwendung von Slots mit ausreichender Kapazität), `:296–300` (Freigabe). Die Arena nullt beim Freigeben nichts; in `vector/src/hnsw` gibt es keinen `zeroize`-Aufruf (B-12). Das verletzt INV-HNSW-SLOT-1 (Slot-Nummern werden bis zum Neuaufbau nicht wiederverwendet).

**Änderung.**

1. Freigabe nullt Payload fp32 und SQ8 (`zeroize`) und markiert den Slot als tot. Die Slot-Nummer bleibt belegt.
2. `free_list`-Wiederverwendung (`arena.rs:121–134`) entfällt. Wiederverwendung ist erst nach dem Neuaufbau einer Generation zulässig (§12.16).
3. Neuaufbau: Generation g+1 ohne gelöschte Dokumente, atomarer Umschalt-Commit, Vernichtung des ArtKey der Generation g. Auslöser: `tombstone_rebuild_ratio` (0,2), `purge_rebuild_max_days` (14), `seal-now`.
4. Kein Rewiring (VETO-F02). Die Stellen `vector/src/hnsw/core_rebuild.rs:614–618` und `hnsw/types.rs:411–412` werden in WP 0.3 auf Rewiring geprüft und dann bestätigt oder geändert.
5. `vector/src/scan.rs`: `VectorScanner` über Slots, Snapshot und Quantisierungscodes.
6. Snapshot-Payload unter DocDek, Graph unter ArtKey (§12.2.8).

**Test (rot zuerst).** `vector/tests/slot_zeroed_after_delete.rs` (fp32 und SQ8 nach Löschung 0). `vector/tests/no_slot_reuse_before_rebuild.rs`. Loom-Test für den Umschalt-Commit, falls die Arena nebenläufig genutzt wird (ungeprüft).

**Akzeptanz.** Der Nadelscan über Slots findet keine Payload gelöschter Einheiten. Der Graph enthält nur `R-GRAPH` bis zum Neuaufbau und diesen Zustand meldet der Bericht.

### WP 6.3 – `TextScanner`, ArtKey-Segmente

**Ist.** Der Textindex liegt als Präfix-Schlüssel `__txt:<collection>:` im `StorageEngine` (B-13; `engine/src/contextra_impl/collections.rs:296–304`). Ob Terme gelöschter Dokumente aus dem Wörterbuch verschwinden, ist ungeprüft. `text/src/tokenizer.rs:30` enthält einen `panic!` (WP 1.3 entfernt ihn).

**Änderung.**

1. Textindex als Segmente unter je einem ArtKey (§12.2.8): Postings, Termwörterbuch, Längen. Neuschreiben erzeugt neuen ArtKey, der alte wird sofort vernichtet (INV-ARTKEY-1).
2. Terme nur gelöschter Dokumente werden aus dem Wörterbuch entfernt; IDF, Dokumentlängen und Zähler werden neu berechnet (§12.4, S3).
3. `text/src/scan.rs`: `TextScanner` über Postings und Termliste.
4. Migration des `__txt:`-Bestands in Segmente ist Teil von `migrate` (WP 4.3-Kern).

**Test (rot zuerst).** `text/tests/term_removed_after_delete.rs`: ein eindeutiger Term verschwindet aus dem Wörterbuch und aus der Rohdatei. `text/tests/idf_recomputed.rs`.

**Akzeptanz.** Der Nadelscan über Textsegmente findet keinen Term eines gelöschten Dokuments, der nur dort vorkommt.

### WP 6.4 – `CacheScanner`

**Ist.** Der Block-Cache liegt in `store` (SIEVE, §7.6). Antwort-, Embedding- und RRF-Caches in `engine` und `privacy`. Der KV-Cache aus `kvcache` ist Archiv und wird `FeatureDisabled` (§12.4).

**Änderung.**

1. Jeder Cache trägt den Scope im Schlüssel, damit `CacheScanner` nach Scope enumerieren kann.
2. Invalidierung und Nullung bei Löschung (S4). Antwort-Caches sind Pflicht (REQ-DEL-N7).
3. Surrogat-Vault in `privacy` (Rand): wird nur gemessen, wenn das Feature aktiv ist; sonst `FeatureDisabled` nach dem Start-Check.

**Test (rot zuerst).** `engine/tests/answer_cache_scope.rs`: eine Antwort aus dem Cache enthält keinen Inhalt einer gelöschten Einheit nach der Saga.

**Akzeptanz.** `CacheScanner` liefert `ResidualReport` mit `units_scanned > 0`.

### WP 6.5 – `CheckpointScanner`, Policy

**Ist.** `checkpoint/src/hardlink_cloner.rs` ist nur für Loom als Shim implementiert (K-04). Der Produktionspfad nutzt `tokio::fs`. Es gibt keinen Scanner und keine Policy. Hardlink-Klone derselben SSTables liegen im Datenverzeichnis unter Kontrolle von Contextra (S6).

**Änderung.**

1. `checkpoint/src/scan.rs`: `CheckpointScanner` über alle Checkpoints und Pins.
2. Policy je Collection (§12.5.4): `Rewrite` (Standard), `ExpireNow`, `ExpireAfterHold` (nur unter Legal Hold, höchstens `checkpoint_hold`). Pins zählen zur Schicht S6.
3. `Partial{outstanding: Checkpoints}` bis zum Rewrite, mit Anzahl, ältestem Alter und Ablaufdatum.
4. Der Loom-Shim wird in ein `cfg(loom)`-Testmodul verschoben (K-04).
5. Entfernen der Abhängigkeit `checkpoint → contextra-core` in WP 2.3.

**Test (rot zuerst).** `checkpoint/tests/rewrite_policy.rs`: nach Rewrite enthält kein Checkpoint einen Chiffrat-Block mit vernichtetem Schlüssel lesbar.

**Akzeptanz.** Die Schicht S6 hat einen Witness mit `CheckpointScanner`.

### WP 6.6 – `KeyScanner`, Nadelscan, Inventar

**Änderung.**

1. Pre-Destruction-Inventar je `key_id`: Liste der Chiffratblöcke (Datei, Offset, Länge), als Digest und Zähler (§12.7.3 Nr. 1). Blöcke ohne Zuordnung sind ein Fehler (fail-closed).
2. Nadelscan: Suche nach der Wrap-Nonce des vernichteten Schlüssels in KeyTable, Keystore, Sidecars, Checkpoints, Temp und `.bak` (§12.7.3 Nr. 2). Ergebnis: null Treffer. Die Nadel wird nach Abschluss aus dem Journal gelöscht und nur als Digest geführt.
3. Kanarien-Probe: ein Kanarien-Datensatz mit eigenem DocDek wird über denselben Vernichtungspfad (D1 bis D6) vernichtet; danach muss die Entschlüsselung scheitern (§12.7.3 Nr. 3).

**Test (rot zuerst).** `crypto/tests/needle_scan.rs`: eine vernichtete Nonce, die in einer Testdatei liegt, wird gefunden (Positivkontrolle).

**Akzeptanz.** Der Bericht enthält Nadelscan und Inventar für jeden Schlüssel, der in der Löschung vorkam.

### WP 6.7 – Lineage- und Sidecar-Scanner

**Ist.** Die Lineage-Kanten liegen heute im Graph (`graph/src/cascade.rs:84` enthält einen zweiten `DeletionProof`-Typ; B-06). Der Graph ist Archiv. Die Schicht S5 (`AuditLineage`) verliert damit ihren Quellcode.

**Entscheidung E-L1 (Abschn. 17).** Es wird ein kleiner `LineageScanner`-Port in `ports` definiert. Die Standardimplementierung deckt Sidecars ab. Lineage-Kanten werden, falls der Graph archiviert ist, nur dann gemessen, wenn Daten vorhanden sind; sonst meldet der Start-Check (B3) `FeatureDisabled`.

**Änderung.** `engine/src/lineage/scan.rs` mit `LineageScanner` über Sidecars (`MFAU`, `MFRV`, `MFKR`) und über Referenz-Hashes. Ableitungen mit weiteren Quellen werden neu abgeleitet oder als `stale` markiert (INV-DERIVED-1).

**Test (rot zuerst).** `engine/tests/lineage_cascade.rs`: eine abgeleitete Notiz mit einer gelöschten Quelle wird neu abgeleitet oder als `stale` markiert und im Bericht als eigener Eintrag geführt.

### WP 6.8 – `ScanSession`, `ControlToken`, Mint-Härtung

**Ist.** `types/src/witness.rs:56–67`: `issue_from_scanner` ist `pub` und nur `#[doc(hidden)]` geschützt (B-05, U-40). `LayerCleanupProof::new_after_verified_empty(…, usize)` (`crypto/src/deletion_proof.rs:183–228`) nimmt eine Zahl vom Aufrufer (B-03). `tools_crud.rs:198,200` und `contextra-db/examples/verify_deletion_proof_external.rs:112–114` rufen sie mit Konstanten `0` auf.

**Änderung.**

1. `ports::scan`: `ControlToken` (Positivkontrolle bestanden), `ScanSession` (nur über `LayerScanner::begin_scan`), `LayerWitness` (Felder privat, `pub(crate)`-Konstruktor, nur in `ScanSession::finish`).
2. Entfernen von `issue_from_scanner` und `new_after_verified_empty`. Ersetzung: `LayerScanner::scan` mit Default-Implementierung (`begin_scan`, `scan_units`, `finish`).
3. `tools_crud.rs` und das Beispiel in `contextra-db` verlieren die Konstanten. Die Aufrufe entfallen, bis der Orchestrator in Phase 7 verdrahtet ist (WP 1.2 Schritt 4).
4. Gate `witness-mint` (§17.3): jede Konstruktion von `LayerWitness` außerhalb von `ports::scan` ist rot, auch in Tests des Produktionscodes. Ausnahme: `testkit`.

**Test (rot zuerst).** `ports/tests/ui/` mit `trybuild`: externe Konstruktion von `LayerWitness` und `ScanSession` schlägt zur Kompilierung fehl. `engine/tests/review_stub_proof.rs` (Regression, §12.7.5): Bericht mit Restbestand → `Err`.

**Akzeptanz.** `rg "new_after_verified_empty|issue_from_scanner" crates` liefert 0 Treffer.

---

## 12. Phase 7 – Orchestrator

Der Orchestrator liegt in `engine` (Ring 3, §4.3). Er verdrahtet Scanner und Purger über Ports und ist der einzige Aufrufer von `DeletionProof::issue` (§12.7.5).

### WP 7.1 – `ErasedSet`, Lesesperre, Key-Lock

**Ist.** Es gibt keinen `ErasedSet` und keine Lesesperre. `engine/src/collection/crud/delete.rs:8–11` gibt den Key-Lock sofort frei (`drop(_guard)` vor `begin_transaction`; B-21). Damit ist REQ-DEL-012 verletzt.

**Änderung.**

1. `engine/src/erasure/erased_set.rs`: Eintrag `{entry_seq, scope_ref, subject_ref_hash?, unit_ref_hash[], ref_epoch, requested_at, state, prev_mac}` als Sidecar `MFES` (WP 4.2). `unit_ref_hash = HMAC(K_ref, doc_key_id)`.
2. Lesesperre in allen Lesepfaden: `get`, `search`, `hydrate`, `export`, Checkpoint-Erzeugung, `as_of` (REQ-DEL-011, REQ-RST-4, INV-ASOF-ERASE-1). Die Liste der Pfade wird per `rg` erhoben und als Tabelle in `docs/spec/READ-PATHS.md` geführt. Jeder neue Lesepfad muss dort eingetragen sein (Gate `read-path-guard`, neu).
3. `delete.rs`: `drop(_guard)` entfällt. Der Key-Lock wird über die gesamte Saga gehalten (REQ-DEL-012, §12.9). Konkurrierendes `remember` auf dieselbe Einheit wird abgewiesen (§12.2.5 Regel 5).
4. Hinweis: `delete` nutzt `check_and_trigger_community_detection`, das in WP 2.2 entfällt.

**Test (rot zuerst).** `engine/tests/read_paths_erased.rs`: für jeden Eintrag der Lesepfad-Tabelle ist der Zugriff auf eine gesperrte Einheit `Err(Erased)`. `engine/tests/loom_put_delete.rs` (AC-13): `put` und `delete` auf dieselbe ID lassen nichts wieder auftauchen (loom).

**Akzeptanz.** Die Lesepfad-Tabelle ist vollständig, und der Test deckt alle Zeilen ab.

### WP 7.2 – Saga, Journal, Typestate

**Änderung.**

1. `engine/src/forget/saga.rs`: Zustände aus §12.8.1 als Typestate (Rust-Typen mit Zustandsparameter). Ungültige Übergänge sind Kompilierfehler, wo möglich, sonst Laufzeitfehler mit `ErasureError::InvalidTransition`.
2. `engine/src/forget/plan.rs`: Schritte 1 bis 10 aus §12.8.2 als Plan. Fast-Path (Schritte 1 bis 7) und Hygiene (8 bis 10) getrennt.
3. `engine/src/forget/recovery.rs`: Recovery in B7. Jede Saga ohne `Completed` wird fortgesetzt oder als `Partial{outstanding}` gemeldet (REQ-FGT-2).
4. `engine/src/forget/guard.rs`: Forget-Guard (§13.3, INV-FORGET-GUARD-1).
5. Journal vor jedem Schritt (REQ-FGT-1). `Frozen` schreibt den `ErasedSet`-Eintrag vor der ersten Löschoperation (REQ-FGT-3).
6. INV-DEL-ORDER-1: Schritt 5 (`KeyDestroyed`) beginnt erst, wenn Siegel 2 für jede Ebene-A-Schicht ausgestellt und das Inventar durabel ist.
7. Abbruch nur zwischen Schritten (REQ-FGT-6).
8. Der `RequestContext` kommt vom Aufrufer; keine Fallbacks auf `TenantId::SYSTEM` (REQ-FGT-7).

**Test (rot zuerst).** `engine/tests/saga_transitions.rs`: Tabelle aller Übergänge. `engine/tests/saga_crash_matrix.rs`: Absturz nach jedem Schritt; Recovery führt zum gleichen Endzustand. `engine/tests/order_invariant.rs`: Schritt 5 ohne Siegel 2 wird verweigert.

**Akzeptanz.** Jede Saga endet in `Completed`, `Partial` mit Grund und Frist, oder `Restricted`. Keine andere Endung.

### WP 7.3 – Backup Spur D/K, Restore-Guard, Mirror

**Ist.** Checkpoints sind Hardlink-Klone (B-19). Es gibt keine Trennung in Spur D und Spur K, keinen Restore-Guard und keinen Mirror.

**Änderung.**

1. `checkpoint/src/tracks.rs`: Spur D (Daten ohne KeyTable, Keystore, `ErasedSet`) und Spur K (KeyTable-Snapshot mit `valid_until`, `key_backup_max_age` 30 Tage) (§12.17.1).
2. `checkpoint/src/restore.rs`: Restore-Guard vor dem ersten Lesezugriff: (1) `ErasedSet` des Zielsystems laden, (2) jeden Eintrag erneut anwenden und wiederhergestellte DocDeks vernichten, (3) Messung wie §12.7, (4) `restore_reapplied` ins Journal, (5) Audit mit Grund (§12.9). Der Guard läuft vor B9.
3. Schlüssel-Backups mit überschrittenem `valid_until` oder unbestimmbarem Alter werden verweigert (`RestoreBeyondHorizon`, REQ-RST-3). Start-Check: `key_backup_max_age ≤ erased_set_retention` (B0).
4. `engine/src/erasure/mirror.rs`: `ErasedSetMirror` (Port aus WP 3.2) schreibt jeden Eintrag an das Ziel außerhalb des Datenverzeichnisses. Im Regulated-Tier Pflicht. Mirror nicht erreichbar: `Frozen` blockiert nicht, Eintrag wird `MirrorPending`, `doctor` alarmiert nach 24 Stunden (H-11).
5. INV-NO-RESURRECTION-1: beim Start und nach jedem Restore wird für jeden `ErasedSet`-Eintrag ab `Frozen` geprüft, dass kein lebender DocDek oder ArtKey der Einheit in der KeyTable steht.

**Test (rot zuerst).** `engine/tests/restore_erased_unit.rs`: Backup mit Einheit, Löschung, Restore; danach ist die Einheit unlesbar, und der Guard hat den Schlüssel erneut vernichtet. `checkpoint/tests/key_backup_horizon.rs`: abgelaufenes `valid_until` wird verweigert.

**Akzeptanz.** Ein Restore ohne Mirror ist nur mit `--accept-unguarded` möglich und wird im Bericht als `restore_unguarded` ausgewiesen.

### WP 7.4 – Bericht V5 und Siegel

**Ist.** `crypto/src/deletion_proof.rs:353–466` (`DeletionProof::issue`): ignoriert `_custody` (U-42, Zeile 358), füllt gelöschte Schlüssel mit `Vec::new()` (Zeilen 411, 413, 452, 541), bildet unbekannte Layer-Strings still auf `LsmMemtable` ab (B-05). Im Repo gibt es höchstens `SignatureVersion::V3` (U-39). V4 existiert nicht.

**Änderung.**

1. `crypto/src/proof/v5.rs`: explizites Layout ohne `serde`-Derive (Feldreihenfolge, Endianness, Längen; §12.7.5). Magic `MFDR`. V5 ist das einzige schreibbare Format. V4 bleibt unvergeben (E46).
2. `crypto/src/proof/legacy.rs`: Lesepfade V1 bis V3, mit `coverage_known = false`.
3. `DeletionProof::issue` in der Signatur aus §12.7.5. Die Funktion prüft REQ-PRF-2 vollständig und liefert `Err` bei fehlender Pflichtschicht, falscher Klasse, leerem `excluded` oder fehlenden Annahmen.
4. Signatur Ed25519 mit dem Berichtsschlüssel aus `KmsProvider::sign_report` (INV-PROOF-KEY-1). Golden-File und Fuzz-Ziel (REQ-PRF-5).
5. Siegel 1 bis 4 (§12.8.5), jedes als Eintrag in die AuditChain mit Verkettung (REQ-DEL-N27, REQ-PRF-7).
6. `doctor verify-report` (WP 8.4) prüft offline Signatur, Prüfsumme, Klasse, Siegelkette und Zeitanker (REQ-PRF-6).

**Test (rot zuerst).** `crypto/tests/proof_v5_rules.rs`: jede Regel aus REQ-PRF-2 einzeln. `crypto/tests/proof_golden.rs`. `engine/tests/proof_seal_chain.rs`: vier Siegel, verkettet, mit korrektem Zustand je Siegel.

**Akzeptanz.** Ein Bericht kann nur über die Saga entstehen (Gate `witness-mint`, `deletion-types-once`).

### WP 7.5 – Claim-Generator und Satzbibliothek

**Ist.** Gate `claim-generator` fehlt (§17.3). `deletion-honesty` ist grün, aber sieht keine Lücken (B-18).

**Änderung.**

1. `crypto/src/claims.rs`: Satzbibliothek als Funktionen, die aus einem `ReachedClass` und Messwerten Sätze erzeugen. Jeder Satz ist an einen Messwert gebunden (INV-CLAIM-BIND-1).
2. Kein freier Text in Außentexten. Die Ausgabe kommt nur aus der Bibliothek.
3. Gate `claim-generator` (xtask-Modul, neu): sucht Außentexte mit Löschbegriffen und prüft die Bindung.
4. `deletion-honesty` prüft die Zuordnung von Klasse und Satz (REQ-DEL-035).

**Test (rot zuerst).** `crypto/tests/claims_bound.rs`: für jede Klasse entsteht genau ein Satz, dessen Bindung auf den Messwert zeigt. AC-24.

### WP 7.6 – Fristen, Mixed-Policy, `RecipientLedger`, `LegacyUnkeyed`, Restmetadaten-Test

**Änderung.**

1. `engine/src/erasure/deadlines.rs`: `requested_at` und Schwellen (§12.19.1). Jeder `Partial`-Zustand erzeugt je Schwelle genau einen Alarm (REQ-DEL-036). Contextra sendet nichts nach außen.
2. Mixed-Policy (§12.15): Mixed-Segmente führen zu `ReviewRequired`. Der Zustand ist im Saga-Typestate enthalten (WP 7.2).
3. `RecipientLedger`-Implementierung: Empfänger eines Berichts oder einer Bescheinigung (§12.19.2). Kein Sicherheitsport.
4. `LegacyUnkeyed` (§12.2.5 Regel 7): Begrenzung der Klasse auf P, bis `migrate --rekey-units` abgeschlossen ist. `doctor` weist den Anteil aus.
5. Restmetadaten-Selbsttest (§12.7.6): drei Prüfungen (Linkage, Record-Isolation, Inference) plus Entropietest. Ergebnis ist eine Warnung, kein Rechtsurteil. Läuft in `doctor` (WP 8.4).

**Test (rot zuerst).** `engine/tests/deadline_alarm_once.rs`. `crypto/tests/legacy_unkeyed_cap.rs`. `crypto/tests/residual_selftest.rs` mit synthetischen Chiffratblöcken.

---

## 13. Phase 8 – Fassade, Kernhärtung, Schnittstellen

### WP 8.1 – Composition Root und Boot B0–B9

**Ist.** `crates/contextra/src/builder.rs` existiert. Die Fassade hat `default = ["fast", "candle"]` (B-15; Ziel `embedded`, WP 2.3). Das Boot-Protokoll B0–B9 ist nicht als Typestate umgesetzt.

**Änderung.**

1. `engine/src/boot/`: jede Phase als eigene Funktion, Reihenfolge per Typestate erzwungen (INV-BOOT-1, Gate `boot-order`).
   - B0 Konfigurationsprüfung, u. a. `key_backup_max_age ≤ erased_set_retention`.
   - B1 Dateisystemprüfung (ext4, xfs, btrfs, zfs, APFS, NTFS erlaubt; NFS, SMB, FUSE, Cloud-Sync-Ordner abgelehnt, §7.12) und Verzeichnissperre.
   - B2 `KmsProvider` (fehlend: Start verweigert).
   - B3 Scanner-Vorhandensein; `FeatureDisabled` nur mit Datenbelegung (§12.4 Regel 1).
   - B4 bis B5 WAL-Kette, Replay.
   - B6 `ErasedSet` und Mirror-Abgleich.
   - B7 Journal-Replay, Saga-Fortsetzung, Wiederauferstehungsprüfung.
   - B8 Sidecars.
   - B9 Verkehr erlaubt. Der Restore-Guard läuft davor.
2. Selbsttest vor B9 (Schreiben und Lesen einer Kontrollzeile).
3. Die Systemuhr-Implementierung (WP 3.5) liegt in `contextra/src/builder.rs` (Composition Root). Dort ist sie die einzige erlaubte Stelle.

**Test (rot zuerst).** `engine/tests/boot_order.rs`: falsche Reihenfolge kompiliert nicht. `engine/tests/boot_b2_refuses.rs`: ohne KMS startet nichts.

### WP 8.2 – `remember`/`recall`, Embedding-Identität, Index je Tenant

**Änderung.**

1. `remember` erzeugt bei neuer Einheit genau einen DocDek (§12.2.5 Regel 1) und schreibt über den WAL-Batch.
2. `recall` über Planner aus §11.2. `max_results` ist begrenzt (Gate `check-max-results-unbound`).
3. Embedding-Identität (§11.1, INV-EMBED-1/-2): `EmbeddingProvider` wird über den Port injiziert. Ohne Backend (`EmbeddingBackend = None`) gilt nur Textsuche.
4. Index je Tenant (INV-TENANT-IDX-1): kein gemeinsamer Index zwischen Mandanten.
5. `db` kennt keine Inferenz-Backends (Kantenregel 8).

**Test (rot zuerst).** `engine/tests/remember_creates_docdek.rs`. `engine/tests/tenant_index_isolation.rs`.

### WP 8.3 – MCP: drei Werkzeuge, Pipeline, Forget-Guard, `confirm`, IPC

**Ist.** Der Werkzeugsatz `contextra_*` hat 15 Werkzeuge (B-16; §13.2 verlangt drei). Konfiguration über `std::env::var` (`mcp/src/config.rs`, `proof_key_env.rs`; WP 1.2 entfernt die Schlüsselpfade).

**Entscheidung und Ist-Abbildung.**

| Ist-Werkzeug (Scan/B-16) | Ziel v1.0 | Hinweis |
| --- | --- | --- |
| `contextra_insert` | `memory_remember` | Mandant aus Konfiguration |
| `contextra_search` | `memory_recall` | Budget aus Konfiguration |
| `contextra_forget` | `memory_forget` | über Saga, Forget-Guard, `confirm` |
| `contextra_delete` | entfällt (Werkzeug) | Löschen nur über `memory_forget` |
| `contextra_drop_collection` | entfällt (Werkzeug) | CLI und Fassade (§13.2) |
| `contextra_cloud_query`, `contextra_relate`, `contextra_relate_n_ary`, `contextra_consolidate`, `contextra_plugin_status` | entfallen | Archiv-Abhängigkeiten (WP 2.3) |
| übrige Werkzeuge | entfallen oder CLI | Entscheidung je Werkzeug in der PR-Beschreibung |

**Änderung.**

1. Fünf Pipeline-Stufen je Aufruf (§13.2): Schema und Größe, Prinzipal, Autorisierung gegen Kategorie, Scope und RevocationLog (fail-closed), Budget und Rate, Audit vor und nach der Ausführung ohne Nutzinhalte.
2. Annotationen aus der Kategorie: `memory_recall` `readOnlyHint`, `memory_remember` Schreiben, `memory_forget` `destructiveHint`.
3. `confirm.rs`: Außenkanal für Destruktives (zwei Bestätigungen im Regulated-Tier, H-10).
4. `memory_forget` liefert sofort einen `DeletionReceipt` mit Siegel 1 und einem `PendingDeletion`-Handle (§12.8.8). Kein `DeletionProof` in der Antwort außerhalb der Saga. `class`, `claims`, `excluded` gemäß §12.13.
5. `print_stdout` ist im MCP-Binär verboten (INV-STDOUT-1, Lint-Satz §17.3).
6. `unsafe` in `mcp` ist nicht zulässig (§4.6).

**Test (rot zuerst).** `mcp/tests/tool_list_snapshot.rs`: genau drei Werkzeuge mit den Annotationen. `mcp/tests/no_proof_without_principal.rs`: Aufruf ohne Prinzipal → `Denied`. `mcp/tests/pipeline_order.rs`.

**Akzeptanz.** Gate `public-api` zeigt keine `contextra_*`-Werkzeuge. Keine Konstante `0` oder Literalschlüssel im MCP-Code (REQ-KEY-010, neues Gate `no-literal-secrets`).

### WP 8.4 – `doctor`, CLI, `migrate`

**Änderung.**

1. `doctor` prüft: Dateisystem (B1), K0-Betrieb (rot), Schlüsselstatus, `verify-report` (REQ-PRF-6, offline), `crypto-drill` (halbjährlich, §12.2.9), Restmetadaten-Selbsttest (WP 7.6), `ClockRegression` (§12.8.4), Mirror-Alarm, `restore_unguarded`.
2. CLI-Befehle: `doctor`, `confirm`, `verify-report`, `seal-now`, `report <id>`, `migrate [--dry-run] [--rekey-units]`.
3. Die CLI gehört in `contextra` (Composition Root, Ring 4). Entscheidung E-C1 (Abschn. 17) legt fest, ob sie in `contextra/src/bin/` oder in `mcp` liegt.
4. `contextra migrate` nutzt den Kern aus WP 4.3 und erzeugt vor jeder Änderung ein verifiziertes Backup (REQ-FMT-6).

**Test (rot zuerst).** `contextra/tests/cli_doctor_red_on_k0.rs`. `contextra/tests/migrate_dry_run.rs`.

### WP 8.5 – SSI-Nachweis, loom-Inventar, Fuzz-Matrix

**Ist.** `supports_ssi_tracking` gibt konstant `true` zurück (`store/src/lsm/ops.rs:53`, U-06). Die SSI-Validierung in `mvcc/src/ssi.rs` ist zu prüfen (SOLO-Port `SsiValidator`, Scan §7).

**Änderung.**

1. `supports_ssi_tracking` liefert das tatsächliche Verhalten des Speichers oder wird entfernt.
2. T-SSI-1: Write-Skew-Test. Serialisierbarkeit verlangt den Abbruch einer der beiden Transaktionen.
3. loom-Inventar: `mvcc` und `engine/collection/kv_lock.rs` (§8.2). Jeder loom-Test ist im Inventar mit Zweck und Laufzeit geführt.
4. Fuzz-Matrix: WAL, SSTable, Sidecar, KeyTable, `MFDR`, AuditChain, RevocationLog, Schema-Dekoder der Wire-Schicht (xtask-heavy).

**Test (rot zuerst).** `mvcc/tests/ssi_write_skew.rs`. Gate `loom-inventory`.

---

## 14. Phase 9 – Abnahme

### WP 9.1 – Gate-Satz vollständig

Die Gate-Tabelle in Abschn. 15 wird in `governance/gates.toml` vollständig geführt. Jedes neue Gate hat einen Selbsttest mit synthetischem Repo (REQ-QA-008). Gates ohne Ausführung sind rot (INV-EVIDENCE-1).

### WP 9.2 – AC-Matrix, Crash-Matrix, Canary-Läufe

1. AC-Matrix: jede Akzeptanzbedingung aus §12.12 hat einen Test und einen Ergebnis-Eintrag.
2. Crash-Matrix: alle Absturzpunkte aus §12.8.4 und §7.3.1 mit `FaultVfs`.
3. Canary-Läufe: reproduzierbar, mit Positivkontrolle, Protokoll archiviert (§17.9).

### WP 9.3 – Benchmarks und `budgets.toml`

`bench` (Ring Werkzeug) misst Recovery-Zeit je Tier (§16), Löschlatenz (Fast-Path), Recall-Stabilität und Re-Wrap-Kosten je Epoch Seal (O-24, O-40). Fremde Baselines sind Pflicht für Vergleichsaussagen (§16.3).

### WP 9.4 – Release-Kriterien und Review

1. Release-Kriterien nach §18.3 vollständig dokumentiert.
2. Root-`CHANGELOG.md` anlegen (B-17); `docs/CHANGELOG.md` bleibt als Verweis.
3. Review-Paket: Evidenzordner `evidence/`, Gate-Protokolle, AC-Matrix.

---

## 15. Gate-Plan: Status am Klon und Phase, die das Gate grün macht

Quelle „Scan“ = `contextra_scan_gates.txt`. Quelle „Spec“ = Gesamtspezifikation §17.3. Spalte „Grün durch“ nennt das WP, nach dem das Gate passieren muss.

| Gate | Quelle | Status am Klon | Grün durch | Pflicht |
| --- | --- | --- | --- | --- |
| `ring-layering` (Spec: `ring-dag`, K-14) | Scan | rot: 2 Fehler, 66 Warnungen | 2.5 | ja |
| `product-hull` | Spec §4.5 | fehlt (`products/` existiert nicht) | 2.3, 8.1 | ja |
| `module-removal` | Spec §4.3 | nur im CI prüfbar | 2.4 | ja |
| `feature-consistency` | Scan | rot: 24 tote Features; `unexpected_cfgs = "warn"` | 0.4, 2.3 | ja |
| `test-surface` | Scan | rot: 18 | 1.3 | ja |
| `no-silent-noop` | Scan | warn: 5, davon 4 in Loom-Shims (K-04) | 0.2 (Einordnung) | ja |
| `crypto-rng` | Scan | warn: 7 | 3.5 | ja |
| `determinism` | Scan | rot: 44 (35 aktiv, 9 Archiv, K-07) | 3.5, 2.1b | ja |
| `unwired-pub` | Scan | warn: 94 | 7.x, 8.x | nein |
| `deletion-honesty` | Scan | grün, prüft aber nichts Relevantes (B-18) | 7.5 | ja |
| `raw-carrier` | Scan | rot: 2; Kanarienmuster und Stufe S3 fehlen | 6.1, 9.2 | ja |
| `unsafe-audit` | Scan | rot: 5 Inseln | 2.6 | ja |
| `doc-stamp` | Scan | warn: 7 | 0.3 | ja |
| `panic-surface` | Scan | warn: 2 | 1.3, 4.4 | ja |
| `security-ports` | Spec §5.4 | Datei fehlt | 0.4 (Datei), 3.2 (Ports), 5.x (Implementierungen) | ja |
| `async-io` | Scan | rot: 18 (11 aktiv nach Archiv, K-08) | 3.3, 4.1 | ja |
| `key-purposes` | Spec §12.3 | Datei fehlt | 0.4, 5.2 | ja |
| `witness-mint` | Spec §12.7.4 | nicht als Gate-Name vorhanden | 6.8 | ja |
| `surface-coverage` | Spec §12.4 | nicht vorhanden | 6.x, 9.1 | ja |
| `claim-generator` | Spec §12.11 | nicht vorhanden | 7.5 | ja |
| `format-registry` | Spec §7.11 | ungeprüft (Entsprechung `format-corpus`) | 4.3 | ja |
| `no-security-stub` | Spec §17.3 | nicht vorhanden | 1.1 | ja |
| `no-literal-secrets` | REQ-KEY-010 | nicht vorhanden | 1.2, 8.3 | ja |
| `no-prod-mock` | neu (WP 3.4) | nicht vorhanden | 3.4 | ja |
| `read-path-guard` | neu (WP 7.1) | nicht vorhanden | 7.1 | ja |
| `boot-order` | Spec §17.3 | nicht vorhanden | 8.1 | ja |
| `unbounded-spawn` | REQ-ASYNC-3 | nicht vorhanden | 4.1 | ja |
| `loc-budget` | Spec §1.3 | Werkzeug `tools/loc.py` vorhanden, Gate fehlt | 9.1 | ja |
| `loom-inventory` | Spec §17.1 | ungeprüft | 8.5 | ja |
| `await_holding_lock` (Clippy-Lint) | Spec §17.3 | fehlt im Workspace-Lint-Satz (K-13) | 0.4 | ja |
| `ratchet` | Spec §6, §17 | Datei vorhanden, Schema neu (K-02) | 0.4 | ja |

## 16. Testmatrix: Anforderung → Test → Arbeitspaket

| Anforderung | Testdatei (Ziel) | WP |
| --- | --- | --- |
| REQ-DEL-008 (keine Konstante als Messung) | `ports/tests/` (`trybuild`) | 6.8 |
| REQ-PRF-2 (Regeln für `issue`) | `crypto/tests/proof_v5_rules.rs` | 7.4 |
| REQ-PRF-1 / U-21 (kein Stub) | `engine/tests/review_stub_proof.rs` | 6.8, 7.4 |
| REQ-DEL-011 (Lesesperre) | `engine/tests/read_paths_erased.rs` | 7.1 |
| REQ-DEL-012 / AC-13 (Key-Lock über Saga) | `engine/tests/loom_put_delete.rs` | 7.1 |
| INV-KDF-1 (nur Argon2id) | `crypto/tests/kdf_argon2_only.rs` | 5.1 |
| LA-4 (kein Kollateralverlust) | `store/tests/destroy_leaves_others_readable.rs` | 5.3 |
| INV-HNSW-SLOT-1 | `vector/tests/no_slot_reuse_before_rebuild.rs` | 6.2 |
| INV-NO-RESURRECTION-1 | `engine/tests/restore_erased_unit.rs` | 7.3 |
| INV-SIDECAR-1 / §7.9 | `store/tests/sidecar_chain.rs` | 4.2 |
| INV-FORMAT-1 / REQ-FMT-3 | `store/tests/format_corpus_roundtrip.rs` | 4.3 |
| REQ-FGT-1 (Journal vor Schritt) | `engine/tests/saga_crash_matrix.rs` | 7.2 |
| INV-DEL-ORDER-1 | `engine/tests/order_invariant.rs` | 7.2 |
| INV-CORE-ISOLATION-1 | Gate `core-isolation` (`cargo metadata`, aufgelöste Features) | 2.3 |
| INV-SCAN-1 (keine Schlüssel über API) | Gate `public-api` | 1.3 |
| REQ-MCP-004 (fünf Pipeline-Stufen) | `mcp/tests/pipeline_order.rs` | 8.3 |
| INV-BOOT-1 | `engine/tests/boot_order.rs` | 8.1 |
| T-SSI-1 | `mvcc/tests/ssi_write_skew.rs` | 8.5 |
| AC-1 (Positivkontrolle) | `store/tests/lsm_scan_needle.rs` | 6.1 |
| AC-24 (Satzbindung) | `crypto/tests/claims_bound.rs` | 7.5 |

## 17. Entscheidungspunkte (vor dem jeweiligen WP zu klären)

Diese Punkte sind in dieser Fassung **nicht** entschieden. Die Empfehlung ist der Vorschlag des Autors; die Freigabe erfolgt über ADR (§18.6 der Spec).

| ID | Frage | Optionen | Empfehlung | Blockiert |
| --- | --- | --- | --- | --- |
| E-W1 | Wo liegt `LayerWitness` und `ScanSession`? | `types` (Spec §12.7.4) oder `ports::scan` | `ports::scan`. `types` nur Grundtypen. Begründung: Konstruktion nur über Trait-Default-Methode, die `types` nicht sicher kapseln kann. | 3.1, 6.8 |
| E-B1 | Budgets `types`, `ports`, `sys` werden gesprengt (K-12) | ADR mit Zuwachs; Auslagerung in andere Crates | ADR mit gemessenem Ist nach WP 2.3. Zuwachs nur für Löschtypen, nicht für Archiv-Reste. | 3.1–3.3 |
| E-K1 | `store/src/kv/` und `kv_shredding` | behalten oder streichen | streichen, sofern `rg` außerhalb des Archivs keine Nutzung zeigt | 5.6 |
| E-L1 | Lineage-Kanten liegen im archivierten Graph (B-06, `cascade.rs`) | Lineage in Engine migrieren oder nur Sidecar-Ableitungen messen | nur Sidecar-Ableitungen; Lineage-Kanten sind ein Produktentscheid vor v1.0 | 6.7 |
| E-C1 | Ort der CLI | `contextra/src/bin/` oder `mcp` | `contextra` (Ring 4, Composition Root) | 8.4 |
| E-M1 | Migrationskern vorziehen (K-16) | ja / nein | ja | 4.3 |
| E-A1 | Async-Grenze in Ring 0 (Spec §6, Spike S-ASYNC) | Spike-Ergebnis entscheidet über Umfang von Phase 4 | Spike in Phase 0 als reine Messung durchführen; Schwelle 70 % laut Spec | 4.1 |
| E-S2 | K2-Plattformen (TPM/HSM) | in v1.0 ausliefern oder Port vorsehen | Port vorsehen, nicht ausliefern (Spec §12.2.3) | 5.5 |
| E-U1 | Windows-Garantie | Garantie oder Best-Effort | Best-Effort, `platform_guarantee = false` (D-11) | 3.3 |
| E-T1 | Standort der Systemuhr-Implementierung | Composition Root oder `ports` | Composition Root (`contextra/src/builder.rs`) | 3.5, 8.1 |
| E-N1 | Verwendung von `thread_rng` an `hmac.rs:251,367,456` (K-07) | Zähler, IDs oder Nonces | Prüfung in WP 3.5, Punkt 4; danach Erlaubnisliste eng fassen | 3.5 |

## 18. Risiken

1. **Kompilierung nicht belegt.** Solange WP 0.1 nicht grün ist, sind alle Aufwandsaussagen unsicher. WP 0.1 blockiert alles.
2. **Formatwechsel (WP 5.3).** Datenverlust bei fehlerhafter Migration. Gegenmaßnahmen: Golden-Files aus dem Ist-Format (K-17), Dry-Run, verifiziertes Backup, Migration in Schritten (erst Blob mit altem Umschlag, dann Umschlag).
3. **Entkopplung (WP 2.2, 2.3).** 286 Dateien referenzieren `contextra_core` (K-11). Ein fehlerhaftes Migrationsskript bricht viele Crates gleichzeitig. Gegenmaßnahme: ein Commit je Kante, `cargo check` nach jedem Schritt.
4. **Vollständigkeit der Lesesperre (WP 7.1).** Ein übersehener Lesepfad ist eine Lücke im Löschversprechen. Gegenmaßnahme: `rg`-Inventar, Tabelle `READ-PATHS.md`, Gate `read-path-guard`.
5. **Sicherheitsversprechen gegenüber Verhalten.** Windows (D-11) und K2/K3 (D-10) dürfen im Bericht nicht als Garantie erscheinen. Gegenmaßnahme: `platform_guarantee` und `CapabilityUnsupported` werden im Bericht geführt und vom Gate `deletion-honesty` geprüft.
6. **Archiv-Abhängigkeiten als Scope-Creep.** Lineage (E-L1), RRF in `engine` (Kantenregel 10), `cognition` in `db`. Gegenmaßnahme: Entscheidungen E-L1 und E-K1 vor Phase 2 treffen.
7. **Testlaufzeit.** Argon2id mit 250 ms macht Tests langsam. Gegenmaßnahme: Testprofil nur hinter `cfg(test)` oder `test-hooks` (WP 5.1).
8. **Blockade durch offene Entscheidungen.** E-W1 und E-B1 blockieren Phase 3. Gegenmaßnahme: Freigabe vor Beginn von Phase 3.

## 19. Definition of Done je Phase und nächste Schritte

| Phase | Abnahme (alle Punkte erfüllt) |
| --- | --- |
| 0 | Evidenz vollständig unter `evidence/baseline-a547a42/`; jede rote Stelle in `OPEN.md`; Gate-Läufe `--full` archiviert; ADR D-01 bis D-14 angelegt; E-A1 gemessen; `cargo tree -e features` für B-23 vorliegt |
| 1 | `rg "no_crypto_stubs\|encryption-at-rest"` liefert 0 Treffer; keine Literal-Schlüssel und keine Konstante `0` im MCP; Gates `test-surface` und `panic-surface` grün |
| 2 | Tag `archiv/v0`; `cargo metadata` zeigt 16 Kern-Crates; `contextra-core` existiert nicht mehr; `cargo build -p contextra --no-default-features --features embedded` grün; `core-isolation` grün |
| 3 | `types`, `ports`, `sys` kompilieren mit den neuen Modulen; `security-ports` listet alle Sicherheitsports; `determinism` ist gegenüber 44 gesunken |
| 4 | `async-io` in aktiven Crates: 0 Verstöße; `docs/spec/FORMATS.md` existiert; Golden-Files für `MFW2`, `MFW3`, `MFSX`, `MFMN` grün; `MFST`-Lesepfad entfernt |
| 5 | Schlüsselhierarchie aktiv; Umschlag mit Padmé; Destroy-Matrix grün; Epoch Seal idempotent; `kv_shredding` entfernt |
| 6 | Alle Scanner liefern `ResidualReport` nur über `ScanSession`; Nadelscan mit Positivkontrolle grün; `witness-mint` grün |
| 7 | Saga mit Typestate; Lesesperre über alle Pfade (Tabelle vollständig); Bericht V5 mit Golden-File; Restore-Guard vor B9 |
| 8 | Genau drei MCP-Werkzeuge; `boot-order` grün; `doctor` meldet K0 rot; CLI `migrate --dry-run` |
| 9 | Gate-Satz grün; AC-Matrix vollständig; Canary-Protokoll archiviert; `budgets.toml` gesetzt |

**Nächste Schritte in dieser Reihenfolge:**

1. WP 0.1: Build-, Test- und Clippy-Baseline auf `a547a42` erzeugen. Bis dahin gibt es keine belastbare Aufwandsaussage.
2. Entscheidungen E-W1, E-B1, E-K1, E-L1, E-C1 und E-M1 treffen und als ADR festhalten.
3. Spike S-ASYNC (E-A1) und die Budget-Messung (E-B1) als reine Messung durchführen.
4. WP 0.4: Governance-Dateien erweitern (K-02) und die drei fehlenden Dateien anlegen.
5. Erst danach Phase 1 beginnen. Phase 1 ist klein und behebt die schwersten Sicherheitsbefunde (B-02, B-04).

**Vorbehalt.** Diese Fassung ist eine Arbeitsunterlage im Sinne von Spec §18.7. Sie ändert keine Soll-Aussagen. Die Zeilenangaben gelten für `a547a42` und sind aus Stichproben und dem Scan belegt; was nicht geprüft wurde, ist in §0 und in den Abschnitten mit „ungeprüft“ benannt. Es gibt keine Stunden- oder Personentage, weil die Kompilierbarkeit nicht belegt ist.
