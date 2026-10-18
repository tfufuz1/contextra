// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Meta-tests proving that the DST runner detects intentionally buggy SUT implementations.
// INVARIANTEN: Every faulty SUT MUST trigger a divergence error; NaiveWalSut MUST pass 2,000 seeds.

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use contextra_testkit::dst::op::DstOp;
use contextra_testkit::dst::p07_runner::P07DstRunner;
use contextra_testkit::dst::p07_sut::{NaiveWalSut, SystemUnderTest};
use contextra_testkit::sim_fs::{SimFs, SimOpenOptions, SimVfs};
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

// -----------------------------------------------------------------------------
// Buggy SUT (a): WAL that NEVER calls fsync
// -----------------------------------------------------------------------------
struct FaultyWalNoFsync {
    sim_fs: SimFs,
    wal_path: PathBuf,
    active_map: BTreeMap<Vec<u8>, Vec<u8>>,
    staged: Vec<DstOp>,
    seq: u64,
}

impl FaultyWalNoFsync {
    fn new(sim_fs: SimFs) -> Self {
        Self {
            sim_fs,
            wal_path: PathBuf::from("no_fsync.log"),
            active_map: BTreeMap::new(),
            staged: Vec::new(),
            seq: 0,
        }
    }
}

impl SystemUnderTest for FaultyWalNoFsync {
    fn apply(&mut self, op: &DstOp) -> io::Result<()> {
        match op {
            DstOp::Put { key, value } => self.staged.push(DstOp::Put {
                key: key.clone(),
                value: value.clone(),
            }),
            DstOp::Delete { key } => self.staged.push(DstOp::Delete { key: key.clone() }),
            DstOp::Forget { key } => self.staged.push(DstOp::Forget { key: key.clone() }),
            DstOp::Rollback => self.staged.clear(),
            DstOp::Commit => {
                self.commit()?;
            }
            DstOp::Sync => {}
        }
        Ok(())
    }

    fn commit(&mut self) -> io::Result<u64> {
        if self.staged.is_empty() {
            return Ok(self.seq);
        }
        let mut opts = SimOpenOptions::new();
        opts.create(true).write(true);
        let file = self.sim_fs.open(&self.wal_path, &opts)?;
        let offset = file.len()?;

        self.seq += 1;
        let mut payload = Vec::new();
        for op in self.staged.drain(..) {
            match op {
                DstOp::Put { key, value } => {
                    payload.push(1u8);
                    payload.extend_from_slice(&(key.len() as u32).to_le_bytes());
                    payload.extend_from_slice(&key);
                    payload.extend_from_slice(&(value.len() as u32).to_le_bytes());
                    payload.extend_from_slice(&value);
                    self.active_map.insert(key, value);
                }
                DstOp::Delete { key } | DstOp::Forget { key } => {
                    payload.push(2u8);
                    payload.extend_from_slice(&(key.len() as u32).to_le_bytes());
                    payload.extend_from_slice(&key);
                    payload.extend_from_slice(&0u32.to_le_bytes());
                    self.active_map.remove(&key);
                }
                _ => {}
            }
        }

        // Write without calling file.sync_data() or sim_fs.sync_dir()!
        file.write_at(&payload, offset)?;
        Ok(self.seq)
    }

    fn crash_and_recover(&mut self, sim_fs: &SimFs) -> io::Result<()> {
        self.sim_fs = sim_fs.clone();
        self.active_map.clear();
        self.staged.clear();

        let mut opts = SimOpenOptions::new();
        opts.read(true);
        let file = match self.sim_fs.open(&self.wal_path, &opts) {
            Ok(f) => f,
            Err(_) => return Ok(()),
        };

        let file_len = file.len()? as usize;
        let mut buf = vec![0u8; file_len];
        let read = file.read_at(&mut buf, 0)?;
        buf.truncate(read);

        let mut offset = 0;
        while offset < buf.len() {
            if offset + 1 > buf.len() {
                break;
            }
            let tag = buf[offset];
            offset += 1;
            if offset + 4 > buf.len() {
                break;
            }
            let klen =
                u32::from_le_bytes(buf[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
            offset += 4;
            if offset + klen > buf.len() {
                break;
            }
            let key = buf[offset..offset + klen].to_vec();
            offset += klen;
            if offset + 4 > buf.len() {
                break;
            }
            let vlen =
                u32::from_le_bytes(buf[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
            offset += 4;
            if offset + vlen > buf.len() {
                break;
            }
            let val = buf[offset..offset + vlen].to_vec();
            offset += vlen;

            if tag == 1 {
                self.active_map.insert(key, val);
            } else {
                self.active_map.remove(&key);
            }
        }

        Ok(())
    }

    fn snapshot(&self) -> BTreeMap<Vec<u8>, Vec<u8>> {
        self.active_map.clone()
    }
}

// -----------------------------------------------------------------------------
// Buggy SUT (b): WAL that renames without calling sync_dir
// -----------------------------------------------------------------------------
struct FaultyWalNoDirSync {
    sim_fs: SimFs,
    tmp_path: PathBuf,
    log_path: PathBuf,
    active_map: BTreeMap<Vec<u8>, Vec<u8>>,
    staged: Vec<DstOp>,
}

impl FaultyWalNoDirSync {
    fn new(sim_fs: SimFs) -> Self {
        Self {
            sim_fs,
            tmp_path: PathBuf::from("wal.tmp"),
            log_path: PathBuf::from("wal.log"),
            active_map: BTreeMap::new(),
            staged: Vec::new(),
        }
    }
}

impl SystemUnderTest for FaultyWalNoDirSync {
    fn apply(&mut self, op: &DstOp) -> io::Result<()> {
        match op {
            DstOp::Put { key, value } => self.staged.push(DstOp::Put {
                key: key.clone(),
                value: value.clone(),
            }),
            DstOp::Delete { key } => self.staged.push(DstOp::Delete { key: key.clone() }),
            DstOp::Forget { key } => self.staged.push(DstOp::Forget { key: key.clone() }),
            DstOp::Rollback => self.staged.clear(),
            DstOp::Commit => {
                self.commit()?;
            }
            DstOp::Sync => {}
        }
        Ok(())
    }

    fn commit(&mut self) -> io::Result<u64> {
        if self.staged.is_empty() {
            return Ok(1);
        }

        let mut opts = SimOpenOptions::new();
        opts.create(true).write(true).truncate(true);
        let file = self.sim_fs.open(&self.tmp_path, &opts)?;

        let mut payload = Vec::new();
        for op in self.staged.drain(..) {
            match op {
                DstOp::Put { key, value } => {
                    payload.push(1u8);
                    payload.extend_from_slice(&(key.len() as u32).to_le_bytes());
                    payload.extend_from_slice(&key);
                    payload.extend_from_slice(&(value.len() as u32).to_le_bytes());
                    payload.extend_from_slice(&value);
                    self.active_map.insert(key, value);
                }
                DstOp::Delete { key } | DstOp::Forget { key } => {
                    payload.push(2u8);
                    payload.extend_from_slice(&(key.len() as u32).to_le_bytes());
                    payload.extend_from_slice(&key);
                    payload.extend_from_slice(&0u32.to_le_bytes());
                    self.active_map.remove(&key);
                }
                _ => {}
            }
        }

        file.write_at(&payload, 0)?;
        file.sync_data()?;

        // Rename without parent sync_dir!
        self.sim_fs.rename(&self.tmp_path, &self.log_path)?;
        Ok(1)
    }

    fn crash_and_recover(&mut self, sim_fs: &SimFs) -> io::Result<()> {
        self.sim_fs = sim_fs.clone();
        self.active_map.clear();

        let mut opts = SimOpenOptions::new();
        opts.read(true);
        let file = match self.sim_fs.open(&self.log_path, &opts) {
            Ok(f) => f,
            Err(_) => return Ok(()),
        };

        let file_len = file.len()? as usize;
        let mut buf = vec![0u8; file_len];
        let read = file.read_at(&mut buf, 0)?;
        buf.truncate(read);

        let mut offset = 0;
        while offset < buf.len() {
            if offset + 1 > buf.len() {
                break;
            }
            let tag = buf[offset];
            offset += 1;
            if offset + 4 > buf.len() {
                break;
            }
            let klen =
                u32::from_le_bytes(buf[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
            offset += 4;
            if offset + klen > buf.len() {
                break;
            }
            let key = buf[offset..offset + klen].to_vec();
            offset += klen;
            if offset + 4 > buf.len() {
                break;
            }
            let vlen =
                u32::from_le_bytes(buf[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
            offset += 4;
            if offset + vlen > buf.len() {
                break;
            }
            let val = buf[offset..offset + vlen].to_vec();
            offset += vlen;

            if tag == 1 {
                self.active_map.insert(key, val);
            } else {
                self.active_map.remove(&key);
            }
        }

        Ok(())
    }

    fn snapshot(&self) -> BTreeMap<Vec<u8>, Vec<u8>> {
        self.active_map.clone()
    }
}

// -----------------------------------------------------------------------------
// Buggy SUT (c): WAL that ignores fsync errors (fsyncgate)
// -----------------------------------------------------------------------------
struct FaultyWalFsyncGate {
    sim_fs: SimFs,
    wal_path: PathBuf,
    active_map: BTreeMap<Vec<u8>, Vec<u8>>,
    staged: Vec<DstOp>,
    seq: u64,
}

impl FaultyWalFsyncGate {
    fn new(sim_fs: SimFs) -> Self {
        Self {
            sim_fs,
            wal_path: PathBuf::from("fsyncgate.log"),
            active_map: BTreeMap::new(),
            staged: Vec::new(),
            seq: 0,
        }
    }
}

impl SystemUnderTest for FaultyWalFsyncGate {
    fn apply(&mut self, op: &DstOp) -> io::Result<()> {
        match op {
            DstOp::Put { key, value } => self.staged.push(DstOp::Put {
                key: key.clone(),
                value: value.clone(),
            }),
            DstOp::Delete { key } => self.staged.push(DstOp::Delete { key: key.clone() }),
            DstOp::Forget { key } => self.staged.push(DstOp::Forget { key: key.clone() }),
            DstOp::Rollback => self.staged.clear(),
            DstOp::Commit => {
                self.commit()?;
            }
            DstOp::Sync => {}
        }
        Ok(())
    }

    fn commit(&mut self) -> io::Result<u64> {
        if self.staged.is_empty() {
            return Ok(self.seq);
        }

        let mut opts = SimOpenOptions::new();
        opts.create(true).write(true);
        let file = self.sim_fs.open(&self.wal_path, &opts)?;
        let offset = file.len()?;

        self.seq += 1;
        let mut payload = Vec::new();
        for op in self.staged.drain(..) {
            match op {
                DstOp::Put { key, value } => {
                    payload.push(1u8);
                    payload.extend_from_slice(&(key.len() as u32).to_le_bytes());
                    payload.extend_from_slice(&key);
                    payload.extend_from_slice(&(value.len() as u32).to_le_bytes());
                    payload.extend_from_slice(&value);
                    self.active_map.insert(key, value);
                }
                DstOp::Delete { key } | DstOp::Forget { key } => {
                    payload.push(2u8);
                    payload.extend_from_slice(&(key.len() as u32).to_le_bytes());
                    payload.extend_from_slice(&key);
                    payload.extend_from_slice(&0u32.to_le_bytes());
                    self.active_map.remove(&key);
                }
                _ => {}
            }
        }

        file.write_at(&payload, offset)?;

        // Fsync error ignored! Returns Ok success regardless of fsync error!
        let _ = file.sync_data();
        let _ = self.sim_fs.sync_dir(Path::new("."));

        Ok(self.seq)
    }

    fn crash_and_recover(&mut self, sim_fs: &SimFs) -> io::Result<()> {
        self.sim_fs = sim_fs.clone();
        self.active_map.clear();

        let mut opts = SimOpenOptions::new();
        opts.read(true);
        let file = match self.sim_fs.open(&self.wal_path, &opts) {
            Ok(f) => f,
            Err(_) => return Ok(()),
        };

        let file_len = file.len()? as usize;
        let mut buf = vec![0u8; file_len];
        let read = file.read_at(&mut buf, 0)?;
        buf.truncate(read);

        let mut offset = 0;
        while offset < buf.len() {
            if offset + 1 > buf.len() {
                break;
            }
            let tag = buf[offset];
            offset += 1;
            if offset + 4 > buf.len() {
                break;
            }
            let klen =
                u32::from_le_bytes(buf[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
            offset += 4;
            if offset + klen > buf.len() {
                break;
            }
            let key = buf[offset..offset + klen].to_vec();
            offset += klen;
            if offset + 4 > buf.len() {
                break;
            }
            let vlen =
                u32::from_le_bytes(buf[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
            offset += 4;
            if offset + vlen > buf.len() {
                break;
            }
            let val = buf[offset..offset + vlen].to_vec();
            offset += vlen;

            if tag == 1 {
                self.active_map.insert(key, val);
            } else {
                self.active_map.remove(&key);
            }
        }

        Ok(())
    }

    fn snapshot(&self) -> BTreeMap<Vec<u8>, Vec<u8>> {
        self.active_map.clone()
    }
}

// -----------------------------------------------------------------------------
// Buggy SUT (d): WAL that partially applies a transaction
// -----------------------------------------------------------------------------
struct FaultyWalPartialCommit {
    sim_fs: SimFs,
    wal_path: PathBuf,
    active_map: BTreeMap<Vec<u8>, Vec<u8>>,
    staged: Vec<DstOp>,
}

impl FaultyWalPartialCommit {
    fn new(sim_fs: SimFs) -> Self {
        Self {
            sim_fs,
            wal_path: PathBuf::from("partial.log"),
            active_map: BTreeMap::new(),
            staged: Vec::new(),
        }
    }
}

impl SystemUnderTest for FaultyWalPartialCommit {
    fn apply(&mut self, op: &DstOp) -> io::Result<()> {
        match op {
            DstOp::Put { key, value } => self.staged.push(DstOp::Put {
                key: key.clone(),
                value: value.clone(),
            }),
            DstOp::Delete { key } => self.staged.push(DstOp::Delete { key: key.clone() }),
            DstOp::Forget { key } => self.staged.push(DstOp::Forget { key: key.clone() }),
            DstOp::Rollback => self.staged.clear(),
            DstOp::Commit => {
                self.commit()?;
            }
            DstOp::Sync => {}
        }
        Ok(())
    }

    fn commit(&mut self) -> io::Result<u64> {
        if self.staged.is_empty() {
            return Ok(1);
        }

        let mut opts = SimOpenOptions::new();
        opts.create(true).write(true);
        let file = self.sim_fs.open(&self.wal_path, &opts)?;
        let mut offset = file.len()?;

        // Write each operation individually without commit record or transaction atomicity
        for op in self.staged.drain(..) {
            let mut payload = Vec::new();
            match op {
                DstOp::Put { key, value } => {
                    payload.push(1u8);
                    payload.extend_from_slice(&(key.len() as u32).to_le_bytes());
                    payload.extend_from_slice(&key);
                    payload.extend_from_slice(&(value.len() as u32).to_le_bytes());
                    payload.extend_from_slice(&value);
                    self.active_map.insert(key, value);
                }
                DstOp::Delete { key } | DstOp::Forget { key } => {
                    payload.push(2u8);
                    payload.extend_from_slice(&(key.len() as u32).to_le_bytes());
                    payload.extend_from_slice(&key);
                    payload.extend_from_slice(&0u32.to_le_bytes());
                    self.active_map.remove(&key);
                }
                _ => continue,
            }
            file.write_at(&payload, offset)?;
            offset += payload.len() as u64;
        }

        let _ = file.sync_data();
        let _ = self.sim_fs.sync_dir(Path::new("."));
        Ok(1)
    }

    fn crash_and_recover(&mut self, sim_fs: &SimFs) -> io::Result<()> {
        self.sim_fs = sim_fs.clone();
        self.active_map.clear();

        let mut opts = SimOpenOptions::new();
        opts.read(true);
        let file = match self.sim_fs.open(&self.wal_path, &opts) {
            Ok(f) => f,
            Err(_) => return Ok(()),
        };

        let file_len = file.len()? as usize;
        let mut buf = vec![0u8; file_len];
        let read = file.read_at(&mut buf, 0)?;
        buf.truncate(read);

        let mut offset = 0;
        // Partially applies records encountered without checking commit boundaries
        while offset < buf.len() {
            if offset + 1 > buf.len() {
                break;
            }
            let tag = buf[offset];
            offset += 1;
            if offset + 4 > buf.len() {
                break;
            }
            let klen =
                u32::from_le_bytes(buf[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
            offset += 4;
            if offset + klen > buf.len() {
                break;
            }
            let key = buf[offset..offset + klen].to_vec();
            offset += klen;
            if offset + 4 > buf.len() {
                break;
            }
            let vlen =
                u32::from_le_bytes(buf[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
            offset += 4;
            if offset + vlen > buf.len() {
                break;
            }
            let val = buf[offset..offset + vlen].to_vec();
            offset += vlen;

            if tag == 1 {
                self.active_map.insert(key, val);
            } else {
                self.active_map.remove(&key);
            }
        }

        Ok(())
    }

    fn snapshot(&self) -> BTreeMap<Vec<u8>, Vec<u8>> {
        self.active_map.clone()
    }
}

// =============================================================================
// Meta-Test Assertions
// =============================================================================

#[test]
fn test_meta_detects_buggy_sut_a_no_fsync() {
    let mut divergence_found = false;
    for seed in 2000..2050 {
        if P07DstRunner::run_seed(seed, 40, |sim_fs| FaultyWalNoFsync::new(sim_fs.clone())).is_err()
        {
            divergence_found = true;
            break;
        }
    }
    assert!(
        divergence_found,
        "DST Runner MUST detect divergence in FaultyWalNoFsync (SUT without fsync)"
    );
}

#[test]
fn test_meta_detects_buggy_sut_b_no_dir_sync() {
    let mut divergence_found = false;
    for seed in 3000..3050 {
        if P07DstRunner::run_seed(seed, 40, |sim_fs| FaultyWalNoDirSync::new(sim_fs.clone()))
            .is_err()
        {
            divergence_found = true;
            break;
        }
    }
    assert!(
        divergence_found,
        "DST Runner MUST detect divergence in FaultyWalNoDirSync (SUT without parent sync_dir)"
    );
}

#[test]
fn test_meta_detects_buggy_sut_c_fsyncgate() {
    let mut divergence_found = false;
    for seed in 4000..4050 {
        if P07DstRunner::run_seed(seed, 40, |sim_fs| FaultyWalFsyncGate::new(sim_fs.clone()))
            .is_err()
        {
            divergence_found = true;
            break;
        }
    }
    assert!(
        divergence_found,
        "DST Runner MUST detect divergence in FaultyWalFsyncGate (SUT ignoring fsync errors)"
    );
}

#[test]
fn test_meta_detects_buggy_sut_d_partial_commit() {
    let mut divergence_found = false;
    for seed in 5000..5050 {
        if P07DstRunner::run_seed(seed, 40, |sim_fs| {
            FaultyWalPartialCommit::new(sim_fs.clone())
        })
        .is_err()
        {
            divergence_found = true;
            break;
        }
    }
    assert!(
        divergence_found,
        "DST Runner MUST detect divergence in FaultyWalPartialCommit (SUT partially applying txns)"
    );
}

#[test]
fn test_naive_wal_sut_passes_2000_seeds() {
    let mut passes = 0;
    for seed in 10_000..12_000 {
        let res = P07DstRunner::run_seed(seed, 30, |sim_fs| {
            NaiveWalSut::new(sim_fs.clone(), PathBuf::from("naive_wal_2k.log"))
        });
        assert!(
            res.is_ok(),
            "NaiveWalSut MUST pass seed {} without divergence, got: {:?}",
            seed,
            res.err()
        );
        passes += 1;
    }
    assert_eq!(passes, 2000, "NaiveWalSut MUST pass 2,000 seeds");
    println!("NaiveWalSut successfully passed 2,000 DST seeds!");
}
