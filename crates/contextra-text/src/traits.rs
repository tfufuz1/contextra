//! Traits für Index-Oberfläche und Wartung gemäß SPEC §4.3.

use crate::error::Result;

/// Kern-Index-Schnittstelle (Klasse A / SPEC §4.3).
pub trait Index<Id, Query: ?Sized> {
    /// Treffertyp der Suchoperation.
    type Hit;

    /// Fügt ein Dokument mit gegebener ID und Byte-Nutzlast in den Index ein.
    fn insert(&mut self, id: Id, payload: &[u8]) -> Result<()>;

    /// Entfernt ein Dokument anhand seiner ID aus dem Index.
    ///
    /// Gibt `true` zurück, wenn das Dokument vorhanden war und gelöscht wurde.
    fn remove(&mut self, id: &Id) -> Result<bool>;

    /// Führt eine Suche anhand der Query aus und liefert bis zu `limit` Treffer zurück.
    fn search(&self, query: &Query, limit: usize) -> Result<Vec<Self::Hit>>;
}

/// Neutrale Wartungsschnittstelle (Klasse A / SPEC §4.3).
pub trait Maintain {
    /// Statistiktyp der Kompaktierungsoperation.
    type Stats;

    /// Führt eine Kompaktierung durch und gibt Segmente bzw. gelöschte Einträge tatsächlich frei.
    fn compact(&mut self) -> Result<Self::Stats>;

    /// Bereinigt fragmentierte interne Strukturen.
    fn vacuum(&mut self) -> Result<()>;
}
