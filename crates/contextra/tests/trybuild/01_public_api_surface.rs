// Testet, dass die minimale öffentliche API-Oberfläche von contextra fehlerfrei kompiliert.
use contextra::{system_defaults, Contextra, DocKey, DocRef, EngineBuilder, Error, Secret, TenantContext};

#[tokio::main]
async fn main() {
    let secret = Secret::new(b"sehr-sichere-passphrase-123".to_vec());
    let rt = tokio::runtime::Handle::current();

    let _builder: EngineBuilder<_, _> = Contextra::builder()
        .data_dir("./target/trybuild_test_db")
        .passphrase(secret)
        .runtime(rt);

    let _sys = system_defaults();
    let _err = Error::NotFound;

    let doc_key = DocKey::try_new(b"doc-123".to_vec()).unwrap();
    let _doc_ref = DocRef {
        subject: None,
        key: doc_key,
    };
}
