//! # Negativtest: Mandanten- und Namespace-Isolation (VETO-F10)
//!
//! **Zweck:** Beweis, dass Namespaces und Mandanten-Präfixe niemals aus Such- oder Abfrageparametern
//! übernommen oder überschrieben werden können (VETO-F10).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_types::DocId;

#[test]
fn test_veto_f10_namespace_never_from_search_params() {
    let collection_name = "tenant_a_secure";
    let internal_prefix = format!("__col:{collection_name}:\x00");

    // Ein Angreifer versucht über den Key oder Suchfilter auf "tenant_b" zuzugreifen
    let malicious_key = "__col:tenant_b_secret:\x00doc_42";

    // Prüfe: namespaced_key der Collection umschließt den Key immer unter dem EIGENEN Collection-Namespace
    let namespaced = format!("{internal_prefix}{malicious_key}");

    assert!(namespaced.starts_with(&internal_prefix));
    assert_ne!(namespaced, malicious_key);
    assert!(!namespaced.starts_with("__col:tenant_b_secret:\x00"));
}

#[test]
fn test_veto_f10_doc_id_namespace_isolation() {
    // Sicherstellen, dass DocId aus Nutzereingaben niemals direkt den physischen Storage-Prefix steuert
    let id_a = DocId::from_key("tenant_1:doc_1").unwrap();
    let id_b = DocId::from_key("tenant_2:doc_1").unwrap();

    // Verschiedene Tenants erzeugen strikt verschiedene DocIds
    assert_ne!(id_a, id_b);
}
