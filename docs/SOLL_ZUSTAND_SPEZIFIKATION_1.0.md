# Contextra — Endprodukt-Spezifikation (SOLL-Zustand, Single Source of Truth)

**Version:** 1.0 · **Stand:** 14.10.2026 · **IST-Basis:** `tfufuz1/contextra` @ `f00bb2a`
**Rolle:** Principal Senior Rust Architect (finale Entscheidungsinstanz)
**Sprache der Spezifikation:** Deutsch · Code-Bezeichner, Pfade und Invarianten-IDs bleiben unverändert.

> Dieses Dokument ist in sich geschlossen. Es verweist auf kein anderes Planungsdokument. Verweise auf Quellcode-Pfade beziehen sich auf das Repository. Alle Aussagen über den IST-Zustand sind als solche markiert und beziehen sich auf den oben genannten Commit.

---

## 0. Gebrauchsanweisung (für Menschen und Agenten)

### 0.1 Zweck
Dieses Dokument legt fest, **was Contextra sein soll**, **warum**, **was dafür am Code zu ändern ist** und **in welcher Reihenfolge**. Es wird jeder Agentenanfrage beigelegt. Es ersetzt alle früheren Architektur-, Algorithmen-, Löschnachweis- und Produktpläne.

### 0.2 Rangordnung bei Konflikten
1. Harte Governance-Dateien des Repos (`CONSTITUTION.md`, `AGENTS.md`, `VETOES.md`) gelten weiter. Widerspricht diese Spezifikation einer davon, wird zuerst ein ADR (§13) beschlossen, das die Governance-Datei ändert. Kein Agent löst einen solchen Konflikt eigenmächtig.
2. Diese Spezifikation (SOLL).
3. Der Code (IST). Weicht der Code von der Spezifikation ab, ist das entweder ein Bug im Code oder eine fehlende Spezifikationsänderung. Beides wird per PR behoben. Der Code gilt nie stillschweigend als Wahrheit.
4. `capabilities.toml` ist die maschinenlesbare Kopie der Ring-Tabelle (§4.1). Sie wird aus dieser Spezifikation synchronisiert und per Gate geprüft (G-04).

### 0.3 Regeln für Agenten
- **R-A1:** Pro PR genau ein Arbeitspaket (AP, §12). Der AP nennt Dateien, Akzeptanzkriterien und Gates. Außerhalb des AP wird nichts geändert.
- **R-A2:** Ein AP ist erst fertig, wenn seine Akzeptanzkriterien als Test oder Gate laufen. Beschreibung ohne Test zählt nicht.
- **R-A3:** Jeder neue `pub`-Symbol braucht einen Produktionsaufrufer und einen Test über die Fassade (REG-06).
- **R-A4:** Wo diese Spezifikation „Re-Verifikation" verlangt, wird der IST-Zustand zuerst gemessen und das Ergebnis im PR festgehalten. Erst danach wird geändert.
- **R-A5:** Offene Punkte (§14) werden nicht geraten. Sie werden dem Principal Architect als Frage vorgelegt.
- **R-A6:** Rechtliche und normative Aussagen (DSGVO, Beweiskraft, BSI) dürfen nur in der Form aus §5.14 in Doku, Fehlermeldungen oder Marketing erscheinen.

### 0.4 Verifikationsstatus dieser Spezifikation
- Geprüft durch Lesen des Quellcodes: Ring-Zuordnung, Feature-Definitionen, `KvDeleteMode`, `PhysicalScope`, `erase_document`, Builder-Wiring, MCP-Abhängigkeiten, MCP-Mandantenfallback, Mutex-Typ des Group-Commit, Gate-Registry, Unwrap-Ratsche.
- Als gemessen übernommen (bei M0 erneut zu messen, AP-M0.3): `cargo check --locked` grün für `contextra-engine` und `contextra`; Hüllengröße 26; `unsafe`-Verteilung; Zählungen zu `Instant::now`, FMA, `partial_cmp`.
- **Nicht** ausgeführt: Tests, Loom, Miri, Fuzzing, Mutation, Benchmarks, Clippy, NEON-Pfad, `--all-features`.
- Mathematische Aussagen (§9) sind analytisch aus den Formeln abgeleitet und noch nicht durch Experimente belegt.

---

## 1. Produktdefinition und Release-Schnitt

### 1.1 Leitmotiv
> **Jede Fähigkeit ist ein Plugin. Was nicht aktiviert ist, existiert weder im Binary noch zur Laufzeit. Der Löschnachweis ist das Kernversprechen, sein Preis ist opt-in.**

### 1.2 Endprodukte

| ID | Produkt | Inhalt | Release |
|---|---|---|---|
| **EP1** | Contextra Memory | Lokaler Mandanten-Speicher, Hybridsuche (Vektor + BM25 + RRF), MCP-Server, Löschung in den Modi `Off` und `Assured` | **v0.1.0**, danach Gate-basiert 1.0.0 |
| **EP2** | Contextra Proof | Modus `Attested`: Layer-Scans, signierter Löschbeweis v4, unabhängiger Verifier, Audit-Export, AVV-Generator | nach EP1 |
| **EP3** | Contextra Retrieval+ | Wissensgraph, PathRAG, OpenIE-Ingestion, Graph-Löschung | nach EP2 |
| **EP4** | Contextra Inference | Lokale Embedder (candle/onnx/ollama), KV-Cache (KIVI, SnapKV, Prefix-Trie, Tier-2) | nach EP3 |
| **EP5** | Contextra Cognition | Adapt (PID, Bandit), Router, Cognition (Konsolidierung), Agent | experimentell |
| **EP6** | Contextra Guard | Privacy/Egress-Schutz, Sandbox (WASI-Merge), Lizenzprüfung | nach Bedarf |

### 1.3 Versionierung
- Erstes Release: **v0.1.0** (Workspace-Version ist `0.1.0`).
- **1.0.0 ist ein Gate, kein Meilenstein.** Es wird vergeben, wenn alle vier Bedingungen erfüllt sind:
  1. Upgrade-Test mit Golden-Store-Fixtures (altes Manifest, `purge/`-Intents, Ledger) grün.
  2. Crash-Matrix CRASH-01 bis CRASH-08 (§5.12) grün.
  3. Rechtliche Formulierungen nach §5.14 geprüft und freigegeben.
  4. Formatstabilitätsregel (§5.9) beschlossen und durch Tests abgesichert.
- Formatstabilität wird erst ab 1.0.0 zugesagt. Vorher gilt: Upgrade-on-open, nie Downgrade.

### 1.4 Was EP1 verspricht und was nicht
**EP1 verspricht:** schnelle lokale Hybridsuche, strikte Mandantentrennung, deterministisches Verhalten, im Modus `Assured` logische und physische Löschung im Speicher des Stores (Tombstone, SSTable-Rewrite, Überschreiben und Entfernen der Altdateien, Anti-Resurrection-Ledger, Restore-Guard).
**EP1 verspricht nicht:** einen signierten Löschbeweis (das ist EP2), Löschung auf SSD-Controller-Ebene, in Backups, Swap, Core-Dumps oder im Prozessspeicher, rechtsverbindliche Beweiskraft.

---

## 2. Verbindliche Leitregeln

Diese Regeln gelten für jeden Code und jede Entscheidung. Ein Verstoß ist ein Merge-Blocker.

| ID | Regel |
|---|---|
| **REG-01** | **Zero-Panic.** Kein `unwrap`, `expect`, `panic!`, `unreachable!` oder unchecked Indexing im Produktivcode von Ring 0 und Ring 1. Test-, Bench- und Fuzz-Code ist ausgenommen. Die Ratsche `governance/ratchet.toml:unwrap_exact` (IST 502) ist das einzige Gate und darf nur sinken. |
| **REG-02** | **Ring-DAG.** Abhängigkeiten laufen nur von höheren auf niedrigere Ringe (gleicher Ring nur laut Ordnung §4.2). `capabilities.toml` ist die Kopie der Ring-Tabelle. |
| **REG-03** | **Determinismus über Ports.** Zeit, Zufall und IDs kommen ausschließlich über `Clock`, `Rng`, `IdGenerator` aus `contextra-ports`. Direkte Aufrufe von `Instant::now`, `SystemTime::now`, `thread_rng`, `rand::random` sind außerhalb der Port-Implementierungen und außerhalb von Tests/Benches verboten (Gate G-09). |
| **REG-04** | **Drei Unsafe-Inseln:** `contextra-sys`, `contextra-simd`, `contextra-wire`. Kein `unsafe` anderswo. Jedes `unsafe`-Fragment hat einen `// SAFETY:`-Kommentar. Für generierten Code in `wire` gilt eine explizite, dokumentierte Ausnahme (kein Kommentar je Block, dafür ein Datei-Header und ein Generator-Test). Alle übrigen Crates tragen `#![forbid(unsafe_code)]`. |
| **REG-05** | **Single-Node.** Keine Multi-Node-, Sharding-, Raft- oder Cluster-Logik. „Leader" bedeutet ausschließlich den Group-Commit-Leader im Store. Es gibt keine Replikas. |
| **REG-06** | **Wirksamkeit.** Kein `pub`-Symbol ohne Produktionsaufrufer und ohne End-to-End-Test über die Fassade. Entweder verdrahten, auf `pub(crate)` reduzieren oder löschen. |
| **REG-07** | **Opt-in-Kosten.** Nicht aktivierte Features verursachen weder Code im Binary noch CPU-Kosten. Jedes Feature kompiliert isoliert (Gate G-06). |
| **REG-08** | **Fail-closed.** Fehler, Unbekanntes und Unvollständiges führen zu Ablehnung, nie zu stillem Erfolg. Das gilt für Scans, Resume, Lizenz, Mandantenbindung, Speicherplatzprobe, Manifest-Prüfung. |
| **REG-09** | **Builder-Ehrlichkeit.** Jeder Builder-Schalter hat eine nachweisbare Wirkung. `build()` validiert die Verdrahtung gegen den tatsächlichen Zustand, nie gegen hartcodierte Werte. |
| **REG-10** | **Snapshot-Konsistenz.** Alle Signale einer Anfrage lesen denselben Sequenzstand über ein `ReadSnapshot`-Token. |
| **REG-11** | **Sperrenordnung.** Die globale Lock-Reihenfolge aus §7.1 gilt crate-übergreifend. Synchrone Locks werden nie über `.await` gehalten. |
| **REG-12** | **Tool-Klassifikation.** Jedes MCP-Tool ist klassifiziert und feature-gated. Ein Tool ohne Klassifikation wird nicht registriert. |
| **REG-13** | **Erasure-Opt-in und Ports-Only.** Kern-Crates importieren nie Erasure-Plugin- oder Evidence-Crates. Sie sprechen nur Ports aus `contextra-ports`. |
| **REG-14** | **Fehlerfreie Evidenz.** Ein Zeuge oder Beweis entsteht nur aus einem real ausgeführten Scan. Es gibt keine Konstanten-, Default- oder `None`-Zeugen. Fehler ergeben keinen Beweis. |
| **REG-15** | **Tenant-Matrix.** Jeder Löschtest läuft über `LsmStorage` **und** über `TenantScopedStorage`. |
| **REG-16** | **Ehrliche Claims.** Dokumentation, Fehlermeldungen und API-Namen behaupten nie mehr als technisch bewiesen ist (§5.14). |
| **REG-17** | **Formate nie downgraden.** Jedes persistente Format ist versioniert. Öffnen upgradet, ein Downgrade wird abgelehnt. |

---

## 3. IST-Zustand und Befundregister

### 3.1 Strukturdaten (IST @ `f00bb2a`)
- 34 Crates in `crates/`, dazu 7 Fuzz-Crates und 3 Tooling-Mitglieder (44 Workspace-Mitglieder), rund 380k Zeilen Rust.
- Hülle des EP1-Binaries (`contextra-mcp`, Default-Features): 26 Crates. Mit `--no-default-features` ebenfalls 26.
- `contextra-core` ist eine tote Fassade: 139 Dateien in `src/`, 404 Dateien im Workspace referenzieren `contextra_core`, 11 `Cargo.toml` hängen davon ab. Viele Symbole kommen per `pub use contextra_ports::*` hindurch.
- Fassade `contextra`: `default = ["fast", "candle"]`. Engine: `default = ["graph","rank","adapt","kvcache","candle"]`.
- Das Löschpfad-Gerüst existiert, aber ohne Implementierungen: `contextra-ports/src/erasure.rs` (223 Zeilen) hat **keine** `impl` für `ErasureBackend`, `LayerScan`, `WriteGate`, `RestoreGuard`, `ErasureHooks`. `Attested` ist nur eine Enum-Variante.
- Store enthält bereits: `erasure_ledger/`, `lsm/purge/` (`intent.rs`, `resume.rs`, `orchestrator.rs`), `lsm/fence.rs`, `lsm/witness.rs`, `ManifestFormatTracker` mit `FormatError::DowngradeAttempt`.
- Ring laut `capabilities.toml` (maßgeblich für IST): Ring 0: `types, ports, wire, sys, durable-fs, simd, mvcc, text, rank, vector, graph, crypto, adapt, audit-export, avv-generator, core`. Ring 1: `store, checkpoint, kvcache`. Ring 2: `infer-candle, infer-ollama, infer-onnx, sandbox`. Ring 3: `engine, db, privacy, cognition, router, agent`. Ring 4: `contextra, mcp, py, license`. Tooling: `testkit, xtask-heavy` und Bench/Fuzz.

### 3.2 Befundregister

Schweregrad: 🔴 Blocker (verletzt das Kernversprechen oder Sicherheitsinvarianten), 🟠 hoch, 🟡 mittel. „Re-V." = Befund stammt aus einer älteren Messung und muss bei M0 am HEAD neu verifiziert werden (AP-M0.3), bevor er behoben wird.

| ID | Sev | Befund (IST) | Soll-Behebung | AP |
|---|---|---|---|---|
| **F-01** | 🔴 | `erase_document` (`contextra-engine/src/collection/crud/delete.rs`) ruft **kein** `ledger_append` auf (nur Kommentare), stellt aber einen `ErasureLedger`-Zeugen mit `entry_confirmed: true` und `decision_chain_mac: [0u8; 32]` aus. Text- und Relations-Zeugen werden mit `None` gebaut, Fehler per `if let Ok` verschluckt. | Zeugen-Fabrikation ersatzlos entfernen. Ohne echten Scan kein Zeuge (REG-14). | M0.2 |
| **F-02** | 🔴 | Keine API-Semantik für Teilerfolg: Nach Ledger-Eintrag setzt Recovery den Tombstone durch, auch wenn der Aufrufer zuvor ein `Err` sah (SSI-Konflikt, I/O, Cancellation). Im Code kann `index.delete(..).await?` nach erfolgreichem `db_tx.commit()` noch `Err` liefern. | Commit-Punkt-Semantik und `ErasureIndeterminate` (§5.3). Cancellation-sicherer Löschpfad. | M2.6 |
| **F-03** | 🔴 | `PhysicalScope` (`ports/src/erasure.rs`) hat `pub`-Felder, `pub fn new`, `Default`, `Deserialize`. Leerer Prefix matcht jeden Key. `LogicalScope::Keys/Prefixes` tragen keine `TenantId`. `matches_key` ist linear je Key. | Opaker Typ mit Invarianten (§5.4). | M1.1 |
| **F-04** | 🔴 | Port-Typen erfüllen die Zielinvarianten nicht: `LayerObservation` ohne Digest (nur `details: String`); `WriteGate::check_allowed(key)` je Key; `HookToken` ist `Copy` und öffentlich; `purged(scans)` akzeptiert leere Liste; `FreeSpaceProbe` fehlt im Port; Zeitstempel sind Wall-Clock; `layer_id` ist `u8`. | Port-Neudefinition (§5.5). Es gibt keine Implementierungen, die Änderung ist billig. | M1.2 |
| **F-05** | 🔴 | Tenant-Pfad umgeht Fence und Zeugen: `delete.rs` castet per `downcast_ref::<LsmStorage>()`; unter `TenantScopedStorage` ergibt das `None`. `tenant_id()` liefert per `unwrap_or_default()` still einen Default-Mandanten, `kv_hooks.remove_doc_segments` kann den falschen Mandanten treffen. Insgesamt 5 Downcasts. | `ErasureBackend` über `as_erasure_backend()`, fail-closed Mandantenauflösung (§5.11). | M2.3 |
| **F-06** | 🔴 | Purge-Korrektheit (Re-V.): obsolete SSTables bleiben unscrubbt (L4), Lesefehler werden verschluckt (L5), Rang/Seq-Grenzen gehen beim Rewrite verloren (L6), `None`-Probe wird als 0 Byte frei gelesen (L8), Purge trifft logische statt physische Schlüssel (N-K), eine Intent-Datei je Store kollidiert (N-E), `Vec`+Doppellesen (N-F), Orphan-GC ohne Scrub (N-G), Failpoints per Umgebungsvariable (N-J). | Purge v2 (§5.7). | M2.1 |
| **F-07** | 🔴 | Abgeleitete Daten liegen außerhalb des Löschumfangs (§5.10): HNSW-mmap-Knoten (im Code als „explicit residual risk" markiert, `core_rebuild.rs`), `quantize`, `rabitq`, `diskann`, KV-Cache-Tier-2, Prefix-Trie, Graph-Merge, inkrementelle SVD, Conformal-Kalibrierset, Bandit-Zustand, LLM-Triples, aktives WAL-Segment (Klartext bis Rotation), Snapshot-Pins (ein langer Reader blockiert Zeroisierung unbegrenzt). | Inventar mit Strategie je Speicher (§5.10), Pin-Policy, WAL-Rotation. | M5.7–M5.9 |
| **F-08** | 🟠 | Beweisformat inkonsistent: 7 Layer (`u8`, `[LayerProof;7]`) gegen LayerIds 1..7/20/21/26/40; Public Key liegt im Beweis (Selbstzertifizierung); `proof_v3.rs` enthält `construct_v4_payload` mit bincode unter dem v4-Domain-Tag (zwei Serialisierungen, ein Tag); `verify` statt `verify_strict`; `bincode` im Signaturpfad. | Beweis v4 normativ (§6), v3-Erzeugung löschen. | M1.4, M7 |
| **F-09** | 🟠 | Einzel-Dokument-Löschung läuft über Rewrite und Zeroing, den auf SSD/CoW schwächsten Mechanismus; Crypto-Erase nur auf Collection/Tenant. Schlüssel-Backup mit Wartefrist würde Shredding aushebeln. | Zwei-Stufen-Crypto-Erase, kein DEK-Backup, ehrliche Claims (§5.13, §5.14). | M5.6 |
| **F-10** | 🟠 | Keine Hochwassermarke (`erasure_high_water_mark` 0 Treffer). Manifest-Integrität (MAC) unspezifiziert. Verhalten bei Lizenzablauf unklar. | §5.9, §5.15. | M5.1 |
| **F-11** | 🔴 | Fence-TOCTOU: `ErasureFence` (`parking_lot::RwLock`) wird nur am Eingang von `put`/`delete` geprüft, vor dem Group-Commit. `commit_mutex` ist `tokio::sync::Mutex` und wird vor dem WAL-I/O freigegeben und für den Memtable-Publish neu erworben. Zwischen Check und Publish kann eine Fence aktiv werden. | Fence-Protokoll B mit Re-Check im Publish-Fenster (§5.8). | M4.1 |
| **F-12** | 🟠 | Lock-Hierarchie unvollständig dokumentiert; Regel „keine Locks über `.await`" ist falsch formuliert (`erase_document` hält tokio-`kv_locks` über mehrere Awaits inkl. Commit). | §7.1. | M4.3 |
| **F-13** | 🔴 | MCP: `validation.rs` liest `CONTEXTRA_TENANT_ID` und fällt auf `TenantId::SYSTEM` zurück (fail-open). `server_dispatch.rs` nutzt `block_in_place`+`block_on`. `contextra-ports` mit `test-utils` steht in `[dependencies]`. Harte Abhängigkeiten auf `contextra[router, encryption-at-rest]`, `adapt`, `crypto`, `privacy`. Features `fast/guard/graph` fehlen. | §8. | M3.6 |
| **F-14** | 🟠 | Hülle 26 statt ≤ 17: `candle` im Default, Engine-Defaults ziehen `graph/adapt/kvcache/candle`, MCP bindet Parkplatz-Crates hart. | §4.4, §4.5. | M3.2–M3.4 |
| **F-15** | 🟠 | `verify_wiring` existiert (`wiring_selftest.rs`), ist aber tautologisch: `builder.rs` übergibt `has_license_gate = true` hartcodiert und verwirft das Ergebnis (`let _report`). Sieben Builder-Schalter (`with_privacy`, `with_adapt`, `with_kms`, `with_kv_shredding`, `with_checkpoint`, `with_avv_generator`, `with_audit_export`) haben keine Wirkung. | §4.7. | M3.5 |
| **F-16** | 🟠 | `KvDeleteMode::default()` ist `CryptoShred` (`store/src/kv/delete_mode.rs`), Test `test_kv_delete_mode_default` zementiert das. | Default `TombstoneOnly`; Builder setzt `CryptoShred` bei `encryption-at-rest`. | M1.5 |
| **F-17** | 🟠 | `contextra-core` tote Fassade (s. 3.1). | Ersatzlos entfernen per Skript-PRs (§4.9). | M3.1 |
| **F-18** | 🟠 | Vier parallele Ringmodelle (Tabellen in Plänen, `capabilities.toml`, Layer 0–10 in `SOURCE_OF_TRUTH.md`). 15 von 21 vergleichbaren Crates widersprechen `capabilities.toml`. Plan nennt Unsafe-Inseln `vector/kvcache/store`, die kein `unsafe` haben. | Ein Modell (§4.1–§4.2). | M0.4, M4.5 |
| **F-19** | 🟡 | Alle 46 `unsafe` in `wire` liegen in `contextra_generated.rs` ohne `// SAFETY`. Ein Plan wollte `unsafe` in `kvcache` (`aligned_alloc.rs`). | REG-04; `kvcache` bleibt `forbid(unsafe_code)`, ausgerichtete Allokation als sichere Fassade in `sys`. | M4.6 |
| **F-20** | 🟠 | Determinismus nicht durchgesetzt: `Instant::now` ~81, `SystemTime::now` 7, `thread_rng` 6 Treffer (grob, inkl. Inline-Tests); kein Workspace-`clippy.toml`; Workspace-Lints ohne `indexing_slicing`, `arithmetic_side_effects`, `float_cmp`; `partial_cmp` 38× in `src/`; 25 FMA-Stellen in `simd` (u. a. NEON `vfmaq_f32`); `ReadSnapshot` plante `Instant`. | §7.4–§7.6, Gate G-09. | M4.7 |
| **F-21** | 🟠 | 512 `pub fn` ohne Aufrufer, 1.057 undokumentierte Schnittstellen. Vier Reaper (`start_expiry_reaper`, `start_orphan_reaper`, `start_thermostat_reaper`, `start_kv_cache_eviction`) werden im `open`-Pfad nie gestartet. | §10. | M3.7, M6.4 |
| **F-22** | 🟠 | Einzelbefunde: `apply_temporal_validity_filter_at` castet Unix-Zeit in `TxId`; `PendingDeleteJournal::process` verwirft `let _ = self.flush()`; `kv_shredding.rs:247` offenes TODO (lockfreie Gruppen-Revokation); `AgentContext.status` ist `pub`; `engine`/`store` binden `sandbox` (Ring-Verletzung). | §10.2. | M3.7 |
| **F-23** | 🟡 | Governance: `deletion-guarantee-gate` fehlt in `xtask/registry.toml`; doppelte/abweichende Dokumentvarianten; absolute `file:///`-Links; `contextra-bench` und Fuzz-Crates fehlen in `capabilities.toml`. | M0.4, §11. | M0.4 |
| **F-24** | 🟠 | `docid-128` ändert das On-Disk-Format und verträgt sich nicht mit „jedes Feature isoliert". | Format-Feature mit Manifest-Eintrag (§4.5). | M4.4 |
| **F-25** | 🟡 | 1-%-p99-Gate auf CI-Hardware ist statistisch nicht messbar. | Instruktionszähler plus Toleranz (§11.3). | M4.2 |
| **F-26** | 🟠 | Mathematische Fehler und Lücken in Algorithmenbeschreibungen (§9). | §9 normativ. | M8–M10 |
| **F-27** | 🟠 | Ledger: IST-Format weicht vom Zielformat ab (`ErasureLedger`/`ledger_append` im Store vorhanden, Zielformat nicht). Unbegrenztes Wachstum, Verhalten bei fehlendem MAC-Schlüssel und Kompaktierung ungeklärt. | §5.9. | M5.2 |
| **F-28** | 🟠 | Krypto-Aussagen widersprüchlich (AES-GCM-SIV gegen AES-256-GCM; „deterministisches CSPRNG-Schlüsselmaterial"; „BSI-TR-02102-konform"; „rechtsverbindlich nach § 371a ZPO"). | §5.13, §5.14. | M0.5 |
| **F-29** | 🟡 | Annahmen über Kompilierfehler (Engine/Fassade), ein stets `CapabilityUnsupported` liefernden `Collection::delete()` und ein noch fehlendes `erasure.rs` treffen am HEAD nicht zu: `delete()` übergibt `assurance: S0`, `erasure.rs` existiert (223 Zeilen), `cargo check --locked` ist grün. | Messen statt annehmen. | M0.3 |
| **F-30** | 🟠 | `contextra-crypto` (7.669 Zeilen) bündelt Schlüsselverwaltung, KV-Verschlüsselung, Löschbeweis, Audit-Kette. Store, Vector, Engine, Kvcache u. a. hängen daran; Beweiscode landet im EP1-Binary. Fuzz-Targets (`deletion_proof_tamper` u. a.) liegen in `crypto`. | Crate-Split (§4.8). | M5.5 |
| **F-31** | 🟠 | Ring-Verletzungen im IST (aus `Cargo.toml` gelesen; reguläre und Dev-Kanten in M0.3 trennen): `store`(1)→`sandbox`(2); `graph`(0)→`store`(1) und `store`→`graph` (Zyklus-Verdacht über Dev-Kante); `engine`(3)→`license`(4); `engine`(3)→`sandbox`(2) ist erlaubt, aber ungeplant hart; `db`(3)→`cognition`(3) gegen die Ordnung; `checkpoint`→`store` im selben Ring. | Auflösung per Ports und Ordnung (§4.3). | M4.5 |

---

## 4. Zielarchitektur

### 4.1 Ring-Modell (SOLL, einziges gültiges Modell)

Es gibt genau ein Ringsystem (Ring 0 bis 4 plus Tooling). Andere Schichtnummerierungen (z. B. „Layer 0–10") heißen „Topo-Ebene" und sind nur eine Sortierhilfe der Dokumentation.

| Crate | Ring | Unsafe-Insel | Produkt | Rolle |
|---|---|---|---|---|
| `contextra-types` | 0 | nein | EP1 | Basistypen, IDs, `ContextraError`, `DeterministicTimestamp` |
| `contextra-ports` | 0 | nein | EP1 | Alle Port-Traits (Clock, Rng, IdGenerator, StorageEngine, Erasure-Ports, ReadSnapshot, MergeOperatorFactory, LicenseGate, …) |
| `contextra-wire` | 0 | **ja** (generierter Code) | EP1 | Wire-Format (FlatBuffers-Ableitung) |
| `contextra-sys` | 0 | **ja** | EP1 | OS-Schnittstellen, `SystemClock`, `SystemRng`, sichere ausgerichtete Allokation, mmap-Hilfen |
| `contextra-durable-fs` | 0 | nein | EP1 | fsync-Disziplin, `scrub_and_remove`, `sync_dir`, Journal |
| `contextra-simd` | 0 | **ja** | EP1 | SIMD-Kernel (AVX2, NEON, Scalar) |
| `contextra-crypto-core` | 0 | nein | EP1 | KeyManager, KDF, AEAD, HMAC, SecureBuf, WAL-Crypto (**neu durch Split**) |
| `contextra-mvcc` | 0 | nein | EP1 | Snapshots, SSI-Validator |
| `contextra-text` | 0 | nein | EP1 | BM25, invertierter Index |
| `contextra-rank` | 0 | nein | EP1 | RRF-Fusion, Kalibrator (Conformal), DiBud |
| `contextra-vector` | 0 | nein | EP1 | HNSW und Quantisierung; Delegation an `simd` |
| `contextra-adapt` | 0 | nein | EP5 | PID, Bandit |
| `contextra-graph` | 0 | nein | EP3 | CSR-Graph, PPR, PathRAG (rein algorithmisch, **kein** Store-Import) |
| `contextra-crypto-kv` | 0 | nein | EP1 (Feature) | KV-Segmentverschlüsselung, Revocation-Log (**neu**, Feature `kv-encryption`) |
| `contextra-proof-verify` | 0 | nein | EP2 | Unabhängiger Verifier, nur `ed25519-dalek` und `blake3` (**neu**) |
| `contextra-audit-export`, `contextra-avv-generator` | 0 | nein | EP2 | Export und AVV-Generator |
| `contextra-store` | 1 | nein | EP1 | LSM, WAL, Manifest, Ledger, Purge, Fence |
| `contextra-checkpoint` | 1 | nein | EP1 | Checkpoints (nutzt `Clock`/`IdGenerator`) |
| `contextra-kvcache` | 1 | nein (`forbid(unsafe_code)`) | EP4 | KV-Cache |
| `contextra-infer-candle`, `-ollama`, `-onnx` | 2 | nein | EP4 | Inferenz-Backends |
| `contextra-sandbox` | 2 | nein | EP6 | WASI-Merge-Operatoren; implementiert `MergeOperatorFactory` |
| `contextra-privacy` | 3 | nein | EP6 | DLP, Egress-Filter |
| `contextra-engine` | 3 | nein | EP1 | Collections, Hybridsuche, Löschpfad |
| `contextra-db` | 3 | nein | EP1 | Mehrmandanten-API über Engine |
| `contextra-erasure-evidence` | 3 | nein | EP2 | Beweisausstellung, Policy, Zeugen (**neu durch Split**) |
| `contextra-cognition`, `-router`, `-agent` | 3 | nein | EP5 | Kognition, Routing, Agent |
| `contextra` | 4 | nein | EP1 | Fassade und Composition Root |
| `contextra-mcp` | 4 | nein | EP1 | MCP-Server (stdio) |
| `contextra-py` | 4 | nein | EP1/EP5 | Python-Bindings |
| `contextra-license` | 4 | nein | EP6 | Lizenzprüfung (optional, nicht in EP1) |
| `contextra-testkit`, Bench, Fuzz, `xtask*` | Tooling | nein | — | In `capabilities.toml` eingetragen |

Entfernt: `contextra-core` (§4.9), `contextra-crypto` (§4.8). Es gibt kein Crate `contextra-errors`.

### 4.2 Ordnung innerhalb eines Rings
```
Ring 0:  types < ports < wire < sys < durable-fs < simd < crypto-core < mvcc < text < rank
              < vector < adapt < graph < crypto-kv < proof-verify < audit-export < avv-generator
Ring 1:  store < checkpoint < kvcache
Ring 2:  sandbox, infer-candle, infer-ollama, infer-onnx   (untereinander unabhängig)
Ring 3:  privacy < engine < erasure-evidence < db < cognition < router < agent
Ring 4:  license < contextra < mcp, py
```

### 4.3 Abhängigkeitsmatrix und Auflösung der Verletzungen

| Crate | Darf abhängen von | Darf **nicht** abhängen von |
|---|---|---|
| `store` | `types, ports, sys, durable-fs, crypto-core, mvcc, vector` (+ `graph` nur mit Store-Feature `graph`) | `crypto-kv` (außer Feature `kv-encryption`), `erasure-evidence`, `sandbox`, `license`, `engine` |
| `vector` | `types, ports, sys, durable-fs, simd, crypto-core` | `store`, `erasure-evidence` |
| `graph` | `types, ports, adapt` | `store`, `engine`, `crypto-*` |
| `checkpoint` | `types, ports, store` | `engine`, `crypto-kv` |
| `kvcache` | `types, ports, sys, crypto-core` | `store`, `engine` |
| `engine` | `types, ports, store, vector, text, rank, mvcc, checkpoint` (+ optional `graph`, `adapt`, `kvcache`, Inferenz) | `crypto-*` direkt, `erasure-evidence`, `license`, `sandbox` direkt |
| `db` | `engine, store, ports, types` | `cognition` (regulär) |
| `erasure-evidence` | `types, ports, crypto-core, durable-fs` | `store`, `engine`, `license` |
| `mcp` | `contextra` (Fassade), `types`, `ports` | `crypto*`, `adapt`, `privacy`, `wire`, `rank` direkt |

### 4.4 Hülle und Produkt-Manifest EP1

```toml
# products/ep1.toml
[product]
id = "EP1"
name = "Contextra Memory"
status = "release-0.1"
target_binary = "contextra-mcp"

[crates]
allowed = [
  "contextra-types", "contextra-ports", "contextra-sys", "contextra-wire",
  "contextra-simd", "contextra-durable-fs", "contextra-crypto-core",
  "contextra-mvcc", "contextra-text", "contextra-rank", "contextra-store",
  "contextra-vector", "contextra-checkpoint", "contextra-engine",
  "contextra-db", "contextra", "contextra-mcp",
]   # 17 Crates
forbidden = [
  "contextra-core", "contextra-crypto", "contextra-graph", "contextra-cognition",
  "contextra-adapt", "contextra-router", "contextra-agent", "contextra-privacy",
  "contextra-sandbox", "contextra-license", "contextra-kvcache",
  "contextra-infer-candle", "contextra-infer-ollama", "contextra-infer-onnx",
  "contextra-erasure-evidence", "contextra-crypto-kv", "contextra-proof-verify",
]

[build]
features = ["fast"]

[gate]
command = "cargo tree -p contextra-mcp -e normal --no-default-features --features fast"
max_crates = 18
```

### 4.5 Feature-Matrix (SOLL)

```toml
# contextra/Cargo.toml
[features]
default = ["fast"]
fast = ["contextra-engine/fast"]

graph      = ["contextra-engine/graph"]
adapt      = ["contextra-engine/adapt"]
kvcache    = ["contextra-engine/kvcache"]
cognition  = ["dep:contextra-cognition"]
router     = ["dep:contextra-router"]
agent      = ["dep:contextra-agent"]

candle = ["dep:contextra-infer-candle"]
ollama = ["dep:contextra-infer-ollama"]
onnx   = ["dep:contextra-infer-onnx"]

privacy            = ["dep:contextra-privacy"]
sandbox            = ["dep:contextra-sandbox"]
license            = ["dep:contextra-license"]
dev-open-gate      = ["license", "contextra-license/dev-open-gate"]
encryption-at-rest = ["contextra-store/encryption-at-rest", "dep:contextra-crypto-kv"]

erasure          = ["contextra-store/erasure", "contextra-engine/erasure"]
erasure-evidence = ["erasure", "dep:contextra-erasure-evidence", "license"]

docid-128 = ["contextra-types/docid-128"]

sovereign  = ["privacy", "erasure-evidence", "encryption-at-rest"]
compliance = ["sovereign", "contextra-audit-export", "contextra-avv-generator"]
```

### 4.6 DurabilityMode × ErasureMode

| DurabilityMode ↓ / ErasureMode → | `Off` | `Assured` | `Attested` |
|---|---|---|---|
| `Full` | ✅ | ✅ | ✅ |
| `WalNoHmac` | ✅ | ✅ (Health-Warnung `W-DURABILITY-WAL-NOHMAC`) | ❌ `INV-DURABILITY-RING` |
| `MemoryOnly` | ✅ | ❌ `INV-DURABILITY-RING` | ❌ `INV-DURABILITY-RING` |

### 4.7 Builder-Ehrlichkeit und Wiring
1. `ContextraBuilder::build()` ruft `verify_wiring()` zwingend auf.
2. `verify_wiring` prüft den tatsächlichen Zustand: Jeder gesetzte Schalter muss einen realen, nicht-`None`-Port besitzen.
3. Wirkungslose Schalter aus dem IST werden in M3.5 entweder verdrahtet oder entfernt.
4. Neue Builder-API: `with_erasure_mode(ErasureMode)`, `with_erasure_hooks(...)`, `with_license_gate(...)`.
5. `KvDeleteMode::default()` ist `TombstoneOnly`. Builder setzt `CryptoShred` bei `encryption-at-rest`.

### 4.8 Crate-Split `contextra-crypto`
`contextra-crypto` wird in `contextra-crypto-core` (Ring 0), `contextra-crypto-kv` (Ring 0), `contextra-erasure-evidence` (Ring 3) und `contextra-proof-verify` (Ring 0) zerlegt.

### 4.9 Entfernung von `contextra-core`
`contextra-core` wird ersatzlos entfernt. Symbole wandern per Skript-PRs in `types`, `mvcc`, `wire` und `ports`.

### 4.10 Manifeste der übrigen Produkte (EP2 bis EP6)
EP2 (Proof), EP3 (Retrieval+), EP4 (Inference), EP5 (Cognition), EP6 (Guard) bauen modular auf EP1 auf.

---

## 5. Löscharchitektur (Kernversprechen)

### 5.1 Drei Modi
- **`Off`:** Tombstone + normale Compaction. Zero overhead.
- **`Assured`:** Anti-Resurrection Ledger + Write-Fence + Purge v2 (Streaming Rewrite + Scrub) + RestoreGuard.
- **`Attested` (EP2):** Assured + Layer-Scans + Signierter `DeletionProofV4` (Ed25519) + Verifier.

### 5.2 Garantien je Modus (Vertrag)
Nach `Ok(DeleteOutcome::Assured)` und abgeschlossenem Purge (`PurgeState::Completed`) existiert kein lesbarer Datensatz des Scopes in Memtable, SSTables, Blob-Dateien und Manifest-Schlüsselgrenzen. Alle Altdateien und WAL-Segmente des Scopes wurden gescrubbt und entfernt.

### 5.3 Commit-Punkt und API-Semantik
Commit-Punkt einer Löschung ist das erfolgreiche, durable Anhängen des `ErasureIntent`-Records an das Ledger (fsync).
- Vor dem Commit-Punkt: `EraseError::NotStarted(cause)`.
- Nach dem Commit-Punkt: Löschung wird zwingend abgeschlossen. Bei Spätfehlern erhält der Aufrufer `Err(EraseError::Indeterminate { purge_id, cause })`.

```rust
pub enum DeleteOutcome {
    Logical { tx: TxId },
    Assured { tx: TxId, purge: PurgeHandle },
}

pub enum EraseError {
    NotStarted(ContextraError),
    Indeterminate { purge_id: PurgeId, cause: ContextraError },
}
```

### 5.4 Scopes
```rust
pub enum LogicalScope {
    Document   { tenant: TenantId, collection: CollectionName, doc: DocId },
    Collection { tenant: TenantId, collection: CollectionName },
    Tenant     { tenant: TenantId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalScope { /* private Felder */ }
```

### 5.5 Port-Spezifikation `contextra-ports::erasure`
```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub enum ErasureMode { #[default] Off, Assured, Attested }

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PurgeId(pub [u8; 16]);

#[must_use]
pub struct HookToken { id: u64, _no_copy: () }

pub trait ErasureBackend: Send + Sync {
    fn physical_scope(&self, scope: &LogicalScope) -> Result<PhysicalScope, ContextraError>;
    fn acquire_fence(&self, scope: &PhysicalScope, id: PurgeId) -> Result<FenceGuard, ContextraError>;
    fn purge_physical(&self, scope: &PhysicalScope, as_of: TxId, probe: &dyn FreeSpaceProbe) -> Result<PurgeOutput, PurgeError>;
}

pub trait StorageErasureExt { fn as_erasure_backend(&self) -> Option<&dyn ErasureBackend>; }
```

### 5.6 Löschpfad und Zustandsmaschine
Zustände in `PurgeState`: `LedgerDurable`(1), `Tombstoned`(2), `PurgePending`(3), `Purging`(4), `Purged`(5), `KeyQuarantined`(6), `Completed`(7), `Scanning`(8), `Attested`(9).

### 5.7 Purge v2 (Streaming-Rewrite)
1. Vorprüfung Speicherplatz via `probe.available_bytes()`.
2. Active/Immutable Memtables flushen.
3. SSTable Entry-Strom lesen und direkt Ersatz-SSTables schreiben (O(1) RAM).
4. Records mit `scope.matches_key(key)` und `seq <= as_of` verwerfen.
5. Manifest-Austausch in einer Transaktion.
6. Altdateien via `contextra_durable_fs::scrub_and_remove` verwerfen (Nullen überschreiben, fsync, unlink).

### 5.8 Write-Fence (Protokoll B: Drain-Fence)
`ErasureGate` zählt `inflight`-Batches. `acquire_fence` setzt `active = true`, wartet auf `inflight == 0` (Drain) und blockiert neue Batches via `check_batch`.

### 5.9 Ledger v1, Manifest-Marke, Formatversionierung
Ledger-Segmentdateien `ledger/<seq>.ledger`: Segment-Header (Magic "CTRALEDG", prev_chain_hash) und Records (type, tx, scope_digest, chain_hash, HMAC-SHA-256 MAC). Nach Purged werden Klartext-Scopes durch Digests ersetzt.

### 5.10 Inventar abgeleiteter und redundanter Daten
Alle abgeleiteten Speicher (Vektor-Slots, Text-Postings, Checkpoints, KV-Cache-Blöcke, Cache-Einträge) besitzen konkrete Bereinigungsstrategien. Snapshot-Pins unterliegen `max_snapshot_age` (15 min) und `pin_grace` (30 s).

### 5.11 Mandantenpfad
Fail-closed Mandantenauflösung ohne `downcast_ref::<LsmStorage>()` oder `unwrap_or_default()`.

### 5.12 Crash- und Chaos-Matrix
12 Testfälle CRASH-01 bis CRASH-12 decken alle Absturzzeitpunkte ab.

### 5.13 Crypto-Erase (nur mit `encryption-at-rest`)
Zweistufig: Stufe 1 `KeyRevoked` (Quarantäne 72 h Tenant/Collection, 0 h Dokument), Stufe 2 `KeyDestroyed`.

### 5.14 Zulässige Claims
Ehrliche Aussagen gemäß REG-16: Keine Behauptung von "unwiederbringlich gelöscht" oder "garantiert SSD-sicher".

### 5.15 Lizenz und Löschmodi
Die Lizenz steuert nur das Ausstellen neuer Beweise (`Attested`), blockiert nie `open()` oder Assured-Löschungen.

---

## 6. Evidenz und Löschbeweis (EP2, Modus `Attested`)

- **Kryptosuite:** AES-256-GCM-SIV, Argon2id, HKDF-SHA-256, HMAC-SHA-256, BLAKE3, Ed25519 (`verify_strict`).
- **Beweisformat v4:** TLV (u16 Tag, u32 Length Big-Endian), Critical-Bit `0x8000`, Domain-Tag `contextra/deletion-proof/v4\0`.
- **Layer-Modell:** 14 DeletionLayers (IDs 1–7, 20–26).
- **Verifier `contextra-proof-verify`:** Standalone-Verifier nur mit `ed25519-dalek` und `blake3`.

---

## 7. Concurrency, MVCC, Determinismus

### 7.1 Globale Lock-Reihenfolge (REG-11)
```
Engine-Ebene (async):  collections → kv_locks → embedder
Erasure-Ebene:        ledger_append_lock → fence_install_lock
Store-Ebene:          commit_mutex → pending_commit_queue → truncate_lock → state
```

### 7.2 Snapshot-Isolation (REG-10)
`ReadSnapshot { id: SnapshotId, max_seq: TxId }` als Pflichtparameter aller Lesefunktionen.

### 7.3 MVCC-Bereinigung
Read-Set-Tracking im Lesepfad zur vollständigen SSI-Validierung.

### 7.4 Zeit, Zufall, IDs (REG-03)
Ports `Clock`, `Rng`, `IdGenerator` liefern `DeterministicTimestamp { wall_ms, mono_ctr }`.

### 7.5 Clippy-Durchsetzung (Gate G-09)
`disallowed-methods` für `Instant::now`, `SystemTime::now`, `thread_rng`, `partial_cmp`.

### 7.6 Fließkomma-Politik
Reproduzierbarkeit je ISA-Pfad. Bit-Identität über ISAs nur für Integer-/Festkomma-Pfade (RRF/DiBud Q64.64). HNSW-Aufbau mit kanonischem Kernel. NaN/Inf-Ablehnung am Eingang. Sortierung via `f64::total_cmp`.

---

## 8. Fassade und MCP

- Composition Root in `contextra`.
- Strikte Mandantenbindung beim MCP-Prozessstart (kein `SYSTEM`-Fallback).
- Async Dispatch via `spawn_blocking`.
- MCP-Tools klassifiziert (`ReadOnly`, `WriteLocal`, `EgressExternal`).

---

## 9. Algorithmen (normativ)

- **9.1 Hybridsuche & RRF:** Festkomma `FixedRankScore(u128)` Q64.64, `k=60`.
- **9.2 DiBud:** Budgetierte RRF-Zertifizierung auf materialisierten Listen, strikt `>`.
- **9.3 Conformal Calibration:** Split Conformal `q̂ = ⌈(n+1)(1-α)⌉`-ter Score, ACI-Adaption.
- **9.4 KV-Cache:** KIVI 2-Bit Quantisierung, SnapKV Attention Eviction, Prefix-Trie je Mandant, Tier 2 mmap-zstd.
- **9.5 Graph & OpenIE:** Ingestion-Events mit Cache-Replay.
- **9.6 PathRAG:** Budgeted k-Path Diffusion, `AGT-GRAPH-001` Laufzeitprüfung.
- **9.7 Sketched Bandit:** Sparse JL-Projektion (Achlioptas), Reset bei Drift.
- **9.8 Spektrale SVD:** Brand Rang-1 SVD Update mit Truncation auf `k_max=64`.

---

## 10. Verdrahtungs-Doktrin
Unverdrahtete `pub fn` reduzieren oder verdrahten. Ratsche `find-unwired-gaps` (Gate G-14).

---

## 11. Qualitätssicherung & Gates
20 Gates G-01 bis G-20 in `xtask/registry.toml`.

---

## 12. Roadmap und Arbeitspakete (M0 bis M11)
- **M0:** Governance, Baseline, Hotfix F-01.
- **M1:** Port-Vertragstypen, `PhysicalScope`, `DeleteOutcome`, `PurgeState`.
- **M2:** Purge v2, Streaming-Rewrite, `ErasureBackend`.
- **M3:** Core-Cut (`contextra-core` Entfernung), Hülle ≤ 18 Crates, Builder, MCP.
- **M4:** Write-Fence Drain Protokoll B, Locks, Formatversionierung, Determinismus.
- **M5:** Assured vollständig, Ledger v1, Crypto-Erase zweistufig, Crate-Split `crypto`.
- **M6:** Release-Härtung v0.1.0.
- **M7–M11:** EP2 (Proof), EP3 (Retrieval+), EP4 (Inference), EP5 (Cognition), EP6 (Guard).

---

## 13. Entscheidungsregister (ADR-E01 bis E37)
Final beschlossene ADRs E01 bis E37 bilden das Entscheidungsfundament.

---

## 14. Offene Punkte, Risiken & Ehrlichkeitsklausel
Ehrlicher Umfang: Keinerlei Zusagen bezüglich SSD-Remanenz, Dateisystem-Snapshots, Swap oder Prozessspeicher.

---
*Ende der Spezifikation.*
