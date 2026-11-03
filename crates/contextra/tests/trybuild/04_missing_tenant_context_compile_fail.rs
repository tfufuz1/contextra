// Compile-Fail Test: Fehlender TenantContext verhindert Zugriff auf Collection (SPEC 7.1, D-07, T-API-1)
use contextra::Contextra;

#[tokio::main]
async fn main() {
    // Auf Contextra gibt es keine mandantenlose Methode collection(...)
    // Man muss erst .tenant(ctx).await aufrufen!
    fn check_requires_tenant_ctx(db: &Contextra) {
        let _ = db.collection("default", Default::default());
    }
}
