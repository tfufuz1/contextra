use crate::error::StoreResult as Result;
use std::path::PathBuf;
use super::super::{WalEntry, WalVersion};

pub(crate) enum WalCommand {
    Append {
        payload: Vec<u8>,
        last_hmac_val: [u8; 32],
        ack: tokio::sync::oneshot::Sender<Result<()>>,
    },
    Truncate {
        offset: u64,
        new_last_hmac: [u8; 32],
        ack: tokio::sync::oneshot::Sender<Result<()>>,
    },
    Seal {
        ack: tokio::sync::oneshot::Sender<Result<PathBuf>>,
    },
    Rewrite {
        replayed_entries: Vec<(u64, WalEntry, u64)>,
        integrity_key: [u8; 32],
        ack: tokio::sync::oneshot::Sender<Result<()>>,
    },
    Scan {
        file_size: u64,
        item_tx: tokio::sync::mpsc::UnboundedSender<(u64, WalEntry, u64)>,
        ack: tokio::sync::oneshot::Sender<Result<WalVersion>>,
    },
}

impl std::fmt::Debug for WalCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Append {
                payload,
                last_hmac_val,
                ..
            } => f
                .debug_struct("Append")
                .field("payload_len", &payload.len())
                .field("last_hmac_val", last_hmac_val)
                .finish(),
            Self::Truncate {
                offset,
                new_last_hmac,
                ..
            } => f
                .debug_struct("Truncate")
                .field("offset", offset)
                .field("new_last_hmac", new_last_hmac)
                .finish(),
            Self::Seal { .. } => f.debug_struct("Seal").finish(),
            Self::Rewrite {
                replayed_entries, ..
            } => f
                .debug_struct("Rewrite")
                .field("replayed_entries_len", &replayed_entries.len())
                .finish(),
            Self::Scan { file_size, .. } => f
                .debug_struct("Scan")
                .field("file_size", file_size)
                .finish(),
        }
    }
}
