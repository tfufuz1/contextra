# Contextra Store — Audit-Bericht: `rank` in Manifest-Einträgen

**Datum:** 2026-10-12
**Autor:** Storage-Architekt
**Komponente:** `contextra-store` (`manifest`, `lsm::recovery`, `compaction`)
**Status:** Doku-Korrektur / Auditspezifikation (Befund externer Audit)

---

## 1. Übersicht & Zusammenfassung

Dieser Bericht beschreibt die exakte Bedeutung, Historie und Berechnungslogik des Feldes **`rank`** in den Manifest-Einträgen (`ManifestEntry::Replace`) des Storage-Engines `contextra-store`.

Zusammenfassend gilt:
1. **Festgelegter Disk-Wert (`0u64`):** Das in der binären Nutzlast von `ManifestEntry::Replace` gespeicherte Feld `rank` ist im Schreibpfad **abgekündigt (Formatabkündigung S-01 / T-04)** und wird im Serialisierungspfad stets als `0u64` geschrieben, um binäre Layoutkompatibilität zu wahren.
2. **Rekonstruktions-Zähler (`next_rank`):** Bei der Manifest-Rekonstruktion im Arbeitsspeicher (`reconstruct_valid_sstables`) bezeichnet `rank` einen internen monotonen Index, der die Einfüge- und Ersetzungsreihenfolge der aktiven SSTables während der Replay-Phase abbildet.
3. **Recovery-Verwerfung:** Bei der LSM-Recovery in `recovery.rs` wird das Tupel-Element `_rank` explizit verworfen. Die Sichtbarkeit und Versionierung von Schlüssel-Wert-Paaren im LSM-Tree erfolgt ausschließlich über die LSM-Sequenznummern (`max_seq` & `!TOMBSTONE_BIT`).

---

## 2. Detaillierte Code-Fundstellen & Semantik

### 2.1 Formatabkündigung & Schreibpfad (`entry.rs`)
* **Fundstelle:** `crates/contextra-store/src/manifest/entry.rs:6-7` sowie `entry.rs:80`
* **Code-Verhalten:**
  In `ManifestEntry::Replace { sstables: Vec<SstableMeta>, rank: u64 }` wird beim Schreiben das Feld `rank` im Payload mit `0u64` belegt:
  ```rust
  // FORMATABKÜNDIGUNG (S-01 / T-04): Der Schreibpfad schreibt kein rank mehr in Replace-Einträge (schreibt stets 0u64 für binäre Nutzlastkompatibilität).
  payload.extend_from_slice(&0u64.to_le_bytes()); // Abgekündigtes rank-Feld (S-01): stets 0u64
  ```
* **Lesepfad (`entry.rs:266-271`):**
  Der Deserialisierer liest die 8 Bytes für `rank` weiterhin aus dem Byte-Stream, um Kompatibilität mit historischen Manifest-Dateien zu gewährleisten. Der eingelesene Wert wird jedoch im Laufzeitsystem nicht für Sortier- oder Sichtbarkeitsentscheidungen herangezogen.

### 2.2 Monotoner Rekonstruktionsindex (`core.rs`)
* **Fundstelle:** `crates/contextra-store/src/manifest/core.rs:6-7` sowie `core.rs:358-408`
* **Code-Verhalten:**
  In `ManifestCore::reconstruct_valid_sstables` verwaltet der Algorithmus die Gültigkeit von SSTables nach `Add`- und `Replace`-Aktionen über einen Zähler `next_rank`:
  ```rust
  // `next_rank` führt eine reine Monotonie-über-die-Add-Historie und ist NICHT identisch mit dem
  // abgekündigten `rank`-Feld in `ManifestEntry::Replace`.
  let mut next_rank = 0u64;
  ...
  valid_map.insert(key, next_rank);
  next_rank += 1;
  ...
  sorted.sort_by_key(|(_, rank)| *rank);
  ```
* **Bedeutung:**
  `next_rank` dient in dieser In-Memory-Funktion einzig dazu, die Rekonstruktionsreihenfolge (Reihenfolge des Hinzufügens) der gültigen SSTable-Pfade in einer sortierten Liste `Vec<(PathBuf, u64)>` stabil zu halten.

### 2.3 Verwerfung bei der Recovery (`recovery.rs`)
* **Fundstelle:** `crates/contextra-store/src/lsm/recovery.rs:252`
* **Code-Verhalten:**
  Wenn die LSM-Engine beim Systemstart die aktiven SSTables lädt, verwirft sie den zweiten Wert des Tupels (`_rank`):
  ```rust
  manifest
      .reconstruct_valid_sstables()
      .into_iter()
      .map(|(path, _rank)| path)
  ```
* **Bedeutung:**
  Der Rekonstruktions-Rank hat keinen Einfluss auf die spätere Lese-Sichtbarkeit.

### 2.4 Compaction-Engine (`compaction/engine.rs`)
* **Fundstelle:** `crates/contextra-store/src/compaction/engine.rs:298`
* **Code-Verhalten:**
  Beim Erzeugen eines neuen `ManifestEntry::Replace` nach erfolgreicher Compaction übergibt die Compaction-Engine explizit `rank: 0`:
  ```rust
  ManifestEntry::Replace {
      sstables: merged_metas,
      rank: 0,
  }
  ```

---

## 3. Fazit & Dokumentationskorrektur

Die Untersuchung des Quellcodes bestätigt:
- Das Disk-Feld `rank` in Manifest-Dateien ist ein **Legacy-Artefakt (S-01/T-04)** und hat keinen Einfluss auf die Korrektheit oder Sichtbarkeit der Daten.
- Der interne Zähler `next_rank` bei der Manifest-Rekonstruktion dient rein der Stabilisierung der Manifest-Eintragsabfolge im Arbeitsspeicher.
- Lese-Sichtbarkeit und Versionierung von SSTables im LSM-Tree werden in `contextra-store` strikt und ausschließlich über die Sequenznummern (`max_seq` & `seq_no`) geregelt.
