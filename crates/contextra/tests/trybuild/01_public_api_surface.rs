// Testet, dass die öffentliche API-Oberfläche von contextra fehlerfrei kompiliert.

fn main() {
    let _b = contextra::builder(768);
    let _mode = contextra::EncryptionMode::Unencrypted;
    let _metric = contextra::DistanceMetric::Cosine;
    let _err = contextra::ContextraError::invalid_input("test");
    let _rng = contextra::OsRngAdapter::new();
}
