//! Test-Fixturen für das Contextra-Lab (Dokumentmuster, gesicherte DEKs, Deletion-Callbacks).
//!
//! AR-4: Keine Produktionsabhängigkeiten zu store, engine oder erasure.

use std::collections::HashMap;

/// Repräsentiert ein Dokumentmuster mit bekannter Nadel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentPattern {
    /// Dokument-ID oder Tag.
    pub doc_id: [u8; 16],
    /// Bekannte Nadel (Suchmuster).
    pub needle: Vec<u8>,
    /// DEK für das Dokument (zur Aufbewahrung für MK3-Prüfung).
    pub dek: [u8; 32],
}

/// Test-Store Fixture für Differenz- und Löschkettenprüfungen.
#[derive(Debug, Clone, Default)]
pub struct StoreFixture {
    /// Gespeicherte Dokumente nach ID.
    pub documents: HashMap<[u8; 16], DocumentPattern>,
    /// Gelöschte Dokumente.
    pub deleted: HashMap<[u8; 16], DocumentPattern>,
}

impl StoreFixture {
    /// Erstellt eine neue leere Fixture.
    pub fn new() -> Self {
        Self::default()
    }

    /// Fügt ein Dokument mit bekannter Nadel und DEK hinzu.
    pub fn insert(&mut self, doc_id: [u8; 16], needle: Vec<u8>, dek: [u8; 32]) {
        self.documents.insert(
            doc_id,
            DocumentPattern {
                doc_id,
                needle,
                dek,
            },
        );
    }

    /// Löst eine Löschung über eine vom Aufrufer übergebene Funktion aus (AR-4: keine Produktionsabhängigkeit).
    pub fn execute_deletion<F>(&mut self, target_id: &[u8; 16], mut delete_fn: F) -> bool
    where
        F: FnMut(&[u8; 16], &mut HashMap<[u8; 16], DocumentPattern>) -> bool,
    {
        if let Some(pattern) = self.documents.get(target_id).cloned() {
            let res = delete_fn(target_id, &mut self.documents);
            if res {
                self.deleted.insert(*target_id, pattern);
            }
            res
        } else {
            false
        }
    }
}
