# Findings: STORE-MOD-H

Beobachtungen und externe Befunde außerhalb des exklusiven Eigentums von Agent H:

## 1. Cargo.toml Blocker (Agent G Eigentum)
- **Datei:** `crates/contextra-store/Cargo.toml:17`
- **Beobachtung:**
  In `[features]` ist definiert:
  ```toml
  workspace = ["dep:contextra-types", "dep:contextra-ports"]
  ```
  Unter `[dependencies]` sind jedoch `contextra-types` und `contextra-ports` als reguläre Abhängigkeiten ohne `optional = true` deklariert:
  ```toml
  contextra-types = { workspace = true }
  contextra-ports = { workspace = true }
  ```
- **Folge:** Cargo verweigert das Laden des Manifests mit der Fehlermeldung:
  `feature 'workspace' includes 'dep:contextra-types', but 'contextra-types' is not an optional dependency`.
- **Vorschlag / Patch für Agent G:**
  In `crates/contextra-store/Cargo.toml` die beiden Zeilen wie folgt anpassen:
  ```toml
  contextra-types = { workspace = true, optional = true }
  contextra-ports = { workspace = true, optional = true }
  ```

## 2. Fehlerverschlucken in `compat/scanner/factory.rs`
- **Datei:** `crates/contextra-store/src/compat/scanner/factory.rs:37` (vormals `compat/scanner/mod.rs`)
- **Beobachtung:**
  Beim Aufruf von `scan_view()` via `runtime.block_on(...)` wird im Fehlerfall ein leeres `Vec::new()` zurückgegeben:
  ```rust
  let view = match runtime.block_on(async { Ok(storage.scan_view().await) }) {
      Ok(v) => v,
      Err(_) => return Vec::new(),
  };
  ```
  Der Fehler wird still verworfen. Da dies bestehendes Laufzeitverhalten ist, wurde es unverändert beibehalten (Nicht-Ziel: keine Verhaltensänderung).
