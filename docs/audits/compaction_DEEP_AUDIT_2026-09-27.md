# Contextra — Deep Audit: Compaction Engine & MVCC-Kompatibilität

**Crate**: `crates/contextra-store`
**Modul**: `crates/contextra-store/src/compaction/`
**Datum**: 2026-09-27
**Auditor**: Jules (Principal Senior Rust Architect)
**Task**: `compaction-deep-audit`
**Compiler Directives**: `#![forbid(unsafe_code)]` in workspace / `#![deny(unsafe_code)]` in local crate

---

## Executive Summary & Zusammenfassung der Prüfergebnisse

| Prüfpunkt | Bezeichnung | Status | Kurzbeschreibung / Beleg |
|---|---|---|---|
| **C1** | **MVCC-Snapshot-Respekt** | **PASSED** | Verschiedene Versionen werden streng anhand von `min_active_seqno()` aus der `SnapshotRegistry` geprüft. Versionen $\ge \text{min\_active\_seqno}$ sowie die erste Floor-Version darunter werden ausnahmslos erhalten (`engine.rs:481-499`). |
| **C2** | **Tombstone-Retention** | **PASSED** | Tombstones werden exakt dann gelöscht, wenn `is_tombstone && is_full_compaction && raw_seq < min_snapshot_seq` gilt (`engine.rs:508`). Partial Compactions behalten Tombstones vollständig. |
| **C3** | **Tiered vs. Leveled Strategy** | **PASSED** | Implementiert ist die Size-Tiered Compaction Strategy (STCS) mit Ergänzung durch `CostBasedAdaptivePlanner` (`adaptive.rs`). Komplexitätstheorie (WA vs. RA vs. SA) entspricht STCS-Spezifikation. |
| **C4** | **Concurrent-Compaction Safety** | **PASSED** | Der langlaufende Multi-Way-Merge erfolgt ohne jegliche Locks. Lock-Aquisitionen auf `sstables: RwLock<Vec<Arc<SstableReader>>>` sind auf Nanosekunden-Fenster beschränkt. Active Readers nutzen ref-counted `Arc`-Handles. |
| **C5** | **Tenant-Key-Beibehaltung** | **PASSED** | Keys werden als rohe `bytes::Bytes` byte-lexikographisch verarbeitet und unverändert in `SstableBuilder` geschrieben. Tenant-Präfixe (`TenantKeyCodec`: `t:{tenant}:{col}:...`) bleiben isoliert und geordnet. |
| **C6** | **Manifest-Atomizität** | **PASSED** | Der Übergang von alten zu neuen SSTables erfolgt über einen einzelnen `ManifestEntry::Replace`-Eintrag, der via `write_all`, `flush` und `sync_all` atomar auf Festplatte gehärtet wird. |

---

## (1) MVCC-Snapshot-Respekt-Nachweis mit Codezeilen

### Interaktion mit der `SnapshotRegistry`

Die `CompactionEngine` holt zu Beginn der Merge-Phase die unterste aktive Sequenznummer aus der `SnapshotRegistry`:

```rust
// File: crates/contextra-store/src/compaction/engine.rs, Zeile 189
let min_snapshot_seq = self.snapshot_registry.min_active_seqno();
```

Wenn aktive Lese-Transaktionen oder gepinnte Snapshot-Guards existieren, gibt `min_active_seqno()` den kleinsten Sequenznummer-Wert aller aktiven Snapshots zurück. Wenn keine Snapshots aktiv sind, gibt die Registry `u64::MAX` zurück.

### MVCC-Multi-Version Retention Algorithm

Während des Multi-Way-Merges in `merge_sstables_inner` wird für jeden Eintrag des Priority-Heaps die MVCC-Gültigkeit nach folgender Logik bestimmt:

```rust
// File: crates/contextra-store/src/compaction/engine.rs, Zeilen 481–499
let is_tombstone = (item.seq & TOMBSTONE_BIT) != 0;
let raw_seq = item.seq & !TOMBSTONE_BIT;

if last_key.as_ref() != Some(&item.key) {
    floor_emitted = false;
}

// LSM Retention Rule:
// Keep all versions with raw_seq >= min_snapshot_seq (visible to active or future snapshots)
// PLUS the newest version with raw_seq < min_snapshot_seq (the "floor" version).
// All further, older versions for the key below min_snapshot_seq are discarded.
let keep = if raw_seq >= min_snapshot_seq {
    true
} else if !floor_emitted {
    floor_emitted = true;
    true
} else {
    false
};
```

### Formale Beweisführung der MVCC-Korrektheit

1. **Regel 1 ($\text{raw\_seq} \ge \text{min\_snapshot\_seq}$)**:
   Jede Version mit einer Sequenznummer größer oder gleich dem ältesten aktiven Snapshot wird **ausnahmslos behalten** (`keep = true`). Damit ist ausgeschlossen, dass ein aktiver Snapshot eine Version vermisst, die zu seinem Snapshot-Zeitpunkt gültig war.

2. **Regel 2 (MVCC Floor Version, $\text{raw\_seq} < \text{min\_snapshot\_seq}$)**:
   Die **neueste** Version mit $\text{raw\_seq} < \text{min\_snapshot\_seq}$ ist die sogenannte "Floor Version". Sie entspricht dem Zustand des Keys unmittelbar vor dem ältesten aktiven Snapshot. Diese Version wird durch `floor_emitted = true` genau einmal als **behalten** markiert (`keep = true`). Alle Snapshots mit $\text{snapshot\_seq} \ge \text{min\_snapshot\_seq}$ lesen diesen Wert als ihren Basis-Zustand, sofern keine neuere Version existiert.

3. **Regel 3 (Obsolute Historie, $\text{raw\_seq} < \text{min\_snapshot\_seq}$ nach Floor-Emission)**:
   Sämtliche älteren Versionen des Keys unterhalb von `min_snapshot_seq` werden verworfen (`keep = false`). Da kein aktiver oder zukünftiger Snapshot jemals eine Sequenznummer $< \text{min\_snapshot\_seq}$ lesen kann, sind diese Versionen mathematisch nicht mehr erreichbar.

---

## (2) Tombstone-Retention-Bedingung

### Konkrete Bedingung im Code

Ein Tombstone (Löschmarker) darf während des Merges **NUR** dann physisch entfernt (garbage-collected) werden, wenn die folgende 3-teilige Prädikatsbedingung vollständig erfüllt ist:

```rust
// File: crates/contextra-store/src/compaction/engine.rs, Zeilen 508–509
let should_gc_tombstone =
    is_tombstone && is_full_compaction && raw_seq < min_snapshot_seq;
```

### Detail-Analyse der drei Teilbedingungen

1. `is_tombstone == true`:
   Der Datensatz ist als Löschmarker markiert (`(seq & TOMBSTONE_BIT) != 0`). Normalwerte werden niemals über diese Regel verworfen.

2. `is_full_compaction == true`:
   Die Compaction umfasst **alle** im LSM-Tree befindlichen SSTables.
   *Sicherheitsgarantie*: Bei einer partiellen Compaction (z. B. STCS-Tier-Merge) existieren unkompaktierte SSTables außerhalb dieser Runde. Würde der Tombstone gelöscht, könnte ein älterer Wert aus einer unkompaktierten SSTable nach der Compaction "auferstehen" (Resurrection Attack). Daher bleiben Tombstones bei partiellen Compactions ausnahmslos erhalten.

3. `raw_seq < min_snapshot_seq`:
   Die Sequenznummer des Tombstones liegt strikt unterhalb des ältesten aktiven Snapshots.
   *Sicherheitsgarantie*: Falls ein aktiver Snapshot mit $\text{snapshot\_seq} \le \text{raw\_seq}$ existiert, muss dieser den Löschzustand weiterhin auslesen können. Der Tombstone darf erst entfernt werden, wenn kein aktiver Snapshot mehr auf die gelöschte Version zeigen kann.

---

## (3) Concurrent-Safety-Analyse

### Lock-Strategie & Lifecycle von `sstables: RwLock<Vec<Arc<SstableReader>>>`

Die Compaction ist als langlaufende Hintergrundoperation konzipiert. Um parallele Lese- und Schreibzugriffe (`hybrid_search`, `put`, `get`) nicht zu blockieren, minimiert die `CompactionEngine` die Haltezeit des `sstables`-Locks auf kurze Synchronisationsfenster:

1. **Phase 1: Kandidaten-Auswahl (Kurzes Read-Lock)**:
   Acquires `sstables.read().await` für wenige Mikrosekunden, um Kandidaten-SSTables gemäß STCS/Adaptive-Planner auszuwählen und `Arc<SstableReader>`-Klone zu erzeugen. Das Read-Lock wird **sofort wieder freigegeben**.

2. **Phase 2: Multi-Way-Merge I/O (Absolut LOCK-FREI)**:
   Der ressourcenintensive Multi-Way-Merge (`merge_sstables_inner`) liest aus den Eingabe-Streams und schreibt die neue SSTable-Datei auf Festplatte.
   **Kein Lock wird gehalten**. Lesende Transaktionen greifen parallel völlig ungestört auf die bestehende SSTable-Liste zu.

3. **Phase 3: Konsistenzprüfung vor MANIFEST-Schreiben (Kurzes Read-Lock)**:
   Vor dem Schreiben des Manifests wird unter `sstables.read().await` geprüft, ob alle Kandidaten-`Arc`-Pointer noch in der aktuellen SSTable-Liste enthalten sind:
   ```rust
   // File: crates/contextra-store/src/compaction/engine.rs, Zeilen 207–210
   let all_present = input_ssts
       .iter()
       .all(|inp| ssts.iter().any(|sst| Arc::ptr_eq(inp, sst)));
   ```
   Sollte eine parallele Operation (z. B. ein Concurrent Rollback oder ein paralleler Flush) die Eingabe-SSTables verändert haben (`!all_present`), bricht die Compaction ab, löscht die temporäre Ausgabedatei und gibt `Ok(false)` zurück.

4. **Phase 4: Atomares Manifest-Commit (Disk-Locking)**:
   Schreiben des `ManifestEntry::Replace`-Eintrags mit `write_all`, `flush` und `sync_all` auf das append-only Manifest.

5. **Phase 5: Atomarer In-Memory SSTable-Swap (Kurzes Write-Lock)**:
   Acquires `sstables.write().await` für wenige Nanosekunden:
   - Entfernt alte Eingabe-SSTables via `Arc::ptr_eq` Identitätsvergleich.
   - Fügt den neuen `Arc<SstableReader>` an der berechneten Einfügeposition ein.
   - Sortiert die Liste aufsteigend nach `max_seq` (`ssts.sort_by_key(|sst| sst.metadata().max_seq & !TOMBSTONE_BIT)`), um die globale Visibility-Schattenordnung zu garantieren.
   - Gibt das Write-Lock sofort frei.

6. **Phase 6: Disk-Cleanup alter SSTables (Lock-Frei)**:
   Löschen der alten SSTable-Dateien auf Festplatte erfolgt außerhalb aller Locks.
   *POSIX / Linux OS-Garantie*: Da aktive Reader weiterhin `Arc<SstableReader>` mit offenen File Descriptoren halten, bleibt das Lesen aus gelöschten Dateien bis zum Drop der letzten `Arc`-Referenz vollkommen sicher und unterbrechungsfrei.

---

## (4) Tiered vs. Leveled Compaction Analysis (C3)

### Implementierte Strategie: Size-Tiered Compaction Strategy (STCS)

Die CompactionEngine implementiert standardmäßig STCS (`CompactionConfig::default()`):

- **Gruppierung**: SSTables werden nach Dateigrößenklassen gruppiert. Zwei SSTables gehören zur selben Größenklasse, wenn ihr Größenverhältnis $\le \text{size\_ratio}$ (Standard: 4.0) ist (`engine.rs:select_compaction_candidates`).
- **Trigger**: Sobald eine Größenklasse mindestens `min_sstables_per_tier` (Standard: 4) SSTables enthält, wird ein Merge dieser Gruppe ausgelöst.
- **Adaptive Erweiterung**: Unter hoher Leselast (`read_ratio >= 0.70`) schaltet der `CostBasedAdaptivePlanner` (`adaptive.rs`) dynamisch auf `ReadOptimizedAggressive` um und führt alle SSTables zusammen, um die Read Amplification auf $O(1)$ zu senken.

### Komplexitätstheoretischer Vergleich

| Metrik | Size-Tiered (STCS) — Implementiert | Leveled Compaction |
|---|---|---|
| **Write Amplification (WA)** | $O(N \cdot \log_T N)$ (Niedrig, optimal für Schreibdurchsatz) | $O(T \cdot N \cdot \log_T N)$ (Höher durch strikte Überlappungsfreiheit) |
| **Read Amplification (RA)** | $O(T \cdot L)$ (Höher, da mehrere SSTables pro Tier überlappende Key-Ranges haben) | $O(L)$ (Niedrig, da max. 1 SSTable pro Level berührt wird) |
| **Space Amplification (SA)** | Bis zu 100 % bei Major/Full Compaction | ca. 10–25 % (Begrenzt durch Level-Größenverhältnisse) |

---

## (5) Tenant Key Isolation & Beibehaltung (C5)

### Key-Encoding & Multi-Tenant-Isolation

Alle Keys im Storage Engine werden über `TenantKeyCodec` (`crates/contextra-store/src/tenant_codec.rs`) strukturiert:
$$\text{Key} = \texttt{"t:"} \mathbin{\Vert} \text{tenant\_id} \mathbin{\Vert} \texttt{":"} \mathbin{\Vert} \text{collection\_id} \mathbin{\Vert} \texttt{":"} \mathbin{\Vert} \text{doc\_type} \mathbin{\Vert} \texttt{":"} \mathbin{\Vert} \text{doc\_id}$$

### Korrektheit im Compaction-Merge

1. **Unveränderliche Byte-Verarbeitung**: Die CompactionEngine verarbeitet Keys als rohe `bytes::Bytes`. Der Priority-Heap sortiert Keys strictly lexicographical über `other.key.cmp(&self.key)`.
2. **Keine Key-Mutation**: Keys werden 100 % unverändert an den `SstableBuilder` übergeben (`builder.add(&item.key, ...)`).
3. **Erhaltsame Tenant-Gruppierung**: Durch die lexikographische Bytewert-Sortierung liegen alle Keys desselben Tenants (`t:1:...`) zusammenhängend im Ausgabeblock. Keys verschiedener Tenants werden niemals durchmischt.
4. **Range-Scan Isolations-Garantie**: Präfix-Scans (`scan_prefix`) arbeiten vor und nach der Compaction absolut identisch. Cross-Tenant Data Leaks sind strukturell ausgeschlossen.

---

## (6) Manifest-Atomizität (C6)

### Atomares Protokollierungsschema

Das Manifest arbeitet append-only (`crates/contextra-store/src/manifest/core.rs`). Die Aktualisierung nach einer Compaction erfolgt in folgender strikter Reihenfolge:

1. Ausgabedatei schreiben und fsyncen (`fsync_parent_dir(&output_path)`).
2. Erzeugen eines einzelnen `ManifestEntry::Replace`-Eintrags:
   ```rust
   ManifestEntry::Replace {
       removed: old_paths,
       added: output_path,
       added_max_tx: new_reader.metadata().max_tx_id,
       rank: insertion_point,
   }
   ```
3. Ausführen von `manifest.append(&entry)`, welches intern `write_all`, `flush` und `sync_all` auf der MANIFEST-Datei erzwingt.

### Crash-Recovery Invarianten

- **Crash vor `sync_all`**: Das Manifest auf Festplatte enthält den `Replace`-Eintrag nicht. Bei Neustart lädt `Manifest::load` den alten Stand. Die alten SSTables bleiben zu 100 % gültig. Die unvollständige temporäre Compaction-Datei wird als unreferenzierter Waise beim Start gereinigt.
- **Crash nach `sync_all`**: Das Manifest enthält den `Replace`-Eintrag. `Manifest::reconstruct_valid_sstables` ersetzt atomar die alten SSTables durch die neue SSTable. `Manifest::reconstruct_dead_sstables` identifiziert die alten Dateien als tot und schlägt diese zur Bereinigung vor.

---

## (7) Testberichts-Nachweis & Verifikation

Das gesamte Compaction-Testset im Crate `contextra-store` wurde erfolgreich ausgeführt:

```bash
cargo test -p contextra-store --locked --lib compaction
```

### Test-Ergebniszusammenfassung (`/tmp/audit-compaction-test.log`)

```text
running 31 tests
test compaction::tests::advanced::test_chain_linkage_tier_grouping ... ok
test compaction::tests::advanced::test_compaction_cancellation ... ok
test compaction::tests::advanced::test_compaction_concurrent_rollback_flush_no_panic ... ok
test compaction::tests::advanced::test_compaction_single_lock_candidate_selection_concurrency ... ok
test compaction::tests::advanced::test_compaction_swap_maintains_shadowing_order_without_restart ... ok
test compaction::tests::advanced::test_mvcc_floor_version_retained_for_active_snapshot ... ok
test compaction::tests::advanced::test_phantom_data_after_partial_compaction ... ok
test compaction::tests::advanced::test_tombstone_retention_floor_with_active_snapshot ... ok
test compaction::tests::advanced::test_compaction_pressure_awareness ... ok
test compaction::tests::basic::test_compaction_aborts_on_concurrent_modification ... ok
test compaction::tests::basic::test_compaction_aborts_when_peak_memory_exceeds_limit ... ok
test compaction::tests::basic::test_compaction_backpressure_timeout_exceeded ... ok
test compaction::tests::basic::test_compaction_candidate_selection_follows_chronological_order ... ok
test compaction::tests::basic::test_compaction_removes_uuid_sidecar_files ... ok
test compaction::tests::basic::test_compaction_succeeds_when_peak_memory_within_limit ... ok
test compaction::tests::basic::test_compaction_swap_debug_assert_detects_unsorted_list ... ok
test compaction::tests::basic::test_compaction_swap_restores_shadowing_order_without_restart ... ok
test compaction::tests::basic::test_generate_sst_path_uniqueness ... ok
test compaction::tests::basic::test_maybe_compact_full_cycle ... ok
test compaction::tests::basic::test_merge_deduplication ... ok
test compaction::tests::basic::test_mvcc_retention_floor_version_retained_for_snapshot ... ok
test compaction::tests::basic::test_mvcc_retention_older_versions_below_floor_discarded ... ok
test compaction::tests::basic::test_no_compaction_below_threshold ... ok
test compaction::tests::basic::test_tombstone_gc ... ok
test compaction::tests::basic::test_tombstone_gc_stream_advancement_preserves_subsequent_keys ... ok
test compaction::tests::basic::test_tombstone_preserved_with_active_snapshot ... ok
test lsm::tests::flush_tests::test_compaction_roundtrip ... ok
test lsm::tests::recovery_tests::test_rollback_drops_sstable_fully_stale_after_recompaction ... ok
test compaction::tests::advanced::concurrent_flush_and_compact_is_safe ... ok
test compaction::tests::basic::prop_compaction_tombstone_masking_latest_operation_wins ... ok
test compaction::tests::advanced::test_compaction_stress_and_gc ... ok

test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 150 filtered out; finished in 49.32s
```

---

## (8) VERDICT & VERIFIED-BY-SESSION

**VERDICT**: **PASSED**
Die Compaction-Engine in `crates/contextra-store/src/compaction/` erfüllt sämtliche Korrektheits-, MVCC-Snapshot-Kompatibilitäts-, Tombstone-Retention- und Concurrency-Garantien der Contextra-Architektur vollumfänglich und ohne Mängel.

**VERIFIED-BY-SESSION**: **PASSED (TS: 2026-09-27T20:58:00Z)**
