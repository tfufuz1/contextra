// FILE-CONTEXT: Deutsches Juristisches Fachvokabular (domain::legal_de).
// ZWECK: Definiert Komposita-Staemme und geschuetzte Begriffe fuer domaenenspezifische Sprachanforderungen.
// QUELLE: data/legal_de.txt (Offene Rechts- und Justizterminologie der Bundesrepublik Deutschland).
// REGELN: Kleingeschrieben, alphabetisch sortiert, keine Duplikate, Staemme und geschuetzte Begriffe ueberschneiden sich nicht, keine Eigennamen/Marken.

use super::{parse_domain_resource, DomainVocabulary, ParsedVocabulary};
use std::sync::OnceLock;

const RAW_LEGAL_DATA: &str = include_str!("../../data/legal_de.txt");

static PARSED_LEGAL: OnceLock<ParsedVocabulary> = OnceLock::new();

fn get_parsed_legal() -> &'static ParsedVocabulary {
    PARSED_LEGAL.get_or_init(|| parse_domain_resource(RAW_LEGAL_DATA))
}

/// Deutsches Juristisches Fachvokabular.
pub struct LegalDomainVocabulary;

impl DomainVocabulary for LegalDomainVocabulary {
    fn compound_stems(&self) -> &'static [&'static str] {
        get_parsed_legal().compound_stems
    }

    fn protected_terms(&self) -> &'static [&'static str] {
        get_parsed_legal().protected_terms
    }

    fn domain_id(&self) -> &'static str {
        "legal_de"
    }
}

/// Exportierte Hilfsfunktionen für direkten Slice-Zugriff.
pub fn get_legal_compound_stems() -> &'static [&'static str] {
    get_parsed_legal().compound_stems
}

pub fn get_legal_protected_terms() -> &'static [&'static str] {
    get_parsed_legal().protected_terms
}

/// Statische Referenz-Slices für Abwärtskompatibilität in vorhandenen Tests.
pub static LEGAL_COMPOUND_STEMS: std::sync::LazyLock<&'static [&'static str]> =
    std::sync::LazyLock::new(get_legal_compound_stems);

pub static LEGAL_PROTECTED_TERMS: std::sync::LazyLock<&'static [&'static str]> =
    std::sync::LazyLock::new(get_legal_protected_terms);
