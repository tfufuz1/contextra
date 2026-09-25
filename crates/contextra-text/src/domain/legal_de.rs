use super::DomainVocabulary;

/// German legal domain vocabulary (*Deutsches Juristisches Fachvokabular*).
pub struct LegalDomainVocabulary;

/// Domain-specific compound stems for German legal language.
pub const LEGAL_COMPOUND_STEMS: &[&str] = &[
    "Akte",
    "Antragsteller",
    "Anwalt",
    "Anwaltschaft",
    "Auflagenerteilung",
    "Beklagte",
    "Berufung",
    "Beschwerde",
    "Beweisaufnahme",
    "Einspruch",
    "Entschädigung",
    "Frist",
    "Gericht",
    "Gesetzesentwurf",
    "Gutachten",
    "Haftung",
    "Insolvenz",
    "Jurist",
    "Kanzlei",
    "Kausalität",
    "Klage",
    "Kläger",
    "Kündigung",
    "Mandat",
    "Nichtigkeitsklage",
    "Ordnungswidrigkeit",
    "Paragraf",
    "Prozess",
    "Recht",
    "Rechtsmittel",
    "Revision",
    "Schadenersatz",
    "Schiedsspruch",
    "Strafmaß",
    "Straftat",
    "Testamentseröffnung",
    "Urteil",
    "Verfassungsbeschwerde",
    "Vergleich",
    "Verjährung",
    "Vertrag",
    "Vertretung",
    "Vollmacht",
    "Vollstreckung",
    "Widerruf",
    "Zeugenaussage",
];

/// Legal domain terms that must NOT be treated as stopwords.
pub const LEGAL_PROTECTED_TERMS: &[&str] = &[
    "Akte", "Eid", "Frist", "Gut", "Jura", "Klage", "Notar", "Recht", "Tat",
];

impl DomainVocabulary for LegalDomainVocabulary {
    fn compound_stems(&self) -> &'static [&'static str] {
        LEGAL_COMPOUND_STEMS
    }

    fn protected_terms(&self) -> &'static [&'static str] {
        LEGAL_PROTECTED_TERMS
    }

    fn domain_id(&self) -> &'static str {
        "legal_de"
    }
}
