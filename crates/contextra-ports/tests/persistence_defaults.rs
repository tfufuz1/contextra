//! Verträge der Standardmethoden im Port-Modul `persistence` (SPEC §4, SA-2, SA-3).
//!
//! Die Tests laufen mit allen Features und mit `--features persistence` allein.

use std::io;
use std::path::Path;

use contextra_ports::persistence::{File, Fs};

/// Minimale Datei: nur die Pflichtmethoden, alle Standardmethoden bleiben.
struct NullFile;

impl File for NullFile {
    fn read_at(&self, _buf: &mut [u8], _off: u64) -> io::Result<usize> {
        Ok(0)
    }
    fn write_all_at(&self, _buf: &[u8], _off: u64) -> io::Result<()> {
        Ok(())
    }
    fn truncate(&self, _len: u64) -> io::Result<()> {
        Ok(())
    }
    fn len(&self) -> io::Result<u64> {
        Ok(0)
    }
    fn sync_data(&self) -> io::Result<()> {
        Ok(())
    }
    fn sync_all(&self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn punch_hole_standard_ist_unsupported() {
    assert!(matches!(
        NullFile.punch_hole(0, 1),
        Err(e) if e.kind() == io::ErrorKind::Unsupported
    ));
}

#[test]
fn read_batch_standard_nullt_den_rest() {
    let f = NullFile;
    let mut buf = [0xFFu8; 4];
    let mut reqs = [contextra_ports::persistence::ReadReq {
        buf: &mut buf,
        offset: 0,
    }];
    assert!(f.read_batch(&mut reqs).is_ok());
    assert_eq!(buf, [0u8; 4], "nicht gelesene Bytes werden genullt");
}

/// Dateisystem ohne Überschreibungen: nur die Pflichtmethoden.
struct NullFs;

impl Fs for NullFs {
    fn create_dir_all(&self, _p: &Path, _perm: contextra_ports::persistence::Perm) -> io::Result<()> {
        Ok(())
    }
    fn open(
        &self,
        _p: &Path,
        _mode: contextra_ports::persistence::OpenMode,
    ) -> io::Result<Box<dyn File>> {
        Ok(Box::new(NullFile))
    }
    fn rename(&self, _from: &Path, _to: &Path) -> io::Result<()> {
        Ok(())
    }
    fn remove_file(&self, _p: &Path) -> io::Result<()> {
        Ok(())
    }
    fn remove_dir(&self, _p: &Path) -> io::Result<()> {
        Ok(())
    }
    fn list_dir(&self, _p: &Path) -> io::Result<Vec<contextra_ports::persistence::DirEntry>> {
        Ok(Vec::new())
    }
    fn sync_dir(&self, _p: &Path) -> io::Result<()> {
        Ok(())
    }
    fn lock(
        &self,
        _p: &Path,
        _mode: contextra_ports::persistence::LockMode,
    ) -> io::Result<Box<dyn contextra_ports::persistence::LockGuard>> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }
    fn caps(&self) -> contextra_ports::persistence::FsCaps {
        contextra_ports::persistence::FsCaps::default()
    }
}

#[test]
fn hard_link_und_reflink_standard_sind_unsupported() {
    let fs = NullFs;
    let from = Path::new("a");
    let to = Path::new("b");
    assert!(matches!(
        fs.hard_link(from, to),
        Err(e) if e.kind() == io::ErrorKind::Unsupported
    ));
    assert!(matches!(
        fs.reflink(from, to),
        Err(e) if e.kind() == io::ErrorKind::Unsupported
    ));
}
