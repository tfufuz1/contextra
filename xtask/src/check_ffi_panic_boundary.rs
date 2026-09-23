// FILE-CONTEXT
// ZWECK: CI-Gate — stellt sicher, dass Funktionen mit `py.allow_threads` in FFI-Boundary-Dateien
//        immer auch `catch_unwind(AssertUnwindSafe(...))` enthalten, wenn `panic = "abort"` im
//        Release-Profil aktiv ist.
// INVARIANTEN: False-Positives sind schlimmer als False-Negatives — im Zweifelsfall warn(), nicht error().
// ERSTELLT: 2026-09-13 (Audit-Befund P-4, Architektur-Review 2026-09-13)

use std::fs;
use std::path::Path;

/// Dateien, die FFI-Boundaries implementieren und auf catch_unwind geprüft werden.
/// ERWEITERBAR: Füge hier neue FFI-Crates hinzu, wenn sie py.allow_threads verwenden.
const FFI_BOUNDARY_FILES: &[&str] = &["crates/contextra-py/src/lib.rs"];

/// Gibt `true` zurück, wenn das Root-Cargo.toml im `[profile.release]`-Block `panic = "abort"` enthält.
pub fn panic_abort_active(workspace_root: &Path) -> bool {
    let cargo_toml_path = workspace_root.join("Cargo.toml");
    let content = match fs::read_to_string(&cargo_toml_path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    if let Some(release_pos) = content.find("[profile.release]") {
        let release_section = &content[release_pos..];
        // Examine lines within the profile.release section until next section header
        for line in release_section.lines().skip(1) {
            let line_trimmed = line.trim();
            if line_trimmed.starts_with('[') {
                break;
            }
            if line_trimmed.starts_with("panic") && line_trimmed.contains("\"abort\"") {
                return true;
            }
        }
    }

    false
}

/// Prüft eine einzelne FFI-Boundary-Datei auf `py.allow_threads` und `catch_unwind`.
pub fn check_file(path: &Path) -> Result<(), String> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[WARN] Datei nicht lesbar: {:?} ({})", path, e);
            return Ok(());
        }
    };

    let allow_threads_count = content.matches("py.allow_threads").count();
    let catch_unwind_count = content.matches("catch_unwind").count();

    if allow_threads_count > 0 && catch_unwind_count == 0 {
        return Err(format!(
            "Datei enthält {}x `py.allow_threads`, aber 0x `catch_unwind`. Bei panic=abort ist das gefährlich.",
            allow_threads_count
        ));
    }

    if allow_threads_count > 0 && catch_unwind_count < allow_threads_count {
        eprintln!(
            "[WARN] {:?}: {}x `py.allow_threads` gefunden, aber nur {}x `catch_unwind`.",
            path, allow_threads_count, catch_unwind_count
        );
    } else if allow_threads_count > 0 {
        println!(
            "[OK] {:?} ({}/{} geschützt)",
            path, catch_unwind_count, allow_threads_count
        );
    }

    Ok(())
}

/// Gibt `true` zurück wenn alle Checks bestanden, `false` bei blockierendem Befund.
pub fn run_check_ffi_panic_boundary(workspace_root: &Path) -> bool {
    let abort_active = panic_abort_active(workspace_root);
    if !abort_active {
        println!(
            "[SKIP] panic != \"abort\" im Release-Profil — FFI-Panic-Boundary-Check nicht nötig."
        );
        return true;
    }
    println!("[INFO] panic = \"abort\" aktiv — prüfe FFI-Boundary-Dateien auf catch_unwind...");

    let mut all_ok = true;
    for file in FFI_BOUNDARY_FILES {
        let path = workspace_root.join(file);
        match check_file(&path) {
            Ok(()) => {}
            Err(msg) => {
                eprintln!("[FAIL] {}: {}", file, msg);
                all_ok = false;
            }
        }
    }
    all_ok
}

// VERWENDUNG: `cargo xtask check-ffi-panic-boundary`
// CI-INTEGRATION: Füge diesen Befehl als separaten Step in `.github/workflows/merge-gate.yml`
//                 ein (als NICHT-blockierender Warn-Step in einem ersten Rollout,
//                 dann nach 1 Sprint auf blockierend hochstufen).
// ERWEITERUNG: Um eine neue FFI-Crate zu überwachen, füge ihren Pfad zu FFI_BOUNDARY_FILES hinzu.
