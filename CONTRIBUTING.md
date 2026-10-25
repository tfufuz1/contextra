# Contributing to Contextra

Vielen Dank für dein Interesse, zu Contextra beizutragen! Dieses Dokument bündelt alle Richtlinien für Entwicklerinnen und Entwickler. Es basiert auf der normativen **Spezifikation 1.2** ([`docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`](docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md), §27.0, §28.0) sowie den Governance-Regeln (§28.0).

## 1. Governance & Rangfolge (§28.0)

Bei allen Beiträgen gilt die verbindliche Quell- und Entscheidungs-Rangfolge (§28.1):
1. **Spezifikation 1.2** (`docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`, §0, §1)
2. **Vetos & Prinzipien** (§4.1, §5.0, P1–P30)
3. **Crate-Inventar & Reifegrade** (`capabilities.toml`)
4. **Agenten-Betriebsanleitung** (`AGENTS.md`)
5. **Quellcode & grüne Gates**

Es ist verboten, Qualitäts-Gates, Lints oder Toolchain-Pins durch Ausnahmen abzuschwächen (REQ-QA-010).

## 2. Kurzeinstieg (Setup & Build)

### Voraussetzungen
- **Rust Toolchain:** Stable Rust (1.80+) mit `cargo` und `rustfmt`.
- **Just Task Runner:** `just` für Entwicklungsbefehle.

### Grundprinzipien & Invarianten
- **Single-Node & Air-Gap (REQ-ARCH-001):** Keine Laufzeit-Annahmen über Cloud-Dienste; lokale Inferenz via Candle (`[R]`, §14.1) oder Ollama (`[R]`, §14.2).
- **Zero-Panic-Doktrin (P7, §27.6):** Produktionscode darf im regulären Pfad niemals `panic!`, `.unwrap()` oder `.expect()` enthalten (`Result<T, E>` mit `ContextraError` nutzen).
- **Safe Rust & Ring-Schichtenreinheit (§6.0, §6.10):** `#![forbid(unsafe_code)]` in allen Crates außerhalb der 3 Unsafe-Inseln (`contextra-simd`, `contextra-sys`, `contextra-wire`; §6.7). Abhängigkeiten fließen strikt unidirektional von Ring 4/3 nach Ring 0 (REQ-ARCH-003).
- **Löschaussagen & Transparenz (INV-CLAIMS-1, REQ-DEL-006):** Aussagen zur Löschung folgen zwingend der Außenformulierung (§4.3, §20.12). Pauschale Werbeaussagen wie „100 % gelöscht“ oder „DSGVO-konform“ sind unzulässig.

### Local Build & Verification Commands
```bash
cargo check --workspace                 # Kompilierbarkeit prüfen
just check                              # Clippy-Warnungen als Fehler behandeln
cargo check --examples                  # Examples prüfen
just dag-check                          # Ring-Schichtenarchitektur (DAG) verifizieren (§6.10)
cargo xtask jules-preflight             # Preflight Verification Gate
```

## 3. Wie beigetragen wird (PR-Ablauf & Branching)

1. **Branching & Arbeitsablauf:** Erstelle einen Feature-Branch für deine Änderungen. Halte Diffs minimal und fokussiert (Minimal-Diff-Prinzip).
2. **Preflight Checks:** Führe vor jedem PR `cargo xtask jules-preflight` aus. Alle Gates müssen lokal grün sein (§27.6).
3. **Pull Request:** Reiche den PR ein. Stelle sicher, dass Commit-Nachrichten präzise sind und Invarianten (§5.2) nicht verletzt werden.
4. **Lizenzierung:** Beiträge stehen unter der [MIT License](LICENSE-MIT) und der [Apache License 2.0](LICENSE-APACHE).

## 4. Testregeln & Qualitätssicherung (§27.0)

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

## 5. Weiterführende Dokumente

- **Systemarchitektur & Ring-Modell:** [`ARCHITECTURE.md`](ARCHITECTURE.md)
- **Agenten-Betriebsanleitung:** [`AGENTS.md`](AGENTS.md)
- **Normative Spezifikation:** [`docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`](docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md)
