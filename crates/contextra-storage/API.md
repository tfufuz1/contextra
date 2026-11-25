# API: contextra-storage

> Erzeugt aus dem Quellcode am 2026-11-25, Repository HEAD ce4579a56. Eine Zeile je Item: Datei:Zeile und die **erste Zeile** der Signatur. Mehrzeilige Signaturen sind nur angeschnitten. Nicht kompiliert, daher ohne Typprüfung. Testcode (`tests`, `testing`, `test_doubles`) ist ausgeschlossen.

Umfang: 877 öffentliche Items, 3 crate-interne Sichtbarkeitsgrenzen.

## Innen: Abhängigkeiten zu anderen Workspace-Crates

- keine

## Außen: öffentliche Schnittstelle (`pub`)

```rust
src/catalog/edit.rs:7  pub struct PurgeBatchEdit { ticket: [u8; 16], remove_file_id: u64, replacement: Option<FileEntry> }
src/catalog/edit.rs:10  pub fn replace(ticket: [u8; 16], old_file_id: u64, replacement: FileEntry) -> Result<Self, Error>
src/catalog/edit.rs:14  pub fn remove(ticket: [u8; 16], file_id: u64) -> Self
src/catalog/edit.rs:17  pub fn ticket(&self) -> &[u8; 16] { &self.ticket }
src/catalog/edit.rs:18  pub fn into_op(self) -> CatalogOp
src/catalog/edit.rs:24  pub struct UnlinkAckEdit { ticket: [u8; 16], file_id: u64 }
src/catalog/edit.rs:28  pub fn after_durable_unlink(ticket: [u8; 16], file_id: u64) -> Self { Self { ticket, file_id } }
src/catalog/edit.rs:29  pub fn into_op(self) -> CatalogOp { CatalogOp::UnlinkAck { ticket: self.ticket, file_id: self.file_id } }
src/catalog/edit.rs:33  pub struct PurgeCompleteEdit { ticket: [u8; 16] }
src/catalog/edit.rs:38  pub fn after_owner_attestations(ticket: [u8; 16]) -> Self { Self { ticket } }
src/catalog/edit.rs:39  pub fn into_op(self) -> CatalogOp { CatalogOp::PurgeComplete { ticket: self.ticket } }
src/catalog/edit.rs:43  pub enum CatalogEdit
src/catalog/edit.rs:50  pub fn into_op(self) -> CatalogOp
src/catalog/file.rs:14  pub enum OpenMode { ReadOnly, ReadWrite }
src/catalog/file.rs:16  pub struct CatalogFile
src/catalog/file.rs:30  pub fn open_current(
src/catalog/file.rs:50  pub fn open(
src/catalog/file.rs:126  pub fn append_durable(
src/catalog/file.rs:139  pub fn append_group_durable(
src/catalog/file.rs:210  pub fn append_edits_durable(
src/catalog/file.rs:224  pub fn reserve_space_ids_durable(
src/catalog/file.rs:238  pub fn path(&self) -> &Path { &self.path }
src/catalog/file.rs:239  pub fn state(&self) -> &CatalogState { &self.state }
src/catalog/file.rs:240  pub fn next_sequence(&self) -> u64 { self.next_sequence }
src/catalog/file.rs:241  pub fn append_offset(&self) -> u64 { self.append_offset }
src/catalog/file.rs:242  pub fn is_writable(&self) -> bool { self.writable }
src/catalog/file.rs:243  pub fn has_torn_tail(&self) -> bool { self.torn_tail }
src/catalog/mod.rs:2  pub mod edit
src/catalog/mod.rs:3  pub mod file
src/catalog/mod.rs:4  pub mod ops
src/catalog/mod.rs:5  pub mod record
src/catalog/mod.rs:6  pub mod reserve
src/catalog/mod.rs:7  pub mod rollover
src/catalog/mod.rs:8  pub mod state
src/catalog/mod.rs:10  pub use ops::{CatalogOp, FileKind}
src/catalog/mod.rs:11  pub use record::{CatalogIntegrity, CatalogRecord}
src/catalog/mod.rs:12  pub use state::{CatalogState, FileMeta}
src/catalog/ops.rs:20  pub enum FileKind { Run, Blob }
src/catalog/ops.rs:23  pub struct FileEntry
src/catalog/ops.rs:35  pub enum CatalogOp
src/catalog/ops.rs:54  pub fn encode(&self, out: &mut Vec<u8>) -> Result<(), Error>
src/catalog/ops.rs:118  pub fn decode(input: &[u8]) -> Result<(Self, usize), Error>
src/catalog/record.rs:12  pub enum CatalogIntegrity { Plain, Authenticated }
src/catalog/record.rs:15  pub struct CatalogRecord
src/catalog/record.rs:23  pub fn encode_record(
src/catalog/record.rs:61  pub fn decode_record(
src/catalog/reserve.rs:6  pub const STAMP_RESERVATION: Stamp = 1 << 20
src/catalog/reserve.rs:7  pub const ID_RESERVATION: u64 = 1 << 16
src/catalog/reserve.rs:8  pub const SPACE_ID_RESERVATION: u64 = 1 << 16
src/catalog/reserve.rs:11  pub struct Reservation { pub first: u64, pub through: u64 }
src/catalog/reserve.rs:14  pub fn contains(&self, value: u64) -> bool { value >= self.first && value <= self.through }
src/catalog/reserve.rs:19  pub fn next_stamp_reservation(current_limit: Stamp, amount: Stamp) -> Result<(CatalogOp, Reservation), Error>
src/catalog/reserve.rs:27  pub fn next_id_reservation(current_limit: u64, amount: u64) -> Result<(CatalogOp, Reservation), Error>
src/catalog/reserve.rs:35  pub fn next_space_id_reservation(current_limit: u64, amount: u64) -> Result<(CatalogOp, Reservation), Error>
src/catalog/reserve.rs:43  pub fn after_reopen(stamp_limit: Stamp, id_limit: u64, space_id_limit: u64) -> Result<(Stamp, u64, u64), Error>
src/catalog/rollover.rs:12  pub struct CurrentPointer { pub generation: u64 }
src/catalog/rollover.rs:15  pub fn encode(self) -> Result<[u8; CURRENT_LEN], Error>
src/catalog/rollover.rs:28  pub fn decode(bytes: &[u8]) -> Result<Self, Error>
src/catalog/rollover.rs:42  pub fn publish_current(fs: &dyn Fs, root: &Path, pointer: CurrentPointer) -> Result<(), Error>
src/catalog/rollover.rs:65  pub fn read_current(fs: &dyn Fs, root: &Path) -> Result<CurrentPointer, Error>
src/catalog/state.rs:8  pub type FileMeta = FileEntry
src/catalog/state.rs:11  pub struct CatalogCluster
src/catalog/state.rs:17  pub struct GraveyardEntry
src/catalog/state.rs:23  pub struct CatalogState
src/catalog/state.rs:49  pub fn fold(records: &[CatalogRecord]) -> Result<Self, Error>
src/catalog/state.rs:63  pub fn apply(&mut self, op: &CatalogOp) -> Result<(), Error>
src/catalog/state.rs:177  pub fn spaces(&self) -> &BTreeMap<SpaceId, Protection> { &self.spaces }
src/catalog/state.rs:178  pub fn clusters(&self) -> &BTreeMap<ClusterId, CatalogCluster> { &self.clusters }
src/catalog/state.rs:179  pub fn files(&self) -> &BTreeMap<u64, FileMeta> { &self.files }
src/catalog/state.rs:180  pub fn graveyard(&self) -> &BTreeMap<([u8; 16], u64), GraveyardEntry> { &self.graveyard }
src/catalog/state.rs:181  pub fn purge_is_complete(&self, ticket: &[u8; 16]) -> bool { self.purge_complete.contains(ticket) }
src/catalog/state.rs:182  pub fn replay_floor(&self) -> (u64, u64) { self.replay_floor }
src/catalog/state.rs:183  pub fn redo_mark(&self) -> (u64, u64, Stamp) { self.redo_mark }
src/catalog/state.rs:184  pub fn stamp_limit(&self) -> Stamp { self.stamp_limit }
src/catalog/state.rs:185  pub fn id_limit(&self) -> u64 { self.id_limit }
src/catalog/state.rs:186  pub fn space_id_limit(&self) -> u64 { self.space_id_limit }
src/catalog/state.rs:187  pub fn epoch(&self) -> u64 { self.epoch }
src/checkpoint/create.rs:15  pub const CHECKPOINT_MANIFEST_MAGIC: &[u8; 8] = b"CTXSCKP1"
src/checkpoint/create.rs:18  pub struct CheckpointFileEntry
src/checkpoint/create.rs:25  pub struct CheckpointManifest
src/checkpoint/create.rs:37  pub fn encode(&self) -> Vec<u8>
src/checkpoint/create.rs:60  pub fn decode(bytes: &[u8]) -> Result<Self, Error>
src/checkpoint/create.rs:174  pub struct CheckpointCreator
src/checkpoint/create.rs:178  pub fn write_manifest(
src/checkpoint/create.rs:208  pub fn read_manifest(fs: &dyn Fs, target_dir: &Path) -> Result<CheckpointManifest, Error>
src/checkpoint/mod.rs:2  pub mod create
src/checkpoint/mod.rs:3  pub mod restore
src/checkpoint/restore.rs:10  pub struct RestoreOptions
src/checkpoint/restore.rs:19  pub struct RestoreReport
src/checkpoint/restore.rs:26  pub struct CheckpointRestorer
src/checkpoint/restore.rs:30  pub fn validate_for_restore(
src/checkpoint/restore.rs:58  pub fn reconcile_tombstones(
src/codec/checksums.rs:5  pub fn crc32c(bytes: &[u8]) -> u32
src/codec/checksums.rs:17  pub fn append_crc32c(out: &mut Vec<u8>, bytes: &[u8])
src/codec/checksums.rs:21  pub fn verify_crc32c(bytes: &[u8], expected_le: [u8; 4]) -> bool
src/codec/comparator.rs:8  pub struct EntryKey<'a>
src/codec/comparator.rs:22  pub fn cmp_entry(left: &EntryKey<'_>, right: &EntryKey<'_>) -> Ordering
src/codec/format.rs:15  pub enum IntegrityMode { Plain = 1, Authenticated = 2, Encrypted = 3 }
src/codec/format.rs:25  pub struct FormatRecord
src/codec/format.rs:31  pub fn encode(record: FormatRecord, root_key: &Key32, kdf: &dyn Kdf, mac: &dyn Mac) -> Result<Vec<u8>, Error>
src/codec/format.rs:46  pub fn decode(input: &[u8], root_key: &Key32, kdf: &dyn Kdf, mac: &dyn Mac, supported_features: u32) -> Result<FormatRecord, Error>
src/codec/header.rs:5  pub const HEADER_LEN: usize = 43
src/codec/header.rs:6  pub const HEADER_VERSION: u16 = 1
src/codec/header.rs:9  pub struct FileHeader
src/codec/header.rs:19  pub fn encode(&self) -> Result<[u8; HEADER_LEN], Error>
src/codec/header.rs:34  pub fn decode(input: &[u8], expected_magic: [u8; 8], expected_kind: u8, supported_features: u32) -> Result<Self, Error>
src/codec/keys_codec.rs:14  pub struct KeysRecord
src/codec/keys_codec.rs:19  pub fn encode(record: &KeysRecord, root_key: &Key32, mac: &dyn Mac) -> Result<Vec<u8>, Error>
src/codec/keys_codec.rs:36  pub fn decode(input: &[u8], root_key: &Key32, mac: &dyn Mac) -> Result<KeysRecord, Error>
src/codec/mod.rs:2  pub mod checksums
src/codec/mod.rs:3  pub mod comparator
src/codec/mod.rs:4  pub mod format
src/codec/mod.rs:5  pub mod header
src/codec/mod.rs:6  pub mod keys_codec
src/codec/mod.rs:7  pub mod seal
src/codec/mod.rs:8  pub mod tags
src/codec/mod.rs:9  pub mod varint
src/codec/seal.rs:14  pub enum SealMode { Plain, Authenticated, Encrypted }
src/codec/seal.rs:17  pub enum Aad<'a>
src/codec/seal.rs:28  pub fn encode(self) -> Vec<u8>
src/codec/seal.rs:49  pub fn seal_frame(
src/codec/seal.rs:90  pub fn unseal_frame(
src/codec/tags.rs:7  pub enum Kind
src/codec/tags.rs:16  pub enum TagError
src/codec/tags.rs:25  pub fn tag(stamp: Stamp, kind: Kind) -> Result<u64, TagError>
src/codec/tags.rs:36  pub fn untag(value: u64) -> Result<(Stamp, Kind), TagError>
src/codec/varint.rs:7  pub enum VarintError
src/codec/varint.rs:26  pub fn varint_put(out: &mut Vec<u8>, mut value: u64)
src/codec/varint.rs:35  pub fn varint_get(input: &[u8]) -> Result<(u64, usize), VarintError>
src/config.rs:9  pub const DEFAULT_VIEW_LEASE_NANOS: u64 = 60 * 1_000_000_000
src/config.rs:10  pub const DEFAULT_VIEW_LEASE_MAX_NANOS: u64 = 600 * 1_000_000_000
src/config.rs:13  pub enum OpenProfile
src/config.rs:20  pub struct Config
src/config.rs:101  pub fn with_filesystem(mut self, root: PathBuf, fs: Arc<dyn Fs>) -> Self
src/config.rs:107  pub fn with_anchor(mut self, anchor: Arc<dyn Anchor>) -> Self
src/config.rs:112  pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self
src/config.rs:117  pub fn with_entropy(mut self, entropy: Arc<dyn Entropy>) -> Self
src/config.rs:122  pub fn with_key_provider(mut self, provider: Arc<dyn KeyProvider>) -> Self
src/config.rs:127  pub fn with_catalog_auth(mut self, key: Arc<Key32>, mac: Arc<dyn Mac>) -> Self
src/config.rs:134  pub fn validate(&self) -> Result<(), Error>
src/engine/branches.rs:12  pub const MAIN_BRANCH: BranchId = BranchId(0)
src/engine/branches.rs:13  pub const DEFAULT_MAX_BRANCHES: usize = 1024
src/engine/branches.rs:14  pub const DEFAULT_MAX_DEPTH: u8 = 8
src/engine/branches.rs:17  pub enum ForkPoint
src/engine/branches.rs:23  pub struct BranchOptions
src/engine/branches.rs:34  pub enum BranchState
src/engine/branches.rs:41  pub struct BranchMeta
src/engine/branches.rs:53  pub fn id(&self) -> BranchId { self.id }
src/engine/branches.rs:54  pub fn parent(&self) -> Option<BranchId> { self.parent }
src/engine/branches.rs:55  pub fn fork_stamp(&self) -> Stamp { self.fork_stamp }
src/engine/branches.rs:56  pub fn head_stamp(&self) -> Stamp { self.head_stamp }
src/engine/branches.rs:57  pub fn depth(&self) -> u8 { self.depth }
src/engine/branches.rs:58  pub fn state(&self) -> BranchState { self.state }
src/engine/branches.rs:59  pub fn name(&self) -> &[u8] { self.name.as_slice() }
src/engine/branches.rs:60  pub fn ttl(&self) -> Option<Duration> { self.ttl }
src/engine/branches.rs:61  pub fn is_live(&self) -> bool { self.state == BranchState::Live }
src/engine/branches.rs:67  pub struct BranchTable
src/engine/branches.rs:76  pub fn new(history_floor: Stamp, main_head: Stamp, max_branches: usize, max_depth: u8) -> Result<Self, Error>
src/engine/branches.rs:97  pub fn history_floor(&self) -> Stamp { self.history_floor }
src/engine/branches.rs:98  pub fn len(&self) -> usize { self.branches.values().filter(|branch| !matches!(branch.state, BranchState::Dropped { .. })).count() }
src/engine/branches.rs:99  pub fn is_empty(&self) -> bool { self.branches.is_empty() }
src/engine/branches.rs:100  pub fn get(&self, id: BranchId) -> Option<&BranchMeta> { self.branches.get(&id) }
src/engine/branches.rs:101  pub fn iter(&self) -> impl Iterator<Item = &BranchMeta> { self.branches.values() }
src/engine/branches.rs:107  pub fn fork(&mut self, id: BranchId, parent: BranchId, point: ForkPoint, options: BranchOptions, parent_head: Stamp, commit_stamp: Stamp) -> Result<Stamp, Error>
src/engine/branches.rs:135  pub fn advance_head(&mut self, id: BranchId, expected: Stamp, next: Stamp) -> Result<(), Error>
src/engine/branches.rs:146  pub fn mark_merged(&mut self, source: BranchId, target: BranchId, expected_source_head: Stamp, expected_target_head: Stamp, at: Stamp) -> Result<(), Error>
src/engine/branches.rs:164  pub fn drop_branch(&mut self, id: BranchId, at: Stamp) -> Result<(), Error>
src/engine/branches.rs:174  pub fn set_history_floor(&mut self, floor: Stamp) -> Result<(), Error>
src/engine/branches.rs:184  pub fn visibility_chain(&self, branch: BranchId, view_stamp: Stamp) -> Result<Vec<(BranchId, Stamp)>, Error>
src/engine/cache.rs:7  pub const DEFAULT_CACHE_MAX_BYTES: usize = 16 * 1024 * 1024
src/engine/cache.rs:23  pub struct ReadCache
src/engine/cache.rs:36  pub fn new(max_bytes: usize) -> Self
src/engine/cache.rs:48  pub fn len(&self) -> usize
src/engine/cache.rs:52  pub fn is_empty(&self) -> bool
src/engine/cache.rs:56  pub fn current_bytes(&self) -> usize
src/engine/cache.rs:61  pub fn get(
src/engine/cache.rs:82  pub fn put(
src/engine/cache.rs:125  pub fn invalidate_cluster(&mut self, cluster: ClusterId)
src/engine/cache.rs:137  pub fn invalidate_branch(&mut self, cluster: ClusterId, branch: BranchId)
src/engine/cache.rs:149  pub fn clear(&mut self)
src/engine/committer.rs:11  pub struct CommitMutation
src/engine/committer.rs:21  pub trait CommitMutationDecoder: Send + Sync
src/engine/committer.rs:28  pub trait CommitPublisher: Send + Sync
src/engine/committer.rs:35  pub struct MemtableCommitPublisher
src/engine/committer.rs:42  pub fn new(memtable: Arc<Memtable>, versions: Arc<VersionRegistry>, decoder: Arc<dyn CommitMutationDecoder>) -> Self
src/engine/committer.rs:93  pub struct Committer<'a>
src/engine/committer.rs:101  pub fn new(writer: RedoWriter<'a>, publisher: Box<dyn CommitPublisher>) -> Self
src/engine/committer.rs:114  pub fn commit(&mut self, frames: &[EncodedFrame], durability: Durability) -> Result<GroupAck, Error>
src/engine/committer.rs:154  pub fn last_published_stamp(&self) -> Stamp { self.last_published }
src/engine/committer.rs:156  pub fn is_poisoned(&self) -> bool { self.poisoned }
src/engine/cursor.rs:8  pub trait BlockPruner: Send + Sync
src/engine/cursor.rs:13  pub struct ResumeToken
src/engine/cursor.rs:21  pub fn encode(&self) -> Vec<u8>
src/engine/cursor.rs:32  pub fn decode(bytes: &[u8]) -> Result<Self, Error>
src/engine/cursor.rs:81  pub struct Cursor
src/engine/cursor.rs:91  pub fn new(
src/engine/cursor.rs:107  pub fn from_resume_token(token: ResumeToken, lease_deadline_mono: u64) -> Self
src/engine/cursor.rs:122  pub fn cluster(&self) -> ClusterId
src/engine/cursor.rs:126  pub fn branch(&self) -> BranchId
src/engine/cursor.rs:130  pub fn view_stamp(&self) -> Stamp
src/engine/cursor.rs:134  pub fn is_exhausted(&self) -> bool
src/engine/cursor.rs:138  pub fn last_key(&self) -> Option<&[u8]>
src/engine/cursor.rs:143  pub fn renew_lease(&mut self, next_deadline_mono: u64)
src/engine/cursor.rs:150  pub fn check_validity(&self, now_mono: u64, gate_index: &GateIndex) -> Result<(), Error>
src/engine/cursor.rs:159  pub fn step(&mut self, key: Vec<u8>)
src/engine/cursor.rs:163  pub fn mark_exhausted(&mut self)
src/engine/cursor.rs:168  pub fn may_prune_block(
src/engine/cursor.rs:182  pub fn resume_token(&self) -> Option<ResumeToken>
src/engine/extension.rs:7  pub const EXTENSION_BLOCK_MAGIC: &[u8; 8] = b"CTXSEXT1"
src/engine/extension.rs:11  pub enum PruneContract
src/engine/extension.rs:17  pub struct ExtensionBlock
src/engine/extension.rs:24  pub fn encode(&self) -> Vec<u8>
src/engine/extension.rs:34  pub fn decode(bytes: &[u8]) -> Result<Self, Error>
src/engine/extension.rs:81  pub fn filter_visible(
src/engine/feed.rs:8  pub enum FeedPayload
src/engine/feed.rs:15  pub struct FeedItem
src/engine/feed.rs:22  pub struct ConsumerFeed
src/engine/feed.rs:28  pub fn new(instance: InstanceId, initial_watermark: Stamp) -> Self
src/engine/feed.rs:35  pub fn instance(&self) -> InstanceId
src/engine/feed.rs:39  pub fn watermark(&self) -> Stamp
src/engine/feed.rs:44  pub fn advance_watermark(&mut self, next_stamp: Stamp, durable_stamp: Stamp) -> Result<(), Error>
src/engine/feed.rs:60  pub fn project_item(
src/engine/graveyard.rs:5  pub struct Graveyard<T> { retired: Vec<Arc<T>> }
src/engine/graveyard.rs:12  pub fn new() -> Self { Self::default() }
src/engine/graveyard.rs:14  pub fn retire(&mut self, object: Arc<T>)
src/engine/graveyard.rs:18  pub fn pending(&self) -> usize { self.retired.len() }
src/engine/graveyard.rs:22  pub fn reclaim(&mut self) -> Vec<T>
src/engine/guard_window.rs:11  pub const MAX_GUARD_WINDOW_ENTRIES: usize = 1_000_000
src/engine/guard_window.rs:14  pub struct GuardScope
src/engine/guard_window.rs:21  pub struct GuardMutation
src/engine/guard_window.rs:31  pub struct GuardWindow
src/engine/guard_window.rs:38  pub fn new(capacity: usize) -> Result<Self, Error>
src/engine/guard_window.rs:47  pub fn record(&mut self, mutation: GuardMutation) -> Result<(), Error>
src/engine/guard_window.rs:61  pub fn check_point(&self, snapshot: Stamp, fingerprint: u64) -> Result<(), Error>
src/engine/guard_window.rs:71  pub fn check_range(&self, snapshot: Stamp, scope: GuardScope) -> Result<(), Error>
src/engine/guard_window.rs:81  pub fn floor(&self) -> Stamp
src/engine/guard_window.rs:85  pub fn len(&self) -> usize { self.entries.len() }
src/engine/guard_window.rs:86  pub fn is_empty(&self) -> bool { self.entries.is_empty() }
src/engine/idem.rs:7  pub const DEFAULT_MAX_IDEM_ENTRIES: usize = 65_536
src/engine/idem.rs:10  pub enum IdemState
src/engine/idem.rs:16  pub struct IdemEntry
src/engine/idem.rs:23  pub enum IdemAction
src/engine/idem.rs:28  pub struct IdemRegistry
src/engine/idem.rs:40  pub fn new(max_entries: usize) -> Self
src/engine/idem.rs:51  pub fn len(&self) -> usize
src/engine/idem.rs:55  pub fn is_empty(&self) -> bool
src/engine/idem.rs:65  pub fn check_or_reserve(
src/engine/idem.rs:116  pub fn commit(&mut self, token: IdemToken, stamp: Stamp, durable: bool) -> Result<(), Error>
src/engine/idem.rs:137  pub fn abort(&mut self, token: IdemToken)
src/engine/idem.rs:146  pub fn trim(&mut self, floor_stamp: Stamp, now_mono: u64)
src/engine/memtable.rs:12  pub struct MemKey
src/engine/memtable.rs:40  pub fn new(branch: BranchId, user_key: Vec<u8>, tag: u64) -> Self
src/engine/memtable.rs:45  pub struct MemValue
src/engine/memtable.rs:59  pub struct Memtable
src/engine/memtable.rs:68  pub fn new() -> Self { Self { partitions: RwLock::new(HashMap::new()) } }
src/engine/memtable.rs:70  pub fn insert(&self, cluster: ClusterId, key: MemKey, kind: u8, payload: &[u8]) -> Result<(), Error>
src/engine/memtable.rs:83  pub fn get(&self, cluster: ClusterId, key: &MemKey) -> Result<Option<Arc<MemValue>>, Error>
src/engine/memtable.rs:91  pub fn detach_cluster(&self, cluster: ClusterId) -> Result<Option<DetachedPartition>, Error>
src/engine/memtable.rs:98  pub fn approx_bytes(&self) -> Result<u64, Error>
src/engine/memtable.rs:103  pub fn cluster_count(&self) -> Result<usize, Error>
src/engine/memtable.rs:108  pub struct DetachedPartition { partition: Arc<ClusterPartition> }
src/engine/memtable.rs:111  pub fn approx_bytes(&self) -> u64 { self.partition.approx_bytes.load(Ordering::Acquire) }
src/engine/memtable.rs:112  pub fn is_empty(&self) -> bool { self.partition.entries.is_empty() }
src/engine/memtable.rs:113  pub fn visit(&self, mut visitor: impl FnMut(&MemKey, &Arc<MemValue>))
src/engine/mod.rs:2  pub mod branches
src/engine/mod.rs:3  pub mod cache
src/engine/mod.rs:4  pub mod committer
src/engine/mod.rs:5  pub mod cursor
src/engine/mod.rs:6  pub mod extension
src/engine/mod.rs:7  pub mod feed
src/engine/mod.rs:8  pub mod graveyard
src/engine/mod.rs:9  pub mod guard_window
src/engine/mod.rs:10  pub mod idem
src/engine/mod.rs:11  pub mod memtable
src/engine/mod.rs:12  pub mod packgroups
src/engine/mod.rs:13  pub mod page
src/engine/mod.rs:14  pub mod registry
src/engine/mod.rs:15  pub mod segments
src/engine/mod.rs:16  pub mod version
src/engine/mod.rs:17  pub mod views
src/engine/mod.rs:18  pub mod volatile
src/engine/packgroups.rs:7  pub const MAX_PACKGROUP_GATE_BATCH: usize = 4096
src/engine/packgroups.rs:10  pub struct PackKey(pub [u8; 16])
src/engine/packgroups.rs:13  pub enum PackPolicy
src/engine/packgroups.rs:20  pub struct PackGroup
src/engine/packgroups.rs:28  pub fn new(key: PackKey, policy: PackPolicy, protection: Protection) -> Self
src/engine/packgroups.rs:37  pub fn add_member(&mut self, cluster: ClusterId) -> Result<(), Error>
src/engine/packgroups.rs:42  pub fn remove_member(&mut self, cluster: &ClusterId) -> bool
src/engine/packgroups.rs:46  pub fn len(&self) -> usize
src/engine/packgroups.rs:50  pub fn is_empty(&self) -> bool
src/engine/packgroups.rs:57  pub fn split_pack_expired(
src/engine/page.rs:7  pub struct PageBudget
src/engine/page.rs:24  pub struct PageEntry
src/engine/page.rs:32  pub struct PageCollector
src/engine/page.rs:40  pub fn new(budget: PageBudget) -> Self
src/engine/page.rs:49  pub fn is_empty(&self) -> bool
src/engine/page.rs:53  pub fn len(&self) -> usize
src/engine/page.rs:57  pub fn total_weight(&self) -> u64
src/engine/page.rs:61  pub fn total_bytes(&self) -> usize
src/engine/page.rs:66  pub fn can_accept(&self, entry: &PageEntry) -> bool
src/engine/page.rs:87  pub fn try_push(&mut self, entry: PageEntry) -> Result<bool, Error>
src/engine/page.rs:102  pub fn finish(self, next_key: Option<Vec<u8>>) -> (Vec<PageEntry>, Option<Vec<u8>>)
src/engine/registry.rs:14  pub struct VersionRegistry { roots: RwLock<Roots> }
src/engine/registry.rs:21  pub fn new() -> Self
src/engine/registry.rs:28  pub fn snapshot(&self) -> Result<Arc<Version>, Error>
src/engine/registry.rs:36  pub fn publish(&self, version: Arc<Version>) -> Result<(), Error>
src/engine/registry.rs:40  pub fn publish_group(&self, versions: &[Arc<Version>]) -> Result<(), Error>
src/engine/registry.rs:68  pub fn view_at(&self, stamp: Stamp) -> Result<Arc<Version>, Error>
src/engine/registry.rs:77  pub fn view_durable(&self) -> Result<Arc<Version>, Error>
src/engine/registry.rs:82  pub fn advance_recovered(&self, stamp: Stamp) -> Result<(), Error>
src/engine/registry.rs:92  pub fn prune_before(&self, floor: Stamp) -> Result<(), Error>
src/engine/segments.rs:7  pub enum SegmentClass
src/engine/segments.rs:19  pub struct EraseAttestation
src/engine/segments.rs:29  pub trait SegmentOwner: Send + Sync
src/engine/version.rs:11  pub struct VersionEntry
src/engine/version.rs:21  pub fn new(cluster: ClusterId, branch: BranchId, key: Vec<u8>, stamp: Stamp, kind: u8, payload: Vec<u8>) -> Self
src/engine/version.rs:27  pub struct Version
src/engine/version.rs:38  pub fn empty() -> Self { Self::default() }
src/engine/version.rs:40  pub fn new(
src/engine/version.rs:53  pub fn commit(
src/engine/version.rs:76  pub fn visible_stamp(&self) -> Stamp { self.visible_stamp }
src/engine/version.rs:77  pub fn durable_stamp(&self) -> Stamp { self.durable_stamp }
src/engine/version.rs:78  pub fn gc_floor(&self) -> Stamp { self.gc_floor }
src/engine/version.rs:79  pub fn partitions(&self) -> &[Arc<DetachedPartition>] { &self.partitions }
src/engine/version.rs:80  pub fn durable_partitions(&self) -> &[Arc<DetachedPartition>] { &self.durable_partitions }
src/engine/version.rs:81  pub fn changes(&self) -> &[VersionEntry] { &self.changes }
src/engine/version.rs:85  pub fn get(&self, cluster: ClusterId, branch: BranchId, key: &[u8], cap: Stamp) -> Option<&VersionEntry>
src/engine/version.rs:102  pub fn get_on_branch<'a>(
src/engine/version.rs:118  pub fn at_stamp(&self, stamp: Stamp) -> Result<Self, Error>
src/engine/version.rs:133  pub fn with_gc_floor(&self, gc_floor: Stamp) -> Result<Self, Error>
src/engine/version.rs:150  pub fn compact_history(&self, gc_floor: Stamp) -> Result<Self, Error>
src/engine/views.rs:17  pub enum SnapshotPoint
src/engine/views.rs:32  pub struct VersionView
src/engine/views.rs:38  pub fn capture(
src/engine/views.rs:71  pub fn point(&self) -> SnapshotPoint { self.point }
src/engine/views.rs:75  pub fn stamp(&self) -> Stamp { self.root.visible_stamp() }
src/engine/views.rs:79  pub fn durable_stamp(&self) -> Stamp { self.root.durable_stamp() }
src/engine/views.rs:81  pub fn root(&self) -> &Arc<Version> { &self.root }
src/engine/views.rs:87  pub fn point_mutation(
src/engine/volatile.rs:15  pub struct ChangeDisposition
src/engine/volatile.rs:24  pub fn validate_change(
src/engine/volatile.rs:86  pub struct VolatileValue
src/engine/volatile.rs:98  pub struct VolatileStore
src/engine/volatile.rs:107  pub fn new() -> Self { Self { state: Ranked::new(State::default()) } }
src/engine/volatile.rs:109  pub fn put(
src/engine/volatile.rs:131  pub fn get(
src/engine/volatile.rs:149  pub fn delete(
src/engine/volatile.rs:162  pub fn erase_space(&self, space: SpaceId) -> Result<usize, Error>
src/engine/volatile.rs:169  pub fn erase_cluster(&self, cluster: ClusterId) -> Result<usize, Error>
src/engine/volatile.rs:176  pub fn clear(&self) -> Result<usize, Error>
src/engine/volatile.rs:183  pub fn len(&self) -> Result<usize, Error> { Ok(self.state.lock()?.0.len()) }
src/engine/volatile.rs:184  pub fn is_empty(&self) -> Result<bool, Error> { Ok(self.state.lock()?.0.is_empty()) }
src/env/anchor.rs:20  pub struct MemAnchor
src/env/anchor.rs:25  pub fn new(initial: AnchorValue) -> Self
src/env/anchor.rs:66  pub struct FileAnchor
src/env/anchor.rs:73  pub fn open_or_init(
src/env/budget.rs:11  pub struct BudgetLimits
src/env/budget.rs:19  pub fn from_total(total_bytes: u64) -> Self
src/env/budget.rs:30  pub fn with_class(mut self, class: ResourceClass, bytes: u64) -> Self
src/env/budget.rs:52  pub struct Budget { inner: Arc<BudgetInner> }
src/env/budget.rs:55  pub fn new(limits: BudgetLimits) -> Result<Self, Error>
src/env/budget.rs:65  pub fn from_total(total_bytes: u64) -> Result<Self, Error>
src/env/budget.rs:69  pub fn used(&self) -> u64 { self.inner.total_used.load(Ordering::Acquire) }
src/env/budget.rs:70  pub fn class_used(&self, class: ResourceClass) -> u64
src/env/budget.rs:73  pub fn class_limit(&self, class: ResourceClass) -> u64
src/env/faultfs.rs:11  pub enum Fault
src/env/faultfs.rs:22  pub struct FaultPlan
src/env/faultfs.rs:36  pub struct FaultFs<F: Fs>
src/env/faultfs.rs:42  pub fn new(inner: F, plan: FaultPlan) -> Self
src/env/faultfs.rs:55  pub fn op_count(&self) -> u64
src/env/faultfs.rs:59  pub fn bytes_read(&self, role: FileRole) -> u64
src/env/faultfs.rs:65  pub fn inner(&self) -> &F { &self.inner }
src/env/faultfs.rs:68  pub fn ghost_journal(&self) -> Result<Vec<Vec<u8>>, Error>
src/env/faultfs.rs:76  pub fn crash_image(&self) -> Result<super::memfs::MemFs, Error>
src/env/governor.rs:12  pub enum WorkClass
src/env/governor.rs:23  pub struct GovernorConfig
src/env/governor.rs:34  pub fn validate(self) -> bool
src/env/governor.rs:41  pub struct Governor
src/env/governor.rs:47  pub fn new(config: GovernorConfig) -> Option<Self>
src/env/governor.rs:56  pub fn observe_free_space(&self, free_bytes: u64) -> bool
src/env/governor.rs:77  pub fn is_disk_full(&self) -> bool
src/env/governor.rs:83  pub fn admits(&self, work: WorkClass) -> bool
src/env/governor.rs:92  pub const fn maintenance_io_basis_points(&self) -> u16
src/env/memfs.rs:11  pub struct MemFs
src/env/memfs.rs:52  pub fn new() -> Self
src/env/memfs.rs:58  pub fn crash_image(&self) -> Result<Self, Error>
src/env/memfs.rs:82  pub fn set_free_space(&self, bytes: u64)
src/env/mod.rs:2  pub mod anchor
src/env/mod.rs:3  pub mod budget
src/env/mod.rs:4  pub mod faultfs
src/env/mod.rs:5  pub mod governor
src/env/mod.rs:6  pub mod memfs
src/env/mod.rs:7  pub mod sim
src/env/sim.rs:8  pub struct SimClock
src/env/sim.rs:16  pub fn new(initial_utc_millis: u64) -> Self
src/env/sim.rs:25  pub fn advance_mono_nanos(&self, delta: u64)
src/env/sim.rs:31  pub fn advance_utc_millis(&self, delta: u64)
src/env/sim.rs:37  pub fn set_skew(&self, skew: ClockSkew)
src/env/sim.rs:83  pub struct SimEntropy
src/env/sim.rs:88  pub fn new(seed: u64) -> Self
src/erase/cluster_table.rs:7  pub enum ClusterState { Live, Retired, Shredded, Purged }
src/erase/cluster_table.rs:10  pub struct ClusterEntry
src/erase/cluster_table.rs:21  pub struct ClusterTable { entries: BTreeMap<ClusterId, ClusterEntry> }
src/erase/cluster_table.rs:24  pub fn new() -> Self { Self::default() }
src/erase/cluster_table.rs:26  pub fn register(&mut self, id: ClusterId, protection: Protection) -> Result<&ClusterEntry, Error>
src/erase/cluster_table.rs:36  pub fn get(&self, id: ClusterId) -> Option<&ClusterEntry> { self.entries.get(&id) }
src/erase/cluster_table.rs:37  pub fn len(&self) -> usize { self.entries.len() }
src/erase/cluster_table.rs:38  pub fn is_empty(&self) -> bool { self.entries.is_empty() }
src/erase/cluster_table.rs:39  pub fn iter(&self) -> impl Iterator<Item = &ClusterEntry> { self.entries.values() }
src/erase/cluster_table.rs:41  pub fn mark_retired(&mut self, id: ClusterId, stamp: Stamp) -> Result<(), Error>
src/erase/cluster_table.rs:50  pub fn mark_shredded(&mut self, id: ClusterId) -> Result<(), Error>
src/erase/cluster_table.rs:59  pub fn mark_purged(&mut self, id: ClusterId) -> Result<(), Error>
src/erase/cluster_table.rs:68  pub fn require_live(&self, id: ClusterId) -> Result<&ClusterEntry, Error>
src/erase/gate.rs:14  pub struct GateIndex
src/erase/gate.rs:20  pub fn new() -> Self { Self::default() }
src/erase/gate.rs:21  pub fn generation(&self) -> u64 { self.generation }
src/erase/gate.rs:22  pub fn erased_at(&self, cluster: ClusterId) -> Option<Stamp> { self.erased_at.get(&cluster).copied() }
src/erase/gate.rs:23  pub fn len(&self) -> usize { self.erased_at.len() }
src/erase/gate.rs:24  pub fn is_empty(&self) -> bool { self.erased_at.is_empty() }
src/erase/gate.rs:28  pub fn check_visible(&self, cluster: ClusterId, _view_stamp: Stamp) -> Result<(), Error>
src/erase/gate.rs:38  pub fn apply_cluster_gate(
src/erase/ledger.rs:9  pub const MAX_LEDGER_RECORD_BYTES: usize = 256 * 1024
src/erase/ledger.rs:10  pub const MAX_INTENT_TOMBS: usize = 4096
src/erase/ledger.rs:11  pub const MAX_LEDGER_BYTES: u64 = 64 * 1024 * 1024
src/erase/ledger.rs:17  pub enum LedgerEventKind
src/erase/ledger.rs:39  pub struct LedgerEvent
src/erase/ledger.rs:51  pub struct LedgerRecord
src/erase/ledger.rs:61  pub struct LedgerWriter<'a>
src/erase/ledger.rs:71  pub fn recover(file: &'a dyn File, key: &'a Key32, mac_impl: &'a dyn Mac) -> Result<(Self, Vec<LedgerRecord>), Error>
src/erase/ledger.rs:93  pub fn append(&mut self, clock_sample: u64, event: LedgerEvent) -> Result<LedgerRecord, Error>
src/erase/ledger.rs:106  pub fn head(&self) -> [u8; 32] { self.head }
src/erase/ledger.rs:107  pub fn next_sequence(&self) -> u64 { self.next_sequence }
src/erase/ledger.rs:108  pub fn is_poisoned(&self) -> bool { self.poisoned }
src/erase/ledger.rs:111  pub fn encode_record(record: &LedgerRecord, key: &Key32, mac: &dyn Mac) -> Result<Vec<u8>, Error>
src/erase/ledger.rs:121  pub fn decode_record(input: &[u8], expected_sequence: u64, previous_hash: [u8; 32], key: &Key32, mac: &dyn Mac) -> Result<(LedgerRecord, usize), Error>
src/erase/links.rs:12  pub const DEFAULT_MAX_LINKS: usize = 8
src/erase/links.rs:13  pub const DEFAULT_MAX_GATE_CLUSTERS: usize = 4096
src/erase/links.rs:14  pub const DEFAULT_MAX_LINK_DEPTH: usize = 64
src/erase/links.rs:17  pub struct LinkPlan
src/erase/links.rs:24  pub fn generation(&self) -> u64 { self.generation }
src/erase/links.rs:25  pub fn derived(&self) -> ClusterId { self.derived }
src/erase/links.rs:26  pub fn sources(&self) -> &[ClusterId] { &self.sources }
src/erase/links.rs:30  pub struct LinkGraph
src/erase/links.rs:51  pub fn new(max_links: usize, max_gate_clusters: usize, max_depth: usize) -> Result<Self, Error>
src/erase/links.rs:58  pub fn generation(&self) -> u64 { self.generation }
src/erase/links.rs:59  pub fn direct_sources(&self, derived: ClusterId) -> impl Iterator<Item = ClusterId> + '_
src/erase/links.rs:65  pub fn plan_link(&self, clusters: &ClusterTable, derived: ClusterId, sources: &[ClusterId]) -> Result<LinkPlan, Error>
src/erase/links.rs:93  pub fn apply_link(&mut self, clusters: &ClusterTable, plan: LinkPlan) -> Result<(), Error>
src/erase/links.rs:118  pub fn erase_closure(&self, source: ClusterId) -> Result<Vec<ClusterId>, Error>
src/erase/links.rs:126  pub fn unlink(&mut self, expected_generation: u64, derived: ClusterId) -> Result<(), Error>
src/erase/mod.rs:2  pub mod cluster_table
src/erase/mod.rs:3  pub mod gate
src/erase/mod.rs:4  pub mod ledger
src/erase/mod.rs:5  pub mod links
src/erase/mod.rs:6  pub mod protocol
src/erase/mod.rs:7  pub mod receipt
src/erase/mod.rs:8  pub mod restrictions
src/erase/mod.rs:9  pub mod tickets
src/erase/protocol.rs:22  pub const MAX_GATE_CLUSTERS: usize = 4096
src/erase/protocol.rs:25  pub struct EraseRequest
src/erase/protocol.rs:45  pub struct EraseCoordinator
src/erase/protocol.rs:82  pub fn new() -> Self
src/erase/protocol.rs:90  pub fn ticket(&self, ticket: Ticket) -> Option<&EraseTicket>
src/erase/protocol.rs:94  pub fn list_open_tickets(&self) -> Vec<Ticket>
src/erase/protocol.rs:103  pub fn recover_from_record(&mut self, record: &LedgerRecord) -> Result<(), Error>
src/erase/protocol.rs:230  pub fn record_intent(
src/erase/protocol.rs:316  pub fn apply_gate(
src/erase/protocol.rs:379  pub fn destroy_keys(
src/erase/protocol.rs:455  pub fn start_purge(
src/erase/protocol.rs:490  pub fn complete_purge(
src/erase/protocol.rs:536  pub fn finalize(
src/erase/receipt.rs:11  pub enum MeasureStatus
src/erase/receipt.rs:18  pub enum MeasureKind
src/erase/receipt.rs:26  pub struct MeasureReport
src/erase/receipt.rs:32  pub struct ReceiptStageEntry
src/erase/receipt.rs:39  pub struct ProviderAttestation
src/erase/receipt.rs:46  pub struct AnchorEvidence
src/erase/receipt.rs:52  pub struct ErasureReceipt
src/erase/receipt.rs:76  pub enum ReceiptVerdict
src/erase/receipt.rs:83  pub fn compute_status_caps(
src/erase/receipt.rs:137  pub fn build_canonical_bytes(&self) -> Vec<u8>
src/erase/receipt.rs:208  pub fn verify_receipt(
src/erase/restrictions.rs:12  pub struct Effects(u8)
src/erase/restrictions.rs:15  pub const BLOCK_ERASE: u8 = 0b001
src/erase/restrictions.rs:16  pub const BLOCK_WRITE: u8 = 0b010
src/erase/restrictions.rs:17  pub const BLOCK_READ: u8 = 0b100
src/erase/restrictions.rs:20  pub fn new(bits: u8) -> Result<Self, Error>
src/erase/restrictions.rs:27  pub fn blocks_erase(self) -> bool { self.0 & Self::BLOCK_ERASE != 0 }
src/erase/restrictions.rs:28  pub fn blocks_write(self) -> bool { self.0 & Self::BLOCK_WRITE != 0 }
src/erase/restrictions.rs:29  pub fn blocks_read(self) -> bool { self.0 & Self::BLOCK_READ != 0 }
src/erase/restrictions.rs:30  pub fn bits(self) -> u8 { self.0 }
src/erase/restrictions.rs:34  pub enum LiftPolicy { OwnerOrAdmin, FourEyes }
src/erase/restrictions.rs:36  pub struct RestrictionSpec
src/erase/restrictions.rs:44  pub struct Restriction
src/erase/restrictions.rs:55  pub fn id(&self) -> u64 { self.id }
src/erase/restrictions.rs:56  pub fn reason(&self) -> u16 { self.reason }
src/erase/restrictions.rs:57  pub fn effects(&self) -> Effects { self.effects }
src/erase/restrictions.rs:58  pub fn set_stamp(&self) -> Stamp { self.set_stamp }
src/erase/restrictions.rs:59  pub fn lifted_stamp(&self) -> Option<Stamp> { self.lifted_stamp }
src/erase/restrictions.rs:60  pub fn is_active_at(&self, monotone_now: u64) -> bool
src/erase/restrictions.rs:65  pub enum LiftAuthorization<'a>
src/erase/restrictions.rs:76  pub struct AdminCap { _seal: () }
src/erase/restrictions.rs:81  pub fn from_authorized_context() -> Self { Self { _seal: () } }
src/erase/restrictions.rs:84  pub struct RestrictionTable
src/erase/restrictions.rs:92  pub fn new(policy: LiftPolicy) -> Self
src/erase/restrictions.rs:96  pub fn get(&self, id: u64) -> Option<&Restriction> { self.entries.get(&id) }
src/erase/restrictions.rs:97  pub fn iter(&self) -> impl Iterator<Item = &Restriction> { self.entries.values() }
src/erase/restrictions.rs:98  pub fn active_at(&self, now: u64) -> impl Iterator<Item = &Restriction>
src/erase/restrictions.rs:102  pub fn ensure_erase_allowed(&self, now: u64) -> Result<(), Error>
src/erase/restrictions.rs:109  pub fn ensure_write_allowed(&self, now: u64) -> Result<(), Error>
src/erase/restrictions.rs:116  pub fn ensure_read_allowed(&self, now: u64) -> Result<(), Error>
src/erase/restrictions.rs:124  pub fn apply_set(&mut self, id: u64, stamp: Stamp, spec: RestrictionSpec) -> Result<(), Error>
src/erase/restrictions.rs:141  pub fn validate_lift(
src/erase/restrictions.rs:163  pub fn apply_lift(&mut self, id: u64, stamp: Stamp) -> Result<(), Error>
src/erase/tickets.rs:10  pub enum EraseLevel { Logical, Physical, Scrubbed, Cryptographic }
src/erase/tickets.rs:13  pub enum EraseScope { Cluster, Range, Branch, Space }
src/erase/tickets.rs:16  pub enum TicketStage
src/erase/tickets.rs:28  pub enum Residual
src/erase/tickets.rs:36  pub enum TicketStatus
src/erase/tickets.rs:46  pub struct CompletionEvidence
src/erase/tickets.rs:56  pub enum TicketEvidence
src/erase/tickets.rs:71  pub struct EraseTicket
src/erase/tickets.rs:84  pub fn new(ticket: Ticket, scope: EraseScope, protection: Protection, level: EraseLevel) -> Result<Self, Error>
src/erase/tickets.rs:92  pub fn ticket(&self) -> Ticket { self.ticket }
src/erase/tickets.rs:93  pub fn scope(&self) -> EraseScope { self.scope }
src/erase/tickets.rs:94  pub fn level(&self) -> EraseLevel { self.level }
src/erase/tickets.rs:95  pub fn stage(&self) -> TicketStage { self.stage }
src/erase/tickets.rs:96  pub fn status(&self) -> TicketStatus { self.status }
src/erase/tickets.rs:97  pub fn gate_stamp(&self) -> Option<Stamp> { self.gate_stamp }
src/erase/tickets.rs:98  pub fn last_sequence(&self) -> u64 { self.last_seq }
src/erase/tickets.rs:101  pub fn escalate(&mut self, level: EraseLevel) -> Result<(), Error>
src/erase/tickets.rs:112  pub fn apply(&mut self, seq: u64, event: TicketEvidence) -> Result<(), Error>
src/keys/cache.rs:28  pub struct KeyCache
src/keys/cache.rs:37  pub struct Key32Handle
src/keys/cache.rs:47  pub fn with_key<T>(&self, use_key: impl FnOnce(&Key32) -> Result<T, Error>) -> Result<T, Error>
src/keys/cache.rs:64  pub fn new(
src/keys/cache.rs:78  pub fn get(&self, cluster: ClusterId, epoch: u32) -> Result<Key32Handle, Error>
src/keys/cache.rs:133  pub fn prefetch(&self, request: &[(ClusterId, u32)]) -> Result<(), Error>
src/keys/cache.rs:174  pub fn invalidate(&self, cluster: ClusterId) -> Result<u64, Error>
src/keys/mod.rs:2  pub mod cache
src/keys/mod.rs:3  pub mod wrapper
src/keys/wrapper.rs:13  pub fn wrap_root(
src/keys/wrapper.rs:22  pub fn unwrap_root(cipher: &dyn Cipher, kek: &Key32, wrapped: &[u8]) -> Result<Key32, Error>
src/keys/wrapper.rs:26  pub fn wrap_space_dek(
src/keys/wrapper.rs:38  pub fn unwrap_space_dek(
src/lib.rs:8  pub use contextra_storage_ports as ports
src/lib.rs:10  pub mod catalog
src/lib.rs:11  pub mod checkpoint
src/lib.rs:12  pub mod codec
src/lib.rs:13  pub mod config
src/lib.rs:14  pub mod engine
src/lib.rs:15  pub mod env
src/lib.rs:16  pub mod erase
src/lib.rs:17  pub mod keys
src/lib.rs:18  pub mod maint
src/lib.rs:19  pub mod metrics
src/lib.rs:20  pub mod recovery
src/lib.rs:21  pub mod spaces
src/lib.rs:22  pub mod store
src/lib.rs:23  pub mod sync
src/lib.rs:25  pub mod testing
src/lib.rs:82  pub struct ClusterSpec
src/lib.rs:185  pub struct Storage
src/lib.rs:191  pub struct StorageView
src/lib.rs:200  pub fn open(config: Config) -> Result<Storage, Error>
src/lib.rs:287  pub fn catalog(&self) -> &CatalogState
src/lib.rs:291  pub fn catalog_snapshot(&self) -> Result<CatalogState, Error>
src/lib.rs:296  pub fn instance_id(&self) -> InstanceId
src/lib.rs:300  pub fn erase_epoch(&self) -> Result<u64, Error>
src/lib.rs:304  pub fn create_space(&self, name: &str, options: SpaceOptions) -> Result<SpaceId, Error>
src/lib.rs:354  pub fn create_cluster(&self, spec: ClusterSpec) -> Result<ClusterId, Error>
src/lib.rs:397  pub fn list_clusters(&self, protection: Option<Protection>) -> Result<Vec<ClusterEntry>, Error>
src/lib.rs:407  pub fn link_clusters(&self, derived: ClusterId, sources: &[ClusterId]) -> Result<(), Error>
src/lib.rs:414  pub fn restrict_cluster(
src/lib.rs:430  pub fn lift_restriction(
src/lib.rs:447  pub fn apply(&self, change: Change) -> Result<Receipt, Error>
src/lib.rs:659  pub fn durable_barrier(&self) -> Result<Receipt, Error>
src/lib.rs:682  pub fn view(&self) -> Result<StorageView, Error>
src/lib.rs:686  pub fn view_durable(&self) -> Result<StorageView, Error>
src/lib.rs:690  pub fn view_at(&self, stamp: Stamp) -> Result<StorageView, Error>
src/lib.rs:706  pub fn branch_fork(
src/lib.rs:727  pub fn branch_drop(&self, cluster: ClusterId, branch: BranchId) -> Result<(), Error>
src/lib.rs:738  pub fn flush_cluster(&self, space: SpaceId, cluster: ClusterId) -> Result<Option<PathBuf>, Error>
src/lib.rs:849  pub fn erase_cluster(
src/lib.rs:1003  pub fn checkpoint_create(&self, target_dir: &Path) -> Result<CheckpointManifest, Error>
src/lib.rs:1035  pub fn checkpoint_restore(
src/lib.rs:1062  pub fn stamp(&self) -> Stamp
src/lib.rs:1066  pub fn durable_stamp(&self) -> Stamp
src/lib.rs:1070  pub fn renew(&mut self, extend_nanos: u64) -> Result<(), Error>
src/lib.rs:1081  pub fn filter_visible(&self, cluster: ClusterId) -> Result<(), Error>
src/lib.rs:1092  pub fn get(
src/lib.rs:1186  pub fn page(
src/maint/blob_gc.rs:16  pub struct BlobGcJob
src/maint/blob_gc.rs:20  pub struct BlobGcRunner
src/maint/blob_gc.rs:24  pub fn plan_gc(
src/maint/blob_gc.rs:35  pub fn execute_unlinks(
src/maint/compaction.rs:16  pub struct CompactionPlan
src/maint/compaction.rs:22  pub struct Compactor
src/maint/compaction.rs:26  pub fn install_compacted_run(
src/maint/flush.rs:15  pub struct FlushJob
src/maint/flush.rs:22  pub struct FlushRunner
src/maint/flush.rs:30  pub fn install_run(
src/maint/mod.rs:2  pub mod blob_gc
src/maint/mod.rs:3  pub mod compaction
src/maint/mod.rs:4  pub mod flush
src/maint/mod.rs:5  pub mod purge
src/maint/mod.rs:6  pub mod reserve
src/maint/mod.rs:7  pub mod scheduler
src/maint/mod.rs:8  pub mod scrub
src/maint/purge.rs:16  pub struct PurgePlan
src/maint/purge.rs:22  pub struct PurgeRunner
src/maint/purge.rs:27  pub fn plan_purge(
src/maint/purge.rs:56  pub fn unlink_dead_files(
src/maint/reserve.rs:9  pub const REQUIRED_PURGE_RESERVE_BYTES: u64 = 128 * 1024 * 1024
src/maint/reserve.rs:15  pub struct ScratchReserve
src/maint/reserve.rs:22  pub fn open(fs: &dyn Fs, root: &Path, slot_bytes: u64) -> Result<Self, Error>
src/maint/reserve.rs:68  pub fn slot_bytes(&self) -> u64 { self.slot_bytes }
src/maint/reserve.rs:70  pub fn writer_slot(&self) -> Result<ScratchSlot, Error>
src/maint/reserve.rs:78  pub fn recovery_path(&self) -> PathBuf { slot_path(&self.root, 1) }
src/maint/reserve.rs:81  pub struct ScratchSlot
src/maint/reserve.rs:87  pub fn path(&self) -> &Path { &self.path }
src/maint/scheduler.rs:9  pub enum MaintPriority
src/maint/scheduler.rs:18  pub struct MaintTask
src/maint/scheduler.rs:38  pub struct MaintScheduler
src/maint/scheduler.rs:44  pub fn new() -> Self
src/maint/scheduler.rs:51  pub fn schedule(&mut self, priority: MaintPriority, task_name: &'static str) -> u64
src/maint/scheduler.rs:62  pub fn pop_next(&mut self) -> Option<MaintTask>
src/maint/scheduler.rs:66  pub fn is_empty(&self) -> bool
src/maint/scheduler.rs:70  pub fn len(&self) -> usize
src/maint/scheduler.rs:75  pub fn compute_gc_floor(
src/maint/scrub.rs:8  pub struct ScrubReport
src/maint/scrub.rs:14  pub struct Scrubber
src/maint/scrub.rs:18  pub fn scrub_block(raw_block_bytes: &[u8]) -> Result<bool, Error>
src/metrics.rs:6  pub enum MetricKind
src/metrics.rs:13  pub enum MetricUnit
src/metrics.rs:22  pub struct MetricDef
src/metrics.rs:46  pub const METRICS: &[MetricDef] = &[
src/metrics.rs:85  pub fn validate_labels(name: MetricName, labels: &[MetricLabel]) -> bool
src/metrics.rs:97  pub fn validate_sample(name: MetricName, sample: MetricSample) -> bool
src/recovery/integrity.rs:6  pub enum KeyRecoveryState { Available, Shredded }
src/recovery/integrity.rs:11  pub fn reconcile_cluster_key(probe: KeyState, erase_proven: bool) -> Result<KeyRecoveryState, Error>
src/recovery/integrity.rs:21  pub struct IntegrityDecision
src/recovery/integrity.rs:28  pub fn check_anchor(
src/recovery/mod.rs:2  pub mod integrity
src/recovery/mod.rs:3  pub mod open
src/recovery/mod.rs:4  pub mod replay
src/recovery/mod.rs:5  pub mod scan
src/recovery/open.rs:20  pub struct OpenedInstance
src/recovery/open.rs:36  pub fn open_instance(
src/recovery/replay.rs:15  pub trait ReplaySink
src/recovery/replay.rs:24  pub trait ControlReplayApplier: Send + Sync
src/recovery/replay.rs:30  pub struct EngineReplaySink
src/recovery/replay.rs:38  pub fn new(
src/recovery/replay.rs:88  pub struct ReplaySummary
src/recovery/replay.rs:97  pub fn replay(scan: &RecoveryScan<'_>, sink: &mut dyn ReplaySink) -> Result<ReplaySummary, Error>
src/recovery/replay.rs:101  pub fn replay_with_retired(
src/recovery/scan.rs:10  pub struct RecoveryScan<'a>
src/recovery/scan.rs:19  pub trait GateScopeResolver
src/recovery/scan.rs:25  pub fn scan<'a>(
src/recovery/scan.rs:36  pub fn scan_with_resolver<'a>(
src/spaces.rs:6  pub use contextra_storage_ports::DataClass
src/spaces.rs:8  pub const DEFAULT_PURE_THRESHOLD: u64 = 8 * 1024 * 1024
src/spaces.rs:9  pub const DEFAULT_VALUE_SEPARATION_MIN: u32 = 1024
src/spaces.rs:10  pub const DEFAULT_MAX_KEY_BYTES: u32 = 1024
src/spaces.rs:11  pub const MAX_KEY_BYTES: u32 = 4096
src/spaces.rs:12  pub const MAX_WEIGHT_SLOTS: u8 = 4
src/spaces.rs:15  pub enum SpaceClass { Durable, Volatile }
src/spaces.rs:18  pub enum ExpiryBucket { Month, Quarter, Year }
src/spaces.rs:21  pub enum PackPolicy
src/spaces.rs:27  pub enum SealPolicy { ByMode, Cluster }
src/spaces.rs:30  pub enum HistoryPolicy { KeepLatest, KeepUntilFloor, AppendOnly }
src/spaces.rs:33  pub enum Compression { None }
src/spaces.rs:36  pub struct SpaceOptions
src/spaces.rs:71  pub fn volatile() -> Self
src/spaces.rs:76  pub fn validate(&self, integrity: IntegrityMode) -> Result<(), Error>
src/spaces.rs:109  pub fn protection_allowed(&self, protection: Protection) -> bool
src/spaces.rs:116  pub struct SpaceSpec
src/spaces.rs:123  pub fn validate(&self, integrity: IntegrityMode) -> Result<(), Error>
src/store/blob/gc_index.rs:10  pub struct BlobId(pub u64)
src/store/blob/gc_index.rs:13  pub struct BlobIncarnation(pub u64)
src/store/blob/gc_index.rs:16  pub struct BlobRef
src/store/blob/gc_index.rs:29  pub struct BlobGcIndex
src/store/blob/gc_index.rs:34  pub fn new() -> Self { Self::default() }
src/store/blob/gc_index.rs:36  pub fn add_reference(&mut self, reference: BlobRef) -> Result<(), Error>
src/store/blob/gc_index.rs:46  pub fn remove_reference(&mut self, reference: BlobRef) -> Result<(), Error>
src/store/blob/gc_index.rs:56  pub fn remove_candidates(&self, known_files: impl IntoIterator<Item = (BlobId, BlobIncarnation)>) -> Vec<(BlobId, BlobIncarnation)>
src/store/blob/gc_index.rs:62  pub fn live_reference_count(&self, blob: BlobId, incarnation: BlobIncarnation) -> u64
src/store/blob/mod.rs:2  pub mod gc_index
src/store/blob/mod.rs:3  pub mod reader
src/store/blob/mod.rs:4  pub mod record
src/store/blob/mod.rs:5  pub mod writer
src/store/blob/reader.rs:23  pub struct BlobReader
src/store/blob/reader.rs:30  pub fn open(
src/store/blob/reader.rs:58  pub fn header(&self) -> &FileHeader
src/store/blob/reader.rs:62  pub fn file_len(&self) -> u64
src/store/blob/reader.rs:66  pub fn read_raw_record(
src/store/blob/reader.rs:87  pub fn read_sealed(
src/store/blob/record.rs:7  pub const RECORD_UID_LEN: usize = 16
src/store/blob/record.rs:8  pub const RECORD_OVERHEAD: usize = RECORD_UID_LEN + 4 + 4
src/store/blob/record.rs:11  pub struct BlobRecord
src/store/blob/record.rs:24  pub fn encode(&self, max_frame_bytes: usize) -> Result<Vec<u8>, Error>
src/store/blob/record.rs:39  pub fn decode(bytes: &[u8], max_frame_bytes: usize) -> Result<Self, Error>
src/store/blob/writer.rs:12  pub const BLOB_FILE_MAGIC: [u8; 8] = *b"CTXBLB01"
src/store/blob/writer.rs:13  pub const BLOB_FILE_KIND: u8 = 2
src/store/blob/writer.rs:14  pub const BLOB_PREALLOC_BYTES: u64 = 1024 * 1024
src/store/blob/writer.rs:24  pub struct BlobRef
src/store/blob/writer.rs:32  pub struct BlobSealContext<'a>
src/store/blob/writer.rs:44  pub fn blob_filename(blob_id: u64, incarnation: u64) -> String
src/store/blob/writer.rs:48  pub struct BlobWriter
src/store/blob/writer.rs:61  pub fn create(
src/store/blob/writer.rs:105  pub fn blob_id(&self) -> u64
src/store/blob/writer.rs:109  pub fn incarnation(&self) -> u64
src/store/blob/writer.rs:113  pub fn append_sealed(
src/store/blob/writer.rs:146  pub fn append_raw_record(
src/store/blob/writer.rs:182  pub fn finish(mut self, fs: &dyn Fs) -> Result<PathBuf, Error>
src/store/blob/writer.rs:222  pub fn repack_survivors(
src/store/mod.rs:2  pub mod blob
src/store/mod.rs:3  pub mod redo
src/store/mod.rs:4  pub mod run
src/store/redo/cut.rs:9  pub struct BarrierCut
src/store/redo/cut.rs:18  pub fn cut_segment(
src/store/redo/cut.rs:57  pub fn can_retire_segment(
src/store/redo/frame.rs:13  pub enum FrameKind { Change, Gate, Control, Reserve }
src/store/redo/frame.rs:16  pub struct OpHead
src/store/redo/frame.rs:27  pub struct FrameHead
src/store/redo/frame.rs:39  pub struct EncodedFrame
src/store/redo/frame.rs:46  pub struct OpPayload<'a>
src/store/redo/frame.rs:52  pub struct DecodedFrame<'a>
src/store/redo/frame.rs:73  pub fn encode_frame(frame: &EncodedFrame, previous: &[u8; 32], key: &Key32, mac: &dyn Mac) -> Result<Vec<u8>, Error>
src/store/redo/frame.rs:120  pub fn decode_frame<'a>(input: &'a [u8], previous: &[u8; 32], key: &Key32, mac: &dyn Mac) -> Result<DecodedFrame<'a>, Error>
src/store/redo/mod.rs:2  pub mod cut
src/store/redo/mod.rs:3  pub mod frame
src/store/redo/mod.rs:4  pub mod reader
src/store/redo/mod.rs:5  pub mod reserve
src/store/redo/mod.rs:6  pub mod segment
src/store/redo/mod.rs:7  pub mod torn_tail
src/store/redo/mod.rs:8  pub mod writer
src/store/redo/reader.rs:13  pub struct ScanResult<'a>
src/store/redo/reader.rs:21  pub struct RedoReader
src/store/redo/reader.rs:27  pub fn scan<'a>(
src/store/redo/reader.rs:124  pub fn heads_only<'a>(&self, result: &'a ScanResult<'a>) -> impl Iterator<Item = (&'a super::frame::FrameHead, Stamp)> + 'a
src/store/redo/reserve.rs:8  pub struct ReserveWriter
src/store/redo/reserve.rs:18  pub fn initialize(file: Box<dyn File>, capacity: u64) -> Result<Self, Error>
src/store/redo/reserve.rs:25  pub fn remaining(&self) -> u64 { self.capacity.saturating_sub(self.used) }
src/store/redo/reserve.rs:28  pub fn append_durable(&mut self, record: &[u8]) -> Result<u64, Error>
src/store/redo/segment.rs:5  pub const DEFAULT_SEGMENT_BYTES: u64 = 64 * 1024 * 1024
src/store/redo/segment.rs:6  pub const DEFAULT_MARK_BYTES: u64 = 8 * 1024 * 1024
src/store/redo/segment.rs:7  pub const DEFAULT_ALIGNMENT: u64 = 4096
src/store/redo/segment.rs:8  pub const GROUP_MAGIC: &[u8; 4] = b"R8G1"
src/store/redo/segment.rs:10  pub fn align_up(value: u64, alignment: u64) -> Result<u64, Error>
src/store/redo/segment.rs:19  pub fn group_padding(offset: u64, alignment: u64) -> Result<usize, Error>
src/store/redo/torn_tail.rs:4  pub enum TailState { Clean, Torn, CorruptMidLog }
src/store/redo/torn_tail.rs:8  pub fn tail_state(has_later_group: bool) -> TailState
src/store/redo/writer.rs:12  pub struct GroupAck
src/store/redo/writer.rs:18  pub struct RedoWriter<'a>
src/store/redo/writer.rs:31  pub fn new(
src/store/redo/writer.rs:46  pub fn durable_stamp(&self) -> Stamp { self.durable_stamp }
src/store/redo/writer.rs:50  pub fn is_poisoned(&self) -> bool { self.poisoned }
src/store/redo/writer.rs:52  pub fn append_group(&mut self, frames: &[EncodedFrame], durability: Durability) -> Result<GroupAck, Error>
src/store/redo/writer.rs:124  pub fn needs_rotation(&self, additional_bytes: u64) -> Result<bool, Error>
src/store/run/block.rs:15  pub const MAX_DECODED_BLOCK_BYTES: usize = 8 * 1024 * 1024
src/store/run/block.rs:16  pub const TARGET_BLOCK_BYTES: usize = 16 * 1024
src/store/run/block.rs:19  pub struct BlockEntry
src/store/run/block.rs:35  pub fn stamp(&self) -> Stamp { self.tag >> 8 }
src/store/run/block.rs:41  pub fn encode(entries: &[BlockEntry], max_bytes: usize) -> Result<Vec<u8>, Error>
src/store/run/block.rs:89  pub fn decode(bytes: &[u8], max_bytes: usize) -> Result<Vec<BlockEntry>, Error>
src/store/run/block.rs:172  pub fn compare(left: &BlockEntry, right: &BlockEntry) -> std::cmp::Ordering
src/store/run/builder.rs:16  pub const RUN_PREALLOC_BYTES: u64 = 4 * 1024 * 1024
src/store/run/builder.rs:17  pub const MAX_PACK_META_BYTES: usize = 1024 * 1024
src/store/run/builder.rs:19  pub struct BuiltRun { pub path: PathBuf, pub file_bytes: u64, pub directory: ClusterDirectory }
src/store/run/builder.rs:21  pub struct RunFileBuilder
src/store/run/builder.rs:32  pub fn create(fs: &dyn Fs, path: &Path, header: &FileHeader) -> Result<Self, Error>
src/store/run/builder.rs:42  pub fn append_section(
src/store/run/builder.rs:75  pub fn finish(mut self, pack_meta: &[u8], extensions: &[ExtensionBlock]) -> Result<BuiltRun, Error>
src/store/run/copy.rs:9  pub trait RawSectionChecksumVerifier
src/store/run/copy.rs:16  pub fn copy_sections(
src/store/run/directory.rs:10  pub const MAX_DIRECTORY_ENTRIES: usize = 1_000_000
src/store/run/directory.rs:13  pub struct ClusterDirectoryEntry
src/store/run/directory.rs:24  pub struct ClusterDirectory
src/store/run/directory.rs:29  pub fn new(entries: Vec<ClusterDirectoryEntry>) -> Result<Self, Error>
src/store/run/directory.rs:34  pub fn entries(&self) -> &[ClusterDirectoryEntry] { &self.entries }
src/store/run/directory.rs:36  pub fn find(&self, cluster: ClusterId) -> Option<&ClusterDirectoryEntry>
src/store/run/directory.rs:41  pub fn encode(&self) -> Result<Vec<u8>, Error>
src/store/run/directory.rs:65  pub fn decode(bytes: &[u8], max_entries: usize) -> Result<Self, Error>
src/store/run/extension.rs:8  pub const MAX_EXTENSION_BLOCKS: usize = 4096
src/store/run/extension.rs:11  pub struct ExtensionBlock
src/store/run/extension.rs:25  pub fn new(tag: u16, version: u16, payload: Vec<u8>) -> Result<Self, Error>
src/store/run/extension.rs:31  pub fn encode(blocks: &[ExtensionBlock], max_bytes: usize) -> Result<Vec<u8>, Error>
src/store/run/extension.rs:53  pub fn decode(bytes: &[u8], max_bytes: usize, max_blocks: usize) -> Result<Vec<ExtensionBlock>, Error>
src/store/run/filter.rs:8  pub const FILTER_KIND_BLOOM_V1: u8 = 1
src/store/run/filter.rs:9  pub const MAX_FILTER_BYTES: usize = 8 * 1024 * 1024
src/store/run/filter.rs:13  pub struct BloomFilter
src/store/run/filter.rs:29  pub fn new(expected_items: u32, false_positive_rate: f64, salt: [u8; 16]) -> Result<Self, Error>
src/store/run/filter.rs:46  pub fn insert(&mut self, key: &[u8])
src/store/run/filter.rs:55  pub fn may_contain(&self, key: &[u8]) -> bool
src/store/run/filter.rs:64  pub fn encode(&self) -> Result<Vec<u8>, Error>
src/store/run/filter.rs:79  pub fn decode(bytes: &[u8], max_bytes: usize) -> Result<Self, Error>
src/store/run/format.rs:5  pub const RUN_FOOTER_MAGIC: [u8; 8] = *b"CTXRUN01"
src/store/run/format.rs:6  pub const RUN_FOOTER_LEN: usize = 44
src/store/run/format.rs:9  pub struct RunFooter
src/store/run/format.rs:17  pub fn encode(self, file_len: u64) -> Result<[u8; RUN_FOOTER_LEN], Error>
src/store/run/format.rs:30  pub fn decode(bytes: &[u8], file_len: u64) -> Result<Self, Error>
src/store/run/mod.rs:2  pub mod block
src/store/run/mod.rs:3  pub mod builder
src/store/run/mod.rs:4  pub mod copy
src/store/run/mod.rs:5  pub mod directory
src/store/run/mod.rs:6  pub mod extension
src/store/run/mod.rs:7  pub mod filter
src/store/run/mod.rs:8  pub mod format
src/store/run/mod.rs:9  pub mod pack_meta
src/store/run/mod.rs:10  pub mod rangedel
src/store/run/mod.rs:11  pub mod reader
src/store/run/mod.rs:12  pub mod section
src/store/run/mod.rs:13  pub mod skip_scan
src/store/run/mod.rs:14  pub mod subindex
src/store/run/mod.rs:15  pub mod table_cache
src/store/run/pack_meta.rs:12  pub enum PackKind
src/store/run/pack_meta.rs:33  pub struct PackMeta
src/store/run/pack_meta.rs:42  pub fn encode(&self) -> Result<[u8; PACK_META_LEN], Error>
src/store/run/pack_meta.rs:68  pub fn decode(bytes: &[u8]) -> Result<Self, Error>
src/store/run/rangedel.rs:9  pub const MAX_RANGE_MARKERS: usize = 1_000_000
src/store/run/rangedel.rs:12  pub struct RangeMarker
src/store/run/rangedel.rs:29  pub fn stamp(&self) -> Result<Stamp, Error>
src/store/run/rangedel.rs:39  pub fn encode(markers: &[RangeMarker], max_bytes: usize) -> Result<Vec<u8>, Error>
src/store/run/rangedel.rs:60  pub fn decode(bytes: &[u8], max_bytes: usize, max_markers: usize) -> Result<Vec<RangeMarker>, Error>
src/store/run/reader.rs:14  pub const MAX_PACK_META_BYTES: usize = 1024 * 1024
src/store/run/reader.rs:15  pub const MAX_EXTENSION_REGION_BYTES: usize = 16 * 1024 * 1024
src/store/run/reader.rs:17  pub struct RunReader
src/store/run/reader.rs:28  pub fn open(
src/store/run/reader.rs:77  pub fn header(&self) -> &FileHeader { &self.header }
src/store/run/reader.rs:78  pub fn footer(&self) -> RunFooter { self.footer }
src/store/run/reader.rs:79  pub fn file_len(&self) -> u64 { self.file_len }
src/store/run/reader.rs:80  pub fn directory(&self) -> &ClusterDirectory { &self.directory }
src/store/run/reader.rs:81  pub fn pack_meta(&self) -> &[u8] { &self.pack_meta }
src/store/run/reader.rs:82  pub fn extensions(&self) -> &[ExtensionBlock] { &self.extensions }
src/store/run/reader.rs:84  pub fn read_section(&self, cluster: ClusterId, max_bytes: usize) -> Result<Option<Zeroizing<Vec<u8>>>, Error>
src/store/run/reader.rs:91  pub fn section_meta(&self, cluster: ClusterId) -> Option<&ClusterDirectoryEntry>
src/store/run/reader.rs:96  pub fn get_entry(
src/store/run/reader.rs:111  pub fn scan_entries(
src/store/run/section.rs:21  pub struct SectionSealContext<'a>
src/store/run/section.rs:34  pub struct EncodedSection
src/store/run/section.rs:43  pub struct SectionLayout
src/store/run/section.rs:57  pub struct RunChecksumVerifier
src/store/run/section.rs:68  pub fn encode_cluster_section(
src/store/run/section.rs:272  pub fn parse_section_layout(section: &[u8]) -> Result<SectionLayout, Error>
src/store/run/section.rs:360  pub struct SectionReadContext<'a>
src/store/run/section.rs:371  pub fn get_in_section(
src/store/run/section.rs:447  pub fn scan_section(
src/store/run/skip_scan.rs:6  pub struct BlockSkipQuery<'a>
src/store/run/skip_scan.rs:15  pub fn block_skip_scan<'a>(
src/store/run/subindex.rs:11  pub struct SubIndexEntry
src/store/run/subindex.rs:32  pub struct SubIndex { entries: Vec<SubIndexEntry> }
src/store/run/subindex.rs:35  pub fn new(entries: Vec<SubIndexEntry>) -> Result<Self, Error>
src/store/run/subindex.rs:40  pub fn entries(&self) -> &[SubIndexEntry] { &self.entries }
src/store/run/subindex.rs:42  pub fn candidate(&self, key: &[u8], stamp_cap: Stamp) -> Option<&SubIndexEntry>
src/store/run/subindex.rs:47  pub fn encode(&self, max_bytes: usize) -> Result<Vec<u8>, Error>
src/store/run/subindex.rs:73  pub fn decode(bytes: &[u8], max_bytes: usize, max_entries: usize) -> Result<Self, Error>
src/store/run/table_cache.rs:23  pub struct TableCache
src/store/run/table_cache.rs:30  pub fn new(max_open_files: usize) -> Result<Self, Error>
src/store/run/table_cache.rs:35  pub fn pin(&self, fs: &dyn Fs, path: &Path) -> Result<Arc<dyn File>, Error>
src/store/run/table_cache.rs:85  pub fn resident(&self) -> Result<usize, Error> { Ok(self.lock()?.files.len()) }
src/store/run/table_cache.rs:86  pub fn capacity(&self) -> usize { self.capacity }
src/store/run/table_cache.rs:89  pub fn evict_unpinned(&self) -> Result<usize, Error>
src/sync/epoch.rs:8  pub struct EpochValue
src/sync/epoch.rs:13  pub struct Epoch(AtomicU64)
src/sync/epoch.rs:16  pub const fn new(state: u8, generation: u64) -> Option<Self>
src/sync/epoch.rs:24  pub fn load(&self) -> EpochValue
src/sync/epoch.rs:30  pub fn transition(&self, expected: EpochValue, next_state: u8) -> Result<EpochValue, EpochValue>
src/sync/mod.rs:2  pub mod epoch
src/sync/mod.rs:3  pub mod ranked
src/sync/ranked.rs:12  pub struct Ranked<T, const RANK: u8>
src/sync/ranked.rs:17  pub const fn new(value: T) -> Self
src/sync/ranked.rs:21  pub fn lock(&self) -> Result<RankedGuard<'_, T, RANK>, Error>
src/sync/ranked.rs:34  pub fn into_inner(self) -> Result<T, Error>
src/sync/ranked.rs:39  pub struct RankedGuard<'a, T, const RANK: u8>
src/sync/ranked.rs:60  pub struct RankStack
src/sync/ranked.rs:63  pub fn current() -> Vec<u8>
```

## Innen: crate-interne Grenzen (`pub(crate)`, `pub(super)`, `pub(in …)`)

```rust
src/catalog/record.rs:9  pub(super) const MAX_RECORD_BYTES: usize = 1 << 20
src/catalog/state.rs:57  pub(crate) fn apply_record(&mut self, record: &CatalogRecord) -> Result<(), Error>
src/store/run/table_cache.rs:11  pub(crate) fn run_io() -> IoCtx { IoCtx { role: FileRole::Run, purpose: "run_table_read" } }
```
