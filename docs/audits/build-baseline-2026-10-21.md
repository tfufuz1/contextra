# Reproduzierbarer Build-Bericht (Build Baseline)

**Datum:** 2026-10-21
**Scope:** Build-Baseline-Analyse am HEAD
**Autor:** Release- und Build-Engineer (Jules)

---

## 1. Umgebung & Toolchain

- **Commit-SHA:** `864519d324f281874ef847469e0ffafc2c366934`
- **rustc --version:** `rustc 1.89.0 (29483883e 2025-08-04)`
- **cargo --version:** `cargo 1.89.0 (c24e10642 2025-06-23)`
- **Toolchain-Quelle:** Pin via `rust-toolchain.toml`
  ```toml
  [toolchain]
  channel = "1.89.0"
  components = ["rustfmt", "clippy"]
  ```

---

## 2. Befehlsausführung und Exit-Codes

### 2.1 `cargo metadata --locked --format-version 1 > /dev/null`
- **Exit-Code:** `0`
- **Ausgabe:** (Keine Fehler, Metadaten erfolgreich aufgelöst)

---

### 2.2 `cargo check --workspace --all-targets`
- **Exit-Code:** `101`
- **Wörtliche Ausgabe (gekürzt):**
```text
    Checking contextra-sys v0.1.0 (/app/crates/contextra-sys)
error[E0432]: unresolved import `crypto_rng`
  --> crates/contextra-sys/src/lib.rs:32:9
   |
32 | pub use crypto_rng::OsCryptoRng;
   |         ^^^^^^^^^^ use of unresolved module or unlinked crate `crypto_rng`
   |
   = help: if you wanted to use a crate named `crypto_rng`, use `cargo add crypto_rng` to add it to your `Cargo.toml`

For more information about this error, try `rustc --explain E0432`.
error: could not compile `contextra-sys` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```

---

### 2.3 `cargo test --workspace --no-run`
- **Exit-Code:** `101`
- **Wörtliche Ausgabe (gekürzt):**
```text
warning: output filename collision.
The example target `collections` in package `contextra-db v0.1.0 (/app/crates/contextra-db)` has the same output filename as the example target `collections` in package `contextra v0.1.0 (/app/crates/contextra)`.
Colliding filename is: /app/target/debug/examples/collections
The targets should have unique names.
Consider changing their names to be unique or compiling them separately.
This may become a hard error in the future; see <https://github.com/rust-lang/cargo/issues/6313>.
...
   Compiling contextra-sys v0.1.0 (/app/crates/contextra-sys)
error[E0432]: unresolved import `crypto_rng`
  --> crates/contextra-sys/src/lib.rs:32:9
   |
32 | pub use crypto_rng::OsCryptoRng;
   |         ^^^^^^^^^^ use of unresolved module or unlinked crate `crypto_rng`
   |
   = help: if you wanted to use a crate named `crypto_rng`, use `cargo add crypto_rng` to add it to your `Cargo.toml`

For more information about this error, try `rustc --explain E0432`.
error: could not compile `contextra-sys` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```

---

## 3. Übersicht der fehlerhaften Crates

| Crate | Anzahl Fehler | Anzahl Warnungen | Erste Fehlermeldung (wörtlich) | Datei:Zeile | Fehlercode |
|---|---|---|---|---|---|
| `contextra-sys` | 1 | 0 | `error[E0432]: unresolved import crypto_rng` | `crates/contextra-sys/src/lib.rs:32` | `E0432` |

---

## 4. Zuordnung der Fehler zu Folge-Prompts

| Fehler / Befund | Betroffene Datei | Zuordnung Folge-Prompt | Bemerkung |
|---|---|---|---|
| `unresolved import crypto_rng` (E0432) | `crates/contextra-sys/src/lib.rs:32` | **Q1** | Bereinigung der Entropie-Foundation und Entfernung des verwaisten `crypto_rng`-Re-Exports in `contextra-sys`. |

---

## 5. Prüfung der Aussagen aus dem Gesamtregister am HEAD

### BLD-1: `contextra-ports/src/rng.rs` kompiliert
- **Urteil:** **widerlegt** (Aussage des Registers, dass `contextra-ports` nicht kompiliere, ist widerlegt; `contextra-ports` kompiliert vollständig)
- **Beleg:**
  Der Befehl `cargo check -p contextra-ports --all-targets` liefert Exit-Code `0`.
  Der Befehl `cargo test -p contextra-ports` führt 36 Unit-Tests und alle Integrationstests erfolgreich aus (`test result: ok. 36 passed; 0 failed`).
  Die vermuteten fehlenden Trait-Implementierungen (`EntropyRng` für `SystemRng`, `next_u64` etc.) sind in `crates/contextra-ports/src/rng.rs` vollständig implementiert und getestet.

---

### LOG-2: Doppelter Match-Arm `check-commit-diff-integrity` in `xtask/src/main.rs`
- **Urteil:** **bestätigt**
- **Beleg:**
  In `xtask/src/main.rs` befindet sich der Arm `"check-commit-diff-integrity" =>` an zwei Stellen:
  - Zeile 2148
  - Zeile 2437
  Beim Ausführen von `cargo check -p xtask` erzeugt der Compiler folgende Warnung:
  ```text
  warning: unreachable pattern
      --> xtask/src/main.rs:2437:9
       |
  2148 |         "check-commit-diff-integrity" => {
       |         ----------------------------- matches all the relevant values
  ...
  2437 |         "check-commit-diff-integrity" => {
       |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no value can reach this
  ```

---

### GOV-2: Gate-Funktionen ohne Aufrufer
- **Urteil:** **bestätigt**
- **Beleg:**
  Bei Ausführung von `cargo check -p xtask` meldet der Compiler unbelegte Gate-Funktionen (`dead_code` Warnungen):
  - `loop_guard_record` (`xtask/src/harness/loop_guard.rs:151:8`)
  - `loop_guard_check` (`xtask/src/harness/loop_guard.rs:166:8`)
  - `loop_guard_reset` (`xtask/src/harness/loop_guard.rs:237:8`)
  - `run_loop_guard` (`xtask/src/harness/loop_guard.rs:250:8`)
  - `run_scope_guard` (`xtask/src/agent_lifecycle/../harness/scope_guard.rs:93:8`)
  - `calculate_p95` (`xtask/src/check_bandit_latency_budget.rs:33:8`)
  - `check_veto_deadlines_from_content` (`xtask/src/veto_deadline_gate.rs:227:8`)
  - `get_crate_ring` (`xtask/src/check_ring_layering.rs:164:8`)

---

## 6. Zweiter Lauf nach Merge von Q1 und Ergebnisdifferenz

- **Status am HEAD:** Q1 ist auf dem aktuellen Branch/HEAD `864519d324f281874ef847469e0ffafc2c366934` noch nicht gemergt.
- **Ergebnisdifferenz-Analyse:**
  1. **Aktueller Lauf (vor Q1):** `cargo check --workspace --all-targets` schlägt mit Exit-Code 101 fehl, da `contextra-sys` die nicht existierende Modulreferenz `crypto_rng` importiert.
  2. **Prognostizierter Lauf (nach Merge von Q1):**
     Durch Entfernen der Zeile `pub use crypto_rng::OsCryptoRng;` in `crates/contextra-sys/src/lib.rs:32` entfällt der einzige Compiler-Blocker im gesamten Workspace.
     Isolierte Prüfungen aller weiteren Workspace-Crates bestätigen, dass sämtliche übrigen Pakete (einschließlich `contextra-ports`, `contextra-crypto`, `contextra-store`, `contextra-engine`, `contextra-cognition`, `contextra-mcp` und `xtask`) fehlerfrei kompilieren.
     Nach Merge von Q1 werden somit `cargo check --workspace --all-targets` sowie `cargo test --workspace --no-run` mit Exit-Code `0` durchlaufen.
