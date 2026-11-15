---
name: acta-gates
description: "Führt die Architektur- und Hygienegates (cargo xtask gates) sowie Test, Clippy und fmt für das Acta-Repo aus und ordnet Befunde ein. Verwenden nach jeder Änderung an Cargo.toml, Crate-Quelltext, rings.toml, deny.toml oder vor einem Commit."
---

# Gates und Qualitätsprüfung (Acta)

## Ablauf

Im Repo-Root in dieser Reihenfolge ausführen. Jeder Schritt muss mit Exit 0 enden, bevor der nächste zählt:

```
cd /root/acta
cargo check --workspace --all-targets --locked
cargo xtask gates
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
```

Schneller Testlauf: `cargo nextest run --workspace --locked`. Lieferketten-Gate (G-SUPPLY, noch nicht im xtask): `cargo deny check`, `cargo audit`, `cargo cyclonedx`, `cargo geiger`. Stand: advisories und sources gruen; `licenses` und `bans` rot wegen interner Pfad-Abhaengigkeiten und `LicenseRef-Proprietary` (Befund in `docs/UMGEBUNG.md`, Entscheidung offen, deny.toml nicht eigenmaechtig aendern). Mutationstests: `cargo mutants -p <crate>`. Abdeckung: `cargo llvm-cov -p <crate> --summary-only`.

Lange Läufe im Hintergrund starten, Log nach `/root/scratch/` schreiben und Exit-Code gesondert prüfen. `| tail` maskiert den Exit-Code, daher ihn nicht über eine Pipe ermitteln.

## Gates und was sie prüfen

| Gate | Prüfung | Typischer Befund und Fix |
|---|---|---|
| g1 | Level-Graph gegen `rings.toml` und `cargo metadata`, Dev-Abhängigkeiten ausgenommen | Abhängigkeit auf gleichem oder höherem Level. Abhängigkeit entfernen oder Crate verschieben (Ebene ändern nur mit ADR). |
| g-def | Alle Crates in `default-members` | Neuer Crate fehlt in `default-members` in `Cargo.toml`. |
| g-hub | Produktions-LOC gegen Budget | Budget überschritten. Code aufteilen oder ADR-Eintrag anlegen. Budget nicht still erhöhen. |
| g-unsafe | `unsafe` nur in `acta-sys`, jeder Block mit `// SAFETY:` | `unsafe` in anderem Crate: entfernen oder nach `acta-sys` verschieben. |
| g-notokio | kein `tokio` in Level 0 bis 7 | `tokio` aus Cargo.toml dieses Crates entfernen. |
| g-lint | Workspace-Lints in jedem Crate (`[lints] workspace = true`) | Abschnitt in `Cargo.toml` ergänzen. |
| g-word | verbotene Formulierungen aus SPEC 1.6 im Quelltext | Formulierung ersetzen, siehe Skill `acta-spec-check`. |

Gates, die noch nicht im `xtask` umgesetzt sind: G2, G3, G-CRY, G-FS, G-ERA, G-READ, G-NOSERDE-DISK, G-SUPPLY, G-ZUSAGE. Sie stehen als Lücke in `xtask/src/main.rs` (Kopfkommentar) und in der Wurzel-`CLAUDE.md`. Ihre Umsetzung nicht als erledigt melden.

## Hinweise zu Clippy

- Workspace-Lints verbieten `unwrap`, `expect`, `panic`, Indexierung und `let _ = result`. In Tests sind `unwrap` und Co. über `clippy.toml` erlaubt.
- `should_implement_trait`: Wenn eine Methode bewusst vom std-Operator abweicht (z. B. saturierend, `Option`-Rückgabe wie in `acta-base/src/fx.rs`), `#[allow(clippy::should_implement_trait, reason = "...")]` mit Begründung setzen. Kein stilles `allow`.
- `type_complexity`: Komplexe Typen als `type`-Alias führen (Beispiel: `Gate` in `xtask`).

## Hinweise zu fmt

`cargo fmt --all` ändert nur Workspace-Mitglieder. `archive/` ist nicht Teil des Workspace und wird von fmt nicht angefasst.

## Was zu berichten ist

Nur Befunde mit Dateipfad und Zeile nennen. Wenn ein Gate fehlschlägt, nicht mit einer Erklärung abschwächen. Das Gate ist die Quelle der Wahrheit, bis die SPEC geändert wird.
