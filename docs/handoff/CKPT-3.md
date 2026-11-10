# Handoff CKPT-3 — contextra-checkpoint Entkopplung

## Status & Abbruch am Start-Gate

Gemäß Auftragsvorgabe:
> **START-GATE (alle Befehle müssen erfolgreich sein, sonst Abbruch)**
> ```bash
> test -d crates/contextra-ids && test -d crates/contextra-env
> grep -q 'contextra_ids' crates/contextra-types/src/lib.rs
> grep -q 'contextra_env' crates/contextra-ports/src/lib.rs
> cargo check -p contextra-checkpoint --all-targets   (Ergebnis in die Baseline)
> ```
> *Schlägt einer der Prüfbefehle vor dem letzten fehl, ist das Fundament-Paket P0 (FND-1) oder eine der genannten Vorbedingungen nicht gemergt: Brich ab, schreibe nur `docs/handoff/CKPT-3.md` mit dem Blocker und ändere keinen Code.*

### Start-Gate Prüfergebnisse (Stand origin/main)

1. `test -d crates/contextra-ids && test -d crates/contextra-env`
   - **Ergebnis:** Exit-Code 0. Die Verzeichnisse existieren zwar, enthalten derzeit jedoch nur `PLAN.md` und `SPEC.md` (noch kein `Cargo.toml` oder Quellcode).
2. `grep -q 'contextra_ids' crates/contextra-types/src/lib.rs`
   - **Ergebnis:** FEHLGESCHLAGEN (Exit-Code 1)
   - Re-Export / Verweis auf `contextra_ids` in `crates/contextra-types/src/lib.rs` ist nicht vorhanden.
3. `grep -q 'contextra_env' crates/contextra-ports/src/lib.rs`
   - **Ergebnis:** FEHLGESCHLAGEN (Exit-Code 1)
   - Re-Export / Verweis auf `contextra_env` in `crates/contextra-ports/src/lib.rs` ist nicht vorhanden.
4. `cargo check -p contextra-checkpoint --all-targets`
   - **Ergebnis:** ERFOLGREICH (0 Fehler)

### Identifizierter Blocker

Das Fundament-Paket **P0 (FND-1)** zur vollständigen Auslagerung und Einbindung von `contextra-ids` und `contextra-env` (insb. Einbindung in `types` und `ports`) ist noch nicht gemergt. Die Vorbedingungen für die Kern-Closure von `contextra-checkpoint` auf `ids` und `env` sind daher nicht erfüllt.

### Ergriffene Maßnahme

Gemäß expliziter Instruktion des Auftrags wurde die Bearbeitung am Start-Gate abgebrochen. Es wurde **keinerlei Quellcode geändert**.

---

## Koordinationsanforderung

- **Voraussetzung:** Merge des Fundament-Pakets P0 (FND-1):
  - Bereitstellung des Rust-Crates `crates/contextra-ids` (inkl. `Cargo.toml`, `src/`)
  - Bereitstellung des Rust-Crates `crates/contextra-env` (inkl. `Cargo.toml`, `src/`)
  - Einbindung von `contextra_ids` in `crates/contextra-types/src/lib.rs`
  - Einbindung von `contextra_env` in `crates/contextra-ports/src/lib.rs`
- Sobald P0 gemergt ist, kann CKPT-3 (Entkopplung von `contextra-checkpoint` auf `SnapshotSource`, schmale Dateinaht, `CheckpointError` und `compat/`) ausgeführt werden.
