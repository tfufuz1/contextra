//! Fehlerinjektion über `FaultVfs` (Spec 16.2 `FaultVfs`, 16.4 Absturzmatrix, WP-7, Z4).

mod common;

use acta_fs::durable::DurableVfs;
use acta_fs::fault::{FaultAction, FaultPlan, FaultPoint, FaultVfs};
use acta_fs::vfs::Vfs;
use std::io::{self, ErrorKind};
use std::path::Path;

/// Schreibt `data` in eine neue Datei und synchronisiert sie.
fn write_new<V: Vfs>(vfs: &V, p: &Path, data: &[u8]) -> io::Result<()> {
    let f = vfs.create_new(p)?;
    f.write_all_at(0, data)?;
    f.sync_data().map_err(io::Error::other)
}

/// Liest eine Datei vollständig über den gegebenen Port.
fn read_all<V: Vfs>(vfs: &V, p: &Path) -> io::Result<Vec<u8>> {
    let f = vfs.open_read(p)?;
    let mut buf = vec![0_u8; usize::try_from(f.len()?).map_err(|_| io::Error::other("len"))?];
    f.read_exact_at(0, &mut buf)?;
    Ok(buf)
}

/// Ersetzungsablauf: neue Fassung schreiben, synchronisieren, atomar umbenennen.
fn replace<V: Vfs>(vfs: &V, dir: &Path, target: &Path) -> io::Result<()> {
    let tmp = dir.join("state.tmp");
    write_new(vfs, &tmp, b"new")?;
    vfs.rename_atomic(&tmp, target)
}

#[test]
fn fsync_failure_is_final_for_the_file() -> io::Result<()> {
    let dir = common::scratch("fsync_final")?;
    let vfs = FaultVfs::new(DurableVfs);
    vfs.arm(FaultPlan {
        point: FaultPoint::Sync,
        skip: 0,
        action: FaultAction::Fail(ErrorKind::Other),
    });

    let f = vfs.create_new(&dir.join("a"))?;
    f.write_all_at(0, b"data")?;
    assert!(
        f.sync_data().is_err(),
        "injected fsync failure must surface"
    );
    // Der innere Port würde jetzt Erfolg melden. Die Datei bleibt trotzdem entwertet.
    assert!(
        f.sync_data().is_err(),
        "second fsync must not count as stable"
    );
    assert!(
        f.write_all_at(4, b"more").is_err(),
        "no writes after failed fsync"
    );

    // Positivkontrolle: eine andere, ungestörte Datei synchronisiert ohne Fehler.
    let g = vfs.create_new(&dir.join("b"))?;
    g.write_all_at(0, b"x")?;
    g.sync_data().map_err(io::Error::other)?;
    assert!(!vfs.is_crashed());
    Ok(())
}

#[test]
fn enospc_on_write_is_reported_and_retryable() -> io::Result<()> {
    let dir = common::scratch("enospc")?;
    let vfs = FaultVfs::new(DurableVfs);
    vfs.arm(FaultPlan {
        point: FaultPoint::Write,
        skip: 0,
        action: FaultAction::Fail(ErrorKind::StorageFull),
    });
    let p = dir.join("a");
    let f = vfs.create_new(&p)?;
    let err = f
        .write_all_at(0, b"first")
        .err()
        .ok_or_else(|| io::Error::other("injected write did not fail"))?;
    assert_eq!(err.kind(), ErrorKind::StorageFull);

    // ENOSPC ist kein Absturz: derselbe Schreibvorgang gelingt beim zweiten Versuch.
    f.write_all_at(0, b"second")?;
    assert!(!vfs.is_crashed());
    assert_eq!(read_all(&DurableVfs, &p)?, b"second");
    Ok(())
}

#[test]
fn torn_write_leaves_only_the_prefix_and_crashes() -> io::Result<()> {
    let dir = common::scratch("torn")?;
    let vfs = FaultVfs::new(DurableVfs);
    vfs.arm(FaultPlan {
        point: FaultPoint::Write,
        skip: 0,
        action: FaultAction::TornWrite(4),
    });
    let p = dir.join("a");
    let f = vfs.create_new(&p)?;
    assert!(f.write_all_at(0, b"abcdefgh").is_err());
    assert!(vfs.is_crashed());

    // Auf dem Datenträger liegt nur das Präfix. Der Port selbst ist nicht mehr erreichbar.
    assert_eq!(read_all(&DurableVfs, &p)?, b"abcd");
    assert!(f.len().is_err());
    Ok(())
}

#[test]
fn crash_blocks_every_later_operation() -> io::Result<()> {
    let dir = common::scratch("crash_blocks")?;
    let vfs = FaultVfs::new(DurableVfs);
    vfs.arm(FaultPlan {
        point: FaultPoint::Create,
        skip: 0,
        action: FaultAction::Crash,
    });
    assert!(vfs.create_new(&dir.join("a")).is_err());
    assert!(vfs.is_crashed());
    assert!(vfs.list(&dir).is_err());
    assert!(vfs.open_read(&dir.join("a")).is_err());
    // Die Datei wurde nie angelegt.
    assert!(DurableVfs.list(&dir)?.is_empty());
    Ok(())
}

#[test]
fn atomic_replace_is_all_or_nothing_under_crash() -> io::Result<()> {
    // Absturz an jedem Operationspunkt des Ersetzungsablaufs. Vor dem Umbenennen
    // ist das Ziel noch unverändert, nach dem Umbenennen ist es vollständig neu.
    // Der Absturz liegt hier immer vor dem Umbenennen, also muss das Ziel "old" sein.
    let points = [
        FaultPoint::Create,
        FaultPoint::Write,
        FaultPoint::Sync,
        FaultPoint::Rename,
    ];
    for point in points {
        let dir = common::scratch(&format!("replace_{point:?}"))?;
        let target = dir.join("state");
        write_new(&DurableVfs, &target, b"old")?;

        let vfs = FaultVfs::new(DurableVfs);
        vfs.arm(FaultPlan {
            point,
            skip: 0,
            action: FaultAction::Crash,
        });
        assert!(
            replace(&vfs, &dir, &target).is_err(),
            "crash at {point:?} must abort the replace"
        );
        assert_eq!(
            read_all(&DurableVfs, &target)?,
            b"old",
            "crash at {point:?} left the target partially replaced"
        );
    }

    // Positivkontrolle: ohne Störung ersetzt der Ablauf das Ziel vollständig.
    let dir = common::scratch("replace_clean")?;
    let target = dir.join("state");
    write_new(&DurableVfs, &target, b"old")?;
    replace(&FaultVfs::new(DurableVfs), &dir, &target)?;
    assert_eq!(read_all(&DurableVfs, &target)?, b"new");
    Ok(())
}
