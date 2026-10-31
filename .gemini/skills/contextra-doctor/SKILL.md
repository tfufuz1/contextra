---
name: contextra-doctor
description: >-
  Use this skill to diagnose and repair Contextra build failures, workspace manifest inconsistencies,
  compiler errors (such as B-01 delete_segment and B-03 durable-fs), xtask replacements, and Cargo dependency issues.
---

# Contextra Doctor – Build & Workspace Reparaturanleitung

Dieser Skill leitet Entwickler und Agenten an, den Contextra-Workspace in einen sauberen, baubaren und testbaren Zustand zu bringen und zu halten.

## Schnell-Diagnose

Führe das mitgelieferte Diagnose-Skript aus:
```bash
bash .agents/skills/contextra-doctor/scripts/doctor.sh
```

---

## 1. Workspace-Integrität & Manifest-Prüfung

### Prüfung
```bash
cargo metadata --no-deps --format-version 1 > /dev/null
```
- **Exit-Code 0**: Workspace-Manifeste und Pfade sind konsistent.
- **Exit-Code 101**: Mindestens ein Workspace-Mitglied oder eine Pfad-Abhängigkeit existiert nicht im Dateisystem.

### Reparatur fehlender Mitglieder
Falls Crates aus dem Workspace entfernt oder wiederhergestellt werden sollen:
1. Prüfe `Cargo.toml` im Root:
   - `[workspace].members`
   - `[workspace].default-members`
   - `[workspace.dependencies]`
2. Stelle sicher, dass jeder referenzierte relative Pfad tatsächlich ein Verzeichnis mit `Cargo.toml` besitzt.
3. Crates, die nicht gebaut werden sollen, sauber aus `members` entfernen oder hinter Feature-Gates legen.

---

## 2. Bekannte Compiler- und Baufehler (Runbooks)

### Runbook B-01: Typfehler im Löschpfad (`delete_segment`)
- **Datei**: `crates/contextra-store/src/kv/segment.rs:152-157`
- **Problem**: `registry.revoke_subkey(group_id)` gibt `Result<bool, CryptoError>` zurück, aber `delete_segment` gibt fälschlicherweise `bool` zurück.
- **Lösung**:
  - Signatur von `delete_segment` auf `Result<bool, ContextraError>` umstellen.
  - Fehler mit `?` propagieren (kein `unwrap_or(false)` oder `.ok()`, da dies einen Löschfehler verschlucken würde).
  - Aufrufer und Modultests entsprechend auf `Result` anpassen.

### Runbook B-02: `xtask`-Bereinigung
- **Problem**: Das `xtask`-Crate existiert nicht mehr, wird aber noch in Konfigurationen referenziert.
- **Lösung**:
  1. `.cargo/config.toml`: Alias `xtask = ...` entfernen oder durch direkte Cargo-Kommandos ersetzen.
  2. `justfile`: Aufrufe von `cargo xtask <subcmd>` durch Cargo-Aliase oder Shell-Skripte ersetzen.
  3. `.github/workflows/*`: `xtask`-Aufrufe durch `cargo test --workspace` und spezifische Prüfkommandos ersetzen.

### Runbook B-03: `contextra-durable-fs` Syntax- und Importfehler
- **Dateien**:
  - `crates/contextra-durable-fs/src/journal.rs:183`: Lexer-Fehler (Zeichenliteral `'` mit Zeilenumbruch durch `'\n'` ersetzen).
  - `crates/contextra-durable-fs/src/lib.rs` / `scrub.rs`: Importe aus `contextra-sys` anpassen. `contextra-sys` exportiert nun `durable_flush_file`, `durable_flush_dir`, `write_all_durable`.
- **Lösung**: Importe an die tatsächlich exportierten Sys-Funktionen anpassen und Vfs-Traiterfüllung testen.

### Runbook B-06: `contextra-ingest` Module
- **Problem**: `fmt-docx` und `fmt-sheets` verweisen auf nicht existente Hilfsmodule; `chunking/stubs.rs` verweist auf `paged_impl::draft_paged`.
- **Lösung**: Modulimporte auf `paged` korrigieren, XML/OPC-Helper sauber re-exportieren.

### Runbook B-07: `contextra-py` Bindings & FusionWeights
- **Problem**: `FusionWeights` liegt in `contextra_types`, nicht in `contextra_rank`.
- **Lösung**: Import in `crates/contextra-py/src/crud_macros.rs` auf `contextra_types::FusionWeights` anpassen.

---

## 3. Verifikations-Pipeline

Nach Behebung von Fehlern führe folgende Prüfkette aus:

```bash
# 1. Metadaten-Check
cargo metadata --no-deps --format-version 1 > /dev/null

# 2. Workspace Check
cargo check --workspace --all-targets

# 3. Deny-Lints & Clippy
cargo clippy --workspace --all-targets -- -D warnings \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic \
  -D clippy::indexing_slicing

# 4. Kern-Testsuite
cargo test --workspace --exclude contextra-py
```
