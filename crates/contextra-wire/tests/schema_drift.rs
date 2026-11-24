//! # Schema Drift Test für FlatBuffers IPC
//!
//! **Zweck:**
//! 1. Prüft den SHA-256 Hash der Datei `schemas/contextra.fbs` gegen die gepinnte Prüfsumme
//!    in `schemas/contextra.fbs.sha256`. Bei Abweichung wird der neue Hash zusammen mit
//!    der Regenerationsanweisung ausgegeben.
//! 2. Falls `flatc` im Systempfad (`PATH`) installiert ist und exakt die vorgeschriebene
//!    Version `flatc version 24.12.23` meldet, wird der Code in ein temporäres Verzeichnis
//!    neu generiert und byte-genau mit `src/contextra_generated.rs` verglichen.
//!    Fehlt `flatc` oder weicht die Version ab, wird eine Warnung auf stderr ausgegeben.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Berechnet den SHA-256 Hash eines Byte-Slices gemäß FIPS 180-4 ohne externe Abhängigkeiten.
fn sha256_digest(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, w_elem) in w.iter_mut().take(16).enumerate() {
            let offset = i * 4;
            let bytes = [
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ];
            *w_elem = u32::from_be_bytes(bytes);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut h_var = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h_var
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h_var = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(h_var);
    }

    let mut out = [0u8; 32];
    for (i, &val) in h.iter().enumerate() {
        let b = val.to_be_bytes();
        out[i * 4..(i + 1) * 4].copy_from_slice(&b);
    }
    out
}

fn to_hex_string(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

fn crate_root_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn test_schema_hash_matches_pin() {
    let crate_dir = crate_root_path();
    let schema_file = crate_dir.join("schemas/contextra.fbs");
    let pin_file = crate_dir.join("schemas/contextra.fbs.sha256");

    assert!(
        schema_file.exists(),
        "Schema-Datei fehlt: {}",
        schema_file.display()
    );
    assert!(pin_file.exists(), "Pin-Datei fehlt: {}", pin_file.display());

    let schema_content = fs::read(&schema_file)
        .unwrap_or_else(|e| panic!("Fehler beim Lesen von {}: {e}", schema_file.display()));
    let expected_hash = fs::read_to_string(&pin_file)
        .unwrap_or_else(|e| panic!("Fehler beim Lesen von {}: {e}", pin_file.display()))
        .trim()
        .to_lowercase();

    let computed_hash = to_hex_string(&sha256_digest(&schema_content));

    if computed_hash != expected_hash {
        panic!(
            "\n\n================================================================================\n\
            SCHEMA-DRIFT DETEKTIERT!\n\
            Die Prüfsumme von '{}' weicht von der gepinnten Prüfsumme ab.\n\n\
            Erwartet (Pin):  {}\n\
            Tatsächlich:     {}\n\n\
            Regenerationsanleitung:\n\
            1. Wenn flatc 24.12.23 installiert ist, führe folgenden Befehl im Crate-Verzeichnis aus:\n\
               flatc --rust -o src schemas/contextra.fbs\n\
            2. Aktualisiere die Pin-Datei schemas/contextra.fbs.sha256 mit dem neuen SHA-256 Hash:\n\
               echo \"{}\" > schemas/contextra.fbs.sha256\n\
            3. Verifiziere die Tests mit: cargo test -p contextra-wire\n\
            ================================================================================\n",
            schema_file.display(),
            expected_hash,
            computed_hash,
            computed_hash,
        );
    }
}

// Hilfsfunktion: Führt dasselbe Post-Processing durch wie build.rs
fn wrap_tab_get_calls_in_unsafe(line: &str) -> String {
    let pattern = "self._tab.get";
    if !line.contains(pattern) {
        return line.to_string();
    }

    let mut result = String::with_capacity(line.len() + 32);
    let mut search_from = 0;

    while let Some(rel_idx) = line[search_from..].find(pattern) {
        let idx = search_from + rel_idx;

        let prefix = &line[..idx];
        if prefix.trim_end().ends_with("unsafe {") || prefix.contains("unsafe") {
            result.push_str(&line[search_from..idx + pattern.len()]);
            search_from = idx + pattern.len();
            continue;
        }

        if let Some(rel_paren_open) = line[idx + pattern.len()..].find('(') {
            let paren_open = idx + pattern.len() + rel_paren_open;

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

fn post_process_generated_code(output_file: &Path) {
    let content = match fs::read_to_string(output_file) {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut modified = false;
    let new_lines: Vec<String> = content
        .lines()
        .map(|line| {
            let processed = wrap_tab_get_calls_in_unsafe(line);
            if processed != line {
                modified = true;
            }
            processed
        })
        .collect();

    if modified {
        let mut new_content = new_lines.join("\n");
        if content.ends_with('\n') {
            new_content.push('\n');
        }
        let _ = fs::write(output_file, new_content);
    }
}

#[test]
fn test_generated_code_matches_flatc_output_if_available() {
    let required_version = "24.12.23";
    let crate_dir = crate_root_path();
    let schema_file = crate_dir.join("schemas/contextra.fbs");
    let committed_file = crate_dir.join("src/contextra_generated.rs");

    let version_output = Command::new("flatc").arg("--version").output();

    let flatc_available = match version_output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{stdout} {stderr}");
            if combined.contains(required_version) {
                true
            } else {
                eprintln!(
                    "\n\
                    ********************************************************************************\n\
                    HINWEIS: 'flatc' ist im PATH vorhanden, entspricht aber nicht der Soll-Version!\n\
                    Erwartet: flatc Version '{}'\n\
                    Erhalten: '{}'\n\
                    Der Byte-Gleichheitstest des generierten Codes wird übersprungen.\n\
                    ********************************************************************************\n",
                    required_version,
                    combined.trim()
                );
                false
            }
        }
        _ => {
            eprintln!(
                "\n\
                ********************************************************************************\n\
                HINWEIS: 'flatc' ist nicht im PATH verfügbar oder nicht ausführbar.\n\
                Der Byte-Gleichheitstest des generierten Codes wird übersprungen.\n\
                Der Hash-Pin-Test 'test_schema_hash_matches_pin' prüft die Schemaintegrität.\n\
                ********************************************************************************\n"
            );
            false
        }
    };

    if !flatc_available {
        return;
    }

    // Wenn flatc mit korrekter Version verfügbar ist:
    let temp_dir =
        std::env::temp_dir().join(format!("contextra_wire_flatc_test_{}", std::process::id()));
    let _ = fs::create_dir_all(&temp_dir);

    let status = Command::new("flatc")
        .args([
            "--rust",
            "-o",
            temp_dir.to_str().expect("valid temp path"),
            schema_file.to_str().expect("valid schema path"),
        ])
        .status();

    let gen_file = temp_dir.join("contextra_generated.rs");
    if let Ok(st) = status {
        assert!(st.success(), "flatc Aufruf ist fehlgeschlagen");
        post_process_generated_code(&gen_file);

        let generated_content = fs::read(&gen_file)
            .unwrap_or_else(|e| panic!("Fehler beim Lesen der neu generierten Datei: {e}"));
        let committed_content = fs::read(&committed_file)
            .unwrap_or_else(|e| panic!("Fehler beim Lesen von {}: {e}", committed_file.display()));

        let _ = fs::remove_dir_all(&temp_dir);

        assert_eq!(
            generated_content, committed_content,
            "Byte-Abweichung zwischen generierter Datei und committed src/contextra_generated.rs festgestellt!"
        );
    } else {
        let _ = fs::remove_dir_all(&temp_dir);
        panic!("flatc konnte nicht ausgeführt werden trotz erfolgreicher Versionsprüfung");
    }
}
