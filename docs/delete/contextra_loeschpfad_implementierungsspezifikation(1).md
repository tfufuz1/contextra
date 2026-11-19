# Contextra: Implementierungsspezifikation Löschpfad (Löschgarantie und Löschnachweis)

**Basis:** `contextra_loeschpfad_audit.md` (HEAD `b877c18`), neu gegen `tfufuz1/contextra` geprüft: Erstanalyse an `77acbaa49`, danach `git pull` und **Nachprüfung an HEAD `a969237ab`** (Merge PR #5125, 62 weitere Commits). Dazu `docs/analysis/*` vom 19./20.10.2026. Wo ich nichts anderes schreibe, gilt der Befund an beiden Ständen.
**Rolle:** Principal Rust Architect. Dieses Dokument ersetzt das Audit nicht, es macht daraus bauanleitungsreife Arbeitspakete.

---

## 0. Evidenz und was ich tatsächlich ausgeführt habe

| Klasse | Bedeutung |
|---|---|
| **R** | Reproduziert: Test geschrieben und ausgeführt, Ergebnis beobachtet |
| **C** | Compiler-/Testlauf-Ergebnis beobachtet |
| **L** | Quellcode am HEAD `77acbaa49` gelesen, nicht ausgeführt |
| **H** | Hypothese, braucht Test (Test ist in der Spezifikation benannt) |
| **A** | Aussage aus `docs/analysis/*` oder dem Audit, am HEAD nicht selbst nachgeprüft |

**Durchgeführt:**
- Repository geklont und später per `git pull` aktualisiert, Toolchain **rustc/cargo 1.89.0** (exakt der Pin aus `rust-toolchain.toml`) per `apt` installiert. Das Audit hatte nichts kompilieren können; das ist hier anders.
- `cargo check -p contextra-store --lib` und `-p contextra-engine --lib` am unveränderten HEAD (beide Stände): **beide schlagen fehl** (B0). Am neuen HEAD sind es zusätzlich 9 Fehler durch die Port-Konsolidierung (B0-d).
- Lokale Minimalfixes, **nur in meinem Klon**: (a) `contextra-store`: 11 gelöschte Zeilen (`0001-fix-build-break-store.patch`); (b) `contextra-ports/src` auf den Stand `77acbaa49` zurückgesetzt, um B0-d zu umgehen. Danach bauen Store **und** Engine (`cargo check` grün, 39 s).
- Repro-Tests (**beide rot am neuen HEAD**): `zz_repro_n1_resume_noop.rs` (N1) und `zz_repro_wp5_swap_order.rs` (Swap-Fehler, WP-5). Beide R.
- Repro-Test für den Drop-Pfad über die Fassade (`zz_repro_drop_path.rs`, Engine); Ergebnis siehe 3.5.
- Vierzehn bestehende Purge-/Fence-/WAL-Testdateien einzeln ausgeführt (Abschnitt 2.3).

**Nicht durchgeführt:** Die Engine-Tests brauchen mehr als 5 Minuten Compile-Zeit; ich habe nur den Drop-Pfad-Repro angestoßen. Aussagen über `forget`, Witness-Ausstellung und Ledger-Aufrufer sind **L**, nicht **R**. Kein Fuzzing, keine Mutationstests, keine Loom-Läufe (Loom-Tests existieren, `erasure_fence_loom.rs` wurde nicht ausgeführt). Rechtliche Aussagen übernehme ich aus dem Audit (Klasse A); ich habe sie nicht geprüft.

**Zur Zuverlässigkeit der `docs/analysis`-Dokumente:** Sie stammen von automatisierten Auditoren auf älteren Commits (z. B. ERA auf `de5000ba`). Mehrere nennen Testläufe („130 passed“), die am heutigen HEAD nicht reproduzierbar sind, weil das Store-Crate dort nicht kompiliert. Ich übernehme daraus nur, was ich am HEAD bestätigt habe, und kennzeichne den Rest als A.

**Was „100 % sauber“ hier heißt:** Ich kann keine Fehlerfreiheit garantieren. Ich definiere stattdessen ein überprüfbares *Done-Kriterium* (Abschnitt 7). Wer „sauber“ sagt, muss dieses Kriterium maschinell prüfen können.

---

## 1. Kurzurteil (korrigiert gegenüber dem Audit)

1. **Der Hauptzweig baut nicht (B0), und es wird schlechter.** Zwei parallel gemergte Perf-PRs (#5107, #5109) ließen in `contextra-store` eine doppelte Methode und einen verwaisten Aufruf zurück (bis heute nicht behoben). Ein später gemergter Commit („consolidate SystemClock and SystemRng in sys“, `fa55bf663`) entfernte die Typen aus `contextra-ports`, ohne die Aufrufer anzupassen, und die neuen Typen implementieren die Port-Traits nicht. Außerdem kompilieren drei Fence-/Crash-Testdateien nicht. **Jede weitere Aussage über „grüne Tests“ am HEAD ist damit hinfällig**, und genau die Tests, die Fence und Crash-Matrix absichern sollen, laufen nicht.
2. **N1 ist real und reproduziert (R).** Resume nach Absturz in P4 entfernt das Intent und lässt die Daten liegen. Der Test `purge_orchestrator_resume.rs` grünt, weil er ohne Storage läuft.
3. **Zwei weitere Befunde sind reproduziert (R), nicht nur gelesen:**
   - **Swap-Fehler in `purge_keys` (WP-5/5a):** Wird eine SSTable durch den Purge leer und eine spätere umgeschrieben, ordnet der RAM-Swap die Reader falsch zu. Danach liefert `get` für einen *nicht gelöschten* Schlüssel eine **ältere, überschriebene Version**. Das Löschen macht fremde Altwerte wieder sichtbar.
   - **N23:** Jede Standard-Löschung einer Collection meldet „Physical purge NotPerformed“, obwohl der Purge gelaufen ist (Rohsweep sauber). Der Driver verlangt fest den Claim `S2V`, die Standard-Assurance ist `S1`; die Zeugenausstellung scheitert nach dem Purge. Der vorhandene Test `drop_collection_layer_claims.rs` erwartet `NotPerformed` und zementiert den Fehler.
4. **N3 ist am HEAD behoben (L).** `purge_keys` schreibt die Obsolet-Liste vor dem Manifest-Tausch ins Journal (`ops/purge.rs`, Abschnitt „Durably record obsolete file paths ... BEFORE manifest transition“). Das Audit ist hier überholt.
5. **N4 und L10 sind teilweise behoben, mit neuen Mängeln (L).** PR #5110 entfernte `let _ = run_purge` und die falsche „ErasureSupervisor“-Zeile. Dafür gibt es neue Fehler im Drop-Pfad (D1 bis D6, Abschnitt 3).
6. **K5 und L7 sind seit heute durch einen Fast-Path angegangen, der selbst ein Race hat (L).** Die Prüfung `is_active()` im Commit-Pfad läuft **vor** dem `InflightGuard`. Das ist ein Check-then-Act-Fenster (N8), dazu kommt eine zu schwache Speicherordnung (N9).
7. **Neu und schwerwiegend:** WAL-Retire und WAL-Scope-Purge haben keinen Produktionsaufrufer, der `PurgeDriver` übergibt keinen WAL-Callback (N10). `ledger_append` hat keinen Produktionsaufrufer, das Anti-Resurrection-Ledger wird nie beschrieben (N11). `WAL-002` (HWM-Vergleich mit sich selbst) ist am HEAD bestätigt und löschrelevant (N12).
8. **Rechtliche Einordnung** aus dem Audit (Abschnitt 4 dort) bleibt unverändert gültig als Recherche, nicht als Beratung.

---

## 2. Abgleich: Audit gegen HEAD `77acbaa49`

### 2.1 Statusänderungen gegenüber dem Audit

| ID | Audit-Stand | Stand am HEAD | Ev. |
|---|---|---|---|
| N1 | offen, kritisch | **bestätigt und reproduziert**; Aufruf bei `recovery.rs:564-565`, Überspringen bei `orchestrator.rs:304` | R |
| N2 | offen | bestätigt: `reconstruct_scope` nie gesetzt, `PurgeScope::Keys(Vec::new())` (`resume.rs`) | L |
| N3 | offen, hoch | **behoben** (Journal vor `append_batch`) | L |
| N4 | offen, kritisch | **teilweise**: `let _` weg, Report-Typ `DropCollectionReport`; neue Mängel D1 bis D6 | L |
| N5 | offen | bestätigt, präzisiert: `PersistentCheckpointStore` hat keinen Produktionskonstruktor; `with_restore_filter` keinen Produktionsaufrufer | L |
| N6 | offen | bestätigt (Ports ohne Produktionsimplementierung) | A |
| N7 | WAL-Retire verschluckt Lesefehler | **behoben** (`retire.rs` propagiert `?`). Aber: kein Produktionsaufrufer (N10) | L |
| K5 | drei Read-Locks je Schreibvorgang | **behoben im Sinn der Performance** (atomarer Fast-Path), neues Race N8/N9 | L |
| L7 | globale Fence | **behoben**: Orchestrator nutzt `acquire_scope_fence_for(prefixes)` | L |
| L5 | offen im WAL-Retire | siehe N7, behoben | L |
| L9 | `write_seq` zählt nur Abweisungen | nicht erneut geprüft | A |
| E6 | `issue` öffentlich | **bestätigt**: `#[doc(hidden)] pub fn issue`, vier Produktionsaufrufer in Engine | L |
| E2 | `bincode` im Signaturpfad | **bestätigt, verschlimmert**: `collections.rs` baut den signierten Payload per Hand mit `bincode` | L |
| ERA-001 | `let _ = remove_file` in `wal/scope_purge.rs` | **bestätigt**, jetzt Zeilen 417, 435, 440 (drei Stellen) | L |

### 2.2 Gegenprüfung der ERA-Analyse (`ERA_erasure_ANALYSE_2026-10-19.md`)

| ERA-Aussage | Ergebnis |
|---|---|
| H8 „Verdrahtung Forget → Purge → Witness → Ledger → Proof läuft e2e, CONFIRMED“ | **Widerlegt am HEAD.** Das Ledger wird nie beschrieben (N11), WAL-Retire läuft nie (N10), Resume ist ein No-op (N1). |
| H1 „Canary nach Purge zu 100 % eliminiert“ | Teilweise bestätigt: `purge_obsolete_raw_sweep` läuft nach lokalem Fix grün (C). Das deckt nur den direkten Aufruf von `purge_keys` plus `dispose`, nicht den Produktionspfad über `PurgeDriver`. |
| ERA-001 | bestätigt (L), siehe oben |
| ERA-002 (HNSW `let _ = update_backlink`) | A, nicht geprüft |
| ERA-003 (`NoOpRestoreFilter`) | A. Der relevante Befund ist N5 (der Filter wird gar nicht verdrahtet). |
| ERA-004, ERA-005 | A, nicht geprüft |
| „Q5: 0 Locks über `.await`“ | **Zweifelhaft**: `ErasureGate::acquire_fence` hält `self.install` (Mutex-Guard `_guard`) über mehrere `.await` (`fence.rs`, Drain-Schleife). H |

### 2.3 Testbaseline am HEAD (mit lokalem Store-Fix, einzeln ausgeführt)

| Testdatei | Ergebnis |
|---|---|
| `purge_keys_compaction`, `purge_keys_crash_atomic`, `purge_obsolete_raw_sweep`, `purge_orchestrator_resume`, `purge_orchestrator_failpoints`, `lsm_purge_fence_tests`, `purge_dispose_scrub_policy` | grün |
| `lsm_purge_compaction_tests` | **schlägt fehl** (`test_purge_keys_success` erwartet `PurgeBlocked::Unimplemented`, das `purge_keys` nicht mehr liefert; Test ist veraltet) |
| `lsm_purge_fail_closed` | **kompiliert nicht** (`unresolved import contextra_core`, `LsmStorage::put` nicht gefunden; Test gegen eine alte API) |
| `purge_crash_matrix_dst` | **kompiliert nicht** (E0782, Trait ohne `dyn`) |
| `purge_prefix_fence_concurrency` | **kompiliert nicht** (E0616, Feld `erasure_fence` privat) |
| `erasure_fence_scope` | **kompiliert nicht** (E0425, `guard`/`_guard` nicht gefunden) |
| `purge_wal_retire_errors` | **schlägt fehl** (`retire_segments_up_to` liefert keinen Fehler bei Replay-Fehler, Zeile 62) |
| `wal_scope_purge` | „0 passed“ (Datei enthält keine ausgeführten Tests, vermutlich hinter Feature) |

---

## 3. Befundregister (konsolidiert)

Schweregrad: **K** kritisch (Löschversprechen falsch oder Datenverlust), **H** hoch, **M** mittel, **N** niedrig.

### 3.1 Bau und Test

- **B0-a (K), C:** `contextra-store`: `engine.rs:101` und `engine.rs:116` definieren beide `erasure_fence`; `ops/write.rs:200` ruft `check_write_batch_allowed_scope`, das nicht existiert. Ursache: Merge von #5107 und #5109. Fix: 11 Zeilen, siehe Patch.
- **B0-b (K→H), C:** `contextra-engine`: `purge_driver.rs` `match blocked` deckte `ReadError`, `WriteError`, `ManifestError` nicht ab (E0004). **Am neuen HEAD „repariert“**, indem die drei harten I/O-Fehler als `PurgeHandle::Blocked` mit Hinweistext getarnt wurden („Check disk health ...“). Das kompiliert, bestätigt aber genau den Designfehler N13: ein Aufrufer, der bei `Blocked` wiederholt, wiederholt einen defekten Zustand.
- **B0-d (K), C:** Am neuen HEAD scheitern `contextra-store` (9 Fehler: `contextra_ports::SystemClock`/`SystemRng` nicht gefunden, u. a. `wal/hmac.rs`, `erasure_ledger/mod.rs`, `lsm/recovery.rs`, `compaction/engine.rs`) und rund 30 weitere Dateien im Workspace (Grep: `mcp`, `adapt`, `checkpoint`, `db`, `cognition`, `rank`, `crypto`, `testkit`, Engine). Ein naiver Importtausch auf `contextra_sys::SystemClock` genügt nicht: `error[E0277]: SystemClock: Clock` bzw. `SystemRng: SeededRng` nicht erfüllt. Die Konsolidierung ist also unvollständig (Adapter zwischen Ring 0 und Port fehlt). Das berührt den Löschpfad direkt, weil Ledger-Öffnung (`erasure_ledger/mod.rs`) und Recovery (`recovery.rs:30`) die Zufallsquelle nutzen.
- **B0-c (H), C:** Vier Testdateien kompilieren nicht (`purge_crash_matrix_dst`, `purge_prefix_fence_concurrency`, `erasure_fence_scope`, `lsm_purge_fail_closed`), zwei schlagen fehl (`purge_wal_retire_errors`, `lsm_purge_compaction_tests`); siehe 2.3. Es gibt kein CI-Gate, das `cargo test --no-run --workspace` erzwingt, sonst wäre der Merge nicht durchgegangen.

### 3.2 Purge-Kern

- **N1 (K), R:** Resume im Produktionspfad führt keinen Purge aus (Abschnitt 5, WP-3/WP-4).
- **N2 (K), L:** Scope wird nie rekonstruiert, `scope_hash` nie geprüft.
- **N13 (H), L:** Fehlertaxonomie: `PurgeBlocked` vermischt Vorbedingungen („blockiert, später wiederholbar“) mit harten Fehlern (`ReadError`, `WriteError`, `ManifestError`). Der Orchestrator macht daraus wiederum `OrxPurgeError::Io(...)` mit Zeichenketten-Suche nach `"Failpoint triggered:"` (`PurgeOutcome::into_result`). Fehlerklassifikation per `contains()` auf Nachrichtentext ist technische Schuld.
- **N14 (M), L:** `PurgePhase` hat 16 Varianten, davon 8 „Legacy“-Doppelgänger mit gleicher `order()`. `PartialEq` (abgeleitet) und `Ord` (über `order()`) sind widersprüchlich: `Admitted != P1Intent`, aber `cmp == Equal`. Das verletzt den `Ord`-Vertrag. Die Orchestrator-Kommentare nennen „P1..P8“, der Code verwendet gemischt beide Namensräume.
- **N15 (M), L:** `purge_keys` sammelt alle überlebenden Einträge einer SSTable in einem `Vec` (`ops/purge.rs:134`). Speicherbedarf = Größe der SSTable. Verstößt gegen die Zusage eines Streaming-Purge (Test `purge_streaming_memory.rs` existiert; ob er das wirklich prüft, ist H).
- **N16 (M), L:** `purge_keys` räumt `sst-purge-*.tmp` nach Fehlern nicht auf. Die Datei enthält nur die *überlebenden* Einträge, nicht die gelöschten; sie ist also kein Leck für diesen Purge. Sie wird aber zum Leck, sobald ein dort enthaltener Überlebender später selbst gelöscht wird, falls der Orphan-GC `.tmp`-Dateien nicht erfasst (H, Test in WP-5). Zusätzlich verwirft `let _ = fsync_parent_dir(...)` (`ops/purge.rs:213`) den Fehler; ohne Verzeichnis-Sync ist der `rename` nicht dauerhaft, der Manifest-Tausch kann dann auf eine Datei zeigen, die nach einem Absturz fehlt.
- **N17 (M), H:** Nach dem Purge liegen Blöcke der alten SSTable möglicherweise weiter im `block_cache` (RAM) und die Datei ist ggf. noch von Lesern offen/gemappt. Zu prüfen (WP-6).
- **N20 (H), L:** Alle Purges teilen sich **eine** Intent-Datei (`storage_dir/purge.intent`, `orchestrator.rs`). Zwei gleichzeitige Purges mit disjunkten Präfixen (die Fence erlaubt das) überschreiben sich per `atomic_replace` gegenseitig das Intent; nach einem Absturz kennt der Resume nur noch den letzten. Außerdem ist `purge_id = blake3(scope_hash ‖ as_of_tx)[..16]` deterministisch: zwei Löschungen desselben Scopes im selben `as_of_tx` erhalten dieselbe ID.
- **Resume-Verfügbarkeit (M), L:** Schlägt der Resume beim Öffnen fehl, gibt `open` einen Fehler zurück (`recovery.rs:566-571`). Eine nicht scrubbare Datei (`RequireScrub`) macht die Datenbank damit **dauerhaft unöffenbar**. Fail-closed ist richtig, aber es fehlt der definierte Betreiberpfad.

### 3.3 Fence

- **N8 (K), L:** TOCTOU im Commit-Pfad. `write.rs:372-390` prüft `is_active()` und `check_write_allowed_scope`, **danach** (`write.rs:408` bzw. `781`) wird erst `InflightGuard::new` erzeugt. Ablauf: Writer sieht `active == false` → Purge installiert Fence, liest `inflight == 0`, Drain fertig → Purge flusht und schreibt SSTables um → Writer erhöht `inflight` und schreibt einen Schlüssel aus dem gefencten Präfix. Der Schreibvorgang läuft **nach** dem Purge und überlebt ihn.
- **N9 (K), L:** `acquire_fence` setzt `active` mit `Release` und liest `inflight` mit `Acquire`; der Writer erhöht `inflight` mit `SeqCst`, liest `active` mit `Acquire`. Das ist ein Store-Load-Muster (Dekker). Ohne `SeqCst` auf beiden Seiten kann auf schwach geordneten Architekturen (ARM) jede Seite die Schreibaktion der anderen übersehen.
- **N18 (M), L:** Im Commit-Pfad prüft die Fence-Schleife nur `IndexOp::Insert` und `IndexOp::Delete{data: Some}`; alles andere läuft per `_ => continue` durch. Sobald eine neue `IndexOp`-Variante Schlüssel trägt, wird sie ungeprüft durchgelassen.
- **N19 (N), L:** `PurgeScope::Keys` wird als *Präfix*-Fence eingetragen (`orchestrator.rs`: `PurgeScope::Keys(keys) => keys.clone()`). Der Schlüssel `doc:1` sperrt damit auch `doc:10`. Sicher, aber überbreit; muss dokumentiert oder durch Exakt-Fence ersetzt werden.

### 3.4 WAL, Ledger, Restore

- **N10 (K), L:** `retire_segments_up_to` und `execute_scope_purge` haben **keinen** Produktionsaufrufer; `PurgeDriver::run_physical` ruft `execute_purge(..., None)` (kein WAL-Callback), `OrxHooks::default()` hat `wal_retire: None`. Der Raw-Sweep-Test bleibt grün, weil `flush` im Normalfall WAL-Segmente entsorgt (C, nur Standardfall). Segmente der Klassen `Archived`/`Backup`/`Sealed` sind ungeprüft (H).
- **N11 (K), L:** `LsmStorage::ledger_append` wird außerhalb der Definition nirgends aufgerufen. Das Anti-Resurrection-Ledger bleibt leer. `ErasureMode::Assured` („Ledger append before tombstone“) ist dadurch nicht implementiert.
- **N12 (H), L:** WAL-002 bestätigt: `recovery.rs:387-405` setzt `check_hmac = expected_hwm`, sobald das Manifest eine SSTable kennt, und vergleicht dann `expected_hwm` mit sich selbst. Ein abgeschnittenes WAL-Ende (inkl. **Löschmarker**) wird nicht erkannt. Für die Löschung heißt das: gelöschte Daten können durch Abschneiden eines Tombstones wieder sichtbar werden.
- **ERA-001 (M), L:** drei `let _ = remove_file(...)` in `wal/scope_purge.rs:417,435,440`.
- **N22 (M), L:** `DpiRestoreFilter` (Engine) und `StreamingRestoreGuard` (Store) halten jetzt zwei `Option`-Felder (`ledger`, `ledger_handle`), von denen genau eines gesetzt sein muss. Sind beide `None`, liefert `DpiRestoreFilter::should_drop` `false` (**fail-open**: nichts wird gefiltert). Über die Konstruktoren ist der Zustand heute nicht erreichbar, aber der Typ erlaubt ihn. Der unerlaubte Zustand muss unrepräsentierbar sein (`enum LedgerSource { Direct(Arc<ErasureLedger>), Handle(Arc<ErasureLedgerHandle>) }`).
- **N5 (M), L:** Restore-Filter wird nicht verdrahtet. Präzisierung: `PersistentCheckpointStore` wird außerhalb von Tests und Benches nirgends konstruiert (Grep über `crates/` und `src/`), das gesamte Checkpoint-/Restore-Subsystem ist also im Produktionspfad unverdrahtet, nicht nur der Filter. Das Subsystem selbst ist fail-closed (`checkpoint/store.rs`: `has_deletion_history && restore_filter.is_none()` → Fehler). Heute entsteht daraus kein Auferstehungsrisiko, aber jede künftige Verdrahtung ohne Filter wäre ein stiller Rückschritt.

### 3.5 Nachweis und Zeugen

- **E6 (H), L:** `LayerWitness::issue` ist `#[doc(hidden)] pub`. Produktionsaufrufer: `forget.rs:234`, `deletion/text_witness.rs:57`, `deletion/relations_witness.rs:53`, Vector `witness.rs`. `issue_sealed(&WitnessIssuerToken)` ruft dieselbe Funktion; das Siegel schützt nichts, solange `issue` öffentlich ist.
- **E2 (H), L:** `bincode` in `deletion_proof/*`, `audit_chain.rs` und **handgebaut in `collections.rs`** für den signierten Payload. `bincode` ist kein kanonisches Format (Integer-Kodierung konfigurationsabhängig, Enum-Varianten-Index positionsabhängig).
- **Drop-Pfad D1 bis D6 (H), L**, `collections.rs`:
  - **D1 (R):** `timestamp: 0` (`:464`), keine `Clock`. Repro `zz_repro_drop_path.rs` über die Fassade: `proof.timestamp=0`, `covered=[LsmMemtable]`, `excluded=[]`. Der Beweis trägt kein Datum.
  - **D2:** Der Beweis wird mit handgebautem Payload und `signature_version: 4` signiert, **bevor** der physische Purge läuft. Der Payload entspricht nicht dem V4-TLV-Format aus `contextra-crypto` (A: K_ENG-002 meldet Ablehnung durch `tools/contextra-verifier`). Ein externer Verifizierer kann den Beweis voraussichtlich nicht prüfen.
  - **D3 (L):** `PhysicalPurgeStatus::Performed { records_purged: deleted_keys.len(), bytes_reclaimed: 0 }` (`:585`). Die Zahl stammt nicht aus dem `PurgeReport`, sie ist **erfunden**. (Im Repro wird `Performed` nie erreicht, siehe N23; die Zahl ist trotzdem im Code.)
  - **N23 (K), R:** **Jede Standard-Löschung einer Collection meldet „Physical purge NotPerformed“, obwohl der Purge gelaufen ist.** Repro: Positivkontrolle `positive_control_clean=false` (Marker auf Platte), `drop_collection_with_report` → `NotPerformed { reason: "... Failed to issue witnesses_v2: ... claim S2V requires assurance >= S2, got S1" }`, danach Rohsweep **sauber**. Ursache: `PurgeDriver::run_physical` setzt `claim: WitClaimText::S2V` fest (`purge_driver.rs:241`), `ForgetOptions::default().assurance` ist aber `S1` (`forget_options.rs:80`). Die Ausstellung des Zeugen wird nach dem Purge abgelehnt, der Fehler überschreibt das Ergebnis, und der Phase-B-Beweis entfällt. Zwei Aussagen sind damit falsch: der Report (untertreibt: Daten sind weg) und der Fehlertext (nennt eine „Ausführungsstörung“, wo ein Konfigurationswiderspruch vorliegt). Der bestehende Test `drop_collection_layer_claims.rs` erwartet ausdrücklich `NotPerformed` und **bestätigt damit den Fehler als Soll**. Dieser Test muss mit WP-10 umgeschrieben werden.
  - **D4:** `collection_id` = erste 8 Byte von `blake3(name)` ohne Tenant (`:448`). Zwei Mandanten mit gleichem Collection-Namen erhalten dieselbe ID.
  - **D5:** `tenant_prefix` für den Reverse-Sweep ist `tenant_id.to_le_bytes()` (`:515`), der Purge-Scope dagegen ASCII `t:{id}:`. Zwei Kodierungen für dieselbe Sache; der Sweep sucht vermutlich nach dem falschen Muster (H).
  - **D6:** `self.collections.write().remove(name)` läuft **vor** dem Löschen; schlägt ein späterer Schritt fehl, ist die Collection im Speicher entfernt, im Speicher der Platte aber teilweise vorhanden. Kein Rollback.
  - Abgedeckte Schichten: `covered_layers: [LsmMemtable]`, `excluded_scopes: []`. Vektor-, Text- und Graph-Schichten der Collection werden weder gelöscht noch als ausgeschlossen deklariert. Ein Beweis mit leerer Ausschlussliste behauptet implizit Vollständigkeit.
  - Instanz-ID: Ledger-ID oder ein Hash des Pfads. Ein Pfad-Hash macht Beweise nach dem Verschieben des Verzeichnisses unzuordenbar.

---

## 4. Zielbild: Invarianten und Zustandsmaschine

### 4.1 Invarianten (jede wird durch mindestens einen benannten Test erzwungen, Abschnitt 6)

| ID | Invariante |
|---|---|
| **INV-ERA-1** Keine Falschaussage | Jede Aussage in Report, Beweis, Fehlertext oder Doku über Löschung folgt aus einem getypten Zustand. Es gibt keine frei formulierten „reason“-Strings, die Zustände behaupten. |
| **INV-ERA-2** Fence-Vollständigkeit | Ein Schreibvorgang auf einen Schlüssel im gefencten Scope wird entweder abgewiesen oder ist vollständig abgeschlossen, **bevor** der Purge seinen Memtable-Flush beginnt. Ein Dritter existiert nicht. |
| **INV-ERA-3** Purge-Totalität | Nach Phase `Finalizing` enthält keine Datei und kein Puffer des Datenverzeichnisses Bytes von Schlüsseln oder Werten des Scopes: Live-SSTables, obsolete SSTables, `.tmp`, WAL (alle Klassen), Journal, Intent, Scope-Datei, Block-Cache. Ausnahme sind dokumentierte Restrisiken (SSD-Remanenz, Backups, Swap), die im Bericht als getypte `ResidualRisk`-Einträge stehen. |
| **INV-ERA-4** Crash-Idempotenz | Für jeden Absturzpunkt gilt: Neustart + Resume führt zum selben Endzustand wie ein ungestörter Lauf. Resume meldet **nie** „erledigt“, ohne die Phasenaktion tatsächlich ausgeführt oder positiv verifiziert zu haben. |
| **INV-ERA-5** Fail-closed | Unbekannt, mehrdeutig oder nicht rekonstruierbar ist `Err` oder `Pending`, niemals `Ok`. |
| **INV-ERA-6** Nachweis nur aus Messung | Ein Zeuge entsteht nur aus einem Scan-Ergebnis mit `matches_found == 0`; ein Beweis nur aus einer Menge verifizierter Zeugen. Beides ist auf Typebene (Sichtbarkeit) und per Test erzwungen. |
| **INV-ERA-7** Ein Pfad | `forget`, `forget_collection`, `forget_tenant` und `drop_collection` laufen durch **denselben** Koordinator. Es gibt keinen zweiten Code-Pfad mit anderer Garantie (L13 aus dem Audit). |
| **INV-ERA-8** Single-Flight | Zu jedem Zeitpunkt läuft höchstens ein Purge-Intent je Datenbank. |

### 4.2 Phasenmodell des Purge (Soll)

`phase` im Intent bedeutet **zuletzt betretene Phase**. Beim Resume wird diese Phase **erneut ausgeführt**; jede Phasenaktion ist idempotent. Die numerischen Codes bleiben mit dem bestehenden Intent-Format v1 identisch (`order()` liefert 0 bis 9), damit alte Intents lesbar bleiben.

| Code | Phase (neu) | Aktion | Idempotenz-Beleg | Crash-Zustand → Resume |
|---|---|---|---|---|
| 1 | `Admitted` | Scope-Datei sealed schreiben (fsync + Dir-Sync), dann Intent v2 | Scope-Datei per `atomic_replace`, gleicher Inhalt | nur Scope-Datei, kein Intent → Orphan-GC scrubbt sie; Intent da → weiter |
| 2 | `Fenced` | Fence installieren, Drain | `acquire` für gleiche `purge_id` ist wiederholbar | Fence ist RAM-Zustand, nach Neustart weg; wird neu installiert |
| 3 | `Flushed` | Memtable flushen | Flush leerer Memtable ist No-op | – |
| 4 | `Rewriting` | `purge_keys`: Rewrite, Journal, Manifest-Tausch | Zweiter Lauf findet keinen Treffer; Obsolet-Liste liegt **schon im Journal** | Crash vor Manifest-Tausch: Journal enthält Dateien, die noch live sind → siehe WP-5 Regel J1 |
| 5 | `Disposing` | Journal abarbeiten: Reader-Drain, Scrub, Remove | `journal.process` überspringt fehlende Dateien | Journal bleibt bis zur Entfernung |
| 6 | `WalRetiring` | WAL aller Klassen entsorgen | wie `retire` | – |
| 7 | `Witnessing` | Scans, Zeugen, Ledger-Abschluss | Scans sind reine Lesevorgänge | – |
| 8 | `Finalizing` | Beweis ausstellen | gleiche Eingaben → gleiche Beweis-ID | – |
| 9 | `Released` | Fence frei, Scope-Datei scrubben+entfernen, dann Intent entfernen | Reihenfolge: **erst** Scope-Datei, **dann** Intent | Intent da, Scope-Datei weg → Resume beendet nur noch Release |

---

## 5. Arbeitspakete

Jedes Paket nennt: Ziel, betroffene Stellen, Soll-Signaturen, Ablauf, Fehlerverhalten, Tests, Akzeptanz, Abhängigkeiten. Die Reihenfolge steht in Abschnitt 7.

### WP-0 Baubarkeit und Gate (Voraussetzung für alles)

**Ziel:** `cargo test --workspace --no-run` und `cargo test -p contextra-store -p contextra-engine` laufen am Hauptzweig grün; ein CI-Gate verhindert den Rückfall.

**Schritte**
1. `engine.rs`: zweite Definition von `erasure_fence` und den falsch platzierten Doc-Kommentar entfernen. `ops/write.rs`: den Aufruf `check_write_batch_allowed_scope` entfernen (die Prüfung steht direkt darüber). Beides ist als `0001-fix-build-break-store.patch` beigelegt. **Beachte:** WP-6 ersetzt diese Prüfstellen ohnehin; der Patch dient nur der Baubarkeit.
2. `purge_driver.rs:168`: kein Wildcard-Arm. Die Abbildung `PurgeBlocked → (Grund, Hinweis)` wandert als Methode **neben den Typ** (WP-1), damit der erschöpfende `match` im definierenden Crate steht.
3. Die sechs kaputten Testdateien aus 2.3 einzeln reparieren oder löschen. **Löschen nur, wenn ein Ersatztest aus Abschnitt 6 dieselbe Eigenschaft prüft.** Ein auskommentierter oder gelöschter Test ohne Ersatz ist technische Schuld.
4. CI-Gate (neu, in `xtask`): `cargo xtask gate-compile-all` führt `cargo check --workspace --all-targets` und `cargo test --workspace --no-run` aus und ist **required check** für Merges. Zusätzlich ein Merge-Queue-Regelwerk: Zwei PRs, die dieselbe Datei berühren (`fence.rs`, `write.rs`), werden gegeneinander neu gebaut (die Ursache von B0 war ein Merge zweier einzeln grüner PRs).
5. **Port-Konsolidierung (B0-d) vollenden:** `contextra-sys` implementiert die Port-Traits nicht, weil Ring 0 die Ports nicht kennen darf. Der Adapter (`impl Clock for SystemClock`, `impl SeededRng for SystemRng`) gehört in den **Composition Root** oder ein Ring-1-Crate, das beide sieht; alle ~30 Aufrufer werden **in demselben PR** umgestellt. Bis dahin gilt: kein Merge, der `cargo check --workspace --all-targets` bricht.
6. `cargo xtask check-ring-layering-full` (ERA-004, Klasse A) prüfen und beheben, falls es am HEAD noch fehlschlägt.

**Akzeptanz:** Gate rot am heutigen HEAD, grün nach den Schritten. Das Gate hat einen Selbsttest: ein absichtlich kaputter Commit auf einem Testzweig muss es rot machen.

### WP-1 Fehlertaxonomie des Purge

**Ziel:** Vorbedingung („nicht gestartet, wiederholbar“) und harter Fehler („teilweise ausgeführt, Resume nötig“) sind verschiedene Typen. Kein Fehlerklassifizieren per Textsuche.

**Soll-Typen** (`contextra-store/src/compaction/purge_filter.rs` und `lsm/purge/failpoint.rs`)

```rust
/// Vorbedingung nicht erfüllt. Es wurde nichts verändert; Wiederholung ist zulässig.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PurgeBlocked {
    #[error("active MVCC snapshot pins seq {min_seq}")]
    ActiveSnapshot { min_seq: u64 },
    #[error("checkpoint clones present: {copy_ids:?}")]
    CheckpointClones { copy_ids: Vec<String> },
    #[error("reader pinned")]
    ReaderPinned,
    #[error("compaction busy")]
    CompactionBusy,
    #[error("another purge is active")]
    PurgeInProgress,
    #[error("insufficient disk space: need {needed}, free {free}")]
    InsufficientDiskSpace { needed: u64, free: u64 },
    #[error("unclassified files in data dir: {count}")]
    UnclassifiedFiles { count: usize },
    #[error("secure delete unsupported: {reason}")]
    SecureDeleteUnsupported { reason: String },
    #[error("free space unknown")]
    UnknownFreeSpace,
}

impl PurgeBlocked {
    /// Genau ein erschöpfender `match`; neue Variante ⇒ Compilerfehler HIER, nicht im Driver.
    pub fn retry_hint(&self) -> &'static str { /* je Variante */ }
}

/// Harter Fehler nach Beginn der Veränderung. Zustand ist über Intent und Journal wiederaufnehmbar.
#[derive(Debug, thiserror::Error)]
pub enum PurgeError {
    #[error(transparent)] Blocked(#[from] PurgeBlocked),
    #[error("read {file}@{offset}: {source}")]
    Read { file: PathBuf, offset: u64, #[source] source: std::io::Error },
    #[error("write {file}: {source}")]
    Write { file: PathBuf, #[source] source: std::io::Error },
    #[error("manifest: {0}")] Manifest(#[source] ContextraError),
    #[error("journal: {0}")]  Journal(#[source] std::io::Error),
    #[error("fence: {0}")]    Fence(#[from] FenceError),
    #[error("intent: {0}")]   Intent(#[source] std::io::Error),
    #[error("scope unavailable for purge {purge_id:?}")]
    ScopeUnavailable { purge_id: [u8; 16] },
    #[error("scope hash mismatch for purge {purge_id:?}")]
    ScopeMismatch { purge_id: [u8; 16] },
    #[error("dispose refused: {0}")] DisposeRefused(String),
    #[cfg(any(test, feature = "fault-injection"))]
    #[error("failpoint {0}")] Failpoint(&'static str),
}
```

**Regeln**
- **Kein `#[non_exhaustive]`** auf `PurgeBlocked`. Es würde in fremden Crates Wildcard-Arme erzwingen und genau den Fehler B0-b unsichtbar machen.
- `PurgeBlocked::Unimplemented` und `ScanTimeout` entfernen, wenn kein Produktionscode sie erzeugt (Grep am HEAD: nur Tests referenzieren `Unimplemented`). Die zwei Tests, die es erwarten, sind ohnehin veraltet (2.3).
- `PurgeOutcome::Failed { phase, error_msg: String }` wird `Failed { phase: PurgePhase, error: PurgeError }`. `into_result` verliert die Textsuche nach `"Failpoint triggered:"`.
- `purge_keys` liefert `Result<PurgeReport, PurgeError>`. Das `map_err(|_| PurgeBlocked::CompactionBusy)` bei Flush-Fehlern (heute `ops/purge.rs`, Schritt 2) entfällt: ein Flush-Fehler ist `PurgeError::Write`, nicht „Compaction busy“ (L12 aus dem Audit).
- `PurgeDriver` bildet `PurgeError::Blocked(b)` auf `PurgeHandle::Blocked { reason: b.to_string(), retry_hint: b.retry_hint() }` ab. **Alle anderen** `PurgeError` werden als `Err` propagiert, nicht als `Blocked` getarnt. Heute landen I/O-Fehler als „Blocked“; ein Aufrufer, der auf `Blocked` wiederholt, würde einen defekten Zustand endlos wiederholen.

**Tests:** `purge_error_classification` (jede `PurgeBlocked`-Variante → `retry_hint` nicht leer, getrennt über `strum::EnumIter`); `driver_propagates_hard_errors` (Fault-Injection `Write` → `Err`, nicht `Blocked`).
**Akzeptanz:** `rg 'contains\("Failpoint' crates/` liefert nichts; `rg 'PurgeBlocked::CompactionBusy' crates/contextra-store/src/lsm/ops/purge.rs` liefert nur echte Busy-Fälle.

### WP-2 Eine `PurgePhase`

**Ziel:** Eine Enum, kein Legacy-Zwilling, `Ord` konsistent mit `Eq`.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PurgePhase {
    Admitted = 1, Fenced = 2, Flushed = 3, Rewriting = 4,
    Disposing = 5, WalRetiring = 6, Witnessing = 7, Finalizing = 8, Released = 9,
}
impl TryFrom<u8> for PurgePhase { type Error = InvalidPhaseCode; /* 1..=9 */ }
```

- **Code 0** (`P0Init`) wird nie persistiert (`execute_purge_outcome` schreibt als erstes `P1Intent`) und entfällt. Ein gelesener Code 0 ist `InvalidData`.
- `Serialize/Deserialize` entfällt (die Persistenz ist die u8-Kodierung des Intents). Kein zweites Wire-Format.
- `PartialOrd/Ord` werden **abgeleitet**; da die Diskriminanten die Ordnung festlegen, kann kein Widerspruch mehr entstehen.
- Alle `intent.phase <= PurgePhase::X`-Vergleiche im Orchestrator werden durch eine Methode ersetzt, die die Absicht benennt: `phase.must_run(PurgePhase::Fenced)` (= `phase <= Fenced`). Das beseitigt die Mischung aus `P3Flush`/`Flush` im selben `if`.
- Migration: Intent-Version bleibt 1 für Phasencodes; Version 2 kommt mit WP-3. Ein gelesener v1-Intent hat keine Scope-Referenz und führt zu `ScopeUnavailable` (WP-3).

**Tests:** `phase_codes_roundtrip` (alle 9), `phase_code_zero_rejected`, `phase_order_total` (Eigenschaftstest: `a < b` ⇔ `a as u8 < b as u8`).

### WP-3 Dauerhafter Scope und Single-Flight (behebt N2, N20)

**Entscheidung (Begründung am Ende des Pakets):** Der Scope wird **verschlüsselt in einer eigenen Datei** neben dem Intent gespeichert. Alternative (Rekonstruktion aus dem Ledger) siehe „Verworfen“.

**Typen und Format**

```rust
// lsm/purge/scope_store.rs (neu)
pub struct PurgeScopeStore { dir: PathBuf, sealer: Arc<dyn ScopeSealer> }

pub trait ScopeSealer: Send + Sync {
    /// AEAD; `aad` bindet an purge_id und scope_hash.
    fn seal(&self, aad: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, ScopeSealError>;
    fn open(&self, aad: &[u8], sealed: &[u8]) -> Result<zeroize::Zeroizing<Vec<u8>>, ScopeSealError>;
}

impl PurgeScope {
    /// Kanonische Bytes: Tag "KEYS:" | "PREFIXES:", dann je Eintrag u64-LE-Länge + Bytes.
    /// Identisch zur heutigen Hash-Eingabe in `compute_scope_hash`.
    pub fn to_canonical_bytes(&self) -> Vec<u8>;
    pub fn from_canonical_bytes(b: &[u8]) -> Result<Self, ScopeDecodeError>; // strikt: keine Resttbytes
    pub fn hash(&self) -> [u8; 32] { *blake3::hash(&self.to_canonical_bytes()).as_bytes() }
}
```

- Dateiname `purge-<purge_id hex>.scope`. Inhalt: Magic `ORXS`, Version, Nonce, Ciphertext. `aad = purge_id ‖ scope_hash`.
- Schlüssel: Unterschlüssel des Master-Schlüssels (`blake3::derive_key("contextra/purge-scope/v1", master)`, analog zu `LedgerKey::derive_from_master_secret`). Bei **unverschlüsselten** Datenbanken (kein Master-Schlüssel): `PlainSealer` mit CRC statt AEAD; die Datei enthält dann nichts, was nicht ohnehin im Klartext in den SSTables stünde. Sie wird trotzdem gescrubbt.
- **Schreibreihenfolge bei `Admitted`:** (1) Scope-Datei schreiben, fsync Datei **und** Verzeichnis, (2) Intent v2 mit `purge_id` schreiben. Absturz zwischen (1) und (2): verwaiste Scope-Datei ohne Intent; der Orphan-GC beim Öffnen scrubbt und entfernt jede `purge-*.scope` ohne zugehöriges Intent.
- **Release-Reihenfolge:** erst Scope-Datei scrubben und entfernen, dann Intent entfernen (Tabelle 4.2). Ein Intent bei Phase `Released` ohne Scope-Datei ist gültig.
- **Intent v2:** Format-Version 2; zusätzlich das Feld `scope_present: bool`. Ein v1-Intent hat keinen Scope. Beim Resume eines v1-Intents mit Phase < `Released` → `PurgeError::ScopeUnavailable`. Es wird **nicht** geraten und **nicht** leer weitergemacht.
- **Hash-Prüfung (schließt N2):** Nach `open` und Dekodierung wird `scope.hash() == intent.scope_hash` mit konstanter Zeit verglichen, sonst `ScopeMismatch`.
- **Single-Flight (N20):** `LsmStorage` bekommt `purge_slot: tokio::sync::Mutex<()>`. `execute_purge` nimmt `try_lock_owned` mit kurzem Timeout; schlägt es fehl: `PurgeBlocked::PurgeInProgress`. Damit genügt weiter **eine** Intent-Datei. Die `purge_id` wird `blake3(scope_hash ‖ as_of_tx ‖ nonce128)[..16]`, `nonce128` aus einem injizierten `IdSource`-Port (deterministisch in Tests). So kollidieren zwei Löschungen desselben Scopes nicht.

**Betreiberpfad bei nicht lösbarem Resume (offene Entscheidung E-1, Abschnitt 8):** `open` bleibt fail-closed. Vorschlag: `contextra-admin purge abandon --purge-id <hex> --reason <text>` schreibt ein signiertes Audit-Ereignis „Purge abandoniert, Löschversprechen für diesen Scope nicht erfüllt“ und entfernt Intent und Scope-Datei. Ohne diesen Pfad ist die Datenbank bei dauerhaft nicht scrubbaren Dateien unöffenbar.

**Tests**
- `scope_roundtrip` (Eigenschaftstest, 10 000 Fälle, leere Schlüssel, 0-Byte-Präfix, Schlüssel mit eingebetteten `0x00`)
- `scope_tamper_rejected` (jedes geflippte Byte → `ScopeMismatch` oder AEAD-Fehler)
- `scope_trailing_bytes_rejected`
- `legacy_v1_intent_not_silently_completed` (v1-Intent in Phase 4 → `open` liefert `ScopeUnavailable`, Intent bleibt erhalten, **kein** `Ok`)
- `concurrent_purges_single_flight` (zwei Purges, disjunkte Präfixe → einer `PurgeInProgress`)
- `orphan_scope_file_scrubbed_on_open`

**Verworfen:** Scope aus dem Ledger rekonstruieren. Das Ledger speichert nur *Keyed Hashes* (`erasure_ledger/entry.rs`, `compute_keyed_hash`), nicht die Schlüssel. Daraus lässt sich ein Scope für `matches_key` nicht zurückgewinnen. Das ist datenschutzfreundlich, schließt die Rekonstruktion aber aus.

### WP-4 Resume führt den Purge wirklich aus (behebt N1)

**Ziel:** Es gibt keinen Codepfad, in dem ein Orchestrator ohne Storage einen Purge „abschließt“.

**Struktur**
1. `PurgeOrchestrator` wird generisch über ein Backend und verliert `storage: Option<&LsmStorage>`:

```rust
#[async_trait::async_trait]
pub trait PurgeBackend: Send + Sync {
    async fn acquire_fence(&self, purge_id: u64, scope: &PurgeScope) -> Result<FenceHandle, PurgeError>;
    async fn flush_memtable(&self) -> Result<(), PurgeError>;
    async fn purge_keys(&self, fence: &FenceHandle, scope: &PurgeScope, as_of_tx: u64,
                        probe: Option<&dyn FreeSpaceProbe>) -> Result<PurgeReport, PurgeError>;
    async fn dispose(&self, policy: DisposePolicy) -> Result<OrxDisposeReport, PurgeError>;
    async fn retire_wal(&self) -> Result<WalRetireReport, PurgeError>;
}
pub struct PurgeOrchestrator<B: PurgeBackend> { backend: B, /* intent, failpoints */ }
```
   `LsmStorage` implementiert `PurgeBackend`. Tests nutzen `MockBackend` (nur `#[cfg(any(test, feature = "testing"))]`). `new_standalone` entfällt; der Test `purge_orchestrator_resume.rs` wird auf `MockBackend` umgestellt und prüft zusätzlich, dass `purge_keys` **aufgerufen wurde** (Zähler im Mock).
2. `OrxHooks` (sechs `Option<Box<dyn Fn>>`-Felder, alle in Produktion `None`) entfällt. Die Hooks sind Methoden des Backends. `Option`-Hooks, die in der Produktion nie gesetzt werden, sind der Mechanismus, der N1 und N10 verursacht hat: Ein vergessener Hook ist ein stiller Nichtstun-Pfad.
3. **Resume wandert aus `open_internal` heraus, bis `Self` existiert.** Heute läuft er in `recovery.rs:564-565`, wo noch keine `LsmStorage`-Instanz vorliegt. Neu:

```rust
pub(super) async fn open_internal(config: LsmConfig, ..) -> Result<Self> {
    /* Manifest, WAL-Replay, SSTables, Memtable aufbauen */
    let storage = Self { /* alle Felder */ };
    storage.recover_pending_purge().await?;   // NEU: Backend = &storage
    storage.run_orphan_gc().await?;           // NACH dem Resume
    Ok(storage)
}

async fn recover_pending_purge(&self) -> Result<(), ContextraError> {
    match OrxPurgeIntent::read_from_file(&self.intent_path())? {
        Some(intent) => {
            let scope = self.scope_store.load_verified(&intent)?;   // WP-3
            PurgeOrchestrator::new(self).resume(intent, scope).await
                .map(|_| ()).map_err(ContextraError::from)
        }
        None => self.dispose_pending_journal().await, // heutiger Zweig "Intent weg, Journal da"
    }
}
```
   Die Reihenfolge Resume → Orphan-GC ist Pflicht: Der Orphan-GC darf Dateien, die im `pending_delete.journal` stehen, nicht als Waisen behandeln.
4. **Phasenregeln beim Resume** (Tabelle 4.2): Phase 2 erneute Fence, Phase 3 erneuter Flush, Phase 4 erneuter `purge_keys`, danach unverändert. `purge_keys` im Wiederholungsfall findet keine Treffer und liefert eine leere Obsolet-Liste; die Dateien stehen schon im Journal (WP-5, Regel J1), Phase 5 verarbeitet sie.
5. **Beweispflicht des Resume:** Vor `Released` führt der Orchestrator eine **Verifikation** aus: ein Streaming-Scan über alle Live-SSTables auf `scope.matches_key`. Treffer ⇒ `PurgeError::Read`-freier Fehler `PurgeError::ResidueFound { file }` (neue Variante), Intent bleibt. Das ist die Absicherung dagegen, dass ein künftiger Fehler wieder „erledigt“ meldet, ohne gelöscht zu haben (INV-ERA-4). Kosten: ein weiterer Lesedurchlauf je Purge; messen (WP-12).

**Fehlerverhalten:** Resume-Fehler ⇒ `open` liefert `Err` (fail-closed, bestehendes Verhalten); `tracing::error!` mit `purge_id` und Phase. Die Entscheidung E-1 regelt den Betreiberausweg.

**Tests (zwingend, Reihenfolge der Entstehung)**
1. `zz_repro_n1_resume_noop.rs` (liegt bei; **rot** am HEAD, grün nach WP-3/WP-4).
2. `resume_matrix_all_phases`: für jede Phase 1 bis 8 und jeden Failpoint aus `failpoint.rs`: Storage mit Marker-Daten aufbauen, Crash an Failpoint, `drop`, neu öffnen, **Rohsweep** über das gesamte Datenverzeichnis (`contextra_testkit::erasure_harness::raw_sweep`) auf Marker; zusätzlich: Überlebende lesbar, `purge.intent`/Scope-Datei/Journal weg. Positive Kontrolle vor dem Purge (Marker vorhanden) ist Pflicht (Muster aus `purge_obsolete_raw_sweep.rs`).
3. `resume_with_mock_backend_calls_purge_keys` (Zähler > 0).
4. `resume_detects_residue_and_keeps_intent` (Backend, das `purge_keys` still nichts tut → `ResidueFound`, Intent bleibt).
5. `purge_crash_matrix_dst` repariert (WP-0) und auf die neue Phasen-Enum umgestellt.

**Akzeptanz:** Test 1 und 2 grün; `rg 'new_standalone' crates/` leer; `rg 'Option<HookCallback>' crates/` leer.

### WP-5 `purge_keys`: Rewrite, Journal, Manifest, RAM-Swap

**Ziel:** Streaming, fehlerfrei propagierend, absturzsicher, ohne Rest im RAM.

**Regeln**
- **J1 (Journal-Reihenfolge):** Obsolete Dateien werden **vor** dem Manifest-Tausch ins Journal geschrieben (am HEAD erfüllt, bleibt Pflicht). Folge: Das Journal kann Dateien enthalten, die noch **live** sind (Absturz zwischen Journal-Eintrag und Manifest-Tausch). `dispose` darf eine Journal-Datei deshalb nur entfernen, wenn sie **nicht** im Manifest steht. Heute prüft `dispose.rs` das nicht (es scrubbt/entfernt jede Journal-Datei bedingungslos). Neuer Schritt in `dispose_obsolete_files_with_policy`: Live-Menge aus dem Manifest lesen; Journal-Pfade in der Live-Menge werden **nicht** entsorgt, sondern aus dem Journal entfernt (der Purge hat den Tausch nie vollzogen, Phase 4 läuft erneut). Ohne diese Regel zerstört ein Resume nach Absturz zwischen Journal und Manifest **lebende Daten**. Test `journal_entry_for_live_file_is_not_disposed`.
- **Streaming (N15):** Statt `valid_entries: Vec<_>` wird der `SstableBuilder` beim ersten Überlebenden **lazy** erzeugt und Eintrag für Eintrag gefüllt. Gibt es keinen Überlebenden, entfällt der Builder und die SSTable bekommt `ManifestEntry::Remove`. Speicherbedarf O(Block). Test `purge_streaming_memory` prüft das heute möglicherweise nicht; er bekommt eine echte Messung (Zähl-Allokator, SSTable > 64 MiB, Obergrenze Spitzen-Heap < 8 MiB).
- **Temp-Aufräumen (N16):** `sst-purge-*.tmp` wird bei jedem Fehlerpfad entfernt (RAII-Guard `TmpFile`, `Drop` scrubbt und entfernt, Fehler werden geloggt **und** im Rückgabewert gesammelt). Orphan-GC erfasst `.tmp`.
- **Verzeichnis-Sync:** `fsync_parent_dir(...)` propagiert den Fehler (`PurgeError::Write`). Sync **nach** `rename` und **vor** Manifest-Tausch.
- **Fence als Beweis im Typ:** `purge_keys(&self, fence: &FenceHandle, ...)` verlangt eine gehaltene Fence. Ein Aufruf ohne Fence ist nicht mehr kompilierbar. Damit entfällt die Annahme „der Orchestrator hat gefenct“.
- **Position im RAM-Swap (L6):** Bleibt wie am HEAD (positionserhaltend). Die Zuordnung „n-ter neuer Reader gehört zum n-ten Pfad mit Ersatz“ im aktuellen Code ist **fragil**: Sie passt nur, wenn `new_readers` und `replaced_old_paths` dieselbe Reihenfolge haben **und** leere SSTables (nur `Remove`, kein Reader) richtig übersprungen werden. Der Code füllt `replacement_map` über `reader_iter.next()` für jeden Pfad in `replaced_old_paths`, also auch für Pfade, die nur entfernt wurden. Ergebnis: Ein entfernter Pfad vor einem ersetzten Pfad **konsumiert den Reader des falschen Pfads**. **Das ist ein Logikfehler, reproduziert (R):** Test `zz_repro_wp5_swap_order.rs` (SSTs alt→neu: `A=[m1]`, `X=[k=old]`, `B=[k=new, m2]`, Scope `{m1,m2}`) liefert nach dem Purge `get(k) == "old"` statt `"new"`. Ursache: `A` wird leer (nur `Remove`, kein Reader), `B` bekommt einen Reader `RB`; der Iterator ordnet `RB` dem Pfad `A` zu und entfernt `B`. `RB` steht damit **vor** `X` und wird von `X` überdeckt. Folge: ein Löschvorgang macht **ältere, überschriebene Werte anderer Schlüssel wieder sichtbar**. Das ist ein Wiederauferstehungs-/Stale-Read-Fehler im Löschkern selbst. Der Test `purge_two_ssts_first_becomes_empty` ist der behobene Repro. Fix: `Vec<(PathBuf, Option<Arc<SstableReader>>)>` direkt beim Rewrite aufbauen, keine nachträgliche Zuordnung über Iterator-Parität.
- **Reader-Drain vor Scrub (N17):** Nach dem RAM-Swap bleiben alte `Arc<SstableReader>` in laufenden Lesern. `dispose` wartet auf `Arc::strong_count == 1` je alter Datei (max. N Sekunden, sonst `PurgeBlocked::ReaderPinned` und Journal bleibt). Scrub unter einem offenen Leser liefert diesem Nullen statt Daten.
- **Block-Cache:** Nach dem Swap werden alle Cache-Blöcke der alten Datei-IDs invalidiert (`block_cache.invalidate_file(id)`; falls die API fehlt, ist sie Teil dieses Pakets). Test `purge_invalidates_block_cache` (Marker lesen → Cache warm → Purge → Heap-Sweep, siehe WP-12).
- **Compaction-Ausschluss (L11, H):** Purge hält dasselbe Primitiv, mit dem Compaction ihre Auswahl-bis-Tausch-Phase serialisiert. Welches Primitiv das ist, habe ich nicht gelesen (Prüfauftrag: `compaction/engine.rs`, `lsm/ops/compaction.rs`). Test `purge_vs_compaction_race` (Loom oder deterministischer Scheduler).
- **`PurgeReport` ist die einzige Quelle für Zahlen.** `records_purged` und `bytes_reclaimed` kommen aus dem Rewrite (heute korrekt gezählt), nicht aus `deleted_keys.len()` (D3).
- **Disk-Probe:** `free_space_opt == None` bedeutet „unbekannt“; das ist am HEAD „erlaubt“ (L8-Fix). Für `Assured/Attested` ist „unbekannt“ **nicht** erlaubt: `UnknownFreeSpace` (Variante existiert). Ein Purge, der mitten im Rewrite an `ENOSPC` scheitert, hinterlässt Teilzustand. Entscheidung E-2 (Abschnitt 8).

**Tests:** `purge_two_ssts_first_becomes_empty`, `journal_entry_for_live_file_is_not_disposed`, `purge_tmp_cleanup_on_write_error`, `purge_fsync_dir_error_propagates`, `purge_streaming_memory` (echte Messung), `purge_invalidates_block_cache`, `purge_vs_compaction_race`, `purge_waits_for_pinned_reader`.

---

### WP-5a Sofortfix des Swap-Fehlers (unabhängig, zuerst nach WP-0)

Der in WP-5 beschriebene Zuordnungsfehler korrumpiert Lesesichten und hängt von keinem anderen Paket ab. Er wird als **eigener, kleiner PR** vorgezogen:

1. Roten Test committen: `zz_repro_wp5_swap_order.rs` umbenennen in `purge_two_ssts_first_becomes_empty` (Ort: `crates/contextra-store/tests/`).
2. Beim Rewrite pro SSTable direkt `(old_path, Option<Arc<SstableReader>>)` in einen `Vec` legen, in der Reihenfolge der Eingabeliste. Keine Zuordnung über `reader_iter.next()`.
3. RAM-Swap: ein einziger Durchlauf über `sst_guard`, der jede Position gegen eine `HashMap<PathBuf, Option<Arc<_>>>` nachschlägt (`Some(Some(r))` ersetzt, `Some(None)` entfernt, `None` behält).
4. Zusatztests (Eigenschaftstest): Zufällige Folge von 2 bis 8 SSTables, zufällig überlappende Schlüssel und Scope; Orakel ist ein `BTreeMap`-Modell der neuesten Version je Schlüssel ohne Scope-Schlüssel. Nach dem Purge muss `get` für **jeden** Nicht-Scope-Schlüssel dem Modell entsprechen.

### WP-6 Fence-Protokoll (behebt N8, N9, N18, N19)

**Ziel:** INV-ERA-2 gilt bewiesen (Argument) und getestet (Loom plus Stresstest). Die Performance des Normalbetriebs bleibt innerhalb eines gemessenen Budgets.

**Protokoll (Soll)**

```rust
impl ErasureFence {
    /// Autoritative Schreibschleuse. Reihenfolge ist Teil des Vertrags:
    /// 1) inflight erhöhen (SeqCst), 2) erst DANACH Aktivität und Tabelle prüfen.
    pub fn enter_write<'a, I>(&self, keys: I) -> Result<WriteTicket<'_>, FenceError>
    where I: IntoIterator<Item = FencedKey<'a>>;
}

#[must_use = "Ticket muss bis nach WAL-Append UND Memtable-Apply leben"]
pub struct WriteTicket<'f> { fence: &'f ErasureFence }
impl Drop for WriteTicket<'_> { fn drop(&mut self) { self.fence.gate.dec_inflight(); } }

pub enum FencedKey<'a> { Key(&'a [u8]), Unknown } // Unknown: Operation ohne Schlüsselbytes
```

`enter_write`:
1. `gate.inflight.fetch_add(1, SeqCst)`; Ticket sofort konstruieren, damit jeder Fehlerpfad per `Drop` wieder freigibt.
2. `if gate.active.load(SeqCst)`: je Schlüssel Scope-Zustand, Präfix-Tabelle (`ArcSwap::load`), Dokument-Sperren prüfen. `FencedKey::Unknown` wird bei aktiver Fence **abgewiesen** (fail-closed). Treffer erhöht `scope_write_seq` und liefert `Err(FenceError::…)`.
3. `Ok(ticket)`.

`acquire` (Purge-Seite):
1. Kurz `install`-Sperre (`parking_lot::Mutex`, **nie** über ein `.await` gehalten): Überlappungen prüfen, Tabelle per `ArcSwap::store` ersetzen, **danach** `active.store(true, SeqCst)`; Sperre loslassen.
2. Drain **außerhalb** der Sperre: `Notified`-Future zuerst anlegen und `enable()`, dann `inflight.load(SeqCst) == 0` prüfen, dann awaiten (kein verlorener Wecker, kein 2-ms-Polling). Timeout ⇒ Sperre kurz nehmen, `release_fence_internal`, `Err(FenceError::DrainTimeout)`.

**Korrektheitsargument (Dekker-Muster).** Writer: `W1: inflight+=1 (SeqCst)`, `W2: load active (SeqCst)`. Purge: `P1: store table`, `P2: store active=true (SeqCst)`, `P3: load inflight (SeqCst)`. Alle vier Operationen liegen in der einen totalen SeqCst-Ordnung. Fall (i) `W1 < P3`: Purge sieht den Writer (`inflight ≥ 1`), wartet, sein Schreibvorgang endet **vor** dem Drain-Ende und damit vor dem Flush. Fall (ii) `P3 < W1`: Dann gilt `P2 < P3 < W1 < W2`, der Writer sieht `active == true`, und `P1 < P2` stellt sicher, dass er auch die Tabelle sieht. Ein Writer, der die Fence übersieht und nach dem Drain schreibt, ist ausgeschlossen. (Heutiger Code: `W2 < W1`, also genau der ausgeschlossene Fall; plus `Release/Acquire` statt `SeqCst`.)

**Weitere Pflichten**
- **Authoritativ ist nur das Ticket.** Die Prüfungen in `put`, `delete_batch`, `delete_prefix` (heute `write.rs:31, 89, 154, 190`) sind frühe, **unverbindliche** Ablehnungen und werden als solche kommentiert. Der Commit-Pfad (`commit`, Gruppen-Commit-Leader) ruft `enter_write` mit **allen** Schlüsseln aller Transaktionen des Batches und hält das Ticket bis zum Ende der Memtable-Anwendung.
- **Erschöpfend (N18):** `fn fenced_keys(op: &IndexOp<…>) -> FencedKey<'_>` mit vollständigem `match` über alle Varianten, ohne `_`-Arm; `#![deny(clippy::wildcard_enum_match_arm)]` im Modul. `IndexOp::Delete { data: None }` ⇒ `FencedKey::Unknown`. (Prüfauftrag: Definition von `IndexOp` in `contextra-mvcc` lesen und je Variante entscheiden.)
- **Exakt vs. Präfix (N19):** `FenceTarget::{Exact(Vec<u8>), Prefix(Vec<u8>)}`. `PurgeScope::Keys` ⇒ `Exact`, `PurgeScope::Prefixes` ⇒ `Prefix`. Überlappungslogik pro Paar definiert.
- **Async statt Laufzeit-Bastelei:** `acquire_scope_fence_for` wird `async fn`. Der aktuelle Code erzeugt je nach Laufzeit `block_in_place` + `block_on` oder startet einen Thread mit **neuer** Tokio-Runtime (`fence.rs`, `acquire_scope_fence_for`). Das ist technische Schuld und ein Deadlock-Risiko. `FenFenceGuard::drop` bleibt synchron (Release ist ein kurzer Mutex-Abschnitt).
- **`FenceError`:** `CollectionDropping`, `Conflict`, `DrainTimeout`, `ScopeFenced` trennen; heute dient `Conflict` auch für „Runtime-Build fehlgeschlagen“ (`map_err(|_| FenceError::Conflict)`). Fehlerursachen nicht umdeuten.
- **Kosten:** Zwei atomare RMW (`fetch_add`, `fetch_sub`) je **Batch**, nicht je Schlüssel. Der „Zero-Lock-Fast-Path“ aus #5107/#5109 wird damit **nicht** zero-atomic, bleibt aber lock-frei. Budget und Messung: `benches/fence_hot_path.rs`; Annahme: Commit-Latenz-Mehrkosten < 1 %. Wird das verfehlt: Entscheidung E-3 (Abschnitt 8). Eine Variante ohne RMW (Epochenzähler mit Revalidierung nach dem Schreiben) ist **nicht** zulässig, weil ein bereits ausgeführter Schreibvorgang nicht zurückgenommen werden kann.

**Tests**
- `loom_fence_no_post_drain_write`: zwei Threads; Writer: `enter_write` → bei `Ok` schreibt in eine geteilte Zelle (`Cell` per `loom::cell::UnsafeCell`) → Drop. Purge: `acquire` → liest die Zelle als „Snapshot“ → setzt `drained`. Assertion: Hat der Writer ein Ticket erhalten, ist sein Schreibvorgang im Snapshot sichtbar. **Negativkontrolle:** dieselbe Modellierung mit der heutigen Reihenfolge (Prüfung vor `inc`) muss Loom rot färben. Ohne diese Kontrolle beweist der Test nichts.
- `fence_commit_vs_purge_stress`: 8 Writer, Zielpräfix = Scope, 20 000 Iterationen mit festem Seed; Assertion nach `purge_keys`: Rohsweep auf Marker sauber, und jeder Writer-Erfolg liegt vor dem Fence-Zeitpunkt (Sequenznummern).
- `ticket_released_on_every_error_path` (Zähler `inflight == 0` nach jedem Fehlerpfad im Commit).
- `fenced_keys_exhaustive` (Compile-Zeit-Schutz plus Test je Variante).
- `drain_timeout_releases_fence`, `no_lost_wakeup_drain` (Loom).
- Reparierte Testdateien aus WP-0: `erasure_fence_scope`, `purge_prefix_fence_concurrency` (nutzen ein öffentliches Test-Zugriffsobjekt statt des privaten Feldes `erasure_fence`).

**Akzeptanz:** Loom rot für alte Reihenfolge, grün für neue. `rg 'block_in_place|new_current_thread' crates/contextra-store/src/lsm/fence.rs` leer.

### WP-7 WAL: Entsorgung, Scope-Purge, Verankerung (behebt N10, N12, ERA-001)

**Befund-Kern:** Der Produktionspfad entsorgt WAL nur beiläufig durch den Flush. Das ist für sauber gerotierte Segmente im Normalfall richtig (Raw-Sweep-Test grün, C), aber nicht garantiert für (a) **gemischte** Segmente (Frames gelöschter *und* später geschriebener fremder Schlüssel, die Fence ist präfixbezogen, andere Schlüssel schreiben weiter), (b) Segmente der Klassen `Sealed`, `Archived`, `Backup`, (c) verschlüsselungslose WALs (CRY-001: `EncryptedWal` unverdrahtet, A).

**P6-Aktion (`WalRetiring`), im `PurgeBackend::retire_wal`:**
1. **Rotieren:** aktives Segment versiegeln, damit der Schnitt scharf ist. Schreiber für andere Schlüssel laufen in ein neues Segment weiter.
2. **Anker zuerst:** Manifest-`WalCheckpoint { hmac }` für den neuen Anfang **vor** jeder Entsorgung durable schreiben (fsync). Nach einem Absturz zwischen Anker und Entsorgung bleiben Segmente liegen; der nächste Lauf entsorgt sie idempotent.
3. **Vollständig überholte Segmente** (alle Frames ≤ Flush-Sequenz): `retire_segments_up_to(wal_dir, flush_tx)` **mit Scrub**. `durable_remove` allein genügt für INV-ERA-3 nicht; es wird `scrub_if_safe` mit derselben `DisposePolicy` wie in P5 verwendet.
4. **Gemischte Segmente:** `execute_scope_purge(wal_dir, scope)` schreibt das Segment ohne Scope-Frames neu und verkettet den HMAC-Pfad neu. **Prüfauftrag (H):** ich habe `wal/scope_purge.rs` nur an den `let _`-Stellen gelesen. Zu klären und per Test festzuschreiben: Wie wird die Kette nach dem Herausnehmen von Frames neu verknüpft, und wie wird der Manifest-Anker danach konsistent gehalten? Test `wal_scope_purge_preserves_chain_and_reopens`: Rewrite → Neustart → kein `WalTruncationDetected`, fremde Schlüssel unverändert, Scope-Marker weg.
5. **Klassen `Archived`/`Backup`:** Das sind Kopien. Sie gehören in den `CopyRegistry`-Mechanismus (existiert: `DpiCopyObserver`). Ohne ausdrückliche Freigabe (`PurgeMode::PurgeWithScrub`) liefert der Purge `PurgeBlocked::CheckpointClones`; mit Freigabe werden sie wie in Schritt 3 behandelt. Nie stillschweigend liegen lassen **und** „gelöscht“ melden.
6. **Bericht:** `WalRetireReport` wird Teil des `PurgeReport` (`wal_segments_retired`, `wal_segments_rewritten`, `wal_frames_purged`), damit der Zeuge (WP-9) echte Zahlen hat.

**Verdrahtung:** `PurgeBackend::retire_wal` ist Pflicht-Methode (WP-4), kein `Option`-Hook. `PurgeDriver::run_physical` übergibt keinen `None`-Callback mehr.

**ERA-001:** Die drei `let _ = tokio::fs::remove_file(...)` in `wal/scope_purge.rs` (417, 435, 440) werden durch `remove_if_exists(path).await?` ersetzt:
```rust
pub async fn remove_if_exists(p: &Path) -> std::io::Result<()> {
    match tokio::fs::remove_file(p).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}
```
Danach `fsync_parent_dir(...).await?` (steht teils schon). Dasselbe Muster ersetzt `let _ = fp;` (Zeile 458, totes Statement) durch echte Nutzung oder Entfernung.

**WAL-002 (N12) Verankerung:** Die heutige Prüfung (`recovery.rs:387-405`) vergleicht `expected_hwm` mit sich selbst, sobald eine SSTable im Manifest steht. Soll:

```rust
enum AnchorCheck { HwmInReplay, FirstFramePrevMatches, EmptyAfterRetire }
fn verify_wal_anchor(hwm: &[u8;32], replay: &ReplayInfo, manifest: &Manifest)
    -> Result<AnchorCheck, ContextraError> {
    if replay.hmac_set.contains(hwm)                      { return Ok(HwmInReplay); }
    if replay.first_prev_hmac.as_ref() == Some(hwm)       { return Ok(FirstFramePrevMatches); }
    if replay.is_empty() && manifest.last_retired_tail() == Some(hwm) { return Ok(EmptyAfterRetire); }
    Err(ContextraError::wal_truncation_detected(*hwm, replay.tail_hmac()))
}
```
Das erkennt Verlust **unterhalb** des HWM (entfernte oder beschnittene alte Segmente). Es erkennt **nicht** das Abschneiden von Frames **nach** dem HWM, weil das Manifest nur beim Flush fortgeschrieben wird. Für die Löschung folgt: Das Abschneiden eines Tombstones nach dem HWM kann Daten wieder sichtbar machen. Der Schutz dagegen ist **nicht** die Manifest-Prüfung, sondern der Ledger-Abgleich beim Öffnen (WP-8: `reapply_tombstones_from_ledger`). Der externe Anker aus ADR-042 (Audit-Chain) ist die einzige Maßnahme gegen einen Angreifer mit WAL-Schreibrecht; das bleibt Entscheidung E-4.

**Tests:** `wal_retire_mixed_segment_raw_sweep`, `wal_retire_scrubs_not_just_unlinks`, `wal_backup_class_blocks_without_authorization`, `wal_scope_purge_preserves_chain_and_reopens`, `wal_scope_purge_propagates_remove_error` (Failpoint für `remove_file`), `wal_truncation_below_hwm_detected`, `wal_retired_after_flush_opens_clean`, `purge_wal_retire_errors` repariert (heute rot), `rg 'let _ =' crates/contextra-store/src/wal/` mit leerer Allowlist.

### WP-8 Ledger und `ErasureMode` (behebt N11, N6)

**Bestand (L):** `ErasureLedger` speichert **Keyed Hashes** (`compute_keyed_hash(LedgerKey, storage_scope, bytes)`, Domain `contextra/erasure-ledger/v1`), nicht Schlüssel. Das ist datenschutzfreundlich (Audit 4g) und bleibt. `decide(storage_scope, key, tx) -> LedgerDecision`, `append(&[LedgerEntry])` existieren; `LsmStorage::ledger_append` hat keinen Aufrufer außerhalb der Definition. `StreamingRestoreGuard` und `wal/replay_filter.rs` konsumieren `decide`.

**Soll: Wer ruft wann auf.** Ausschließlich der Koordinator (WP-10). Schrittfolge je `ErasureMode`:

| Schritt | `Off` | `Assured` | `Attested` |
|---|---|---|---|
| Scope-Datei + Intent (WP-3) | nein | **ja** | **ja** |
| Ledger-Append (fsync), `Prefix`/`Key` je Scope-Element | nein | **ja, vor Tombstone** | **ja, vor Tombstone** |
| Tombstone-Commit | ja | ja | ja |
| Fence, Flush, Rewrite, Dispose, WAL (WP-4 bis WP-7) | optional (`PurgeMode`) | **ja** | **ja** |
| Verifikationsscan über alle Schichten | nein | Store | alle deklarierten Schichten |
| Beweis (WP-9) | nein | nein | **ja** |

- **Idempotenz:** Ledger-Append desselben `keyed_hash` ist erlaubt (gleicher Eintrag, gleiche Wirkung); der Resume der Phase `Admitted` wiederholt Append und Tombstone ohne Schaden.
- **Kein Re-Execute aus dem Ledger:** Aus Keyed Hashes lassen sich keine Schlüssel rekonstruieren (s. WP-3, „Verworfen“). Das Ledger **sperrt** (Restore, WAL-Replay-Filter), es **führt nichts aus**. Die Ausführungspflicht trägt allein die Scope-Datei mit Intent.
- **Wiederauferstehung nach Truncation (Brücke zu N12):** Beim Öffnen läuft `reapply_tombstones_from_ledger`: Für jeden Ledger-Eintrag wird geprüft, ob ein sichtbarer Schlüssel mit passendem Keyed Hash existiert; falls ja, wird er per Tombstone erneut unterdrückt und ein `ResurrectionRepaired`-Ereignis protokolliert. **Kostenwarnung:** Das erfordert einen Scan aller Schlüssel; es ist nur akzeptabel mit einem Bloom-/Index-Beschleuniger oder als Hintergrundjob nach dem Öffnen. Bis das gebaut ist, ist die Zusage „Truncation kann Gelöschtes nicht zurückbringen“ **nicht** gegeben und darf nicht gemacht werden (INV-ERA-1).
- **`ErasureMode` wird gelesen:** Heute wird der Wert nur am Feature-Check (`lifecycle.rs`) ausgewertet (N6). Der Koordinator wählt die Schritte per `mode.requires_ledger()` / `requires_attestation()` (existieren in `contextra-ports/src/erasure.rs`). Kein zweites Flag (`deletion_proof_active`, W1 aus dem Audit) darf dasselbe steuern; es wird entfernt.
- **Format:** Ledger-Format bleibt v1, keine Migration nötig.
- **Retention (E-5, rechtlich):** Das Ledger speichert dauerhaft pseudonymisierte Löschereignisse. Aufbewahrungsdauer und Löschregel für das Ledger selbst sind **nicht** Engine-Code, sondern Löschkonzept (Audit 4f/4g). Die Engine liefert nur den Hook `Ledger::compact(older_than)` mit schriftlich entschiedener Frist.

**Tests:** `ledger_append_precedes_tombstone` (Failpoint zwischen beiden: Ledger hat den Eintrag, Resume vollendet), `forget_assured_writes_ledger` (heute **rot**: Ledger leer nach `forget`), `forget_off_writes_no_ledger`, `restore_after_forget_rejects_key` (End-to-End, WP-11), `ledger_privacy_no_plain_keys_on_disk` (Rohsweep des Ledger-Verzeichnisses auf Klartext-Schlüssel).

---

### WP-9 Zeugen und Beweis (behebt E6, E2, D1, D2, N23)

**9.1 Zeugen entstehen nur aus Messung (INV-ERA-6).**
- `LayerWitness::issue` wird `pub(crate)`. Das `#[doc(hidden)] pub` heute ist ein Verstecken, kein Schutz. Die vier Produktionsaufrufer (`forget.rs:234`, `text_witness.rs:57`, `relations_witness.rs:53`, Vector-Witness) und die Testaufrufer wechseln auf:

```rust
// contextra-crypto
pub struct ScanOutcome { pub layer: DeletionLayer, pub units_scanned: u64,
    pub matches_found: u64, pub scanner: ScannerId, pub scanned_at_tx: TxId }
impl LayerWitness {
    /// Verweigert bei matches_found > 0 und bei units_scanned == 0 ohne ausdrückliche
    /// `EmptyLayer`-Deklaration. Setzt Zeitstempel über den Clock-Port.
    pub fn from_scan(o: ScanOutcome, scope_hash: [u8; 32], claim: WitClaimText,
                     assurance: WitAssurance, clock: &dyn Clock) -> Result<Self>;
}
```
- **Ehrliche Grenze:** Innerhalb eines Prozesses kann ein Crate mit Zugriff auf `ScanOutcome::new` immer fälschen; Rust-Sichtbarkeit schützt vor *versehentlicher*, nicht vor *böswilliger* Fabrikation. Der Zweck ist, die in L1 gefundene Fehlerklasse (Zeuge mit festen Werten) unmöglich zu machen. Dazu ein xtask-Gate `witness-origin`: `ScanOutcome::new` darf nur in Dateien unter `**/scan*.rs`, `**/residue_scan.rs`, `**/witness.rs` vorkommen (Allowlist-Datei, Änderungen brauchen CODEOWNERS-Review).
- `claim`/`assurance` werden **nicht** im Driver fest verdrahtet (N23). Der Koordinator leitet den Claim aus `ErasureState` und `opts.assurance` ab; ein Widerspruch (`claim > assurance`) ist ein **Konfigurationsfehler vor** Beginn des Purge (`ErasureError::AssuranceUnreachable`), nicht ein Fehler danach.

**9.2 Kanonische Kodierung statt `bincode` (E2).**
- Neues Modul `contextra-crypto::canonical`: TLV, Tag `u16` LE, Länge `u32` LE, Wert. Ganzzahlen fester Breite LE; Zeichenketten UTF-8 mit Länge; `Option` als Präsenz-Byte; Aufzählungen als Tag + Nutzlast; **keine Maps** (nur sortierte Listen); Felder in fester Tag-Reihenfolge, doppelte oder unbekannte kritische Tags ⇒ Ablehnung.
- Alle Signaturpayloads (`proof_v4`, `audit_chain`, `layer_ids`, `verify`) und die handgebaute Variante in `collections.rs` wechseln darauf. `bincode` bleibt höchstens als *lesender* Legacy-Pfad (v2/v3-Verifikation) hinter dem Feature `legacy-verify`; dort gilt das bestehende Erzeugungsverbot (`#[deprecated]`).
- **Goldene Vektoren:** `tests/golden/proof_v4_*.hex` (mindestens ein Beweis je `DeletionScope`-Variante, ein Beweis mit leerer und einer mit voller `excluded_scopes`-Liste). Ein Test erzeugt den Beweis neu und vergleicht Bytes. Änderungen an den Vektoren brauchen CODEOWNERS-Review.
- Cross-Check: `tools/contextra-verifier` verifiziert einen vom Engine-Pfad erzeugten Beweis (Test `engine_proof_verifies_with_external_verifier`). Das hätte D2/K_ENG-002 gefunden.

**9.3 Ein Signierschlüssel, eine Instanz-ID (L10, E4).**
- Trait `ProofSignerProvider { fn signer(&self) -> Result<&dyn ProofSigner>; fn db_instance_id(&self) -> [u8; 16]; }`, einmal im Composition Root gebaut. `drop_collection(name, tenant, proof_key: &[u8])` verliert den Schlüsselparameter (Entscheidung E-6: API-Bruch). Heute gibt es zwei Schlüsselquellen im selben Aufruf: den vom Aufrufer übergebenen `proof_key` (signiert den Phase-A-Beweis) und `db_dir/proof_ed25519.key` (Wiring, Phase B).
- `db_instance_id` stammt **ausschließlich** aus dem Ledger-Handle. Der heutige Rückfall auf `blake3(db_dir)[..16]` entfällt; ohne Ledger gibt es keinen `Attested`-Modus (`ErasureError::AttestationRequiresLedger`).
- Zeitstempel: nur über den `Clock`-Port; `timestamp == 0` ist ein Konstruktionsfehler (`ProofBuilder::build` lehnt ab).

**9.4 Zweistufiger Beweis (Phase A/B) als Zustand.** Phase A signiert „Tombstone committed“ (`covered = {LsmMemtable}`), Phase B (`supersedes = A`) erst nach `Verified`. Welche `DeletionLayer` ein Beweis nennen darf, ergibt sich **ausschließlich** aus `ErasureState` plus gemessenen Zeugen (Tabelle in WP-10). Beweis vor dem Purge zu signieren ist nur Phase A, und der zurückgegebene Beweis muss als solcher erkennbar sein (`proof.stage`).

**Tests:** `layer_witness_issue_not_public` (trybuild: Aufruf außerhalb des Crates kompiliert nicht), `from_scan_rejects_matches`, `from_scan_rejects_zero_units_without_declaration`, `proof_canonical_golden`, `proof_canonical_rejects_duplicate_tag`, `proof_timestamp_nonzero`, `engine_proof_verifies_with_external_verifier`, `assurance_contradiction_rejected_before_purge` (Regressionstest zu N23), `no_second_signer_key` (Grep-Test auf `proof_ed25519.key`-Pfadbau außerhalb des Providers).

### WP-10 Ein Koordinator, ein Pfad (behebt N4, N23, D1 bis D6; INV-ERA-1, INV-ERA-7)

**Ziel:** `forget`, `forget_collection`, `forget_tenant`, `drop_collection(_with_report)` rufen **eine** Funktion. Heute gibt es mindestens zwei Pipelines (`execute_forget_pipeline` in `forget.rs` und die Inline-Pipeline in `collections.rs`) mit unterschiedlicher Garantie (L13 aus dem Audit).

```rust
pub struct ErasureRequest {
    pub tenant: TenantId, pub scope: LogicalScope,   // Document | Collection | Tenant
    pub mode: ErasureMode, pub opts: ForgetOptions,
}
pub enum ErasureState {
    Requested, Tombstoned, Purged, WalRetired,
    Verified { witnesses: WitnessSet },
    Attested { proof: ProofRef },
}
pub struct ErasureOutcome {
    pub id: ErasureRequestId, pub state: ErasureState,
    pub pending: Option<PendingReason>,     // None ⇔ Zielzustand des Modus erreicht
    pub purge: Option<PurgeReport>,         // echte Zahlen, nie abgeleitet
}
#[derive(Debug, thiserror::Error)]
pub enum PendingReason {                    // KEIN String-Feld, das Zustände behauptet
    #[error("blocked: {0}")]        Blocked(PurgeBlocked),
    #[error("failed in {phase:?}")] Failed { phase: PurgePhase, kind: PurgeErrorKind },
    #[error("layer {0:?} not scanned")] LayerUnscanned(DeletionLayer),
}
impl ErasureCoordinator { pub async fn execute(&self, req: ErasureRequest) -> Result<ErasureOutcome>; }
```

**Regeln**
1. **Zustand statt Prosa (INV-ERA-1).** Jeder Text im Report entsteht per `Display` aus `PendingReason`/`ErasureState`. `PhysicalPurgeStatus::Performed { records_purged, bytes_reclaimed }` bezieht beide Zahlen aus `PurgeReport` (D3). Eine hartcodierte Begründung wie „managed by background ErasureSupervisor“ (N4) ist nicht mehr ausdrückbar.
2. **Fehler nach dem Purge verändern das Purge-Ergebnis nicht (N23).** Scheitert die Zeugenausstellung nach einem erfolgreichen Purge, ist der Zustand `Purged` mit `pending = Failed { phase: Witnessing, .. }`, **nicht** „NotPerformed“. Der Aufrufer sieht: Daten weg, Nachweis fehlt, Wiederholung des Nachweises möglich (die Phase `Witnessing` ist idempotent, WP-4).
3. **Fehlschlag persistiert (N4).** Bleibt ein Purge `Pending`, bleibt das Intent samt Scope-Datei liegen; der nächste Start (WP-4) oder ein expliziter `resume_erasure(id)` setzt fort. Es gibt **keinen** Hintergrund-Supervisor, solange keiner verdrahtet ist; `EngineLifecycle`/`ErasureSupervisor` (heute nur in Tests erzeugt) werden entweder real verdrahtet oder **gelöscht**. Toter Code mit Dokumentationsanspruch ist die Quelle der Falschaussage aus N4.
4. **Collection-Drop im Besonderen (D4 bis D6).**
   - Reihenfolge: Intent und Ledger-Append (WP-8) → Tombstone-Commit → **erst dann** Registry-Eintrag entfernen. Schlägt der Commit fehl, bleibt die Collection registriert (D6). Ein Zwischenzustand `CollectionState::Dropping` wird persistiert, damit ein Neustart zwischen Commit und Purge die Collection nicht „wiederbelebt“.
   - **Eine** Ableitung der `collection_id` mit Tenant: `derive_collection_id(tenant, name)`. Der Code berechnet sie heute zweimal: `let _collection_id = derive_collection_id(name)?;` (ungenutzt) und danach manuell `blake3(name)[0..8]` (D4).
   - **Eine** Schlüsselkodierung: Scope-Präfixe **und** Reverse-Sweep-Nadeln kommen aus `tenant_codec` (heute `format!("t:{}:")` für den Scope, `tenant_id.to_le_bytes()` für die Nadel; D5). Test `sweep_needles_match_purge_scope_encoding`.
   - **Deklarierte Schichten:** Solange Vektor-, Text- und Graph-Schicht der Collection nicht gelöscht und gescannt werden, enthält der Beweis `excluded_scopes = [VectorIndex, TextIndex, Graph]` (Typ, nicht Text). `covered_layers` ist heute `[LsmMemtable]` mit leerer Ausschlussliste, was Vollständigkeit impliziert. Entscheidung E-9: blockieren oder deklarieren; Vorschlag: deklarieren.
5. **Claims-Tabelle (aus `ErasureState`):**

| Zustand | darf als gelöscht behauptet werden | Beweisstufe |
|---|---|---|
| `Requested` | nichts | – |
| `Tombstoned` | logisch (`LsmMemtable`) | A |
| `Purged` | `SsTableAllLevels` **nur**, wenn Verifikationsscan (WP-4, Schritt 5) sauber | – |
| `WalRetired` | `WalAllSegments` mit Report-Zahlen | – |
| `Verified` | genau die Schichten mit Zeuge `matches_found == 0` | – |
| `Attested` | wie `Verified`, signiert | B |

6. **Claims-Gate:** `docs/deletion/CLAIMS.toml` listet jede öffentliche Zusage mit zugehörigem Testnamen. `cargo xtask claims-gate` (an das vorhandene `symbol-exists`-Gate angelehnt) bricht, wenn ein Test fehlt oder ein Satz in `README.md`, `docs/deletion/*`, Fehlertexten und Reason-Strings eine Formulierung wie „gelöscht heißt gelöscht“ oder „garantiert“ ohne Eintrag enthält.

**Tests:** `drop_collection_default_options_reports_performed` (**rot am HEAD**, Regression zu N23), `witness_failure_after_purge_keeps_purged_state`, `drop_collection_failed_commit_keeps_registry`, `drop_collection_restart_between_commit_and_purge_resumes`, `drop_collection_proof_declares_unscanned_layers`, `pending_reason_is_typed` (Reflexions-/Grep-Test: kein `String`-Feld namens `reason` in Outcome-Typen), `forget_and_drop_share_pipeline` (Aufrufzähler im Koordinator), und das alte `drop_collection_layer_claims.rs` wird auf den neuen Zustand umgeschrieben.

### WP-11 Restore und Anti-Resurrection (N5, N22, ERA-003)

- **Entscheidung E-7 zuerst:** Das Checkpoint-/Restore-Subsystem hat heute keinen Produktionskonstruktor. Entweder wird es verdrahtet (dann mit Filter, unten) oder es wird aus dem Produktionsbaum **entfernt** (`#[cfg(feature = "checkpoint")]`, Default aus) und dokumentiert. Ein verwaister Restore-Pfad mit fail-closed-Sperre ist kein Zustand, der „fertig“ heißt.
- **Typ statt `Option`:** `PersistentCheckpointStore::new(storage, name, policy: RestorePolicy)` mit `enum RestorePolicy { Filter(Arc<dyn CpyRestoreFilter>), ForbidAfterErasure }`. Das heutige `restore_filter: Option<_>` plus Laufzeitprüfung `has_deletion_history && restore_filter.is_none()` wird zum Compilezeit-Vertrag. `NoOpRestoreFilter` (ERA-003) nur unter `#[cfg(any(test, feature = "testing"))]`.
- **N22:** `enum LedgerSource { Direct(Arc<ErasureLedger>), Handle(Arc<ErasureLedgerHandle>) }` in `DpiRestoreFilter` und `StreamingRestoreGuard`; der `else { false }`-Zweig verschwindet.
- **Verdrahtung:** Composition Root baut `DpiRestoreFilter::for_lsm_handle(handle)` bei `mode.requires_ledger()`.
- **Backups:** Restore aus Datenträger-Backups läuft durch denselben Guard; die Frist, bis zu der Backups Gelöschtes noch enthalten dürfen, steht im Löschkonzept (Audit 4h), nicht im Code.
- **Tests:** `delete_then_restore_keeps_deleted_deleted` (Fassade, End-to-End), `restore_without_filter_after_erasure_fails_closed`, `ledger_source_both_none_unrepresentable` (trybuild), `restore_rejects_key_erased_after_checkpoint_tx`.

### WP-12 Restspuren, Performance, Verifikation

- **Block-Cache (N17):** `BlockCache::invalidate_file(file_id)`; Test über die Cache-API (`contains_file(id) == false` nach Purge), keine Heap-Durchsuchung. Optionaler Nightly-Test (nur Linux) durchsucht `/proc/self/mem` auf Markerbytes; er ist wegen Allokator-Verhalten nicht deterministisch und darf nie als Gate dienen.
- **mmap/Handles:** Prüfauftrag, ob `SstableReader` Dateien mappt. Falls ja: `munmap` vor dem Scrub (`Arc`-Drain in WP-5) ist Pflicht; sonst liest ein Leser Nullen oder Alt-Daten.
- **Speicher:** Scope-Puffer in `Zeroizing`, `PurgeScope` implementiert `Drop` mit Zeroize der Schlüsselbytes. Priorität niedrig (die Schlüssel sind Dokument-IDs, keine Geheimnisse im Kryptosinn, aber personenbeziehbar).
- **Restrisiken typisiert:** `enum ResidualRisk { SsdWearLeveling, FilesystemJournal, CopyOnWrite, Backup(CopyId), SwapOrPageCache, UnscrubbedRemoval(PathBuf) }`. Der `PurgeReport` führt sie auf; die Produktkommunikation (Audit Abschnitt 4) leitet daraus ab.
- **Performance-Budget (messen, nicht annehmen):** Fence-Hot-Path (WP-6), Purge je 100 MiB SSTable (Wanduhr, Spitzen-Heap), Verifikationsscan (WP-4 Schritt 5), WAL-Retire. Messprotokoll nach `docs/BENCHMARK_PROTOCOL.md`; Baselines vor der Änderung festhalten, weil heute nichts davon läuft (Build kaputt).
- **Fuzzing (`cargo-fuzz`):** `OrxPurgeIntent::from_bytes`, `PurgeScope::from_canonical_bytes`, Scope-Datei-Dekodierung, WAL-Frames im Scope-Purge. Ziel: kein Panic, kein Speicherwachstum jenseits der Eingabegröße.
- **Mutationstests:** Die Dateien `lsm/ops/purge.rs`, `lsm/fence.rs`, `lsm/purge/*.rs`, `compaction/purge_filter.rs`, `wal/retire.rs`, `wal/scope_purge.rs` in `mutants-guarantees.toml` aufnehmen; Ziel null überlebende Mutanten in Logikfunktionen (`matches_key`, Phasenordnung, Swap-Zuordnung, Fence-Tabellenprüfung). Der Swap-Fehler aus WP-5 ist ein Beispiel, das ein solcher Lauf finden muss.

---

## 6. Testmatrix (Invariante → Test → Paket)

| Invariante | Test | WP | Art |
|---|---|---|---|
| INV-ERA-1 | `pending_reason_is_typed`, `claims_gate`, `drop_collection_default_options_reports_performed` | 10 | Gate + Integration |
| INV-ERA-2 | `loom_fence_no_post_drain_write` (mit Negativkontrolle), `fence_commit_vs_purge_stress` | 6 | Loom + Stress |
| INV-ERA-3 | `resume_matrix_all_phases`, `wal_retire_mixed_segment_raw_sweep`, `purge_invalidates_block_cache`, `purge_obsolete_raw_sweep` (bestehend) | 4, 5, 7, 12 | Rohsweep |
| INV-ERA-4 | `zz_repro_n1_resume_noop` (→ `resume_executes_purge`), `resume_detects_residue_and_keeps_intent`, `purge_crash_matrix_dst` | 3, 4 | Crash-Matrix |
| INV-ERA-5 | `legacy_v1_intent_not_silently_completed`, `fence_unknown_key_rejected`, `restore_without_filter_after_erasure_fails_closed` | 3, 6, 11 | Fail-closed |
| INV-ERA-6 | `layer_witness_issue_not_public`, `from_scan_rejects_matches`, `witness-origin`-Gate | 9 | trybuild + Gate |
| INV-ERA-7 | `forget_and_drop_share_pipeline` | 10 | Integration |
| INV-ERA-8 | `concurrent_purges_single_flight` | 3 | Integration |
| Swap-Korrektheit | `purge_two_ssts_first_becomes_empty` + Eigenschaftstest gegen `BTreeMap`-Modell | 5a | Property |
| Journal-Sicherheit | `journal_entry_for_live_file_is_not_disposed` | 5 | Crash |
| Beweisformat | `proof_canonical_golden`, `engine_proof_verifies_with_external_verifier` | 9 | Golden + Cross |

Jeder Test, der einen Befund belegt, wird **zuerst rot committet** (eigener Commit, CI-Ausnahme `expected-fail` nur für diesen Commit), dann folgt der Fix. So bleibt die Historie ein Beweis, dass der Test den Fehler wirklich fängt.

---

## 7. Reihenfolge, PR-Zuschnitt und Done-Kriterium

### 7.1 Abhängigkeiten

```
WP-0 ─┬─ WP-5a (Swap-Fix, sofort)
      ├─ WP-1 ─┬─ WP-2 ─ WP-3 ─ WP-4 ─┬─ WP-7 ─ WP-8 ─┬─ WP-10 ─ WP-11
      │        └────── WP-5 ─ WP-6 ─┘                │
      └─ WP-9 (parallel zu 3-8, Verdrahtung in WP-10) ┘
WP-12 begleitet jedes Paket (Messungen, Fuzz, Mutation).
```
Begründung: WP-5 braucht den `FenceHandle`-Typ aus WP-6; deshalb wird der Swap-Fehler (WP-5a) herausgelöst und zuerst behoben. WP-4 braucht Fehlertypen (WP-1), eine Phasen-Enum (WP-2) und den Scope (WP-3). WP-10 hängt an allem, weil es die Pipeline umschaltet.

### 7.2 PR-Regeln
- Ein Paket = ein PR, wenn ≤ 600 geänderte Zeilen ohne Tests; sonst nach Teilzielen schneiden (Typen → Verdrahtung → Umschalten).
- Jeder PR: roter Test-Commit, grüner Fix-Commit, `cargo xtask gate-compile-all` grün (WP-0), Merge-Queue baut gegen aktuellen `main`.
- Kein PR löscht einen Test ohne benannten Ersatz in Abschnitt 8.

### 7.3 Done-Kriterium („ohne technische Schulden“, maschinell prüfbar)

| Gate | Prüfung |
|---|---|
| G1 | `cargo check --workspace --all-targets` und `cargo test --workspace --no-run` grün |
| G2 | `cargo test --workspace` grün (keine `#[ignore]` in den Löschpfad-Dateien ohne Ticket) |
| G3 | `rg 'let _ =' ` in den Pfaden `lsm/ops/purge.rs`, `lsm/purge/`, `lsm/fence.rs`, `wal/scope_purge.rs`, `wal/retire.rs`, `deletion/`, `contextra_impl/{forget,collections}.rs`: **0 Treffer** außer in einer reviewten Allowlist mit Begründung |
| G4 | `rg '\.ok\(\)'` im selben Pfadsatz: nur mit Kommentar `// IGNORE-OK: <Grund>` |
| G5 | `rg 'TODO\|FIXME\|todo!\|unimplemented!\|unwrap\(\)\|expect\('` in Nicht-Test-Code des Pfadsatzes: 0 |
| G6 | Kein `#[allow(...)]` im Pfadsatz ohne `reason = "..."` |
| G7 | `rg 'Option<HookCallback>\|new_standalone\|PurgePhase::P[0-9]'` : 0 |
| G8 | Raw-Sweep-Matrix (`resume_matrix_all_phases`) grün auf Linux **und** in der Windows-/macOS-CI, soweit `scrub_if_safe` dort definiert ist |
| G9 | Loom-Test grün **und** Negativkontrolle rot (Selbsttest des Gates) |
| G10 | Mutationstest: 0 überlebende Mutanten in den Logikfunktionen aus WP-12 |
| G11 | `claims-gate` grün |
| G12 | `cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check`, `cargo xtask check-ring-layering-full` grün |
| G13 | Fuzz-Ziele aus WP-12 laufen 10 Minuten ohne Fund in der Nightly-CI |
| G14 | Doku-Drift: `docs/deletion/*` nennt keine Phase, keinen Typ und keinen Dateipfad, der im Code nicht existiert (`symbol-exists`-Gate) |

Erst wenn G1 bis G14 grün sind, darf jemand „fertig“ sagen. Das heißt nicht „fehlerfrei“, sondern „kein bekannter Befund aus Abschnitt 3 offen und jede Regel maschinell geprüft“.

---

## 8. Offene Entscheidungen (brauchen einen Besitzer, nicht den Code)

| ID | Frage | Vorschlag |
|---|---|---|
| E-1 | Betreiberpfad, wenn ein Resume dauerhaft scheitert (`RequireScrub` bei nicht scrubbaren Dateien): `open` bleibt fail-closed, die DB ist sonst unöffenbar | `contextra-admin purge abandon` mit signiertem Audit-Ereignis „Löschversprechen für diesen Scope nicht erfüllt“ |
| E-2 | `FreeSpaceProbe` liefert „unbekannt“: erlauben oder blockieren? | Für `Assured`/`Attested` blockieren (`UnknownFreeSpace`) |
| E-3 | Budget des Fence-Hot-Path (zwei RMW je Batch) | Akzeptieren, wenn Commit-Latenz < 1 % steigt; sonst Batch-Gruppierung vergrößern, nicht die Reihenfolge ändern |
| E-4 | Externer Anker gegen WAL-Truncation **nach** HWM (ADR-042) und `reapply_tombstones_from_ledger` | Beides vor jeder Zusage „Truncation bringt Gelöschtes nicht zurück“; bis dahin nicht zusagen |
| E-5 | Aufbewahrung und Löschung des Ledgers und der Beweise selbst | Rechtliche Entscheidung (Audit 4f/4g); Engine liefert nur `compact(older_than)` |
| E-6 | `proof_key`-Parameter in `drop_collection` entfernen (API-Bruch) | Entfernen; Signer kommt aus dem Composition Root |
| E-7 | Checkpoint/Restore verdrahten oder aus dem Produktionsbaum nehmen | Entscheidung vor WP-11; kein Zwischenzustand |
| E-8 | `ScopeSealer` bei unverschlüsselten DBs: Klartext-Scope-Datei mit CRC erlaubt? | Ja, mit Scrub; dokumentieren |
| E-9 | Collection-Drop, solange Vektor/Text/Graph nicht physisch gelöscht werden: blockieren oder `excluded_scopes` deklarieren | Deklarieren (typisiert), nie `[]` |
| E-10 | Port-Konsolidierung (B0-d): Adapter im Composition Root oder in einem Ring-1-Crate | Ring-1-Crate `contextra-sys-adapters`; Aufrufer in einem PR umstellen |

**Rechtliches:** Abschnitt 4 des Audits (DSGVO, eIDAS, EuGH C-413/23 P, DIN 66399, NIST SP 800-88) übernehme ich als Recherche **unverändert und ungeprüft**. Die Zitate hat das Audit zum Teil „aus der Erinnerung“ genannt. Vor jeder Kundenzusage prüft ein Datenschutzjurist.

---

## Anhang A: Reproduktion

Toolchain: `apt-get install -y rustc-1.89 cargo-1.89` (entspricht dem Pin), `ln -s /usr/bin/cargo-1.89 ~/bin/cargo`.

| Befund | Befehl | Erwartung heute |
|---|---|---|
| B0 | `cargo check -p contextra-store -p contextra-engine --lib` | Fehler (B0-a/b/d) |
| N1 | `cargo test -p contextra-store --test zz_repro_n1_resume_noop` | **rot**, „Marker liegt weiter auf der Platte“ |
| WP-5 Swap | `cargo test -p contextra-store --test zz_repro_wp5_swap_order` | **rot**, `get(k) == "old"` |
| N23, D1 | `cargo test -p contextra-engine --test zz_repro_drop_path -- --nocapture` (Compile > 5 min) | `NotPerformed` trotz sauberem Sweep, `timestamp=0` |
| Baseline | `cargo test -p contextra-store --test <Datei>` je Datei aus 2.3 | wie Tabelle 2.3 |

Lokale Hilfsänderungen in meinem Klon (nicht Teil der Spezifikation): `0001-fix-build-break-store.patch` und der Stand von `contextra-ports/src` aus `77acbaa49`.
