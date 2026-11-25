#![no_main]

use arbitrary::Arbitrary;
use contextra_store::memtable::MemTable;
use libfuzzer_sys::fuzz_target;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Arbitrary, Debug, Clone)]
enum MemTableOp {
    Put {
        key: Vec<u8>,
        value: Vec<u8>,
        seq_no: u64,
        tx_id: u64,
    },
    Get {
        key: Vec<u8>,
    },
    GetAtSeq {
        key: Vec<u8>,
        seq_no: u64,
        max_tx: u64,
    },
    Rollback {
        tx_id: u64,
    },
}

#[derive(Arbitrary, Debug)]
struct ConcurrentMemTableInput {
    thread_count: u8,
    ops: Vec<MemTableOp>,
}

fuzz_target!(|input: ConcurrentMemTableInput| {
    if input.ops.is_empty() {
        return;
    }

    let num_threads = match (input.thread_count % 3) + 2 {
        2 => 2,
        3 => 3,
        _ => 4,
    };

    let memtable = Arc::new(MemTable::new());
    let ops = Arc::new(input.ops);

    let (tx, rx) = mpsc::channel();

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_idx| {
            let mt = Arc::clone(&memtable);
            let ops = Arc::clone(&ops);
            let tx = tx.clone();

            thread::spawn(move || {
                for (idx, op) in ops.iter().enumerate() {
                    if idx % num_threads != thread_idx {
                        continue;
                    }

                    match op {
                        MemTableOp::Put {
                            key,
                            value,
                            seq_no,
                            tx_id,
                        } => {
                            mt.put(bytes::Bytes::copy_from_slice(key), bytes::Bytes::copy_from_slice(value), *seq_no, *tx_id);
                        }
                        MemTableOp::Get { key } => {
                            let _ = mt.get(key);
                        }
                        MemTableOp::GetAtSeq {
                            key,
                            seq_no,
                            max_tx,
                        } => {
                            let _ = mt.get_at_seq(key, *seq_no, *max_tx);
                        }
                        MemTableOp::Rollback { tx_id } => {
                            mt.rollback(*tx_id);
                        }
                    }
                }
                let _ = tx.send(thread_idx);
            })
        })
        .collect();

    drop(tx);

    let mut completed = 0;
    let timeout = Duration::from_secs(2);
    let start_time = std::time::Instant::now();

    while completed < num_threads {
        if rx.recv_timeout(Duration::from_millis(10)).is_ok() {
            completed += 1;
        }
        if start_time.elapsed() > timeout {
            panic!("Deadlock or execution timeout detected in fuzz_memtable_concurrent");
        }
    }

    for handle in handles {
        let _ = handle.join();
    }

    let _ = memtable.size();
    let _ = memtable.is_empty();
    let _ = memtable.iter();
    let _ = memtable.iter_latest();
});
