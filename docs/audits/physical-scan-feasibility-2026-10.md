# Audit-Bericht: Machbarkeit physischer Emptiness-Scanner in `contextra-store`

**Datum:** 2026-10-08
**Task:** T-2026-0236
**Crate:** `contextra-store` (Ring 1)
**Scope:** `docs/audits/physical-scan-feasibility-2026-10.md`, `.jules/tasks/T-2026-0236.toml`
**Status:** Belegt / Abgeschlossen (Analyse ohne Codeänderung)

---

## 1. Einleitung & Zielstellung

Dieser Audit-Bericht untersucht die Machbarkeit, Voraussetzungen und Grenzen eines physischen Emptiness-Scanners im Crate `contextra-store`. Der Scanner soll beweisen können, dass für einen bestimmten Löschscope (z. B. Key-Präfix `__col:<name>:`) keine lesbaren Daten mehr in MemTable(s), SSTables und WAL-Segmenten vorliegen.

---

## 2. Antworten auf die Fragen (Q1–Q7)

### Q1. Symbol-Inventur

- **(a) Accessor für live SSTable-Dateien laut Manifest:**
  - **Ergebnis:** Existiert **nicht** als öffentliche produktive API auf `LsmStorage`.
  - **Datei:Zeile:**
    - `crates/contextra-store/src/manifest/core.rs:277`: `Manifest::reconstruct_valid_sstables(entries: &[ManifestEntry]) -> Vec<(PathBuf, u64)>` (`pub`)
    - `crates/contextra-store/src/lsm/engine.rs:37`: `LsmStorage.sstables` (`pub(super) Arc<RwLock<Vec<Arc<SstableReader>>>>`)
    - `crates/contextra-store/src/lsm/engine.rs:290`: `LsmStorage::sstables_for_test(&self)` (`#[doc(hidden)] pub fn`)
  - **Status:** Symbol `contextra_store::manifest::core::Manifest::reconstruct_valid_sstables` verifiziert (PASS).

- **(b) Reader-API für Präfix-/Key-Existenz inkl. Tombstone-Erkennung auf SSTable-Ebene:**
  - **Ergebnis:** **Existiert**.
  - **Datei:Zeile:**
    - `crates/contextra-store/src/sstable/reader.rs:207`: `pub struct SstableReader`
    - `crates/contextra-store/src/sstable/reader.rs:567`: `SstableReader::get_at(&self, key: &[u8], max_seq: u64, max_tx: u64) -> Result<Option<(Bytes, u64, u64)>>`. Das zweite Tupel-Element ist `seq_no`. Wenn `(seq_no & TOMBSTONE_BIT) != 0` gilt (`crates/contextra-store/src/sstable/reader.rs:659`), ist der Eintrag ein Tombstone.
    - `crates/contextra-store/src/sstable/reader_ext.rs:8`: `SstableReader::scan_prefix(&self, prefix: &[u8]) -> Result<Vec<(Bytes, Bytes, u64, u64)>>`. Enthaltene Tombstones tragen Bit 63 (`TOMBSTONE_BIT`) in `seq_no`.
  - **Status:** Symbole `SstableReader`, `SstableReader::get_at`, `SstableReader::scan_prefix` verifiziert (PASS).

- **(c) Zugriff auf Immutable-Memtables (`lsm/mod.rs`, `lsm/ops/*`):**
  - **Ergebnis:** Existiert **nur modul-intern** (`pub(super)`).
  - **Datei:Zeile:**
    - `crates/contextra-store/src/lsm/guard.rs:12`: `LsmState.immutable_memtables` (`pub(super) Vec<Arc<MemTable>>`)
    - `crates/contextra-store/src/lsm/engine.rs:35`: `LsmStorage.state` (`pub(super) RwLock<LsmState>`)
    - `crates/contextra-store/src/lsm/ops/read.rs:104` & `crates/contextra-store/src/lsm/scan.rs:196`: Interner Zugriff via `state.read()`.
  - **Status:** Kein öffentlicher Accessor für externe Aufrufer vorhanden.

- **(d) Full-Compaction für einen Key-Bereich anzustoßen und auf Abschluss zu warten:**
  - **Ergebnis:** **Existiert nicht**.
  - **Datei:Zeile:**
    - `crates/contextra-store/src/compaction/engine.rs:125`: `CompactionEngine::maybe_compact(&self, ...)` (`pub async fn`) führt automatisch selektierte Size-Tiered/Adaptive Compactions ganzer SSTables aus.
    - `crates/contextra-store/src/compaction/engine.rs:403`: `CompactionEngine::merge_sstables(&self, ...)` (`pub async fn`) führt N-Way-Merges über eine gegebene Liste ganzer SSTables aus.
  - **Status:** Symbole `CompactionEngine::maybe_compact` und `CompactionEngine::merge_sstables` verifiziert (PASS). Es gibt keine API für zielgerichtete Präfix-/Bereichs-Full-Compaction mit Warten auf Abschluss.

---

### Q2. MVCC & Compaction Retention

- **Bleiben überschattete Put-Versionen physisch in SSTables bis zur Compaction?**
  - **Ergebnis:** **BELEGT (Ja)**.
  - **Datei:Zeile:** `crates/contextra-store/src/compaction/engine.rs:657–680` (`merge_sstables_inner`).
  - **Begründung:** Im LSM-Tree schreibt `delete_prefix` lediglich Tombstones (`IndexOp::Delete`). Überschattete `Put`-Einträge in älteren SSTables bleiben physisch unverändert auf der Festplatte erhalten, bis ein Compaction-Merge (`merge_sstables_inner`) die betroffenen SSTables erfasst und gefaltet hat.

- **Wie bestimmt sich `min_snapshot_seq` und kann ein offener Snapshot Versionen pinnen?**
  - **Ergebnis:** **BELEGT**.
  - **Datei:Zeile:**
    - `crates/contextra-store/src/compaction/engine.rs:116`: `CompactionEngine::min_snapshot_seq_bound(&self) -> u64` ruft `self.floor.floor()` auf.
    - `crates/contextra-mvcc/src/lease.rs:18` & `snapshot.rs:88`: Gemäß MVCC-Invariante I-2 berechnet sich der Floor aus `min(snapshot_registry.min_active_seqno(), tx_buffer.min_read_snapshot())`.
    - `crates/contextra-store/src/compaction/engine.rs:660` & `:764`: In `merge_sstables_inner` werden Versionen/Tombstones nur verworfen oder gefaltet, wenn `raw_seq <= min_snapshot_seq` gilt.
  - **Begründung:** Ein nicht freigegebener `SnapshotLease` hält `min_active_seqno()` dauerhaft auf einem alten Wert fest. Dadurch werden alle älteren Put-Versionen und Tombstones physisch in den SSTables gepinnt und dürfen von der Compaction nicht GC'd werden.

- **Welche Aussage über „leer" ist daher überhaupt möglich?**
  - **Ergebnis:** **BELEGT**.
  - **Aussage:** Ein reiner Read-Scan oder SSTable-Lookthrough belegt ausschließlich **logische Unsichtbarkeit**, jedoch **keine physische Emptiness** (Abwesenheit von Bytes auf Disk). Physische Emptiness ist erst garantiert, wenn:
    1. Alle active und immutable MemTables auf Disk geflusht wurden.
    2. Der MVCC-Floor `min_snapshot_seq` über die Sequenznummer aller Löschungen im Scope gestiegen ist (keine offenen Snapshots).
    3. Eine **FULL Compaction** über alle betroffenen SSTables ausgeführt wurde.
    4. Die alten, compagierten SSTable-Dateien vom Dateisystem gelöscht wurden.

---

### Q3. Verschlüsselung

- **Sind SSTable-Blöcke/WAL-Segmente verschlüsselt (`wal::KeyManager`)?**
  - **Ergebnis:** **BELEGT (Ja)**.
  - **Datei:Zeile:**
    - `crates/contextra-crypto/src/crypto/key_manager.rs:25` (re-exported in `crates/contextra-store/src/wal/mod.rs:270`): `pub use contextra_crypto::crypto::KeyManager;`
    - `crates/contextra-store/src/sstable/reader.rs:446`: `read_block_at_file` nutzt `km.decrypt_auto_nonce(...)` für SSTable-Blöcke.
    - `crates/contextra-store/src/wal/replay.rs:490` & `io.rs:379`: WAL-Frames werden mit AEAD entschlüsselt.

- **Muss ein Scanner Blöcke entschlüsseln?**
  - **Ergebnis:** **BELEGT (Ja)**.
  - **Begründung:** Auf BytE-Ebene sind verschlüsselte Blöcke ununtercheidbar von kryptografischem Rauschen. Ein Emptiness-Scanner muss SSTable-Blöcke und WAL-Frames mit dem passenden `KeyManager` entschlüsseln, um Indizes und Keys auf Scope-Zugehörigkeit zu prüfen.

- **Welche Konsequenz hat das für die Aussage „kein lesbarer Klartext"?**
  - **Ergebnis:** **BELEGT**.
  - **Begründung:** Liegt `KeyManager`-Verschlüsselung vor, befinden sich ohne Key-Zugriff ohnehin keine Klartext-Bytes auf der Disk. Ein externer Scanner ohne Schlüssel sieht jedoch nur Chiffretext und kann nicht feststellen, welche Daten darin stecken. Erst ein autorisierter Scanner mit `KeyManager`, der alle geflushten Blöcke entschlüsselt und 0 Treffer für den Scope vorfindet, kann belegen: „Keine lesbaren Klartext-Daten des Scopes auf Disk".

---

### Q4. Nebenläufigkeit (TOCTOU)

- **Welche Sperren/Epochen existieren, um einen konsistenten Scan zu garantieren?**
  - **Ergebnis:** **BELEGT**.
  - **Datei:Zeile:**
    - `crates/contextra-store/src/lsm/engine.rs:37`: `LsmStorage.sstables` (`Arc<RwLock<Vec<Arc<SstableReader>>>>`)
    - `crates/contextra-store/src/lsm/ops/read.rs:101` & `crates/contextra-store/src/lsm/scan.rs:197`: Reader halten `sstables.read()`.
    - `crates/contextra-store/src/sstable/reader.rs:208`: `SstableReader.file` ist `Arc<std::fs::File>`.
    - `crates/contextra-store/src/compaction/engine.rs:242` & `:275`: Compaction tauscht die SSTable-Liste unter `sstables.write()` atomar aus und löscht veraltete Dateien von Disk.
  - **TOCTOU-Analyse:**
    - **In-Memory Scan (über `LsmStorage`):** Das Halten von `Arc<SstableReader>` wirkt als Referenzzähl-Anker. Der geöffnete File-Descriptor (`Arc<std::fs::File>`) bleibt auch dann gültig, wenn Compaction die Datei auf der Festplatte via `remove_file` löscht.
    - **Externer Offline-Scan (über Disk-MANIFEST):** Ein externer Scanner, der zuerst `MANIFEST` von Disk liest (`crates/contextra-store/src/manifest/core.rs:32`) und anschließend versucht, die ermittelten `.sst`-Pfade separat zu öffnen, unterliegt einer TOCTOU-Rennbedingung. Parallel laufende Compactions können Dateien löschen, bevor der Scanner sie öffnen kann. Außerhalb von `LsmStorage` existiert **keine** Dateisystem-Sperre oder Epochen-Anker.

---

### Q5. Kosten

- **Abschätzung des Scanaufwands:**
  - **Punkt-Lookups (exakter Schlüssel):** $O(N_{\text{sst}})$ Bloom-Filter-Prüfungen. Bei Treffer $O(\log B)$ Binärsuche im Index + $O(1)$ Block-Entschlüsselung.
  - **Präfix-Scan (z. B. `__col:<name>:`):** Whole-SSTable Bloom-Filter (`crates/contextra-store/src/sstable/bloom.rs:10`) indizieren ausschließlich exakte Keys (`BloomFilter::may_contain`). Sie unterstützen **keine** Präfix-Abfragen. Daher muss bei einem Präfix-Scan für jede SSTable der Index durchsucht ($O(\log B)$) und jeder potentiell passende Block entschlüsselt werden. Aufwand: $O(\sum_{\text{sst}} \text{Blocks im Präfix-Bereich})$.

- **Rolle von Bloom-Filtern:**
  - **Datei:Zeile:** `crates/contextra-store/src/sstable/reader.rs:570` & `:596`
  - **Verhalten:**
    - `may_contain == false`: Key garantiert nicht vorhanden (0 % False Negatives).
    - `may_contain == true`: Key eventuell vorhanden (False Positive möglich).
  - **Auswirkung:** Ein False Positive veranlasst den Scanner dazu, einen Block unnötigerweise zu entschlüsseln und zu durchsuchen ("falscher Fehlschlag" beim Filter-Skip = verschmerzbarer Zusatzaufwand). Findet der darauffolgende Block-Scan keine Einträge, bestätigt der Scanner korrekt die Abwesenheit. Bloom-Filter verursachen somit **niemals einen falschen Erfolg** (fälschliches Attestieren von Datenfreiheit).

---

### Q6. WAL (Write-Ahead Log)

- **Wann werden Segmente entfernt?**
  - **Ergebnis:** **BELEGT**.
  - **Datei:Zeile:** `crates/contextra-store/src/lsm/ops/compaction.rs:188–212` (`flush`).
  - **Ablauf:** Flushed MemTables übertragen ihre Daten in eine neue SSTable. Nach dem Schreiben von `ManifestEntry::WalCheckpoint { hmac }` in die `MANIFEST`-Datei werden alle gesiegelten WAL-Dateien in `pending_sealed_wals` via `tokio::fs::remove_file` gelöscht.

- **Wie lässt sich „WAL enthält keine Daten des Scopes" belegen, ohne jedes Segment zu entschlüsseln?**
  - **Ergebnis:** **BELEGT**.
  - **Begründung:** Für ungesiegelte/aktive WAL-Segmente gar nicht, da WAL-V3-Frames (`crates/contextra-store/src/wal/encode.rs`) aus AEAD-verschlüsseltem Payload `(key, value, seq, tx)` bestehen und keinen unverschlüsselten Präfix-Index oder Bloom-Filter besitzen.
  - **Indirekter Nachweis:** Wenn (1) ein synchroner Flush erzwungen wurde, der alle alten WAL-Segmente bis zum `WalCheckpoint` löscht, UND (2) die im verbleibenden aktiven WAL-Segment enthaltenen Transaktionen/Sequenznummern nachweislich strikt höher sind als die Sequenznummer des Löschvorgangs.

- **Rolle der Manifest-HWM-Gegenprüfung:**
  - **Datei:Zeile:** `docs/audits/wal-v1-tail-2026-10.md:Q2` & `crates/contextra-store/src/lsm/recovery.rs:360` (`verify_wal_chain_completeness`).
  - **Begründung:** Die Manifest High-Water-Mark (`manifest_hwm`) stellt sicher, dass WAL-Replay nach einem Crash exakt am verankerten Stand stoppt. Truncations oder abgeschnittene/manipulierte WAL-Tails werden erkannt, sodass keine gelöschten Einträge (Resurrection) wiederhergestellt werden.

---

### Q7. Schlussfolgerungen & API-Erweiterungen

#### 7.1 Klassifikation der Layer

1. **(a) Sofort scanbar:**
   - Active MemTable & Immutable MemTables (über `state.read()` intern in `contextra-store`).
2. **(b) Erst nach Full-Compaction scanbar:**
   - SSTable-Layer (`SstableReader`). Vor einer Full-Compaction enthalten ältere SSTable-Dateien physisch überschattete `Put`-Versionen und Tombstones. Erst nach einer Full-Compaction mit `min_snapshot_seq > deletion_seq` sind veraltete Versionen physisch bereinigt.
3. **(c) Derzeit nicht seriös scanbar:**
   - Active/ungesiegelte WAL-Segmente (kein Index/Bloom-Filter; erfordert Voll-Scan aller Frames mit `KeyManager`).
   - Physische SSTable-Dateien ohne geladenen `KeyManager` (verschlüsselt).
   - Externe Offline-Disk-Scans von `.sst`-Dateien wegen TOCTOU-Rennbedingungen ohne In-Memory `Arc<SstableReader>`-Anker.

#### 7.2 Nötige API-Erweiterungen in `contextra-store` (Auflistung)

- **MemTable-Layer:**
  - `LsmStorage::has_prefix_in_memtables(&self, prefix: &[u8]) -> bool`: Prüft active und immutable MemTables synchron auf Vorhandensein von Schlüsseln mit dem Präfix.
- **SSTable-Layer:**
  - `LsmStorage::live_sstables(&self) -> Vec<Arc<SstableReader>>`: Öffentlicher Accessor für die aktuell im Speicher geankerten live SSTable-Reader (vermeidet TOCTOU-Races).
  - `LsmStorage::force_full_compaction(&self) -> Result<()>` / `CompactionEngine::compact_range(...)`: Löst eine synchrone Full-Compaction für alle SSTables aus und wartet auf deren Abschluss.
  - `LsmStorage::verify_physical_emptiness_in_sstables(&self, prefix: &[u8]) -> Result<bool>`: Iteriert über alle Live-SSTables unter `sstables.read()` und prüft via `SstableReader::scan_prefix`, ob physisch noch Einträge oder Tombstones des Präfix existieren.
- **WAL-Layer:**
  - `LsmStorage::flush_and_truncate_wal(&self) -> Result<()>`: Erzwingt einen synchronen Flush aller MemTables und löscht gesiegelte WAL-Segmente bis zum aktuellen Checkpoint.
  - `LsmStorage::verify_emptiness_in_active_wal(&self, prefix: &[u8]) -> Result<bool>`: Liest das verbleibende ungesiegelte WAL-Segment und prüft, ob nach der Deletion-Sequence noch Einträge des Scopes vorliegen.

---

## 3. Symbol-Verifikations-Protokoll

Alle im Bericht genannten Symbole wurden erfolgreich mit `cargo xtask symbol-exists` verifiziert:
- `contextra_store::manifest::core::Manifest::reconstruct_valid_sstables` (PASS)
- `contextra_store::sstable::reader::SstableReader` (PASS)
- `contextra_store::sstable::reader::SstableReader::get_at` (PASS)
- `contextra_store::sstable::reader_ext::SstableReader::scan_prefix` (PASS)
- `contextra_store::compaction::engine::CompactionEngine::maybe_compact` (PASS)
- `contextra_store::compaction::engine::CompactionEngine::merge_sstables` (PASS)
- `contextra_store::lsm::engine::LsmStorage` (PASS)
- `contextra_store::wal::Wal` (PASS)
