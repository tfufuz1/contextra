//! Domänenpräfixe (Spec 4.1). Jeder Zweck hat ein eigenes Präfix. Wiederverwendung ist verboten.

/// Tag einer Betroffenen-Kennung.
pub const TAG: &[u8] = b"acta/tag/v1";
/// Schlüssel für Tags.
pub const TAG_KEY: &[u8] = b"acta/tag-key/v1";
/// Tresor-Eintrag.
pub const VAULT: &[u8] = b"acta/vault/v1";
/// Tresor-Export der Sicherung.
pub const VAULT_BK: &[u8] = b"acta/vaultbk/v1";
/// Dokumentkopf-Schlüssel HK(v).
pub const HDR: &[u8] = b"acta/hdr/v1";
/// Datenschlüssel DEK(v).
pub const DEK: &[u8] = b"acta/dek/v1";
/// Clusterkennung.
pub const CLUSTER: &[u8] = b"acta/cluster/v1";
/// Verzeichnisschlüssel.
pub const DIR: &[u8] = b"acta/dir/v1";
/// Journal-Rahmen.
pub const WAL: &[u8] = b"acta/wal/v1";
/// Segment.
pub const SEG: &[u8] = b"acta/seg/v1";
/// Gate-Log.
pub const GATE: &[u8] = b"acta/gate/v1";
/// Deduplikationssalz.
pub const DEDUP: &[u8] = b"acta/dedup/v1";
/// Blob der Wissensebene.
pub const BLOB: &[u8] = b"acta/blob/v1";
/// Tree der Wissensebene.
pub const TREE: &[u8] = b"acta/tree/v1";
/// Commit der Wissensebene.
pub const COMMIT: &[u8] = b"acta/commit/v1";
/// Commitment für Löschnachweise.
pub const CMT: &[u8] = b"acta/cmt/v1";
/// Eintrag der Hash-Kette.
pub const LEDGER: &[u8] = b"acta/ledger/v1";
/// Checkpoint der Hash-Kette.
pub const CHECKPOINT: &[u8] = b"acta/checkpoint/v1";
/// Löschbericht.
pub const REPORT: &[u8] = b"acta/report/v1";
/// Datensatz des Umschlags v1.
pub const REC: &[u8] = b"acta/rec/v1";
/// Zugriffsprotokoll.
pub const LOG: &[u8] = b"acta/log/v1";
/// Löschvorgang (Arbeitsdaten).
pub const SCOPE: &[u8] = b"acta/scope/v1";
/// Generation des Handle-Registers.
pub const GEN: &[u8] = b"acta/gen/v1";

/// Alle Präfixe. Dient der Prüfung, dass keines doppelt vergeben ist.
pub const ALL: [&[u8]; 23] = [
    TAG, TAG_KEY, VAULT, VAULT_BK, HDR, DEK, CLUSTER, DIR, WAL, SEG, GATE, DEDUP, BLOB, TREE,
    COMMIT, CMT, LEDGER, CHECKPOINT, REPORT, REC, LOG, SCOPE, GEN,
];

#[cfg(test)]
mod tests {
    use super::ALL;

    #[test]
    fn prefixes_are_unique_and_versioned() {
        for (i, a) in ALL.iter().enumerate() {
            assert!(a.starts_with(b"acta/") && a.ends_with(b"/v1"), "bad form");
            for b in ALL.iter().skip(i + 1) {
                assert_ne!(a, b, "duplicate domain prefix");
            }
        }
    }
}
