//! Gemeinsame Testhilfen. Die Verzeichnisse liegen unter dem System-Temp-Verzeichnis
//! und tragen die Prozess-ID und einen Testnamen. Eine Zeit- oder Zufallsquelle ist
//! dafür nicht nötig.

use std::io;
use std::path::PathBuf;

/// Frisches, leeres Testverzeichnis.
pub fn scratch(name: &str) -> io::Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!("acta-fs-{}-{name}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
