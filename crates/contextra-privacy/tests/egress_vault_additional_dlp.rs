use contextra_privacy::egress_vault::{
    is_iban_valid, is_luhn_valid, normalize_payload, BlockReason, EgressClassification, EgressVault,
};

#[tokio::test]
async fn test_valid_iban_blocked_and_invalid_iban_allowed() {
    let vault = EgressVault::try_default().expect("valid vault");

    let valid_iban = "DE89370400440532013000";
    let res_valid = vault.classify(valid_iban).await;
    assert!(
        matches!(res_valid, EgressClassification::Block(BlockReason::SensitivePattern(_))),
        "Valid IBAN must be blocked"
    );

    let invalid_iban = "DE00370400440532013000";
    let res_invalid = vault.classify(invalid_iban).await;
    assert_eq!(
        res_invalid,
        EgressClassification::Allow,
        "Invalid IBAN checksum must be allowed"
    );
}

#[tokio::test]
async fn test_jwt_token_blocked() {
    let vault = EgressVault::try_default().expect("valid vault");

    let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    let res = vault.classify(jwt).await;
    assert!(
        matches!(res, EgressClassification::Block(BlockReason::SensitivePattern(_))),
        "JWT token must be blocked"
    );
}

#[tokio::test]
async fn test_pem_block_blocked() {
    let vault = EgressVault::try_default().expect("valid vault");

    let pem = "-----BEGIN PRIVATE KEY-----\nMIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQC...\n-----END PRIVATE KEY-----";
    let res = vault.classify(pem).await;
    assert!(
        matches!(res, EgressClassification::Block(BlockReason::SensitivePattern(_))),
        "PEM block must be blocked"
    );
}

#[tokio::test]
async fn test_credit_card_luhn_validation() {
    let vault = EgressVault::try_default().expect("valid vault");

    let valid_card = "4532015112830366";
    let res_valid = vault.classify(valid_card).await;
    assert!(
        matches!(res_valid, EgressClassification::Block(BlockReason::SensitivePattern(_))),
        "Credit card with valid Luhn checksum must be blocked"
    );

    let invalid_card = "1111111111111111";
    let res_invalid = vault.classify(invalid_card).await;
    assert_eq!(
        res_invalid,
        EgressClassification::Allow,
        "16-digit number with invalid Luhn checksum must be allowed"
    );
}

#[tokio::test]
async fn test_phone_numbers_blocked() {
    let vault = EgressVault::try_default().expect("valid vault");

    let phone_national = "Call 030 1234567 for help";
    let res_nat = vault.classify(phone_national).await;
    assert!(
        matches!(res_nat, EgressClassification::Block(BlockReason::SensitivePattern(_))),
        "National phone number must be blocked"
    );

    let phone_intl = "Contact +49 170 1234567";
    let res_intl = vault.classify(phone_intl).await;
    assert!(
        matches!(res_intl, EgressClassification::Block(BlockReason::SensitivePattern(_))),
        "International phone number must be blocked"
    );
}

#[tokio::test]
async fn test_ipv4_blocked() {
    let vault = EgressVault::try_default().expect("valid vault");

    let ipv4 = "Server IP is 192.168.1.1";
    let res = vault.classify(ipv4).await;
    assert!(
        matches!(res, EgressClassification::Block(BlockReason::SensitivePattern(_))),
        "IPv4 address must be blocked"
    );
}

#[tokio::test]
async fn test_delimited_credit_card_recognized_via_normalization() {
    let vault = EgressVault::try_default().expect("valid vault");

    let formatted_card = "Card number: 4532-0151-1283-0366";
    let res = vault.classify(formatted_card).await;
    assert!(
        matches!(res, EgressClassification::Block(BlockReason::SensitivePattern(_))),
        "Credit card with hyphen delimiters must be blocked via normalization"
    );

    let formatted_iban = "IBAN: DE89 3704 0044 0532 0130 00";
    let res_iban = vault.classify(formatted_iban).await;
    assert!(
        matches!(res_iban, EgressClassification::Block(BlockReason::SensitivePattern(_))),
        "IBAN with space delimiters must be blocked via normalization"
    );
}

#[test]
fn test_utility_functions_direct() {
    assert!(is_luhn_valid("4532015112830366"));
    assert!(!is_luhn_valid("1111111111111111"));

    assert!(is_iban_valid("DE89370400440532013000"));
    assert!(!is_iban_valid("DE00370400440532013000"));

    assert_eq!(normalize_payload("4532-0151-1283-0366"), "4532015112830366");
    assert_eq!(
        normalize_payload("DE89 3704 0044 0532 0130 00"),
        "DE89370400440532013000"
    );
}
