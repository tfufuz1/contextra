//! Typisierter Scanner-Fluss von Unarmed zu Armed (SPEC §12.1, §12.2, D-72, ER-25).
//!
//! Der Ablauf ist streng:
//!   `Scanner<Unarmed>` → Positivkontrolle → `ControlEvidence` → `arm` → `Scanner<Armed>`
//!   → `record(files, bytes)` → `finish(matches, duration, hash)` → `LayerWitness`
//!
//! Nur `measure`-intern erlaubt (L-5, ER-25).

use super::field::{collect_all_files, read_file_verified, ScanPlan, VerifiedVia};
use super::positive_control::run_positive_control_vfs;
use super::witness::{arm_scanner, finish_scan};
use crate::error::ErasureError;
use contextra_ports::clock::{Clock, SystemClock};
use contextra_ports::vfs::Vfs;
use contextra_types::erasure::{LayerId, LayerWitness, MeasureClass, Scanner, Unarmed};
use std::time::Duration;

/// Führt einen echten Scan über alle Dateien mit vorangehender Positivkontrolle durch.
///
/// 1. Positivkontrolle über Vfs.
/// 2. Scanner scharfschalten (`arm`).
/// 3. Durchsuchen aller Dateien im Zielverzeichnis (inkl. tmp/, alte Segmente, WAL, Snapshots, scope/, Berichte).
/// 4. Je nach Messklasse:
///    - MK1: Strukturscan über Decoder und Rohbytes von Steuerdateien.
///    - MK2: Rohscan nach Klartext-Needles und Mustern.
///    - MK3: Prüfung der Schlüsselabwesenheit (Status zwingend erforderlich).
/// 5. Bei matches > 0 liefert die Funktion `ErasureError::Residue(matches)`.
/// 6. Bei matches == 0 wird ein `LayerWitness` ausgestellt.
pub fn scan_with_control(vfs: &dyn Vfs, plan: &ScanPlan) -> Result<LayerWitness, ErasureError> {
    let clock = SystemClock::new();
    let start_nanos = clock.monotonic_nanos();

    // Schritt 1: Positivkontrolle (fail-closed, ohne stillen Fallback)
    let evidence = run_positive_control_vfs(
        vfs,
        plan.control_file.as_deref(),
        &plan.root_dir,
        &plan.control_pattern,
    )
    .ok_or_else(|| {
        ErasureError::Internal(
            "Positivkontrolle fehlgeschlagen: Kontrollmuster nicht gefunden".into(),
        )
    })?;

    // Schritt 2: Scanner armen
    let unarmed = Scanner::<Unarmed>::new(plan.layer, plan.class, plan.method_id);
    let mut armed = arm_scanner(unarmed, evidence);

    // Schritt 3: Alle Dateien sammeln
    let files = collect_all_files(vfs, &plan.root_dir)?;
    let mut total_bytes = 0u64;
    let mut total_files = 0u64;
    let mut total_matches = 0u64;
    let mut verified_medium = VerifiedVia::Direct;

    // Hasher für deterministischen Zeugen-Hash
    let mut hash_state = blake3::Hasher::new();
    hash_state.update(plan.layer.as_bytes());
    hash_state.update(&(plan.method_id).to_le_bytes());
    hash_state.update(&[match plan.class {
        MeasureClass::Mk1 => 1,
        MeasureClass::Mk2 => 2,
        MeasureClass::Mk3 => 3,
    }]);

    match plan.class {
        MeasureClass::Mk1 => {
            for path in &files {
                let (content, via) = read_file_verified(vfs, path)?;
                total_files = total_files.saturating_add(1);
                total_bytes = total_bytes.saturating_add(content.len() as u64);
                if via != VerifiedVia::Direct {
                    verified_medium = via;
                }

                let path_str = path.to_string_lossy();
                let is_control_file = path_str.contains("manifest")
                    || path_str.ends_with(".wal")
                    || path_str.ends_with(".sst")
                    || path_str.ends_with(".idx")
                    || path_str.contains("vault");

                // Rohbytes der Steuerdateien gegen Target-Patterns
                if is_control_file {
                    for pat in &plan.target_patterns {
                        if !pat.is_empty() && content.len() >= pat.len() {
                            for i in 0..=content.len() - pat.len() {
                                if let Some(window) = content.get(i..i + pat.len()) {
                                    if window == pat.as_slice() {
                                        total_matches = total_matches.saturating_add(1);
                                    }
                                }
                            }
                        }
                    }
                }

                // Decoder anwenden falls passend
                for decoder in &plan.decoders {
                    if decoder.can_decode(path) {
                        for pat in &plan.target_patterns {
                            let matches = decoder.decode_and_check(path, &content, pat)?;
                            total_matches = total_matches.saturating_add(matches);
                        }
                    }
                }

                hash_state.update(blake3::hash(&content).as_bytes());
            }
        }
        MeasureClass::Mk2 => {
            for path in &files {
                let (content, via) = read_file_verified(vfs, path)?;
                total_files = total_files.saturating_add(1);
                total_bytes = total_bytes.saturating_add(content.len() as u64);
                if via != VerifiedVia::Direct {
                    verified_medium = via;
                }

                for pat in &plan.target_patterns {
                    if !pat.is_empty() && content.len() >= pat.len() {
                        for i in 0..=content.len() - pat.len() {
                            if let Some(window) = content.get(i..i + pat.len()) {
                                if window == pat.as_slice() {
                                    total_matches = total_matches.saturating_add(1);
                                }
                            }
                        }
                    }
                }

                hash_state.update(blake3::hash(&content).as_bytes());
            }
        }
        MeasureClass::Mk3 => {
            // MK3 Feldmessung: Schlüsselabwesenheit (fail-closed wenn mk3_status fehlt)
            let st = plan.mk3_status.ok_or_else(|| {
                ErasureError::Internal("MK3-Messung erfordert Mk3StatusInput".into())
            })?;

            if !st.top_slot_zeroed {
                total_matches = total_matches.saturating_add(1);
            }
            if !st.dek_derivation_fails {
                total_matches = total_matches.saturating_add(1);
            }
            if !st.file_slots_absent {
                total_matches = total_matches.saturating_add(1);
            }

            // Auch Dateien im Tresor/Verzeichnis zählen
            for path in &files {
                let (content, via) = read_file_verified(vfs, path)?;
                total_files = total_files.saturating_add(1);
                total_bytes = total_bytes.saturating_add(content.len() as u64);
                if via != VerifiedVia::Direct {
                    verified_medium = via;
                }
                hash_state.update(blake3::hash(&content).as_bytes());
            }
        }
    }

    armed.record(total_files, total_bytes);

    hash_state.update(&total_files.to_le_bytes());
    hash_state.update(&total_bytes.to_le_bytes());
    hash_state.update(&total_matches.to_le_bytes());
    hash_state.update(verified_medium.as_str().as_bytes());
    hash_state.update(blake3::hash(&plan.control_pattern).as_bytes());
    let scan_hash = *hash_state.finalize().as_bytes();

    let elapsed_nanos = clock.monotonic_nanos().saturating_sub(start_nanos);
    let duration = Duration::from_nanos(elapsed_nanos);

    // matches != 0 -> Fehler
    finish_scan(armed, total_matches, duration, scan_hash)
        .map_err(|r| ErasureError::Residue(r.matches))
}

/// Führt einen delegierenden Scan-Fluss für Schichten aus (SPEC §12.1, §12.2, D-72, ER-25).
///
/// Veraltet: Für echte Messungen muss `scan_with_control(vfs, plan)` verwendet werden.
/// Gibt `ErasureError::NotImplemented` zurück, um Scheinerfolge zu verhindern (P1, HR-10).
#[deprecated(note = "Verwende scan_with_control(vfs, plan) für echte Scans")]
pub fn run_scanner_flow(
    _layer: LayerId,
    _class: MeasureClass,
    _method_id: u16,
    _control_pattern: &[u8],
) -> Result<LayerWitness, ErasureError> {
    Err(ErasureError::NotImplemented(
        "run_scanner_flow ohne Vfs/Plan ist deaktiviert. Verwende scan_with_control.".into(),
    ))
}
