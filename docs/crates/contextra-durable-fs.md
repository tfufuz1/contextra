# contextra-durable-fs — Crate-Spezifikation (Durabilitätsgarantien & Crash-Sicherheit)

**Ebene:** E0 (Autarkes Fundament / Ring 0)  
**Kern-Closure:** Leer (0 interne Workspace-Abhängigkeiten)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-durable-fs`  
**Bindung:** `docs/SPEC.md` §2 (Architektur), §5.3 (Lösch-Saga fsync-Anker), §13 (INV-1, INV-6)

---

## 1. Zweck & Systemrolle

`contextra-durable-fs` stellt verlässliche, crash-sichere Dateisystemoperationen und synchrone Disk-Flushes für das Contextra-System bereit. Es bildet das I/O-Durabilitätsfundament für das Write-Ahead-Log (WAL), das Manifest, die SSTable-Erstellung und das Erasure-Journal.

### Die Kernversprechen
1. **Vergiftung bei Sync-Fehlern (`INV-FSYNC-1`):** Schlägt ein `fsync`- oder Flush-Aufruf fehl, wird der Dateideskriptor sofort und unwiderruflich vergiftet (Handle-Poisoning). Nachfolgende Schreiboperationen werden abgewiesen, um stille Datenkorruption auszuschließen.
2. **Atomare Dateieretzung (`INV-VFS-ATOMIC-1`):** Mutationen persistierter Tabellen erfolgen strikt über temporäre Dateien im selben Zielverzeichnis, gefolgt von Daten-Sync, atomarem Rename und Verzeichnis-Sync.
3. **Sicheres Schreddern (Scrubbing):** Echtes Überschreiben physischer Dateien mit Nullen vor dem `unlink` zur Unterstützung der DSGVO-Bereinigung.

---

## 2. Architektur & Komponenten

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            contextra-durable-fs                             │
│                                                                             │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │           Vfs            │             │          platform          │   │
│   │  (StdVfs, RealFs,        │             │ (sync_file, sync_dir_path, │   │
│   │   write_atomic, TmpGuard)│             │  FlushKind: Data/Metadata) │   │
│   └─────────────┬────────────┘             └─────────────┬──────────────┘   │
│                 │                                        │                  │
│                 ▼                                        ▼                  │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │     health & Poison      │             │           scrub            │   │
│   │ (AtomicHealthState,      │             │(scrub_file_with_rate_limit,│   │
│   │  HealthEvent, INV-FSYNC-1)             │ ScrubPolicy, ScrubReceipt) │   │
│   └──────────────────────────┘             └────────────────────────────┘   │
│                                                                             │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │         journal          │             │       fault_injector       │   │
│   │(AppendOnlyJournal,       │             │(FaultInjector, NoFault,    │   │
│   │ PendingDeleteJournal)    │             │ Crash-Punkt-Simulation)    │   │
│   └──────────────────────────┘             └────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Durabilitäts-Invarianten

### 3.1 Handle-Poisoning (`INV-FSYNC-1`)

Wird beim Aufruf von `sync_data()` oder `sync_all()` ein Betriebssystem-Fehler (z. B. EIO, ENOSPC) gemeldet, wechselt der `AtomicHealthState` irreversibel auf `HealthState::Poisoned`:
- Alle nachfolgenden `write()`- oder `sync()`-Aufrufe auf diesem Deskriptor scheitern sofort mit `Err(DurableFsError::HandlePoisoned)`.
- Dies verhindert, dass ein Betriebssystem nach einem unbemerkten Puffer-Verwerfen (Page Cache Drop) inkonsistente Teilblöcke auf Platte schreibt.

### 3.2 Atomares Schreiben (`INV-VFS-ATOMIC-1`)

Die Methode `vfs.write_atomic(target_path, data)` führt die normative 4-Schritte-Sequenz aus:
1. **Temporäre Datei anlegen:** Erzeugt `.target_path.tmp.<pid>.<uuid>` im **selben Verzeichnis** wie das Ziel (garantiert denselben Dateisystem-Mount / Inode-Pool).
2. **Schreiben & Synchronisieren:** Vollständiges Schreiben der Nutzlast, gefolgt von `sync_file` (Daten und Metadaten der temporären Datei durabel).
3. **Atomares Rename:** Ersetzen der Zieldatei via `std::fs::rename`.
4. **Verzeichnis-Synchronisation:** Aufruf von `sync_dir_path(parent_dir)` via `contextra-sys`, damit der neue Verzeichniseintrag crash-resistent auf dem Speichermedium persistiert ist.

### 3.3 Dateisystem-Klassifikation (`FsClass`)

Prüfung der Umgebung vor Aktivierung von Durabilitätsgarantien:
- **`FsClass::LocalSupported`:** Lokale Dateisysteme (ext4, xfs, btrfs, APFS, NTFS) mit garantierten POSIX/Win32-Flush-Semantiken.
- **`FsClass::Network` / `FsClass::Fuse` / `FsClass::CloudSynced`:** Unsichere Mounts (NFS, SMB, FUSE, Cloud-Ordner), bei denen `sync_dir` unwirksam sein kann. Führt bei `durability_mode = Required` zur kontrollierten Verweigerung (`Err(DurableFsError::UnsupportedFilesystem)`).

---

## 4. Rolle in der Löschkette (SPEC §5)

### 4.1 Die 4 fsync-Ankerpunkte der Lösch-Saga

`contextra-durable-fs` stellt die Durabilitätsanker bereit, an denen die Saga-Zustände im `ErasureJournal` fixiert werden:
1. **fsync #1 (Intent-Journal):** Vor der Einleitung von Sperren MUSS der Löschauftrag durabel im Journal stehen.
2. **fsync #2 (Gated-Zustand):** Nach Erwerb der `CommitBarrier` wird der `Gated`-Zustand synchron festgeschrieben.
3. **fsync #3 (KeyRevoked):** Nach dem Nullen der Schlüsselslots im Vault wird der Widerruf durabel bestätigt.
4. **fsync #4 (Abschluss & Ledger):** Das Ledger AUSSERHALB des Datenverzeichnisses wird synchron geschrieben.

### 4.2 Physisches Schreddern (`scrub`)

- **`scrub_file_with_rate_limit(path, policy)`:** Vor dem Löschen (`unlink`) einer ersetzten SSTable oder eines gelöschten Indexsegments überschreibt der Scrubber die Dateiblöcke sequentiell mit Nullen oder pseudozufälligen Mustern (`ScrubPolicy`).
- **Ratenbegrenzung:** Schützt die SSD/NVMe vor I/O-Spitzen während Hintergrundbereinigungen.
- **`ScrubReceipt`:** Gibt kryptographischen Beleg über bereinigte Byteanzahl zurück.

---

## 5. Append-Only-Journal (`journal`)

- **`AppendOnlyJournal`:** Binäres, CRC32C-geschütztes Log für Transaktionsabsichten und Lösch-Sagas.
- **`PendingDeleteJournal`:** Verzeichnet zur physischen Löschung vorgemerkte Dateien, um verwaiste temporäre Dateien nach einem Systemabsturz beim nächsten Start restlos zu entfernen.

---

## 6. Abhängigkeiten

| Art | Bibliothek |
|-----|-----------|
| **Intern** | Keine (0 interne Workspace-Abhängigkeiten) |
| **Extern** | `crc32fast`, `parking_lot`, `thiserror`, `tracing` |

---

## 7. Invarianten

1. **`#![forbid(unsafe_code)]`** im gesamten Crate.
2. **Zero-Panic-Policy:** Dateisystemfehler führen niemals zu Panics, sondern stets zu `Result<T, DurableFsError>`.
3. **Kein ungesynchronisiertes Rename:** Vor `rename` MUSS die temporäre Datei gesynct sein; nach `rename` MUSS das Elternverzeichnis gesynct sein (`INV-VFS-ATOMIC-1`).
4. **Irreversibles Handle-Poisoning:** Ein einmal fehlgeschlagener Flush sperrt den betroffenen Deskriptor dauerhaft (`INV-FSYNC-1`).

---

## 8. Test-Pflichten

| Test-ID | Beschreibung |
|---------|-------------|
| T-DFS1 | `write_atomic`: Zieldatei existiert nach erfolgreichem Aufruf vollständig und durabel |
| T-DFS2 | `write_atomic` Crash-Simulation: Abbruch vor Rename hinterlässt keine korrupte Zieldatei |
| T-DFS3 | Handle-Poisoning: Injizierter Fsync-Fehler blockiert nachfolgende Schreibaufrufe |
| T-DFS4 | `scrub_file`: Überschriebene Datei enthält vor dem Unlink ausnahmslos Nullen |
| T-DFS5 | `PendingDeleteJournal`: Nach Crash-Simulation werden registrierte Restdateien aufgeräumt |
| T-DFS6 | `FsClass`: Erkennt Netzwerk- oder FUSE-Dateisysteme korrekt |
