use anyhow::{Context, Result};
use contextra_ports::StorageEngine;
use contextra_store::lsm::{DurabilityMode, LsmConfig, LsmStorage};
use contextra_types::TxId;
use redb::{Database, Durability, TableDefinition};
use std::path::Path;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub const REDB_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("bench_store");

pub trait EngineHarness: Send + Sync {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<()>;
    fn put_batch(&self, items: &[(&[u8], &[u8])]) -> Result<()>;
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn range_scan(&self, start_key: &[u8], limit: usize) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
    fn flush(&self) -> Result<()>;
    fn close(self: Box<Self>) -> Result<()>;
}

pub struct ContextraHarness {
    storage: Arc<LsmStorage>,
    rt: Arc<Runtime>,
    next_tx_id: std::sync::atomic::AtomicU64,
}

impl ContextraHarness {
    pub async fn open(path: &Path, rt: Arc<Runtime>) -> Result<Self> {
        let config = LsmConfig {
            path: path.to_path_buf(),
            group_commit_window_micros: 0, // Fsync per commit (immediate WAL fsync)
            durability_mode: DurabilityMode::Full,
            ..Default::default()
        };
        let storage = Arc::new(
            LsmStorage::new(config)
                .await
                .context("failed to open Contextra storage")?,
        );

        let last_tx = storage
            .last_tx_id()
            .await
            .context("failed to get last committed tx")?;
        let last_tx_val = last_tx.0;

        Ok(Self {
            storage,
            rt,
            next_tx_id: std::sync::atomic::AtomicU64::new(last_tx_val + 1),
        })
    }

    pub fn open_sync(path: &Path, rt: Arc<Runtime>) -> Result<Self> {
        let rt_clone = Arc::clone(&rt);
        rt.block_on(Self::open(path, rt_clone))
    }
}

impl EngineHarness for ContextraHarness {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        let tx = TxId(self.next_tx_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst));
        self.rt.block_on(async {
            self.storage.put(tx, key, value).await?;
            self.storage.commit(tx).await?;
            Ok::<(), contextra_types::ContextraError>(())
        })
        .context("Contextra put failed")
    }

    fn put_batch(&self, items: &[(&[u8], &[u8])]) -> Result<()> {
        let tx = TxId(self.next_tx_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst));
        self.rt.block_on(async {
            for (k, v) in items {
                self.storage.put(tx, k, v).await?;
            }
            self.storage.commit(tx).await?;
            Ok::<(), contextra_types::ContextraError>(())
        })
        .context("Contextra put_batch failed")
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.rt.block_on(async {
            let res = self.storage.get(key).await?;
            Ok::<Option<Vec<u8>>, contextra_types::ContextraError>(res.map(|b| b.to_vec()))
        })
        .context("Contextra get failed")
    }

    fn range_scan(&self, start_key: &[u8], limit: usize) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        self.rt.block_on(async {
            let (res, _) = self
                .storage
                .scan_prefix_bounded(start_key, limit, None)
                .await?;
            Ok::<Vec<(Vec<u8>, Vec<u8>)>, contextra_types::ContextraError>(res)
        })
        .context("Contextra range_scan failed")
    }

    fn flush(&self) -> Result<()> {
        self.rt.block_on(async {
            self.storage.flush().await?;
            Ok::<(), contextra_types::ContextraError>(())
        })
        .context("Contextra flush failed")
    }

    fn close(self: Box<Self>) -> Result<()> {
        drop(self.storage);
        Ok(())
    }
}

pub struct RedbHarness {
    db: Database,
}

impl RedbHarness {
    pub fn open(path: &Path) -> Result<Self> {
        let db_path = path.join("redb_bench.redb");
        let db = Database::create(db_path).context("failed to create redb database")?;

        // Initialize table
        let write_txn = db.begin_write().context("redb init txn")?;
        {
            let _table = write_txn
                .open_table(REDB_TABLE)
                .context("redb init table")?;
        }
        write_txn.commit().context("redb init commit")?;

        Ok(Self { db })
    }
}

impl EngineHarness for RedbHarness {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        let mut write_txn = self.db.begin_write().context("redb begin_write")?;
        write_txn.set_durability(Durability::Immediate); // Fsync per commit
        {
            let mut table = write_txn.open_table(REDB_TABLE).context("redb open_table")?;
            table.insert(key, value).context("redb insert")?;
        }
        write_txn.commit().context("redb commit")?;
        Ok(())
    }

    fn put_batch(&self, items: &[(&[u8], &[u8])]) -> Result<()> {
        let mut write_txn = self.db.begin_write().context("redb begin_write")?;
        write_txn.set_durability(Durability::Immediate); // Fsync per commit
        {
            let mut table = write_txn.open_table(REDB_TABLE).context("redb open_table")?;
            for (k, v) in items {
                table.insert(*k, *v).context("redb insert batch")?;
            }
        }
        write_txn.commit().context("redb commit batch")?;
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let read_txn = self.db.begin_read().context("redb begin_read")?;
        let table = read_txn.open_table(REDB_TABLE).context("redb open_table read")?;
        let result = table.get(key).context("redb get")?;
        Ok(result.map(|access| access.value().to_vec()))
    }

    fn range_scan(&self, start_key: &[u8], limit: usize) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let read_txn = self.db.begin_read().context("redb begin_read scan")?;
        let table = read_txn.open_table(REDB_TABLE).context("redb open_table scan")?;
        let range = table.range(start_key..).context("redb range")?;

        let mut results = Vec::with_capacity(limit);
        for item in range.take(limit) {
            let (k, v) = item.context("redb scan item")?;
            results.push((k.value().to_vec(), v.value().to_vec()));
        }
        Ok(results)
    }

    fn flush(&self) -> Result<()> {
        // redb commits with Durability::Immediate flushes on every transaction commit
        Ok(())
    }

    fn close(self: Box<Self>) -> Result<()> {
        drop(self.db);
        Ok(())
    }
}
