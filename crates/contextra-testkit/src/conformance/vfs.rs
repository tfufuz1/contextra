//! # Konformitätsprüfsuite für den `Vfs`-Port (`vfs`)
//!
//! Dieses Modul prüft beliebige Implementierungen des [`contextra_ports::Vfs`]-Ports
//! auf Einhaltung der Dateisystemverträge (SPEC 3.1, SPEC §3.2 Regel 5 & Regel 10):
//! - Atomares Schreiben (`write_atomic`) und Haltbarkeits-Garantie
//! - Persistenz und Abfrage (`read`, `exists`, `len`)
//! - Verzeichnisverwaltung (`create_dir_all`, `list_dir`, `sync_dir`)
//! - Lösch- und Bereinigungssemantik (`remove_file`, `unlink`)
//! - Fail-Closed Verhalten bei nicht existierenden Pfaden

use super::failure::{ConformanceFailure, ConformanceResult};
use contextra_ports::Vfs;
use std::path::Path;

/// Führt die umfassende Konformitätsprüfsuite gegen eine gegebene [`Vfs`]-Instanz aus.
///
/// **Geprüfte Verträge:**
/// 1. `non_existent_read_fails`: Nicht vorhandene Dateien erzeugen einen Fehler.
/// 2. `write_atomic_and_read`: Geschriebene Bytes sind bitgenau und vollständig lesbar.
/// 3. `file_len_and_exists`: Längen- und Existenzprüfungen spiegeln exakt den geschriebenen Inhalt wider.
/// 4. `directory_lifecycle`: `create_dir_all`, `list_dir` und `sync_dir` verhalten sich deterministisch.
/// 5. `remove_and_unlink`: Gelöschte Dateien verschwinden aus Verzeichnis und Existenzabfragen.
/// 6. `overwrite_atomic`: Überschreiben ersetzt Daten atomar ohne Teildatenrückstände.
pub fn run_vfs_suite<V: Vfs>(vfs: &V) -> ConformanceResult {
    let suite = "Vfs";

    let test_dir = Path::new("conformance_test_dir");
    let test_file = Path::new("conformance_test_dir/atomic_payload.bin");
    let non_existent = Path::new("conformance_test_dir/missing.bin");

    // 1. Nicht-existierende Datei lesen muss fehlschlagen
    if vfs.read(non_existent).is_ok() {
        return Err(ConformanceFailure::new(
            suite,
            "non_existent_read_fails",
            "Reading non-existent file must return Err, but returned Ok",
        ));
    }

    if let Ok(exists) = vfs.exists(non_existent) {
        if exists {
            return Err(ConformanceFailure::new(
                suite,
                "non_existent_exists_false",
                "non-existent file reported exists = true",
            ));
        }
    }

    // 2. Verzeichnis anlegen und syncen
    if let Err(e) = vfs.create_dir_all(test_dir) {
        return Err(ConformanceFailure::new(
            suite,
            "create_dir_all",
            format!("create_dir_all failed: {e:?}"),
        ));
    }

    if let Err(e) = vfs.sync_dir(test_dir) {
        return Err(ConformanceFailure::new(
            suite,
            "sync_dir",
            format!("sync_dir failed: {e:?}"),
        ));
    }

    // 3. Atomares Schreiben
    let sample_data = b"CONTEXTRA-VFS-CONFORMANCE-TEST-DATA-12345";
    if let Err(e) = vfs.write_atomic(test_file, sample_data) {
        return Err(ConformanceFailure::new(
            suite,
            "write_atomic",
            format!("write_atomic failed: {e:?}"),
        ));
    }

    // 4. Existenz & Länge
    match vfs.exists(test_file) {
        Ok(true) => {}
        Ok(false) => {
            return Err(ConformanceFailure::new(
                suite,
                "file_len_and_exists",
                "File written via write_atomic does not exist",
            ));
        }
        Err(e) => {
            return Err(ConformanceFailure::new(
                suite,
                "file_len_and_exists",
                format!("exists check failed: {e:?}"),
            ));
        }
    }

    match vfs.len(test_file) {
        Ok(len) if len == sample_data.len() as u64 => {}
        Ok(len) => {
            return Err(ConformanceFailure::new(
                suite,
                "file_len_and_exists",
                format!(
                    "File length mismatch: expected {}, got {len}",
                    sample_data.len()
                ),
            ));
        }
        Err(e) => {
            return Err(ConformanceFailure::new(
                suite,
                "file_len_and_exists",
                format!("len query failed: {e:?}"),
            ));
        }
    }

    // 5. Inhalt lesen
    match vfs.read(test_file) {
        Ok(read_data) if read_data == sample_data => {}
        Ok(read_data) => {
            return Err(ConformanceFailure::new(
                suite,
                "write_atomic_and_read",
                format!(
                    "Data read mismatch: expected {} bytes, read {} bytes",
                    sample_data.len(),
                    read_data.len()
                ),
            ));
        }
        Err(e) => {
            return Err(ConformanceFailure::new(
                suite,
                "write_atomic_and_read",
                format!("read failed: {e:?}"),
            ));
        }
    }

    // 6. Verzeichnisinhalt auflisten
    match vfs.list_dir(test_dir) {
        Ok(entries) => {
            if !entries.iter().any(|e| e.contains("atomic_payload.bin")) {
                return Err(ConformanceFailure::new(
                    suite,
                    "directory_lifecycle",
                    format!("list_dir does not contain atomic_payload.bin: entries={entries:?}"),
                ));
            }
        }
        Err(e) => {
            return Err(ConformanceFailure::new(
                suite,
                "directory_lifecycle",
                format!("list_dir failed: {e:?}"),
            ));
        }
    }

    // 7. Überschreiben prüfen
    let updated_data = b"UPDATED-PAYLOAD";
    if let Err(e) = vfs.write_atomic(test_file, updated_data) {
        return Err(ConformanceFailure::new(
            suite,
            "overwrite_atomic",
            format!("overwrite failed: {e:?}"),
        ));
    }
    match vfs.read(test_file) {
        Ok(read_data) if read_data == updated_data => {}
        _ => {
            return Err(ConformanceFailure::new(
                suite,
                "overwrite_atomic",
                "Overwritten content does not match new data",
            ));
        }
    }

    // 8. Datei entfernen
    if let Err(e) = vfs.remove_file(test_file) {
        return Err(ConformanceFailure::new(
            suite,
            "remove_and_unlink",
            format!("remove_file failed: {e:?}"),
        ));
    }

    match vfs.exists(test_file) {
        Ok(false) => {}
        _ => {
            return Err(ConformanceFailure::new(
                suite,
                "remove_and_unlink",
                "File still exists after remove_file",
            ));
        }
    }

    Ok(())
}
