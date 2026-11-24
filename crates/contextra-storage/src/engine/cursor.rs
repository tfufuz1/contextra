//! View/Stamp-bound cursor, resume tokens and block pruning (SPEC §7, §11.4, INV-VIEW-3, INV-PRUNE-1).
#![forbid(unsafe_code)]

use crate::erase::gate::GateIndex;
use contextra_storage_ports::{BranchId, ClusterId, Error, Stamp};

/// Optional hook for skipping entire data blocks without decrypting payloads (SPEC §12.2, K-29).
pub trait BlockPruner: Send + Sync {
    fn should_prune(&self, min_key: &[u8], max_key: &[u8]) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeToken {
    pub cluster: ClusterId,
    pub branch: BranchId,
    pub last_key: Vec<u8>,
    pub view_stamp: Stamp,
}

impl ResumeToken {
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(32 + self.last_key.len());
        buf.extend_from_slice(b"CTXSRTK1");
        buf.extend_from_slice(self.cluster.as_bytes());
        buf.extend_from_slice(&self.branch.0.to_be_bytes());
        buf.extend_from_slice(&self.view_stamp.to_be_bytes());
        buf.extend_from_slice(&(self.last_key.len() as u32).to_be_bytes());
        buf.extend_from_slice(&self.last_key);
        buf
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 8 + 16 + 4 + 8 + 4 || &bytes[..8] != b"CTXSRTK1" {
            return Err(Error::Corrupt {
                place: "invalid resume token format",
            });
        }
        let mut cluster_raw = [0u8; 16];
        cluster_raw.copy_from_slice(&bytes[8..24]);
        let cluster = ClusterId::from_wire(cluster_raw);
        let branch = BranchId(u32::from_be_bytes(bytes[24..28].try_into().map_err(
            |_| Error::Corrupt {
                place: "resume token branch",
            },
        )?));
        let view_stamp = u64::from_be_bytes(bytes[28..36].try_into().map_err(|_| {
            Error::Corrupt {
                place: "resume token stamp",
            }
        })?);
        let key_len = u32::from_be_bytes(bytes[36..40].try_into().map_err(|_| {
            Error::Corrupt {
                place: "resume token key length",
            }
        })?) as usize;
        let expected_len = 40usize
            .checked_add(key_len)
            .ok_or(Error::Corrupt {
                place: "resume token length overflow",
            })?;
        if bytes.len() < expected_len {
            return Err(Error::Corrupt {
                place: "resume token key truncated",
            });
        }
        if bytes.len() > expected_len {
            return Err(Error::Corrupt {
                place: "resume token trailing bytes",
            });
        }
        let last_key = bytes[40..expected_len].to_vec();
        Ok(Self {
            cluster,
            branch,
            last_key,
            view_stamp,
        })
    }
}

pub struct Cursor {
    cluster: ClusterId,
    branch: BranchId,
    view_stamp: Stamp,
    lease_deadline_mono: u64,
    last_key: Option<Vec<u8>>,
    exhausted: bool,
}

impl Cursor {
    pub fn new(
        cluster: ClusterId,
        branch: BranchId,
        view_stamp: Stamp,
        lease_deadline_mono: u64,
    ) -> Self {
        Self {
            cluster,
            branch,
            view_stamp,
            lease_deadline_mono,
            last_key: None,
            exhausted: false,
        }
    }

    pub fn from_resume_token(token: ResumeToken, lease_deadline_mono: u64) -> Self {
        Self {
            cluster: token.cluster,
            branch: token.branch,
            view_stamp: token.view_stamp,
            lease_deadline_mono,
            last_key: if token.last_key.is_empty() {
                None
            } else {
                Some(token.last_key)
            },
            exhausted: false,
        }
    }

    pub fn cluster(&self) -> ClusterId {
        self.cluster
    }

    pub fn branch(&self) -> BranchId {
        self.branch
    }

    pub fn view_stamp(&self) -> Stamp {
        self.view_stamp
    }

    pub fn is_exhausted(&self) -> bool {
        self.exhausted
    }

    pub fn last_key(&self) -> Option<&[u8]> {
        self.last_key.as_deref()
    }

    /// Renews lease deadline up to max allowed duration (SPEC §7).
    pub fn renew_lease(&mut self, next_deadline_mono: u64) {
        if next_deadline_mono > self.lease_deadline_mono {
            self.lease_deadline_mono = next_deadline_mono;
        }
    }

    /// Step boundary check: checks lease expiry and cluster erasure gate before proceeding.
    pub fn check_validity(&self, now_mono: u64, gate_index: &GateIndex) -> Result<(), Error> {
        if now_mono >= self.lease_deadline_mono {
            return Err(Error::BelowFloor);
        }
        gate_index.check_visible(self.cluster, self.view_stamp)?;
        Ok(())
    }

    /// Advances the cursor with the latest visited key.
    pub fn step(&mut self, key: Vec<u8>) {
        self.last_key = Some(key);
    }

    pub fn mark_exhausted(&mut self) {
        self.exhausted = true;
    }

    /// Evaluates the optional BlockPruner only for ShadowSafe owners on primary branch 0 (SPEC §12.2, K-29).
    pub fn may_prune_block(
        &self,
        pruner: Option<&dyn BlockPruner>,
        shadow_safe: bool,
        min_key: &[u8],
        max_key: &[u8],
    ) -> bool {
        if !shadow_safe || self.branch != BranchId(0) {
            return false;
        }
        pruner.is_some_and(|p| p.should_prune(min_key, max_key))
    }

    /// Generates a resume token representing the current cursor state.
    pub fn resume_token(&self) -> Option<ResumeToken> {
        self.last_key.as_ref().map(|k| ResumeToken {
            cluster: self.cluster,
            branch: self.branch,
            last_key: k.clone(),
            view_stamp: self.view_stamp,
        })
    }
}
