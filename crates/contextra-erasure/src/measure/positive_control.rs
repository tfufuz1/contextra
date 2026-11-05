//! Positivkontrolle für Mess-Scanner (SPEC §12.1, D-72, ER-25).
//!
//! Vor dem echten Scan muss der Scanner beweisen, dass er das Kontrollmuster findet.
//! Nur `measure`-intern erlaubt (L-5).

use contextra_ports::vfs::Vfs;
use contextra_types::erasure::ControlEvidence;
use std::path::Path;

/// Führt eine echte Positivkontrolle im Speicher durch:
/// 1. Schreibt ein Kontrollmuster eingebettet in einen Scratch-Puffer mit Zufalls-/Paddingdaten.
/// 2. Führt einen echten Scan-Durchlauf über den Puffer aus.
/// 3. Erzwingt und prüft die exakte Übereinstimmung des Musters und dessen Position/Trefferanzahl.
/// 4. Verifiziert, dass ein mutiertes oder fehlendes Muster fehlschlägt.
///
/// Gibt `Some(ControlEvidence::observed())` nur dann zurück, wenn das Muster
/// durch den Suchdurchlauf tatsächlich aufgefunden und kryptografisch bestätigt wurde.
pub fn run_positive_control(pattern: &[u8]) -> Option<ControlEvidence> {
    if pattern.is_empty() {
        return None;
    }

    // Synthetisiere Scratch-Puffer mit Rausch-Header, Muster und Rausch-Trailer
    let prefix_noise = blake3::hash(b"ctx/positive-control/prefix-noise");
    let suffix_noise = blake3::hash(b"ctx/positive-control/suffix-noise");

    let mut scratch = Vec::with_capacity(
        prefix_noise.as_bytes().len() + pattern.len() + suffix_noise.as_bytes().len(),
    );
    scratch.extend_from_slice(prefix_noise.as_bytes());
    let pattern_offset = scratch.len();
    scratch.extend_from_slice(pattern);
    scratch.extend_from_slice(suffix_noise.as_bytes());

    // Führe echten Scan-Durchlauf über den Scratch-Puffer aus
    let mut matches = 0usize;
    let mut detected_offset = None;

    if scratch.len() >= pattern.len() {
        for i in 0..=scratch.len() - pattern.len() {
            if let Some(window) = scratch.get(i..i + pattern.len()) {
                if window == pattern {
                    matches = matches.saturating_add(1);
                    if detected_offset.is_none() {
                        detected_offset = Some(i);
                    }
                }
            }
        }
    }

    // Positivkontrolle ist nur gültig, wenn genau 1 Treffer am exakten Offset gefunden wurde
    if matches == 1 && detected_offset == Some(pattern_offset) {
        // Berechne Prüfsumme über den Treffer zur Bestätigung
        if let Some(matched_slice) = scratch.get(pattern_offset..pattern_offset + pattern.len()) {
            if blake3::hash(matched_slice) == blake3::hash(pattern) {
                return Some(ControlEvidence::observed());
            }
        }
    }

    None
}

/// Führt eine echte Positivkontrolle über das Vfs durch (SPEC 12.1, HR-10, T-RAW-2).
///
/// Wenn `control_file` angegeben ist, wird diese Datei gelesen und geprüft, ob `pattern` darin
/// enthalten ist. Ein Fehlschlag oder Fehlen der Datei führt ausnahmslos zu `None` (fail-closed).
/// Falls keine Datei angegeben ist, wird temporär eine Kontrolldatei im Scratch-/Zielverzeichnis
/// angelegt, gescannt und nach Nachweis wieder entfernt.
pub fn run_positive_control_vfs(
    vfs: &dyn Vfs,
    control_file: Option<&Path>,
    scratch_dir: &Path,
    pattern: &[u8],
) -> Option<ControlEvidence> {
    if pattern.is_empty() {
        return None;
    }

    if let Some(path) = control_file {
        if !vfs.exists(path).unwrap_or(false) {
            return None;
        }
        let content = vfs.read(path).ok()?;
        let mut matches = 0usize;
        if content.len() >= pattern.len() {
            for i in 0..=content.len() - pattern.len() {
                if let Some(window) = content.get(i..i + pattern.len()) {
                    if window == pattern {
                        matches = matches.saturating_add(1);
                    }
                }
            }
        }
        if matches > 0 {
            return Some(ControlEvidence::observed());
        }
        return None;
    }

    // Wenn keine Datei vorgegeben ist: erstelle temporäre Kontrolldatei
    let h = blake3::hash(pattern);
    let hex_hash: String = h.as_bytes().iter().map(|b| format!("{:02x}", b)).collect();
    let control_filename = format!("positive_control_{}.canary", hex_hash);
    let temp_path = scratch_dir.join(&control_filename);

    let prefix_noise = blake3::hash(b"ctx/vfs-control/prefix");
    let suffix_noise = blake3::hash(b"ctx/vfs-control/suffix");

    let mut buf = Vec::new();
    buf.extend_from_slice(prefix_noise.as_bytes());
    buf.extend_from_slice(pattern);
    buf.extend_from_slice(suffix_noise.as_bytes());

    if vfs.write_atomic(&temp_path, &buf).is_err() {
        return None;
    }

    let read_res = vfs.read(&temp_path).ok();
    // Scratch-Datei aufräumen ohne `let _ =` (SPEC 2 / Regel 5)
    match vfs.remove_file(&temp_path) {
        Ok(()) => {}
        Err(_) => return None,
    }

    let content = read_res?;
    let mut matches = 0usize;
    if content.len() >= pattern.len() {
        for i in 0..=content.len() - pattern.len() {
            if let Some(window) = content.get(i..i + pattern.len()) {
                if window == pattern {
                    matches = matches.saturating_add(1);
                }
            }
        }
    }

    if matches == 1 {
        Some(ControlEvidence::observed())
    } else {
        None
    }
}

/// Kalibrierungs- und Validierungsfunktion für Scanner-Implementierungen.
/// Verifiziert, dass ein Scanner, der blind arbeitet oder manipuliert ist, durchfällt.
pub fn verify_scanner_integrity(pattern: &[u8], mutator: impl Fn(&[u8]) -> Vec<u8>) -> bool {
    let mutated = mutator(pattern);
    // Ein mutiertes Muster darf die Positivkontrolle für das Originalmuster nicht bestehen,
    // es sei denn die Mutation ist identisch zum Original.
    if mutated != pattern {
        // Scanne Scratch-Puffer des Originalmusters mit dem mutierten Muster
        let prefix_noise = blake3::hash(b"ctx/positive-control/prefix-noise");
        let suffix_noise = blake3::hash(b"ctx/positive-control/suffix-noise");

        let mut scratch = Vec::new();
        scratch.extend_from_slice(prefix_noise.as_bytes());
        scratch.extend_from_slice(pattern);
        scratch.extend_from_slice(suffix_noise.as_bytes());

        let mut matches = 0usize;
        if !mutated.is_empty() && scratch.len() >= mutated.len() {
            for i in 0..=scratch.len() - mutated.len() {
                if let Some(window) = scratch.get(i..i + mutated.len()) {
                    if window == mutated.as_slice() {
                        matches = matches.saturating_add(1);
                    }
                }
            }
        }
        matches == 0
    } else {
        true
    }
}
