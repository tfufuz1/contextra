# Contextra

> **Eingebettete Such- und Speicherbibliothek für lokale RAG-Anwendungen (Single-Node, ohne Cloud-Zwang)**

<!-- doc-stamp: 2026-10-30 -->

Contextra ist eine Rust-Bibliothek für Anwendungen, die Wissen lokal speichern und durchsuchen sollen. Sie kombiniert HNSW-Vektorsuche und BM25-Textsuche, trennt Mandanten strikt und verschlüsselt Daten im Ruhezustand.

---

## Status

**Stand 30.10.2026: Reduktionsfassung, Build derzeit nicht grün.**

Der Repository-Stand wurde am 30.10.2026 auf einen Kern von 14 Crates reduziert. Inferenz, Agenten, MCP, Python-Bindings, Ingestion und die Löschkette sind entfernt. Die Reduktion hinterließ Verweise auf entfernte Crates im Quelltext, außerdem fehlt das Workspace-Mitglied `xtask`. Die Behebung ist in [`docs/PLAN.md`](docs/PLAN.md) Phase R0 beschrieben.

Bis R0 abgeschlossen ist, lassen sich die Beispiele unten nicht kompilieren.

---

## Funktionsumfang (v1)

- Collections und Dokumente mit Mandantentrennung
- Hybride Suche: HNSW-Vektorsuche, BM25-Textsuche, Fusion der Ranglisten mit Kalibrierung
- Deutsche Textverarbeitung: Komposita-Zerlegung, Stoppwörter, Domänenwörterbücher für Recht und Medizin
- MVCC-Transaktionen mit Serializable-Snapshot-Isolation
- LSM-Speicher mit Write-Ahead-Log und Kompaktion
- Verschlüsselung im Ruhezustand, in jedem Build aktiv
- Logisches Löschen über Tombstones

**Nicht enthalten in v1:** Verteilter Betrieb, eingebettete Sprachmodelle, Agenten-Laufzeit, MCP-Server, Python-Bindings, Dokumenten-Ingestion, Compliance-Nachweise. Vollständige Liste: [`docs/SPEC.md`](docs/SPEC.md) §2.3.

---

## Beispiel (Zielzustand nach Phase R0)

```rust
use contextra::builder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Dimension des Embedding-Vektors festlegen und Speicherpfad angeben
    let _db = builder(768).with_storage_path("./daten").build().await?;

    // Weitere Nutzung über Collection und Hybrid-Suche:
    // siehe crates/contextra-db/examples/ (quickstart, hybrid_search, collections)
    Ok(())
}
```

Die Beispiele unter `crates/contextra-db/examples/` zeigen den Einstieg in Collections und Suche.

---

## Crates

| Crate | Ring | Aufgabe |
|---|---|---|
| `contextra` | 4 | Öffentliche Fassade (Composition Root) |
| `contextra-db` | 3 | Anwendungs-API: `prelude` und `advanced` |
| `contextra-engine` | 2 | Collections, Hybridsuche, Transaktionen |
| `contextra-store` | 1 | LSM-Speicher, WAL, SSTables |
| `contextra-types` | 0 | Typen, IDs, Fehler |
| `contextra-ports` | 0 | Trait-Verträge |
| `contextra-sys` | 0 | Systemnahe Schicht (Unsafe-Insel) |
| `contextra-simd` | 0 | SIMD-Distanzen (Unsafe-Insel) |
| `contextra-mvcc` | 0 | Transaktionsisolation |
| `contextra-crypto` | 0 | Verschlüsselung, Schlüsselableitung |
| `contextra-vector` | 0 | HNSW-Vektorindex |
| `contextra-text` | 0 | BM25 und Textverarbeitung |
| `contextra-rank` | 0 | Fusion und Kalibrierung |
| `contextra-testkit` | Tooling | Testwerkzeuge |
| `contextra-bench` | Tooling | Benchmarks (`benchmarks/`) |

Architektur mit Abhängigkeitsgraph: [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

---

## Grenzen der Löschung

Logisches Löschen über Tombstones ist in v1 umgesetzt. Die physische Entfernung erfolgt bei der Kompaktion, der Zeitpunkt ist nicht zugesichert. Contextra liefert **keinen Löschbeweis** und **keine Compliance-Zusage**. Externe Sicherungen, Datenträger-Firmware, Auslagerungsspeicher und Kopien bei Dritten liegen außerhalb des Einflussbereichs der Bibliothek. Details: [`docs/SPEC.md`](docs/SPEC.md) §9.

---

## Dokumentation

- [`docs/SPEC.md`](docs/SPEC.md): Gesamtspezifikation 4.0 (normativ)
- [`docs/PLAN.md`](docs/PLAN.md): Umsetzungsplan 4.0 (Phasen R0 bis R6)
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md): Architekturübersicht (nicht normativ)
- [`docs/CONSTITUTION.md`](docs/CONSTITUTION.md): Verfassung
- [`docs/VETOES.md`](docs/VETOES.md): Vetos (VETO-F02, VETO-F10, VETO-OP03)
- [`docs/CHANGELOG.md`](docs/CHANGELOG.md): Änderungsverlauf
- [`AGENTS.md`](AGENTS.md) und [`CLAUDE.md`](CLAUDE.md): Arbeitsregeln für Agenten

---

## Entwicklung

Befehle nach Abschluss von Phase R0 (Ziel):

```bash
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Der frühere Befehl `cargo xtask` existiert nicht mehr. Die Gate-Strategie ist in `docs/SPEC.md` §12 und D-R2 beschrieben.

---

## Lizenz

Dual lizenziert unter **MIT** oder **Apache-2.0** (siehe `LICENSE-MIT` und `LICENSE-APACHE`).
