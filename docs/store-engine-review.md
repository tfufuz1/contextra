# contextra-store – Storage-Engine Review

> **Datum:** 2026-11-02
> **Rolle:** Storage-Engine-Ingenieur (RocksDB/Pebble/LevelDB-Internals, redb als Referenz)
> **Scope:** systematische Analyse aller Implementierungsebenen in `crates/contextra-store`
> **Normative Referenz:** SPEC 

---

## Überblick & Bewertungsskala

| Stufe | Bedeutung |
|---|---|
| **KRITISCH** | Datenverlust, Korrektheitslücke oder SPEC-Verstoß möglich |
| **HOCH** | Sicherheitsrelevant oder signifikante Operationsgefahr |
| **MITTEL** | Zuverlässigkeits- oder Leistungsproblem, kein unmittelbarer Datenverlust |
| **NIEDRIG / OPTIMIERUNG** | Tech-Schuld, verbesserungswürdige Codequaliät |

---

## 1. WAL (Write-Ahead Log)

### 1.1 HMAC-Kettenvalidierung im Mmap-Replay-Pfad (KRITISCH)

**Befund:** Die Funktion `do_scan_entries_with_callback` in `src/wal/io.rs` initialisiert einen `IntegrityVerifier` korrekt und führt die HMAC-Kette fort – **jedoch nur im Stream-Replay-Pfad.** Der primäre Replay-Pfad in `replay_mmap_with_sink` liest die gesamte WAL mit `tokio::fs::read()` und leitet die Bytes an `parse_mmap_slice_with_sink` weiter.

Im `parse_mmap_slice_with_sink`-Pfad werden **alle** Entries deserialisiert, aber die HMAC-Kettenkonsistenz wird nicht für Einträge inmitten der Datei verifiziert – die Verifikation geschieht ausschließlich auf Entry-Ebene (CRC im Frame). Werden einzelne Einträge ausgeschnitten (Splice-Angriff oder unvollständig erkannte torn-write-Korruption) und durch valide CRC-Bytes ersetzt, kann ein Angreifer mit Schreibzugriff auf Disk-Images valide Entries mit korrekten CRCs platzieren.

**Implikation:** Die HMAC-Kette in `WalEntry` enthält `prev_hmac`, das bei jedem neuen Eintrag die Unversehrtheit der Kette sicherstellt. Es ist unklar, ob der Mmap-Pfad diese Kette beim Replay tatsächlich konsistent validiert. **Wenn nicht, ist der HMAC-Binding-Schutz (SPEC Kapitel 10) im schnellen Mmap-Pfad wirkungslos.**

**Empfehlung:** Kettenvalidierung im `parse_mmap_slice_with_sink`-Pfad mit identischer Logik wie `IntegrityVerifier` sicherstellen. Test: `T-WAL-CHAIN-MMAP`.

---

### 1.2 Torn-Write-Erkennung: Grenzfall am Chunk-Ende (HOCH)

**Befund:** `do_scan_entries_with_callback` erkennt torn writes am Dateiende korrekt (Abbruch bei `UnexpectedEof`). Ein torn write, der zufällig einen gültigen Frame-Length-Wert erzeugt, wird akzeptiert, wenn die Prüfsumme des Payload zufällig stimmt.

**Konkret:** Die Recovery-Policy (SPEC §8.8) erlaubt Toleranz am Dateiende – aber nur dort. Eine AEAD-Entschlüsselungslücke direkt an einer Chunk-Grenze (`pos >= file_size` Check in `replay.rs`) könnte einen partiellen Ciphertext als Tail-Truncation akzeptieren, obwohl er möglicherweise Nutzlast aus zwei verschiedenen Crash-Phasen enthält.

**Empfehlung:** Nach AEAD-Entschlüsselung eines Frames sollte jede innere Entry mit `WalEntry::from_bytes_classified` fail-closed reagieren (bereits implementiert), aber zusätzlich sollte die minimale verbleibende Payload-Größe (nonce + tag ≥ 28 Bytes) vor dem Decrypt-Versuch geprüft werden.

---

### 1.3 Keine Vfs-Konformität im WAL (HOCH – D-04)

**Befund:** Der WAL verwendet ausschließlich `tokio::fs::*` direkt (durch den lokalen `crate::wal::fs`-Wrapper) statt des Vfs-Ports. In `src/lsm/recovery.rs` werden `tokio::fs::read_dir`, `tokio::fs::read`, `tokio::fs::write`, `tokio::fs::rename` und `std::fs::File::open` direkt aufgerufen. Das ist ein **SPEC-Verstoß (D-04)**: I/O muss ausschließlich über den Port `Vfs` stattfinden.

**Kritisch:** `std::fs::File::open` wird in `recovery.rs` Zeile ~718 für den Parent-Dir-Fsync-Fallback verwendet – das ist ein unbedingter `std::fs`-Aufruf außerhalb von `contextra-durable-fs`.

```rust
// recovery.rs ~718
std::fs::File::open(&parent) // ← verbotener std::fs-Zugriff
```

**Empfehlung:** Alle I/O-Operationen (inkl. `read_dir`, Metadata-Abfragen, `rename`) auf den Vfs-Port umstellen. `lsm/recovery.rs` ist die größte Baustelle.

---

### 1.4 WAL-Segment-Naming: flush_id nicht identisch mit Commit-Sequenz (MITTEL)

**Befund:** WAL-Segmente werden als `wal-{flush_id:020}.log` benannt, wobei `flush_id` ein `AtomicU64` ist. Bei Recovery werden Dateinamen nach dem numerischen Sequenz-Komponent (`seq_component`) geordnet. Die `recovery.rs` prüft explizit auf `u128`-Overflow – das ist robust.

**Problem:** Die Beziehung zwischen `flush_id` und Commit-Sequenz ist nicht garantiert monoton. Ein neues WAL-Segment wird vor dem `commit_mutex` allokiert (`Phase 1`), also könnte ein späteres WAL-Segment einen niedrigeren WAL-Inhalt enthalten, wenn der Flush durch einen Rollback konkurriert.

**Empfehlung:** Sicherstellen, dass die Reihenfolge der WAL-Segments bei Recovery durch ihre HMAC-Kette (nicht Dateiname) verifiziert wird. Crash-Matrix-Test: Flush-ID-Sprung nach Rollback.

---

### 1.5 fsync-Fehler-Wiederholung auf gleichem Handle (HOCH – D-55)

**Befund:** Der WAL-Flusher-Actor in `src/wal/flusher.rs` sendet `sync_all()` nach jedem Append-Batch. Bei `sync_all()`-Fehler wird `poisoned` gesetzt. Es gibt jedoch keinen expliziten Schutz gegen eine zweite `sync_all()` auf demselben Handle (was nach D-55 verboten ist). Der `recover_from_poison()`-Pfad in `group_commit.rs` ruft potenziell `wal.recover_from_poison()` auf, was intern wieder `sync_all()` aufrufen könnte.

**Empfehlung:** Sicherstellen, dass `recover_from_poison` niemals `sync_all()` auf dem vergifteten Handle aufruft. Neues Handle öffnen oder Store als fatal markieren.

---

### 1.6 WAL Loom-Mock: Busyloop (NIEDRIG / CI-Gefahr)

**Befund:** Der Loom-Mock in `src/wal/mod.rs` enthält einen expliziten Busyloop ohne Sleep als letzten Fallback für `permissions()`:

```rust
loop {
    if let Ok(m) = std::fs::metadata(".") {
        return m.permissions();
    }
}
```

Das kann CI-Hänger verursachen und setzt `std::fs` im Loom-Kontext voraus.

**Empfehlung:** Loom-Mock mit `std::fs::Permissions::from_mode(0o600)` oder einem fixen Default initialisieren.

---

## 2. MemTable

### 2.1 Doppelte Größen-Buchhaltung (MITTEL)

**Befund:** `MemTable::size()` summiert alle Versionen aller Keys (inkl. älterer MVCC-Versionen). Das bedeutet, dass `memtable_size_limit` tatsächlich mehr Daten im Speicher toleriert als nominell konfiguriert, weil `flush` nur die *neueste* Version pro Key in den SSTable schreibt. Bei MVCC-intensiven Workloads (viele Updates desselben Keys) können die immutable MemTables mehrfach die konfigurierte Größe überschreiten, bevor Flush ausgelöst wird.

**Empfehlung:** Entweder `size()` auf unique-key-Basis berechnen (ohne Überversionen) oder `memtable_size_limit` mit einem MVCC-Faktor-Korridor dokumentieren.

---

### 2.2 `let _ =` auf AtomicUsize in rollback (HOCH – P6)

**Befund:** `src/memtable.rs` Zeile 412:
```rust
let _ = self.size.fetch_sub(...)
```
Das diskardiert das Ergebnis einer `fetch_sub`-Operation. Im `rollback`-Pfad wird `size` um alle Versionen des entfernten Keys korrigiert – ein Fehler in der Berechnung des zu subtrahierenden Betrags führt zu dauerhaft falschem Tracking ohne Warnung.

**Empfehlung:** Saturating subtraction absichern oder die zu subtrahierende Größe vorab validieren. `let _` auf Result-ähnlichen Typen ist P6-widrig.

---

### 2.3 Shard-Bereichs-Fehler bei System-Key-Scans (NIEDRIG / Optimierung)

**Befund:** `shard_range_for_bounds` klassifiziert eine Range als „system" (alle Shards), wenn `start <= b"__" && end >= b"__"`. Für einen Scan von `b"__col:"` bis `b"__col;\xff"` werden alle 16 Shards gelesen, obwohl system-Keys hash-gesharded sind.

**Optimierungspotenzial:** Für spezifische, bekannte System-Prefixes (`__col:`, `__meta:`, etc.) könnte ein Cache-Table (prefix → shard-set) den Scan auf die tatsächlich genutzten Shards reduzieren.

---

## 3. SSTable-Format & Compaction

### 3.1 Dir-Fsync nach alten SST-Löschungen fehlt (HOCH)

**Befund:** In `compaction/engine.rs`, Step 7, werden alte SSTable-Dateien nach dem in-memory-Swap gelöscht:

```rust
// Compaction complete — delete old SSTables
for path in &old_paths {
    if let Err(e) = tokio::fs::remove_file(path).await { ... }
    // uuid_sidecar remove...
}
// ← KEIN fsync_parent_dir nach alten SST-Löschungen!
```

Auf Linux garantiert `remove_file` (unlink) nicht, dass die Verzeichniseinträge nach einem Crash persistiert sind. Nach einem Crash könnten die alten SSTable-Dateien wieder erscheinen. Da der Manifest-Rollover bereits vorher durchgeführt wurde, ist ein Szenario möglich, in dem der Manifest eine `Replace`-Operation kennt, aber beide Dateien (alt + neu) im Verzeichnis erscheinen.

**SPEC-Referenz:** SPEC §5.3 – Level-Dateilöschung mit Dir-Fsync.

**Empfehlung:** `crate::util::fsync_parent_dir(data_path).await?` nach dem Löschloop einfügen.

---

### 3.2 Compaction Write Amplification – kein Budget (MITTEL – D-76)

**Befund:** Es gibt keine implementierte Obergrenze für Write Amplification pro Kompaktierungszyklus. Die Kompaktierungsstrategie (`CostBasedAdaptivePlanner`) wählt zwischen `WriteOptimizedSTCS`, `ReadOptimizedAggressive` und `Balanced`, aber **keine dieser Strategien enthält eine explizite WA-Budget-Obergrenze**. Bei aggressiver Kompaktierung werden alle verfügbaren SSTables (full compaction) gemergt, was im schlimmsten Fall eine WA proportional zur Datenmenge ergibt.

**Optimierungspotenzial:**
- WA-Messung pro Kompaktierungszyklus implementieren (`bytes_written / bytes_read`).
- `max_write_amplification_factor` als konfigurierbare Schranke einführen.
- Vor der Kompaktierung prüfen, ob das WA-Budget noch verfügbar ist.

---

### 3.3 Cluster-bewusste Kompaktierung fehlt (HOCH)

**Befund:** Der Key-Space hat die Struktur `t:{tenant}:{collection}:{type}:{doc_id}`. Weder `CompactionEngine::merge_sstables_with_cancel` noch `CostBasedAdaptivePlanner` berücksichtigen diese Schlüsselhierarchie. Lösch-Cluster (`delete_prefix` auf Collection-Ebene) werden physisch über alle SSTables verteilt, anstatt clusterbewusst zusammengefasst zu werden.

**Implikation:** Lösch-Tombstones eines Tenants/Collection-Paars werden nicht priorisiert kompaktiert, was zu unnötig langen Tombstone-Laufzeiten führt (MVCC-Retention-Overhead) und die Purge-Latenzen der Erasure-Pipeline verlängert.

**Empfehlung:** Kompaktierungsplanung, die Tenant-/Collection-Prefix-Konzentrationen erkennt und priorisiert. Alternativ: Range-basierte Kompaktierung statt globaler Greedy-Auswahl.

---

### 3.4 SSTable-Liste wird O(n log n) bei jedem Flush re-sortiert (MITTEL)

**Befund:** Nach jedem Flush und nach jeder Kompaktierung wird `sstables.sort_by_key(...)` aufgerufen. Bei vielen SSTables ist das eine O(n log n)-Operation unter dem write-lock. Da SSTables monoton wachsende `max_seq` haben, sollte ein Insert-sorted-Algorithmus (O(n) mit binärer Suche) verwendet werden.

**Empfehlung:** `sstables.insert_sorted(new_reader, ...)` mit binärer Suche statt globalem `sort_by_key`.

---

### 3.5 Manifest-Rollover ohne Dir-Fsync der neuen Manifest-Datei (MITTEL)

**Befund:** `manifest/rollover.rs` erstellt eine neue Manifest-Datei bei Überschreitung des Thresholds (64 KB). Die Implementierung ruft `file.sync_all()` für die neue Manifest-Datei auf, aber **nicht** `fsync_parent_dir` nach dem Erstellen der neuen Datei (vor dem Löschen der alten). Der atomare Edit-Cycle ist damit nicht vollständig durable.

---

## 4. Recovery nach Crash

### 4.1 SALT-Datei-Recovery bei non-pristine-Directory (KRITISCH)

**Befund:** `lsm/recovery.rs` prüft bei fehlendem SALT:

```rust
if !directory_is_pristine(&config.path).await? {
    return Err(ContextraError::Storage(
        "SALT file missing in non-pristine database directory".into(),
    ));
}
```

`directory_is_pristine` prüft nur auf `wal-*`, `wal.log`, `*.sst` und `MANIFEST`. Es prüft **nicht** auf `*.sealed.*`-Dateien oder temporäre `SALT.tmp.*`-Dateien. Ein abgebrochener Recovery-Versuch nach einem Crash während `write_salt_atomically` (rename-Phase) kann eine `SALT.tmp.{pid}.{counter}`-Datei hinterlassen, die `directory_is_pristine` **nicht erkennt** – es bleibt fälschlicherweise `true`.

**Kritischer Pfad:** SALT-Temp-Datei vorhanden → `is_pristine = true` → neuer SALT generiert → alle alten WAL/SST-Daten sind mit dem alten SALT verschlüsselt, die Integrity-Keys stimmen nicht mehr → Store nicht recoverable.

**Empfehlung:** `directory_is_pristine` muss auch nach `SALT.tmp.*`-Dateien suchen. Bei Vorhandensein: altes Temp-File entfernen oder umbenennen zu `SALT` (wenn Inhalt valide).

---

### 4.2 WAL-Replay ohne Sequenz-Konsistenz-Check (HOCH)

**Befund:** Bei der Recovery werden alle WAL-Dateien (`wal-*.log`) in Dateiname-Reihenfolge abgespielt. Die Replay-Logik prüft jedoch nicht, ob die Sequenznummern der WAL-Entries lückenlos aufsteigend sind. Eine fehlende WAL-Datei (durch physische Löschung nach einem teilweise gesehenen Crash) würde still ignoriert.

**Empfehlung:** Nach dem Recovery die maximale `seq_no` der letzten bekannten Entry mit der erwarteten `next_seq_no` vergleichen. Bei einer signifikanten Lücke: Fehler statt stiller Recovery.

---

### 4.3 Tombstone-Semantik: physische Bereinigung ohne Erasure-Audit (HOCH)

**Befund:** `erase/layer.rs` ist ein Stub (enthält nur einen Kommentar). Die Kernmission (auditierbare, gemessene Löschung mit Signed Erasure Report) ist nicht implementiert.

`health/reserve.rs` und `health/poison.rs` sind ebenfalls Stubs.

**Implikation:** Die SPEC §8.2 Anforderungen (MK1–MK3 Measurements, Positive Controls, Signed Report) sind nicht erfüllt. Die aktuellen Tombstones werden bei der Kompaktierung physisch entfernt, aber ohne:
- Messung der tatsächlich gelöschten Bytes
- Signed Report
- Disk-Reserve-Prüfung vor Purge

---

### 4.4 Stale SSTables ohne Manifest-Eintrag werden nicht bereinigt (MITTEL)

**Befund:** In `lsm/recovery.rs` werden SST-Dateien geladen, die im Manifest registriert sind. `dead_manifest_sstables` (SSTables die im Manifest als entfernt markiert, aber noch auf Disk vorhanden sind) werden explizit bereinigt. **Jedoch werden SST-Dateien, die auf Disk vorhanden sind, aber nie im Manifest auftauchen** (z.B. aus einem abgebrochenen Flush zwischen `builder.finish()` und `manifest.append_batch()`), nicht systematisch bereinigt.

**Empfehlung:** Bei Recovery alle `.sst`-Dateien im Verzeichnis scannen und gegen die Manifest-Whitelist prüfen. Nicht-registrierte SST-Dateien löschen (mit vorherigem `fsync_parent_dir`).

---

## 5. Concurrency

### 5.1 Intent-Lock-Map unbegrenzt wachsend (MITTEL)

**Befund:** `intent_locks: std::sync::Mutex<HashMap<Vec<u8>, TxId>>` wächst unbegrenzt. Bei besonders langen Transaktionen oder nach abgebrochenen Commits wird das Lock-Cleanup erst am Ende des Commits aufgerufen. Bei vielen parallelen Transaktionen mit `put_if_absent` kann die Map signifikant wachsen.

**Empfehlung:** Kapazitätsgrenze oder TTL für Intent-Locks einführen.

---

### 5.2 Group-Commit-Fenster mit sub-ms-Timer-Granularität (NIEDRIG)

**Befund:** Das Group-Commit-Fenster verwendet `tokio::time::sleep`:

```rust
tokio::select! {
    _ = tokio::time::sleep(Duration::from_micros(storage.config.group_commit_window_micros)) => {}
    _ = notify_full.notified() => {}
}
```

`tokio::time::sleep` hat typisch ~1ms-Granularität auf Linux. `group_commit_window_micros = 500` (500µs) wird faktisch auf ~1ms aufgerundet.

**Empfehlung:** Kommentar oder Warnung, dass sub-millisecond-Präzision systemabhängig ist.

---

### 5.3 Foreground-Schreiber können hinter Purge-I/O blockieren (MITTEL)

**Befund:** Es gibt keinen expliziten Mechanismus, der sicherstellt, dass Foreground-Schreiber nie hinter Purge-I/O blockieren. Kompaktierungsoperationen laufen im gleichen Tokio-Runtime-Pool wie Commits. Bei einem I/O-intensiven Full-Compaction-Vorgang könnten alle Tokio-Worker-Threads mit Kompaktierungs-I/O beschäftigt sein, während Commit-Futures in der Task-Queue warten.

**Empfehlung:** Kompaktierungs-I/O auf eine separate Runtime oder `spawn_blocking` auslagern, um Foreground-Latenz zu garantieren.

---

### 5.4 `last_applied_seq` CAS-Ordering-Frage (NIEDRIG / Analyse)

**Befund:** In `commit.rs`, `apply_mem_updates`:

```rust
match self.last_applied_seq.compare_exchange_weak(
    current,
    max_seq,
    Ordering::Release,    // success
    Ordering::Relaxed,    // failure
)
```

Das `Release`-Ordering auf success ist korrekt für den Speicherzugriff. Die Frage ist, ob alle Pfade, die `last_applied_seq` lesen und darauf basierend Entscheidungen treffen, durch `commit_mutex` korrekt serialisiert sind. `apply_backpressure` liest `budget.memory_used()` ohne Mutex-Schutz und ist damit potentiell racy – unkritisch da nur Best-Effort.

---

## 6. Encryption / Security

### 6.1 `encryption_passphrase` in LsmConfig (HOCH – SPEC D-05)

**Befund:** Die Kommentare in `config.rs` behaupten, `LsmConfig` enthalte keine Passphrase. Aber in `engine.rs` wird `config.encryption_passphrase` referenziert:

```rust
// engine.rs ~428
let key_manager = if let Some(passphrase) = &config.encryption_passphrase {
```

Das bedeutet, `LsmConfig` **enthält doch eine Passphrase** – sie ist nur nicht in `config.rs` dokumentiert. Das ist ein Widerspruch zur Dokumentation und ein Sicherheitsproblem: Passphrases in Konfigurationsstrukturen werden möglicherweise in Logs, Serialisierungen oder Heap-Dumps exponiert.

**Empfehlung:** Passphrase aus `LsmConfig` entfernen; `KeyManager` separat via Dependency Injection übergeben (wie der Kommentar vorgibt).

---

### 6.2 Legacy-Fallback-Integrity-Key: sicherheitsrelevanter Parameter silent discarded (HOCH – P6)

**Befund:** Der Code enthält einen `legacy_integrity_key`-Fallback. Die Implementierung in `io.rs` enthält:

```rust
let _ = (path, fallback_integrity_key, allow_legacy_integrity_key_fallback, &mut using_legacy_key);
```

Das Tuple wird discarded – die Variablen sind vorhanden, aber der eigentliche Legacy-Fallback-Pfad ist mit `let _ =` überbrückt. Das ist P6-widrig und bedeutet, dass Prod-Code existiert, der einen sicherheitsrelevanten Pfad still ignoriert.

**Empfehlung:** Legacy-Fallback vollständig entfernen oder korrekt implementieren. Kein `let _ =` auf sicherheitsrelevante Parameter.

---

## 7. Vfs-Port-Konformität (HOCH – D-04)

**Befund:** Folgende Produktions-Quellen verwenden direkt `tokio::fs` statt des Vfs-Ports:

| Datei | Verstoß |
|---|---|
| `src/lsm/recovery.rs` | `tokio::fs::read_dir`, `tokio::fs::read`, `tokio::fs::write`, `tokio::fs::rename`, `tokio::fs::remove_file`, **`std::fs::File::open`** |
| `src/compaction/engine.rs` | `tokio::fs::remove_file`, `tokio::fs::try_exists` |
| `src/lsm/ops/compaction.rs` (flush) | `tokio::fs::remove_file` |
| `src/manifest/core.rs` | `tokio::fs::OpenOptions`, `tokio::fs::File` |
| `src/wal/flusher.rs` | `crate::wal::fs` (eigener Wrapper, nicht Vfs-Port) |

Die Verwendung des lokalen `crate::wal::fs`-Wrappers ist für die Übergangsphase tolerierbar, aber der `std::fs::File::open`-Aufruf in `recovery.rs` ist ein **bedingungsloser SPEC-Verstoß**.

---

## 8. Fehlende Tests für Crash-Szenarien (MITTEL)

Die folgenden Crash-Szenarien haben keine expliziten Recovery-Tests:

| Szenario | Status |
|---|---|
| Crash nach `SstableBuilder::finish()`, vor `manifest.append()` | Kein Test gefunden |
| Crash während `manifest.append_batch()` (nach 1. Entry, vor 2.) | Kein Test gefunden |
| Crash nach `manifest.append()`, vor `fsync_parent_dir` | Kein Test gefunden |
| Crash während `compaction/engine.rs` Step 7 (SST-Löschung) | Kein Test gefunden |
| `SALT.tmp` vorhanden bei Recovery-Start | Kein Test gefunden |
| WAL-Segment fehlt (durch externen Eingriff) | Kein Test gefunden |

---

## 9. Tenant-Key-Encoding: ASCII statt 78-Byte-KeyBuf (MITTEL / Architektur)

**Befund:** `TenantKeyCodec` kodiert Keys als **ASCII-Strings** (`"t:{tenant_id}:{collection}:{type}:{doc_id}"`). Das ist kein 78-Byte-KeyBuf-Schema (SPEC-Anforderung). String-Keys haben:
- Variable Länge → kein konstanter Lookup-Overhead
- Vergleichs-Overhead bei Scans im Hot-Path
- Keine strukturelle Grundlage für cluster-bewusste Kompaktierung

Die Rollenbeschreibung fordert ausschließlich das 78-Byte-KeyBuf-Schema (`region‖tenant‖coll‖cluster‖doc`). Das ASCII-Schema ist kein zweites erlaubtes Schema.

**Empfehlung:** Migration auf festes 78-Byte-Layout mittels `[u8; 78]` Buffer. Cluster-ID im Key verankern, um cluster-bewusste Kompaktierung zu ermöglichen.

---

## 10. Zusammenfassung: Prioritätsmatrix

| Befund | Priorität | SPEC-Ref |
|---|---|---|
| SALT-Recovery bei non-pristine-Dir (§4.1) | **KRITISCH** | D-04 |
| HMAC-Kettenvalidierung Mmap-Pfad (§1.1) | **KRITISCH** | HR-3, P5 |
| Vfs-Port-Verletzung std::fs in recovery.rs (§7) | **HOCH** | D-04 |
| Dir-Fsync nach SST-Löschung fehlt (§3.1) | **HOCH** | SPEC §5.3 |
| fsync-Fehler auf gleichem Handle (§1.5) | **HOCH** | D-55 |
| Cluster-bewusste Kompaktierung fehlt (§3.3) | **HOCH** | D-76 |
| encryption_passphrase in LsmConfig (§6.1) | **HOCH** | D-05, P5 |
| Legacy-Fallback silent ignore (§6.2) | **HOCH** | HR-3, P6 |
| WAL-Replay ohne Seq-Konsistenz-Check (§4.2) | **HOCH** | T-WAL-1 |
| Erasure-Layer Stub (§4.3) | **HOCH** | SPEC §8.2 |
| `let _ =` auf MemTable-size in rollback (§2.2) | **HOCH** | P6 |
| Stale SST ohne Manifest bei Recovery (§4.4) | **MITTEL** | SPEC §13.1 |
| MemTable-Größen-Buchhaltung (§2.1) | **MITTEL** | – |
| WA-Budget fehlt (§3.2) | **MITTEL** | D-76 |
| SSTable-Sort O(n log n) bei Flush (§3.4) | **MITTEL** | – |
| Intent-Lock-Map unbegrenzt (§5.1) | **MITTEL** | – |
| Manifest-Rollover dir-Fsync (§3.5) | **MITTEL** | D-04 |
| Foreground-Writer vs. Purge-I/O (§5.3) | **MITTEL** | – |
| ASCII-Key statt 78-Byte KeyBuf (§9) | **MITTEL** | SPEC §5.3 |
| Loom-Mock busyloop (§1.6) | **NIEDRIG** | P6 |
| Shard-Scan für System-Keys (§2.3) | **NIEDRIG** | – |
| Group-Commit Timer-Granularität (§5.2) | **NIEDRIG** | – |

---

## 11. Empfohlene nächste Schritte (Arbeitspakete)

1. **WP-STORE-A (KRITISCH):** `directory_is_pristine` um `SALT.tmp.*`- und `*.sealed.*`-Erkennung erweitern + Test `T-STORE-SALT-RECOVERY-1`.
2. **WP-STORE-B (HOCH):** `fsync_parent_dir` nach alten SST-Löschungen in Kompaktierung und Flush + Crash-Recovery-Tests für Phase 3.
3. **WP-STORE-C (HOCH):** HMAC-Kettenvalidierung im Mmap-Replay-Pfad mit `IntegrityVerifier` + Test `T-WAL-CHAIN-MMAP`.
4. **WP-STORE-D (HOCH):** `std::fs`-Aufruf in `recovery.rs` auf Vfs-Port migrieren; Roadmap für vollständige D-04-Konformität.
5. **WP-STORE-E (HOCH):** `encryption_passphrase` aus `LsmConfig` entfernen; `KeyManager` als separaten Konstruktor-Parameter führen.
6. **WP-STORE-F (MITTEL):** Recovery-Scan für stale SST-Dateien ohne Manifest-Eintrag (Orphan-SST-Bereinigung).
7. **WP-STORE-G (langfristig):** 78-Byte-KeyBuf-Migration + cluster-bewusste Kompaktierungsstrategie auf Basis der strukturierten Keys.

---

*Dieser Report ist abgeleitet aus dem Ist-Zustand des Repositorys (2026-11-02); normative Grundlage bleibt SPEC . Bei Widerspruch gilt die SPEC.*
