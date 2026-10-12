# Korrektheit- und MVCC-Kompatibilitäts-Audit: Compaction-Engine (`contextra-store`)

**Datum:** 2026-10-12
**Crate:** `contextra-store`
**Prüfgegenstand:** Compaction-Engine (`crates/contextra-store/src/compaction/`)
**Auditor:** Principal Senior Rust Architect (Jules)
**Status:** Audit abgeschlossen

---

## 0. Session Bootstrap & Systemdiagnose Summary

- **(a) SESSION_HASH & Timestamp:**
  - `SESSION_HASH`: `a1e138fb`
  - `Timestamp`: `2026-10-12T16:43:56Z`
- **(b) BLOCKER / CRITICAL-Scan:**
  - Open Unresolved Findings: **0**
- **(c) Aktive VETOs & Fristen:**
  - `VETO F-02`: `conditional_review_due: 2026-10-07` (Ablaufwarnung; betrifft partial HNSW rebuild, nicht berührt)
  - `VETO OP-03`: `conditional_review_due: 2026-10-07` (Ablaufwarnung; betrifft Voice/Audio, nicht berührt)
  - `VETO F-10`: `permanent_rejected` (Mandantenisolation)
- **(d) Workspace Build-Status:**
  - `cargo check --workspace --exclude contextra-py`: **OK** (0 Fehler, 1 Warnung in `contextra-mcp`)
- **(e) Git-Historie (Letzte geänderte Module):**
  - `contextra-core` (`crates/contextra-core/lib.rs`)
  - `contextra-types` (`crates/contextra-types/`)
  - `contextra-crypto` (`crates/contextra-crypto/`)
- **(f) Ring-Layering Audit:**
  - Strict Compliance: **OK mit 6 dokumentierten/warnenden Abweichungen im Warning-Modus**
- **(g) Unsafe-Inseln Audit:**
  - Status: **Konform** (0 Fehler in `check-unsafe-islands`)
- **(h) Empfehlung:**
  - **GO** für produktive Audits und dokumentative Arbeiten.

---

## 1. MVCC-Snapshot-Respekt-Nachweis (C1)

Die Compaction-Engine garantiert strikte MVCC-Snapshot-Isolations-Invarianten. Keine Version einer Sequenznummer, die noch von einem aktiven Snapshot gelesen werden kann oder muss, wird durch Compaction/GC vorzeitig gelöscht oder überschrieben.

### Code-Nachweis & Zeilenreferenzen (`crates/contextra-store/src/compaction/engine.rs`)

1. **Ermittlung der Snapshot-Watermark (`min_snapshot_seq_bound`):**
   - **Zeile 124–127:**
     ```rust
     #[inline]
     pub fn min_snapshot_seq_bound(&self) -> u64 {
         self.floor.floor()
     }
     ```
   - `self.floor` ist ein `Arc<dyn SnapshotFloor>`, welches in `LsmStorage` an die `SnapshotRegistry` und den aktiven Transaction-Buffer gekoppelt ist. Es berechnet dynamisch den globalen Minimalwert aller aktiven Snapshot-Sequenznummern (`min_active_seqno`).

2. **Abruf der Watermark vor Merge-Beginn:**
   - **Zeile 203:**
     ```rust
     let min_snapshot_seq = self.min_snapshot_seq_bound();
     ```
   - Die Watermark `min_snapshot_seq` wird exakt vor Ausführung des Multi-Way-Merges ermittelt und an `merge_sstables_with_cancel` übergeben.

3. **Verarbeitungslogik im Multi-Way-Merge (`merge_sstables_inner`):**
   - **Zeilen 506–538:**
     ```rust
     if raw_seq > min_snapshot_seq {
         // Version is above floor: keep as separate entry
         let should_gc_tombstone =
             is_tombstone && is_full_compaction && raw_seq < min_snapshot_seq;
         if !should_gc_tombstone {
             self.write_entry_with_rate_limit(...).await?;
         }
         last_key = Some(current_item.key.clone());
     } else if !floor_emitted {
         // Version is at or below the floor (raw_seq <= min_snapshot_seq)
         // Exhaust and fold ALL remaining versions for this key <= min_snapshot_seq into a single entry.
         ...
     }
     ```
   - **Invariante:** Alle Versionen mit `raw_seq > min_snapshot_seq` liegen **oberhalb** des Snapshot-Floors. Sie repräsentieren neuere Datenstände, die für aktive oder zukünftige Snapshots sichtbar sein müssen. Diese Einträge werden unberührt und einzeln in die neue SSTable geschrieben.
   - Erst Versionen mit `raw_seq <= min_snapshot_seq` erreichen den Floor und werden zu genau einer Version konsolidiert (bzw. bei Tombstones gc'd).

---

## 2. Tombstone-Retention-Bedingung (C2)

Tombstones (gelöschte Schlüssel, gekennzeichnet durch das `TOMBSTONE_BIT` in der Sequenznummer) dürfen **nicht** vorschnell entfernt werden, da sonst alte Versionen in tieferen SSTables fälschlicherweise wieder sichtbar werden würden (Ghost Records / Resurrected Keys).

### Konkrete Bedingung im Code

Ein Tombstone wird **ausschließlich** genau dann eliminiert (`should_gc_tombstone == true`), wenn **alle drei** folgenden Bedingungen simultan erfüllt sind:

$$\text{should\_gc\_tombstone} \iff \text{is\_tombstone} \land \text{is\_full\_compaction} \land (\text{raw\_seq} < \text{min\_snapshot\_seq})$$

### Code-Nachweis (`crates/contextra-store/src/compaction/engine.rs`)

- **Zeile 516–520:**
  ```rust
  let should_gc_tombstone =
      is_tombstone && is_full_compaction && raw_seq < min_snapshot_seq;
  if !should_gc_tombstone {
      self.write_entry_with_rate_limit(&mut builder, ...).await?;
  }
  ```
- **Zeile 584–587 (gefoldete Floor-Tombstones):**
  ```rust
  let should_gc_tombstone = is_acc_tombstone
      && is_full_compaction
      && (acc_seq & !TOMBSTONE_BIT) < min_snapshot_seq;
  if !should_gc_tombstone {
      self.write_entry_with_rate_limit(&mut builder, ...).await?;
  }
  ```

### Begründung der Teilbedingungen:
1. `is_tombstone` / `is_acc_tombstone`: Der Eintrag ist explizit als Löschangebot markiert (`seq & TOMBSTONE_BIT != 0`).
2. `is_full_compaction`: **Sämtliche** SSTables der Datenbank nehmen an der Compaction teil. Es existieren keine älteren SSTables im Storage, in denen veraltete Werte desselben Schlüssels überleben könnten. Bei einer teilweisen (tier-spezifischen) Compaction muss der Tombstone erhalten bleiben, um tiefere SSTable-Einträge weiterhin zu beschatten.
3. `raw_seq < min_snapshot_seq`: Kein aktiver MVCC-Snapshot liest auf einem Sequenzstand, an dem der Schlüssel noch existierte oder die Löschung noch nicht stattgefunden hatte.

---

## 3. Strategie-Analyse: Tiered vs. Leveled (C3)

### Implementierte Strategie
In `contextra-store` ist primär eine **Size-Tiered Compaction Strategy (STCS)** implementiert, erweitert um ein **Workload-Adaptives Compaction-Modul** (`CostBasedAdaptivePlanner` in `adaptive.rs`).

- **STCS-Prinzip:** SSTables werden nach Dateigröße in Tier-Größenklassen gruppiert (`size_ratio`, Standard: 4.0). Wenn eine Größenklasse mindestens `min_sstables_per_tier` (Standard: 4) SSTables enthält, werden diese zu einer größeren SSTable zusammengeführt.
- **Adaptive Erweiterung:** Bei hohem Leseanteil (`read_ratio >= adaptive_read_ratio_threshold`, z. B. 0.70) schaltet der Planner in den Modus `ReadOptimizedAggressive` und führt alle Kandidaten-SSTables zusammen (Full Compaction), um die Lese-Verstärkung (Read Amplification) zu minimieren.

### Komplexitätstheoretische Bewertung
- **STCS (Size-Tiered):**
  - *Schreibverstärkung (Write Amplification):* $\mathcal{O}(\log_N(\text{Data Size}))$, sehr gering ($N = \text{size\_ratio}$). Optimal für schreibintensive Workloads.
  - *Leseverstärkung (Read Amplification):* $\mathcal{O}(N \cdot L)$, höher als bei Leveled Compaction, da Schlüsselbereiche innerhalb eines Tiers überlappen können.
  - *Speicherplatzverstärkung (Space Amplification):* Höher (benötigt bis zu 50 % freien Speicherplatz während des Merges eines großen Tiers).
- **Leveled Compaction (Vergleich):**
  - Leveled garantiert strikt disjunkte Schlüsselbereiche pro Level ($L_1, L_2, \dots$), was Lesezugriffe auf max. 1 SSTable pro Level begrenzt, erhöht jedoch die Schreibverstärkung signifikant.

**Fazit:** STCS ist in `contextra-store` korrekt implementiert und durch Key-Overlap-Optimierung (`compute_key_overlap_score` in `adaptive.rs`) sowie adaptive Umschaltung bei Lese-Workloads bezüglich Komplexität und Latenz optimal ausbalanciert.

---

## 4. Concurrent-Compaction-Safety (C4)

Compaction ist eine langlaufende I/O-intensive Hintergrundoperation. Ein Blockieren von Lesezugriffen während des gesamten Merges würde P95/P99-Latenzanforderungen verletzen.

### Synchronisationsmodell & Lock-Semantik (`crates/contextra-store/src/compaction/engine.rs`)

1. **Phasen-Trennung (Locking Strategy):**
   - **Kandidaten-Auswahl (Read-Lock Window):** Ein lfd. Read-Lock auf `sstables: &RwLock<Vec<Arc<SstableReader>>>` wird kurzzeitig erworben (Zeilen 143–185), um Kandidaten auszuwählen und `Arc`-Klone zu erstellen.
   - **Multi-Way-Merge (Lock-Frei):** Der zeitintensive Merge- und Schreibvorgang auf der Festplatte (`merge_sstables_with_cancel`, Zeilen 204–211) findet **vollständig ohne gehaltenes Lock** statt. Parallele Read- und Write-Transaktionen laufen ungehindert auf den bestehenden SSTable-Readern weiter.
   - **In-Memory Swap (Write-Lock Window):** Erst nach erfolgreichem Fsync des neuen SSTable-Files wird kurzzeitig das Write-Lock auf `sstables` erworben (Zeilen 229–283).

2. **Schutz gegen Race Conditions (Concurrent Flush / Rollback):**
   - Unter dem Write-Lock prüft die Compaction Engine mittels Identitätsvergleich (`Arc::ptr_eq`), ob die ursprünglichen Kandidaten-SSTables noch in exakt derselben zusammenhängenden Reihenfolge in der globalen SSTable-Liste vorliegen:
     ```rust
     let is_contiguous = match start_opt {
         Some(start) => {
             if start + input_ssts.len() <= ssts.len() {
                 input_ssts.iter().enumerate().all(|(idx, inp)| Arc::ptr_eq(inp, &ssts[start + idx]))
             } else { false }
         }
         None => false,
     };
     ```
   - Wenn ein konkurrierender Flush oder Rollback die Liste modifiziert hat (`!is_contiguous`), wird die Compaction **sicher abgebrochen**, die temporär erzeugte Datei gelöscht und kein atomarer Swap durchgeführt (Zeilen 248–266).

---

## 5. Tenant-Key-Beibehaltung (C5)

Multi-Tenancy wird in `contextra-store` über `TenantKeyCodec` (`crates/contextra-store/src/tenant_codec.rs`) durch Präfix-Kodierung durchgesetzt: `t:{tenant_id}:{collection_id}:...`.

### Multi-Tenant Isolation während Compaction

1. **Binarer Erhalt der Keys:**
   - Der Merge-Iterator in `CompactionEngine` liest und vergleicht Byte-Schlüssel in roher Form (`current_item.key`).
   - Schlüssel werden während der Compaction weder umcodiert, dekodiert noch gekürzt. Das Präfix `t:{tenant_id}:...` bleibt in den gemergten SSTables **exakt und unwerändert** erhalten.
2. **Keine Vermischung zwischen Tenants:**
   - Da alle Keys lexikographisch sortiert in den SSTables vorliegen und das Tenant-Präfix an erster Stelle steht (`t:1:...` < `t:2:...`), bleiben die Schlüssel verschiedener Mandanten in den zusammengeführten SSTables strikt nach Tenant-Blöcken getrennt.
   - Der `TenantScopedStorage` garantiert bei Lese-Scans via `scan_prefix(codec.scan_prefix())`, dass niemals Daten anderer Tenants zurückgegeben werden (Invariante `INV-TENANT-2`).

---

## 6. Manifest-Atomizität (C6)

Die Dauerhaftigkeit der SSTable-Metadaten und der Schutz gegen Crashes während der Compaction wird über das Append-Only `Manifest` (`crates/contextra-store/src/manifest.rs`) geregelt.

### Atomarer Manifest-Update-Ablauf (`crates/contextra-store/src/compaction/engine.rs`)

1. **Reihenfolge der Operationen:**
   - Das neue SSTable-File wird erzeugt und das Verzeichnis synchronisiert (`fsync_parent_dir`, Zeile 214).
   - Unter dem `sstables.write()`-Lock wird **zuerst** genau ein atomarer `Replace`-Eintrag in das Manifest geschrieben (Zeilen 261–269):
     ```rust
     if let Some(ref manifest) = self.manifest {
         manifest.append(&crate::manifest::ManifestEntry::Replace {
             removed: old_paths.clone(),
             added: output_path.clone(),
             added_max_tx: new_reader.metadata().max_tx_id,
             rank: 0,
         }).await?;
     }
     ```
   - Erst **nach** dem erfolgreichen Manifest-Append werden die alten `SstableReader` aus dem In-Memory-Vektor entfernt und der neue `SstableReader` an der richtigen Stelle eingefügt.
   - Die alten physischen SSTable-Dateien werden erst **nach** Freigabe des Write-Locks außerhalb der kritischen Sektion gelöscht (Zeilen 290–310).

2. **Crash-Resistenz:**
   - Bricht das System *vor* dem Manifest-Append ab, liest der Recovery-Prozess beim Neustart den alten Manifest-Stand. Das neue File ist unreferenziert und wird als Orphan gereinigt.
   - Bricht das System *nach* dem Manifest-Append ab, stellt das Manifest beim Replay den neuen Zustand wieder her (alte Dateien werden ignoriert/gelöscht, neue Datei wird geladen).

---

## 7. Audit-Urteil (VERDICT)

```
================================================================================
VERDICT: APPROVED
EVIDENCE:
- C1_MVCC_RESPECT: min_snapshot_seq_bound() (engine.rs:124), raw_seq > min_snapshot_seq Check (engine.rs:506)
- C2_TOMBSTONE_RETENTION: is_tombstone && is_full_compaction && raw_seq < min_snapshot_seq (engine.rs:516, 584)
- C3_STRATEGY_COMPLEXITY: Size-Tiered (STCS) in select_stcs_candidates (adaptive.rs:223), CostBasedAdaptivePlanner (adaptive.rs:120)
- C4_CONCURRENT_SAFETY: Lock-free multi-way merge (engine.rs:204), is_contiguous swap check under write lock (engine.rs:232)
- C5_TENANT_ISOLATION: Immutable raw key byte processing preserving TenantKeyCodec prefix t:{tenant_id}: (tenant_codec.rs:32)
- C6_MANIFEST_ATOMICITY: ManifestEntry::Replace write under write-lock prior to RAM swap (engine.rs:261)
- TEST_SUITE: 35/35 compaction tests passed in contextra-store (logs/audits/compaction-test.log)
================================================================================
```
