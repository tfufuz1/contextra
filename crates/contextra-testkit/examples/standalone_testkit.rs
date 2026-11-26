//! Eigenständiges Beispiel zur Demonstration von `contextra-testkit` als autarker Baukasten.
//!
//! Zeigt den Einsatz von `ManualClock`, `FaultVfs`, `InMemoryStorageEngine` und Port-Konformitätsprüfungen
//! ohne Abhängigkeiten zu höheren Ringen oder externen Runtimes.

use contextra_ports::{Clock, StorageEngine, Vfs};
use contextra_testkit::conformance::{run_clock_suite, run_vfs_suite};
use contextra_testkit::{FaultVfs, InMemoryStorageEngine, ManualClock};
use contextra_types::TxId;
use std::path::Path;
use std::time::Duration;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== contextra-testkit Standalone Building-Block Example ===");

    // 1. Deterministische Uhr
    let clock = ManualClock::new(1_000_000_000);
    println!("Initial clock time: {} ns", clock.now_unix_nanos());
    clock.advance(Duration::from_nanos(500_000_000));
    println!("Advanced clock time: {} ns", clock.now_unix_nanos());
    assert_eq!(clock.now_unix_nanos(), 1_500_000_000);

    // Konformitätsprüfung für Clock
    let clock_res = run_clock_suite(&clock);
    println!("Clock conformance result: {clock_res:?}");
    assert!(clock_res.is_ok());

    // 2. In-Memory-Vfs mit Fehlerprüfung
    let vfs = FaultVfs::new();
    let p = Path::new("example_dir/test.txt");
    vfs.create_dir_all(Path::new("example_dir"))?;
    vfs.write_atomic(p, b"Baukasten Data")?;
    println!("Vfs file written, len: {}", vfs.len(p)?);
    let read_back = vfs.read(p)?;
    println!("Vfs file content: {}", String::from_utf8_lossy(&read_back));
    assert_eq!(read_back, b"Baukasten Data");

    // Konformitätsprüfung für Vfs
    let vfs_res = run_vfs_suite(&vfs);
    println!("Vfs conformance result: {vfs_res:?}");
    assert!(vfs_res.is_ok());

    // 3. Transaktionale In-Memory-Speicher-Engine
    let storage = InMemoryStorageEngine::new();
    let tx1 = TxId::new(1);
    let key = b"my_key";
    let val = b"my_value";

    storage.put(tx1, key, val).await?;
    storage.commit(tx1).await?;
    let res = storage.get(key).await?;
    println!("StorageEngine get: {res:?}");
    assert_eq!(res.as_deref(), Some(&val[..]));

    println!("=== contextra-testkit demonstration successfully completed! ===");
    Ok(())
}
