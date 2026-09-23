# Contributing to Contextra

Vielen Dank für dein Interesse, zu Contextra beizutragen! Dieses Dokument bündelt alle Richtlinien für menschliche Entwickler.

## 1. Kurzeinstieg (Setup & Build)

### Voraussetzungen
- **Rust Toolchain:** Stable Rust (1.80+) mit `cargo` und `rustfmt`.
- **Just Task Runner:** `just` für Entwicklungsbefehle.

### Grundprinzipien & Invarianten
- **Souveränität & Air-Gap:** Keine Laufzeit-Annahmen über Cloud-Dienste; lokale Inferenz via Ollama (`contextra-ollama`).
- **Zero-Panic-Gesetz:** Produktionscode darf niemals `panic!`, `.unwrap()` oder `.expect()` enthalten (`Result<T, E>` nutzen).
- **Safe Rust & Schichtenreinheit:** `#![forbid(unsafe_code)]` in Produktions-Crates. Abhängigkeiten fließen strikt unidirektional von Layer 4/3 nach Layer 0 (`contextra-core`). Minimal-Diff-Prinzip einhalten.

### Local Build & Verification Commands
```bash
cargo check --workspace --exclude contextra-tauri  # Kompilierbarkeit
just check                                         # Clippy-Warnungen als Fehler
cargo check --examples                             # Examples prüfen
just dag-check                                     # Schichtenarchitektur (DAG) verifizieren
cargo xtask jules-preflight                        # Preflight Verification Gate
```

## 2. Wie beigetragen wird (PR-Ablauf & Branching)

1. **Branching & Arbeitsablauf:** Erstelle einen Feature-Branch für deine Änderungen. Halte Diffs minimal und fokussiert (Minimal-Diff-Prinzip).
2. **Preflight Checks:** Führe vor jedem PR `cargo xtask jules-preflight` aus. Alle Gates müssen lokal grün sein.
3. **Pull Request:** Reiche den PR ein. Stelle sicher, dass die Commit-Nachrichten präzise sind und Invarianten nicht verletzt werden.
4. **Lizenzierung:** Beiträge stehen unter der [MIT License](LICENSE-MIT) und der [Apache License 2.0](LICENSE-APACHE).

## 3. Testregeln & Qualitätssicherung

### Testregeln & -kategorien
- **Anti-Mirroring-Prinzip:** Assertions müssen mit unabhängig ermittelten Referenzwerten arbeiten (kein Wiederholen der Formel im Test).
- **Pflichtabdeckung:** Jeder PR muss Tests für folgende Szenarien enthalten:
  1. *Happy Path*, 2. *Leere Eingaben*, 3. *Einzelne Elemente*, 4. *Grenzwerte* (`u64::MAX`, `f32::INFINITY`), 5. *Fehlerpfade* (Dimension-Mismatches, korrupte Bytes), 6. *Concurrency* (Sperren & Atomics).
- **Proptests & Mutation-Check:** Proptests für SIMD/Skalar-Gegenüberstellungen. Vor Freigabe prüfen: *Schlägt ein Test fehl, wenn ein Operator im Produktionscode umgekehrt wird?*
- **Hermetic Feature Gate Check:** `cargo check -p <crate> --no-default-features` prüft fehlende Default-Feature-Lecks.
- **Allowances:** `.unwrap()` und `.expect()` sind ausschließlich in Test-Code (`#[cfg(test)]`) erlaubt.

### Tests ausführen
```bash
cargo test --workspace --exclude contextra-tauri   # Alle Workspace-Tests
```
Für vertiefende Testregeln und Beispiele siehe [rules/testing.md](rules/testing.md) und [rules/test_quality.md](rules/test_quality.md).

## 4. Agenten-Regeln & Weiterführende Dokumente

- **Für AI-Agenten:** [`AGENTS.md`](AGENTS.md) definiert verbindliche Regeln, Invarianten und Werkzeuge für automatisierte Agenten.
- **Architektur:** Systemlayout und Ring-Modell sind in [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) beschrieben.
