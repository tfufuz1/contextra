# Implementierungsspezifikation pro Crate (SOLL-Zustand für Release 1.0.0)

Dieses Dokument dient als normative Umsetzungsanleitung für den Implementierer zur Erreichung des produktionsreifen SOLL-Zustands gemäß `docs/SPEC.md` (v2.0.0).

---

## Übersicht: Workspace-Architektur und Ring-Modell

```
Ring 4: contextra (Fassade)
Ring 3: contextra-erasure, contextra-engine
Ring 2: contextra-mvcc, contextra-store, contextra-text, contextra-vector, contextra-rank
Ring 1: contextra-sys (unsafe), contextra-simd (unsafe), contextra-durable-fs, contextra-crypto
Ring 0: contextra-types, contextra-ports
dev:    contextra-testkit, tests/arch
```

**Allgemeine Regeln (SPEC §2, §5.2):**
- **AR-1:** Abhängigkeiten zeigen **nur auf gleiche oder niedrigere Ringe**.
- **AR-7:** Kein Netzwerk-Crate im Produktions-Abhängigkeitsbaum (HR-1, HR-2).
- **AR-8:** `unsafe` ist ausschließlich in `contextra-sys` und `contextra-simd` erlaubt; alle anderen Crates erzwingen `#![forbid(unsafe_code)]`.
- **P5 / P6:** Keine `cfg(feature)` für Sicherheitsfunktionen. Keine `unwrap()`, `expect()`, `panic!` oder `let _ =` auf `Result` im Produktivcode.

---

## 1. `contextra-types` (Ring 0)
- **Zweck:** Reine Daten- und Typdefinitionen ohne I/O, ohne `async`, ohne Threads, ohne Zeit- oder Zufallsabfragen.
- **Schnittstellen / Zielmodule:**
  - `ids`: `TenantId`, `CollectionName`, `DocKey`, `SubjectId`; blinde Tags `TenantTag`, `CollTag`, `ClusterTag`, `DocTag`, `TermTag` (je `[u8; 16]`). `Seq(NonZeroU64)`, `TxId(u64)`.
  - `key`: `Region` (`Data=0x01`, `Loc=0x02`, `Meta=0x03`), `KeyBuf`, `KeyRange`, `Matcher` (Bereiche fester Länge + exakte Schlüssel).
  - `erasure`: `EraseTarget`, `ErasureStatus`, `ErasureReport`, `RetentionAttestation`, `RetentionCode`, `ResidualRisk`, `ErasureClass`.
  - `secret`: `SecretBytes` (`Zeroizing<Vec<u8>>`).
  - `error`: `ContextraError` Taxonomie (SPEC §7.4).
- **Zu entfernende Altlasten:**
  - `AutoExtractionMode` (SPEC §4 D-27).
  - Veraltetes `witness.rs` und alte Graph-/Checkpoint-Typen.
- **Verifikation:** Unit-Tests für `KeyBuf`-Roundtrips, `Matcher`-Normalisierung und Property-Tests gegen Panics.

---

## 2. `contextra-ports` (Ring 0)
- **Zweck:** Trait-Definitionen für die Entkopplung von Speicher, Indexen, Löschkern und Plattform.
- **Traits (SPEC §8.2):**
  - `Vfs` / `VfsFile`: Asynchrone Dateisystemabstraktion (konsolidiert).
  - `Clock`, `Rng`, `IdGen`.
  - `StorageEngine`: Asynchrone Methoden für transaktionale Key-Value-Speicher.
  - `TextIndex`, `VectorIndex`.
  - **`ErasureLayer`**: `layer_id()`, `plan(&Matcher) -> LayerPlan`, `purge(&PurgePlan, &Deadline) -> PurgeOutcome`, `scan_residue(&NeedleSet) -> LayerScan`, `positive_control(&Canary) -> bool`.
  - `ReadGate`, `RecordSealer`, `LedgerSink`.
- **Zu entfernende Altlasten:**
  - Deklarationen für `graph`, `graph_index`, `checkpoint`, `scanner`, `license`.
- **Verifikation:** Trait-Contract-Testsuiten gegen Mock- und Real-Implementierungen.

---

## 3. `contextra-sys` (Ring 1, `unsafe`)
- **Zweck:** Kapselung aller Low-Level-Betriebssystemprimitive (`fsync`, `flush_dir`, `rename_durable`, `mlock`).
- **Anforderungen:**
  - Jede `unsafe`-Operation benötigt zwingend einen `// SAFETY:`-Kommentar.
  - Keine sicheren Wrapper mit implizit unsicheren Vorbedingungen (`mmap_readonly`, `reopen_and_dup2` müssen als `pub unsafe fn` deklariert werden).
- **Verifikation:** `cargo test -p contextra-sys`, Miri-Prüfung der Speicherzugriffe.

---

## 4. `contextra-simd` (Ring 1, `unsafe`)
- **Zweck:** Vektordistanzberechnung (L2, Cosine, Dot) via SIMD (AVX2, AVX-512, NEON) mit Laufzeit-Feature-Erkennung.
- **Anforderungen:**
  - **Entfernung des fehlerhaften u8-VNNI-Kernel-Pfads** (`_mm512_dpbusd_epi32`, SPEC D-28).
  - Fokus auf f32-Berechnungen.
  - Jeder SIMD-Kernel muss gegen eine Skalar-Referenzimplementierung auf relative Abweichung ε ≤ 1e-4 getestet sein (`T-SIMD-1`).
- **Verifikation:** Property-Tests mit 10⁵ Zufallsvektoren (`cargo test -p contextra-simd`).

---

## 5. `contextra-durable-fs` (Ring 1)
- **Zweck:** Robuste `RealVfs`-Implementierung von `contextra-ports::Vfs`.
- **Anforderungen:**
  - Atomares Schreiben: Temp-Datei schreiben → `sync_all` → Rename → Verzeichnis-`fsync` (SPEC FI-F01).
  - Keine unbeachteten `sync`-Fehler (kein `let _ = file.sync()`).
  - Strikte Dateiberechtigungen (Verzeichnisse 0700, Dateien 0600).
- **Verifikation:** Fault-Injection-Tests via `FaultVfs` aus `testkit`.

---

## 6. `contextra-crypto` (Ring 1)
- **Zweck:** Kryptografische Primitive (AEAD, KDF, HMAC, Hashketten, Ed25519).
- **Anforderungen:**
  - **Argon2id Mindestparameter:** `MIN_M_COST_KIB >= 65536` (64 MiB), `t >= 3`, `p = 1`.
  - AES-256-GCM-SIV mit Nonce- und AAD-Bindung an den vollständigen Schlüssel.
  - Hash-Ketten für WAL und Löschjournal.
  - Bereinigung von toten Moduldeklarationen (`deletion_proof`, `kv_shredding`, `revocation_log` in `lib.rs` entfernen).
  - Entfernen ungenutzter Abhängigkeiten (`lru`, `regex`, `bincode`).
- **Verifikation:** Known-Answer-Tests gegen offizielle Testvektoren, `T-KDF-1`, `T-ENC-1/2`.

---

## 7. `contextra-mvcc` (Ring 2)
- **Zweck:** Snapshot-Isolation, SSI-Validierung und Sequenzverwaltung.
- **Anforderungen:**
  - **Beseitigung des Sentinel-Bugs (SPEC B-11, D-C02):**
    - `prune_through(bound_seq)`: Wenn `bound_seq` das `TOMBSTONE_BIT` trägt, Fehler zurückgeben statt stillschweigend zu maskieren.
    - Snapshot-Sequenzen als `Option<Seq>` modellieren; niemals `u64::MAX` als echte Sequenz in `pruned_through` übernehmen.
  - Snapshot-Widerruf (`revoke_covering(&Matcher)`), wenn ein Löschauftrag offene Snapshots invalidiert.
- **Verifikation:** Loom-Tests für Nebenläufigkeit, Property-Tests gegen `ReferenceModel`.

---

## 8. `contextra-store` (Ring 2)
- **Zweck:** Transaktionaler blinder LSM-Tree (WAL, Memtable, SSTable, Manifest, Compaction) mit Purge-Primitiven.
- **Anforderungen:**
  - **Löschcluster-Schlüsselraum (SPEC §9.1):** Umstellung von Textpräfixen auf blinde Tags (`Region ‖ tenant_tag ‖ coll_tag ‖ cluster_tag ‖ doc_tag ‖ family ‖ sub`).
  - **Pflichtverschlüsselung:** Entfernen von `Option<KeyManager>`, `encryption_passphrase: None` und Klartext-Fallbacks.
  - **WAL-Integrität:** Unbedingtes Replay mit HMAC-SHA256-Prüfung, Entfernen von `wal-integrity`-Feature-Gates und `#![allow(unexpected_cfgs)]`.
  - **Vfs-Integration:** Ersetzen aller direkten `tokio::fs` / `std::fs`-Aufrufe durch `contextra_ports::Vfs`.
  - **Implementierung von `ErasureLayer`:**
    - `plan(&Matcher)`: Lokalisieren betroffener SSTables.
    - `drain()`: Memtables leeren, versiegelte WALs entfernen und Fehlen prüfen.
    - `rewrite_file()`: Streamendes Umschreiben unter Beibehaltung der Originalsequenzen nicht betroffener Daten (I-2), atomarer Tausch via Manifest.
    - `scan_residue()`: Decoder-Scan über alle Blöcke.
- **Verifikation:** Differential-Tests gegen `ReferenceModel` (`T-DIFF-1`), Crash- und Abrupt-Kill-Tests (`T-CRASH-1..4`).

---

## 9. `contextra-text` (Ring 2)
- **Zweck:** BM25-Volltextindex als abgeleiteter, segmentierter und verschlüsselter Index (SPEC §8.11, §9.7).
- **Anforderungen:**
  - Segmente als verschlüsselte 32-MiB-Dateien. Keine Postings im LSM-Tree.
  - Terme als blinde `term_tag`s.
  - Exakte Neuberechnung der globalen BM25-Statistiken nach jedem Segment-Rewrite.
  - Implementierung von `ErasureLayer` (Segmente neu schreiben ohne gelöschte Dokumente).
- **Verifikation:** `T-TEXT-1` (Parität zu BM25-Referenz), `T-TEXT-3` (Erase-Segment-Rewrite).

---

## 10. `contextra-vector` (Ring 2)
- **Zweck:** HNSW-Vektorsuche mit verschlüsselter Persistenz (SPEC §8.10, §9.8).
- **Anforderungen:**
  - Sofortiges Slot-Zeroing im RAM beim Erase.
  - Verschlüsselte Snapshot-Chunks über `Vfs`.
  - Batch-Rebuild des Graphen bei Tombstone-Quote >= 0,2.
  - Implementierung von `ErasureLayer`.
- **Verifikation:** `T-VEC-1` (Recall@10 >= 0,95 gegen Brute-Force), `T-VEC-2` (Slot-Zeroing und Snapshot-Rewrite).

---

## 11. `contextra-rank` (Ring 2)
- **Zweck:** Reciprocal Rank Fusion (RRF) für Hybridsuche (SPEC §8.13).
- **Anforderungen:**
  - Reduktion auf deterministische `rrf_fuse(lists: &[Vec<Hit>], k: u32, top: usize)`.
  - Vollständige Entfernung von Kalibrierungs- und Bandit-Routing-Abhängigkeiten.
- **Verifikation:** `T-RANK-1` (Deterministische RRF-Fusion).

---

## 12. `contextra-erasure` (Ring 3, Neukomponente)
- **Zweck:** Entkoppelter Löschkern und Drei-Säulen-Orchestrierung (SPEC §8.9, §11).
- **Module:**
  - `erased_set`: In-Memory- und persistierte Sperre (`ReadGate`).
  - `vault`: Schlüsseltresor (`vault.dat`) zur Erzeugung und Zerstörung von Dokument-DEK-Salts.
  - `saga`: Transaktionale Zustandsmaschine (`Requested` -> `Gated` -> `KeyDestroyed` -> `Drained` -> `Rewritten` -> `Unlinked` -> `IndexesCleaned` -> `Measured` -> `Completed`).
  - `measure`: Durchführung der Prüfungen E1 (Rohscan), E2 (Decoder-Scan), E3 (Schlüsselabwesenheit).
  - `report`: Erstellung und Ed25519-Signierung des `ErasureReport`.
  - `ledger`: Append-Only-Log mit `LedgerSink` für Restore-Guard (SPEC §11.8).
- **Verifikation:** `T-VAULT-1..4`, `T-PROOF-1`, `T-SLA-1`, Mutationstests auf `classify`.

---

## 13. `contextra-engine` (Ring 3)
- **Zweck:** Zusammenspiel und Orchestrierung von Speicher, Indexen und Löschung.
- **Anforderungen:**
  - **Bereinigung von Graph-Altlasten:** Beseitigung aller Referenzen auf `graph_index`, `relate`, `community`.
  - **Delete-Lock-Härtung:** Keyed Lock über die gesamte Operation halten (SPEC B-13).
  - Durchgängiges `ReadGate`-Filtering vor Lese- und Suchzugriffen.
  - Transaktionales Schreiben: `Doc`, `DocMap`, `Manifest`, `LOC` in einer Store-Transaktion.
- **Verifikation:** `T-ISO-1..3`, `T-VER-1`, `T-CONC-1`.

---

## 14. `contextra` (Ring 4, Fassade)
- **Zweck:** Einzige öffentliche API für Anwendungsentwickler (SPEC §7).
- **Anforderungen:**
  - `EngineBuilder` mit Pflichtparametern: `passphrase` (`SecretBytes`), `erase_sla`, `runtime`, `vfs`, `clock`, `rng`.
  - Bereinigung aller Re-Exports entfernter Crates (Lizenz, Privacy, Audit-Export).
  - Export von `erasure_checklist()` (SPEC §13.3).
- **Verifikation:** Trybuild-Tests (`T-API-1`), Integrationsbeispiele (`examples/erase_and_verify.rs`).

---

## 15. `contextra-testkit` & `tests/arch` (dev)
- **Zweck:** Umfassende Testinfrastruktur und Architektur-Linting.
- **Anforderungen:**
  - Bereitstellung von `FaultVfs`, `ReferenceModel`, `MemLedger`, `CanaryFactory`.
  - Architekturtests in `tests/arch`:
    - `T-ARCH-1`: `types` enthält weder I/O noch async.
    - `T-ARCH-2`: Nur `TenantScopedStorage` ruft den rohen Store auf.
    - `T-ARCH-3`: Ring-Abhängigkeiten zeigen nur nach unten.
    - `T-LINT-1`: Wortlaut-Lint verbietet unzulässige Begriffe ("DSGVO-konform", "garantiert gelöscht" etc.).
    - `T-LINT-2`: Kein `std::fs` / `tokio::fs` im Store.
- **Verifikation:** `cargo test -p tests-arch`.
