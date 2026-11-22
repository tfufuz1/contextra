//! Minimales autarkes Beispiel für crash-sichere Dateisystem-Operationen.
use contextra_durable_fs::vfs::{StdVfs, Vfs};
use tempfile::tempdir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let path = dir.path().join("durable.txt");
    let vfs = StdVfs;

    vfs.write_atomic(&path, b"crash-consistent-data")?;
    let content = vfs.read(&path)?;
    assert_eq!(content, b"crash-consistent-data");

    println!("Erfolgreich atomar geschrieben und gelesen: {:?}", path);
    Ok(())
}
