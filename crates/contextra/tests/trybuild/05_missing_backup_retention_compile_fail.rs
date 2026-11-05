// Compile-Fail Test: Fehlende backup_retention verhindert Aufruf von .open() / .build() (SPEC 5.7, D-74, T-API-1)
use contextra::{Contextra, Secret};

#[tokio::main]
async fn main() {
    let secret = Secret::new(b"sehr-sichere-passphrase-123".to_vec());
    let rt = tokio::runtime::Handle::current();

    // Aufruf von .open() erfordert EngineBuilder<Set, Set, Set, Set>
    let builder = Contextra::builder()
        .data_dir("./target/trybuild_test_db")
        .passphrase(secret)
        .runtime(rt)
        .dimension(768);

    let _db = builder.open().await;
}
