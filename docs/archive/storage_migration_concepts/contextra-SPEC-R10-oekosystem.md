# SPEC-R10 — Contextra-Ökosystem: Integration von `contextra-storage` (R9.1)

| | |
|---|---|
| Status | **Entwurf zur Freigabe.** Ergänzt `crates/contextra-storage/SPEC.md` (R9.1). R9.1 bleibt für den Speicherkern normativ; R10 regelt Crate-Topologie, Verträge zwischen Crates, Migration und Sofortmaßnahmen. Bei Widerspruch gilt R9.1 für den Kern, R10 für alles darüber. |
| Stand Analyse | 2026-11-24, Snapshot `ccc6934` von `tfufuz1/contextra`. Fünf Read-only-Analysen (Erasure/Crypto/Privacy, Datenschicht, Cognition/RAG, Infrastruktur/Fassade, Storage-Ist-Zustand). **Es wurde nichts gebaut oder getestet.** Befunde mit „verifizieren“ sind aus dem Code abgeleitet und müssen per Test bestätigt werden. |
| Ziel | Ein Gesamtsystem, in dem jede Crate genau eine Verantwortung hat, Löschung über **einen** Mechanismus läuft, und jeder Verbraucher den Kern nur über stabile Ports anspricht. |

---

## 1. Zusammenfassung

1. **Der Speicherkern existiert nur als Fragment.** Die Dokumentation (PHASE-STATUS: „Check grün“) beschreibt mehr, als der Code tut. Das Crate ist nicht im Workspace, eine Schreib-/Lese-Fassade fehlt, Recovery OP1–OP10 ist nicht verdrahtet, alle Fuzz-Ziele sind Platzhalter, und die Invariantenprüfung bricht ab (11 von 52 IDs ohne Testreferenz). Eine Migration auf diesen Stand ist nicht möglich, bevor M0 und M2 stehen.
2. **Erasure ist dreifach implementiert:** `contextra-erasure` (≈ 23 000 Zeilen, Orchestrator, nicht von anderen Crates genutzt), `contextra-storage/erase/*` (≈ 970 Zeilen, Mechanismus-Bausteine) und `contextra-store` plus `contextra-ports::gate` (Read-Gate, Store-seitig). Dazu kommt eine eigene, nicht standardkonforme Krypto in `contextra-erasure/crypto_local.rs`.
3. **`contextra-erasure` wird nicht gelöscht, sondern zerlegt:** Orchestrierung, Bericht, Signatur und Risikoauswertung bleiben als Schicht **oberhalb** des Kerns (SPEC NZ-14). Mechanismus, Krypto und Duplikate fallen weg.
4. **Mehrere Sicherheitsbefunde müssen vor jeder Migration geschlossen werden:** öffentlicher `AdminCap`-Konstruktor, möglicher Fail-open nach `emergency_wipe`, Passphrasen-Ableitung ohne Stretching im Produktpfad, deterministische Nonce in `crypto_local`, nicht durchgesetzte `RestrictedReadCap`.
5. **Konsumenten sprechen den Kern nicht direkt an, sondern über einen Kompatibilitätsport**, der zuerst auf `contextra-store` und später auf den neuen Kern zeigt. So bleibt jede Migrationsstufe testbar und umkehrbar.

---

## 2. Befunde der Bestandsaufnahme

Schweregrad: **K** kritisch (Sicherheit/Datenverlust/Löschversprechen), **H** hoch (blockiert Integration), **M** mittel, **N** niedrig/Hygiene.

### 2.1 Speicherkern (`contextra-storage`)

| ID | Sev. | Befund | Beleg |
|---|---|---|---|
| E10-01 | H | Kein Workspace-Mitglied. Cargo bricht für die Storage-Manifeste mit „believes it's in a workspace“ ab. Der grüne Check in PHASE-STATUS ist mit diesem Stand nicht reproduzierbar (aus Manifesten abgeleitet, **per Cargo zu verifizieren**). | Root-`Cargo.toml` members/exclude; `crates/contextra-storage/crates/*/Cargo.toml` |
| E10-02 | H | Pfadsystem widerspricht sich: SPEC §3.1 nennt `crates/contextra-storage(-ports)`, real liegt der Kern unter `crates/contextra-storage/crates/contextra-storage`. Mehrere Verweise (PHASE-STATUS-Pfad in §0, `docs/implement/*` in `00-INDEX.md`, `IMPLEMENTATION-FOUNDATION.md`) zeigen ins Leere. | SPEC §0, §3.1; `docs/00-INDEX.md` |
| E10-03 | H | Fehlende Workspace-Dependencies: `crossbeam-skiplist`, `chacha20poly1305`, `hkdf`, `sha2`, `zeroize`. `thiserror = "1"` im Kern gegen `"2"` im Workspace. `publish.workspace = true`, aber `[workspace.package]` kennt kein `publish`. | Storage-Manifeste; Root `[workspace.dependencies]` |
| E10-04 | H | Keine Schreib-/Lese-Fassade. `Storage` bietet nur `open` und `catalog()`. Recovery OP2, OP5, OP6, OP8, OP10 sind nicht verdrahtet. Redo-Cut (ER4) und Torn-Tail sind Stubs. | `lib.rs`; `recovery/`; `store/redo/cut.rs` |
| E10-05 | H | Erase-Protokoll ER0–ER7 nicht als Protokoll umgesetzt; `erase/protocol.rs` und `erase/receipt.rs` sind Stubs (je 9 Zeilen). `verify_receipt` existiert nirgends. | `erase/` |
| E10-06 | H | Fuzz: 0 effektive Ziele. Die 8 Dateien unter `fuzz/` haben kein `fuzz_target!`, keine `fuzz/Cargo.toml`. | `fuzz/fuzz_targets/*` |
| E10-07 | H | Formale Modelle M1–M6: nur `formal/README.md`, keine `.tla`-Datei, kein Modelllauf. | `formal/` |
| E10-08 | M | Crash-Sweeps nur für Katalog und Redo. Nicht abgedeckt: Run/Flush, Ledger, Erase-Gate, Blob, Checkpoint, Ende-zu-Ende. `RenameReorder`, `ShortRead`, `BitFlip` sind definiert, aber ungenutzt. | `tests/catalog_crash_sweep.rs`, `tests/crash_sweep_redo.rs` |
| E10-09 | M | Invariantenprüfung bricht ab: 11 von 52 IDs ohne Testreferenz (u. a. INV-ADM-1, INV-BR-5, INV-ERA-13, INV-KEY-5, INV-LED-2, INV-PIPE-1/2, INV-POL-1, INV-PRUNE-1, INV-VER-1, INV-VIEW-3). `tests/invariant_refs.rs` ist `assert!(true)`. | `scripts/check_invariant_refs.sh` |
| E10-10 | M | Testanzahl: TRACE nennt 60 Integrationstests, Zählung ergibt 93. Unit-Tests im Kern: 0. | `docs/TRACE.md` |
| E10-11 | M | `RealFs` ist Platzhalter (4 Zeilen). `env/sim.rs` ist Stub (kein SimClock/SimEntropy/SimKms). FaultFs und MemFs sind implementiert. | `contextra-storage-fs/src/realfs.rs`; `env/sim.rs` |
| E10-12 | M | Redo-Kopf weicht von SPEC §5.5 ab (Magic `R8F1`, Felder `caller/origin/caller_seq/ttl_bucket`, kein `len u32`). Das Format ist noch nicht eingefroren; die Abweichung ist vor G-F zu bereinigen. | `store/redo/frame.rs` |
| E10-13 | M | `seal_frame` wird im Redo-Pfad nicht aufgerufen (nur Blob-Record und Tests). | Aufrufsuche |
| E10-14 | H | **Sicherheit:** `AdminCap::from_authorized_context()` ist `pub fn`; jeder Aufrufer kann eine Admin-Fähigkeit erzeugen. Die Dokumentation verlangt, dass nur die Embedding-Autorisierung das tut. | `erase/restrictions.rs:81` |
| E10-15 | N | Doku-Drift: `formal/README.md` heißt „Formale Modelle für R8“; `docs/`-Index verweist auf R8/R7-Stände. | `formal/README.md` |
| E10-16 | N | Lint-Hygiene: keine eigene `clippy.toml`; `HashMap`/`HashSet` in Kernlogik (`erase/gate.rs`, `engine/memtable.rs`, `keys/cache.rs`, `store/run/table_cache.rs`), obwohl die Root-`clippy.toml` sie verbietet. `tests/layering.rs` kennt keine Storage-Ringe. | Root `clippy.toml`; `tests/layering.rs` |

### 2.2 Erasure und Krypto

| ID | Sev. | Befund | Beleg |
|---|---|---|---|
| E10-20 | K | **Fail-open nach `emergency_wipe` (aus Code abgeleitet, zu verifizieren):** Schlüssel wird auf Null gesetzt und der Cipher-Cache geleert; `cipher()` baut danach aus dem Nullschlüssel neu auf; `encrypt_auto_nonce` prüft keinen Wipe-Zustand. Der Test prüft nur die Nullung. | `contextra-crypto/src/crypto.rs:392-450`; `tests/campaign_j25.rs:346-358` |
| E10-21 | K | **Deterministische Nonce in `crypto_local.rs`:** BLAKE3-XOF-Keystream, Nonce aus demselben Hash, öffentlich im Envelope. Gleiche Eingabe ⇒ gleicher Keystream. Kein Standard-AEAD. | `contextra-erasure/src/crypto_local.rs:136-190` |
| E10-22 | H | Passphrasen-Ableitung im Produktpfad: `KeyManager::try_new` leitet per HKDF-SHA256 direkt aus der Passphrase ab, ohne Stretching. Produktive Aufrufer: `contextra-store` (`lsm/recovery/startup.rs:97`), `contextra-mcp` (`sandbox.rs:281`). Argon2id existiert nur in `try_new_with_kdf`. | `contextra-crypto/src/crypto.rs:105`; `kdf.rs:300` |
| E10-23 | H | Nonce-Präfix nur 32 Bit (`encrypt_auto_nonce`). Der Doc-Kommentar behauptet, dies sei der einzige sichere Pfad; `seal_record_with_nonce` ist öffentlich mit Caller-Nonce. | `crypto.rs:406-430`; `aead.rs` |
| E10-24 | H | `contextra-erasure` ist ein Orphan: keine `contextra-*`-Abhängigkeit; `lib.rs` behauptet „AR-3 erlaubt types, ports, crypto“, das Manifest widerspricht. Nur Workspace-Root und eigenes Fuzz-Crate referenzieren es. | `contextra-erasure/Cargo.toml`, `src/lib.rs:4-6` |
| E10-25 | H | Typ-Duplikate in `contextra-erasure`: `EraseReceipt` (zweimal), `ErasureStatus` (zweimal), `ErasureReport` (zweimal), `InMemoryLedger` (zweimal), `BoxFuture` (viermal). `erasure_legacy.rs` re-exportiert aus `contextra-erasure-spi`, das nicht existiert. | diverse; `contextra-ports/src/erasure_legacy.rs` |
| E10-26 | H | Globaler Zustand: `static GLOBAL_GATE_SET` in `gate/batch.rs:197-212` (prozessweiter Singleton). Widerspricht instanzlokalen IDs und NZ-19. | `gate/batch.rs` |
| E10-27 | H | Bericht wird vor Ledger und `Completed` signiert, aber die Bytes werden verworfen (`let _report_bytes = …`). Widerspricht der ER7-Reihenfolge. | `saga/core.rs:739-742` |
| E10-28 | H | Schlüssel-Abstraktionen vierfach: `storage-ports::KeyProvider`, `contextra-ports::keys::{RecordSealer, FileKeyring}`, `erasure::vault::KeyStore`, `erasure::vault::VaultKeyring`. Keine ist mit SPEC §9.4 vollständig (kein Batch, `probe` ohne Epoche, keine Tomb-API, `keys_in_backups` als `bool`, kein `KeyAssurance`). | `storage-ports/src/keys.rs:60-75`; `erasure/vault/*` |
| E10-29 | H | Erase-Stufen nicht SPEC-konform: `SagaState` hat Pending/Restricted/Lifted/Refused, keine Level-Achse, keine Stage-Achse, keine Statuscaps (`BoundedBackupKeys`, `WeakKeyAssurance`, `LogicalOnly`). Epochen fehlen: `destroy_highest_key` vernichtet nur eine Ebene; INV-KEY-4 verlangt alle. | `saga/machine.rs:27-50`; `saga/seams.rs:74`; `saga/core.rs:508` |
| E10-30 | H | Signaturprüfung uneinheitlich: `verify_strict` in `report/signing.rs` und `ed25519_proof.rs`, normales `verify` in `deletion_proof.rs` und `revocation_log.rs`. | siehe Pfade |
| E10-31 | M | `contextra-crypto` hat `deletion_proof`-Fuzz-Targets, das Modul existiert in `contextra-crypto` nicht. Fuzz-Ziele veraltet. | `contextra-crypto/fuzz` |
| E10-32 | M | `contextra-erasure` Fuzz: ein Target, das nur `blake3::hash` aufruft. Kein Parser-Fuzz. | `contextra-erasure/fuzz` |
| E10-33 | M | Krypto-Stack dreifach: AES-256-GCM-SIV (`contextra-crypto`), XChaCha20-Poly1305 (`contextra-crypto-ref`), BLAKE3-Stream (`contextra-erasure`). Dazu Kopien in `kv_shredding.rs` (AES-GCM-SIV mit OsRng-Nonces). |  |
| E10-34 | M | Ledger dreifach: `storage/erase/ledger.rs`, `erasure/ledger/*` (zwei Definitionen), `contextra-ports/src/ledger.rs`. | |
| E10-35 | M | `contextra-erasure`-Messmodul (`measure/`, `residue.rs`, Typestate-Scanner) ist die einzige produktive Restdaten-Messung im Repo und hat keinen Gegenpart im Storage-Kern. | `contextra-erasure/src/measure*` |

### 2.3 Privacy, Vault, Prompt-Guard

| ID | Sev. | Befund | Beleg |
|---|---|---|---|
| E10-40 | M | `contextra-privacy`: Surrogat = BLAKE3(Entität ‖ Session-Salt), auf 64 Bit gekürzt. Map wird beim Clear gelöscht. Keine `contextra-*`-Abhängigkeit, 166 Tests, kein Fuzz. Surrogat-Vault hält personenbezogene Daten, die bisher nirgends einem Cluster zugeordnet sind. | `egress_vault.rs:97-147` |
| E10-41 | M | `contextra-vault` (excluded): flüchtiger RAM-Staging-Tresor, nur von `contextra-db` genutzt. Namensdublette zu `erasure::FileVault`. | `contextra-vault/src/vault.rs:205` |
| E10-42 | N | `contextra-prompt-guard`: kein Storage-Bezug, kein Krypto. Audit-Datei nur über Feature. Bleibt unverändert. | |

### 2.4 Datenschicht und Konsumenten

| ID | Sev. | Befund | Beleg |
|---|---|---|---|
| E10-50 | H | `contextra-store` (LSM) kennt keine Spaces, Cluster, Stamps, Receipts, Branches. Schreibsemantik ist TxId-basiert, Lesen über `Vec`-Rückgaben. Die Spec-Begriffe sind anders benannt und zum Teil anders definiert. | `contextra-store/src/lsm/engine/facade.rs` |
| E10-51 | H | Einziger produktiver Hauptkonsument: `contextra-db` (≈ 60 Imports inkl. Tests). Seine **öffentliche API** exponiert Store-Typen: `DurabilityMode`, `TenantScopedStorage<Arc<LsmStorage>>`, `SystemPressure`. | `contextra-db/src/lib.rs:61,331-334` |
| E10-52 | H | Löschung umgeht den Vfs-Seam: `compat/scanner/purge.rs` überschreibt SST/WAL mit Nullen über `tokio::fs`. Die Compaction nutzt dagegen den Vfs. | `contextra-store/src/compat/scanner/purge.rs:142-146, 218-224` |
| E10-53 | H | `derive_doc_id` ist unkeyed BLAKE3 über den Schlüssel: deterministisch, aber linkbar. Widerspricht INV-PRIV-1 in der Intention. | `contextra-store/src/lsm/validate.rs:29-38` |
| E10-54 | H | Checkpoint nutzt Hardlinks auf `.sst`-Dateien; `hardlink_cloner.rs` scannt das Datenverzeichnis direkt. `hardlink_cloner.rs:51` implementiert für `contextra_mvcc::SnapshotRegistry`, die Checkpoint-`Cargo.toml` hat aber keine mvcc-Abhängigkeit (zu verifizieren). | `contextra-checkpoint/src/hardlink_cloner.rs` |
| E10-55 | H | Graph: Doc-Purge tombstoned Kanten nur im RAM. Beim Neustart lädt `load_from_storage` sie wieder. **Die Löschung ist nicht durabel.** | `contextra-graph/src/csr/graph_write/tombstone.rs`; `compat/purger.rs:53-70` |
| E10-56 | H | KV-Cache: schreibt über `StorageEngineKvBridge` direkt in den LSM, ohne Segment-Owner, ohne Gate. `delete_tenant` schreibt einen leeren Wert statt zu löschen. Tier-2 ist Klartext (`cipher: None` als Default). Tx-ID = `segment_id.max(1)`, Kollisionen zwischen Mandanten nicht geprüft. | `contextra-kvcache/src/service.rs:129,549-556`; `compat/adapters.rs:42-56` |
| E10-57 | H | Agent: Checkpoint-Metadaten enthalten das komplette Gedächtnis je Schritt (Klartext). `drop_checkpoint` hat keinen Produktionsaufrufer. DLQ-Dedup ist Vollscan. | `contextra-agent/src/engine.rs:596-606`; `contextra-checkpoint/src/store.rs:615` |
| E10-58 | M | Cognition: Quellen nur als `source_doc_ids` im Metadatum, keine `DerivedFrom`-Lineage. Mehrquellen-Synthesen können Personen mischen (verstößt gegen §12.8.21). `text` steht doppelt im Metadatum. | `contextra-cognition/src/consolidation_executor.rs:335-362` |
| E10-59 | M | Ingest bewahrt E-Mail-Header (From/To/Cc) und MSG-Absender bewusst. Für personenbezogene Daten ist das eine Klassifikationsentscheidung, die oben getroffen werden muss. | `contextra-ingest/src/formats/email/eml.rs:54-58` |
| E10-60 | M | Vector-Persistenz schreibt eigene Dateien und Sidecars (`pending.wal`, `tombstone.wal`) außerhalb der Kontrolle von Store/Kern; Lösch-Tombstones liegen dort. | `contextra-vector/src/persistence/*`, `diskann/io.rs` |
| E10-61 | M | MVCC doppelt: `contextra-mvcc` (TxBuffer, SequenceLog, SnapshotRegistry, SSI), `contextra-store` (eigene MVCC-Memtable, Fence), `contextra-vector` (eigene SequenceLog-Instanz), `contextra-storage` (Registry, Views, Guard-Fenster). | |
| E10-62 | M | Snapshot-Pins laufen still ab (`DEFAULT_MAX_PIN_DURATION` 300 s); SPEC verlangt `SnapshotTooOld`. | `contextra-mvcc/src/seq_log.rs:38` |
| E10-63 | M | Observer verwerfen bei Überlast (Drops, Circuit-Breaker); SPEC verlangt `Lagged` mit persistenten Cursorn. | `contextra-store/src/lsm/observer/types.rs` |
| E10-64 | M | Graph und DLQ lesen beim Start den gesamten Store (`scan_prefix(b"")`). Ohne Consumer-Feed mit Watermark skaliert das nicht. | `graph_persist.rs:224`; `dlq.rs:200-230` |
| E10-65 | M | Hart verdrahtete Schlüsselformate außerhalb der Collection-API (`__col:agent:…`, `__graph:*`, `__session_dag:*`, `{ns}:checkpoint:`). Migration muss sie übersetzen. | diverse |
| E10-66 | M | Trust-Label (§12.6) in keinem der neun Cognition-/Rank-Crates umgesetzt. | grep ohne Treffer |
| E10-67 | M | Router liest Kalibrierungsdatei per `std::fs::read`; kein Schreibpfad; `pending_decisions` nur RAM. | `contextra-router/src/seam.rs:45` |
| E10-68 | N | `contextra-checkpoint` `orphan.rs` schreibt per `std::fs::write`, nicht über den Fs-Port. | `orphan.rs:257,430` |

### 2.5 Fassade, Schnittstellen, Infrastruktur

| ID | Sev. | Befund | Beleg |
|---|---|---|---|
| E10-70 | H | **Löschung ohne Beleg nach außen:** MCP `contextra_delete` und `contextra_forget` rufen `col.delete(id)` und liefern `{"deleted": true}`. Kein Receipt, kein Level, kein Status. `forget` ist als Alias für „ohne Verifikationsbeleg“ dokumentiert. | `contextra-mcp/src/server_dispatch.rs`; `tools_crud.rs:144` |
| E10-71 | H | Python-Bindings haben `delete` und `insert`, aber keine Erase-, Memory- oder Konversations-Operation. | `contextra-py/src/bindings/crud_macros.rs:120` |
| E10-72 | M | Fassade `AgentMemory::forget` → `engine.delete`; keine Erase-Semantik. | `contextra/src/agent_memory.rs:266` |
| E10-73 | H | Zwei Port-Welten: `contextra-ports` (Domänen-Ports, Vfs, Clock, keys, erasure, ledger, gate, error) und `contextra-storage-ports` (Kern-Ports). Gleiche Konzepte, andere Signaturen. | Crates |
| E10-74 | H | **Clock-Semantik-Konflikt:** Core `contextra-env::Clock::now_unix_nanos()` liefert bei Pre-Epoch still `0`. Storage-`Clock` ist fail-closed mit Skew/Durable-Floor. Ein Adapter, der den Core-Clock weiterreicht, würde `0` als Frist durchreichen. | `contextra-env/src/clock.rs:18`; `storage-ports/src/traits/anchor.rs:59` |
| E10-75 | M | Entropie: Storage-`Entropy::fill -> Result`; Core-`Rng` ist infallible. | `anchor.rs:70`; `contextra-env/src/rng.rs:32` |
| E10-76 | M | Filesystem dreifach: `contextra-ports::vfs` (pfadbasiert), `contextra-durable-fs::Vfs` (mit `RealFs = StdVfs`, Namensbelegung), Storage-`Fs`. Storage-`RealFs` fehlt. | `contextra-durable-fs/src/vfs.rs:60`; `storage-ports/traits/fs.rs` |
| E10-77 | M | Zwei Null-Reservierungs-Konventionen: `TenantId(SYSTEM=0)` in `contextra-ids` und `SpaceId` 0..255 reserviert. Ohne Mapping-Regel Verwechslungsrisiko. | `contextra-ids/src/lib.rs:79,99`; `storage-ports/ids.rs:41-50` |
| E10-78 | M | Fehler dreifach: `contextra-types::error`, `contextra-ports::error` (`ErrorClass`), Storage-`error.rs`. | |
| E10-79 | M | Workspace-Hygiene: Storage-Crates nicht in `capabilities.toml`, `layering`-Test kennt sie nicht, Mutations-Scope (`.cargo/mutants.toml`) enthält sie nicht, `deny.toml` ohne Storage-Eintrag. | |
| E10-80 | N | Toolchain ist korrekt auf 1.89.0 gepinnt; lokal ist 1.85.1 installiert (TRACE). | `rust-toolchain.toml` |
| E10-81 | N | `contextra-bench` (Workspace-Mitglied) hat keine Storage-Abhängigkeit; Benchmark-Gruppen aus SPEC §17 existieren nicht. | `benchmarks/contextra-bench` |

### 2.6 Positive Befunde

- Siegelrahmen (`codec/seal.rs`) und XChaCha20-Poly1305 mit 24-Byte-Nonce in `contextra-crypto-ref` entsprechen SPEC §5.2 (E-68 im Code erfüllt).
- Redo-Kettenlogik, Kinds 1–4 und `append_group` sind vorhanden.
- Ledger-Hash-Kette mit MAC und Manipulationstests ist vorhanden (4 Tests).
- `contextra-crypto` hat solide Tests (247 `#[test]`, Golden- und RFC-Vektoren, Loom, Proptests, Nonce-Stress). Das Fundament für die Migrationsdaten ist also brauchbar.
- `contextra-prompt-guard` und `contextra-privacy` sind sauber getrennt.
- `contextra-store` ist ein funktionierender Altstand; er muss nicht ersetzt werden, bevor der Ersatz getestet ist.

---

## 3. Zielbild: Schichten und Crate-Topologie

```
┌───────────────────────────────────────────────────────────────────────────┐
│ Oberste Ebene: Fassaden und Schnittstellen                                 │
│   contextra (Fassade) · contextra-mcp · contextra-py · contextra-wire      │
├───────────────────────────────────────────────────────────────────────────┤
│ Anwendungs-Ebene: Wissen, Gedächtnis, Agenten                              │
│   contextra-db · contextra-ingest · contextra-graph · contextra-cognition  │
│   contextra-rank · contextra-text · contextra-vector · contextra-agent     │
│   contextra-router · contextra-sandbox · contextra-kvcache(Cache-Rolle)    │
│   contextra-privacy · contextra-prompt-guard                               │
├───────────────────────────────────────────────────────────────────────────┤
│ Lösch- und Nachweis-Orchestrierung (oberhalb des Kerns, NZ-14)             │
│   contextra-erasure (neu: Saga, Bericht, Signatur, Risiko, Retention)      │
│   contextra-erase-verify (neu: Restdaten-Messung, Oracle-Implementierung)  │
├───────────────────────────────────────────────────────────────────────────┤
│ Kompatibilität (temporär, bis G-CUTOVER)                                   │
│   contextra-store (Altstand, unverändert) · contextra-storage-compat       │
├───────────────────────────────────────────────────────────────────────────┤
│ Speicherkern                                                               │
│   contextra-storage (Kern) · contextra-storage-ports (normative Ports)     │
│   contextra-storage-fs (RealFs/FaultFs-Adapter) · contextra-crypto-ref     │
├───────────────────────────────────────────────────────────────────────────┤
│ Schlüssel-Provider (Adapter, austauschbar)                                 │
│   contextra-keys-file · contextra-keys-cohort · contextra-keys-kms (opt.)  │
│   contextra-crypto (Legacy-Passphrase, nur Migrationspfad)                 │
├───────────────────────────────────────────────────────────────────────────┤
│ Grundlagen                                                                 │
│   contextra-ids · contextra-types · contextra-env · contextra-sys          │
│   contextra-simd · contextra-wire · contextra-testkit                      │
└───────────────────────────────────────────────────────────────────────────┘
```

**Abhängigkeitsregeln (werden per `layering`-Test erzwungen):**

- R-A1 Der Kern (`contextra-storage`) hängt nur von `contextra-storage-ports`, `contextra-ids`, `contextra-env` und bewilligten Krypto-Ports ab. Er kennt keine Domänen-Crates.
- R-A2 Kein Crate oberhalb des Kerns greift auf Kern-Interna zu; nur über `contextra-storage-ports` oder die Fassade `contextra-storage-api` (neu, siehe §4.1).
- R-A3 `contextra-erasure` hängt von `contextra-storage-ports` ab, nie vom Kern-Crate selbst.
- R-A4 `contextra-store` darf nur von `contextra-storage-compat` und `contextra-db` (während der Migration) abhängen; danach nirgends.
- R-A5 Keine Schlüssel-Implementierung im Kern. Der Kern kennt nur `KeyProvider` (Port).
- R-A6 `contextra-ports` (Domänen-Ports) re-exportiert keine Kern-Typen mehr; die bisherigen Storage-nahen Ports dort werden deprecated und durch Verweise auf `contextra-storage-ports` ersetzt.

---

## 4. Crate-Disposition

Legende: **BEHALTEN** (unverändert) · **UMBAU** (Umstrukturierung) · **ERSETZEN** (neue Implementierung, alte entfernen) · **ENTFERNEN** · **NEU**.

### 4.1 Speicher-Kern

| Crate | Entscheidung | Begründung und Maßnahmen |
|---|---|---|
| `contextra-storage` (Kern) | **UMBAU** | In Workspace aufnehmen (Pfad bleibt `crates/contextra-storage/crates/contextra-storage` bis M0-Umzug; danach flach `crates/contextra-storage` mit SPEC-Pfaden). Fehlende Engine-Teile (OP1–OP10, Lese-/Schreib-Fassade) gemäß SPEC R9.1. |
| `contextra-storage-ports` | **UMBAU** | Wird die **einzige normative Port-Quelle** des Speicherbereichs. Clock-Vertrag fail-closed; `KeyProvider` vollständig nach §9.4 (Batch, `probe(c, epoch)`, `list → KeyListEntry`, Tomb-API, `KeysInBackups{No,Bounded,Yes}`, `KeyAssurance`). |
| `contextra-storage-api` | **NEU** | Stabile Fassade für Verbraucher: `Storage`, `Change`, `Receipt`, `View`, `Cursor/Page`, `EraseRequest/Ticket/Receipt`, `ErrorClass` mit `hint()`. Verbraucher importieren ausschließlich von hier. Der Kern bleibt intern. |
| `contextra-storage-fs` | **UMBAU** | `RealFs` (`fsync`-Semantik, Verzeichnis-Sync, `rename`, Vorallokation, Locks) implementieren; **nicht** über `contextra-durable-fs` gehen, weil dessen pfadbasierte API `append_prealloc`, `sync_dir` und Range-Reads nicht trägt (siehe §4.6). |
| `contextra-crypto-ref` | **BEHALTEN** | Referenz für Konformanz. Nur `dev-dependency`. Nach Entfernen von `derive_subkey_fast` gemäß E-68 prüfen. |
| `contextra-keys-file` | **UMBAU** | Referenz-Provider ab P6, auf den neuen `KeyProvider` ziehen. |
| `contextra-keys-cohort` | **UMBAU** | Provider-interne Cohort-Strategie (K-04). Eigenes Crate bleibt, Verträge wie `keys-file`. |
| `contextra-keys-kms` | **NEU** (optional, Open-Core-Adapter) | Adapter für Cloud-/HSM-KMS mit `Destroy`-API und `Disabled`-Karenz. Ohne ihn ist `KeyAssurance::Kms` nicht erreichbar. |
| `contextra-storage-bridge` | **UMBAU** | Platzhalter. Zweck klären: entweder Brücke zu `contextra-store` (dann in `storage-compat` aufgehen) oder Replikationsvorbereitung (NZ-20, dann später). **Entscheidung: in `contextra-storage-compat` aufgehen.** |
| `contextra-storage-compat` | **NEU** | Implementiert den bestehenden `contextra_ports::StorageEngine` über den **alten** `contextra-store` (Phase M1) und später über `contextra-storage-api` (ab M4). Erlaubt Migration Verbraucher für Verbraucher, hinter Feature-Flag `storage-kernel`. |

### 4.2 Erasure, Krypto, Schlüssel

| Crate | Entscheidung | Begründung und Maßnahmen |
|---|---|---|
| `contextra-erasure` | **ERSETZEN (Umbau zum Orchestrator)** | Behalten: Saga-Zustandsmaschine (Stufen auf SPEC §9.2 umstellen), Bericht-Erzeugung, Ed25519-Signatur (`verify_strict` überall), Risikoauswertung, Retention-Entscheidung, Aufträge nach Frist, Restriction-Workflow. Entfernen: `erased_set.rs` (Key-Range-Gate, durch Kern-Gate ersetzt), `crypto_local.rs` (BLAKE3-Stream, E-21), `ledger/persistent.rs` und `ledger/durable.rs` (Kern-Ledger), `journal/*` (durch Redo + Ledger ersetzt), `vault/key_store.rs` und `vault/keyring.rs` (durch `KeyProvider`-Adapter), doppelte Typen (`EraseReceipt`, `ErasureStatus`, `ErasureReport`, `InMemoryLedger`, `BoxFuture`), `GLOBAL_GATE_SET`, `erasure_legacy`-Re-Exports. Abhängigkeiten auf `contextra-storage-api` (nicht auf Kern). `kv_shredding.rs` (AES-GCM-SIV) entfällt: Krypto-Erase läuft über Clusterschlüssel. |
| `contextra-erase-verify` | **NEU** | Aus `contextra-erasure/src/measure*`: Restdaten-Messung als eigenständiges Crate. Implementiert das Löschorakel (SPEC §13.2) über `contextra-storage-ports`-Lesezugriff auf die Dateiliste und `verify_absent`. Liefert `MeasureKind`-Berichte (E10-35). |
| `contextra-erasure-spi` | **ENTFERNEN** (existiert nicht) | Referenzen in `contextra-ports/erasure_legacy.rs` und `contextra-text` entfernen. |
| `contextra-crypto` | **UMBAU (Legacy)** | Bleibt für: WAL-HMAC-Migration (Lesepfad), KV-Cipher-Migration und `contextra-privacy`-Surrogat-Schlüssel. **Nicht mehr** für neue Speicherdaten. Vorgaben: `emergency_wipe` mit Zustandsautomat (Wipe ⇒ alle Operationen `Err(Wiped)`, Test auf Folgeverschlüsselung, E10-20); `KeyManager::try_new` für neue Daten gesperrt (`Argon2id` Pflicht, E10-22); `encrypt_auto_nonce` durch 96-Bit-Zufallsnonce oder Zähler mit eindeutigem Instanz-Präfix ersetzen (E10-23); `seal_record_with_nonce` nur `pub(crate)`. |
| `contextra-privacy` | **BEHALTEN + Anbindung** | Surrogat-Vault wird ein Personendatenbestand: Einträge müssen einer Person/einem Cluster zugeordnet werden (§5.4). Surrogat-Länge auf 128 Bit erhöhen (E10-40). Fuzz für Parser ergänzen. |
| `contextra-vault` (excluded) | **UMBAU** | RAM-Staging wird ein `Volatile`-Space des Kerns (SPEC §4.5). Crate entfällt nach Migration; bis dahin nur von `contextra-db` genutzt. |
| `contextra-prompt-guard` | **BEHALTEN** | Keine Änderung. |

### 4.3 Datenschicht

| Crate | Entscheidung | Begründung und Maßnahmen |
|---|---|---|
| `contextra-store` | **BEHALTEN bis G-CUTOVER, dann ENTFERNEN** | Unverändert bis zur Ablösung (SPEC §14). Keine neuen Features. Einziger Zugang für neue Konsumenten: `contextra-storage-compat`. Purge-Pfad (E10-52) bis zur Ablösung in den Vfs-Seam überführen, damit Löschung nicht mehr am Port vorbei läuft. |
| `contextra-mvcc` | **UMBAU zu Sichtlogik** | Stamp, Snapshot-Pin und `View` gehören in den Kern. `contextra-mvcc` wird reine Hilfsschicht für den Altstand (`TxBuffer`, `SequenceLog` als Kompatibilitätsschicht) und danach entfernt. `SnapshotTooOld` statt stillem Reap (E10-62). |
| `contextra-db` | **UMBAU (Hauptmigration)** | Öffentliche API von Store-Typen befreien: `DurabilityMode` → `Durability` (Ports), `TenantScopedStorage` in eigene Typ-Ebene, `SystemPressure` in Ports. Erase-Ergebnisse (`Erased`, `Restricted`, `UnknownCluster`, `ErasePending`) in `Error` aufnehmen (heute nicht behandelt). |
| `contextra-engine` | **BEHALTEN** | Nur Re-Export von `contextra-db`; nach Umbau von db ohne Änderung. |
| `contextra-vector` | **UMBAU** | Eigene `SequenceLog`-Instanz entfernen (E10-61). Sidecars (`pending.wal`, `tombstone.wal`) in Kern-Segmente (`SegmentOwner`, Klasse `Rebuildable`) überführen. Tombstones als Gate-Filter (SPEC §12.4.1). Keine Indexdatei ohne Lineage. |
| `contextra-text` | **UMBAU** | Nutzt nur `StorageEngine` (Compat) und `compat/purge.rs` (deprecated). Purge auf Segment-Owner umstellen, `derive_doc_id` mit keyed MAC (K-08/INV-PRIV-1). |
| `contextra-checkpoint` | **UMBAU** | Hardlink-Klone (E10-54) entfallen; Checkpoints als Kern-Checkpoint (SPEC §10.3). `orphan.rs` über Fs-Port (E10-68). `hardlink_cloner.rs` entfernen. `store.rs`-Pfade auf `contextra-storage-api`. Orphan-Registry als JSON-Datei in den Kern-Katalog verlagern. |

### 4.4 Wissens- und Gedächtnisschicht

| Crate | Entscheidung | Begründung und Maßnahmen |
|---|---|---|
| `contextra-graph` | **UMBAU** | Entity-, Kanten- und Community-Persistenz als Segment-Owner (Klasse `Rebuildable` für CSR/Communities, `ClusterSections` für Entities/Kanten mit Cluster = Person/Fall). Doc-Purge als durabler Gate-Vorgang, nicht RAM-Tombstone (E10-55). Startup-Vollscan durch Consumer-Feed mit Watermark ersetzen (E10-64). Kanten mit mehreren Quellen: `DerivedFrom`-Lineage statt `source_doc_id`. |
| `contextra-cognition` | **UMBAU** | Konsolidierungs-Synthesen als `DerivedFrom`-Cluster mit Link (§9.6). Cross-Subject-Synthesen verboten, ohne Link-Admission (§12.8.21). `text` doppelt entfernen. Trust-Label (§12.6) umsetzen. |
| `contextra-rank` | **BEHALTEN + Gate-Stelle** | Fusion bleibt reine Berechnung. Pflicht: Eingangslisten sind bereits gefiltert (`filter_visible`), dokumentiert im Crate. Keine Persistenz. |
| `contextra-ingest` | **BEHALTEN + Klassifikation** | Keine Persistenz. Für personenbezogene Header (E10-59): Klassifikation `Personal` als Ausgabe-Eigenschaft des Parsers setzen; Standard `keep_personal_metadata = false` für E-Mail. |
| `contextra-agent` | **UMBAU** | Checkpoints gehören zum **Cluster der Person**, nicht zu einem eigenen Cluster pro Sitzung (Konvention „Session ≠ Cluster“, §12.6). Gedächtnis in Checkpoints als Klasse `Personal` verschlüsselt. DLQ-Dedup über Idempotenz-Records (§6.5), nicht Vollscan. `drop_checkpoint` in die Retention-Logik einhängen. Volatiler Scratch → `Volatile`-Space. |
| `contextra-router` | **UMBAU (klein)** | Kalibrierung als Konfiguration über `contextra-storage-api` oder explizite Datei mit Fs-Port. Schreibpfad ergänzen, wenn Kalibrierung persistent sein soll. |
| `contextra-kvcache` | **UMBAU** | Rolle: **Cache**, nicht Speicher. RAM-Stufe bleibt. Tier-2 nur als Segment-Owner (Klasse `ClusterSections` bzw. `MixedPerClusterKey`), niemals `NoPersonalData`. Verschlüsselung Pflicht bei `DataClass::Personal` (Default `cipher: None` verbieten). Löschung über echtes `delete`, nicht leerer Wert (E10-56). Tenant- und Cluster-Scope; Prefix-Wiederverwendung nur innerhalb eines Clusters. Modell-Fingerprint im Schlüssel. |
| `contextra-adapt` | **BEHALTEN** | Keine Persistenz gefunden (nur per grep geprüft; verifizieren). |
| `contextra-sandbox` | **BEHALTEN** | Keine Persistenz. Eingabe/Ausgabe-Blobs bleiben flüchtig; Ergebnis als Record im Cluster des Auftraggebers, falls persistiert. |

### 4.5 Fassaden und Schnittstellen

| Crate | Entscheidung | Begründung und Maßnahmen |
|---|---|---|
| `contextra` (Fassade) | **UMBAU** | `AgentMemory::forget` umbenennen oder mit Pflichtparameter `EraseLevel` versehen; ohne Beleg nicht mehr `Ok`. Fassade liefert `ErasureReceipt` (oder Ticket bei asynchroner Löschung). |
| `contextra-mcp` | **UMBAU (wichtig für Nutzer)** | `contextra_delete` → nur `Logical` mit ausdrücklicher Angabe im Ergebnis (`status: CompleteWithResidual{LogicalOnly}`). Neues Tool `contextra_erase` mit `level`, `request_id`, `deadline` und Receipt-Ausgabe. `contextra_forget` als deprecated Alias, Verhalten identisch zu `contextra_erase` mit Level `Logical`, Feld `deprecated: true`. Keine ID-, Schlüssel- oder Querytext-Ausgabe in Logs (siehe SPEC §16). |
| `contextra-py` | **UMBAU** | Bindings für `erase`, `receipt`, `verify_receipt`. Keine Store-Typen in Python-API. |
| `contextra-wire` | **BEHALTEN** | Protokoll-Typen; Receipt-Serialisierung nach kanonischen Bytes (SPEC §9.7) ergänzen. |

### 4.6 Grundlagen (Ports, Env, Fs, IDs)

| Crate | Entscheidung | Begründung und Maßnahmen |
|---|---|---|
| `contextra-ids` | **BEHALTEN + Klarstellung** | `TenantId` bleibt Domänen-ID der oberen Schicht. `SpaceId` und `ClusterId` leben im Kern-Port. Mapping `TenantId → ClusterId` nur über `IdentityDirectoryOwner` (SPEC §12.7), nie durch Gleichsetzung. Dokumentieren, dass beide Null-Konventionen getrennt sind (E10-77). |
| `contextra-types` | **UMBAU** | `tombstone.rs` mit `contextra-storage-ports::TombId` abgleichen; eines entfernen. `error.rs` gegen Kern-`ErrorClass` vereinheitlichen. |
| `contextra-env` | **UMBAU** | `Clock::now_unix_nanos()` fallibel machen (`Result`); Pre-Epoch darf nicht `0` liefern (E10-74). Konsumenten wie Cache-Zeitstempel bleiben unberührt, Löschfristen müssen den Storage-Clock nutzen. Rng bleibt; Entropie-Port im Kern separat. |
| `contextra-ports` | **UMBAU (Entflechtung)** | `vfs`, `keys`, `erasure`, `ledger`, `gate` (Storage-nahe Teile): als `#[deprecated]` markieren und auf `contextra-storage-ports` verweisen. Domänen-Ports (LLM, ANN, Embedding u. a.) bleiben. |
| `contextra-durable-fs` | **ENTFERNEN (nach M1)** | Eigene `Vfs`-API mit `RealFs = StdVfs`, Namensbelegung `RealFs` (E10-76). Storage erhält eigenen `RealFs` in `contextra-storage-fs`. Verbraucher außerhalb des Kerns: prüfen; falls keine, entfernen. |
| `contextra-sys` | **BEHALTEN** | Plattformnahe Funktionen; `HostProbe`-Adapter (SPEC §3.2) hier implementieren. |
| `contextra-simd` | **BEHALTEN** | Keine Storage-Abhängigkeit nötig. |
| `contextra-testkit` | **UMBAU** | `FaultVfs` und `ManualClock` durch `contextra-storage-fs::FaultFs` und Sim-Clock ersetzen, damit es nur eine Fehlerinjektion gibt. |
| `contextra-bench` | **UMBAU** | Benchmark-Gruppen Storage / Agent / Retrieval nach SPEC §17 anlegen (E10-81). |

---

## 5. Integrationsverträge zwischen den Crates

### 5.1 Einzige Port-Quelle und Versionierung

- `contextra-storage-ports` ist normativ. Ein Trait ändert sich nur mit Eintrag im Changelog und Konformanztest für alle Implementierungen.
- Ports sind **ohne** Erweiterung oberhalb des Kerns zu implementieren; Adapter-Crates (keys-*, fs) dürfen keine Zusatzsemantik einführen.

### 5.2 Fassade `contextra-storage-api`

| Operation | Signatur (Skizze) | Semantik |
|---|---|---|
| `apply` | `apply(&self, change: Change) -> Result<Receipt, StorageError>` | atomar über Spaces und Branches; Receipt `(instance_id, stamp, durable)`; `Indeterminate` bei Sync-Fehler |
| `view` | `view(&self, at: ViewAt) -> Result<View, StorageError>` | `ViewAt::Durable \| At(Stamp)`; Lease-gebunden (`SnapshotTooOld`) |
| `page` | `View::page(&self, q, budget) -> Result<Page, StorageError>` | Token-Budget, `ResumeToken` versiegelt |
| `filter_visible` | `View::filter_visible(&self, refs: &[RecordRef]) -> Vec<RecordRef>` | Pflicht für jede Extension-Ausgabe (INV-VIS-1) |
| `erase` | `Erasure::request(&self, req: EraseRequest) -> Result<Ticket, EraseError>` | `EraseRequest { request_id, level, scope, deadline }`; kein Erfolg ohne Gate |
| `receipt` | `Erasure::receipt(&self, ticket) -> Result<ErasureReceipt, _>` | kanonische Bytes; `verify_receipt` offline |
| `consume` | `Feed::poll(&self, consumer, max) -> Result<Vec<Change>, Lagged>` | `Redacted`/`Erased`/`Restricted`-Platzhalter, nie Klartext nach Gate |

Verbraucher erhalten keine `Vec`-Rückgaben für Vollscans; jeder Scan läuft über `Cursor`/`Page`.

### 5.3 Löschvertrag über Crates hinweg

Eine Löschung ist ein Ticket mit definierten Stufen (SPEC §9.2). Ablauf:

1. Verbraucher (z. B. `contextra-mcp`) ruft `contextra-erasure::request(...)`.
2. `contextra-erasure` prüft Restriktionen über den Kern (`restrictions()`), setzt den Ticket-Zustand `Requested`, schreibt `EraseIntent` ins Ledger (SPEC §9.5.2).
3. Der Kern führt Gate durch (ER1). Erst danach liefert `View` für diesen Cluster `Erased`.
4. `contextra-erasure` veranlasst über `KeyProvider` die Schlüsselvernichtung (ER2) und wartet auf `Absent`.
5. Purge läuft im Kern-Wartungsbetrieb. Owner-Extensions (`SegmentOwner`) bestätigen per `verify_absent`.
6. `contextra-erase-verify` führt die Pflichtmessungen aus (`MeasureKind`).
7. `contextra-erasure` erzeugt und signiert den Bericht; der Kern liefert die kanonischen Receipt-Bytes.

**Verbot:** Kein Verbraucher löscht über `col.delete`, Dateioperationen oder Purge-Funktionen des Altstands mit Erfolgsrückgabe, sobald der Kern aktiv ist (Lint `no-direct-delete` im Crate-Review; Testfall: MCP-Löschung liefert ohne Receipt keinen Erfolg).

### 5.4 Personenbezug und Cluster-Zuordnung

Jeder Verbraucher, der personenbezogene Daten schreibt, muss:

- `DataClass::Personal` setzen (Kern erzwingt Cluster und Sealed),
- den Cluster über `IdentityDirectoryOwner` (SPEC §12.7) zuordnen,
- Abgeleitete Daten (Synthesen, Embeddings, Caches, Graphkanten) mit `DerivedFrom` verknüpfen oder als Segment-Owner registrieren.

Ohne diese drei Schritte ist das Gesamtsystem nicht löschfähig und darf nicht als „löschbar“ ausgewiesen werden. Das ist eine Freigabebedingung je Verbraucher (siehe §7).

### 5.5 Kryptografie-Grenzen

- Kern und Erasure verwenden ausschließlich `Cipher`, `Mac`, `Kdf` aus `contextra-storage-ports`. Implementierung: `contextra-crypto-ref` (Referenz) oder ein zertifizierter Adapter.
- `contextra-crypto` (Legacy) darf nur im Migrationspfad aufgerufen werden.
- Der `KeyProvider` ist der einzige Ort, an dem Schlüsselmaterial außerhalb des Kerns liegt.
- `contextra-privacy` nutzt für Surrogate einen **eigenen**, im Cluster der betroffenen Person verankerten Schlüssel, damit Surrogat-Löschung denselben Mechanismus nutzt.

### 5.6 Zeit

- Löschfristen, Hold-TTL und Idempotenz-Ablauf nutzen ausschließlich den Storage-`Clock`-Port (fail-closed).
- Verbraucher, die nur Zeitstempel für Anzeige brauchen, dürfen `contextra-env::Clock` nutzen (nach E10-74 fallibel).

### 5.7 Fehler

- Jeder öffentliche Fehler trägt `ErrorClass` (SPEC §3.5) und `hint()`.
- `contextra-types::error` und `contextra-ports::error` werden auf die Kern-Klassen gemappt. Verbraucher matchen nur auf `ErrorClass`, nie auf Textinhalte.

---

## 6. Migrationsplan

Dauern werden nicht angegeben; der Fortschritt wird ausschließlich über Gates gemessen.

| Phase | Inhalt | Gate (Abnahme) |
|---|---|---|
| **M0 Stabilisieren** | Storage-Crates in den Workspace; fehlende Workspace-Deps und `publish`; `thiserror` vereinheitlichen; `cargo check --locked` und `cargo test` unter 1.89.0 in CI. Pfade in SPEC, Docs und PHASE-STATUS korrigieren. Sofortmaßnahmen aus §8 umsetzen. `check_invariant_refs.sh` grün machen oder die fehlenden 11 IDs explizit als offen ausweisen (kein `assert!(true)`). | CI grün; alle 52 IDs entweder getestet oder als offen gelistet; PHASE-STATUS-Behauptungen nur mit reproduzierbarer CI-Ausgabe. |
| **M1 Ports und Adapter** | `contextra-storage-ports` als normative Quelle; `contextra-storage-fs::RealFs` (fsync, Verzeichnis-Sync, Lock); Clock/Entropy-Mapping; `contextra-ports` deprecated; `contextra-storage-compat` implementiert `StorageEngine` über `contextra-store`. | Konformanz-Suite für `StorageEngine`-Compat grün gegen Altstand; `RealFs` besteht Fs-Konformanz (Rename, Dir-Sync, Prealloc, Locks) unter Linux. |
| **M2 Kern-Engine** | Recovery OP1–OP10, Schreib-/Lese-Fassade, Redo-Kopf nach §5.5 korrigieren, `seal_frame` im Redo-Pfad. Erste Ende-zu-Ende-Crash-Sweeps über alle Schreibgrenzen. | Crash-Sweep grün für Commit, Flush, Recovery; O-Files-Orakel findet nach `Done` keine Daten. |
| **M3 Erasure-Konsolidierung** | `contextra-erasure` auf Orchestrator reduzieren, Duplikate und `crypto_local` entfernen, `contextra-erase-verify` anlegen, Bericht vor Ledger/Completed persistieren (E10-27), globaler Zustand entfernen (E10-26), Signaturen einheitlich `verify_strict`. | Kein Orphan mehr (Abhängigkeitsgraph); Receipt-Kanonisierung getestet; Testfall „keine Löschung ohne Receipt“. |
| **M4 Konsumenten** | Reihenfolge: `contextra-db` (hinter Feature `storage-kernel`), `contextra-text`, `contextra-vector`, `contextra-checkpoint`, `contextra-graph`, `contextra-kvcache`, `contextra-agent`, `contextra-cognition`. Fassaden (`contextra`, `mcp`, `py`) zuletzt. | Pro Crate: Konformanz gegen Altstand-Tests; Löschung liefert Receipt; Feed/Watermark statt Vollscan. |
| **M5 Datenmigration** | Export-/Import-Werkzeug (logisch, nicht In-Place): Altbestand über `contextra-store` lesen, Cluster-Zuordnung über Identitätsverzeichnis, Schlüsselwechsel auf `KeyProvider`, `tenant_codec`-Präfixe übersetzen, Legacy-Passphrase-Schlüssel einmalig migrieren, Ergebnis gegen Quelle verifizieren. Keine Dual-Writes (SPEC §14). | Round-Trip-Test auf Produktionsnahem Datensatz; Restore-Matrix (SPEC §10.3); Löschungen im Altbestand vor Migration ausgeführt oder explizit übernommen. |
| **M6 Cutover und Abbau** | G-CUTOVER gemäß SPEC §14. Danach `contextra-store`, `contextra-mvcc`, `contextra-durable-fs`, `contextra-vault` entfernen (nach einem Release Schutzfrist). | Alle produktiven Verbraucher migriert; Rollback-Rehearsal bestanden; externes Fencing für Restore belegt. |

Abhängigkeiten: M1 blockiert M2 (Adapter für Tests); M2 blockiert M4; M3 kann parallel zu M2 laufen, muss aber vor M4 für Löschung abgeschlossen sein; M5 nur nach M4 und G-F.

---

## 7. Freigabebedingungen je Verbraucher

Kein Verbraucher gilt als „migriert“, bevor alle Punkte erfüllt sind:

1. Keine direkte Abhängigkeit auf `contextra-store`, `contextra-mvcc` oder Kern-Interna.
2. Schreibpfade laufen über `Change`/`Receipt`; Vollscans über `Cursor`.
3. Personenbezogene Daten sind als `DataClass::Personal` deklariert und einem Cluster zugeordnet.
4. Abgeleitete Daten sind per `DerivedFrom` verknüpft oder als Segment-Owner registriert.
5. Löschung liefert einen Receipt; `Restricted`, `UnknownCluster`, `ErasePending` und `Lagged` sind im Fehlerpfad behandelt.
6. Nachweis per Test: Löschung entfernt Lesbarkeit in der Fassade **und** im Konsumenten (Canary-Test).
7. Für Index-Konsumenten: `IndexLag` und Generation-Check geprüft.

---

## 8. Sofortmaßnahmen (vor jeder Migration)

Priorität und Reihenfolge:

1. **`AdminCap`-Konstruktor schließen** (E10-14): `from_authorized_context` nur noch mit einem nicht klonbaren Autorisierungs-Token aus dem Embedding; Test, dass ein Aufruf ohne Token nicht kompiliert (`trybuild`).
2. **`emergency_wipe` Fail-closed** (E10-20): nach Wipe jede Operation `Err(Wiped)`; Regressionstest, der nach Wipe einen Chiffrier-Versuch prüft und scheitern lässt. Eigene Ticket-ID im Tracker.
3. **Nonce-Pfad in `contextra-crypto` entschärfen** (E10-21, E10-23): `seal_record_with_nonce` nicht öffentlich; `crypto_local` aus dem Produktpfad entfernen. Bis dahin Warnung im Crate-Level-Doc und Verbot in Review.
4. **Passphrasen-Ableitung** (E10-22): neue Speicherdaten nur über `Argon2id` oder `KeyProvider`. Bestehende Produktivinstanzen: Migrationsplan, bis dahin dokumentierter Restrisiko-Eintrag.
5. **Workspace-Mitgliedschaft und `cargo check --locked`** (E10-01, E10-03).
6. **Löschung ohne Beleg unterbinden oder kennzeichnen** (E10-70): MCP-Antwort um `receipt: null` und `status: NoReceipt` erweitern, bis Erasure-Receipt verfügbar ist. Keine Nutzung von „deleted: true“ als Löschnachweis in Dokumentation.
7. **Clock-Fallibilität** (E10-74): Pre-Epoch-`0` in Löschfristen-Pfaden ausschließen (Assert + Test).
8. **Globaler Gate-Zustand entfernen** (E10-26) vor weiterer Nutzung von `contextra-erasure`.
9. **Bericht vor Ledger persistieren** (E10-27).

Jede Maßnahme erhält einen Test im Crate, der das Fehlverhalten reproduziert, bevor der Fix gemergt wird.

---

## 9. Änderungen an SPEC R9.1 (Errata, verbindlich)

| ID | Änderung |
|---|---|
| R10-E1 | §3.1: Pfad korrigieren auf den tatsächlichen Stand (nach M0 flach, bis dahin verschachtelt). |
| R10-E2 | §0 und Kopfzeile: PHASE-STATUS-Pfad korrigieren (`docs/PHASE-STATUS.md`). |
| R10-E3 | §3.2: `Clock`-Port fallibel; Pre-Epoch-Verhalten ausdrücklich ausschließen. |
| R10-E4 | §4.5/§5: `SpaceOptions` um `IdentityDirectory`-Bezug ergänzen, sofern `DataClass::Personal` — keine Textänderung am Format, nur Verbindlichkeit. |
| R10-E5 | §9.4: `KeyProvider`-Trait um `KeyListEntry` und Batch-Methoden gemäß Code-Angleichung (WP-01.5) ergänzen, Signaturen in Doku nachziehen. |
| R10-E6 | §10.1: OP-Schritte als Pflicht-Testmatrix; Stand-Markierung „nicht verdrahtet“ bis Gate. |
| R10-E7 | §13.1: Fuzz-Ziele nur gelten, wenn `fuzz_target!` vorhanden; Platzhalter zählen nicht. |
| R10-E8 | §13.3: `check_invariant_refs.sh` darf nicht über ein Platzhalter-Testfile erfüllt werden. |
| R10-E9 | §14: Gate-Nachweise (Testanzahl, Crash-Sweeps) werden aus CI-Artefakten generiert, nicht aus TRACE-Texten. |
| R10-E10 | §19: `SD-03` Abhängigkeitsliste um `contextra-storage-api`, `contextra-erase-verify` ergänzen, falls neue Crates. |

Diese Errata sind **Doku-Korrekturen und Verschärfungen**, keine Format-Änderungen. Sie lösen keinen Formatfreeze aus.

---

## 10. Offene Entscheidungen

| # | Frage | Vorschlag | Entscheider |
|---|---|---|---|
| O-1 | Bleibt `contextra-erasure` als Name oder wird es `contextra-erase`? | Name behalten (weniger Churn), Rolle klar als Orchestrator dokumentieren. | Architekturrat |
| O-2 | Wer besitzt `contextra-crypto` nach M6? | Nur noch Legacy-Lesepfad; Entfernung nach Datenmigration. | Krypto-Review |
| O-3 | Commercial vs. Open-Core-Grenze für `keys-kms`, Anchor `Strong` | Gemäß SPEC §2.6: Adapter kommerziell möglich, Ports offen, Basis-Löschung nie hinter Paywall. | Produkt |
| O-4 | `contextra-kvcache` Tier-2 überhaupt persistieren? | Vorschlag: nein in v1; nur RAM-Cache, Persistenz erst nach Segment-Owner-Nachweis. | Produkt/Architektur |
| O-5 | Vector-Index: clusterlokal oder tenantweit? | Vorschlag: clusterlokale Mikroindizes für personenbezogene Daten, tenantweit nur `Rebuildable` mit Gate-Filter (SPEC §12.4.1). | Architektur/Datenschutz |
| O-6 | `contextra-privacy`-Surrogat: welche Lebensdauer und welcher Schlüssel? | Eigener Clusterschlüssel je Person; Surrogat-Vault als Personen-Cluster. | Datenschutz |
| O-7 | `DataClass` für Gesamtsystem-Defaults | Kein stiller Default an externen Grenzen (SPEC §4.7); Fehlende Angabe wird abgewiesen. | Architektur |

---

## 11. Verifikationsstatus und Grenzen dieser Analyse

- **Nicht ausgeführt:** `cargo check`, `cargo test`, `clippy`, Fuzzing, Modellprüfung. Die Aussagen zu Workspace-Mitgliedschaft und fehlenden Dependencies stammen aus Manifesten und müssen per Cargo bestätigt werden.
- **Aus Code abgeleitet, zu verifizieren per Test:** E10-20 (Fail-open nach Wipe), E10-21 (Nonce-Determinismus), E10-14 (AdminCap), E10-27 (verworfener Bericht), E10-55 (nicht durabler Graph-Purge), E10-56 (leerer Wert als Löschung).
- **Nicht untersucht:** `contextra-sys`, `contextra-simd`, `contextra-wire`, `contextra-infer-*` (nur Manifest geprüft); `contextra-adapt` nur per grep.
- **Keine Leistungsaussage:** Keine Messung wurde erhoben. Alle Hypothesen bleiben SPEC §17.
- **Rechtlicher Vorbehalt:** Die Aussage „kryptografisch gelöscht“ ist abhängig von Provider-Eigenschaften (`KeyAssurance`) und Backup-Politik. Keine Rechtsaussage zu Art. 17 DSGVO.

---

## Anhang A — Befund-zu-Maßnahme-Matrix (Kurzform)

| Befund | Maßnahme | Meilenstein |
|---|---|---|
| E10-01, -03 | Workspace, Deps, `thiserror` | M0 |
| E10-02, -15 | Pfade, Doku-Drift | M0 |
| E10-04, -05 | Engine-Fassade, Erase-Protokoll | M2, M3 |
| E10-06, -07, -08 | Fuzz, TLA+, Crash-Sweeps | M2 (Fuzz/Sweeps), parallel (TLA+) |
| E10-09, -10 | Invariantenprüfung | M0 |
| E10-11 | RealFs, Sim | M1 |
| E10-12, -13 | Redo-Kopf, Seal im Redo | M2 |
| E10-14 | AdminCap | M0 (Sofort) |
| E10-20 | Wipe fail-closed | M0 (Sofort) |
| E10-21, -23 | Nonce-Pfad | M0 (Sofort) |
| E10-22 | Passphrasen-Ableitung | M0 (Sofort, Plan) |
| E10-24, -25 | Erasure-Orphan, Duplikate | M3 |
| E10-26, -27 | Globaler Zustand, verworfener Bericht | M0 (Sofort), M3 |
| E10-28, -29, -30 | Keys, Stufen, Signatur | M1, M3 |
| E10-50, -51 | Store-API, db-Schnittstelle | M4 |
| E10-52 | Purge am Port vorbei | M3 (Erasure), M4 |
| E10-53 | Keyed Doc-ID | M4 (text/db) |
| E10-54 | Hardlink-Checkpoint | M4 (checkpoint) |
| E10-55, -64 | Graph-Purge, Vollscan | M4 |
| E10-56 | KV-Cache-Löschung und Klartext | M4 |
| E10-57 | Agent-Checkpoint-Klartext | M4 |
| E10-58, -66 | Lineage, Trust-Label | M4 (cognition) |
| E10-60 | Vector-Sidecars | M4 (vector) |
| E10-61, -62, -63 | MVCC, Pins, Observer | M2, M4 |
| E10-70, -71, -72 | Fassade/MCP/Python ohne Beleg | M0 (Sofort), M4 |
| E10-73, -74, -75, -76, -77, -78 | Ports, Clock, Entropie, Fs, IDs, Fehler | M1 |
| E10-79, -81 | Lint/Mutation/Bench-Scope | M0, M4 |

---

## Anhang B — Änderungsprotokoll dieser Revision

- R10.0 (2026-11-24): Erstfassung aus fünf Read-only-Analysen. Enthält Befundregister, Crate-Disposition, Verträge, Migrationsplan, Sofortmaßnahmen, Errata zu R9.1 und offene Entscheidungen.
