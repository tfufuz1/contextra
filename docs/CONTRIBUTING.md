# Beitragen zu Contextra

<!-- doc-stamp: 2026-10-30 | Normativ: docs/SPEC.md Fassung 4.0, AGENTS.md, CLAUDE.md -->

Vielen Dank für Ihr Interesse. Dieses Dokument beschreibt, wie Änderungen in den Kern von Contextra eingebracht werden.

---

## 1. Voraussetzungen

- Rust-Toolchain laut `rust-toolchain.toml` (MSRV 1.89).
- Optional: `nix develop` mit `flake.nix` für eine reproduzierbare Umgebung.
- Kenntnis von `docs/SPEC.md` §3 (Regeln) und §4 (Ringe).

## 2. Ablauf

1. Aufgabe einem Arbeitspaket (WP) aus `docs/PLAN.md` zuordnen. Fehlt ein WP, bitte zuerst ein Issue mit Begründung anlegen.
2. Branch nach Schema `wp/<nummer>-<kurzbeschreibung>` anlegen, z. B. `wp/r0.3-engine-residuen`.
3. Änderung so klein wie möglich halten. Ein PR umfasst ein Arbeitspaket und höchstens einen Crate-Schnitt.
4. Lokal prüfen:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

5. Dokumentation im selben PR anpassen, wenn sich Ring, Abhängigkeiten, Befunde oder Scope ändern.
6. Vor dem PR auf den aktuellen Hauptzweig rebasen: `git fetch origin && git rebase origin/main`. Danach die Prüfungen erneut ausführen.

## 3. Verbindliche Regeln

- Kein `unwrap`, `expect`, `panic!` oder ungeprüftes Indexieren in Produktionscode (Clippy-Lints im Workspace).
- `unsafe` nur in `contextra-sys` und `contextra-simd`, jeder Block mit `// SAFETY:`.
- Zeit, Zufall und IDs nur über die Ports `Clock`, `Rng`, `IdGen`.
- `fsync`-Fehler werden mit `?` propagiert, nie mit `let _ =` verworfen.
- Keine Lint-Ausnahmen auf Modulebene ohne ADR.
- Keine Stubs oder Platzhalter-Implementierungen für Sicherheitsfunktionen.

## 4. Commit-Nachrichten

```
WP-R0.3(contextra-engine): Kurze Beschreibung

- Was geändert und warum (PLAN §, SPEC §)
- Prüfstatus vorher → nachher
```

## 5. Fragen zum Scope

Fragen, ob eine Funktion in v1 gehört, werden über einen Entwurf für einen ADR geklärt. Die Liste der Nicht-Ziele steht in `docs/SPEC.md` §2.3.

## 6. Lizenz

Beiträge werden unter der Doppellizenz MIT oder Apache-2.0 des Projekts aufgenommen.
