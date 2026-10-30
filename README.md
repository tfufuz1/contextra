<!-- doc-stamp: 2026-10-29 0796b40ed -->

# Contextra

> **Baukasten für lokale LLM- & RAG-Lösungen im europäischen Mittelstand (lokaler Betrieb ohne Cloud-Zwang)**

Contextra ist eine in Rust geschriebene Bibliothek (Single-Node, eingebettet), die als kompakter Baukasten aus entkoppelten Crates strukturiert ist. Sie ermöglicht lokales Wissens- und Speichermanagement ohne Cloud-Zwang, hybride Suche (Vektor + BM25) und Bausteine, die Datenschutzanforderungen technisch unterstützen.

---

## Reduktion 1.0

Das Repository wurde auf 14 Crates unter `crates/` reduziert (`contextra`, `contextra-crypto`, `contextra-db`, `contextra-engine`, `contextra-mvcc`, `contextra-ports`, `contextra-rank`, `contextra-simd`, `contextra-store`, `contextra-sys`, `contextra-testkit`, `contextra-text`, `contextra-types`, `contextra-vector`).
Entfernte Crates (`graph`, `cognition`, `adapt`, `kvcache`, `router`, `agent`, `infer-*`, `sandbox`, `py`, `license`, `core`, `ingest`, `mcp`, `wire`, `checkpoint`, `privacy`, `audit-export`, `avv-generator`) liegen in der Git-Historie (Tag `snapshot/pre-cut` bzw. Commit `eb10652f`) und werden nicht neu angelegt.
`cargo xtask` wurde entfernt; Validierungen erfolgen ausschließlich über standardisierte Cargo-Befehle (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`, `cargo deny`).

---

## Status der Löschkette (nicht Teil von 1.0)

Die erweiterte Löschkette befindet sich im Entwicklungsstatus **experimental** und ist **nicht Teil von 1.0**.

Aktuell erfasst die Grundstruktur der Schichten-Löschung:
- `LsmMemtable` (Memtable)
- `SsTableAllLevels` (SSTables)
- `WalAllSegments` (Write-Ahead Log)
- `TextIndex` (BM25 Text-Index)
- `HnswIndex` (Vektor-Index)

Erweiterte Löschkettenfunktionen (wie automatisierte Schlüsselvernichtung, Repertoire-Saga oder Recipient-Ledger) sind nicht Teil von 1.0.

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

Contextra ist nach einem unidirektionalen Ringmodell aufgebaut (**Ring 4 → Ring 3 → Ring 1 → Ring 0**):

- **Ring 4 (Fassade & Schnittstellen):** `contextra` (Composition Root)
- **Ring 3 (Orchestrierung & DB-Schicht):** `contextra-engine`, `contextra-db`
- **Ring 1 (Persistenz):** `contextra-store`
- **Ring 0 (Fundament & Kerne):** `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-crypto`, `contextra-vector`, `contextra-text`, `contextra-rank`, `contextra-testkit`
- **Unsafe-Inseln (Ring 0):** `contextra-simd`, `contextra-sys`

Die alleinige normative Quelle für Architektur und Invarianten ist [`docs/SPEC.md`](docs/SPEC.md).

---

## Verifizierung & Build-Befehle

Alle Validierungen erfolgen über standardisierte Cargo-Befehle (`cargo xtask` wurde entfernt).

### Grundlegende Befehle

```bash
# Code-Formatierung prüfen
cargo fmt --check

# Gesamten Workspace prüfen
cargo check --workspace

# Clippy Lints prüfen
cargo clippy --workspace --all-targets

# Alle Tests ausführen
cargo test --workspace

# Abhängigkeiten und Lizenzen prüfen
cargo deny check

# Spezifisches Crate isoliert testen
cargo test -p contextra-types
cargo test -p contextra-store
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
