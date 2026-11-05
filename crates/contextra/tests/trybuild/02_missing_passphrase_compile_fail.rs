// Compile-Fail Test: Fehlende Passphrase verhindert Aufruf von .open() / .build() (SPEC 5.7, T-API-1)
use contextra::Contextra;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let rt = tokio::runtime::Handle::current();

    let builder = Contextra::builder()
        .data_dir("./target/trybuild_test_db")
        .runtime(rt)
        .backup_retention(Duration::from_secs(86400))
        .dimension(768);

    let _db = builder.open().await;
}
