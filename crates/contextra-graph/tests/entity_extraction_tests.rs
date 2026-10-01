// FILE-CONTEXT
// ZWECK: Integrations- und Evaluationstests für die regelbasierte Entitäts- und Relations-Extraktion in contextra-graph.

use contextra_graph::csr::CsrGraph;
use contextra_graph::entity_extraction::{
    ingest_extracted_relations, EntityExtractor, RuleBasedEntityExtractor,
};
use contextra_types::DocId;

#[test]
fn test_rule_based_entity_extractor_sample_cases() {
    let extractor = RuleBasedEntityExtractor::new(0.50);

    // Testfall 1: Standard-Kausal- / Arbeitsbeziehung
    let t1 = "Alice arbeitet bei AcmeCorp als Lead Engineer.";
    let r1 = extractor.extract(t1);
    assert_eq!(r1.len(), 1);
    assert_eq!(r1[0].source, "Alice");
    assert_eq!(r1[0].target, "AcmeCorp");
    assert_eq!(r1[0].relation_type, "WORKS_AT");

    // Testfall 2: Wohnort-Beziehung
    let t2 = "Bob wohnt in Berlin seit fünf Jahren.";
    let r2 = extractor.extract(t2);
    assert_eq!(r2.len(), 1);
    assert_eq!(r2[0].source, "Bob");
    assert_eq!(r2[0].target, "Berlin");
    assert_eq!(r2[0].relation_type, "LIVES_IN");

    // Testfall 3: Gründungs-Beziehung (Englisch)
    let t3 = "Carol founded TechStart in 2020.";
    let r3 = extractor.extract(t3);
    assert_eq!(r3.len(), 1);
    assert_eq!(r3[0].source, "Carol");
    assert_eq!(r3[0].target, "TechStart");
    assert_eq!(r3[0].relation_type, "FOUNDED");

    // Testfall 4: Mehrwort-Eigenname mit Kooperation
    let t4 = "Max Mustermann kooperiert mit Siemens Energy an neuen Projekten.";
    let r4 = extractor.extract(t4);
    assert_eq!(r4.len(), 1);
    assert_eq!(r4[0].source, "Max Mustermann");
    assert_eq!(r4[0].target, "Siemens Energy");
    assert_eq!(r4[0].relation_type, "PARTNERED_WITH");

    // Testfall 5: Komplexe / Invertierte Satzstruktur (Verbindungswort / Führung)
    let t5 = "Seit 2018 leitet Dr Frank Miller die Entwicklungsabteilung von GlobalTech.";
    let r5 = extractor.extract(t5);
    // Erwartung bei regelbasierter Heuristik: "Dr Frank Miller" als Subjekt, "Entwicklungsabteilung" als Objekt
    assert!(!r5.is_empty());
    assert_eq!(r5[0].relation_type, "LEADS");
}

#[test]
fn test_confidence_filtering() {
    let extractor = RuleBasedEntityExtractor::new(0.82);

    let text = "Projekt Alpha ist verbunden mit Subsystem Beta."; // base_confidence 0.75
    let res = extractor.extract(text);
    // Wegen min_confidence = 0.82 sollte die Relation gefiltert werden
    assert!(res.is_empty());

    let text_high = "Dr Smith gruendete BioHealth."; // base_confidence 0.90
    let res_high = extractor.extract(text_high);
    assert_eq!(res_high.len(), 1);
}

#[test]
fn test_ingest_extracted_relations_to_graph() {
    let graph = CsrGraph::new();
    let extractor = RuleBasedEntityExtractor::new(0.50);

    let text = "Eva arbeitet bei OpenAI an KI-Systemen.";
    let extracted = extractor.extract(text);
    assert_eq!(extracted.len(), 1);

    let doc_id = DocId::from(42u64);
    let created_hyperedges =
        ingest_extracted_relations(&graph, &extracted, 0.50, Some(doc_id)).expect("ingest");

    assert_eq!(created_hyperedges.len(), 1);
    let he = &created_hyperedges[0];
    assert_eq!(he.participants.len(), 2);
    assert_eq!(he.source_doc_id, Some(doc_id));

    let doc_hyperedges = graph.hyperedges_for_doc(doc_id);
    assert_eq!(doc_hyperedges.len(), 1);
    assert_eq!(doc_hyperedges[0], he.id);
}
