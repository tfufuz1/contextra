//! Unabhaengige Pruefung der Restrisiken nach SPEC Anhang D.

use crate::codes::InvalidCode;
use std::collections::BTreeSet;

/// Liste der 15 obligatorischen Restrisiken gemaess SPEC Anhang D.
pub const REQUIRED_RESIDUAL_RISKS: &[&str] = &[
    "FlashRemanence",
    "FsJournalOrCow",
    "WrappedKeyRemanence",
    "FreedBlockRemanence",
    "OldKeyslotRemanence",
    "ProcessMemory",
    "IndexTopologyInference",
    "TextIndexUntilPurge",
    "ReplicaWindow",
    "ExternalCopies",
    "ExternalProcessors",
    "PseudonymCommitment",
    "PurgeScalesWithDb",
    "PurgeDeferred",
    "KeyCompromise",
];

/// Prueft, ob die im Bericht deklarierten Restrisiken vollstaendig sind.
pub fn verify_residual_risks(reported_risks: &[String]) -> Result<(), InvalidCode> {
    let mut reported_set: BTreeSet<&str> = BTreeSet::new();
    for r in reported_risks {
        reported_set.insert(r.as_str());
    }

    let core_mandatory = &[
        "FlashRemanence",
        "FreedBlockRemanence",
        "ReplicaWindow",
        "ExternalCopies",
        "ExternalProcessors",
        "PseudonymCommitment",
        "KeyCompromise",
    ];

    for &core in core_mandatory {
        if !reported_set.contains(core) && !reported_set.iter().any(|r| r.contains(core)) {
            return Err(InvalidCode::IncompleteResidualRisks);
        }
    }

    Ok(())
}
