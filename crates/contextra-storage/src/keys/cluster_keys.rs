//! # Clusterschlüssel: Port-Form nach SPEC §4 und lokaler Adapter
//!
//! SPEC §16.2, §16.6, D-19: Clusterschlüssel sind zufällig, verwahrt vom `KeyProvider`
//! und nie im Catalog, in `KEYS` oder in einer Datendatei (INV-KEY-1).
//!
//! Der Port `contextra_ports::persistence::KeyProvider` hat heute noch die alte,
//! domänenbasierte Form (P1 / WP-1.3 stellt sie auf `ClusterId` um). Dieses Modul
//! hält deshalb die Zielform aus §4 lokal fest und verbindet sie über
//! [`PortKeys`] mit dem bestehenden Port. Die Clustermethoden, die der Port nicht
//! ausdrücken kann, liefern `ClusterKeyError::Unsupported`. Das ist ehrlich, statt
//! Schlüssel zu erfinden.
//!
//! **SA-12 (Vorschlag, nicht im Port):** `ClusterKeyError` entspricht dem Vorschlag,
//! `KeyError` in §4 um `DestroyFailed` und `Unsupported` zu ergänzen, ohne
//! Inhaltsfelder. Sobald P1 `KeyError` im Port führt, wird dieser Typ dorthin
//! verschoben und hier ersetzt.

use contextra_ports::persistence::{Entropy, KeyProvider, Secret32};
use std::fmt;
use std::io;

/// Kennung eines Clusters (16 Byte, IMPL §2.2.1). Kein Inhalt, nur Identität.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ClusterId(pub [u8; 16]);

/// Vorhandensein eines Clusterschlüssels (SPEC §4 `KeyState`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    /// Schlüssel ist verwahrt und benutzbar.
    Present,
    /// Schlüssel ist vernichtet (dauerhaft und geprüft).
    Absent,
}

/// Fehler des Clusterschlüssel-Ports.
///
/// Ohne Inhaltsfelder, damit keine Schlüsselbytes in Fehlern landen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClusterKeyError {
    /// Schlüssel war nie angelegt.
    NotFound,
    /// Schlüsselquelle nicht erreichbar.
    Unavailable,
    /// Vernichtung ließ sich nicht dauerhaft oder nicht prüfbar abschließen (SA-12).
    DestroyFailed,
    /// Der Port kann diese Operation nicht ausführen (SA-12).
    Unsupported,
}

impl fmt::Display for ClusterKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "Clusterschlüssel nicht angelegt"),
            Self::Unavailable => write!(f, "Schlüsselquelle nicht erreichbar"),
            Self::DestroyFailed => write!(f, "Vernichtung des Clusterschlüssels fehlgeschlagen"),
            Self::Unsupported => write!(f, "Operation vom Schlüsselport nicht unterstützt"),
        }
    }
}

impl std::error::Error for ClusterKeyError {}

/// Zielform des Schlüsselports für Clusterschlüssel (SPEC §4, §16.6).
pub trait ClusterKeys: Send + Sync {
    /// Schlüsselverschlüsselungsschlüssel. Umhüllt nur `KEYS`.
    fn kek(&self) -> Result<Secret32, ClusterKeyError>;

    /// Liefert den Clusterschlüssel einer Epoche. `None` = vernichtet.
    fn cluster_key(&self, c: ClusterId, epoch: u32) -> Result<Option<Secret32>, ClusterKeyError>;

    /// Legt einen Clusterschlüssel an und liefert seine Epoche. Dauerhaft bei Rückkehr.
    fn create_cluster_key(&self, c: ClusterId, e: &dyn Entropy) -> Result<u32, ClusterKeyError>;

    /// Vernichtet alle Epochen des Clusters. Dauerhaft bei Rückkehr, idempotent (D-07).
    fn destroy(&self, c: ClusterId) -> Result<(), ClusterKeyError>;

    /// Prüft, ob Schlüssel des Clusters noch vorhanden sind.
    fn probe(&self, c: ClusterId) -> Result<KeyState, ClusterKeyError>;

    /// Barriere für `create_cluster_key` (INV-KEYP-1).
    fn make_durable(&self) -> Result<(), ClusterKeyError>;
}

/// Adapter: führt die Zielform über den bestehenden, domänenbasierten Port aus.
///
/// Umsetzbar sind `kek` und `make_durable`. Alles, was der Port nicht ausdrücken kann
/// (Clusterschlüssel anlegen, lesen, vernichten, prüfen), liefert `Unsupported`.
pub struct PortKeys<'a> {
    port: &'a dyn KeyProvider,
}

impl<'a> PortKeys<'a> {
    /// Umhüllt einen bestehenden `KeyProvider`.
    pub fn new(port: &'a dyn KeyProvider) -> Self {
        Self { port }
    }
}

/// Jede E/A-Störung des Ports wird als `Unavailable` gemeldet, ohne Details.
fn map_io(_e: io::Error) -> ClusterKeyError {
    ClusterKeyError::Unavailable
}

impl ClusterKeys for PortKeys<'_> {
    fn kek(&self) -> Result<Secret32, ClusterKeyError> {
        self.port.kek().map_err(map_io)
    }

    fn cluster_key(&self, _c: ClusterId, _epoch: u32) -> Result<Option<Secret32>, ClusterKeyError> {
        Err(ClusterKeyError::Unsupported)
    }

    fn create_cluster_key(&self, _c: ClusterId, _e: &dyn Entropy) -> Result<u32, ClusterKeyError> {
        Err(ClusterKeyError::Unsupported)
    }

    fn destroy(&self, _c: ClusterId) -> Result<(), ClusterKeyError> {
        Err(ClusterKeyError::Unsupported)
    }

    fn probe(&self, _c: ClusterId) -> Result<KeyState, ClusterKeyError> {
        Err(ClusterKeyError::Unsupported)
    }

    fn make_durable(&self) -> Result<(), ClusterKeyError> {
        self.port.make_durable().map_err(map_io)
    }
}
