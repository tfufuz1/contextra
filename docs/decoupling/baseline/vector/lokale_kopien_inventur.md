# Lokale Kopien und Typ-Inventur in contextra-vector

| Konzept | Lokaler Typ / Definition | Datei | Referenz im Workspace |
|---|---|---|---|
| DocId | pub struct DocId(pub u64) bzw u128 | src/types.rs:13 | contextra_types::DocId / contextra_ids::DocId |
| Clock | pub trait Clock: Send + Sync | src/types.rs:249, src/contract.rs:43, src/hnsw/clock.rs:10 | contextra_ports::Clock / contextra_env::Clock |
| Rng | pub trait Rng: Send | src/types.rs:255, src/contract.rs:49 | contextra_ports::Rng / contextra_env::Rng |
| ExternalId | pub struct ExternalId(pub u128) | src/contract.rs:55 | contextra_ids::DocId |
| DistanceMetric | pub enum DistanceMetric { Cosine, Euclidean, DotProduct } | src/types.rs:114 | contextra_types::DistanceMetric, contextra_simd::DistanceMetric |
| Metric | pub enum Metric { Cosine, Euclidean, DotProduct } | src/contract.rs:65 | DistanceMetric |
| Error | pub enum Error | src/contract.rs:9 | src/error.rs::VectorError, contextra_types::ContextraError |
| VectorError | pub enum VectorError | src/error.rs:10 | Lokaler Hauptfehlertyp |
| DistanceFn | pub trait DistanceFn: Send + Sync | src/contract.rs:85 | Distanzinjektion |
| Skalare Distanzen | cosine_distance_scalar, euclidean_distance_scalar, dot_product_scalar | src/distance.rs | contextra_simd::kernels::scalar |
| Quantisierte Distanzen | euclidean_distance_sq_u8, cosine_similarity_parts_u8 | src/distance.rs, src/quantize/ | contextra_simd u8 Distanzen |
