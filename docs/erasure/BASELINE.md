# Baseline-Messbericht: `contextra-erasure` (WP-0 / P0)

**Datum:** 2026-11-11
**Ziel-Crate:** `crates/contextra-erasure`
**Git HEAD:** `35a855060`
**Referenzdokument:** `docs/erasure/Loeschkette_Implementierungsspezifikation.md`

---

## 1. Verifikationsbefehle & Testergebnisse

Protokollierung aller im Regelblock geforderten Verifikationsschritte:

| Befehl | Exit-Code | Bestandene Tests | Fehlgeschlagene Tests | Ignorierte Tests | Bemerkung / Relevante Ausgabe |
|---|:---:|:---:|:---:|:---:|---|
| `cargo fmt -p contextra-erasure -- --check` | `1` | — | — | — | Diff in `crates/contextra-erasure/tests/golden_erasure_test.rs:193` und `:206`. Nicht im SCHREIBRECHT von P0 (fremde Datei), bleibt unverändert. |
| `cargo check -p contextra-erasure --no-default-features` | `0` | — | — | — | Sauber kompiliert, keine Warnungen/Fehler. |
| `cargo check -p contextra-erasure --all-features` | `0` | — | — | — | Sauber kompiliert, keine Warnungen/Fehler. |
| `cargo check -p contextra-erasure --no-default-features --features signing` | `0` | — | — | — | Sauber kompiliert. |
| `cargo check -p contextra-erasure --no-default-features --features vault` | `0` | — | — | — | Sauber kompiliert. |
| `cargo check -p contextra-erasure --no-default-features --features async` | `0` | — | — | — | Sauber kompiliert. |
| `cargo clippy -p contextra-erasure --all-targets --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic` | `101` | — | — | — | 4 Fehler `-D deprecated` in Testdatei `crates/contextra-erasure/tests/deletion_proof_v3_validation.rs` (Zeilen 26, 27, 68, 109) wegen `LayerCleanupProof::new_after_verified_empty`. Siehe Abschnitt 4. |
| `cargo clippy -p contextra-erasure --lib --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic` | `0` | — | — | — | Produktionscode (`src/`) ist vollständig clippy-rein (Zero-Panic, keine unwrap/expect/panic, keine Warnungen). |
| `cargo test -p contextra-erasure --all-features` | `0` | **160** | **0** | **1** | Alle 160 Tests bestanden. 1 ignorierter Test (`generate_golden_files` in `golden_erasure_test.rs`). |
| `cargo test -p contextra-erasure --no-default-features` | `0` | **47** | **0** | **0** | Alle 47 isolierten Kern-Tests bestanden. |
| `cargo test -p contextra-erasure --no-default-features --features signing` | `0` | **115** | **0** | **1** | Alle 115 Tests bestanden (1 ignoriert: `generate_golden_files`). |
| `cargo test -p contextra-erasure --no-default-features --features vault` | `0` | **47** | **0** | **0** | Alle 47 Tests bestanden. |
| `cargo test -p contextra-erasure --no-default-features --features async` | `0` | **79** | **0** | **0** | Alle 79 Tests bestanden. |

---

## 2. Abhängigkeits- und Autarkie-Prüfung

### 2.1 Prüfung auf interne Workspace-Abhängigkeiten (`cargo tree`)
Befehl:
```bash
cargo tree -p contextra-erasure | grep -i contextra
```
**Ergebnis:**
```text
contextra-erasure v0.1.0 (/app/crates/contextra-erasure)
```
Es existieren **0** Abhängigkeiten zu `contextra-types`, `contextra-ports`, `contextra-crypto`, `contextra-store` oder anderen `contextra-*` Crates. Das Crate ist vollständig autark.

### 2.2 Direkte Abhängigkeiten (aus `crates/contextra-erasure/Cargo.toml`)
- **Normale Abhängigkeiten:**
  - `ahash` (Workspace)
  - `rand = "0.8"`
  - `subtle = "2.6"`
  - `aes-gcm-siv = { version = "0.11", optional = true }` (Feature `vault`)
  - `thiserror` (Workspace)
  - `tracing` (Workspace)
  - `serde` (Workspace)
  - `serde_json` (Workspace)
  - `zeroize = { version = "1.8.2", features = ["zeroize_derive"] }`
  - `blake3 = "1.8"`
  - `sha2 = "0.10"`
  - `hmac = "0.12"`
  - `ed25519-dalek = { version = "2", features = ["rand_core"], optional = true }` (Feature `signing`)
  - `bytes` (Workspace)
  - `parking_lot` (Workspace)
  - `futures-util = { version = "0.3", optional = true }` (Feature `async`)
  - `tokio = { workspace = true, optional = true }` (Feature `async`)
  - `bincode = "1.3"`
- **Dev-Abhängigkeiten:**
  - `tempfile` (Workspace)
  - `proptest = "1.4"`
  - `loom = "0.7"`
  - `tokio = { workspace = true, features = ["test-util"] }`

### 2.3 Werkzeugprüfung: `tools/decouple_inspector.py`
Befehl:
```bash
python3 tools/decouple_inspector.py -c contextra-erasure --all
```
**Messwerte:**
- **Status:** Exzellent (Baustein Klasse A)
- **Modularitäts-Score:** `99/100`
- **Umfang:** 12.464 LOC in 66 Dateien
- **Schnittstellen:** 30 Traits, 149 Structs, 58 Enums
- **Funktionen:** 473 pub / 757 gesamt
- **Interne Abhängigkeiten:** `0` (völlig autark)
- **Dead-Code-Befund:** 2 ungenutzte private Symbole (als Inventar-Kandidaten vorgemerkt)

---

## 3. Spezifische Tests aus Spec WP-4 und WP-9

Gemäß Spec WP-4 und WP-9 wurden die dort genannten Testsuiten geprüft:

1. **`tests/crash_matrix_recovery.rs` (WP-4):**
   ```bash
   cargo test -p contextra-erasure --test crash_matrix_recovery
   ```
   - **Exit-Code:** `0`
   - **Ergebnis:** 2 passed; 0 failed; 0 ignored (`t_crash_3_fm_3_disk_reserve_pending`, `t_crash_1_fm_matrix_recovery_step_progression`).
2. **`tests/deletion_proof_external_verify_rejects_tamper.rs` (WP-9):**
   ```bash
   cargo test -p contextra-erasure --test deletion_proof_external_verify_rejects_tamper
   ```
   - **Exit-Code:** `0`
   - **Ergebnis:** 5 passed; 0 failed; 0 ignored (`verify_external_rejects_legacy_hmac_proof`, `verify_external_rejects_tampered_deleted_keys_hash`, `verify_external_accepts_valid_proof`, `verify_external_rejects_tampered_timestamp`, `verify_external_rejects_wrong_public_key`).

Beide Testsuiten sind auf HEAD vorhanden und laufen vollständig grün.

---

## 4. Befunde und Fehleranalyse

1. **Formatierungsfehler (`cargo fmt`):**
   - Datei `crates/contextra-erasure/tests/golden_erasure_test.rs` weist Einrückungs-/Klammerungsunterschiede auf Zeilen 193 und 206 auf.
   - Da `tests/golden_erasure_test.rs` außerhalb des Schreibrechts für P0 liegt, wurde keine automatische Formatierung vorgenommen.
2. **Clippy-Fehler bei `--all-targets`:**
   - In `crates/contextra-erasure/tests/deletion_proof_v3_validation.rs` wird die mit `#[deprecated]` markierte Funktion `LayerCleanupProof::new_after_verified_empty` verwendet (Hinweis: „Umstellung auf DeletionProof::issue, siehe P-19“).
   - Clippy mit Flag `-D warnings` bricht deshalb auf diesem Target ab.
   - Der Produktions-Code (`--lib`) ist frei von Fehlern und Warnungen (`Exit 0`).
