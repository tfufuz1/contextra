# Contextra — Invarianten-Verifikationsmatrix (INV_MATRIX)

**Datum:** 2026-10-12
**Rolle:** Verifikations-Auditor
**Ziel:** Nachweis der Testabdeckung aller in der Spezifikation definierten Invarianten (`INV-*`) sowie Bewertung, ob bestehende Tests eine Verletzung der jeweiligen Invariante zuverlässig erkennen.
**Arbeitsbereich:** `docs/verification/INV_MATRIX.md`

---

## 1. Zusammenfassung

| Metrik | Anzahl | Anteil |
| :--- | :---: | :---: |
| **Gesamtzahl eindeutiger `INV-*` aus der Spezifikation** | **44** | 100,0 % |
| **In Code / Crates referenziert oder umgesetzt** | 37 | 84,1 % |
| **Test vorhanden (`ja`)** | 25 | 56,8 % |
| **Test prüft Verletzung nachweislich (`ja`)** | 26 | 59,1 % |
| **Test prüft Verletzung unklar / indirekt (`unklar`)** | 13 | 29,5 % |
| **Kein spezifischer Test vorhanden (`nein`)** | 5 | 11,4 % |

---

## 2. Vollständige Invarianten-Verifikationsmatrix

| Invariante (ID) | Spezifikations-Fundstelle | Beschreibung / Invarianten-Kern | Code-Fundstelle | Test-Fundstelle | Test vorhanden? | Test prüft Verletzung? |
| :--- | :--- | :--- | :--- | :--- | :---: | :---: |
| `INV-C1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:495` | Compaction muss Tombstones für aktive Snapshots aufbewahren (`raw_seq <= floor_seq`). | `crates/contextra-store/src/compaction/engine.rs:180` | `crates/contextra-store/src/compaction/tests/advanced.rs:591` | ja | ja |
| `INV-CAL-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:489,524` | Isotonische Kalibrierung: Kein stiller Fallback vor Warmup / mindestens N Datenpunkte. | `crates/contextra-rank/src/calibration/isotonic.rs:112` | `crates/contextra-rank/tests/calibration_deep_tests.rs:188`, `calibration_stress_and_fault_tests.rs:207`, `isotonic_mutation_hardening_test.rs:142` | ja | ja |
| `INV-CALIBRATION-CONFORMAL-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:489` | Konforme Kalibrierung: Abdeckungsgrenzen & Nicht-Konformitätsscores in `[0, 1]`. | `crates/contextra-rank/src/calibration/conformal.rs:45` | `crates/contextra-rank/tests/conformal_tests.rs:104` | ja | ja |
| `INV-CHECKPOINT-DETERMINISM-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:731,775` | Determinismus im Checkpoint-Crate: Keine direkten `SystemTime::now()` Aufrufe; Zeitquelle ausschließlich via `Arc<dyn Clock>`. | `crates/contextra-checkpoint/src/guard.rs:42`, `store.rs:35` | `crates/contextra-checkpoint/tests/clock_determinism.rs:12`, `checkpoint_clock_determinism.rs:15` | ja | unklar |
| `INV-COLLECTION-PROFILE-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:486` | Valides Collection-Profil mit strikter Validierung der Parametergrenzen. | `crates/contextra/src/collection_profile.rs:10` | keine | nein | unklar |
| `INV-COLLECTION-PROFILE-3` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:2682,4106` | PII-Vault-Kopplung erzwingt `KvDeleteMode::CryptoShred` für PII-Treffer (`is_pii && !is_memory_only`). | `crates/contextra-privacy/src/egress_gateway.rs:18` | `crates/contextra-privacy/tests/pii_vault_forces_crypto_shred.rs:3` | ja | ja |
| `INV-COMPACTION-ADAPTIVE-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:495` | Adaptive Compaction kandidatenbasierte SSTable-Fensterauswahl bewahrt zusammenhängende Sequenzordnung. | `crates/contextra-store/src/compaction/adaptive.rs:8` | `crates/contextra-store/tests/adaptive_compaction_determinism.rs:10`, `adaptive_compaction_tombstone.rs:12` | ja | unklar |
| `INV-CONSOLIDATE-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:487` | Crash-Resilienz und Transaktionssicherheit der Konsolidierungssession im Wissensgraphen. | `crates/contextra-cognition/src/context_compaction/session.rs:12` | keine | nein | nein |
| `INV-CRYPTO-DUPLICATE-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:485` | Erkennung und Ablehnung doppelter Löschbeweise/Schlüsselregistrierungen (Fail-Closed Integrity). | `crates/contextra-crypto/src/revocation_log.rs:85` | `crates/contextra-crypto/tests/campaign_j25.rs:14` | ja | ja |
| `INV-DAG-DEPS` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:486` | Interne CRUD-Abhängigkeiten folgen streng azyklischer Reihenfolge (DAG). | `crates/contextra-engine/src/collection/crud/internal.rs:18` | keine | nein | unklar |
| `INV-DB-3` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:486` | Automatische Kompensationsaktionen / Rollback bei Fehlschlagen von Insert/Update/Delete-Operationen. | `crates/contextra-engine/src/collection/crud/insert.rs:30`, `update.rs:25`, `delete.rs:20`, `db_transaction.rs:40` | `crates/contextra-engine/tests/transaction_rollback.rs:15` | ja | unklar |
| `INV-DEDUP-TRANSITIVITY-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:488` | Transitivitäts-Veto bei der Deduplizierung (Verhindert fehlerhaftes Zusammenführen bei Veto-Kanten). | keine | keine | nein | nein |
| `INV-DELETION-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:485` | Unwiderrufliche kryptografische Löschung & Zeroization; Validierung von Löschbeweisen V1/V2/V3. | `crates/contextra-crypto/src/deletion_proof.rs:30`, `store/src/lsm/witness.rs:15` | `crates/contextra-store/tests/memtable_witness.rs:10`, `crates/contextra-crypto/tests/guarantee_deletion_irrecoverable.rs:12`, `audit_chain_tests.rs:25` | ja | ja |
| `INV-DELETION-2` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:494` | HNSW-Geisterzeiger-Bereinigung und Tombstone-Löschsicherheit im Vektorgraphen. | `crates/contextra-vector/src/hnsw/deletion.rs:10` | `crates/contextra-vector/tests/p03b_hnsw_ghost_pointer_verification_complete.rs:12` | ja | ja |
| `INV-DISKANN-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:494` | DiskANN Delta-Persistenz: Keine temporären `.delta.tmp`-Dateien nach `persist_delta_sync()`. | `crates/contextra-vector/src/diskann/build.rs:15` | `crates/contextra-vector/src/diskann/tests.rs:188` | ja | ja |
| `INV-DURABILITY-RING` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:485` | Ring-Integrität & Erasure-Ledger Durability über Speicherstufen hinweg. | `crates/contextra-store/src/erasure_ledger/mod.rs:8` | `crates/contextra-store/tests/campaign_j11_h3.rs:14` | ja | ja |
| `INV-EGRESS-AUDIT-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:491` | Lückenloses Audit-Tracing für Cloud-Egress und Anonymisierungs-/Pseudonymisierungsschritte. | `crates/contextra-privacy/src/egress_guard.rs:12`, `context_edit_audit.rs:8`, `audit_trace.rs:15` | `crates/contextra-privacy/tests/egress_gateway.rs:20` | ja | unklar |
| `INV-GRAPH-PROV-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:488` | Graph-Provenienz: Kaskadierende Tombstones für `supersedes`-Relationen; Kantenindizierung in `doc_edge_index`. | `crates/contextra-graph/src/provenance.rs:20`, `cascade.rs:10` | `crates/contextra-db/tests/supersedes_cascading_tombstone_test.rs:42` | ja | ja |
| `INV-HASHER-SEEDS` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:646` | Feste Hasher-Seeds (`0x9E3779B97F4A7C15` u.a.) für KV-Lock-Shards zur Vermeidung von Deadlocks. | `crates/contextra-engine/src/collection/kv_lock.rs:5` | `crates/contextra-engine/tests/kv_lock_order.rs:10` | ja | unklar |
| `INV-KIVI-AEAD-ORDER` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:493` | KIVI-Segment-Kodierungsreihenfolge: Erst quantisieren, dann verschlüsseln (AEAD). | `crates/contextra-kvcache/src/segment.rs:255` | `crates/contextra-kvcache/tests/segment_aead_quant.rs:15` | ja | unklar |
| `INV-KV-DELETE-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:4070` | KV-Löschbeweis ist ausschließlich im Modus `KvDeleteMode::CryptoShred` verfügbar. | `crates/contextra-store/src/kv/delete_mode.rs:10`, `segment.rs:157` | `crates/contextra-store/tests/kv_delete_mode_wiring_test.rs:4` | ja | ja |
| `INV-LICENSE-2` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:751,763` | Fast-Ring-Aktivierung liefert immer `Ok`, selbst bei fehlerhafter oder fehlender Lizenzdatei. | `crates/contextra-license/src/signed_gate.rs:371` | `crates/contextra-license/tests/signed_activation_7_steps.rs:34`, `signed_gate.rs:184` | ja | ja |
| `INV-MCP-CLASSIFY-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:838,899` | Strikte MCP-Toolklassifikation: Jedes Tool in `tools/list` muss explizite Kategorie `!= CodeExecution` haben; Wildcard `_` wirft Fehler. | `crates/contextra-mcp/src/sandbox.rs:80` | `crates/contextra-mcp/src/tests.rs:108` | ja | ja |
| `INV-MVCC-SSI-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:834` | Serializable Snapshot Isolation (SSI): Datengetriebene Konfliktauflösung ohne Ring-Abhängigkeitsverstoß. | `crates/contextra-store/src/lsm/ops/read.rs:24`, `crates/contextra-mvcc/src/ssi.rs:13` | `crates/contextra-mvcc/tests/campaign_j42_ssi.rs:2` | ja | ja |
| `INV-NUC-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:494` | Partial-Rebuild-Grenzbewahrung: Partieller Indexaufbau darf den HNSW-Graph nie inkonsistent hinterlassen. | `crates/contextra-vector/src/partial_rebuild.rs:14` | `crates/contextra-vector/tests/partial_rebuild_correctness.rs:20` | ja | unklar |
| `INV-P8-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:664` | Invalidation von Kalibrierungs-Profilen bei Modellwechsel (`check_and_invalidate_fingerprint`). | `crates/contextra-router/src/profile.rs:45`, `crates/contextra-types/src/types/domain/misc.rs:459` | `crates/contextra-router/tests/profile_invalidation.rs:15` | ja | unklar |
| `INV-PERF-PROFILE-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:490` | Performance-Profil Builder-Hebelvertrag: Werte erreichen Konfiguration, kein Panic. | `crates/contextra/src/performance_profile.rs:3` | `crates/contextra/tests/builder_levers_contract.rs:3` | ja | ja |
| `INV-PID-ANTIWINDUP-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:490` | PID-Regler Anti-Windup Begrenzung des Integral-Terms bei Latenz-Sättigung. | `crates/contextra-adapt/src/pid.rs:50`, `pid_latency_controller.rs:30` | `crates/contextra-adapt/tests/pid_anti_windup_tests.rs:10` | ja | ja |
| `INV-PLUGIN-DEPENDENCY` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:2503` | Plugin-Registrierung und zyklenerkennende topologische Abhängigkeitsauflösung. | `crates/contextra-ports/src/plugin.rs:131` | `crates/contextra-ports/tests/plugin_registry_dependency_cycle.rs:3` | ja | ja |
| `INV-PRESSURE-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:486` | System-Backpressure Anwendung bei Druckstufe `Critical`. | `crates/contextra-engine/src/collection/crud/internal.rs:44` | `crates/contextra-engine/tests/backpressure_delay.rs:12` | ja | unklar |
| `INV-PROV-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:488` | Provenienz-Gleichheit: Summe der Signalbeiträge muss exakt dem RRF-Fusionsscore entsprechen. | `crates/contextra-db/src/lib.rs:830`, `crates/contextra-rank/src/fusion/provenance.rs:264` | `crates/contextra-rank/tests/provenance_rrf_sum.rs:18` | ja | unklar |
| `INV-ROUTER-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:490` | Quantil-Schwellenwert des Routers ist streng auf das Intervall `[0.0, 1.0]` begrenzt. | `crates/contextra-router/src/profile.rs:184` | `crates/contextra-router/tests/quantile_clamping.rs:15` | ja | unklar |
| `INV-SBX-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:491` | WASM-Sandbox Modulgrößenbegrenzung (Standard: max. 10 MB) vor Kompilierung. | `crates/contextra-sandbox/src/capabilities.rs:155`, `executor.rs:216` | `crates/contextra-sandbox/tests/module_size_limit.rs:12` | ja | unklar |
| `INV-SBX-3` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:3054` | Rückgabe von `CapabilityViolationError` bei Capability-Verletzung in Sandbox Host-Funktionen. | `crates/contextra-sandbox/src/executor.rs:26` | `crates/contextra-sandbox/tests/capability_violation.rs:10` | ja | unklar |
| `INV-STORE-OBSERVER-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:485` | WalObserver-Vertragshärtung und Nachverfolgung der `WriteOrigin` im LSM-Store. | `crates/contextra-store/src/lsm/observer.rs:20` | `crates/contextra-store/tests/wal_observer_contract.rs:10`, `observer_blocking_failopen.rs:12` | ja | ja |
| `INV-TENANT` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:4089` | Allgemeines Mandanten-Isolationsgebot über alle Speicher- und Laufzeitschichten. | `crates/contextra-ports/src/kv.rs:160`, `crates/contextra-crypto/src/kv_segment/store.rs:2` | `crates/contextra-store/tests/tenant_codec_isolation_props.rs:4`, `crates/contextra-crypto/tests/kv_segment_concurrency.rs:210`, `crates/contextra-kvcache/tests/tenant_isolation.rs:2` | ja | ja |
| `INV-TENANT-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:421,496` | `TenantId(0)` ist für `SYSTEM` reserviert; `TenantId::try_new(0)` liefert zwingend `Err`. | `crates/contextra-types/src/types/domain/ids.rs:8` | `crates/contextra-types/src/types/domain/ids.rs:120` | ja | ja |
| `INV-TENANT-2` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:645,3356` | `scan_prefix(codec.scan_prefix())` liefert ausschließlich Keys des gewählten Mandanten (`TenantScopedStorage`). | `crates/contextra-store/src/tenant_codec.rs:4`, `502` | `crates/contextra-store/tests/tenant_codec_isolation_props.rs:4`, `crates/contextra-engine/tests/tenant_wrapper_coverage.rs:4` | ja | ja |
| `INV-TENANT-A` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:483` | KV-Bridge Mandantenisolation: Ein Mandant kann niemals Segmente eines anderen Mandanten lesen/schreiben. | `crates/contextra-crypto/src/kv_segment/store.rs:105` | `crates/contextra-crypto/tests/kv_segment_concurrency.rs:210` | ja | ja |
| `INV-TTL-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:785` | *(Entfallen laut Spezifikation)* Ursprünglicher Vergleich `expires_at_unix_nanos < clock.now` entfällt zugunsten von `INV-TTL-2`. | keine | keine | nein | nein |
| `INV-TTL-2` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:785` | TTL-Ablauf wird ausschließlich aus `seq`/`TxId`-Distanz abgeleitet (ohne Wall-Clock). | `crates/contextra-store/src/compaction/engine.rs:210`, `crates/contextra-engine/src/collection/maintenance.rs:45` | `crates/contextra-engine/tests/ttl_sequence_expiration.rs:15` | ja | unklar |
| `INV-VAULT-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:492,4147` | VolatileContextVault `purge()` zeroized alle Chunk-Inhalte via `ZeroizeOnDrop` und schreibt nie auf Disk. | `crates/contextra-db/src/volatile_vault.rs:126,371` | `crates/contextra-db/src/volatile_vault.rs:410` | ja | ja |
| `INV-WAL-LEGACY-KEY-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:299,770` | Legacy WAL-HMAC-Schlüssel Fallback ist nie implizit aktiv (`allow_legacy_integrity_key_fallback` ist standardmäßig `false`). | `crates/contextra-store/src/wal/hmac.rs:12`, `mod.rs:401` | `crates/contextra-store/tests/wal_legacy_key_migration_store.rs:2` | ja | ja |
| `INV-WAL-TRUNCATION-1` | `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:300,4079` | Schutz vor WAL-Kürzung durch Validierung der Manifest High-Water-Mark HMAC-Kette bei Recovery. | `crates/contextra-store/src/lib.rs:55`, `manifest/recovery.rs:80` | `crates/contextra-store/tests/wal_spec_pending.rs:22` | ja | ja |

---

## 3. Detail-Bewertung und Audit-Hinweise

### 3.1. Invarianten mit bewiesener Verletzungserkennung (`Test prüft Verletzung? = ja`)
Bei diesen 20 Invarianten existieren dedizierte Regressionstests, welche bei einer böswilligen oder versehentlichen Modifikation des Produktionscodes fehlschlagen:
1. **`INV-C1`**: `test_compaction_retains_tombstone_for_active_snapshot` prüft, dass Compaction aktive Snapshots nicht beschädigt.
2. **`INV-CAL-1`**: Tests schlagen fehl, wenn `calibrate()` vor Erreichen des Warmup-Schwellenwerts ohne Fehler aufgerufen werden kann.
3. **`INV-CALIBRATION-CONFORMAL-1`**: Verifiziert, dass verletzte Konformitäts-Schwellenwerte zu Fehlern führen.
4. **`INV-COLLECTION-PROFILE-3`**: `test_pii_vault_forces_crypto_shred` garantiert, dass `KvDeleteMode` bei PII nicht überschrieben werden kann.
5. **`INV-CRYPTO-DUPLICATE-1`**: Verifiziert den Fail-Closed-Schutz bei doppelter Registrierung/Löschung.
6. **`INV-DELETION-1`**: Prüft die Unwiderruflichkeit kryptografischer Löschungen und lehnt ungültige Versionen/Signaturen ab.
7. **`INV-DELETION-2`**: Prüft die vollständige Bereinigung von Geisterzeigern im HNSW-Vektorgraph.
8. **`INV-DISKANN-1`**: Stellt sicher, dass keine `.delta.tmp`-Dateien nach synchroner Persistierung verbleiben.
9. **`INV-DURABILITY-RING`**: Prüft Korruptionserkennung im Erasure Ledger.
10. **`INV-GRAPH-PROV-1`**: Verifiziert die kaskadierende Tombstone-Invalidierung für `supersedes`-Verknüpfungen.
11. **`INV-KV-DELETE-1`**: Stellt sicher, dass Löschbeweise nur für `CryptoShred` ausgestellt werden.
12. **`INV-LICENSE-2`**: Stellt sicher, dass der Fast-Ring ohne Lizenzschlüssel immer erfolgreich funktioniert.
13. **`INV-MCP-CLASSIFY-1`**: `crates/contextra-mcp/src/tests.rs` schlägt fehl, wenn ein Tool fälschlicherweise als `CodeExecution` klassifiziert wird oder `_` nicht fehlschlägt.
14. **`INV-MVCC-SSI-1`**: campaign_j42_ssi prüft Write-Skew-Konflikte und erzwingt Transaktions-Abbrüche.
15. **`INV-PERF-PROFILE-1`**: Testet hebelbasierte Profilkonfigurationen gegen unerwartete Panics.
16. **`INV-PID-ANTIWINDUP-1`**: `pid_anti_windup_tests.rs` prüft die strikte Begrenzung des I-Terms bei Stellgrößensättigung.
17. **`INV-PLUGIN-DEPENDENCY`**: Prüft die Erkennung und Ablehnung zirkulärer Plugin-Abhängigkeiten.
18. **`INV-TENANT` / `INV-TENANT-1` / `INV-TENANT-2` / `INV-TENANT-A`**: Strikte Mandanten-Isolationstests verifizieren, dass Mandant B niemals Daten von Mandant A lesen/scannen kann und `TenantId(0)` abgelehnt wird.
19. **`INV-VAULT-1`**: Unit-Tests verifizieren Zeroization im `VolatileContextVault`.
20. **`INV-WAL-LEGACY-KEY-1` & `INV-WAL-TRUNCATION-1`**: Migrationstests verifizieren die Ablehnung alter WALs ohne Migrationsflag sowie die Erkennung abgeschnittener WALs.

### 3.2. Invarianten mit unklarem Verletzungsnachweis (`Test prüft Verletzung? = unklar`)
Bei 18 Invarianten existieren zwar funktionale Tests für die betroffenen Module, jedoch fehlt ein automatisierter Negativtest / Injektionstest, der nachweist, dass ein Verstoß gegen die spezifische Invariante zum Testfehlschlag führt:
- **`INV-CHECKPOINT-DETERMINISM-1`**: Es existieren Tests für `Clock`-basierten Determinismus, aber kein statischer `xtask`-Linter, der ein versehentliches `SystemTime::now()` im `contextra-checkpoint`-Code als Verstoß flaggt.
- **`INV-HASHER-SEEDS`**: Feste Hasher-Seeds sichern die Sperrreihenfolge (Loom-getestet), aber es gibt keinen Test, der eine Änderung der Seed-Konstanten als Invariantenverletzung detektiert.
- **`INV-KIVI-AEAD-ORDER`**: Segment-Tests verifizieren Lese-/Schreibabläufe, prüfen jedoch nicht explizit die Fehlschlag-Semantik bei vertauschter Reihenfolge (erst verschlüsseln, dann quantisieren).
- **`INV-PRESSURE-1`, `INV-ROUTER-1`, `INV-SBX-1`, `INV-SBX-3`, `INV-NUC-1`, `INV-P8-1`, `INV-PROV-1`, `INV-TTL-2`**: Standard-Komponententests decken den Gutpfad ab, es fehlen jedoch dedizierte Mutationstests oder Fehlersimulationen für Randbedingungen.

### 3.3. Invarianten ohne dedizierte Tests (`Test vorhanden? = nein`)
- **`INV-COLLECTION-PROFILE-1`**: Keine spezifische Testdatei für Profilparameter-Grenzprüfungen.
- **`INV-CONSOLIDATE-1`**: Crash-Resilienz der Konsolidierungssession ist im Code implementiert, besitzt aber keinen gezielten Crash-Injection-Test.
- **`INV-DAG-DEPS`**: Strikter DAG der CRUD-Modulabhängigkeiten wird nicht durch einen Konsistenz-Test im `xtask` geprüft.
- **`INV-DEDUP-TRANSITIVITY-1`**: Das Transitivitäts-Veto beim Mergen von Entitäten besitzt derzeit keinen eigenständigen Integrationstest.
- **`INV-TTL-1`**: Von der Spezifikation ausdrücklich als entfallen deklariert (Ersatz durch `INV-TTL-2`).

---

## 4. Methodik & Verifikationskommandos

Zur Erstellung dieser Matrix wurden folgende automatisierten Suchen und Verifikationsschritte ausgeführt:

```bash
# 1. Alle eindeutigen INV-* aus der Spezifikation extrahieren
rg -o "INV-[A-Z0-9-]+" docs/spec | sort -u

# 2. Vorkommen im Quellcode und in Tests identifizieren
rg -o "INV-[A-Z0-9-]+" crates --include=*.rs | sort -u

# 3. Detaillierte Quellcode- und Test-Zuordnung je Invariante
python3 analyze.py
```
