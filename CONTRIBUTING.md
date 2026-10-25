# Contributing to Contextra

Vielen Dank für dein Interesse, zu Contextra beizutragen! Dieses Dokument bündelt alle Richtlinien für Entwicklerinnen und Entwickler. Es basiert auf der Normativen Gesamtspezifikation (Fassung 1.2; §27, §28).

## 1. Kurzeinstieg (Setup & Build)

### Voraussetzungen
- **Rust Toolchain:** Stable Rust (1.80+) mit `cargo` und `rustfmt`.
- **Just Task Runner:** `just` für Entwicklungsbefehle.

### Grundprinzipien & Invarianten
- **Single-Node & Air-Gap (§1, §4.1):** Keine Laufzeit-Annahmen über Cloud-Dienste; lokale Inferenz via Candle (`[R]`, §15) oder Ollama (`[R]`, §15).
- **Zero-Panic-Doktrin (§3, §27.6):** Produktionscode darf im regulären Pfad niemals `panic!`, `.unwrap()` oder `.expect()` enthalten (`Result<T, E>` mit `ContextraError` nutzen).
- **Safe Rust & Ring-Schichtenreinheit (§6.0, §6.10):** `#![forbid(unsafe_code)]` in allen Crates außerhalb der 3 Unsafe-Inseln (`contextra-simd`, `contextra-sys`, `contextra-wire`; §6.7). Abhängigkeiten fließen strikt unidirektional von Ring 4/3 nach Ring 0 (§6.10).
- **Governance & Entwicklungsprozess (§28):** Änderungen an Vetos oder Invarianten benötigen eine ADR (`docs/decisions/`). Merges laufen serialisiert über eine Merge-Queue, die das Merge-Ergebnis testet (§28.2).
- **Löschaussagen & Transparenz (INV-CLAIMS-1):** Aussagen zur Löschung folgen zwingend der Außenformulierung (§4.3, §20.12). Werbe- oder Pauschalaussagen zur Löschwirkung oder Rechtskonformität sind unzulässig (§4.3, §20.12).

### Local Build & Verification Commands
```bash
cargo check --workspace                 # Kompilierbarkeit prüfen
just check                              # Clippy-Warnungen als Fehler behandeln
cargo check --examples                  # Examples prüfen
just dag-check                          # Ring-Schichtenarchitektur (DAG) verifizieren (§6.10)
cargo xtask jules-preflight             # Preflight Verification Gate
```

## 2. Wie beigetragen wird (PR-Ablauf & Branching)

1. **Branching & Arbeitsablauf:** Erstelle einen Feature-Branch für deine Änderungen. Halte Diffs minimal und fokussiert (Minimal-Diff-Prinzip).
2. **Preflight Checks:** Führe vor jedem PR `cargo xtask jules-preflight` aus. Alle Gates müssen lokal grün sein (§27.6).
3. **Pull Request:** Reiche den PR ein. Stelle sicher, dass Commit-Nachrichten präzise sind und Invarianten (§5) nicht verletzt werden.
4. **Lizenzierung:** Beiträge stehen unter der [MIT License](LICENSE-MIT) und der [Apache License 2.0](LICENSE-APACHE).

## 3. Testregeln & Qualitätssicherung (§27)

### Testregeln & -kategorien (§27.1–§27.5)
- **Anti-Mirroring-Prinzip:** Assertions müssen mit unabhängig ermittelten Referenzwerten arbeiten (kein Wiederholen der Formel im Test).
- **Pflichtabdeckung:** Jeder PR muss Tests für folgende Szenarien enthalten:
  1. *Happy Path*, 2. *Leere Eingaben*, 3. *Einzelne Elemente*, 4. *Grenzwerte* (`u64::MAX`, `f32::INFINITY`), 5. *Fehlerpfade* (Dimension-Mismatches, korrupte Bytes), 6. *Concurrency* (Sperren & Atomics; INV-LOCK-ORDER-1).
- **Proptests & Mutation-Check:** Proptests für SIMD/Skalar-Gegenüberstellungen. Vor Freigabe prüfen: *Schlägt ein Test fehl, wenn ein Operator im Produktionscode umgekehrt wird?*
- **Hermetic Feature Gate Check:** `cargo check -p <crate> --no-default-features` prüft fehlende Default-Feature-Lecks.
- **Allowances:** `.unwrap()` und `.expect()` sind ausschließlich in Test-Code (`#[cfg(test)]`) erlaubt.

### Tests ausführen
```bash
cargo test --workspace                  # Alle Workspace-Tests
```

## 4. Weiterführende Dokumente

- **Systemarchitektur & Ring-Modell:** [`ARCHITECTURE.md`](ARCHITECTURE.md)
- **Agenten-Betriebsanleitung:** [`AGENTS.md`](AGENTS.md)
- **Governance & Entwicklungsprozess:** [`docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`](docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md) (§28)
- **Normative Spezifikation:** [`docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`](docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md)
