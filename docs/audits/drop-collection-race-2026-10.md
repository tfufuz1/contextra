# Audit-Bericht A-03: `drop_collection` Scan-Race und `collection_id`-Kollision

**Datum:** 2026-10-08
**Scope:** `crates/contextra-engine/src/contextra_impl/collections.rs`, `crates/contextra-engine/src/collection/maintenance.rs`, `crates/contextra-store/src/lsm/ops/write.rs`, `crates/contextra-crypto/src/deletion_proof.rs`
**Modul:** `contextra-engine`
**Invariante:** Doku-Wahrheit, INV-DELETION-1 (nur lesen)

---

## 1. Einleitung & Zusammenfassung

Dieser Bericht untersucht drei architekturelle Fragestellungen rund um den Löschvorgang von Collections (`drop_collection`) in `contextra-engine`:
1. **Q1:** Existenz eines Scan-Race-Zeitfensters während `drop_collection`, bei dem parallele Schreibvorgänge Schlüssel einfügen können, die nach dem `DeletionProof` weiterleben.
2. **Q2:** Kollisionsrisiko bei der Bildung der 64-Bit `collection_id` aus dem Collection-Namen mittels BLAKE3-Kürzung.
3. **Q3:** Konsistenz zwischen dem Quellcode-Kommentar in `collections.rs` und der tatsächlichen Erzeugung von `LayerCleanupProof` bezüglich des deklarierten Layers `DeletionLayer::LsmMemtable`.

---

## 2. Antworten zu den Fragestellungen

### Q1. Scan-Race in `drop_collection`

- **Ergebnis:** **BELEGT**
- **Betroffene Dateien & Zeilen:**
  - `crates/contextra-engine/src/contextra_impl/collections.rs:239–283` (`Contextra::drop_collection`)
  - `crates/contextra-store/src/lsm/ops/write.rs:189–196` (`delete_prefix`)
  - `crates/contextra-crypto/src/deletion_proof.rs:205–218` (`LayerCleanupProof::new_after_verified_empty`)
- **Begründung:**
  1. **Ablauf von `drop_collection`:**
     - **Schritt 1 (Zeilen 239–254):** Initialer Scan über `col_data_prefix` (`"__col:{name}:"`) und `txt_data_prefix` (`"__txt:{name}:"`). Alle gefundenen Schlüssel werden im Vektor `deleted_keys` gesammelt.
     - **Schritt 2 (Zeilen 256–270):** Löschung via `tenant_storage.delete_prefix(tx, ...)` und anschließendem `tenant_storage.commit(tx).await`. In `delete_prefix` (`write.rs:189`) führt der LSM-Store *erneut* einen Scan durch und erzeugt für alle zu diesem Zeitpunkt sichtbaren Schlüssel `IndexOp::Delete`-Tombstones.
     - **Schritt 3 (Zeilen 278–283):** Post-Commit-Verifikationsscan über `col_data_prefix` und `txt_data_prefix` zur Bestimmung verbleibender Live-Einträge (`remaining_col_data`, `remaining_txt_data`).
     - **Schritt 4 (Zeilen 302–326):** Aufruf von `LayerCleanupProof::new_after_verified_empty(...)` und Erzeugung des `DeletionProof`.
  2. **Zeitfenster & Rennbedingung:**
     - Zwischen dem initialen Scan (Zeile 241) bzw. dem Scan in `delete_prefix` (Zeile 260) und dem Commit der Transaktion (Zeile 270) existiert ein ungepuffertes Zeitfenster.
     - Wird während dieses Zeitfensters durch eine parallele Transaktion (z. B. `insert`/`put`) ein neuer Schlüssel in dieselbe Collection geschrieben und committet, tritt Folgendes ein:
       - Der neue Schlüssel wird nicht im initialen Vektor `deleted_keys` erfasst (fehlt somit im signierten `deleted_keys_hash`).
       - Wurde der Schlüssel nach dem `delete_prefix`-Scan eingefügt, wird für ihn kein Tombstone erzeugt.
       - Im Post-Commit-Verifikationsscan (Zeile 278) wird der Schlüssel als lebender Eintrag entdeckt (`remaining_col_data.len() > 0`).
       - `LayerCleanupProof::new_after_verified_empty` schlägt fehl und gibt `Err(ContextraError::Internal("INV-DELETION-1 violation..."))` zurück (`deletion_proof.rs:214`).
       - `drop_collection` bricht ab, obwohl bereits unvollständige Tombstones der Collection in `tx` committet wurden.
  3. **Locks:**
     - Es existiert **kein Lock**, der dieses Fenster schließt. Weder auf `tenant_collections` (Read-Guard entfällt während `drop_collection`), noch auf der Collection selbst (wie z. B. `consolidation_guard`) wird während des Scans, Löschens und Committens eine exklusive Mutation-Sperre gehalten. Das Entfernen aus der In-Memory-Map erfolgt erst in den Zeilen 328–331 *nach* der Proof-Erstellung.

---

### Q2. Kollision der abgeschnittenen `collection_id`

- **Ergebnis:** **BELEGT**
- **Betroffene Dateien & Zeilen:**
  - `crates/contextra-engine/src/contextra_impl/collections.rs:285–292` (`drop_collection`)
  - `crates/contextra-types/src/types/domain/ids.rs:25–35` (`CollectionId`)
- **Begründung:**
  1. **Bildung der ID:**
     - In `collections.rs:285–292` wird die `collection_id` wie folgt generiert:
       ```rust
       let mut hasher = blake3::Hasher::new();
       hasher.update(name.as_bytes());
       let hash_bytes = hasher.finalize();
       let col_id_u64 =
           u64::from_le_bytes(hash_bytes.as_bytes()[0..8].try_into().unwrap_or([1; 8]));
       let collection_id = CollectionId::try_new(col_id_u64).unwrap_or(CollectionId::new(1));
       ```
     - Der Collection-Name (`name.as_bytes()`, bis zu 64 Zeichen) wird mit BLAKE3 gehasht (32 Bytes / 256 Bit Ausgabelänge).
     - Die ersten 8 Bytes (`[0..8]`) werden abgeschnitten und als 64-Bit Little-Endian Unsigned Integer (`u64`) interpretiert.
     - `CollectionId::try_new` wandelt den Wert `0` in `1` um (Invariante: `CollectionId` $\neq 0$).
  2. **Kollisionsmöglichkeit:**
     - Durch das Abschneiden von 256 Bit auf 64 Bit verringert sich der Entropieraum auf $2^{64}$.
     - Nach dem Geburtstags-Paradoxon liegt die Kollisionswahrscheinlichkeit bei ca. $2^{32} \approx 4,3 \times 10^9$ Collection-Namen bei 50 %. Zwei unterschiedliche Namen (z. B. `col_A` und `col_B`) können denselben 64-Bit-BLAKE3-Präfix besitzen.
  3. **Eindeutigkeitsprüfung:**
     - Es existiert **keine Eindeutigkeitsprüfung** und kein Registry-Check gegen bestehende oder historische Collection-IDs.
     - Bei einer Kollision erzeugen zwei verschiedene Collections `name1` und `name2` dieselbe `CollectionId` im `DeletionScope::Collection { collection_id, tenant_id }`, was dazu führt, dass ein `DeletionProof` für `name1` maschinell nicht von einem Proof für `name2` anhand der `CollectionId` unterschieden werden kann.

---

### Q3. Konsistenz des Kommentars zur Layer-Abdeckung (`LsmMemtable`)

- **Ergebnis:** **BELEGT**
- **Betroffene Dateien & Zeilen:**
  - `crates/contextra-engine/src/contextra_impl/collections.rs:295–310` (`drop_collection`)
- **Begründung:**
  1. **Kommentarinhalt (Zeilen 295–301):**
     ```rust
     // LSM deletion via `delete_prefix` is tombstone-only.
     // Post-commit prefix scans verify logical key space emptiness in active MemTable / LSM state.
     // SSTable compaction and WAL truncation are asynchronous background operations; existing SSTable files
     // and WAL segments may still contain physical bytes until full compaction/truncation occurs.
     // Therefore, drop_collection claims ONLY `DeletionLayer::LsmMemtable`.
     // `SsTableAllLevels` or `WalAllSegments` may only be re-added together with a real physical verifier that inspects disk storage.
     ```
  2. **Code-Umsetzung (Zeilen 302–310):**
     ```rust
     let layer_proofs = vec![LayerCleanupProof::new_after_verified_empty(
         DeletionLayer::LsmMemtable,
         remaining_col_data.len() + remaining_txt_data.len(),
     )]
     ...
     ```
  3. **Bewertung:**
     - Der Kommentar stimmt **exakt** mit dem Code überein.
     - Der erzeugte `DeletionProof` deklariert in `covered_layers` ausschließlich den Layer `DeletionLayer::LsmMemtable`.
     - Physische Layer wie `SsTableAllLevels`, `WalAllSegments`, `HnswIndex` oder `CsrGraph` werden vom `drop_collection`-Aufruf explizit nicht beansprucht, da `delete_prefix` im LSM-Store lediglich Tombstones schreibt und keine physische Datei-Löschung oder Compact/Truncate-Operationen durchführt.

---

## 3. Vorschlag Tests (Textbeschreibung)

*Hinweis: Die folgenden Testbeschreibungen dienen der künftigen Verifikation und wurden im Rahmen dieses Auftrags nicht im Code implementiert.*

1. **Test T1: Paralleler Schreibzugriff während `drop_collection` (Scan-Race-Nachweis)**
   - **Ziel:** Nachweis des Zeitfensters und des fehlschlagenden Post-Commit-Verifikationsscans bei parallelem `insert`.
   - **Ablauf:**
     1. Erstelle eine Collection `test_race` und füge zwei Dokumente $D_1, D_2$ ein.
     2. Starte `Contextra::drop_collection("test_race", tenant_1, proof_key)`.
     3. Injiziere über einen Test-Hook / Synchronisationspunkt eine Verzögerung zwischen dem initialen Scan (`collections.rs:241`) und dem `commit` (`collections.rs:270`).
     4. Während der Verzögerung führt ein zweiter Thread `collection.insert("D_new")` aus und committet.
   - **Erwartetes Verhalten:** Der Post-Commit-Scan in `drop_collection` findet $D_{new}$. `LayerCleanupProof::new_after_verified_empty` bricht mit `ContextraError::Internal` ab (INV-DELETION-1 Verstoß), und der Proof wird verweigert.

2. **Test T2: Kollision von 64-Bit BLAKE3 Collection-IDs**
   - **Ziel:** Verifikation, dass unterschiedliche Collection-Namen mit identischem 64-Bit-BLAKE3-Präfix zur gleichen `CollectionId` im `DeletionProof` führen.
   - **Ablauf:**
     1. Ermittle zwei Zeichenketten `name_a` und `name_b`, deren BLAKE3-Hashes in den ersten 8 Bytes identisch sind.
     2. Rufe `drop_collection` für `name_a` auf und speichere `proof_a`.
     3. Rufe `drop_collection` für `name_b` auf und speichere `proof_b`.
   - **Erwartetes Verhalten:** Sowohl `proof_a.scope` als auch `proof_b.scope` enthalten dieselbe `CollectionId(u64)`, was die prinzipielle Ununterscheidbarkeit bei Kollision belegt.

3. **Test T3: Überprüfung der deklarierten DeletionLayer im `DeletionProof`**
   - **Ziel:** Verifikation, dass `drop_collection` ausschließlich `DeletionLayer::LsmMemtable` attestiert.
   - **Ablauf:**
     1. Erstelle eine Collection, füge mehrere Dokumente mit Vektoren und Texten ein.
     2. Führe `drop_collection` aus und fange den zurückgegebenen `DeletionProof` ab.
     3. Welches `proof.covered_layers` wird zurückgegeben?
   - **Erwartetes Verhalten:** `proof.covered_layers` ist eine Liste mit genau einem Element: `DeletionLayer::LsmMemtable`. Keine anderen Layer (`SsTableAllLevels`, `WalAllSegments`, `HnswIndex`, `CsrGraph`) sind enthalten.
