// Compile-Fail Test: Fehlender Runtime-Handle verhindert Aufruf von .open() / .build() (SPEC 5.7, T-API-1)
use contextra::{Contextra, Secret};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let secret = Secret::new(b"sehr-sichere-passphrase-123".to_vec());

    let builder = Contextra::builder()
        .data_dir("./target/trybuild_test_db")
        .passphrase(secret)
        .backup_retention(Duration::from_secs(86400))
        .dimension(768);

    let _db = builder.open().await;
}
