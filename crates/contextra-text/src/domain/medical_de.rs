// FILE-CONTEXT: Deutsches Medizinisches Fachvokabular (domain::medical_de).
// ZWECK: Definiert Komposita-Staemme und geschuetzte Begriffe fuer domaenenspezifische Sprachanforderungen.
// QUELLE: data/medical_de.txt (BfArM ICD-10-GM / MeSH German / BMG Open Data).
// REGELN: Kleingeschrieben, alphabetisch sortiert, keine Duplikate, Staemme und geschuetzte Begriffe ueberschneiden sich nicht, keine Eigennamen/Marken.

use super::{parse_domain_resource, DomainVocabulary, ParsedVocabulary};
use std::sync::OnceLock;

const RAW_MEDICAL_DATA: &str = include_str!("../../data/medical_de.txt");

static PARSED_MEDICAL: OnceLock<ParsedVocabulary> = OnceLock::new();

fn get_parsed_medical() -> &'static ParsedVocabulary {
    PARSED_MEDICAL.get_or_init(|| parse_domain_resource(RAW_MEDICAL_DATA))
}

/// Deutsches Medizinisches Fachvokabular.
pub struct MedicalDomainVocabulary;

impl DomainVocabulary for MedicalDomainVocabulary {
    fn compound_stems(&self) -> &'static [&'static str] {
        get_parsed_medical().compound_stems
    }

    fn protected_terms(&self) -> &'static [&'static str] {
        get_parsed_medical().protected_terms
    }

    fn domain_id(&self) -> &'static str {
        "medical_de"
    }
}

/// Exportierte Hilfsfunktionen für direkten Slice-Zugriff.
pub fn get_medical_compound_stems() -> &'static [&'static str] {
    get_parsed_medical().compound_stems
}

pub fn get_medical_protected_terms() -> &'static [&'static str] {
    get_parsed_medical().protected_terms
}

/// Statische Referenz-Slices für Abwärtskompatibilität in vorhandenen Tests.
pub static MEDICAL_COMPOUND_STEMS: std::sync::LazyLock<&'static [&'static str]> =
    std::sync::LazyLock::new(get_medical_compound_stems);

pub static MEDICAL_PROTECTED_TERMS: std::sync::LazyLock<&'static [&'static str]> =
    std::sync::LazyLock::new(get_medical_protected_terms);
