//! LEGACY-REFERENCE: Fester Korpus (≥ 200 Dokumente + Randfälle) zur Chunker-Divergenz- und Snapshot-Prüfung
//! ZWECK: Eingefrorene Dokumente und Snapshot-Kennzahlen für Differenztests (S-61).

/// Erzeugt die 200 prozeduralen Dokumente mit festem deterministischen Seed plus alle geforderten Randfälle.
pub fn generate_frozen_corpus() -> Vec<(String, String)> {
    let mut corpus = Vec::with_capacity(250);

    // 1. Feste Randfälle:
    // (a) Leer
    corpus.push(("edge_empty".to_string(), "".to_string()));

    // (b) Nur Whitespace
    corpus.push(("edge_whitespace_only".to_string(), "   \n\t\n   \r\n   ".to_string()));

    // (c) CRLF
    corpus.push((
        "edge_crlf".to_string(),
        "# Header CRLF\r\n\r\nLine 1\r\nLine 2\r\n\r\n## Subheader\r\nParagraph text.\r\n".to_string(),
    ));

    // (d) Sehr lange Zeile (10.000 Zeichen ohne Leerzeichen)
    corpus.push((
        "edge_long_line".to_string(),
        format!("# Long Line\n\n{}\n", "a".repeat(10_000)),
    ));

    // (e) Tabellen
    corpus.push((
        "edge_table".to_string(),
        "# Table Document\n\n| Col A | Col B | Col C |\n|---|---|---|\n| val1 | val2 | val3 |\n| 10 | 20 | 30 |\n\nTrailing paragraph.\n".to_string(),
    ));

    // (f) Codezäune (Backticks und Tilden, verschachtelt, mit Sprachbezeichner)
    corpus.push((
        "edge_code_fences".to_string(),
        "# Code Document\n\n```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\n\n~~~python\ndef foo():\n    pass\n~~~\n\n````markdown\nNested fence ```rust\nfn inner() {}\n```\n````\n".to_string(),
    ));

    // (g) Verschachtelte Überschriften H1–H6
    corpus.push((
        "edge_nested_headings".to_string(),
        "# Heading 1\nText 1\n## Heading 2\nText 2\n### Heading 3\nText 3\n#### Heading 4\nText 4\n##### Heading 5\nText 5\n###### Heading 6\nText 6\n".to_string(),
    ));

    // (h) Umlaute und Unicode
    corpus.push((
        "edge_unicode_umlauts".to_string(),
        "# Größenüberprüfung & Äpfel\n\nÜbermäßige Wärme führt zu Verzögerungen. Grünes Öl und süße Früchte.\n\n日本語のテキストやкириллицаも安全に処理される。\n".to_string(),
    ));

    // (i) Emoji
    corpus.push((
        "edge_emoji".to_string(),
        "# Emoji Safety 🚀\n\nMulti-byte emojis: 🦀 🦀 🦀 and flags: 🇩🇪 🇺🇸 🇯🇵.\nFamily sequence: 👨‍👩‍👧‍👦.\n".to_string(),
    ));

    // (j) 1-MiB-Dokument
    let mut large_doc = String::with_capacity(1_050_000);
    large_doc.push_str("# 1-MiB Big Document\n\n");
    let para = "Dies ist ein wiederkehrender Absatz mit verschiedenen Wörtern und Satzzeichen zur Lastmessung.\n\n";
    while large_doc.len() < 1_048_576 {
        large_doc.push_str(para);
    }
    corpus.push(("edge_1mib_large".to_string(), large_doc));

    // 2. 200 deterministische prozedurale Dokumente (Seed: 4242)
    // Einfache LCG-Formel: X_{n+1} = (a * X_n + c) % m
    let mut seed = 4242u64;
    for doc_idx in 0..200 {
        let mut text = String::new();
        seed = (seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)) >> 1;
        let num_sections = (seed % 6) + 1;

        for sec in 0..num_sections {
            seed = (seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)) >> 1;
            let level = (seed % 3) + 1;
            text.push_str(&"#".repeat(level as usize));
            text.push_str(&format!(" Section {sec} Doc {doc_idx}\n\n"));

            seed = (seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)) >> 1;
            let para_type = seed % 4;
            match para_type {
                0 => {
                    text.push_str("Standard fließender Fließtextabsatz mit einigen Wörtern für Token-Messung.\n\n");
                }
                1 => {
                    text.push_str("- Listenpunkt 1 mit Information\n- Listenpunkt 2 mit weiteren Daten\n- Listenpunkt 3\n\n");
                }
                2 => {
                    text.push_str("```json\n{\"id\": \"item\", \"val\": 123}\n```\n\n");
                }
                _ => {
                    text.push_str("| Key | Value |\n|---|---|\n| Alpha | Beta |\n| Gamma | Delta |\n\n");
                }
            }
        }
        corpus.push((format!("doc_proc_{doc_idx:03}"), text));
    }

    corpus
}
