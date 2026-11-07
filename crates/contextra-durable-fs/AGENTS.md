# AGENTS.md — `contextra-durable-fs`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-durable-fs` stellt ein atomares, crash-konsistentes Dateisystem-Fundament (Ring 0) für das Contextra-Baukastensystem bereit.
Es implementiert synchrone Disk-Flushes (`fsync`, `fdatasync`), atomare Ersetzungen (`atomic_replace`), sichere Verzeichnis-Synchronisation und feingranulares Verzeichnis-Locking (`DirLock`).
Als autarker Baustein realisiert es den Standard-Port `Vfs` aus `contextra-ports` und ermöglicht isolierte Datensicherheit für alle Speichermodule.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Durability & File System I/O)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-sys` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **INV-DURABLE-FS-SAFE:** Reine sichere Dateisystem-Verwaltung; Fehler werden deterministisch als `std::io::Result` propagiert.
- **INV-FSYNC-1 (D-55):** Handle-Poisoning bei fatalen fsync-Fehlern in `DurableHandle`; verhindert Folgeoperationen nach I/O-Ausfällen.
- **INV-ENOSPC-1:** Fail-closed Fehlerbehandlung bei vollen Datenträgern (Disk Full) und I/O-Unterbrechungen bei Schreib-, Rename- und Verzeichnis-Sync-Vorgängen.
- **Verzeichnis-Locking (INV-DUR-1):** Einheitliches Directory-Locking über `DirLock` mit `LockPolicy::FailClosed` als Standard.
- **Zero-Panic-Doktrin:** Vollständiges Verbot von `panic!`, `unwrap()` oder `expect()` in produktiven Dateisystem-Pfaden.

## 4. Verbotene Muster
- Nicht-atomares Überschreiben von Daten-Dateien ohne `.tmp` Staging und Parent-Directory `fsync`.
- Verschlucken von fsync- oder flush-Fehlern (`let _ = file.sync_all()` ist strikt verboten).
- Vermischung von FS-Primitiven mit höherer Domänen- oder Transaktionslogik.

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Implementiert:** `Vfs`-Port für atomare Datei- und Verzeichnisoperationen.
- **Bereitgestellt für:** `contextra-store`, `contextra-vector`, `contextra-checkpoint`, `contextra-erasure`.

## 6. Tests, Sabotage & Qualitätssicherung
- **Fault-Injection:** `FailpointVfs` und `FaultInjector` für deterministische Simulation von Crash-Szenarien, ENOSPC und I/O-Timeouts.
- **Sabotage-Tests:** Verifikation der Handle-Poisoning-Invariante nach simulierten fsync-Fehlern.
- **Conformity Tests:** POSIX- und Windows-Atomizitätsprüfung.

## 7. Offene Punkte & Technische Schulden
- Keine.
