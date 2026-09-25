use super::DomainVocabulary;

/// German medical domain vocabulary (*Deutsches Medizinisches Fachvokabular*).
pub struct MedicalDomainVocabulary;

/// Domain-specific compound stems for German medical language.
pub const MEDICAL_COMPOUND_STEMS: &[&str] = &[
    "Allergie",
    "Anamnese",
    "Arzt",
    "Attest",
    "Aufklärung",
    "Befund",
    "Behandlung",
    "Blutbild",
    "Chirurgie",
    "Diagnose",
    "Diagnostik",
    "Dosis",
    "EKG",
    "Einweisung",
    "Einwilligung",
    "Epidemie",
    "Erkrankung",
    "Fieber",
    "Impfung",
    "Infektion",
    "Intensivstation",
    "Kardiologie",
    "Klinik",
    "Krankmeldung",
    "Labor",
    "Medikation",
    "Medizin",
    "Narkose",
    "Onkologie",
    "Pathologie",
    "Patient",
    "Pflege",
    "Praxis",
    "Prognose",
    "Prophylaxe",
    "Prävention",
    "Radiologie",
    "Reanimation",
    "Reha",
    "Rezept",
    "Schmerz",
    "Symptom",
    "Syndrom",
    "Therapie",
    "Transfusion",
    "Trauma",
    "Ultraschall",
    "Untersuchung",
    "Verordnung",
    "Visite",
    "Wunde",
    "Überweisung",
];

/// Medical domain terms that must NOT be treated as stopwords.
pub const MEDICAL_PROTECTED_TERMS: &[&str] =
    &["Arzt", "Dosis", "EKG", "Labor", "OP", "Reha", "Virus", "Wunde"];

impl DomainVocabulary for MedicalDomainVocabulary {
    fn compound_stems(&self) -> &'static [&'static str] {
        MEDICAL_COMPOUND_STEMS
    }

    fn protected_terms(&self) -> &'static [&'static str] {
        MEDICAL_PROTECTED_TERMS
    }

    fn domain_id(&self) -> &'static str {
        "medical_de"
    }
}
