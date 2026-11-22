// FILE-CONTEXT
// ZWECK: 10 typische Markdown-Fixtures für ProseChunker Paritäts- und Vertragstests.
// STAND: TS:2026-10-31T00:00:00Z

pub struct Fixture {
    pub name: &'static str,
    pub content: &'static str,
}

pub fn get_prose_fixtures() -> Vec<Fixture> {
    vec![
        Fixture {
            name: "doc1_headings_and_paras",
            content: r#"# Hauptüberschrift
Dies ist der Einführungstext des Dokuments.

## Erste Sektion
Hier steht der erste Absatz der ersten Sektion.

### Untersektion 1.1
Ein kleinerer Absatz in der Untersektion."#,
        },
        Fixture {
            name: "doc2_fenced_code_blocks",
            content: r#"# Code-Beispiel

Hier ist ein Code-Block:

```rust
// # Dies ist eine Überschrift im Code, aber KEIN Markdown-Heading!
fn main() {
    println!("Hello, World!");
}
```

Und hier geht der Text weiter."#,
        },
        Fixture {
            name: "doc3_gfm_tables",
            content: r#"# Tabellen-Dokument

| ID | Name | Rolle |
|---|---|---|
| 1 | Alice | Admin |
| 2 | Bob | User |
| 3 | Charlie | Guest |

Ein Absatz nach der Tabelle."#,
        },
        Fixture {
            name: "doc4_german_abbreviations",
            content: r#"# Deutsche Texte mit Umlauten

Das ist z. B. ein wichtiger Hinweis. Dr. Müller hat bzw. Prof. Schmidt kann die Unterlagen prüfen.

Abläufe u. a. bezüglich Straße und Übermut sind wunderschön!! S. 42 ff."#,
        },
        Fixture {
            name: "doc5_nested_lists",
            content: r#"# Listen-Beispiel

- Erster Hauptpunkt
  - Unterpunkt A
  - Unterpunkt B
- Zweiter Hauptpunkt
  1. Erster Schritt
  2. Zweiter Schritt"#,
        },
        Fixture {
            name: "doc6_front_matter",
            content: r#"---
title: Systemarchitektur
author: Ingestion Team
version: 1.0
---

# Einführung
Das ist der Dokumentenrumpf nach der Front-Matter."#,
        },
        Fixture {
            name: "doc7_oversized_paragraph",
            content: r#"# Langer Absatz

Dies ist ein sehr langer Absatz mit vielen Informationen. Er soll die Aufspaltungslogik testen. Jeder Satz enthält spezifische Fakten über das System. Die Verarbeitung erfolgt schrittweise und deterministisch. Alle UTF-8-Codepunkte bleiben erhalten. Umlautüberprüfung ist ebenfalls aktiv. Das System skaliert linear mit der Dokumentgröße."#,
        },
        Fixture {
            name: "doc8_headings_only",
            content: r#"# H1 Titel
## H2 Sektion 1
### H3 Untersektion
# H1 Fazit"#,
        },
        Fixture {
            name: "doc9_emoji_unicode",
            content: r#"# Unicode & Emoji Test

Rust ist toll 🦀! Raketenstarts sind schnell 🚀.
Zwischenraum und Emoji-Sequenzen 👨‍👩‍👧‍👦 funktionieren tadellos."#,
        },
        Fixture {
            name: "doc10_footnotes_and_quotes",
            content: r#"# Fußnoten & Zitate

> Dies ist ein bedeutungsvolles Zitat aus der Spezifikation.

Ein Absatz mit Fußnote[^1] im Text.

[^1]: Dies ist die Erklärung der Fußnote."#,
        },
    ]
}
