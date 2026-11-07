# AGENTS.md — `contextra-audit-export`

> **Ring 0 laut `capabilities.toml` (Rolle: Extension)** · Klasse `[E]` (Extension) · Reifegrad `experimental` · Unsafe: nein · Rolle: Audit-Log-Export, DSGVO-Art.-30-Verarbeitungsverzeichnis, BSI-Mapping.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-audit-export --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `types` (laut `capabilities.toml` auch `ports`) |
| Features | keine |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- Compliance-Dokumentation ist **keine Kernlogik**; sie gehört in dieses Extension-Crate, nicht in Store/Index.
- 11 `DeletionProof`-Treffer: Verhältnis zu den Berichten aus `contextra-erasure` klären; keine zweite Berichtstypfamilie.
- Ring 0 widerspricht der Rolle (WP 1.1).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `bsi_mapping.rs`, `markdown_template.rs`, `testkit.rs`, `error.rs` | Mapping, Vorlagen, Testhilfen |

## 4. Verträge & Invarianten

1. Keine absoluten Rechts-/Garantieaussagen; Texte nur aus belegten Messwerten (INV-CLAIMS-1).
2. Keine Nutzinhalte im Export (INV-LOG-1).

## 5. Ports

Keine.

## 6. Nebenläufigkeit & Performance

- Rein funktional.

## 7. Anti-Patterns

- Eigene Beweis-/Berichtstypen.
- „DSGVO-konform“-Formulierungen.

## 8. Tests & Messung

```bash
cargo test -p contextra-audit-export --locked
cargo clippy -p contextra-audit-export --all-targets -- -D warnings
```

- 7 Testdateien.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 1.1 | Ring korrigieren |
| 3.9 | Berichte aus `erasure` konsumieren |
