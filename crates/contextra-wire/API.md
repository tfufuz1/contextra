# Öffentliche API: `contextra-wire`

Vollständige Übersicht aller öffentlichen Typen, Traits, Structs und Funktionen in `contextra-wire`.

## `src/adapter.rs`

| Signatur | Beschreibung |
|---|---|
| `pub struct Unverified` |  |
| `pub struct Verified<T> ` |  |
| `pub trait VerifiableRoot ` | Helper trait providing higher-ranked follow and verification for root FlatBuffers tables. |
| `pub struct WireBuffer<State = Unverified> ` |  |
| `pub fn new(data: Bytes) -> Self ` |  |
| `pub fn verify<T>(` | Verifiziert den Puffer gegen das angegebene FlatBuffers-Root-Schema `T` und die Schutzgrenzen `config`. |
| `pub fn root<'a>(` |  |
| `pub fn root_with_config<'a>(` |  |
| `pub fn as_slice(&self) -> &[u8] ` |  |
| `pub fn into_inner(self) -> Bytes ` |  |
| `pub fn len(&self) -> usize ` |  |
| `pub fn is_empty(&self) -> bool ` |  |

## `src/contextra_generated.rs`

| Signatur | Beschreibung |
|---|---|
| `pub mod contextra ` |  |
| `pub mod ipc ` |  |
| `pub enum EmbeddingOffset {}` | Öffentliches FlatBuffers-Item `EmbeddingOffset` für IPC-Nachrichten. |
| `pub struct Embedding<'a> ` | Öffentliches FlatBuffers-Item `Embedding` für IPC-Nachrichten. |
| `pub _tab: flatbuffers::Table<'a>,` |  |
| `pub const VT_DATA: flatbuffers::VOffsetT = 4` | Öffentliches FlatBuffers-Item `VT_DATA` für IPC-Nachrichten. |
| `pub const VT_QUANTIZED_U8: flatbuffers::VOffsetT = 6` | Öffentliches FlatBuffers-Item `VT_QUANTIZED_U8` für IPC-Nachrichten. |
| `pub const VT_METRIC: flatbuffers::VOffsetT = 8` | Öffentliches FlatBuffers-Item `VT_METRIC` für IPC-Nachrichten. |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self ` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. |
| `pub fn data(&self) -> Option<flatbuffers::Vector<'a, f32>> ` | Öffentliches FlatBuffers-Item `data` für IPC-Nachrichten. |
| `pub fn quantized_u8(&self) -> Option<flatbuffers::Vector<'a, u8>> ` | Öffentliches FlatBuffers-Item `quantized_u8` für IPC-Nachrichten. |
| `pub fn metric(&self) -> i8 ` | Öffentliches FlatBuffers-Item `metric` für IPC-Nachrichten. |
| `pub struct EmbeddingArgs<'a> ` | Öffentliches FlatBuffers-Item `EmbeddingArgs` für IPC-Nachrichten. |
| `pub data: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, f32>>>,` |  |
| `pub quantized_u8: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, u8>>>,` |  |
| `pub metric: i8,` |  |
| `pub struct EmbeddingBuilder<'a: 'b, 'b> ` | Öffentliches FlatBuffers-Item `EmbeddingBuilder` für IPC-Nachrichten. |
| `pub fn add_data(&mut self, data: flatbuffers::WIPOffset<flatbuffers::Vector<'b, f32>>) ` | Öffentliches FlatBuffers-Item `add_data` für IPC-Nachrichten. |
| `pub fn add_quantized_u8(` | Öffentliches FlatBuffers-Item `add_quantized_u8` für IPC-Nachrichten. |
| `pub fn add_metric(&mut self, metric: i8) ` | Öffentliches FlatBuffers-Item `add_metric` für IPC-Nachrichten. |
| `pub fn new(` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. |
| `pub fn finish(self) -> flatbuffers::WIPOffset<Embedding<'a>> ` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. |
| `pub enum ScoredDocumentOffset {}` | Öffentliches FlatBuffers-Item `ScoredDocumentOffset` für IPC-Nachrichten. |
| `pub struct ScoredDocument<'a> ` | Öffentliches FlatBuffers-Item `ScoredDocument` für IPC-Nachrichten. |
| `pub _tab: flatbuffers::Table<'a>,` |  |
| `pub const VT_ID: flatbuffers::VOffsetT = 4` | Öffentliches FlatBuffers-Item `VT_ID` für IPC-Nachrichten. |
| `pub const VT_SCORE: flatbuffers::VOffsetT = 6` | Öffentliches FlatBuffers-Item `VT_SCORE` für IPC-Nachrichten. |
| `pub const VT_METADATA: flatbuffers::VOffsetT = 8` | Öffentliches FlatBuffers-Item `VT_METADATA` für IPC-Nachrichten. |
| `pub const VT_EMBEDDING: flatbuffers::VOffsetT = 10` | Öffentliches FlatBuffers-Item `VT_EMBEDDING` für IPC-Nachrichten. |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self ` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. |
| `pub fn id(&self) -> Option<&'a str> ` | Öffentliches FlatBuffers-Item `id` für IPC-Nachrichten. |
| `pub fn score(&self) -> f32 ` | Öffentliches FlatBuffers-Item `score` für IPC-Nachrichten. |
| `pub fn metadata(&self) -> Option<&'a str> ` | Öffentliches FlatBuffers-Item `metadata` für IPC-Nachrichten. |
| `pub fn embedding(&self) -> Option<Embedding<'a>> ` | Öffentliches FlatBuffers-Item `embedding` für IPC-Nachrichten. |
| `pub struct ScoredDocumentArgs<'a> ` | Öffentliches FlatBuffers-Item `ScoredDocumentArgs` für IPC-Nachrichten. |
| `pub id: Option<flatbuffers::WIPOffset<&'a str>>,` |  |
| `pub score: f32,` |  |
| `pub metadata: Option<flatbuffers::WIPOffset<&'a str>>,` |  |
| `pub embedding: Option<flatbuffers::WIPOffset<Embedding<'a>>>,` |  |
| `pub struct ScoredDocumentBuilder<'a: 'b, 'b> ` | Öffentliches FlatBuffers-Item `ScoredDocumentBuilder` für IPC-Nachrichten. |
| `pub fn add_id(&mut self, id: flatbuffers::WIPOffset<&'b str>) ` | Öffentliches FlatBuffers-Item `add_id` für IPC-Nachrichten. |
| `pub fn add_score(&mut self, score: f32) ` | Öffentliches FlatBuffers-Item `add_score` für IPC-Nachrichten. |
| `pub fn add_metadata(&mut self, metadata: flatbuffers::WIPOffset<&'b str>) ` | Öffentliches FlatBuffers-Item `add_metadata` für IPC-Nachrichten. |
| `pub fn add_embedding(&mut self, embedding: flatbuffers::WIPOffset<Embedding<'b>>) ` | Öffentliches FlatBuffers-Item `add_embedding` für IPC-Nachrichten. |
| `pub fn new(` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. |
| `pub fn finish(self) -> flatbuffers::WIPOffset<ScoredDocument<'a>> ` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. |
| `pub enum SearchResponseOffset {}` | Öffentliches FlatBuffers-Item `SearchResponseOffset` für IPC-Nachrichten. |
| `pub struct SearchResponse<'a> ` | Öffentliches FlatBuffers-Item `SearchResponse` für IPC-Nachrichten. |
| `pub _tab: flatbuffers::Table<'a>,` |  |
| `pub const VT_RESULTS: flatbuffers::VOffsetT = 4` | Öffentliches FlatBuffers-Item `VT_RESULTS` für IPC-Nachrichten. |
| `pub const VT_TOTAL_HITS: flatbuffers::VOffsetT = 6` | Öffentliches FlatBuffers-Item `VT_TOTAL_HITS` für IPC-Nachrichten. |
| `pub const VT_PROCESSING_TIME_MS: flatbuffers::VOffsetT = 8` | Öffentliches FlatBuffers-Item `VT_PROCESSING_TIME_MS` für IPC-Nachrichten. |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self ` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. |
| `pub fn results(` | Öffentliches FlatBuffers-Item `results` für IPC-Nachrichten. |
| `pub fn total_hits(&self) -> u32 ` | Öffentliches FlatBuffers-Item `total_hits` für IPC-Nachrichten. |
| `pub fn processing_time_ms(&self) -> f32 ` | Öffentliches FlatBuffers-Item `processing_time_ms` für IPC-Nachrichten. |
| `pub struct SearchResponseArgs<'a> ` | Öffentliches FlatBuffers-Item `SearchResponseArgs` für IPC-Nachrichten. |
| `pub results: Option<` |  |
| `pub total_hits: u32,` |  |
| `pub processing_time_ms: f32,` |  |
| `pub struct SearchResponseBuilder<'a: 'b, 'b> ` | Öffentliches FlatBuffers-Item `SearchResponseBuilder` für IPC-Nachrichten. |
| `pub fn add_results(` | Öffentliches FlatBuffers-Item `add_results` für IPC-Nachrichten. |
| `pub fn add_total_hits(&mut self, total_hits: u32) ` | Öffentliches FlatBuffers-Item `add_total_hits` für IPC-Nachrichten. |
| `pub fn add_processing_time_ms(&mut self, processing_time_ms: f32) ` | Öffentliches FlatBuffers-Item `add_processing_time_ms` für IPC-Nachrichten. |
| `pub fn new(` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. |
| `pub fn finish(self) -> flatbuffers::WIPOffset<SearchResponse<'a>> ` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. |
| `pub enum VectorIndexUpdateOffset {}` | Öffentliches FlatBuffers-Item `VectorIndexUpdateOffset` für IPC-Nachrichten. |
| `pub struct VectorIndexUpdate<'a> ` | Öffentliches FlatBuffers-Item `VectorIndexUpdate` für IPC-Nachrichten. |
| `pub _tab: flatbuffers::Table<'a>,` |  |
| `pub const VT_ID: flatbuffers::VOffsetT = 4` | Öffentliches FlatBuffers-Item `VT_ID` für IPC-Nachrichten. |
| `pub const VT_EMBEDDING: flatbuffers::VOffsetT = 6` | Öffentliches FlatBuffers-Item `VT_EMBEDDING` für IPC-Nachrichten. |
| `pub const VT_METADATA: flatbuffers::VOffsetT = 8` | Öffentliches FlatBuffers-Item `VT_METADATA` für IPC-Nachrichten. |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self ` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. |
| `pub fn id(&self) -> Option<&'a str> ` | Öffentliches FlatBuffers-Item `id` für IPC-Nachrichten. |
| `pub fn embedding(&self) -> Option<Embedding<'a>> ` | Öffentliches FlatBuffers-Item `embedding` für IPC-Nachrichten. |
| `pub fn metadata(&self) -> Option<&'a str> ` | Öffentliches FlatBuffers-Item `metadata` für IPC-Nachrichten. |
| `pub struct VectorIndexUpdateArgs<'a> ` | Öffentliches FlatBuffers-Item `VectorIndexUpdateArgs` für IPC-Nachrichten. |
| `pub id: Option<flatbuffers::WIPOffset<&'a str>>,` |  |
| `pub embedding: Option<flatbuffers::WIPOffset<Embedding<'a>>>,` |  |
| `pub metadata: Option<flatbuffers::WIPOffset<&'a str>>,` |  |
| `pub struct VectorIndexUpdateBuilder<'a: 'b, 'b> ` | Öffentliches FlatBuffers-Item `VectorIndexUpdateBuilder` für IPC-Nachrichten. |
| `pub fn add_id(&mut self, id: flatbuffers::WIPOffset<&'b str>) ` | Öffentliches FlatBuffers-Item `add_id` für IPC-Nachrichten. |
| `pub fn add_embedding(&mut self, embedding: flatbuffers::WIPOffset<Embedding<'b>>) ` | Öffentliches FlatBuffers-Item `add_embedding` für IPC-Nachrichten. |
| `pub fn add_metadata(&mut self, metadata: flatbuffers::WIPOffset<&'b str>) ` | Öffentliches FlatBuffers-Item `add_metadata` für IPC-Nachrichten. |
| `pub fn new(` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. |
| `pub fn finish(self) -> flatbuffers::WIPOffset<VectorIndexUpdate<'a>> ` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. |
| `pub enum RoleIdOffset {}` | Öffentliches FlatBuffers-Item `RoleIdOffset` für IPC-Nachrichten. |
| `pub struct RoleId<'a> ` | Öffentliches FlatBuffers-Item `RoleId` für IPC-Nachrichten. |
| `pub _tab: flatbuffers::Table<'a>,` |  |
| `pub const VT_ID: flatbuffers::VOffsetT = 4` | Öffentliches FlatBuffers-Item `VT_ID` für IPC-Nachrichten. |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self ` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. |
| `pub fn id(&self) -> u32 ` | Öffentliches FlatBuffers-Item `id` für IPC-Nachrichten. |
| `pub struct RoleIdArgs ` | Öffentliches FlatBuffers-Item `RoleIdArgs` für IPC-Nachrichten. |
| `pub id: u32,` |  |
| `pub struct RoleIdBuilder<'a: 'b, 'b> ` | Öffentliches FlatBuffers-Item `RoleIdBuilder` für IPC-Nachrichten. |
| `pub fn add_id(&mut self, id: u32) ` | Öffentliches FlatBuffers-Item `add_id` für IPC-Nachrichten. |
| `pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> RoleIdBuilder<'a, 'b> ` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. |
| `pub fn finish(self) -> flatbuffers::WIPOffset<RoleId<'a>> ` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. |
| `pub enum HyperEdgeIdOffset {}` | Öffentliches FlatBuffers-Item `HyperEdgeIdOffset` für IPC-Nachrichten. |
| `pub struct HyperEdgeId<'a> ` | Öffentliches FlatBuffers-Item `HyperEdgeId` für IPC-Nachrichten. |
| `pub _tab: flatbuffers::Table<'a>,` |  |
| `pub const VT_ID: flatbuffers::VOffsetT = 4` | Öffentliches FlatBuffers-Item `VT_ID` für IPC-Nachrichten. |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self ` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. |
| `pub fn id(&self) -> u64 ` | Öffentliches FlatBuffers-Item `id` für IPC-Nachrichten. |
| `pub struct HyperEdgeIdArgs ` | Öffentliches FlatBuffers-Item `HyperEdgeIdArgs` für IPC-Nachrichten. |
| `pub id: u64,` |  |
| `pub struct HyperEdgeIdBuilder<'a: 'b, 'b> ` | Öffentliches FlatBuffers-Item `HyperEdgeIdBuilder` für IPC-Nachrichten. |
| `pub fn add_id(&mut self, id: u64) ` | Öffentliches FlatBuffers-Item `add_id` für IPC-Nachrichten. |
| `pub fn new(` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. |
| `pub fn finish(self) -> flatbuffers::WIPOffset<HyperEdgeId<'a>> ` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. |
| `pub enum RoleBindingOffset {}` | Öffentliches FlatBuffers-Item `RoleBindingOffset` für IPC-Nachrichten. |
| `pub struct RoleBinding<'a> ` | Öffentliches FlatBuffers-Item `RoleBinding` für IPC-Nachrichten. |
| `pub _tab: flatbuffers::Table<'a>,` |  |
| `pub const VT_ROLE: flatbuffers::VOffsetT = 4` | Öffentliches FlatBuffers-Item `VT_ROLE` für IPC-Nachrichten. |
| `pub const VT_ENTITY: flatbuffers::VOffsetT = 6` | Öffentliches FlatBuffers-Item `VT_ENTITY` für IPC-Nachrichten. |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self ` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. |
| `pub fn role(&self) -> u32 ` | Öffentliches FlatBuffers-Item `role` für IPC-Nachrichten. |
| `pub fn entity(&self) -> u64 ` | Öffentliches FlatBuffers-Item `entity` für IPC-Nachrichten. |
| `pub struct RoleBindingArgs ` | Öffentliches FlatBuffers-Item `RoleBindingArgs` für IPC-Nachrichten. |
| `pub role: u32,` |  |
| `pub entity: u64,` |  |
| `pub struct RoleBindingBuilder<'a: 'b, 'b> ` | Öffentliches FlatBuffers-Item `RoleBindingBuilder` für IPC-Nachrichten. |
| `pub fn add_role(&mut self, role: u32) ` | Öffentliches FlatBuffers-Item `add_role` für IPC-Nachrichten. |
| `pub fn add_entity(&mut self, entity: u64) ` | Öffentliches FlatBuffers-Item `add_entity` für IPC-Nachrichten. |
| `pub fn new(` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. |
| `pub fn finish(self) -> flatbuffers::WIPOffset<RoleBinding<'a>> ` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. |
| `pub enum HyperEdgeOffset {}` | Öffentliches FlatBuffers-Item `HyperEdgeOffset` für IPC-Nachrichten. |
| `pub struct HyperEdge<'a> ` | Öffentliches FlatBuffers-Item `HyperEdge` für IPC-Nachrichten. |
| `pub _tab: flatbuffers::Table<'a>,` |  |
| `pub const VT_ID: flatbuffers::VOffsetT = 4` | Öffentliches FlatBuffers-Item `VT_ID` für IPC-Nachrichten. |
| `pub const VT_PREDICATE: flatbuffers::VOffsetT = 6` | Öffentliches FlatBuffers-Item `VT_PREDICATE` für IPC-Nachrichten. |
| `pub const VT_PARTICIPANTS: flatbuffers::VOffsetT = 8` | Öffentliches FlatBuffers-Item `VT_PARTICIPANTS` für IPC-Nachrichten. |
| `pub const VT_WEIGHT: flatbuffers::VOffsetT = 10` | Öffentliches FlatBuffers-Item `VT_WEIGHT` für IPC-Nachrichten. |
| `pub const VT_TX_VALID_FROM: flatbuffers::VOffsetT = 12` | Öffentliches FlatBuffers-Item `VT_TX_VALID_FROM` für IPC-Nachrichten. |
| `pub const VT_TX_VALID_TO: flatbuffers::VOffsetT = 14` | Öffentliches FlatBuffers-Item `VT_TX_VALID_TO` für IPC-Nachrichten. |
| `pub const VT_BUSINESS_VALID_FROM: flatbuffers::VOffsetT = 16` | Öffentliches FlatBuffers-Item `VT_BUSINESS_VALID_FROM` für IPC-Nachrichten. |
| `pub const VT_BUSINESS_VALID_TO: flatbuffers::VOffsetT = 18` | Öffentliches FlatBuffers-Item `VT_BUSINESS_VALID_TO` für IPC-Nachrichten. |
| `pub const VT_SOURCE_DOC_ID: flatbuffers::VOffsetT = 20` | Öffentliches FlatBuffers-Item `VT_SOURCE_DOC_ID` für IPC-Nachrichten. |
| `pub const VT_CHILD_EDGE_IDS: flatbuffers::VOffsetT = 22` | Öffentliches FlatBuffers-Item `VT_CHILD_EDGE_IDS` für IPC-Nachrichten. |
| `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self ` | Öffentliches FlatBuffers-Item `init_from_table` für IPC-Nachrichten. |
| `pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(` | Öffentliches FlatBuffers-Item `create` für IPC-Nachrichten. |
| `pub fn id(&self) -> u64 ` | Öffentliches FlatBuffers-Item `id` für IPC-Nachrichten. |
| `pub fn predicate(&self) -> u8 ` | Öffentliches FlatBuffers-Item `predicate` für IPC-Nachrichten. |
| `pub fn participants(` | Öffentliches FlatBuffers-Item `participants` für IPC-Nachrichten. |
| `pub fn weight(&self) -> f32 ` | Öffentliches FlatBuffers-Item `weight` für IPC-Nachrichten. |
| `pub fn tx_valid_from(&self) -> u64 ` | Öffentliches FlatBuffers-Item `tx_valid_from` für IPC-Nachrichten. |
| `pub fn tx_valid_to(&self) -> u64 ` | Öffentliches FlatBuffers-Item `tx_valid_to` für IPC-Nachrichten. |
| `pub fn business_valid_from(&self) -> i64 ` | Öffentliches FlatBuffers-Item `business_valid_from` für IPC-Nachrichten. |
| `pub fn business_valid_to(&self) -> i64 ` | Öffentliches FlatBuffers-Item `business_valid_to` für IPC-Nachrichten. |
| `pub fn source_doc_id(&self) -> u64 ` | Öffentliches FlatBuffers-Item `source_doc_id` für IPC-Nachrichten. |
| `pub fn child_edge_ids(&self) -> Option<flatbuffers::Vector<'a, u64>> ` | Öffentliches FlatBuffers-Item `child_edge_ids` für IPC-Nachrichten. |
| `pub struct HyperEdgeArgs<'a> ` | Öffentliches FlatBuffers-Item `HyperEdgeArgs` für IPC-Nachrichten. |
| `pub id: u64,` |  |
| `pub predicate: u8,` |  |
| `pub participants: Option<` |  |
| `pub weight: f32,` |  |
| `pub tx_valid_from: u64,` |  |
| `pub tx_valid_to: u64,` |  |
| `pub business_valid_from: i64,` |  |
| `pub business_valid_to: i64,` |  |
| `pub source_doc_id: u64,` |  |
| `pub child_edge_ids: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, u64>>>,` |  |
| `pub struct HyperEdgeBuilder<'a: 'b, 'b> ` | Öffentliches FlatBuffers-Item `HyperEdgeBuilder` für IPC-Nachrichten. |
| `pub fn add_id(&mut self, id: u64) ` | Öffentliches FlatBuffers-Item `add_id` für IPC-Nachrichten. |
| `pub fn add_predicate(&mut self, predicate: u8) ` | Öffentliches FlatBuffers-Item `add_predicate` für IPC-Nachrichten. |
| `pub fn add_participants(` | Öffentliches FlatBuffers-Item `add_participants` für IPC-Nachrichten. |
| `pub fn add_weight(&mut self, weight: f32) ` | Öffentliches FlatBuffers-Item `add_weight` für IPC-Nachrichten. |
| `pub fn add_tx_valid_from(&mut self, tx_valid_from: u64) ` | Öffentliches FlatBuffers-Item `add_tx_valid_from` für IPC-Nachrichten. |
| `pub fn add_tx_valid_to(&mut self, tx_valid_to: u64) ` | Öffentliches FlatBuffers-Item `add_tx_valid_to` für IPC-Nachrichten. |
| `pub fn add_business_valid_from(&mut self, business_valid_from: i64) ` | Öffentliches FlatBuffers-Item `add_business_valid_from` für IPC-Nachrichten. |
| `pub fn add_business_valid_to(&mut self, business_valid_to: i64) ` | Öffentliches FlatBuffers-Item `add_business_valid_to` für IPC-Nachrichten. |
| `pub fn add_source_doc_id(&mut self, source_doc_id: u64) ` | Öffentliches FlatBuffers-Item `add_source_doc_id` für IPC-Nachrichten. |
| `pub fn add_child_edge_ids(` | Öffentliches FlatBuffers-Item `add_child_edge_ids` für IPC-Nachrichten. |
| `pub fn new(` | Öffentliches FlatBuffers-Item `new` für IPC-Nachrichten. |
| `pub fn finish(self) -> flatbuffers::WIPOffset<HyperEdge<'a>> ` | Öffentliches FlatBuffers-Item `finish` für IPC-Nachrichten. |
| `pub fn root_as_search_response(` | Öffentliches FlatBuffers-Item `root_as_search_response` für IPC-Nachrichten. |
| `pub fn size_prefixed_root_as_search_response(` | Öffentliches FlatBuffers-Item `size_prefixed_root_as_search_response` für IPC-Nachrichten. |
| `pub fn root_as_search_response_with_opts<'b, 'o>(` | Öffentliches FlatBuffers-Item `root_as_search_response_with_opts` für IPC-Nachrichten. |
| `pub fn size_prefixed_root_as_search_response_with_opts<'b, 'o>(` | Öffentliches FlatBuffers-Item `size_prefixed_root_as_search_response_with_opts` für IPC-Nachrichten. |
| `pub unsafe fn root_as_search_response_unchecked(buf: &[u8]) -> SearchResponse ` | Öffentliches FlatBuffers-Item `root_as_search_response_unchecked` für IPC-Nachrichten. |
| `pub unsafe fn size_prefixed_root_as_search_response_unchecked(` | Öffentliches FlatBuffers-Item `size_prefixed_root_as_search_response_unchecked` für IPC-Nachrichten. |
| `pub fn finish_search_response_buffer<'a, 'b>(` | Öffentliches FlatBuffers-Item `finish_search_response_buffer` für IPC-Nachrichten. |
| `pub fn finish_size_prefixed_search_response_buffer<'a, 'b>(` | Öffentliches FlatBuffers-Item `finish_size_prefixed_search_response_buffer` für IPC-Nachrichten. |

## `src/generated.rs`

| Signatur | Beschreibung |
|---|---|
| `pub mod contextra_generated` |  |

## `src/jsonrpc.rs`

| Signatur | Beschreibung |
|---|---|
| `pub struct JsonRpcConfig ` |  |
| `pub max_message_size: usize,` | Maximum allowed payload size in bytes (default: 10 MiB). |
| `pub max_depth: usize,` | Maximum allowed JSON structural nesting depth (default: 64). |
| `pub max_batch_size: usize,` | Maximum number of requests allowed in a batch payload (default: 100). |
| `pub reject_duplicate_keys: bool,` | Reject JSON objects containing duplicate key names (default: true). |
| `pub enum JsonRpcPayload ` |  |
| `pub struct JsonRpcRequest ` |  |
| `pub jsonrpc: String,` | Always "2.0". |
| `pub id: Option<Value>,` | `None` for notifications (no response expected). |
| `pub method: String,` | Name of the RPC method to invoke. |
| `pub params: Value,` |  |
| `pub fn validate(&self) -> Result<(), JsonRpcError> ` | Validates that the request complies with the JSON-RPC 2.0 specification (fail-closed). |
| `pub struct JsonRpcResponse ` |  |
| `pub jsonrpc: String,` | Always "2.0". |
| `pub id: Option<Value>,` | Request ID (`None` for parse/batch errors). |
| `pub result: Option<Value>,` |  |
| `pub error: Option<JsonRpcError>,` |  |
| `pub struct JsonRpcError ` |  |
| `pub code: i32,` | Numeric error code per JSON-RPC 2.0 specification. |
| `pub message: String,` | Brief error message. |
| `pub data: Option<Value>,` |  |
| `pub fn parse_error(message: impl Into<String>) -> Self ` | Parse error (-32700): Invalid JSON received by server. |
| `pub fn invalid_request(message: impl Into<String>) -> Self ` | Invalid Request (-32600): JSON sent is not a valid Request object. |
| `pub fn method_not_found(message: impl Into<String>) -> Self ` | Method not found (-32601): The method does not exist / is not available. |
| `pub fn invalid_params(message: impl Into<String>) -> Self ` | Invalid params (-32602): Invalid method parameter(s). |
| `pub fn internal_error(message: impl Into<String>) -> Self ` | Internal error (-32603): Internal JSON-RPC error. |
| `pub fn ok(id: Option<Value>, result: Value) -> Self ` | Constructs a successful JSON-RPC response. |
| `pub fn err(id: Option<Value>, code: i32, message: impl Into<String>) -> Self ` | Constructs an error JSON-RPC response. |
| `pub fn parse_jsonrpc_payload(raw: &[u8]) -> Result<JsonRpcPayload, JsonRpcError> ` | Parses raw bytes into a `JsonRpcPayload` using default configuration limits. |
| `pub fn parse_jsonrpc_payload_with_config(` | Parses raw bytes into a `JsonRpcPayload` with strict security and size validation. |

## `src/lib.rs`

| Signatur | Beschreibung |
|---|---|
| `pub mod generated` |  |
| `pub mod adapter` |  |
| `pub mod jsonrpc` |  |
| `pub mod verification` |  |
| `pub use adapter::*` |  |
| `pub use generated::contextra_generated` |  |
| `pub use generated::contextra_generated::contextra::ipc::*` |  |
| `pub use jsonrpc::*` |  |
| `pub use verification::*` |  |
| `pub enum Error ` |  |
| `pub type Result<T> = core::result::Result<T, Error>` | Standard-Result-Typ für `contextra-wire`. |

## `src/verification.rs`

| Signatur | Beschreibung |
|---|---|
| `pub enum FlatBufferError ` |  |
| `pub struct FlatBufferConfig ` |  |
| `pub max_message_size: usize,` | Maximum allowed buffer size in bytes (default: 10 MiB). |
| `pub max_depth: usize,` | Maximum depth of nested FlatBuffers tables (default: 64). |
| `pub max_tables: usize,` | Maximum number of tables evaluated during verification (default: 1,000,000). |
| `pub fn verify_flatbuffer<'a, T>(` | Generic verified root accessor for any FlatBuffers root table `T`. |
| `pub fn verify_and_parse_search_response<'a>(` | Verifies and parses a `SearchResponse` root table from bytes. |
| `pub fn verify_and_parse_embedding<'a>(` | Verifies and parses an `Embedding` root table from bytes. |
| `pub fn verify_and_parse_scored_document<'a>(` | Verifies and parses a `ScoredDocument` root table from bytes. |
| `pub fn verify_and_parse_vector_index_update<'a>(` | Verifies and parses a `VectorIndexUpdate` root table from bytes. |
| `pub fn verify_and_parse_role_id<'a>(` | Verifies and parses a `RoleId` root table from bytes. |
| `pub fn verify_and_parse_hyper_edge_id<'a>(` | Verifies and parses a `HyperEdgeId` root table from bytes. |
| `pub fn verify_and_parse_role_binding<'a>(` | Verifies and parses a `RoleBinding` root table from bytes. |
| `pub fn verify_and_parse_hyper_edge<'a>(` | Verifies and parses a `HyperEdge` root table from bytes. |
