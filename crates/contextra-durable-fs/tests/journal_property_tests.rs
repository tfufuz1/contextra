use contextra_durable_fs::journal::{
    AppendOnlyJournal, JournalSyncPolicy, RecordRecoveryPolicy, RECORD_HEADER_SIZE,
};
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use tempfile::tempdir;

#[test]
fn test_journal_append_and_recovery() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("journal.log");

    {
        let mut journal = AppendOnlyJournal::open(&path).expect("open");
        assert_eq!(journal.append(b"record-1").expect("append"), 1);
        assert_eq!(journal.append(b"record-2").expect("append"), 2);
    }

    {
        let mut journal = AppendOnlyJournal::open_without_recovery(
            &path,
            JournalSyncPolicy::Always,
            std::sync::Arc::new(contextra_durable_fs::AtomicHealthState::new()),
        )
        .expect("reopen");
        let records = journal
            .recover(RecordRecoveryPolicy::FailClosed)
            .expect("recover");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].seq_no, 1);
        assert_eq!(records[0].payload, b"record-1");
        assert_eq!(records[1].seq_no, 2);
        assert_eq!(records[1].payload, b"record-2");

        assert_eq!(journal.append(b"record-3").expect("append"), 3);
    }
}

#[test]
fn test_journal_torn_write_and_recovery_policies() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("journal_torn.log");

    {
        let mut journal = AppendOnlyJournal::open(&path).expect("open");
        journal.append(b"valid-record-1").expect("append");
        journal.append(b"valid-record-2").expect("append");
    }

    // Torn-Write am Ende der Datei simulieren (halber Header angehängt)
    {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&path)
            .expect("open raw");
        file.write_all(b"CTXJ\x03\x00").expect("write torn");
    }

    // 1. FailClosed schlägt fehl
    {
        let mut journal = AppendOnlyJournal::open_without_recovery(
            &path,
            JournalSyncPolicy::Manual,
            std::sync::Arc::new(contextra_durable_fs::AtomicHealthState::new()),
        )
        .expect("open");
        assert!(journal.recover(RecordRecoveryPolicy::FailClosed).is_err());
    }

    // 2. TruncateCorruptTail schneidet ab und stellt gültige Records wieder her
    {
        let mut journal = AppendOnlyJournal::open_without_recovery(
            &path,
            JournalSyncPolicy::Manual,
            std::sync::Arc::new(contextra_durable_fs::AtomicHealthState::new()),
        )
        .expect("open");
        let records = journal
            .recover(RecordRecoveryPolicy::TruncateCorruptTail)
            .expect("recover");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].payload, b"valid-record-1");
        assert_eq!(records[1].payload, b"valid-record-2");
    }
}

#[test]
fn test_journal_bit_rot_detection() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("journal_bitrot.log");

    {
        let mut journal = AppendOnlyJournal::open(&path).expect("open");
        journal.append(b"important-data").expect("append");
    }

    // 1 Byte im Payload kippen (Bit-Rot)
    {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .expect("open raw");
        file.seek(SeekFrom::Start((RECORD_HEADER_SIZE + 2) as u64))
            .expect("seek");
        file.write_all(&[0xFF]).expect("write bitrot");
    }

    {
        let mut journal = AppendOnlyJournal::open_without_recovery(
            &path,
            JournalSyncPolicy::Manual,
            std::sync::Arc::new(contextra_durable_fs::AtomicHealthState::new()),
        )
        .expect("open");
        assert!(journal.recover(RecordRecoveryPolicy::FailClosed).is_err());
    }
}
