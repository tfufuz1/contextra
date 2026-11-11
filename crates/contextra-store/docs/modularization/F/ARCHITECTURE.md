# Architektur-Dokumentation: contextra-store Modul F (LSM-Kern)

## 1. Übersicht & Zuständigkeit
Modul F umfasst den LSM-Kern des Crates `contextra-store`:
- Engine-Fassade (`LsmStorage`, `StorageHealth`)
- Konfiguration (`LsmConfig`, `DurabilityMode`, `DurabilityConfigError`)
- Lesepfad (`ops/read.rs`, hierarchische Suche)
- Bereichs- und Präfixscans (`scan.rs`)
- Flush & LSM-seitige Compaction-Anbindung (`ops/compaction.rs`)
- Recovery & Startup-Lifecycle (`recovery/`)

## 2. Recovery-Phasen (Sequentieller Ablauf)
Beim Starten via `LsmStorage::new(config)` durchläuft die Recovery exakt folgende Schritte:

1. **Dir-Lock & Salt:**
   - Erwerb des `DirLock` auf dem Datenverzeichnis zur Verhinderung paralleler Engine-Instanzen.
   - Einlesen der `SALT`-Datei (32 Bytes). Falls Verzeichnis pristine und SALT fehlt: atomare Neugenerierung via `write_salt_atomically`.
2. **Key-Manager Initialisierung:**
   - Ableitung des Root-Keys aus Passphrase und Salt (Argon2id + HKDF).
3. **WAL-Katalogisierung:**
   - Auflisten aller `wal-*.log` und `wal.log`-Segmente, geordnet nach Zeitstempel/Sequenz.
4. **Manifest-Abgleich:**
   - Einlesen des `MANIFEST`. Rekonstruktion der gültigen SSTables (`valid_manifest_sstables`) und toten SSTables (`dead_manifest_sstables`) sowie des High-Water-Marks (`manifest_hwm`).
5. **WAL-Replay in MemTable:**
   - Sequentielles Einlesen der WAL-Einträge.
   - Puts und Deletes (Tombstones mit gesetztem Bit 63) werden in die `MemTable` übertragen.
   - Uncommittete oder abgebrochene Transaktionen werden beim Segmentwechsel verworfen (`orphan_ops`).
   - Verifikation der HMAC-Kette gegen den High-Water-Mark des Manifests (`verify_wal_chain_completeness`).
6. **Bereinigung temporärer Dateien:**
   - Entfernen unvollständiger `.tmp`-, `SALT.tmp.*`- und `MANIFEST.new.*`-Dateien.
   - Bereinigen toter SSTables, die im Manifest als ersetzt/gelöscht markiert sind.
7. **SSTable-Öffnung & Verknüpfung:**
   - Öffnen der validen SSTables mit dem sharded `BlockCache`.
   - Ermittlung von `max_seq` und `max_tx` über alle SSTables und MemTable.
8. **Subsystem-Initialisierung:**
   - Starten des `SystemPressureMonitor`-Hintergrundtasks.
   - Starten der `CompactionEngine`-Hintergrundschleife mit Überwachung via `TaskTracker`.
9. **Startup-Flush:**
   - Wurden beim Replay signifikante Datenmengen rekonstruiert, erzwingt die Engine einen initialen Flush zur Manifest-Sicherung, bevor alte WAL-Segmente gelöscht werden.
10. **Rollback-Intent-Recovery:**
    - Auffinden unvollständiger `rollback-*.intent`-Dateien und Ausführung des Rollbacks auf die Zieltransaktion (`rollback_to_tx`).

## 3. Lesepfad-Hierarchie (Hierarchical Lookup)
Bei `get(key)` bzw. `get_at_seq(key, seq_no)` gilt die strikte Schichtenreihenfolge:
1. **Aktive MemTable:** Sucht in den sharded BTreeMaps im RAM.
2. **Immutable MemTables:** Durchsucht unveränderliche MemTables von neu nach alt.
3. **SSTables (L0..LN):** Durchsucht geöffnete SSTables von der neuesten zur ältesten.
   - Bloom-Filter-Prüfung (`may_contain`) zur Vermeidung unnötiger Plattenzugriffe.
   - Bereichsprüfung `[first_key, last_key]`.
   - Indexblock-Suche und Datenblock-Decodierung via `BlockCache`.
   - Bei Auffinden eines Tombstones (`TOMBSTONE_BIT`) wird die Suche sofort mit `None` beendet (Shadowing).

## 4. Strukturvertrag E ↔ F
- **Gemeinsame Felder:** Alle Felder von `LsmStorage` sind in `src/lsm/engine.rs` als `pub(super)` mit unveränderten Namen und Typen definiert, sodass der Schreibpfad (Agent E) darauf zugreifen kann.
- **Intent-Locks:** `intent_locks` verbleibt als `Mutex<HashMap<Vec<u8>, TxId>>`, um den Strukturvertrag mit E einzuhalten.
- **Lock-Hierarchie:** Strikte Beachtung der Rangfolge aus `lock_order.rs` (Intent -> commit_mutex -> flush_mutex -> state -> truncate_lock -> sstables -> MemTable-Shards).
