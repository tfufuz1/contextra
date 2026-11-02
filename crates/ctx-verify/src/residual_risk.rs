//! Residual risk catalog verification per SPEC Anhang D.

use std::collections::BTreeSet;
use std::str::FromStr;

/// Canonical Residual Risks per SPEC Anhang D.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ResidualRisk {
    FlashRemanence,
    FsJournalOrCow,
    WrappedKeyRemanence,
    FreedBlockRemanence,
    OldKeyslotRemanence,
    ProcessMemory,
    IndexTopologyInference,
    TextIndexUntilPurge,
    ReplicaWindow,
    ExternalCopies,
    ExternalProcessors,
    PseudonymCommitment,
    PurgeScalesWithDb,
    PurgeDeferred,
    KeyCompromise,
}

impl ResidualRisk {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::FlashRemanence => "FlashRemanence",
            Self::FsJournalOrCow => "FsJournalOrCow",
            Self::WrappedKeyRemanence => "WrappedKeyRemanence",
            Self::FreedBlockRemanence => "FreedBlockRemanence",
            Self::OldKeyslotRemanence => "OldKeyslotRemanence",
            Self::ProcessMemory => "ProcessMemory",
            Self::IndexTopologyInference => "IndexTopologyInference",
            Self::TextIndexUntilPurge => "TextIndexUntilPurge",
            Self::ReplicaWindow => "ReplicaWindow",
            Self::ExternalCopies => "ExternalCopies",
            Self::ExternalProcessors => "ExternalProcessors",
            Self::PseudonymCommitment => "PseudonymCommitment",
            Self::PurgeScalesWithDb => "PurgeScalesWithDb",
            Self::PurgeDeferred => "PurgeDeferred",
            Self::KeyCompromise => "KeyCompromise",
        }
    }

    pub fn parse_risk(s: &str) -> Option<Self> {
        match s {
            "FlashRemanence" => Some(Self::FlashRemanence),
            "FsJournalOrCow" => Some(Self::FsJournalOrCow),
            "WrappedKeyRemanence" => Some(Self::WrappedKeyRemanence),
            "FreedBlockRemanence" => Some(Self::FreedBlockRemanence),
            "OldKeyslotRemanence" => Some(Self::OldKeyslotRemanence),
            "ProcessMemory" => Some(Self::ProcessMemory),
            "IndexTopologyInference" => Some(Self::IndexTopologyInference),
            "TextIndexUntilPurge" => Some(Self::TextIndexUntilPurge),
            "ReplicaWindow" => Some(Self::ReplicaWindow),
            "ExternalCopies" => Some(Self::ExternalCopies),
            "ExternalProcessors" => Some(Self::ExternalProcessors),
            "PseudonymCommitment" => Some(Self::PseudonymCommitment),
            "PurgeScalesWithDb" => Some(Self::PurgeScalesWithDb),
            "PurgeDeferred" => Some(Self::PurgeDeferred),
            "KeyCompromise" => Some(Self::KeyCompromise),
            _ => None,
        }
    }
}

impl FromStr for ResidualRisk {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_risk(s).ok_or("unknown residual risk identifier")
    }
}

impl ResidualRisk {
    /// Returns list of unconditional risks that must always be present in any report (Anhang D).
    pub fn always_required() -> &'static [ResidualRisk] {
        &[
            ResidualRisk::FlashRemanence,
            ResidualRisk::WrappedKeyRemanence,
            ResidualRisk::FreedBlockRemanence,
            ResidualRisk::ReplicaWindow,
            ResidualRisk::ExternalCopies,
            ResidualRisk::ExternalProcessors,
            ResidualRisk::PseudonymCommitment,
            ResidualRisk::KeyCompromise,
        ]
    }
}

/// Verifies that all mandatory residual risks are present in the report.
pub fn verify_residual_risks(reported_risks: &[String]) -> Result<(), &'static str> {
    let mut found = BTreeSet::new();
    for r in reported_risks {
        if let Some(risk) = ResidualRisk::parse_risk(r) {
            found.insert(risk);
        } else {
            return Err("unknown residual risk identifier");
        }
    }

    for req in ResidualRisk::always_required() {
        if !found.contains(req) {
            return Err("missing mandatory residual risk per Anhang D");
        }
    }

    Ok(())
}
