//! INV-IO1 (Ratchet): Produktionscode in `contextra-store` soll Dateioperationen über
//! `StoreIo`/`Vfs` ausführen, nicht über `tokio::fs`/`std::fs`.
//!
//! Dieser Test erzwingt das als Ratchet: Die unten gelisteten Dateien sind bekannte,
//! dokumentierte Ausnahmen mit einer Obergrenze. Neue Dateien oder zusätzliche Aufrufe lassen
//! den Test fehlschlagen. Wer eine Ausnahme abbaut, senkt die Obergrenze (oder entfernt den
//! Eintrag); sie darf nie steigen. Offene Ausnahmen:
//!
//! - `io_seam.rs`: der Real-FS-Adapter (`RealFs`), die einzige legitime Stelle.
//! - `wal/mod.rs`: `secure_fs` für Schlüssel-/Marker-Dateien (`hard_link`, `0600`, ACL) und das
//!   Loom-Mock-Modul; die WAL-Datendatei selbst läuft über `StoreIo`.
//! - `util.rs`, `scanner/*`, `lsm/engine.rs`: noch nicht auf `StoreIo` migriert.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const ALLOWED: &[(&str, usize)] = &[
    ("io_seam.rs", 20),
    ("wal/mod.rs", 18),
    ("util.rs", 1),
    ("scanner/wal.rs", 2),
    ("scanner/sstable.rs", 2),
    ("lsm/engine.rs", 3),
];

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "tests") {
                continue;
            }
            collect(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "tests.rs" || name.ends_with("_tests.rs") {
                continue;
            }
            out.push(path);
        }
    }
}

/// Zählt Produktionszeilen mit `tokio::fs`/`std::fs`; schneidet am ersten `#[cfg(test)] mod x {`
/// ab und ignoriert Zeilenkommentare.
fn production_fs_uses(source: &str) -> usize {
    let lines: Vec<&str> = source.lines().collect();
    let mut count = 0;
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim() == "#[cfg(test)]" {
            let mut j = i + 1;
            while j < lines.len() && lines[j].trim().is_empty() {
                j += 1;
            }
            if lines.get(j).is_some_and(|l| {
                let t = l.trim_start();
                let t = t.strip_prefix("pub ").unwrap_or(t);
                t.starts_with("mod ") && t.trim_end().ends_with('{')
            }) {
                break;
            }
        }
        let code = lines[i].split("//").next().unwrap_or("");
        if code.contains("tokio::fs") || code.contains("std::fs") {
            count += 1;
        }
        i += 1;
    }
    count
}

#[test]
fn production_code_does_not_grow_direct_fs_usage() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect(&src, &mut files);

    let allowed: BTreeMap<&str, usize> = ALLOWED.iter().copied().collect();
    let mut violations = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(&src)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let used = production_fs_uses(&fs::read_to_string(&file).unwrap());
        let limit = allowed.get(rel.as_str()).copied().unwrap_or(0);
        if used > limit {
            violations.push(format!(
                "{rel}: {used} direkte fs-Aufrufe, erlaubt {limit} (INV-IO1: StoreIo/Vfs nutzen)"
            ));
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}
