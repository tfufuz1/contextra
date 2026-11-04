//! # Golden-Test: Fehlermodell und ErrorCode-Werte (SPEC 7.5, D-110, T-API-4)
//!
//! Normativer Test für das Fehlermodell der Fassade:
//! - Fehlertexte und `ErrorCode`-Werte sind eingefroren (D-110).
//! - Jeder Fehler besitzt eine konkrete Handlungsempfehlung (SPEC §6.4, 7.5).
//! - Keine Fehlertexte enthalten Nutzerinhalte, Passphrasen, Schlüssel oder Klartext-Kennungen (HR-4, T-LOG-1).
//! - Keine verbotenen Werbe- oder Löschaussagenbegriffe nach SPEC §6.4.

use contextra::error::{ConfigDetails, ConflictDetails, Error, ErrorCode, LimitDetails};

/// SPEC: T-API-4
#[test]
fn test_error_golden_strings_and_codes() {
    let cases = vec![
        (
            Error::Config(ConfigDetails::VaultPlacement),
            ErrorCode::ConfigVaultPlacement,
            1001,
            "Schlüsseltresor darf nicht im Datenverzeichnis liegen. Bitte separates vault_dir konfigurieren.",
        ),
        (
            Error::Config(ConfigDetails::VaultMissing),
            ErrorCode::ConfigVaultMissing,
            1002,
            "Schlüsseltresor-Verzeichnis nicht gefunden. Bitte Pfad und Zugriffsrechte prüfen.",
        ),
        (
            Error::Config(ConfigDetails::VaultMismatch),
            ErrorCode::ConfigVaultMismatch,
            1003,
            "Schlüsseltresor-Parameter weichen ab. Konfiguration an den vorhandenen Tresor anpassen.",
        ),
        (
            Error::Config(ConfigDetails::UnsupportedFilesystem(42)),
            ErrorCode::ConfigUnsupportedFilesystem,
            1004,
            "Dateisystem wird nicht unterstützt (Code: 42). Bitte Tier-1- oder Tier-2-Dateisystem verwenden.",
        ),
        (
            Error::Config(ConfigDetails::Other("invalid parameter".to_string())),
            ErrorCode::Config,
            1000,
            "Konfigurationsfehler: invalid parameter. Konfigurationsparameter korrigieren.",
        ),
        (
            Error::MissingParameter { parameter: "passphrase" },
            ErrorCode::MissingParameter,
            1005,
            "Pflichtparameter fehlt: passphrase. Bitte den erforderlichen Parameter angeben.",
        ),
        (
            Error::Io,
            ErrorCode::Io,
            2000,
            "Dateisystem- oder I/O-Fehler. Bitte Schreibrechte, Speicherplatz und Pfade prüfen.",
        ),
        (
            Error::Corrupt,
            ErrorCode::Corrupt,
            3000,
            "Datenkorruption festgestellt. Bitte Sicherung oder Restore-Verfahren hinzuziehen.",
        ),
        (
            Error::Auth,
            ErrorCode::Auth,
            4000,
            "Authentisierung fehlgeschlagen. Bitte Passphrase überprüfen.",
        ),
        (
            Error::Erased,
            ErrorCode::Erased,
            5000,
            "Das angeforderte Zielobjekt wurde entfernt.",
        ),
        (
            Error::Restricted,
            ErrorCode::Restricted,
            6000,
            "Die Verarbeitung des Objekts ist eingeschränkt (z. B. Art. 18).",
        ),
        (
            Error::NotFound,
            ErrorCode::NotFound,
            7000,
            "Objekt nicht gefunden. Vorgang mit gültiger Kennung wiederholen.",
        ),
        (
            Error::Conflict(ConflictDetails::IdCollision),
            ErrorCode::ConflictIdCollision,
            8001,
            "Identitätskollision festgestellt. Transaktion wiederholen oder Schlüsselraum prüfen.",
        ),
        (
            Error::Conflict(ConflictDetails::Exists),
            ErrorCode::ConflictExists,
            8002,
            "Eintrag existiert bereits. Bitte andere Kennung wählen oder Aktualisierung nutzen.",
        ),
        (
            Error::Conflict(ConflictDetails::SameIdentity),
            ErrorCode::ConflictSameIdentity,
            8003,
            "Ersetzung erfordert eine veränderte Identität. Identitätsangaben anpassen.",
        ),
        (
            Error::Conflict(ConflictDetails::RetentionMismatch),
            ErrorCode::ConflictRetentionMismatch,
            8004,
            "Aufbewahrungsattestierung widerspricht bestehendem Vorgang. Status prüfen.",
        ),
        (
            Error::Conflict(ConflictDetails::SerializationFailure),
            ErrorCode::ConflictSerializationFailure,
            8005,
            "Serialisierungsfehler in Transaktion. Transaktion erneut ausführen.",
        ),
        (
            Error::Conflict(ConflictDetails::SubjectImmutable),
            ErrorCode::ConflictSubjectImmutable,
            8006,
            "Betroffener kann nach Anlage nicht mehr geändert werden.",
        ),
        (
            Error::Limit(LimitDetails::Capacity),
            ErrorCode::LimitCapacity,
            9001,
            "Speicherkapazität erschöpft. Bereinigung durchführen oder Speicherplatz erweitern.",
        ),
        (
            Error::Limit(LimitDetails::DiskReserve),
            ErrorCode::LimitDiskReserve,
            9002,
            "Mindest-Plattenreserve unterschritten. Freien Speicherplatz auf dem Medium schaffen.",
        ),
        (
            Error::Limit(LimitDetails::Snapshots),
            ErrorCode::LimitSnapshots,
            9003,
            "Maximale Anzahl aktiver Lese-Snapshots erreicht. Nicht benötigte Snapshots freigeben.",
        ),
        (
            Error::Limit(LimitDetails::QueryTooLarge),
            ErrorCode::LimitQueryTooLarge,
            9004,
            "Suchanfrage überschreitet die zulässige Maximalgröße. Anfrage kürzen.",
        ),
        (
            Error::Limit(LimitDetails::VectorMemory),
            ErrorCode::LimitVectorMemory,
            9005,
            "Vektorspeicher-Budget erschöpft. Speicherbudget erhöhen oder Segmente verkleinern.",
        ),
        (
            Error::Limit(LimitDetails::Dimension),
            ErrorCode::LimitDimension,
            9006,
            "Vektordimension überschreitet die Konfigurationsgrenze. Dimension anpassen.",
        ),
        (
            Error::SnapshotRevoked,
            ErrorCode::SnapshotRevoked,
            10000,
            "Snapshot wurde widerrufen. Neuen Lese-Snapshot anfordern.",
        ),
        (
            Error::Cancelled,
            ErrorCode::Cancelled,
            11000,
            "Operation wurde vorzeitig abgebrochen.",
        ),
        (
            Error::Refused,
            ErrorCode::Refused,
            12000,
            "Operation abgewiesen. Prüfen Sie die bestehenden Aufbewahrungspflichten.",
        ),
        (
            Error::Internal,
            ErrorCode::Internal,
            13000,
            "Interner Systemfehler. Systemzustand prüfen und Vorgang wiederholen.",
        ),
    ];

    for (err, expected_code, expected_u32, expected_msg) in cases {
        assert_eq!(
            err.code(),
            expected_code,
            "ErrorCode mismatch for {expected_msg}"
        );
        assert_eq!(
            err.code().as_u32(),
            expected_u32,
            "Numeric code mismatch for {expected_msg}"
        );
        assert_eq!(err.to_string(), expected_msg, "Display message mismatch");
    }
}
