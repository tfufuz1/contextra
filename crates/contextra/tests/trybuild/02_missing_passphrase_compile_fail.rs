// Compile-Fail Test: Fehlende Passphrase verhindert Aufruf von .open() / .build() (SPEC 5.7, T-API-1)
use contextra::Contextra;

#[tokio::main]
async fn main() {
    let rt = tokio::runtime::Handle::current();

    // Der Builder hat hier den Typ EngineBuilder<Missing, Set>
    // Aufruf von .open() ist nur für EngineBuilder<Set, Set> definiert!
    let builder = Contextra::builder()
        .data_dir("./target/trybuild_test_db")
        .runtime(rt);

    let _db = builder.open().await;
}
