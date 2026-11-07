# AGENTS.md — `contextra-checkpoint`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-checkpoint` koordiniert transaktionskonsistente Snapshots und Checkpoints über heterogene Speicherkomponenten hinweg (Ring 1).
Herzstück ist der `CheckpointGuard`, der über RAII-Semantik sicherstellt, dass Transaktionen entweder explizit committet werden oder bei Panic/Drop deterministisch zurückrollen.
Als isolierter Baustein implementiert er `CheckpointCoordinator` und speichert benannte Snapshots (`PersistentCheckpointStore`) mit BLAKE3-Manifest-Verifikation.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 1 (Snapshot-Koordination & Time-Travel)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports`, `contextra-store`, `contextra-mvcc` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **RAII-Commit-Garantie:** Ein nicht explizit committeter `CheckpointGuard` löst beim Drop deterministisch ein Rollback aus.
- **Transaktions-Konsistenz:** Checkpoints bilden exakte, zeitpunktbezogene Schnittmengen über LSM-Store und assoziierte Indizes ab.
- **Manifest-Integrität:** Jeder Checkpoint wird durch einen BLAKE3-Kryptohash geschützt und beim Laden verifiziert.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()` und `expect()`; Fehler bei Snapshot-Erstellung oder -Wiederherstellung werden als `ContextraError` gemeldet.

## 4. Verbotene Muster
- Keine Abhängigkeiten zu `contextra-core` (direkte Nutzung von `contextra-types` und `contextra-ports`).
- Keine Vermischung mit der internen, privaten Snapshot-Logik von `contextra-store::checkpoint`.
- Keine unvollständigen Snapshots bei partiellen I/O-Fehlern (alles-oder-nichts Atomizität).

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Implementiert:** `CheckpointCoordinator`.
- **Konsumiert:** `Clock`-Port, `StorageEngine`-Port, `contextra-mvcc`.
- **Bereitgestellt für:** `contextra-engine`, `contextra-agent`.

## 6. Tests, Sabotage & Qualitätssicherung
- **Rollback-on-Drop Tests:** Verifikation der Rollback-Invariante bei simuliertem Thread-Panic und vorzeitigem Scope-Verlassen.
- **Crash-Recovery Tests:** Wiederherstellung benannter Checkpoints nach unvollständigen Schreibvorgängen.
- **Mutations-Testing:** Prüfung der Erkennung manipulierter Manifest-Dateien.

## 7. Offene Punkte & Technische Schulden
- Keine.
