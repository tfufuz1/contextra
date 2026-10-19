# contextra-avv-generator — Agenten-Hinweise

## Zweck
Generierung rechtlicher Auftragsverarbeitungsverträge (AVV) und Datenschutzvereinbarungen für Contextra Compliance Management.

## Ring & Invarianten
- Ring: 3
- Async erlaubt: ja
- Unsafe erlaubt: nein

## Abhängigkeiten (`may_depend_on`, siehe `capabilities.toml`)
- `contextra-types`

## Bekannte Fallstricke
- Vorlagen-Strings müssen UTF-8-konform sein und rechtliche Platzhalter korrekt maskieren.

- **INV-AVV-TOM-DEFAULTS**: `default_technical_measures()` muss zwingend alle produktinternen Sicherheitsgarantien (Deletion Proofs, Access Control, Isolation) enthalten.
  *Prüfung*: `cargo test -p contextra-avv-generator --lib`
- **INV-AVV-TEMPLATE-UTF8**: Erzeugte AVV-Vorlagen sind strikt UTF-8-konform und frei von unmaskierten Platzhaltern.
  *Prüfung*: `cargo test -p contextra-avv-generator --lib`
- **INV-FORBID-UNSAFE**: Strikte Einhaltung von `#![forbid(unsafe_code)]`.
  *Prüfung*: `cargo check -p contextra-avv-generator`

## 4. Verboten / Anti-Patterns

- **Keine ungültigen rechtlichen Platzhalter**: AVV-Generierung darf bei unvollständigen Pflichtfeldern im `AvvContext` nicht gerendert werden.
- **Keine Unsafe-Operationen**: Strikter Verzicht auf `unsafe` Codebausteine.

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- Der Generator arbeitet rein synchron und zustandslos.
- Keine Mutexes oder Locks vorhanden; vollkommen thread-sicher.

## 6. Verifikation

```bash
cargo test -p contextra-avv-generator
cargo check -p contextra-avv-generator
cargo xtask check-agents-integrity
cargo xtask doctrine-scan --crate contextra-avv-generator
cargo xtask check-unsafe-islands
```

## 7. Bekannte Lücken / SOLL

- Stand: 2026-10-13 (Governance Alignment)
- Ring-Klassifikation: `capabilities.toml` ist die verbindliche Single Source of Truth (P12) und führt `contextra-avv-generator` in Ring 0. Die Zuordnung als zustandsloses Ring-0-Blattmodul stellt sicher, dass AVV-Template-Generatoren und TOM-Garantien zyklusfrei aus allen System-Ebenen referenziert werden können.
