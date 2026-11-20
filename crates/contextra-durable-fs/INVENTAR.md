
## contextra-durable-fs

Pfad: `crates/contextra-durable-fs`

Abhängigkeiten (direkt): thiserror, serde, tracing, crc32fast

### Module

- `/src/error.rs`
- `/src/fault_injector.rs`
- `/src/health.rs`
- `/src/journal.rs`
- `/src/lib.rs`
- `/src/platform.rs`
- `/src/scrub.rs`
- `/src/vfs.rs`

### Öffentliche Typen

- **AppendOnlyJournal** (struct) – src/journal.rs:71
- **AtomicHealthState** (struct) – src/health.rs:86
- **DirLock** (struct) – src/lib.rs:256
- **DurableFsError** (enum) – src/error.rs:11
- **DurableHandle** (struct) – src/lib.rs:129
- **Error** (type) – src/error.rs:45
- **FaultInjectingVfs** (struct) – src/fault_injector.rs:95
- **FaultInjector** (trait) – src/fault_injector.rs:10
- **FlushKind** (enum) – src/platform.rs:12
- **FsClass** (enum) – src/lib.rs:53
- **HealthEvent** (enum) – src/health.rs:73
- **HealthState** (enum) – src/health.rs:18
- **InjectedFault** (enum) – src/fault_injector.rs:82
- **JournalRecord** (struct) – src/journal.rs:62
- **JournalSyncPolicy** (enum) – src/journal.rs:12
- **LockMode** (enum) – src/lib.rs:246
- **NoFault** (struct) – src/fault_injector.rs:34
- **PendingDeleteJournal** (struct) – src/journal.rs:307
- **PoisonedHandle** (struct) – src/lib.rs:125
- **RealFs** (type) – src/vfs.rs:60
- **RecordRecoveryPolicy** (enum) – src/journal.rs:28
- **Result** (type) – src/error.rs:73
- **ScrubOutcome** (enum) – src/scrub.rs:54
- **ScrubPolicy** (enum) – src/scrub.rs:28
- **ScrubProgress** (struct) – src/scrub.rs:65
- **ScrubReceipt** (struct) – src/scrub.rs:21
- **ScrubRefusal** (enum) – src/scrub.rs:37
- **StdVfs** (struct) – src/vfs.rs:57
- **Vfs** (trait) – src/vfs.rs:12

### Öffentliche Funktionen

- `pub fn acquire(dir: &Path, mode: LockMode) -> Result<Self> `  – src/lib.rs:264
- `pub fn allows_writes(self) -> bool `  – src/health.rs:60
- `pub fn append(&mut self, payload: &[u8]) -> Result<u64> `  – src/journal.rs:136
- `pub fn atomic_replace(path: &Path, bytes: &[u8]) -> Result<()> `  – src/lib.rs:330
- `pub fn atomic_replace_with_injector<F: FaultInjector>(`  – src/lib.rs:335
- `pub fn calculate_crc32(data: &[u8]) -> u32 `  – src/journal.rs:45
- `pub fn durable_remove(path: &Path) -> Result<()> `  – src/lib.rs:382
- `pub fn durable_remove_with_injector<F: FaultInjector>(path: &Path, injector: &F) -> Result<()> `  – src/lib.rs:387
- `pub fn flush(&mut self) -> Result<()> `  – src/journal.rs:179
- `pub fn flush(&mut self, kind: FlushKind) -> Result<()> `  – src/lib.rs:171
- `pub fn flush_dir(dir: &Path) -> Result<()> `  – src/lib.rs:80
- `pub fn flush_file(file: &File, kind: FlushKind) -> Result<()> `  – src/lib.rs:75
- `pub fn fs_class(path: &Path) -> Result<FsClass> `  – src/lib.rs:98
- `pub fn get_mut(&mut self) -> Option<&mut File> `  – src/lib.rs:206
- `pub fn get_ref(&self) -> Option<&File> `  – src/lib.rs:197
- `pub fn health_state(&self) -> HealthState `  – src/journal.rs:300
- `pub fn health_state(&self) -> HealthState `  – src/journal.rs:346
- `pub fn inject_dir_fsync_failure(&self) `  – src/fault_injector.rs:135
- `pub fn inject_fsync_failure(&self) `  – src/fault_injector.rs:125
- `pub fn inject_rename_failure(&self) `  – src/fault_injector.rs:130
- `pub fn inject_write_disk_full(&self) `  – src/fault_injector.rs:120
- `pub fn is_poisoned(&self) -> bool `  – src/lib.rs:166
- `pub fn is_poisoned(self) -> bool `  – src/health.rs:66
- `pub fn kind(&self) -> std::io::ErrorKind `  – src/error.rs:50
- `pub fn load(&self) -> HealthState `  – src/health.rs:107
- `pub fn mode(&self) -> LockMode `  – src/lib.rs:303
- `pub fn new() -> Self `  – src/health.rs:99
- `pub fn new(file: File) -> Self `  – src/lib.rs:138
- `pub fn new(inner: V) -> Self `  – src/fault_injector.rs:107
- `pub fn new_with_injector(file: File, injector: F) -> Self `  – src/lib.rs:145
- `pub fn new_with_path_and_injector(file: File, path: PathBuf, injector: F) -> Self `  – src/lib.rs:155
- `pub fn open(journal_path: impl Into<PathBuf>) -> Result<Self> `  – src/journal.rs:315
- `pub fn open(path: impl Into<PathBuf>) -> Result<Self> `  – src/journal.rs:82
- `pub fn open_with_health(`  – src/journal.rs:320
- `pub fn open_with_policy(`  – src/journal.rs:116
- `pub fn open_without_recovery(`  – src/journal.rs:91
- `pub fn path(&self) -> &std::path::Path `  – src/journal.rs:131
- `pub fn pending(&self) -> &[PathBuf] `  – src/journal.rs:358
- `pub fn poison(&mut self) `  – src/lib.rs:215
- `pub fn process<F>(&mut self, mut remove_fn: F) -> Result<usize>`  – src/journal.rs:363
- `pub fn record(&mut self, path: &std::path::Path) -> Result<()> `  – src/journal.rs:351
- `pub fn record_event(&self, event: HealthEvent) -> HealthState `  – src/health.rs:117
- `pub fn recover(&mut self, recovery_policy: RecordRecoveryPolicy) -> Result<Vec<JournalRecord>> `  – src/journal.rs:194
- `pub fn rename_durable(from: &Path, to: &Path) -> Result<()> `  – src/lib.rs:113
- `pub fn scrub_and_remove(path: &Path) -> Result<ScrubReceipt> `  – src/lib.rs:399
- `pub fn scrub_and_remove_with_injector<F: FaultInjector>(`  – src/lib.rs:404
- `pub fn scrub_file_with_rate_limit<F>(`  – src/scrub.rs:75
- `pub fn scrub_if_safe(`  – src/scrub.rs:133
- `pub fn scrub_if_safe_with_injector<F: FaultInjector>(`  – src/scrub.rs:143
- `pub fn simulate_crash(&self) -> Result<()> `  – src/fault_injector.rs:141
- `pub fn sync_dir(dir: &Path) -> Result<()> `  – src/lib.rs:319
- `pub fn sync_dir_path(dir: &Path) -> io::Result<()> `  – src/platform.rs:37
- `pub fn sync_dir_with_injector<F: FaultInjector>(dir: &Path, injector: &F) -> Result<()> `  – src/lib.rs:324
- `pub fn sync_file(file: &File, kind: FlushKind) -> io::Result<()> `  – src/platform.rs:23
- `pub fn transition(self, event: HealthEvent) -> Self `  – src/health.rs:39
- `pub(crate) fn generate_tmp_path(target: &Path) -> Result<PathBuf> `  – src/vfs.rs:132

---
