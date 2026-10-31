# Contextra

> Eingebettete Such- und Speicherbibliothek für lokale RAG-Anwendungen (Single-Node, ohne Cloud-Zwang)

Contextra ist eine Rust-Bibliothek. Sie kombiniert HNSW-Vektorsuche mit BM25-Volltextsuche, trennt Mandanten und verschlüsselt Daten im Ruhezustand.

## Status

**Stand 31.10.2026, Commit `3a0b11699`. Build nicht verifiziert, derzeit nicht grün.**

- Der Workspace enthält 35 Crate-Verzeichnisse. Eine frühere Fassung sprach von 14 Crates. Diese Angabe ist überholt.
- Das Workspace-Mitglied `xtask` hat kein Verzeichnis. Der Build bricht bereits bei der Auflösung des Manifests ab.
- Die Features `onnx` und `adaptive-candidate-pool-sizing` verweisen auf Crates, die nicht als Abhängigkeit deklariert sind.
- Scope-Entscheidung offen: [ADR 0009](docs/decisions/0009-scope-entscheidung-kern-oder-vollumfang.md).
- Die Löschkette ist `experimental`. Sie ist nicht als Garantie zu verstehen.

Vollständige Befunde mit Befehlen: [`docs/analysis/BEFUNDE_2026-10-31.md`](docs/analysis/BEFUNDE_2026-10-31.md).

## Dokumente

| Datei | Zweck |
|---|---|
| [`docs/SPEC.md`](docs/SPEC.md) | Normative Spezifikation (Fassung 4.1) |
| [`docs/PLAN.md`](docs/PLAN.md) | Umsetzungsplan, Phase K0 ist Voraussetzung für alles andere |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Crate-Inventar, Ringmodell, Abhängigkeiten, bekannte Defekte |
| [`docs/decisions/`](docs/decisions/) | Architekturentscheidungen |
| [`docs/CHANGELOG.md`](docs/CHANGELOG.md) | Änderungen |
| [`docs/spec/archiv/`](docs/spec/archiv/) | Frühere Fassungen 3.1 und 4.0, nicht bindend |

## Build

Toolchain ist in `rust-toolchain.toml` auf 1.89.0 festgelegt.

```bash
cargo build --workspace --locked
cargo test --workspace --locked
```

Die Prüfungen B1 bis B8 aus [SPEC §4](docs/SPEC.md#4-build-vertrag) sind Voraussetzung für jeden Merge auf `main`. Ausstehende Ergebnisse stehen in SPEC §4.

## Lizenz

MIT oder Apache-2.0, siehe `LICENSE-MIT` und `LICENSE-APACHE`.
