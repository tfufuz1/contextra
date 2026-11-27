# Contextra – Implementierungsspezifikation: Rückbau der alten Löschkette und Aufbau der Erasure Chain

*Stand 27.11.2026 · Basis: `tfufuz1/contextra`, Branch `main`, Commit `7bd922104` · Rolle: Principal Rust Engineer (Systemprogrammierung, Durability, Concurrency, RAG-Speicherkerne) · Status: Entwurf zur Freigabe*

## 0. Geltungsbereich und Verifikationsstatus

**Auftrag.** Die alte Löschkette (Feature `legacy-erasure` in `contextra-crypto` samt Nachweis-, Shredding- und Widerrufslog-Code sowie den daran hängenden Verweisen) wird entfernt. Die neue Löschkette (Erasure Chain: Saga, Gate, Crypto-Erase, Layer-Purge, Witness, Ledger) wird sauber neu implementiert.

**Quellen.** (1) Das Konzept „Konzept und Spezifikation der vollständigen Löschkette“ (Commit `7bd922104`). Das hochgeladene Exemplar endet bei §5.3. Teststrategie, Arbeitspakete und Betrieb fehlen und sind hier neu spezifiziert. Die WP-Nummern bleiben kompatibel: WP-0 ist der Stop-Ship, WP-4 der prädikatsgefilterte Rewrite. (2) Die ältere Bestandsanalyse (Commit `ed7908c`). Sie ist in Teilen überholt, ihre Anforderungen sind in WP-5 und WP-12 übernommen. (3) Der Code am HEAD, geklont und gelesen.

**Verifikationsgrenze.** In der Analyseumgebung gibt es keine Rust-Toolchain (`cargo` fehlt, `rust-toolchain.toml` verlangt 1.89.0). Es wurde nichts kompiliert und kein Test ausgeführt. „Verifiziert“ heißt in diesem Dokument: Quelltext gelesen. Mit **\[?\]** markierte Aussagen sind nicht belegt und werden in WP-0 geprüft, bevor das betroffene WP beginnt.

**Abweichungen zur Bestandsanalyse.**

- K2 (`LsmPurger` löscht den ganzen Mandanten) ist durch `214e679e2` fail-closed entschärft: `LsmPurger::purge` liefert immer `Err`, die Tests sind `#[ignore]`.
- Die Crates `contextra-erasure`, `-kvcache`, `-privacy` und `contextra-types/src/erasure.rs` (616 Zeilen) sind entfernt (`9a6dd9a05`). Es gibt keinen Orchestrator.
- K1, K3, H1, H3 betreffen `kv_shredding`. Das Modul wird nicht repariert, sondern entfernt (WP-1). Seine Anforderungen leben als Abnahmekriterien in WP-5 weiter: Schlüsselpersistenz fail-closed, dauerhafte Record-Revokation, kein `Clone` auf Schlüsseltypen, Neustart-Test.

## 1. Befund am HEAD (verifiziert)

| Baustein | Ort | Stand | Maßnahme |
| --- | --- | --- | --- |
| Feature-Flags | `contextra-crypto/Cargo.toml`: `default = ["legacy-erasure","kv-encryption","test-utils"]`, `kv-encryption = ["legacy-erasure"]` | Legacy ist Default | WP-1 |
| Löschnachweis v2/v3/v5 | `crypto/src/deletion_proof/` (7 Dateien, ca. 1 450 Zeilen), `ed25519_proof.rs` (244) | Keine Aufrufer außerhalb des Crates; `contextra-store/tests/campaign_j42_audit.rs` nutzt `DeletionProofKeyPair` | löschen |
| Shredding und Widerrufslog | `kv_shredding.rs` (613), `revocation_log.rs` (531) | nur intern genutzt | löschen |
| KV-Segmente | `crypto/src/kv_segment/` (1 655 Zeilen) | `manager.rs` und `kv_cipher.rs` hängen an `KeyRegistry`; `manager.rs` enthält `generate_deletion_proof` | auf RAM-Keyring umbauen |
| Audit-Kette | `crypto/src/audit_chain.rs` | enthält `cfg(legacy-erasure)`-Teil (`EncryptedSalt` über `KeyRegistry`); Ed25519-Kopfsignatur bleibt, `ed25519-dalek` bleibt Dependency | cfg-Teil entfernen |
| `KvDeleteMode` | doppelt: `crypto/src/kv_segment/delete_mode.rs` und `contextra/src/collection_profile.rs` (Invariante `INV-COLLECTION-PROFILE-1`), dazu `ContextraError::KvDeleteModeConfig` in `types/src/error.rs` und `error_dto.rs` | an Legacy-Nachweis gekoppelt | ersetzen durch `ErasureProfile` |
| Tests | 20 Dateien in `crypto/tests` mit Bezug zu Nachweis, Widerruf, Shredding; Bench `deletion_proof_latency_bench`; `contextra/tests/{deletion_e2e,collection_profile_matrix}.rs` | `T-DEL-3` prüft keinen Nachweis | WP-1, WP-12 |
| Alter Layer-Port | `ports/src/erasure.rs` | Fehler als `String`; `Scanner<Armed>` ist ein leerer Platzhalter; `EraseTarget` sind Byte-Bereiche; `PurgeCx { deadline_ms }`; `StorageBackend`/`ErasureTarget` sind leere Marker | WP-2 |
| Gate-Port | `ports/src/gate.rs` | `ReadGate` mit fail-open-Defaults (`check_raw` → `Open`, `gated_seq_raw` → `None`, `gate_epoch` → 0); `KeyBuf` hat 16 Byte, Rohschlüssel 78 Byte | WP-2 |
| Scanner-Port | `ports/src/scanner/*` | `Purger::purge(&RequestContext, &ScanTarget)` mit String-Id; `ScanSession::begin/finish` nur `pub(crate)`; `LayerWitness` ohne `Default`/`Deserialize` | behalten, `Purger` ablösen |
| Barrier, Keyring, Ledger, Observer | `ports/src/{barrier,keys,ledger,observer}.rs` | Traits ohne Implementierung; `LedgerSink` mit `append`/`read_all` | WP-3, 5, 10 |
| LSM-Purger | `store/src/scanner/purge.rs` | Stub, immer `Err` | WP-4 |
| Text-, HNSW-, Graph-Purger | je in `purge.rs`/`purger.rs` | nur im eigenen Crate referenziert | WP-6 bis 8 |
| Löschpfad | `engine/src/collection/crud/delete.rs` | `Collection::delete` schreibt Tombstones für User- und Doc-Key und ruft `index.delete`. `#[tracing::instrument(level = "trace", skip(self))]` erfasst den Parameter `id` im Span | WP-0, 11 |
| Compaction | `store/src/compaction/retention.rs`, `lsm/ops/compaction.rs` | `retain_key_versions(entries, get_seq, floor_seq)`; keine Erase-Variante | WP-4 |
| durable-fs | `AppendOnlyJournal`, `PendingDeleteJournal`, `FaultInjector`, `HealthState`, `scrub.rs` | vorhanden und wiederverwendbar | WP-3, 4 |
| Lock-Ordnung | `store/src/lsm/lock_order.rs`: Ränge 100 bis 950 | kein Rang für Erase | WP-3 |
| Workspace | 19 Member, kein Erasure-Crate; `tests/layering.rs`, `tests/unsafe_islands.rs`, `tests/arch`; `capabilities.toml` führt `legacy-erasure` unter `transitional` |  | WP-1, 2 |

**Wiederverwendbar:** `crypto/src/dek.rs` (`derive_dek`, `wrap_dek`, `unwrap_dek`, `AncestorSalts`, `UnitKey`, `Dek`), `audit_chain.rs`, Ports `ReadGate`/`CommitBarrier`/`PurgeBarrier`/`LedgerSink`/`FileKeyring`/`RecordSealer`, `LayerScanner`/`ScanSession`/`LayerWitness`, durable-fs, Compaction-Framework, `contextra-testkit`.

## 2. Zielbild (normativer Kern)

Das Konzept gilt als Referenz. Hier stehen nur die Teile, an denen die WPs hängen.

- **Klassen.** `delete()` bleibt *Retire* (Tombstone, G0). *Erase* ist ein eigener Pfad mit den Stufen G1 Gated, G2 Crypto-Erased, G3 Physically Purged. Der Auftrag meldet immer die tatsächlich erreichte Stufe.
- **Saga.** `Requested → Planned → Gating → Gated → KeysDestroying → Purging → Verifying → Sealed → Completed`, dazu `Failed(retryable)` und `Quarantined(layer)`. Jeder Übergang wird vor seiner Wirkung ins Journal geschrieben und per `fsync` gesichert. Ab `Gated` gibt es keinen Rücksprung.
- **Neues Crate `contextra-erasure`** (Ring 2) mit `ErasureCoordinator`, `ErasureJournal`, `GateTable`, `ErasureBarriers`, `KeyService`, `SaltVault`, `WitnessAggregator`, `ErasureLedger`. Es hängt nicht von Store, Text, Vektor oder Graph ab. Die Layer implementieren `ErasureLayer` in ihren eigenen Crates, die Verdrahtung passiert im Builder der Facade.
- **Invarianten** (Kurzform von INV-ERA-1 bis 12): feste Reihenfolge Gate → Schlüsselvernichtung → Purge → Witness → Seal; Write-Ahead; kein Wiederaufleben bei Flush, Compaction, Replay, Rebuild, Restore; Idempotenz jedes Schritts; Epoche streng monoton und vor der Schlüsselvernichtung veröffentlicht; Witness nur über `ScanSession`; fail-closed bei unbekanntem Gate; Scope-Treue mit Mandantenprüfung; keine Klartextwerte in Journal, Ledger, Logs; eine Lock-Ordnung; forward-only ab G1; Pins und Snapshots dürfen G1 und G2 nie blockieren.

## 3. Entscheidungen vor Start (ADR)

| ADR | Frage | Empfehlung | Blockiert |
| --- | --- | --- | --- |
| ADR-01 | D-06 („Löschung = Tombstone, kein Nachweis“) für Erase revidieren | Ja. `delete` bleibt Retire, Erase ist neu | WP-3 |
| ADR-02 | Schlüsselraum des Gates: `KeyBuf` (16 Byte) und Rohschlüssel (78 Byte) gegen `TenantKeyCodec` | Gate indexiert strukturell `(Tenant, Collection, DocId/Subject)`. `check_raw` bleibt für Store-interne Pfade. Die Länge 78 ist gegen `store/src/tenant_codec.rs` zu belegen **\[?\]** | WP-2 |
| ADR-03 | Standard-Granularität der Schlüsselableitung | `PerDocument` als Default, `PerSubject` im Profil *regulated* | WP-5 |
| ADR-04 | Schlüssel der KV-Segmente | RAM-only `EphemeralKeyring`, keine Persistenz. Die KV-Segmente sind reiner Cache | WP-1 |
| ADR-05 | `audit_chain` als Ledger-Baustein oder eigener Ledger | Eigener `ErasureLedger` über `LedgerSink`. `audit_chain` nur für die optionale Kopfsignatur prüfen | WP-10 |
| ADR-06 | `ArcSwap` (neue Dependency) oder `parking_lot::RwLock<Arc<_>>` für das Gate | Start mit `RwLock<Arc<GateSnapshot>>`, Benchmark in WP-2. `ArcSwap` nur bei verfehltem Budget (dann `cargo deny`) | WP-2 |
| ADR-07 | Herkunft des Ledger-MAC-Schlüssels, Ablage außerhalb des Datenverzeichnisses | `KeyProvider`-Port, Pfad per Konfiguration. Das Profil *regulated* verweigert den Start bei Pfad unter dem Datenpfad | WP-10 |
| ADR-08 | Tombstone-Konvention (Bit 63 der Sequenz oder Bit 0 der Flags, `types/src/tombstone.rs`) | Eine Konvention festlegen. Beide gesetzt oder keines gesetzt ist ein Fehler | WP-4 |
| ADR-09 | Semantik von `delete_implies_erase` | Nur im Profil *regulated*, Default `false` sonst | WP-11 |

## 4. Arbeitspaketübersicht

| WP | Titel | Hauptsächlich betroffen | Abhängig von | Aufwand |
| --- | --- | --- | --- | --- |
| 0 | Baseline, Stop-Ship, CI-Gates | Workspace, `tools/`, CI | – | S |
| 1 | Rückbau der alten Löschkette | `crypto`, `contextra`, `types`, `store` (Tests) | 0 | M |
| 2 | Ports v2, Typen, Gate, Crate-Skelett | `ports`, `types`, `contextra-erasure` | 1, ADR-02 | L |
| 3 | Journal, Saga, Coordinator, Barriers | `contextra-erasure`, `store` (Lock-Rang) | 2, ADR-01 | L |
| 4 | Store: Erase-Compaction, Flush-/Replay-Gate, WAL-Schnitt | `store` | 2, 3 | XL |
| 5 | Crypto-Erase: SaltVault, KeyService, Sealer | `crypto`, `contextra-erasure` | 2, 3 | L |
| 6 | Textindex | `text` | 3, 4 | M |
| 7 | Vektorindex | `vector` | 3, 4 | L |
| 8 | Graph und Provenienz | `graph`, `engine` | 3, 4 | L |
| 9 | Checkpoints, Snapshots, Caches, Commit-Residuen | `checkpoint`, `mvcc`, `engine`, `crypto` | 3, 4, 5 | M |
| 10 | Witness, Ledger | `contextra-erasure`, `testkit` | 4 bis 9 | M |
| 11 | API, Engine, MCP, Berechtigungen | `contextra`, `engine`, `mcp` | 3, 10 | M |
| 12 | Testmatrix: Crash, Loom, Conformance, E2E | `testkit`, alle | fortlaufend, Abschluss nach 11 | L |
| 13 | Migration, Seal-on-Rewrite, Betrieb | alle | 5, 8, 11 | M |

**Kritischer Pfad:** 0 → 1 → 2 → 3 → 4 → (5, 6, 7, 8 parallel) → 9 → 10 → 11 → 12 → 13. Die Layer-WPs 6 bis 8 benötigen nur den Vertrag aus WP-2 und die Erase-Compaction aus WP-4 und können nach WP-4 parallel laufen. Jedes WP endet mit grünem `cargo build`, `cargo test`, `cargo clippy -D warnings`, `cargo deny check` und `tests/layering.rs`.

## WP-0 – Baseline, Stop-Ship, CI-Gates (S)

**Ziel.** Eine reproduzierbare Ausgangsbasis und belegte Annahmen, bevor Code entfernt oder gebaut wird.

**Aufgaben.**

1. Mit Toolchain 1.89.0 `cargo build`, `test`, `clippy`, `deny` über den Workspace laufen lassen. Ergebnis als `docs/erasure/BASELINE.md` ablegen, inklusive der drei `#[ignore]`-Tests in `store/src/scanner/tests.rs`.
2. Offene Punkte **\[?\]** klären und mit Datei und Zeile dokumentieren: (a) Länge 78 des Rohschlüssels gegen `TenantKeyCodec`; (b) WAL-Lebenszyklus nach Flush: `truncate` oder Segmentrotation (`wal/io.rs`); (c) Aufbewahrung alter Manifest-Generationen; (d) `fsync`/`rename`-Semantik in `HnswIndex::save_internal`; (e) Persistenzpfad des Graphen; (f) Bereinigung von `CommitIntent`; (g) Nutzung von `CheckpointMeta.metadata`; (h) alle Aufrufer von `Purger`/`ScanTarget`; (i) wie `ScanSession::finish` über `LayerScanner::scan` erreichbar ist.
3. Neue CI-Gates (`rg`-basiert, Skripte unter `tools/`): kein `block_on` in Erase- und Purge-Pfaden; kein `let _ =` und kein `unwrap_or_default()` unter `**/erase*/**` und `**/purge*.rs`; kein `#[tracing::instrument]` ohne `skip` auf Funktionen mit Parametern namens `id`, `key`, `subject`. Letzteres schlägt sofort bei `Collection::delete` an und wird in diesem WP behoben (`skip(self, id)` oder HMAC-Feld).
4. Regressionstest, dass `LsmPurger::purge` bis WP-4 `Err` liefert.
5. ADR-01 bis ADR-09 als Dateien unter `docs/erasure/adr/` einbringen und entscheiden lassen.

**Abnahme.** Baseline-Report committed; jede **\[?\]** hat ein Ergebnis; Gates laufen in CI und sind rot-grün getestet; ADRs entschieden oder mit Eigentümer und Frist versehen.

## WP-1 – Rückbau der alten Löschkette (M)

**Ziel.** Die Legacy-Kette ist vollständig und kompilierbar entfernt, ohne den Normalbetrieb zu ändern. Jeder Schritt ist ein eigener PR mit grünem Workspace.

**Reihenfolge.**

1. **Verbraucher lösen.** `contextra-store/tests/campaign_j42_audit.rs` von `DeletionProofKeyPair` befreien (nur `AuditChain` behalten). In `contextra/src/collection_profile.rs` `KvDeleteMode`, das Feld `kv_delete_mode` und die Invariante `INV-COLLECTION-PROFILE-1` entfernen, `collection_profile_matrix.rs` anpassen. `ContextraError::KvDeleteModeConfig` samt DTO-Zweig entfernen. Das ist ein Bruch der öffentlichen API (Version 0.1.0), im Changelog festhalten; Golden-Tests für `error_dto` **\[?\]** nachziehen.
2. **KV-Segmente entkoppeln.** `kv_segment/manager.rs` und `kv_cipher.rs` verwenden `KeyRegistry`. Ersetzen durch `EphemeralKeyring`: nur RAM, `ZeroizeOnDrop`, kein `Clone`, Zugriff über `with_key(|k| …)`. Entfernen: `kv_segment/delete_mode.rs`, `generate_deletion_proof`, `KvSegmentCipher` hinter `cfg(legacy-erasure)`. Der Eviction-Worker bleibt. Tests `kv_segment_*`, `loom_kv_deferred_zeroize`, `kv_crypto_shred_instant_unrecoverable` bleiben und werden angepasst; `kv_delete_mode_wiring_test` entfällt.
3. **`audit_chain.rs`.** Den `cfg(feature = "legacy-erasure")`-Teil (`EncryptedSalt`, Import von `KeyRegistry`) entfernen. Hash-Kette und Ed25519-Kopfsignatur bleiben.
4. **Module löschen.** `deletion_proof/`, `ed25519_proof.rs`, `kv_shredding.rs`, `revocation_log.rs` und ihre Exporte in `crypto/src/lib.rs`. Features: `legacy-erasure` streichen, `kv-encryption = []`, `default` anpassen. Bench `deletion_proof_latency_bench` samt `[[bench]]`-Eintrag entfernen. `capabilities.toml` und `capabilities.fragment.toml` (`transitional`) bereinigen. Das Fuzz-Crate `crypto/fuzz` auf Verweise prüfen.
5. **Tests löschen.** Alle Dateien in `crypto/tests` mit `deletion_proof`, `ed25519_proof`, `revocation_log`, `legacy_hmac`, `campaign_cov_deletion_proof`, `campaign_cov_revocation_log`, `guarantee_deletion_irrecoverable` **\[?\]**; in übrigen Dateien `cfg(feature = "legacy-erasure")`-Blöcke entfernen (`audit_chain_tests`, `crypto_vectors_extended`, `proptests`, `campaign_j26_hypotheses`, `campaign_mut_j17`, `guarantee_audit_chain_tamper`, `crypto_crossimpl_python`).
6. **`contextra/tests/deletion_e2e.rs`.** `T-DEL-3` löschen. `T-DEL-1` (Canary) behalten, mit `#[ignore = "WP-12"]` und Verweis markieren.
7. **Dokumentation.** `docs/CONTEXTRA_INTERFACE_SPECIFICATION.md` mit `tools/generate_interface_spec.py` regenerieren (nicht von Hand), Crate-READMEs bereinigen.

**Nicht entfernen:** `dek.rs`, `audit_chain.rs` (ohne cfg-Teil), `ports/src/scanner/*`, `durable-fs`, `testkit`.

**Abnahme.** `rg -n "legacy-erasure|KeyRegistry|RevocationLog|DeletionProof|kv_shredding|KvDeleteMode|ed25519_proof" crates docs` liefert außer Changelog und ADRs keine Treffer. Workspace-Build, Tests, Clippy, `deny` und `tests/layering.rs` sind grün. Die API-Differenz ist dokumentiert.

**Risiken.** API-Bruch in `ContextraError`; unbekannte Nutzer von `KvDeleteMode` außerhalb des Workspace; Fuzz-Targets, die auf entfernte Module zeigen.

## WP-2 – Ports v2, Typen, Gate, Crate-Skelett (L)

**Ziel.** Ein einziger Layer-Vertrag mit typisierten Fehlern, ein fail-closed Gate und das neue Crate.

**Aufgaben.**

1. **Typen** in `contextra-types`: `EraseTarget { Document, Subject, Collection, Tenant }` (Konzept §4.2), `ResolvedScope`, `Guarantee { G0, G1, G2, G3 }`, `ReasonCode`, `Exception`, `ErasureProfile { granularity, delete_implies_erase, checkpoint_policy }`. `ErasureId` existiert (16 Byte) und wird als Idempotenzschlüssel genutzt. Der alte `EraseTarget` (Byte-Bereiche) und der Re-Export `PortEraseTarget` in `contextra-ports` werden durch den neuen Typ ersetzt.
2. **`ErasureLayer` v2** in `ports/src/erasure.rs`:

```rust
pub trait ErasureLayer: Send + Sync {
    fn id(&self) -> LayerId;
    fn key_coverage(&self) -> KeyCoverage;
    fn resolve<'a>(&'a self, scope: &'a ResolvedScope, cx: &'a PlanCx)
        -> BoxFuture<'a, Result<LayerScope, LayerError>>;      // read-only, deterministisch
    fn on_gate<'a>(&'a self, s: &'a LayerScope)
        -> BoxFuture<'a, Result<(), LayerError>>;              // RAM-Maskierung, idempotent
    fn purge<'a>(&'a self, s: &'a LayerScope, cx: &'a PurgeCx)
        -> BoxFuture<'a, Result<PurgeOutcome, LayerError>>;    // wiederaufnehmbar über cx.cursor
    fn scanner(&self) -> &dyn LayerScanner;                    // Witness nur über ScanSession
}
pub struct LayerError { pub kind: LayerErrorKind, pub layer: LayerId,
                        pub source: Box<dyn Error + Send + Sync> }
pub enum LayerErrorKind { Retryable, Permanent, Poisoned, ScopeViolation, Cancelled }
```

`PurgeCx` erhält `erasure_id`, `clock`, `deadline`, `io_budget`, `cancel`, `cursor`, `progress`, `gate`, `job_keys`. `PurgeOutcome` erhält `cursor: Option<Cursor>` (`None` = fertig). `witness(&Scanner<Armed>)` und der Platzhalter `Scanner<Armed>` entfallen; gemessen wird ausschließlich über `LayerScanner`/`ScanSession` (INV-WITNESS-MINT-1 bleibt). `StorageBackend` und `ErasureTarget` (leere Marker) werden gelöscht. `Purger`/`ScanTarget` werden `#[deprecated]` und fallen mit WP-4, 6, 7, 8 weg.

**Weitere Aufgaben.**

- **`ReadGate`** ohne Default-Implementierungen für `check_raw`, `gated_seq_raw`, `gate_epoch`. Neu: `check_version(&self, k, version_seq)`, damit eine Wiederanlage desselben Schlüssels nach `Sealed` lesbar bleibt, alte Versionen in der Zeitreise aber gesperrt bleiben. Compile-Fail-Test: Eine Implementierung ohne diese Methoden ist ein Fehler.
- **Crate-Skelett `contextra-erasure`**: in `Cargo.toml` (Members und `default-members`), `[package.metadata.contextra] ring = "2"`, Einträge in `capabilities.toml` und `tests/layering.rs`. Zulässige Abhängigkeiten: `ports`, `types`, `crypto`, `durable-fs`. Keine Abhängigkeit zu `store`, `text`, `vector`, `graph`.
- **`GateTable`** (implementiert `ReadGate`): unveränderlicher `GateSnapshot` hinter `RwLock<Arc<_>>` (ADR-06), Copy-on-Write durch Writer, Leser blockieren nie, `check` ist O(log n) ohne I/O. `GateEntry { Restricted{since}, Erased{up_to} }` mit Wasserstand `floor` je Mandant. Fehlender oder nicht ladbarer Zustand wirkt wie `Restricted` (INV-ERA-7).
- **`GatedStorage<S>`** als Decorator um `StorageEngine` (neben `TenantScopedStorage` **\[?\]**). Er deckt Punkt-Get, `scan`, `scan_prefix_bounded`, `get_at_seq` und `AS OF` ab. `check` vor und nach dem Lesen, bei Epochenwechsel wiederholen.
- **Fehler.** `ContextraError::Erased` ohne Inhalte in den Details, inklusive DTO-Abbildung.

**Abnahme.** Proptest gegen ein Referenzmodell (Gate-Entscheidung für beliebige Folgen aus Publish und Compact); Loom-Test für Epochenmonotonie (Release/Acquire) und „Leser sieht nie Neues vor Epoche“; Benchmark `check` bei 10⁶ Einträgen (Zielwert in WP-0 festlegen); Layering-Test grün; jeder Durchsetzungspunkt hat einen Regressionstest.

**Risiken.** Falscher Schlüsselraum (ADR-02) wäre ein Neubau des Gates; Latenz im Lesepfad; Zeitreise-Pfade, die `GatedStorage` umgehen.

## WP-3 – Journal, Saga, Coordinator, Barriers (L)

**Ziel.** Ein durabler, crash-wiederaufnehmbarer Auftrag mit definierter Gate-Sequenz.

**Aufgaben.**

1. **`ErasureJournal`** auf `durable-fs::AppendOnlyJournal` (CRC, `JournalSyncPolicy` mit `fsync` je Record, `RecordRecoveryPolicy`). Ein abgeschnittener letzter Record darf beim Start verworfen werden, jede andere Beschädigung ist ein harter Fehler (fail-closed, Mandant startet nicht). Records als versioniertes `enum JournalRecord` (bincode mit Formatversion): `Requested{id, target_hash}`, `Planned{plan_hash, layers, salts_ref}`, `GateIntent{gated_seq}`, `Gated`, `KeysDestroyed{vault_gen}`, `LayerProgress{layer, cursor, attempt}`, `Verified{witness_root}`, `Sealed{ledger_index}`, `Completed`, `Failed{layer, kind}`. Das Journal enthält nur HMAC-Hashes, keine Schlüssel und keine Inhalte (INV-ERA-9).
2. **Zustandsautomat als reine Funktion:** `fn decide(state: &SagaState, ev: Event) -> Result<(SagaState, Vec<Effect>), SagaError>`. Ein getrennter Interpreter führt `Effect`s aus (Journal schreiben, Gate publizieren, Layer aufrufen). Dadurch sind alle Übergänge ohne I/O testbar, und die Crash-Matrix (WP-12) kann an jedem Effect-Rand abbrechen. Verbotene Übergänge (z. B. `KeysDestroying` ohne `Gated`) liefern `SagaError::IllegalTransition`.
3. **`ErasureCoordinator`:** Annahme mit Idempotenz (`ErasureId` + `plan_hash`; gleiche Id mit anderem Hash ergibt `IdempotencyConflict`), Planung auf Snapshot `s0` (read-only, Layer-`resolve` parallel), Treiber-Schleife, Wiederaufnahme beim Start (alle nicht abgeschlossenen Aufträge). Backoff mit Jitter über die Ports `Clock` und `Rng`; nach 10 Versuchen `Quarantined(layer)` mit Alarm, stündliche Wiederholung läuft weiter. `Poisoned` (fsync-Fehler) fenced den Layer, keine Wiederholung auf demselben Deskriptor.
4. **Gating-Sequenz (exakt):** `CommitBarrier::exclusive()` → Journal `GateIntent{gated_seq := commit_seq}` (`fsync`) → `GateTable::publish` (Epoche++ mit Release) → `SnapshotRegistry::revoke_covering` → Scope-Transaktionen abbrechen (`RollbackStagedAction`) → Barrier freigeben → Journal `Gated`. Die Barrier wird nie über ein fremdes `.await` gehalten.
5. **`ErasureBarriers`** implementiert `CommitBarrier` und `PurgeBarrier` mit owned, cancel-safe Token (`BarrierShared`, `BarrierExclusive`, `CompactionPermit`, `PurgePermit`). `purge_waiting()` meldet der Compaction, dass sie kooperativ abbrechen soll.
6. **Lock-Rang.** In `store/src/lsm/lock_order.rs` einen Rang für die Erase-Barrier ergänzen (Vorschlag 150, zwischen Dokument-Lock 100 und `commit_mutex` 200; gegen den tatsächlichen Commit-Pfad prüfen **\[?\]**), Register und Doku erweitern und einen Test für `INV-LOCK-ORDER-1` ergänzen.
7. **Boot-Reihenfolge** (Facade-Builder): durable-fs-Health → Journal öffnen und Gates laden (nicht ladbar: Mandant startet ohne Daten im Zustand `Restricted`) → `SaltVault`/Keyring öffnen → WAL-Replay mit Gate-Filter → Layer öffnen → offene Aufträge fortsetzen → Verkehr annehmen. Als Test in der `wiring_matrix` verankern.
8. **Platzreserve:** `reserve_bytes` für Journal und Vault vorab, damit G1 und G2 auf vollem Datenträger gelingen.

**Abnahme.** Idempotenz- und Konflikttest; Crash-Punkte CP-1 bis CP-4 und CP-10 bis CP-12 per `FaultInjector`; mit `ManualClock` deterministisch; Test, dass ein fehlgeschlagener Auftrag das Gate nie öffnet (INV-ERA-7, INV-ERA-11); Loom-Test der Barrier gegen parallelen Commit.

**Risiken.** Deadlock zwischen Barrier und `commit_mutex`; Transaktionsabbruch trifft unbeteiligte Transaktionen (Scope-Filter nötig).

## WP-4 – Store: Erase-Compaction, Flush- und Replay-Gate, WAL-Schnitt (XL)

**Ziel.** Ein einziger Rewrite-Pfad (Compaction-Framework), der Scope-Daten physisch entfernt, ohne fremde Daten zu berühren oder Gelöschtes wieder aufleben zu lassen. Ersetzt den gesperrten `LsmPurger` und schließt K2 endgültig.

**Aufgaben.**

1. **Gate in Flush und Recovery.** `lsm/flush.rs`: wartende immutable Memtables beim Flush filtern. `lsm/recovery.rs`: `covers_raw` **vor** Anwendung jedes WAL-Records (INV-ERA-3). Beides mit Regressionstest, dass ein gegateter Schlüssel nach Neustart nicht in einer SST landet.
2. **`CompactionJobKind::Erase`** in `compaction/engine.rs` mit `ErasePredicate { DropKeys(scope), CollapseVersions(index_keys) }`:
   - Kandidaten: alle SSTables, die Scope-Schlüssel enthalten können (Min/Max, Bloom), über **alle** Level. Ist die Hülle nicht vollständig verarbeitbar, bleibt ein `EraseMarker`, damit ein verworfener Tombstone keine älteren Werte wiederbelebt.
   - `DropKeys` verwirft jede Version mit `seq <= gated_seq` unabhängig von `min_active_seqno`. `validate_tombstone_safety` wird so angepasst, dass Erased-Bereiche zulässig sind; Restricted bleibt unantastbar.
   - `CollapseVersions` lässt bei umgeschriebenen Index-Schlüsseln (z. B. `__txt:`-Postings) nur die neueste Version stehen.
   - Ausgabe über den bestehenden `SstableBuilder` **mit demselben `key_manager`** (verschlüsselte Speicher werden nicht übersehen), neue Datei-IDs, Manifest-Swap als Commit-Punkt, alte Dateien ins `PendingDeleteJournal`.
   - Streaming, keine Materialisierung ganzer Dateien oder Schlüsselmengen. Fehler sind typisiert; kein `let _ =`, kein `unwrap_or_default()` (CI-Gate aus WP-0). `PurgeOutcome` zählt nur, was der Manifest-Swap committet hat.
   - Token-Bucket (`max_io_bytes_per_second`), höhere Priorität als reguläre Compaction, kooperativer Abbruch über `PurgeBarrier::purge_waiting()`. Temporäre Dateien `*.erase.tmp` werden als `LayerProgress` registriert und beim Start entfernt.
3. **`StoreErasureLayer`** implementiert `ErasureLayer` (Memtable, immutable Memtables, SSTs, WAL, Block-Cache-Eviction beim Swap). `LsmPurger` und der Stub in `scanner/purge.rs` werden gelöscht. Die drei `#[ignore]`-Tests in `scanner/tests.rs` werden durch neue ersetzt, darunter der K2-Regressionstest: **Nachbardokument und Nachbarmandant bleiben nach dem Purge bit-identisch lesbar.**
4. **WAL-Schnitt.** Memtables mit Scope-Schlüsseln erzwungen flushen, WAL an einer Segmentgrenze kürzen, Altsegment über `PendingDeleteJournal` entfernen. Es wird nie ein einzelner WAL-Record entfernt, die HMAC-Kette bleibt intakt. Fehlt in `wal/io.rs` eine Segmentrotation (WP-0 **\[?\]**), gehört sie in dieses WP.
5. **`delete_prefix` und `drop_collection`.** `delete_prefix` in begrenzten Batches statt `Vec` aller Schlüssel; die Atomarität ersetzt das Gate (WP-11). Optional WP-4b: `RangeDelete{start, end, seq}` in Memtable, SST und WAL.
6. **Datei-Entsorgung.** `PendingDeleteJournal` → `scrub_if_safe` (Best Effort) → `unlink` → `fsync` des Verzeichnisses. Verweigerter Scrub (Hardlink, CoW, lebender Reader) ist für G3 kein Fehler, weil G2 greift, wird aber als `ScrubRefused(reason)` im Witness protokolliert. `fsync`-Fehler setzen `HealthState` auf degradiert.
7. **Preflight** für Temp-Platz (Summe der Kandidaten-SSTs); sonst bleibt der Auftrag bei G2 mit `InsufficientSpace`.
8. ADR-08 (Tombstone-Konvention) umsetzen und in `types/src/tombstone.rs` testen.

**Abnahme.** Conformance C1 bis C7 (WP-12) für `StoreErasureLayer`; `T-DEL-1` (Canary) ist grün **und löst tatsächlich eine Compaction aus**; Nadel-Scan über SSTs, WAL, Manifest und Checkpoint-Verzeichnisse findet nach G3 nichts; Crash-Punkte CP-8 und CP-9 an jedem `fsync` und `rename` per `fault_vfs`; Benchmark zeigt, dass Erase-Jobs den p99 normaler Lesezugriffe innerhalb des vereinbarten Budgets halten.

**Risiken.** Schreibverstärkung bei breiter Hülle; `ENOSPC`; Hardlink-Klone halten Dateien am Leben (WP-9); Konflikte mit laufender Standard-Compaction.

## WP-5 – Crypto-Erase: SaltVault, KeyService, Sealer (L)

**Ziel.** G2 als Primärgarantie: Nach Vernichtung eines Salts sind alle Kopien des Primärinhalts unlesbar, auch in Checkpoints, Hardlinks und Flash-Resten. Der Anforderungskatalog der Bestandsanalyse (R2, R3, R7, R8) ist hier Abnahmekriterium.

**Aufgaben.**

1. **Hierarchie** auf `crypto/src/dek.rs`: Root (`KeyProvider`-Port) → `UnitKey` je Mandant → `derive_dek(unit, AncestorSalts{t,c,s,d}, DocTag)` → AES-256-GCM-SIV. Die Signatur von `DocTag` und das Verhalten bei fehlender Ebene sind zu prüfen **\[?\]**.
2. **`SaltVault`:** eine Datei pro Mandant, Salts je Segment unter einem Vault-Epoch-Key (VEK) per `wrap_dek` umhüllt. Schreiben als Copy-on-Write: Temp-Datei → `fsync` → `rename` → `fsync` des Verzeichnisses. Vernichtung bedeutet neue Generation ohne den Salt, danach VEK-Rotation (verbleibende Salts unter `VEK(n+1)`, `VEK(n)` per `FileKeyring::destroy` vernichten). Batch-Fenster (Default 1 s oder 1 000 Aufträge), Segmentierung großer Vaults. Recovery für CP-5 bis CP-7 (Generation prüfen, idempotent neu schreiben).
3. **Fail-closed statt stiller Neuerzeugung.** Fehlt ein erwarteter Vault oder ist er beschädigt, startet der Mandant nicht und meldet einen klaren Fehler. Niemals wird still ein neuer Schlüssel erzeugt (das war K3 der Bestandsanalyse).
4. **Schlüsseltypen ohne `Clone`.** `UnitKey`, `Dek`, `Vek` und Job-Schlüssel sind `ZeroizeOnDrop`, ohne `Clone`/`Copy`, `Debug` nur redigiert, Zugriff über `with_key(|k| …)`. Compile-Fail-Tests belegen das (H3/R7 der Bestandsanalyse). Prüfen, ob `UnitKey`/`Dek` in `dek.rs` derzeit `Clone` ableiten **\[?\]**.
5. **`RecordSealer`:** Seal-Rahmen mit Formatversion und AAD (Mandant, Collection, Hash des `DocKey`, Typ, Formatversion). Pflicht im Profil *regulated* für `StoredDocument` (inklusive Embedding), WAL-Payloads, Forward-Index `fw:{doc}`, Graph-Entity-Payload und `CheckpointMeta.metadata`. Nicht versiegelt (G3 ist dort Pflicht): Postings, Term-Dictionary, HNSW-Vektoren, CSR-Struktur. Dual-Read für versiegelt und unversiegelt (Formatversion im Header). Ein Zugriff nach Salt-Vernichtung liefert `Erased`, nicht „Datenfehler“.
6. **`KeyService`** als Fassade: Ableitung, Vernichtung je Ebene, `KeyCoverage`-Meldung je Layer, Ausweis „G2 nicht verfügbar für diesen Scope“, wenn das Profil die gewünschte Feinheit nicht trägt (ADR-03). `destroy` ist idempotent und forward-only.
7. **Job-Schlüssel (`JobVault`)** für layerprivate Zwischenstände (z. B. Termmenge des Textindex), versiegelt, erst bei `Sealed` vernichtet.

**Abnahme.** Neustart-Test: Salt vernichten, Prozess neu starten, Entschlüsselung bleibt unmöglich (R8); Nachbardokument derselben Collection bleibt lesbar (R3); Vault-Crash-Test an jedem Schritt (CP-5 bis CP-7); korrupter Vault führt zu Startverweigerung; Compile-Fail-Tests für `Clone`; Property-Test: `derive_dek` ist deterministisch und trennt Mandanten; Seal/Open-Roundtrip mit manipuliertem AAD schlägt fehl.

**Risiken.** Vault-Größe bei `PerDocument` (ca. 32 Byte je Dokument); Rotationskosten O(Vault); Schlüssel im Prozessspeicher (siehe Grenzen in Konzept §2.2).

## WP-6 – Textindex (M)

**Ziel.** Dokumentbezogene Termdaten vollständig entfernen, ohne den Tokio-Kontext zu blockieren.

**Aufgaben.**

1. `TextPurger` (`text/src/inverted/purge.rs`) wird async und implementiert `ErasureLayer`; `futures::executor::block_on` entfällt (CI-Gate). CPU-lastige Teile über `spawn_blocking` mit Cancel-Prüfung.
2. **Termmenge** wird in `resolve` aus `fw:{doc}` gelesen und als Job-Blob unter dem Job-Schlüssel (WP-5) abgelegt, weil `fw` nach der Schlüsselvernichtung unlesbar ist. Alle späteren Schritte verwenden den Blob.
3. Pro Dokument: Postings aller Terme, `dl:`, `fw:`, `tbs`, leere Term-Dictionary-Einträge (df = 0), BM25-Statistik (N, avgdl) atomar, Morphologie- und Trie-Caches (`BM25MorphIndex`) für entfernte Terme.
4. Posting-Listen sind neue LSM-Versionen; die alten Versionen entfernt WP-4 per `CollapseVersions`. Ohne diesen Schritt bleibt die Dokument-Term-Zuordnung in alten SSTables.
5. Teilfehler werden nie zu Erfolg: `PurgeReport.units_failed > 0` ergibt `Err(Retryable)`.
6. Lesepfad: Kandidatenstrom (BM25/WAND) filtert Postings vor Scoring und Top-k durch das Gate; Hydration prüft erneut.
7. `TextPurger` aus dem deprecated-Zustand lösen und den `Purger`-Adapter entfernen.

**Abnahme.** Conformance C1 bis C7; Needle-Test: Terme des gelöschten Dokuments tauchen weder in Postings noch im Dictionary noch in Statistiken auf; BM25-Scores der Nachbardokumente ändern sich nur im erwarteten Maß (df/N); kein `block_on` im Pfad.

## WP-7 – Vektorindex (L)

**Ziel.** HNSW-Daten werden nicht nur markiert, sondern physisch entfernt, ohne den Recall zu zerstören.

**Aufgaben.**

1. **Phase A (G1):** Maskierung über `deleted_nodes` und Gate, Über-Fetch gleicht Top-k aus, Hydration prüft erneut.
2. **Phase B (G3):** Segment-Rewrite. Neues Segment ohne die Knoten bauen, Nachbarn über einen Reverse-Adjazenzindex (statt O(Layer × N)-Scan) bereinigen und mit der Nachbarschaftsheuristik neu verbinden. Temp-Datei → `fsync` → `rename` → `fsync` des Verzeichnisses → neu mappen → Altmapping nach Reader-Drain (epochenbasiert) freigeben → Altdatei über `PendingDeleteJournal`. mmap-Knoten werden nie nur markiert.
3. RAM-Knoten: `zeroize` von Vektor und Arena-Slots bleibt. Entry-Point je Layer als höchster lebender Knoten.
4. Fehler aus `update_backlink` werden propagiert (kein `let _ =`). Der `write_mutex` wird nicht über Datei-I/O gehalten, nur für den Zeiger-Swap (Shadow-Aufbau).
5. **Recall-Guard:** Nach dem Purge Stichprobe mit Self-Queries gegen den Vorzustand. Fällt der Recall um mehr als 2 Prozentpunkte (konfigurierbar), wird ein Partial-Rebuild aus dem LSM-Bestand mit Gate-Filter geplant.
6. Zielauflösung nur über `ResolvedScope`, nie über das mehrdeutige Parsen von `target_id` (F-10). Koaleszenzfenster `max_rewrite_interval` für G3, G1 bleibt sofort.

**Abnahme.** Conformance C1 bis C7; Nadel-Scan über die mmap-Datei und alle Vorgängerdateien; Recall-Test vor und nach Purge innerhalb der Schranke; Crash-Test an jedem Schritt des Rewrites; kein Datei-I/O unter `write_mutex`.

**Risiken.** Rewrite-Kosten O(Indexgröße); Speicherspitze durch Shadow-Aufbau; Recall-Verlust bei vielen Löschungen.

## WP-8 – Graph und Provenienz (L)

**Ziel.** Primäre und abgeleitete Graphdaten eines Scopes werden vollständig und mandantengenau entfernt.

**Aufgaben.**

1. **Pflicht-Provenienz:** Kanten, Entities, Hyperedges, Community-Zuordnungen und Summaries tragen `derived_from: SmallVec<[DocId; 2]>` (kein `Option`). Betroffen sind `graph/src` und die Erzeuger in `engine/src/extraction.rs` und `decay_controller.rs`. Beim Erase: leere Restmenge ergibt Purge, mehrere Quellen ergeben Entfernen der gelöschten Quelle und `NeedsRecompute` (bis zur Neuberechnung nicht lesbar). Altbestand ohne Provenienz wird als `ProvenanceUnknown` im Plan ausgewiesen und senkt die gemeldete Garantie (Migration in WP-13).
2. **`GraphPurger` neu fassen:** Mandantenprüfung bei jedem Zugriff; kein `TxId::MAX`, kein Umgehen der MVCC-Sicht; Zielauflösung nur über `ResolvedScope` (F-08, F-10). Primäre Entities (Name, Properties, Labels) entfernen, Hyperedges mit weniger als 2 Teilnehmern entfernen. `insert_hyperedge` im Commit erhält eine `TxId` (heute ohne).
3. **Kaskaden-Queue** `__graph:cascade_queue:` (`MAX_HYPEREDGE_CASCADE_FANOUT` synchron, Rest über Queue): Der Fortschritt wird als Layer-`cursor` im Journal geführt. Queue-Einträge enthalten Ids und werden versiegelt oder spätestens bei `Sealed` entfernt, sonst bleibt ein Residuenkanal.
4. **Physisch:** `CsrGraph::compact` schreibt Arrays neu und zeroisiert Property-Arenen. Der Persistenzpfad des Graphen (Snapshots, Dateien) wird inventarisiert (WP-0 **\[?\]**) und in den Witness aufgenommen.
5. **Communities:** betroffene Communities mit Gate-Filter neu berechnen, Summaries aus gelöschten Quellen verwerfen.
6. Lesepfad: Nachbarn, Traversierung und Communities prüfen Provenienz gegen das Gate.

**Abnahme.** Conformance C1 bis C7; Property-Test: Nach Erase ist kein Objekt mit leerer Provenienz erreichbar; Nachbargraph bleibt unverändert, solange er nicht von der gelöschten Quelle abhängt; Crash während der Kaskade setzt über den Cursor fort (CP-8).

**Risiken.** Rückwirkende Provenienz für Altbestand; teure Community-Neuberechnung; Hyperedge-Semantik bei geteilten Quellen.

## WP-9 – Checkpoints, Snapshots, Caches, Commit-Residuen (M)

**Ziel.** Dritte (Pins, Klone, Caches, Commit-Puffer) verhindern weder G1 noch G2 und bleiben für G3 nicht unbemerkt.

**Aufgaben.**

1. **Checkpoints** (`checkpoint/src/store.rs`, `hardlink_cloner.rs`): `restore_checkpoint` prüft das Gate und lehnt `tainted` Checkpoints ab (`seq_no >= scope.min_write_seq`). `CheckpointTaintPolicy { Drop, Rewrite, LegalHold }` mit Default `Drop` nach `checkpoint_taint_grace` (24 h) im Profil *regulated*. Hardlinks: obsolete SSTs werden unter **allen** Namen entfernt, Scrub erst bei `nlink == 1`. `drop_checkpoint` propagiert Entpin-Fehler (heute nur `warn`). `CheckpointMeta.metadata` nur per Allowlist oder versiegelt.
2. **Snapshots und Pins:** `SnapshotRegistry::revoke_covering(tenant, 0..gated_seq, gated_seq)` wird im Gating (WP-3) aufgerufen. Manuell gesetzte Pins erhalten `max_pin_age` (Default 7 Tage) und Alarm bei Überschreitung. Die Sicherheit für Leser kommt vom Gate, nicht vom Entzug.
3. **Caches:** Embedding-, KV-, Query-Ergebnis- und Block-Cache sowie entschlüsselte Schlüssel-Caches werden in `on_gate` nach Scope geräumt. Cache-Schlüssel enthalten die Gate-Epoche, ein Eintrag aus älterer Epoche ist nie ein Treffer.
4. **Commit-Residuen:** `record_keys_with_old_values` und das Kompensations-Ledger halten Altwerte nur versiegelt oder als Referenz und werden nach Commit oder Rollback zeroisiert. `CommitIntent` wird nach Abschluss entfernt (Bereinigung in WP-0 **\[?\]** belegen). Laufende Transaktionen mit Scope-Bezug werden beim Gate abgebrochen, der Commit-Leader lehnt Schlüssel unter Gate ab.
5. **Export/Import:** `export_memories` filtert Gate-Scope, Import prüft das Gate. Exporte außerhalb des Systems werden im Ergebnis als „außerhalb des Scopes“ ausgewiesen.
6. **Telemetrie:** Schlüssel, Subjekt- und Dokument-Ids gelangen nur als HMAC in Spans, Logs und Metriken (INV-ERA-9); das CI-Gate aus WP-0 erzwingt das.

**Abnahme.** Restore eines tainted Checkpoints wird abgelehnt; ein offener Snapshot oder Pin verhindert G1 und G2 nicht, verzögert G3 nur mit Alarm; Hardlink-Test (zwei Namen, ein Inode); Cache-Test mit Epochenwechsel; Zeroize-Test der Commit-Puffer.

## WP-10 – Witness und Ledger (M)

**Ziel.** Gemessene statt behauptete Löschung und ein manipulationserkennendes Betriebsprotokoll.

**Aufgaben.**

1. **`WitnessAggregator`:** ruft je Layer `LayerScanner::scan` auf (einziger Mint-Pfad, `LayerWitness` ohne `Default`/`Deserialize`). Techniken: `Structural`, `DecryptedView`, `NeedleScan` (Byte-Scan über Dateien, WAL, mmap, Checkpoints, Logs). Nadeln (DocKey-Bytes, DocId in LE/BE, Dezimal- und Hexform, Terme aus dem Job-Blob) existieren nur im RAM und im Job-Schlüssel-Blob. Versiegelter Inhalt ist absichtlich nicht auffindbar (das ist G2).
2. **Stufenermittlung:** G3 genau dann, wenn jeder Layer `remaining = 0` meldet. `NotMeasured(reason)` oder `FeatureDisabled` senkt auf G2 und erscheint als `Exception`. `witness_root` ist ein BLAKE3-Merkle-Digest über die Layer-Digests in `LayerId`-Reihenfolge. `remaining > 0` führt zurück nach `Purging` (`attempt++`, begrenzt).
3. **`ErasureLedger`** implementiert `LedgerSink`. Der heutige `LedgerEntry` (`commitment`, `range_set`, `gated_seq`, `completed_at`, `chain_hash`, `ledger_mac`) wird um `erasure_id_hmac`, `guarantee`, `layers_root`, `exceptions`, `plan_hash`, `vault_gen`, `policy_version` erweitert, mit Formatversion. Beobachtung am Bestand: `compute_chained` bindet `ledger_mac` nicht in `chain_hash` ein. Die Spezifikation verlangt, dass der MAC **alle** Felder inklusive `chain_hash` abdeckt und der Verifier beides prüft. `LedgerSink::append` und `read_all` liefern `Result<_, String>`; auf typisierte Fehler umstellen.
4. **Commitment** `= HMAC(ledger_key, plan_hash ‖ erasure_id)`: Ein Prüfer mit Schlüssel kann nachrechnen, ein Dritter kann nichts aufzählen.
5. **Ablage** außerhalb des Datenverzeichnisses (ADR-07), `append` mit `fsync`, Rotation mit `fsync` des Verzeichnisses. Idempotenz über `commitment` (CP-11: kein Doppeleintrag).
6. **Integrität:** `verify_ledger()` prüft Kette und MAC. Bei `LedgerCorrupt` werden Erase-Aufträge abgelehnt (fail-closed), Lesen bleibt möglich. Kürzung am Ende erkennt die Kette nicht; deshalb veröffentlicht ein `LedgerAnchor`-Port periodisch den Kopf-Hash (WORM, Syslog oder externes Log). `LedgerSink::prune` schreibt einen `PruneMarker` mit dem Hash des abgeschnittenen Teils.
7. **Observer:** `ErasureObserver` erhält `ErasureEvent` für `Gated`, `Restricted`, `Completed`. Observer-Fehler blockieren die Saga nie.
8. **Optional (WP-10\*):** Ed25519-Signatur des Kopf-Hashs für Dritte über die vorhandene Kopfsignatur in `audit_chain.rs`. Nicht Teil des Kernumfangs (ADR-01).

**Abnahme.** Test C4: Wird ein Canary absichtlich nicht entfernt, meldet der Scanner `remaining > 0` und die Stufe bleibt G2. Manipuliertes, gekürztes und umsortiertes Ledger wird erkannt (Kürzung nur mit Anker). Zweimaliges `append` desselben `commitment` ergibt einen Eintrag. Ein echter Test ersetzt das frühere `T-DEL-3`: Verifikation mit falschem Schlüssel schlägt fehl.

## WP-11 – API, Engine, MCP, Berechtigungen (M)

**Ziel.** Erase ist explizit und sicher bedienbar. `delete` bleibt, was es ist.

**Aufgaben.**

1. **Facade:**

```rust
impl Contextra {
    pub async fn erase(&self, req: EraseRequest) -> Result<ErasureHandle>;           // kehrt nach G1 zurück
    pub async fn erase_dry_run(&self, t: EraseTarget) -> Result<ErasurePlanSummary>; // Mengen, keine Inhalte
    pub async fn erase_status(&self, id: ErasureId) -> Result<ErasureStatus>;
    pub async fn erase_resume(&self, id: ErasureId) -> Result<()>;                   // Operator, nie ungated
    pub async fn verify_erasure_ledger(&self) -> Result<LedgerVerification>;
}
```

`EraseRequest { id, target, reason, policy, await_level }`, `ErasurePolicy { checkpoint, reinsert, sla, allow_g2_unavailable }`, `ErasureStatus { state, reached, layers, exceptions, ledger_index }`. Wiederholung mit gleicher `ErasureId` liefert den bestehenden Status.

**Weitere Aufgaben.**

- **Retire bleibt Retire.** `Collection::delete`, `forget` und `ctx_retire` schreiben weiter Tombstones (G0); Dokumentation und MCP-Antwort (`semantics: "tombstone"`) bleiben ausdrücklich. Mit `delete_implies_erase = true` (nur Profil *regulated*, ADR-09) löst `delete` zusätzlich `erase(Document)` aus.
- **`drop_collection`** wird zu `erase(Collection)`: Der atomare, durable Fakt ist der Gate-Eintrag. Tombstones und `__col_idx` werden danach in begrenzten Batches geschrieben; das Crash-Fenster zwischen den zwei Transaktionen (F-09) entfällt. HNSW, Graph und Checkpoints laufen über ihre Layer.
- **Subjektindex** `__subj:{tenant}:{subject_hmac}:{doc_id}` wird bei Schreibzugriffen mit `subject` transaktional gepflegt (Pflichtfeld im Profil *regulated*). Ohne Subjektindex ist nur `Document`-Erase möglich.
- **MCP** (`mcp/src/server_tools.rs`): `ctx_erase` (Schreib-Tool, `confirm_required`, `reason` Pflicht, `dry_run` Default `true`) und `ctx_erase_status`; Eintrag in `tool_registry_classification`. `contextra_drop_collection` ruft `erase` auf, die Antwort enthält `semantics: "erase"`, `reached`, `erasure_id`.
- **Berechtigung:** neue Capability `erase` in `capabilities.toml`; `Tenant`-Scope erfordert Administratorrolle. Zugriff auf gegatete Inhalte liefert `ContextraError::Erased`.
- **Verdrahtung** im Facade-Builder (`wiring_matrix`): Layer-Registry, Gate, Barriers, KeyService, Ledger.

**Abnahme.** API-Tests inklusive Idempotenz und Dry-Run (keine Inhalte in der Antwort); MCP-Klassifikationstest; Rechte-Test für `Tenant`-Scope; Test, dass `delete` ohne Profilflag keinen Erase auslöst.

## WP-12 – Testmatrix: Crash, Loom, Conformance, E2E (L)

**Ziel.** Jede Garantie hat einen Test, der ohne sie rot wird. Das WP läuft parallel zu allen anderen, die Layer-Suites sind Teil von deren Abnahme.

**Bausteine.**

1. **Conformance-Suite** in `contextra-testkit/src/conformance/erasure.rs` (Feature `erasure-conformance` existiert): C1 Idempotenz, C2 Wiederaufnahme an jedem Cursor-Punkt, C3 Scope-Isolation (Nachbarmandant und Nachbardokument bit-identisch), C4 Witness-Ehrlichkeit, C5 Fehlerinjektion (jeder injizierte I/O-Fehler ergibt `Err`, nie `Ok`), C6 Cancel-Safety an jedem `.await`, C7 Gate-Respekt nach `on_gate`. Pflicht für jede `ErasureLayer`-Implementierung.
2. **Crash-Matrix** CP-1 bis CP-12 (Konzept §5.1) mit `FaultInjector` und `fault_vfs`, Muster aus `checkpoint_systematic_crash`. Jeder Crash-Punkt prüft den Zustand nach Neustart und das Ergebnis der Wiederaufnahme.
3. **Loom:** Epochenmonotonie und Sichtbarkeit des Gates; Barrier gegen parallelen Commit; Compaction gegen Purge (`purge_waiting`); Eviction-Worker-Shutdown (bestehende Loom-Tests bleiben).
4. **Modelltest:** Referenzmodell im Testkit; zufällige Folgen aus Write, Retire, Erase, Flush, Compaction, Restart, Restore. Invariante: Nach Erase liest das Modell `Erased`, Nachbarn bleiben unverändert, kein Wiederaufleben.
5. **Resurrection-Tests:** Gelöschtes bleibt unlesbar nach Flush, Compaction, WAL-Replay, Index-Rebuild, Backfill, Restore und Konsolidierung.
6. **E2E** in `contextra/tests`: `T-DEL-1` (Canary nach echter Compaction, 0 Treffer), Neustart nach Salt-Vernichtung, Scope-Isolation zwischen Mandanten, Zeitreise auf alte Versionen, Restore eines tainted Checkpoints, Ledger-Manipulation, `drop_collection` mit Crash.
7. **Fehlerinjektion:** `EIO`, `ENOSPC`, `fsync`-Fehler, Torn Writes, Reordering.
8. **Mutationstests:** Garantien aus `mutants-guarantees.toml` um Gate, Saga-Reihenfolge, Witness und Ledger erweitern.
9. **Statische Gates:** Compile-Fail-Tests für `LayerWitness`, Schlüsseltypen ohne `Clone` und `ReadGate` ohne Defaults; `rg`-Gates aus WP-0.

## WP-13 – Migration, Seal-on-Rewrite, Betrieb (M)

**Ziel.** Bestandsdaten und Betrieb ziehen nach, ohne falsche Garantien zu behaupten.

**Aufgaben.**

1. **Seal-on-Rewrite:** Compaction versiegelt unversiegelte Records beim Umschreiben (Hook aus WP-4/5). Zusätzlich ein Backfill-Job, rate-limitiert, wiederaufnehmbar, mit Fortschritt im Journal-Stil. Bis zum Abschluss weist der Plan für Altbestand nur G1 und G3 aus.
2. **Provenienz-Backfill** für Altbestand; `ProvenanceUnknown` bleibt im Plan sichtbar und senkt die Garantie, bis die Migration abgeschlossen ist.
3. **Betriebsvorgaben:** kein Swap, keine Core-Dumps; Backups unter eigenen, vernichtbaren Schlüsseln; Ledger-Pfad außerhalb des Datenverzeichnisses; `LedgerAnchor` angebunden; Platzreserven für Journal und Vault.
4. **Runbooks** für `Quarantined(layer)`, `Poisoned`, `InsufficientSpace`, `LedgerCorrupt`, hängende Pins, abgelaufene Wiederaufnahme. Metriken und Alarme (Dauer je Stufe, Quarantäne, Gate-Größe, Ledger-Alter), nur mit HMAC-Labels.
5. **Rollout:** Erase hinter einem Feature-/Profilschalter in Stufen (nur Dry-Run → `Document` → `Subject` → `Collection` → `Tenant`). Ab G1 gibt es kein Zurück (INV-ERA-11); Rollback bedeutet daher: Neuaufträge stoppen, laufende zu Ende führen.
6. **Nutzerdokumentation:** Garantiestufen, Grenzen (externe Kopien, Flash, Heap/Swap), Unterschied Retire und Erase.

**Abnahme.** Migrationslauf auf einem Testdatenbestand mit gemischtem Format; Plan weist die reduzierte Garantie korrekt aus; Runbook-Übung für zwei Fehlerklassen.

## 5. Fehlerpolitik und Konsistenz (verbindlich)

| Klasse | Beispiele | Reaktion |
| --- | --- | --- |
| `Retryable` | transientes `EIO`, Lock-Timeout, `ENOSPC` | Backoff 1 s bis 5 min mit Jitter, nach 10 Versuchen `Quarantined(layer)` mit Alarm, stündliche Wiederholung läuft weiter |
| `Permanent` | ungültiger Scope, `ScopeAmbiguous`, Mandantenverletzung | vor G1 beenden, danach Quarantäne mit Alarm |
| `Poisoned` | `fsync`-Fehler | Layer fencen, `HealthState` degradiert, Datei neu öffnen und per Prüfsumme validieren, Auftrag bleibt `Gated` |
| `ScopeViolation` | Layer wollte Nachbardaten berühren | Abbruch und Sicherheitsalarm |

Die Sichtbarkeit ist durch das Gate atomar, die Layer konvergieren idempotent. Die Mehrschicht-Konsistenz des Commits (`CommitIntent`, Kompensation) wird davon nicht vorausgesetzt.

## 6. Definition of Done und Release-Gates

1. Kein Treffer für Legacy-Symbole (WP-1) und keine `rg`-Verstöße (WP-0).
2. Alle sieben Conformance-Prüfungen sind für Store, Text, Vektor und Graph grün.
3. Crash-Matrix CP-1 bis CP-12 ist grün, jeweils mit systematischer Fehlerinjektion.
4. `T-DEL-1` (Canary nach echter Compaction) und der Neustart-Test nach Salt-Vernichtung sind grün.
5. Ein Nadel-Scan nach G3 findet in keiner Datei Scope-Bytes; Ausnahmen stehen als `Exception` im Ergebnis.
6. Das Gate ist in allen Durchsetzungspunkten aus Konzept §4.4 mit je einem Regressionstest belegt.
7. Ledger-Verifikation erkennt Manipulation; `LedgerCorrupt` verweigert Erase und erlaubt Lesen.
8. `cargo build`, `test`, `clippy -D warnings`, `deny`, `tests/layering.rs`, `tests/unsafe_islands.rs` sind grün.
9. Benchmarks halten die vereinbarten Budgets für Lesepfad (Gate), Erase-Compaction (Einfluss auf p99) und Vault-Rotation.

## 7. Risikoregister

| Risiko | Wirkung | Gegenmaßnahme |
| --- | --- | --- |
| Gate-Schlüsselraum passt nicht zum Store-Layout (ADR-02) | Neubau des Gates | WP-0 belegt `TenantKeyCodec` vor WP-2 |
| Compaction-Hülle unvollständig | Wiederaufleben alter Werte | `EraseMarker`, Resurrection-Tests in WP-12 |
| Lock-Inversion Barrier/`commit_mutex` | Deadlock im Commit | eigener Rang, Loom-Test (WP-3) |
| Vault-Rotation O(Größe) | Latenz bei vielen Aufträgen | Batch-Fenster, Segmentierung (WP-5) |
| HNSW-Rewrite senkt den Recall | Qualitätsverlust | Recall-Guard, Partial-Rebuild (WP-7) |
| Altbestand unversiegelt, ohne Provenienz | nur G1 und G3 möglich | ehrlicher Ausweis, Migration (WP-13) |
| Pins, Snapshots, Klone halten Daten | G3 verzögert | G1/G2 unabhängig, Alarm, `Drop`-Politik (WP-9) |
| API-Bruch durch Entfernen von `KvDeleteMode` und Fehlervariante | Downstream-Fehler | Changelog, Versionierung (WP-1) |
| Statische Analyse ohne Build | unentdeckte Kompilierfehler und Annahmen | WP-0 Baseline, alle **\[?\]** vor Beginn belegen |

## 8. Offene Fragen

1. Wie lang ist der Rohschlüssel tatsächlich, und passt `KeyBuf` (16 Byte) zu den Gate-Prüfungen im Store (ADR-02)?
2. Hat der WAL Segmentrotation oder nur `truncate`?
3. Werden alte Manifest-Generationen aufbewahrt, und enthalten Footer Schlüsselbytes?
4. Wo liegt der Persistenzpfad des Graphen, und gibt es dort Klartext?
5. Leiten `UnitKey` und `Dek` heute `Clone` ab?
6. Welche Standard-Granularität (ADR-03) und welche SLA-Werte für G3 gelten je Profil?
7. Gibt es Nutzer von `KvDeleteMode` und `ContextraError::KvDeleteModeConfig` außerhalb dieses Workspaces?
8. Soll die optionale Ed25519-Kopfsignatur des Ledgers (WP-10\*) im ersten Release enthalten sein?
