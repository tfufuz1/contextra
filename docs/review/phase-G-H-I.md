# Phase G, H & I: Plattform, Governance-Realität & Ende-zu-Ende Verifikation

## 1. Plattform, Portabilität & Reproduzierbarkeit (Phase G)
- **Plattformmatrix (`target_os`)**:
  - Im Code existieren **7 `cfg(target_os)` Abfragen**.
  - Spezifische POSIX/Linux-Pragmas (`apply_dontdump`, `MADV_DONTDUMP`, POSIX Read-Only Filesystem Chmod Error Test) sind in `crates/contextra-sys/src/posix.rs` hinter `#[cfg(target_os = "linux")]` gekapselt.
  - *Befund (S2)*: Windows-Dateisperren und Windows-Directory-fsync (`FlushFileBuffers` / `ReOpenFile`) werden in `contextra-sys` und `contextra-store` nur generisch oder als No-Op behandelt, was auf Windows-Systemen nach Crashs zu Datenverlust in WAL/SSTables führen kann (Portabilitätslücke).
- **Build-Reproduzierbarkeit**: `Cargo.lock` ist committed. `flatc` Binary ist im Host-System installiert.

---

## 2. Dokumentation, Spezifikationsabweichung & Governance (Phase H)
- **Dokumentations-Drift**:
  - `README.md` verweist in der Kopfzeile noch auf "Fassung 1.2" und `CONTEXTRA_SPEZIFIKATION_v15.md`, während die operative Arbeitsgrundlage Fassung 1.3 darstellt.
  - `AGENTS.md` trägt den Stand `2026-10-25 (Fassung 1.2)`.
  - *Befund (S3)*: Dokumentations-Drift zwischen Versionierungs-Hedging in README/AGENTS.md und der normativen Zielspezifikation v1.3.

---

## 3. Ende-zu-Ende Verifikation & Abläufe (Phase I)
- **Ablauf 1: `remember` -> `recall` -> `forget` -> `restart` -> `recall`**:
  - `remember` schreibt Werte ins WAL, aktualisiert Memtable, HNSW-Graph und Inverted Index.
  - `forget` setzt Read-Fence, leitet `PurgeScope` ein und widerruft den Schlüssel im `KeyRegistry`.
  - *Gegenproben-Befund*: Der DeletionProof quittiert HNSW-Indexbereinigungen per Stub (`|| Ok(true)` in `deletion_proof.rs:1537`). Zwar ist der Schlüssel widerrufen und Lesezugriffe über die Fassade gesperrt, jedoch verbleiben Vektoreinträge im physischen HNSW-Graph.
- **Ablauf 2: Crash-Recovery & WAL Replay**:
  - Bei abrupter Unterbrechung (Process Crash) stellt B0-B9 Recovery den konsistenten Zustand aus dem WAL wieder her. Unvollständige WAL-Frames am Ende der Datei werden ordnungsgemäß als Torn-Tail erkannt und abgeschnitten.
