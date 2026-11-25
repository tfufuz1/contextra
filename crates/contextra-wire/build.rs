//! # Build-Skript für `contextra-wire`
//!
//! **Ring & Klasse:** Ring 0 (Build-Phase)
//!
//! ## Zweck
//! Dieses Build-Skript prüft zur Compile-Zeit, ob das FlatBuffers-Schema `schemas/contextra.fbs`
//! vorhanden ist und ob der FlatBuffers-Compiler `flatc` auf dem Host-System verfügbar ist.
//! Falls ja, wird der Rust-Code neu generiert und mittels Post-Processing nachbearbeitet,
//! um Compiler-Warnungen bezüglich fehlender `unsafe`-Blöcke bei `_tab.get`-Aufrufen zu beheben.
//! Ist `flatc` nicht installiert, wird auf die bereits committete generierte Datei zurückgegriffen.

use std::path::Path;
use std::process::Command;

// ANCHOR[contextra-wire:BLD-001]
/// Hauptfunktion des Build-Skripts.
///
/// # Ablauf von `main`:
/// 1. Überprüfen, ob die FlatBuffers-Schemadatei `../../schemas/contextra.fbs` existiert.
/// 2. Falls vorhanden: Registrierung von `cargo:rerun-if-changed` für die Schemadatei.
/// 3. Prüfen, ob `flatc` auf dem System ausführbar ist.
/// 4. Wenn `flatc` verfügbar: Aufruf von `flatc --rust -o src ../../schemas/contextra.fbs`.
/// 5. Nach erfolgreicher Generierung: Post-Processing via `post_process_generated_code`.
/// 6. Falls `flatc` fehlt, aber generierter Code existiert: Warnung ausgeben und existierenden Code nutzen.
/// 7. Falls weder `flatc` noch die Datei existieren: Abbruch mit Fehlermeldung.
fn main() {
    let schema_path = "../../schemas/contextra.fbs";
    let out_dir = "src";

    if Path::new(schema_path).exists() {
        println!("cargo:rerun-if-changed={schema_path}");

        let output_file = Path::new(out_dir).join("contextra_generated.rs");

        let flatc_exists = Command::new("flatc")
            .arg("--version")
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if flatc_exists {
            if let Ok(status) = Command::new("flatc")
                .args(["--rust", "-o", out_dir, schema_path])
                .status()
            {
                assert!(status.success(), "flatc failed to generate code");
                post_process_generated_code(&output_file);
            }
        } else if !output_file.exists() {
            eprintln!("flatc not found and generated code does not exist. Please install flatbuffers compiler.");
            std::process::exit(1);
        } else {
            println!("cargo:warning=flatc not found, using existing generated code.");
        }
    }
}

// ANCHOR[contextra-wire:BLD-002]
/// Unsafe-Aufrufmuster des FlatBuffers-Generators, die `unsafe`-Blöcke benötigen.
///
/// Der Generator (`flatc 24.12.23`) erzeugt diese Aufrufe in Funktionen, die ohne `unsafe`-Block
/// geschrieben sind. `unsafe_op_in_unsafe_fn` ist im Crate als `deny` gesetzt und wird bewusst
/// nicht über Allow-Attribute umgangen, sondern durch explizite `unsafe`-Blöcke erfüllt.
const UNSAFE_CALL_PATTERNS: [&str; 4] = [
    "self._tab.get",
    "flatbuffers::Table::new",
    "flatbuffers::root_unchecked",
    "flatbuffers::size_prefixed_root_unchecked",
];

/// Kapselt Aufrufe eines Musters in `unsafe { ... }`, falls sie nicht bereits gewrappt sind.
///
/// # Ablauf
/// 1. Prüfen, ob das Muster in der Zeile vorkommt.
/// 2. Suchen der öffnenden und passenden schließenden Klammer des Aufrufs.
/// 3. Voranstellen von `unsafe { ` und Anhängen von ` }`.
fn wrap_unsafe_calls(line: &str, pattern: &str) -> String {
    if !line.contains(pattern) {
        return line.to_string();
    }

    let mut result = String::with_capacity(line.len() + 32);
    let mut search_from = 0;

    while let Some(rel_idx) = line[search_from..].find(pattern) {
        let idx = search_from + rel_idx;

        // Check if this occurrence is already preceded by `unsafe` on the same line
        let prefix = &line[..idx];
        if prefix.trim_end().ends_with("unsafe {") || prefix.contains("unsafe") {
            // Already wrapped
            result.push_str(&line[search_from..idx + pattern.len()]);
            search_from = idx + pattern.len();
            continue;
        }

        // Find the opening parenthesis of `get::<...>(...`
        if let Some(rel_paren_open) = line[idx + pattern.len()..].find('(') {
            let paren_open = idx + pattern.len() + rel_paren_open;

            // Find matching closing parenthesis
            let mut depth = 0;
            let mut close_paren = None;
            for (i, ch) in line[paren_open..].char_indices() {
                if ch == '(' {
                    depth += 1;
                } else if ch == ')' {
                    depth -= 1;
                    if depth == 0 {
                        close_paren = Some(paren_open + i);
                        break;
                    }
                }
            }

            if let Some(end_idx) = close_paren {
                result.push_str(&line[search_from..idx]);
                result.push_str("unsafe { ");
                result.push_str(&line[idx..=end_idx]);
                result.push_str(" }");
                search_from = end_idx + 1;
                continue;
            }
        }

        result.push_str(&line[search_from..idx + pattern.len()]);
        search_from = idx + pattern.len();
    }

    result.push_str(&line[search_from..]);
    result
}

/// Liefert den `// SAFETY:`-Kommentar für eine Zeile, die einen `unsafe`-Block erhält.
///
/// Das Workspace-Lint `undocumented_unsafe_blocks` verlangt eine Begründung direkt vor jedem
/// `unsafe`-Block. Der Generator liefert keine, daher wird sie hier mit dem Vertrag des jeweiligen
/// Aufrufs ergänzt.
fn safety_comment_for(line: &str) -> &'static str {
    if line.contains("flatbuffers::Table::new") {
        "// SAFETY: Der Aufrufer von `follow` garantiert, dass `buf` ein gültiger FlatBuffers-Puffer ist und `loc` auf eine Tabelle darin zeigt."
    } else if line.contains("root_unchecked") {
        "// SAFETY: Der Aufrufer von `root_as_*_unchecked` garantiert, dass `buf` ein verifizierter FlatBuffers-Puffer ist."
    } else {
        "// SAFETY: `self._tab` referenziert einen geprüften Tabellenpuffer; die VTable-Offsets stammen aus dem Generator-Layout."
    }
}

// ANCHOR[contextra-wire:BLD-003]
/// Liest die erzeugte FlatBuffers-Datei ein und wendet alle `UNSAFE_CALL_PATTERNS` zeilenweise an.
///
/// # Parameter
/// - `output_file`: Pfad zur Zieldatei `contextra_generated.rs`.
pub fn post_process_generated_code(output_file: &std::path::Path) {
    let content = match std::fs::read_to_string(output_file) {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut modified = false;
    let new_lines: Vec<String> = content
        .lines()
        .map(|line| {
            let processed = UNSAFE_CALL_PATTERNS
                .iter()
                .fold(line.to_string(), |acc, pattern| wrap_unsafe_calls(&acc, pattern));
            if processed != line {
                modified = true;
                let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
                format!("{indent}{}\n{processed}", safety_comment_for(line))
            } else {
                processed
            }
        })
        .collect();

    if modified {
        let mut new_content = new_lines.join("\n");
        if content.ends_with('\n') {
            new_content.push('\n');
        }
        if let Err(e) = std::fs::write(output_file, new_content) {
            eprintln!("Failed to write post-processed FlatBuffers code: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Prüft, dass wrap_unsafe_calls ungeschützte tab.get-Aufrufe korrekt in unsafe Blöcke packt.
    #[test]
    fn test_wrap_tab_get_calls() {
        let p = "self._tab.get";
        let input1 = "    self._tab.get::<u32>(RoleId::VT_ID, Some(0)).unwrap()";
        let expected1 = "    unsafe { self._tab.get::<u32>(RoleId::VT_ID, Some(0)) }.unwrap()";
        assert_eq!(wrap_unsafe_calls(input1, p), expected1);

        let input2 =
            "    self._tab.get::<flatbuffers::ForwardsUOffset<&str>>(ScoredDocument::VT_ID, None)";
        let expected2 = "    unsafe { self._tab.get::<flatbuffers::ForwardsUOffset<&str>>(ScoredDocument::VT_ID, None) }";
        assert_eq!(wrap_unsafe_calls(input2, p), expected2);

        let input3 = "    unsafe { self._tab.get::<i8>(Embedding::VT_METRIC, Some(0)).unwrap()}";
        assert_eq!(wrap_unsafe_calls(input3, p), input3);
    }

    // Prüft das Wrappen der Table- und Root-Aufrufe, die flatc 24.12.23 ungeschützt erzeugt.
    #[test]
    fn test_wrap_table_and_root_calls() {
        let input1 = "    Self { _tab: flatbuffers::Table::new(buf, loc) }";
        let expected1 = "    Self { _tab: unsafe { flatbuffers::Table::new(buf, loc) } }";
        assert_eq!(wrap_unsafe_calls(input1, "flatbuffers::Table::new"), expected1);

        let input2 = "  flatbuffers::root_unchecked::<SearchResponse>(buf)";
        let expected2 = "  unsafe { flatbuffers::root_unchecked::<SearchResponse>(buf) }";
        assert_eq!(wrap_unsafe_calls(input2, "flatbuffers::root_unchecked"), expected2);

        let input3 = "  flatbuffers::size_prefixed_root_unchecked::<SearchResponse>(buf)";
        let expected3 = "  unsafe { flatbuffers::size_prefixed_root_unchecked::<SearchResponse>(buf) }";
        assert_eq!(
            wrap_unsafe_calls(input3, "flatbuffers::size_prefixed_root_unchecked"),
            expected3
        );
        // Muster darf nicht über das Präfix des anderen greifen.
        assert_eq!(wrap_unsafe_calls(input3, "flatbuffers::root_unchecked"), input3);
    }
}
