# AGENTS.md — `contextra-avv-generator`

> **Ring 0 laut `capabilities.toml` (Rolle: Extension)** · Klasse `[E]` (Extension) · Reifegrad `experimental` · Unsafe: nein · Rolle: Vorlagengenerator für Auftragsverarbeitungsverträge (Art. 28 DSGVO).
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-avv-generator --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `types` |
| Features | keine |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- Reine Dokumentgenerierung; keine Laufzeitabhängigkeit des Kerns. `capabilities.toml` nennt `privacy → avv-generator` (nicht in Cargo).
- Ring 0 widerspricht der Rolle (WP 1.1).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `template.rs`, `instructions.rs`, `assistance.rs`, `audit.rs`, `confidentiality.rs`, `data_subject_rights.rs`, `termination.rs` | Vertragsbausteine |

## 4. Verträge & Invarianten

1. Generierte Texte sind **Vorlagen, keine Rechtsberatung**; keine Zusage von Konformität.
2. Deterministische Ausgabe (gleiche Eingabe ⇒ gleicher Text).

## 5. Ports

Keine.

## 6. Nebenläufigkeit & Performance

- Rein funktional.

## 7. Anti-Patterns

- Rechtliche Zusicherungen im Text.
- Abhängigkeit des Kerns.

## 8. Tests & Messung

```bash
cargo test -p contextra-avv-generator --locked
cargo clippy -p contextra-avv-generator --all-targets -- -D warnings
```

- 3 Testdateien.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 1.1 | Ring korrigieren |
