//! # Property-Test: Keine Geheimnisse in Error Display und Debug
//!
//! SPEC §6.4 & §7: Fehlertypen dürfen niemals sensible Parameter (wie Passphrasen,
//! Master-Keys oder Seed-Bytes) über `Display` oder `Debug` preisgeben.

use contextra::error::{ConfigDetails, ConflictDetails, LimitDetails};
use contextra::Error;

#[test]
fn test_errors_do_not_leak_secrets() {
    let test_secret = "super_secret_plant_passphrase_xyz123";

    let errors = vec![
        Error::Config(ConfigDetails::Other("Ungültige Konfiguration".to_string())),
        Error::MissingParameter {
            parameter: "passphrase",
        },
        Error::Io,
        Error::Corrupt,
        Error::Auth,
        Error::Erased,
        Error::Restricted,
        Error::NotFound,
        Error::Conflict(ConflictDetails::IdCollision),
        Error::Limit(LimitDetails::Capacity),
        Error::SnapshotRevoked,
        Error::Cancelled,
        Error::Refused,
        Error::Internal,
    ];

    for err in errors {
        let display_str = format!("{err}");
        let debug_str = format!("{err:?}");

        assert!(
            !display_str.contains(test_secret),
            "Geheimnis in Display gefunden: {display_str}"
        );
        assert!(
            !debug_str.contains(test_secret),
            "Geheimnis in Debug gefunden: {debug_str}"
        );

        // Prüfe auch auf verbotene Begriffe nach SPEC §6.4
        let forbidden_terms = [
            "DSGVO-konform",
            "garantiert gelöscht",
            "100 % sicher",
            "100% sicher",
            "unwiederbringlich",
            "rechtskonform",
        ];
        for forbidden in forbidden_terms {
            assert!(
                !display_str.contains(forbidden),
                "Verbotener Begriff '{forbidden}' in Error Display: {display_str}"
            );
            assert!(
                !debug_str.contains(forbidden),
                "Verbotener Begriff '{forbidden}' in Error Debug: {debug_str}"
            );
        }
    }
}
