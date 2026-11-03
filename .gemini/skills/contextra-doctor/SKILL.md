---
name: contextra-doctor
description: >-
  Use this skill to diagnose Contextra workspace and build problems: manifest consistency, the set of workspace members,
  compiler errors, lockfile drift, Vfs usage, and forbidden dependencies, according to docs/SPEC.md.
---

# Workspace und Build diagnostizieren

Maßgeblich: [`docs/SPEC.md`](../../../../docs/SPEC.md), Abschnitte 5.1 bis 5.3, D-54, D-93, D-94 und F-32 bis F-36.

## 1. Workspace-Integrität

```bash
cargo metadata --no-deps --format-version 1 > /dev/null
```

- Exit-Code 0: Manifeste und Pfade sind konsistent.
- Exit-Code ungleich 0: ein Workspace-Mitglied oder eine Pfadabhängigkeit fehlt.

**Soll-Workspace (SPEC 5.1):** 14 Produktionscrates (`contextra-types`, `-ports`, `-sys`, `-simd`, `-durable-fs`, `-crypto`, `-mvcc`, `-store`, `-text`, `-vector`, `-rank`, `-erasure`, `-engine`, `contextra`) sowie die Dev-Mitglieder `contextra-testkit`, `contextra-lab`, `ctx-verify` und `contextra-arch-tests`. Im Ist-Zustand fehlen `contextra-erasure`, `contextra-lab` und `ctx-verify` als Mitglieder.

Keine Crates außerhalb dieser Liste dürfen in `members` oder `dependencies` stehen (D-84).

## 2. Runbooks

### Runbook 1: Lockfile veraltet

- **Befund:** `cargo check --locked` bricht ab (F-32).
- **Behebung:** `cargo check --workspace` ohne `--locked` aktualisiert `Cargo.lock`. Die Änderung gehört in denselben Commit wie die Manifest-Änderung. Danach mit `--locked` prüfen (D-93).

### Runbook 2: `unsafe fn` aufgerufen

- **Befund:** `unsafe fn contextra_sys::mmap_readonly` wird in `store` und `vector` aufgerufen (F-32).
- **Behebung:** Keine `mmap`-Lesezugriffe. Die Lesezugriffe laufen über positionelles Lesen des Ports `Vfs` (D-94). Aufrufer, die `forbid(unsafe_code)` setzen, rufen keine `unsafe fn` auf.

### Runbook 3: Feature-Gates ohne Deklaration

- **Befund:** `cfg(feature = "…")` ohne Eintrag in `[features]`; `#[allow(unexpected_cfgs)]` maskiert das (F-06).
- **Behebung:** `allow` entfernen. Die Funktion entfällt, wenn sie zu D-84 gehört, oder wird als verbindlicher Teil des Produktpfads integriert.

### Runbook 4: `std::fs` oder `tokio::fs` außerhalb von `durable-fs`

- **Befund:** direkter Dateizugriff in `store` (169 Stellen) und `vector` (51 Stellen), Stand F-29.
- **Behebung:** nur der Port `Vfs` aus `contextra-ports` (D-04, T-LINT-2).

### Runbook 5: Netzwerk-Crate im Produktionsbaum

- **Befund:** `tokio` mit `features = ["full"]` zieht `mio` und `socket2` in den Baum (F-16).
- **Behebung:** nur `rt-multi-thread`, `sync`, `time`, `macros` (D-54). Prüfung mit `cargo tree -e normal`.

### Runbook 6: Abhängigkeitskante verletzt die Matrix

- **Befund:** eine Kante fehlt in SPEC 5.3 (z. B. `vector` → `sys`, `vector` → `mvcc`, F-35).
- **Behebung:** Kante entfernen oder über einen Port aus `ports` lösen (D-88). Prüfung durch `contextra-arch-tests`.

## 3. Prüfbefehle

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo test -p contextra-arch-tests --locked
```
