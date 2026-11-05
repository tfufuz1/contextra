// Compile-Fail Test: Fehlende dimension verhindert Aufruf von .open() / .build() (SPEC 5.7, D-89, T-API-1)
use contextra::{Contextra, Secret};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let secret = Secret::new(b"sehr-sichere-passphrase-123".to_vec());
    let rt = tokio::runtime::Handle::current();

    // Aufruf von .open() erfordert EngineBuilder<Set, Set, Set, Set>
    let builder = Contextra::builder()
        .data_dir("./target/trybuild_test_db")
        .passphrase(secret)
        .runtime(rt)
        .backup_retention(Duration::from_secs(86400));

    let _db = builder.open().await;
}
