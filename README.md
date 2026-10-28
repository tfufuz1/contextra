# Contextra

> **Baukasten für lokale, datenschutzkonforme LLM- & RAG-Lösungen im europäischen Mittelstand**

Contextra ist eine in Rust geschriebene Bibliothek (Single-Node, eingebettet), die als kompakter Baukasten aus entkoppelten Crates strukturiert ist. Sie ermöglicht lokales Wissens- und Speichermanagement ohne Cloud-Zwang, hybride Suche (Vektor + BM25) und eine technisch begründete, messbare Löschkette nach DSGVO.

---

## Architektur & Ringmodell

Contextra ist nach einem unidirektionalen Ringmodell aufgebaut (**Ring 4 → Ring 3 → Ring 2 → Ring 1 → Ring 0**):

- **Ring 4 (Fassade & Schnittstellen):** `contextra` (Composition Root), `contextra-mcp`, `contextra-py`, `contextra-license`
- **Ring 3 (Orchestrierung & Schutz):** `contextra-engine`, `contextra-db`, `contextra-privacy`, `contextra-cognition`, `contextra-router`, `contextra-agent`
- **Ring 2 (Inferenz & Isolation):** `contextra-infer-candle`, `contextra-infer-onnx`, `contextra-infer-ollama`, `contextra-sandbox`
- **Ring 1 (Persistenz):** `contextra-store`, `contextra-checkpoint`, `contextra-kvcache`
- **Ring 0 (Fundament & Kerne):** `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-crypto`, `contextra-vector`, `contextra-text`, `contextra-rank`, `contextra-adapt`, `contextra-audit-export`, `contextra-avv-generator`
- **Unsafe-Inseln (Ring 0):** `contextra-simd`, `contextra-sys`, `contextra-wire`

Die alleinige normative Quelle für Architektur und Invarianten ist [`docs/SPEC.md`](docs/SPEC.md).

---

## Verifizierung & Build-Befehle

Alle Validierungen und Quality Gates werden über Cargo und `cargo xtask` ausgeführt.

### Grundlegende Befehle

```bash
# Gesamten Workspace prüfen
cargo check --workspace --locked

# Alle Tests ausführen
cargo test --workspace --locked

# Clippy Lints prüfen
cargo clippy --workspace --all-targets --locked

# Spezifisches Crate isoliert testen
cargo test -p contextra-types --locked
cargo test -p contextra-store --locked
```

### Quality Gates (`cargo xtask`)

```bash
# Alle aktiven Qualitäts-Gates prüfen
cargo run -p xtask -- check

# Einzelnes Gate ausführen (z. B. ring-dag)
cargo run -p xtask -- gate check --gate ring-dag

# Übersicht aller konfigurierten Gates anzeigen
cargo run -p xtask -- --list-gates
```

---

## Dokumentation & Spezifikation

- [`docs/SPEC.md`](docs/SPEC.md) – Normative Gesamtspezifikation (Soll-Zustand, Invarianten, Baukasten-Modell)
- [`docs/PLAN.md`](PLAN.md) – Umsetzungsplan (Phase 0 bis Phase 8)
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) – Übersicht über Architektur, Ringe und Crate-Katalog
- [`docs/CONSTITUTION.md`](docs/CONSTITUTION.md) – Bindende Verfassung
- [`docs/VETOES.md`](docs/VETOES.md) – Bindende Vetos (VETO-F10, VETO-F02, VETO-OP03, VETO-NET1)
- [`capabilities.toml`](capabilities.toml) – Single Source of Truth für Ringe, Reifegrade und Crate-Abhängigkeiten

---

## Lizenz

Dual-lizenzierte Open-Source-Software unter **MIT** OR **Apache-2.0**.
