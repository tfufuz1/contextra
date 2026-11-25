# AGENTS.md — contextra-avv-generator
> Ring 0 · experimental · Quelle: capabilities.toml · Spec: III.22 / K.4

## 1. Zweck
Generiert Vorlagen für Auftragsverarbeitungsverträge (AVV) gemäß Art. 28 DSGVO für Contextra.
Referenziert technische und organisatorische Maßnahmen (TOMs) wie kryptographische Löschbelege und Egress-Schutz.
Stellt rechtskonforme Textbausteine für Verantwortliche und Auftragsverarbeiter bereit.

## 2. Modul-Karte

| Datei | Verantwortung |
|---|---|
| `lib.rs` | `AvvContext`, `TechnicalMeasure`, public API `render_avv_markdown()` und `default_technical_measures()` |
| `template.rs` | Rendering-Engine für Art. 28 DSGVO AVV-Markdown-Dokumente |
| `error.rs` | Fehlerklasse `AvvGeneratorError` für Kontext- und Rendering-Fehler |

## 3. Invarianten

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

- Ring-Diskrepanz: `capabilities.toml` führt Crate in Ring 0, während Spec III.22 es als Ring 4 Utility beschreibt.
