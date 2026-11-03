//! Reciprocal Rank Fusion für die Hybridsuche.
//!
//! # Contextra Rank (`contextra-rank`)
//!
//! **Ring:** Ring 2 (Ranking & Fusion, SPEC §8.11, D-26)
//! **Klasse:** Rand `[R]`
//! **Reifegrad:** Aktiv (Kern v1)
//!
//! ## Zweck des Crates
//! Dieses Crate implementiert die deterministische Reciprocal Rank Fusion (RRF)
//! für die Hybridsuche im Contextra-System:
//! - Kanonische Schnittstelle: [`rrf_fuse`]
//! - Fester Glättungsfaktor: $k = 60$ ([`K_RRF`])
//! - Reine Festkomma-Arithmetik ($\lfloor 2^{40} / (60 + \text{Rang}) \rfloor$), keine Gleitkommazahlen
//! - Deterministischer Tie-Break (primär Score absteigend, sekundär `doc_tag` aufsteigend, tertiär `cluster_tag` aufsteigend)
//! - Bit-genaue Permutationsinvarianz bezüglich der Listenreihenfolge
//! - Schrankenberechnung für ungesehene Dokumente ([`unseen_bound`])
//! - Keine Hintergrund-Threads, kein `unsafe` (`#![forbid(unsafe_code)]`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod bound;
pub mod error;
pub mod rrf;

pub use bound::unseen_bound;
pub use error::RankError;
pub use rrf::{rrf_fuse, Candidate, DocHandle, Fused, RrfScore, K_RRF};
