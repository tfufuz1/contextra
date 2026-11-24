//! Deterministic seeded workload generator for simulation and crash-sweep tests (SP-VERIFY).

use contextra_storage_ports::traits::Entropy;
use contextra_storage_ports::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkloadOp {
    Put { cluster_idx: usize, key: Vec<u8>, value: Vec<u8> },
    Delete { cluster_idx: usize, key: Vec<u8> },
    Gate { cluster_idx: usize },
}

pub fn generate_workload(
    entropy: &dyn Entropy,
    num_clusters: usize,
    num_ops: usize,
) -> Result<Vec<WorkloadOp>, Error> {
    if num_clusters == 0 {
        return Ok(Vec::new());
    }
    let mut ops = Vec::with_capacity(num_ops);
    let mut buf = [0u8; 8];
    for step in 0..num_ops {
        entropy.fill(&mut buf)?;
        let raw = u64::from_le_bytes(buf);
        let cluster_idx = (raw as usize) % num_clusters;
        let kind = (raw >> 8) % 10;
        let key = format!("k_{:04}", (raw >> 16) % 32).into_bytes();
        if kind < 7 {
            let value = format!("val_{step}_{:08x}", raw >> 24).into_bytes();
            ops.push(WorkloadOp::Put {
                cluster_idx,
                key,
                value,
            });
        } else if kind < 9 {
            ops.push(WorkloadOp::Delete { cluster_idx, key });
        } else {
            ops.push(WorkloadOp::Gate { cluster_idx });
        }
    }
    Ok(ops)
}
