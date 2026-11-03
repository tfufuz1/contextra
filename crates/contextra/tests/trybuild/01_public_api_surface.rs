// Testet, dass die minimale öffentliche API-Oberfläche von contextra fehlerfrei kompiliert (SPEC §7, T-API-1).
use contextra::{
    system_defaults, Collection, CollectionOptions, Contextra, DocHandle, DocKey, DocRef,
    Document, EngineBuilder, EraseMode, EraseReceipt, EraseRequest, EraseTarget, ErasureId,
    ErasureStatus, Error, Filter, HealthReport, Hit, Missing, RetentionAttestation, RetentionCode,
    ScanRange, SearchPage, Secret, Set, SignedReport, Tenant, TenantContext, TenantRef, Timing,
};

#[tokio::main]
async fn main() {
    let secret = Secret::new(b"sehr-sichere-passphrase-123".to_vec());
    let rt = tokio::runtime::Handle::current();

    let _builder: EngineBuilder<Missing, Missing> = Contextra::builder();
    let configured_builder: EngineBuilder<Set, Set> = Contextra::builder()
        .data_dir("./target/trybuild_test_db")
        .passphrase(secret.clone())
        .runtime(rt)
        .erase_sla_secs(86400)
        .backup_retention_days(365);

    let _sys = system_defaults();
    let _err = Error::NotFound;

    let doc_key = DocKey::try_new(b"doc-123".to_vec()).unwrap();
    let doc_ref = DocRef {
        subject: None,
        key: doc_key,
    };

    let doc = Document {
        content: "Beispiel".to_string(),
        metadata: vec![("category".to_string(), "test".to_string())],
        vector: Some(vec![0.1; 768]),
    };

    let filter = Filter::new();
    let _range_all = ScanRange::All;

    // Compile-Pass für Signaturen
    fn check_signatures(coll: &Collection, doc_ref: DocRef, doc: Document, filter: Filter) {
        let _ = coll.insert(doc_ref.clone(), doc.clone());
        let _ = coll.update(doc_ref.clone(), doc.clone());
        let _ = coll.get(doc_ref.clone(), None);
        let _ = coll.soft_delete(doc_ref.clone());
        let _ = coll.scan(ScanRange::All);
        let _ = coll.search_text("query", 10);
        let _ = coll.search_vector(vec![0.0; 768], 10, Some(filter));
    }
}
