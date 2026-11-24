//! Maintenance task scheduler and priority lanes (SPEC §11, §16, SP-MAINT).
#![forbid(unsafe_code)]

use contextra_storage_ports::Stamp;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaintPriority {
    Idle = 0,
    BlobGc = 1,
    Compact = 2,
    Flush = 3,
    Urgent = 4,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaintTask {
    pub priority: MaintPriority,
    pub id: u64,
    pub task_name: &'static str,
}

impl Ord for MaintTask {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.id.cmp(&self.id))
    }
}

impl PartialOrd for MaintTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct MaintScheduler {
    queue: BinaryHeap<MaintTask>,
    next_id: u64,
}

impl MaintScheduler {
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
            next_id: 1,
        }
    }

    pub fn schedule(&mut self, priority: MaintPriority, task_name: &'static str) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        self.queue.push(MaintTask {
            priority,
            id,
            task_name,
        });
        id
    }

    pub fn pop_next(&mut self) -> Option<MaintTask> {
        self.queue.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Computes gc_floor as minimum across active view stamps, hold stamps, and consumer watermarks.
    pub fn compute_gc_floor(
        view_stamps: &[Stamp],
        hold_stamps: &[Stamp],
        consumer_watermarks: &[Stamp],
        durable_stamp: Stamp,
    ) -> Stamp {
        let mut min_stamp = durable_stamp;
        for s in view_stamps {
            if *s < min_stamp {
                min_stamp = *s;
            }
        }
        for s in hold_stamps {
            if *s < min_stamp {
                min_stamp = *s;
            }
        }
        for s in consumer_watermarks {
            if *s < min_stamp {
                min_stamp = *s;
            }
        }
        min_stamp
    }
}
