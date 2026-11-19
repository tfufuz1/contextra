//! Canonical lineage structures and reverse index (SPEC §8).

use std::collections::{BTreeMap, BTreeSet};

use contextra_erasure::types_local::LayerId;
use contextra_ids::{DocId, SubjectRef};
use contextra_privacy::DataClass;
use contextra_types::{ContextraError, Result};

/// A non-empty collection. It prevents derived writes without a source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonEmpty<T>(Vec<T>);

impl<T> NonEmpty<T> {
    /// Builds a non-empty collection.
    pub fn new(values: Vec<T>) -> Result<Self> {
        if values.is_empty() {
            return Err(ContextraError::PolicyViolation(
                "derived data requires lineage".into(),
            ));
        }
        Ok(Self(values))
    }

    /// Returns the values in their canonical order.
    pub fn as_slice(&self) -> &[T] { &self.0 }
}

/// A source referenced by an item of derived data.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SourceId {
    /// A source document.
    Document(DocId),
    /// A personal-data unit.
    Subject(SubjectRef),
}

impl Clone for SourceId {
    fn clone(&self) -> Self {
        match self {
            Self::Document(doc) => Self::Document(*doc),
            Self::Subject(subject) => Self::Subject(SubjectRef::from_bytes(*subject.as_bytes())),
        }
    }
}

/// Versioned source reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceRef {
    /// Class of the source record.
    pub source_class: DataClass,
    /// Opaque source identifier.
    pub source_id: SourceId,
    /// Source version observed while deriving the artifact.
    pub source_version: u64,
}

impl PartialOrd for SourceRef {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SourceRef {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        data_class_rank(self.source_class)
            .cmp(&data_class_rank(other.source_class))
            .then_with(|| self.source_id.cmp(&other.source_id))
            .then_with(|| self.source_version.cmp(&other.source_version))
    }
}

fn data_class_rank(class: DataClass) -> u8 {
    match class {
        DataClass::K => 0,
        DataClass::P => 1,
        DataClass::D => 2,
        DataClass::E => 3,
    }
}

/// Mandatory provenance for a D-class write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineageRef {
    /// Canonically sorted and deduplicated source list.
    pub sources: NonEmpty<SourceRef>,
}

impl LineageRef {
    /// Validates an externally supplied canonical source list.
    pub fn try_new(sources: Vec<SourceRef>) -> Result<Self> {
        let sources = NonEmpty::new(sources)?;
        if sources.as_slice().windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(ContextraError::PolicyViolation(
                "lineage sources must be sorted and deduplicated".into(),
            ));
        }
        Ok(Self { sources })
    }

    /// Produces the canonical representation for trusted, local construction.
    pub fn canonical(mut sources: Vec<SourceRef>) -> Result<Self> {
        sources.sort();
        sources.dedup();
        Self::try_new(sources)
    }
}

/// Location of a derived artifact in an E1 layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DerivedRef {
    /// Layer owning the artifact.
    pub layer: LayerId,
    /// Segment inside the layer.
    pub segment_id: u64,
    /// Offset in the segment.
    pub offset: u64,
}

/// Reverse lineage index. Every multi-source artifact is recorded under every source.
#[derive(Debug, Default)]
pub struct LineageIndex { by_source: BTreeMap<SourceRef, BTreeSet<DerivedRef>> }

impl LineageIndex {
    /// Inserts a derived reference under each of its sources.
    pub fn insert(&mut self, lineage: &LineageRef, derived: DerivedRef) {
        for source in lineage.sources.as_slice() {
            self.by_source.entry(source.clone()).or_default().insert(derived);
        }
    }

    /// Returns all artifacts depending on exactly this version of a source.
    pub fn derived_of(&self, source: &SourceRef) -> Vec<DerivedRef> {
        self.by_source.get(source).map_or_else(Vec::new, |refs| refs.iter().copied().collect())
    }

    /// Removes an artifact from every source entry and prunes empty entries.
    pub fn remove(&mut self, derived: DerivedRef) {
        self.by_source.retain(|_, refs| { refs.remove(&derived); !refs.is_empty() });
    }
}
