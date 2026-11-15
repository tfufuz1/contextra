//! Verträge von `DurableVfs` (Spec 3.7, WP-5 bis WP-7, 16.2 Einheits- und Integrationstests).

mod common;

use acta_fs::durable::{scrub_file, DurableVfs};
use acta_fs::vfs::Vfs;
use std::io::{self, ErrorKind};

#[test]
fn write_then_read_at_offsets_roundtrips() -> io::Result<()> {
    let dir = common::scratch("roundtrip")?;
    let f = DurableVfs.create_new(&dir.join("a"))?;
    f.write_all_at(0, b"hello")?;
    f.write_all_at(10, b"world")?;
    assert_eq!(f.len()?, 15);

    let mut tail = [0_u8; 5];
    f.read_exact_at(10, &mut tail)?;
    assert_eq!(&tail, b"world");

    // Der Bereich zwischen den Schreibvorgängen ist mit Nullen gefüllt.
    let mut gap = [9_u8; 5];
    f.read_exact_at(5, &mut gap)?;
    assert_eq!(gap, [0_u8; 5]);
    Ok(())
}

#[test]
fn create_new_refuses_an_existing_file() -> io::Result<()> {
    let dir = common::scratch("create_new")?;
    let p = dir.join("a");
    DurableVfs.create_new(&p)?;
    let err = DurableVfs
        .create_new(&p)
        .err()
        .ok_or_else(|| io::Error::other("second create_new succeeded"))?;
    assert_eq!(err.kind(), ErrorKind::AlreadyExists);
    Ok(())
}

#[test]
fn read_past_end_fails_with_unexpected_eof() -> io::Result<()> {
    let dir = common::scratch("eof")?;
    let p = dir.join("a");
    DurableVfs.create_new(&p)?.write_all_at(0, b"abc")?;
    let f = DurableVfs.open_read(&p)?;
    let mut buf = [0_u8; 4];
    let err = f
        .read_exact_at(0, &mut buf)
        .err()
        .ok_or_else(|| io::Error::other("short file read succeeded"))?;
    assert_eq!(err.kind(), ErrorKind::UnexpectedEof);
    Ok(())
}

#[test]
fn rename_atomic_replaces_the_target() -> io::Result<()> {
    let dir = common::scratch("rename")?;
    let target = dir.join("state");
    let tmp = dir.join("state.tmp");
    DurableVfs.create_new(&target)?.write_all_at(0, b"old")?;
    let new = DurableVfs.create_new(&tmp)?;
    new.write_all_at(0, b"new")?;
    new.sync_data().map_err(io::Error::other)?;

    DurableVfs.rename_atomic(&tmp, &target)?;

    let f = DurableVfs.open_read(&target)?;
    let mut buf = [0_u8; 3];
    f.read_exact_at(0, &mut buf)?;
    assert_eq!(&buf, b"new");
    let gone = DurableVfs
        .open_read(&tmp)
        .err()
        .ok_or_else(|| io::Error::other("tmp still exists after rename"))?;
    assert_eq!(gone.kind(), ErrorKind::NotFound);
    Ok(())
}

#[test]
fn list_returns_full_paths_in_ascending_order() -> io::Result<()> {
    let dir = common::scratch("list")?;
    for name in ["c", "a", "b"] {
        DurableVfs.create_new(&dir.join(name))?;
    }
    let names: Vec<String> = DurableVfs
        .list(&dir)?
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    assert_eq!(names, ["a", "b", "c"]);
    Ok(())
}

#[test]
fn scrub_zeroes_content_before_removal() -> io::Result<()> {
    let dir = common::scratch("scrub")?;
    let p = dir.join("secret");
    DurableVfs.create_new(&p)?.write_all_at(0, b"secret")?;

    // Positivkontrolle: Der Inhalt ist vor dem Entfernen sichtbar und nach dem Scrub Null.
    scrub_file(&p)?;
    let mut buf = [1_u8; 6];
    DurableVfs.open_read(&p)?.read_exact_at(0, &mut buf)?;
    assert_eq!(buf, [0_u8; 6]);

    DurableVfs.remove_scrub(&p)?;
    let gone = DurableVfs
        .open_read(&p)
        .err()
        .ok_or_else(|| io::Error::other("scrubbed file still exists"))?;
    assert_eq!(gone.kind(), ErrorKind::NotFound);
    Ok(())
}

#[test]
fn sync_data_succeeds_on_a_healthy_file() -> io::Result<()> {
    let dir = common::scratch("sync")?;
    let f = DurableVfs.create_new(&dir.join("a"))?;
    f.write_all_at(0, b"durable")?;
    f.sync_data().map_err(io::Error::other)?;
    f.sync_data().map_err(io::Error::other)?;
    Ok(())
}

#[test]
fn direct_read_matches_buffered_read_at_unaligned_offsets() -> io::Result<()> {
    let dir = common::scratch("direct")?;
    let p = dir.join("pattern");
    let len = 3 * 4096 + 123;
    let pattern: Vec<u8> = (0..len).map(|i| (i % 251) as u8).collect();
    let w = DurableVfs.create_new(&p)?;
    w.write_all_at(0, &pattern)?;
    w.sync_data().map_err(io::Error::other)?;

    let direct = DurableVfs.open_direct_read(&p)?;
    // Ausgerichtete, unausgerichtete, blockübergreifende und Endbereiche.
    for (off, n) in [
        (0_usize, 4096_usize),
        (4090, 100),
        (8190, 2),
        (len - 77, 77),
        (1, 1),
    ] {
        let mut got = vec![0_u8; n];
        direct.read_exact_at(off as u64, &mut got)?;
        assert_eq!(got, &pattern[off..off + n], "offset {off}, length {n}");
    }

    let mut beyond = [0_u8; 8];
    let err = direct
        .read_exact_at(len as u64 - 4, &mut beyond)
        .err()
        .ok_or_else(|| io::Error::other("direct read past end succeeded"))?;
    assert_eq!(err.kind(), ErrorKind::UnexpectedEof);
    Ok(())
}
