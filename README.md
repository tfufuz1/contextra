<!-- doc-stamp: 2026-10-29 0796b40ed -->

# Contextra

> **Baukasten für lokale LLM- & RAG-Lösungen im europäischen Mittelstand (lokaler Betrieb ohne Cloud-Zwang)**

Contextra ist eine in Rust geschriebene Bibliothek (Single-Node, eingebettet), die als kompakter Baukasten aus entkoppelten Crates strukturiert ist. Sie ermöglicht lokales Wissens- und Speichermanagement ohne Cloud-Zwang, hybride Suche (Vektor + BM25) und Bausteine, die Datenschutzanforderungen technisch unterstützen.

---

## Status der Löschkette

Die Löschkette befindet sich im Entwicklungsstatus **experimental** ([`docs/SPEC.md`](docs/SPEC.md) §9.10).

Aktuell werden durch Layer-Scanner messtechnisch fünf Schichten erfasst (`crates/contextra-engine/src/contextra_impl/erasure.rs`):
- `LsmMemtable` (Memtable)
- `SsTableAllLevels` (SSTables)
- `WalAllSegments` (Write-Ahead Log)
- `TextIndex` (BM25 Text-Index)
- `HnswIndex` (Vektor-Index)

Noch nicht umgesetzt sind:
- Schlüsselvernichtung und Schlüsselhierarchie
- Lesesperre (`ErasedSet`)
- Löschsaga
- Backup-Guard
- Empfängerregister (`RecipientLedger`)

---

## Grenzen der Löschung

Folgende Bereiche sind nicht durch Contextra kontrollierbar und bilden explizite Ausschlüsse ([`docs/SPEC.md`](docs/SPEC.md) §9.9, C-12.20):
- Externe Sicherungen des Betreibers
- Datenträger-Firmware (Flash-Speicher, Wear-Leveling, Überprovisionierung)
- Prozessspeicher, Auslagerungsdateien (Swap) und Core-Dumps außerhalb des gehärteten Prozesses
- Kopien und übermittelte Daten bei Dritten
- Modellparameter
- Host-Prompt-Historien und Verläufe

Das Ergebnis ist ein technischer Befund über das lokale Datenverzeichnis, keine Rechtsauskunft und keine Zertifizierung. Weitere Details befinden sich in [`docs/SPEC.md`](docs/SPEC.md) §9.

---

## Architektur & Ringmodell

Contextra ist nach einem unidirektionalen Ringmodell aufgebaut (**Ring 4 → Ring 3 → Ring 2 → Ring 1 → Ring 0**):

- **Ring 4 (Fassade & Schnittstellen):** `contextra` (Composition Root), `contextra-mcp`, `contextra-py`, `contextra-license`
- **Ring 3 (Orchestrierung & Schutz):** `contextra-engine`, `contextra-db`, `contextra-privacy`, `contextra-cognition`, `contextra-router`, `contextra-agent`
- **Ring 2 (Inferenz & Isolation):** `contextra-infer-candle`, `contextra-infer-onnx`, `contextra-infer-ollama`, `contextra-sandbox`
- **Ring 1 (Persistenz):** `contextra-store`, `contextra-checkpoint`, `contextra-kvcache`
- **Ring 0 (Fundament & Kerne):** `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-crypto`, `contextra-vector`, `contextra-text`, `contextra-rank`, `contextra-graph`, `contextra-adapt`, `contextra-audit-export`, `contextra-avv-generator`
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
- [`docs/PLAN.md`](docs/PLAN.md) – Umsetzungsplan (Phase 0 bis Phase 8)
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) – Übersicht über Architektur, Ringe und Crate-Katalog
- [`docs/CONSTITUTION.md`](docs/CONSTITUTION.md) – Bindende Verfassung
- [`docs/VETOES.md`](docs/VETOES.md) – Bindende Vetos (VETO-F10, VETO-F02, VETO-OP03, VETO-NET1)
- [`capabilities.toml`](capabilities.toml) – Single Source of Truth für Ringe, Reifegrade und Crate-Abhängigkeiten

---

## Lizenz

Dual-lizenzierte Open-Source-Software unter **MIT** OR **Apache-2.0**.
