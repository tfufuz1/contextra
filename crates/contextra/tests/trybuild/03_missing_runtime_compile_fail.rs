// Compile-Fail Test: Fehlender Runtime-Handle verhindert Aufruf von .open() / .build()
use contextra::{Contextra, Secret};

#[tokio::main]
async fn main() {
    let secret = Secret::new(b"sehr-sichere-passphrase-123".to_vec());

    // Der Builder hat hier den Typ EngineBuilder<Set, Missing>
    // Aufruf von .open() ist nur für EngineBuilder<Set, Set> definiert!
    let builder = Contextra::builder()
        .data_dir("./target/trybuild_test_db")
        .passphrase(secret);

    let _db = builder.open().await;
}
