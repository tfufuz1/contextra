//! # Key-Value- und Hyperedge-Link-Operationen (`kv_links`)
//!
//! Dieses Modul erweitert `PyCollection` um direkte Key-Value-Speicheroperationen (KV)
//! sowie um Verlinkungsmechanismen für semantische Zettelkasten-Relationen (A-MEM)
//! und bidirektionale Dokumentenbeziehungen.
//!
//! ## Ring-Zugehörigkeit
//! Ring 4 (FFI-Boundary / Bindings).
//!
//! ## Architektur & Funktionsweise
//! Während reguläre Collection-Methoden Vektorindizes und Volltextindizes aktualisieren,
//! erlauben die hier definierten Methoden:
//! - **Reine KV-Speicherung:** Schneller Durchgriff auf den Key-Value-Store ohne Vektor-Indexierung (`put_kv`, `get_kv`).
//! - **Bidirektionale Relationen:** Atomare Verknüpfung zweier Dokumentenschlüssel mit einem Bezeichnungslabel.
//! - **Semantische Zettelkasten-Links (`link_memories`):** Typisierte Relationen (`Elaborates`, `Contradicts`, `Supersedes`, `References`)
//!   zwischen Dokumenten für assoziative Wissensgraphen.
//!
//! Alle Methoden kapseln asynchrone Rust-Calls via `run_blocking_ffi` und geben während
//! der I/O-Wartezeit den Python GIL (Global Interpreter Lock) frei.

use contextra_types::{DocId, LinkRelation};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::bindings::collection::PyCollection;
use crate::bindings::common::*;

// ANCHOR[contextra-py:LINK-001]
/// Parst einen String ohne Allokation und unabhängig von Groß-/Kleinschreibung in eine `LinkRelation`.
///
/// # Zweck
/// Konvertiert nutzerdefinierte Relationsbezeichnungen aus Python in das typsichere
/// Rust-Enum `LinkRelation` aus `contextra_types`.
///
/// # Unterstützte Relationen:
/// - `"elaborates"`: Vertieft, erklärt oder ergänzt den Zielknoten.
/// - `"contradicts"`: Widerspricht der Aussage des Zielknotens.
/// - `"supersedes"`: Löst den Zielknoten ab (neuere Version / Korrektur).
/// - `"references"`: Zitiert oder verweist auf den Zielknoten.
///
/// # Fehlerfälle
/// Gibt einen `PyValueError` zurück, falls der übergebene String keinem der vier
/// definierten Beziehungstypen entspricht.
pub fn parse_link_relation(s: &str) -> PyResult<LinkRelation> {
    if s.eq_ignore_ascii_case("elaborates") {
        Ok(LinkRelation::Elaborates)
    } else if s.eq_ignore_ascii_case("contradicts") {
        Ok(LinkRelation::Contradicts)
    } else if s.eq_ignore_ascii_case("supersedes") {
        Ok(LinkRelation::Supersedes)
    } else if s.eq_ignore_ascii_case("references") {
        Ok(LinkRelation::References)
    } else {
        Err(PyValueError::new_err(format!(
            "Unknown link relation '{}'. Valid relations are: 'elaborates', 'contradicts', 'supersedes', 'references'",
            s
        )))
    }
}

// NOTE: Wandelt LinkRelation für Python-Rückgaben in einen statischen String um.
fn relation_as_str(r: LinkRelation) -> &'static str {
    match r {
        LinkRelation::Elaborates => "elaborates",
        LinkRelation::Contradicts => "contradicts",
        LinkRelation::Supersedes => "supersedes",
        LinkRelation::References => "references",
    }
}

#[pymethods]
impl PyCollection {
    // ANCHOR[contextra-py:KV-001]
    /// Speichert ein reines Key-Value-Paar direkt im Speicher ab, ohne Vektor- oder Volltextindizes zu verändern.
    ///
    /// # Parameter
    /// - `py`: Das Python GIL-Token.
    /// - `id`: Der Primärschlüssel als String oder Int (wird validiert).
    /// - `value`: Ein Python-Dictionary mit den Nutzdaten (wird in JSON serialisiert).
    ///
    /// # Ablauf
    /// 1. Schlüssel validieren (`validate_id_obj`).
    /// 2. Python-Dictionary in JSON konvertieren (`dict_to_json`).
    /// 3. GIL freigeben und Tokio `block_on` auf `inner.put_kv` ausführen.
    /// 4. Rust-Fehler in Python-Exceptions übersetzen (`contextra_err`).
    ///
    /// # Fehlerfälle
    /// - `PyValueError` bei ungültigem Schlüsseltyp.
    /// - `ContextraIOError` / `ContextraError` bei I/O- oder Speicherfehlern.
    pub fn put_kv<'py>(
        &self,
        py: Python<'py>,
        id: &Bound<'py, PyAny>,
        value: &Bound<'py, PyDict>,
    ) -> PyResult<()> {
        let id_str = validate_id_obj(id)?;
        let json_val = dict_to_json(value)?;
        let rt = &self.runtime;
        run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(self.inner.put_kv(&id_str, &json_val))
                .map_err(contextra_err)
        })
    }

    // ANCHOR[contextra-py:KV-002]
    /// Speichert ein Key-Value-Paar nur dann, wenn der Schlüssel noch nicht im Storage existiert.
    ///
    /// # Zweck
    /// Verhindert versehentliches Überschreiben existierender Einträge (Conditional Put).
    ///
    /// # Ablauf
    /// 1. Schlüssel validieren (`validate_id_obj`).
    /// 2. Wert als JSON vorbereiten.
    /// 3. Asynchron `inner.put_kv_if_absent` unter GIL-Release ausführen.
    ///
    /// # Fehlerfälle
    /// - `PyValueError` bei ungültigem Schlüssel.
    /// - `ContextraIOError` bei I/O-Fehlern.
    pub fn put_kv_if_absent<'py>(
        &self,
        py: Python<'py>,
        id: &Bound<'py, PyAny>,
        value: &Bound<'py, PyDict>,
    ) -> PyResult<()> {
        let id_str = validate_id_obj(id)?;
        let json_val = dict_to_json(value)?;
        let rt = &self.runtime;
        run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(self.inner.put_kv_if_absent(&id_str, &json_val))
                .map_err(contextra_err)
        })
    }

    // ANCHOR[contextra-py:KV-003]
    /// Liest ein Key-Value-Paar direkt anhand des Schlüssels aus dem Speicher.
    ///
    /// # Rückgabe
    /// - `Some(dict)` falls der Eintrag existiert (als Python-Dictionary deserialisiert).
    /// - `None` falls kein Eintrag zu diesem Schlüssel vorhanden ist.
    ///
    /// # Ablauf
    /// 1. Schlüssel prüfen (`validate_id_obj`).
    /// 2. `inner.get_kv` blockierend unter GIL-Release ausführen.
    /// 3. Gefundenes JSON-Objekt zurück in ein Python-Objekt transformieren (`json_to_py`).
    pub fn get_kv<'py>(
        &self,
        py: Python<'py>,
        id: &Bound<'py, PyAny>,
    ) -> PyResult<Option<PyObject>> {
        let id_str = validate_id_obj(id)?;
        let rt = &self.runtime;
        let opt_val = run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(self.inner.get_kv(&id_str))
                .map_err(contextra_err)
        })?;
        match opt_val {
            Some(val) => Ok(Some(json_to_py(py, &val)?)),
            None => Ok(None),
        }
    }

    // ANCHOR[contextra-py:LINK-002]
    /// Erstellt atomar eine ungerichtete bzw. beidseitige Relation zwischen zwei Dokumenten.
    ///
    /// # Parameter
    /// - `from`: ID des Ursprungsdokuments.
    /// - `to`: ID des Zieldokuments.
    /// - `label`: Beschriftung/Klassifikation der Beziehung.
    ///
    /// # Ablauf
    /// 1. Beide IDs sowie das Label validieren (`validate_label`).
    /// 2. Asynchron `inner.relate_bidirectional` unter GIL-Release aufrufen.
    pub fn relate_bidirectional<'py>(
        &self,
        py: Python<'py>,
        from: &Bound<'py, PyAny>,
        to: &Bound<'py, PyAny>,
        label: &str,
    ) -> PyResult<()> {
        let from_str = validate_id_obj(from)?;
        let to_str = validate_id_obj(to)?;
        validate_label(label)?;
        let rt = &self.runtime;
        let label_owned = label.to_string();
        run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(
                self.inner
                    .relate_bidirectional(&from_str, &to_str, &label_owned),
            )
            .map_err(contextra_err)
        })
    }

    // ANCHOR[contextra-py:LINK-003]
    /// Verknüpft zwei Memory-Knoten mit einer typisierten Zettelkasten-Relation (A-MEM).
    ///
    /// # Parameter
    /// - `from`: ID des Ausgangsdokuments.
    /// - `to`: ID des Zieldokuments.
    /// - `relation`: Einer der Werte: "elaborates", "contradicts", "supersedes", "references".
    ///
    /// # Ablauf
    /// 1. IDs und Relations-String parsen (`parse_link_relation`).
    /// 2. In interne `DocId`s auflösen (`DocId::from_key`).
    /// 3. Link via `inner.link_memories` atomar persistieren.
    pub fn link_memories<'py>(
        &self,
        py: Python<'py>,
        from: &Bound<'py, PyAny>,
        to: &Bound<'py, PyAny>,
        relation: &str,
    ) -> PyResult<()> {
        let from_str = validate_id_obj(from)?;
        let to_str = validate_id_obj(to)?;
        let rel = parse_link_relation(relation)?;
        let from_doc_id = DocId::from_key(&from_str).map_err(contextra_err)?;
        let to_doc_id = DocId::from_key(&to_str).map_err(contextra_err)?;
        let rt = &self.runtime;
        run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(self.inner.link_memories(from_doc_id, to_doc_id, rel))
                .map_err(contextra_err)
        })
    }

    // ANCHOR[contextra-py:LINK-004]
    /// Ruft alle ausgehenden Memory-Links für ein bestimmtes Dokument ab.
    ///
    /// # Rückgabe
    /// Eine Python-Liste von Dictionaries mit den Schlüsseln `target`, `relation` und `created_at_tx`.
    ///
    /// # Ablauf
    /// 1. ID validieren und in `DocId` konvertieren.
    /// 2. `inner.get_links` unter GIL-Release abfragen.
    /// 3. Rust-Linkstrukturen in Python-Dictionaries überführen.
    pub fn get_links<'py>(
        &self,
        py: Python<'py>,
        id: &Bound<'py, PyAny>,
    ) -> PyResult<Vec<PyObject>> {
        let id_str = validate_id_obj(id)?;
        let doc_id = DocId::from_key(&id_str).map_err(contextra_err)?;
        let rt = &self.runtime;
        let links = run_blocking_ffi(py, &self.poisoned, || {
            rt.block_on(self.inner.get_links(doc_id))
                .map_err(contextra_err)
        })?;

        let mut py_res = Vec::with_capacity(links.len());
        for link in links {
            let dict = PyDict::new(py);
            dict.set_item("target", link.target.inner())?;
            dict.set_item("relation", relation_as_str(link.relation))?;
            dict.set_item("created_at_tx", link.created_at_tx.inner())?;
            py_res.push(dict.into_any().unbind());
        }
        Ok(py_res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Prüft, dass alle gültigen Relationsbezeichnungen unabhängig von Groß-/Kleinschreibung korrekt geparst werden.
    #[test]
    fn test_parse_link_relation_valid_cases() {
        assert_eq!(
            parse_link_relation("elaborates").expect("should parse"),
            LinkRelation::Elaborates
        );
        assert_eq!(
            parse_link_relation("Elaborates").expect("should parse"),
            LinkRelation::Elaborates
        );
        assert_eq!(
            parse_link_relation("ELABORATES").expect("should parse"),
            LinkRelation::Elaborates
        );

        assert_eq!(
            parse_link_relation("contradicts").expect("should parse"),
            LinkRelation::Contradicts
        );
        assert_eq!(
            parse_link_relation("Contradicts").expect("should parse"),
            LinkRelation::Contradicts
        );

        assert_eq!(
            parse_link_relation("supersedes").expect("should parse"),
            LinkRelation::Supersedes
        );
        assert_eq!(
            parse_link_relation("SuperSedes").expect("should parse"),
            LinkRelation::Supersedes
        );

        assert_eq!(
            parse_link_relation("references").expect("should parse"),
            LinkRelation::References
        );
        assert_eq!(
            parse_link_relation("REFERENCES").expect("should parse"),
            LinkRelation::References
        );
    }

    // Prüft, dass ungültige oder leere Relationsbezeichnungen mit einem Fehler abgewiesen werden.
    #[test]
    fn test_parse_link_relation_invalid_cases() {
        assert!(parse_link_relation("unknown").is_err());
        assert!(parse_link_relation("").is_err());
        assert!(parse_link_relation("related").is_err());
    }
}
