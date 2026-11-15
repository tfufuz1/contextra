---
name: acta-crate-scaffold
description: "Legt einen neuen Acta-Crate oder eine neue interne Abhängigkeit an, gemäß Level-Modell, Hub-Regeln, Budgets und Workspace-Lints. Verwenden, wenn ein Crate in Cargo.toml aufgenommen, verschoben oder ein Vertrags-Crate angelegt werden soll."
---

# Neuen Crate anlegen (Acta)

Ein Crate ist nur zulässig, wenn er in SPEC 3.3 vorgesehen ist. Andernfalls zuerst einen ADR-Eintrag vorschlagen. Ein Sammel-Crate für „Typen“ oder „Ports“ ist verboten (HR-1, 3.4).

## 1. Vorprüfung

- Ist der Crate in der Tabelle SPEC 3.3 aufgeführt? Level, Budget und erlaubte Abhängigkeiten notieren.
- Ist er ein Vertrags-Crate? Dann HR-2 und HR-3 prüfen: Der Trait gehört dem Crate, der ihn braucht, wenn der Erbringer höher liegt.
- Liegen alle geplanten Abhängigkeiten in der Whitelist (SPEC 3.8)?

## 2. Verzeichnis und Manifest

```
crates/<name>/
  Cargo.toml
  CLAUDE.md
  src/lib.rs   (oder src/main.rs für Binaries)
```

`Cargo.toml` nach dem Muster bestehender Crates aufbauen:

```toml
[package]
name = "acta-<name>"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
publish.workspace = true
description = "Acta Level <N>"

[features]
default = []

[dependencies]
acta-base.workspace = true
thiserror.workspace = true

[dev-dependencies]

[lints]
workspace = true
```

Pflichtpunkte:
- `[lints] workspace = true` (G-LINT).
- `default = []` (HR-5). Features sind additiv und haben keine Rückkante nach oben.
- Keine `tokio`-Abhängigkeit unterhalb Level 8 (G-NOTOKIO).
- Abhängigkeiten nur auf niedrigerem Level (G1).

## 3. Workspace einbinden

In der Wurzel-`Cargo.toml`:
1. Pfad zu `members` hinzufügen.
2. Pfad zu `default-members` hinzufügen (G-DEF).
3. Eintrag unter `[workspace.dependencies]` als `acta-<name> = { path = "crates/acta-<name>" }` anlegen.

In `rings.toml`:
1. Level unter `[levels]` eintragen.
2. Budget unter `[budgets]` eintragen (Pflicht für G-HUB, sonst wird der Crate nicht gezählt).
3. Bei isoliertem Crate unter `[isolated]` eintragen.

## 4. Quelltext

- `src/lib.rs` mit Kopfkommentar: Level, Budget, SPEC-Kapitel.
- Einen Fehlertyp mit `thiserror`, `#[non_exhaustive]`, implementiert `ClassifiedError` aus `acta-base`.
- Öffentliche Items dokumentieren (Lint `missing_docs`).
- Keine Alias-Re-Exports (HR-6).

## 5. CLAUDE.md des Crates

Die Vorlage aus den bestehenden Crates übernehmen. Level, Budget, Aufgabe (aus SPEC 3.3), erlaubte und tatsächliche Abhängigkeiten sowie Archivstatus eintragen.

## 6. Prüfung

```
cd /root/acta
cargo check --workspace --all-targets --locked
cargo xtask gates
cargo clippy -p acta-<name> --all-targets --locked -- -D warnings
cargo fmt --all -- --check
```

Alle Befehle müssen mit Exit 0 enden.

## Bei Abhängigkeitsproblemen

- G1 meldet Level-Verletzung: Die Abhängigkeit ist falsch. Nicht den Level in `rings.toml` ändern, um den Befund zu umgehen. Stattdessen den Crate verschieben oder die Schnittstelle anders schneiden.
- G-HUB meldet Budget-Überschreitung: aufteilen oder ADR anlegen.
