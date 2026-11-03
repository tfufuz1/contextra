// Compile-Fail Test: Fehlender TenantContext verhindert Zugriff auf Collection
use contextra::Contextra;

#[tokio::main]
async fn main() {
    // Auf Contextra gibt es keine mandantenlose Methode collection(...)
    // Man muss erst .tenant(ctx).await aufrufen!
    fn check_requires_tenant_ctx(db: &Contextra) {
        let _ = db.collection("default", Default::default());
    }
}
