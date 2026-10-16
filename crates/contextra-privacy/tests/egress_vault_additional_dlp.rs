#![forbid(unsafe_code)]

use contextra_privacy::egress_vault::{
    BlockReason, EgressClassification, EgressClassifier, EgressVault,
};

#[tokio::test]
async fn test_valid_iban_blocked_and_invalid_checksum_allowed() {
    let vault = EgressVault::try_default().expect("valid vault");

    // Valid German IBAN with correct ISO 7064 MOD 97-10 checksum
    let valid_iban = "DE89370400440532013000";
    let res_valid = vault.classify(valid_iban).await;
    assert!(
        matches!(res_valid, EgressClassification::Block(BlockReason::SensitivePattern(_))),
        "Valid IBAN must be blocked"
    );

    // Invalid IBAN (checksum digit changed from 0 to 1 -> fails MOD 97-10)
    let invalid_iban = "DE89370400440532013001";
    let res_invalid = vault.classify(invalid_iban).await;
    assert_eq!(
        res_invalid,
        EgressClassification::Allow,
        "Invalid IBAN checksum must NOT be blocked (reduces false positives)"
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
        "PEM private key block must be blocked"
    );
}

#[tokio::test]
async fn test_credit_card_luhn_validation_and_delimiters() {
    let vault = EgressVault::try_default().expect("valid vault");

    // Valid credit card with correct Luhn checksum (4532015112830366)
    let valid_card = "4532015112830366";
    let res_valid = vault.classify(valid_card).await;
    assert!(
        matches!(res_valid, EgressClassification::Block(BlockReason::SensitivePattern(_))),
        "Valid credit card with Luhn checksum must be blocked"
    );

    // Valid credit card with delimiters (dashes & spaces) -> recognized via normalization
    let delimited_card_dashes = "4532-0151-1283-0366";
    let res_delimited_dashes = vault.classify(delimited_card_dashes).await;
    assert!(
        matches!(res_delimited_dashes, EgressClassification::Block(BlockReason::SensitivePattern(_))),
        "Delimited credit card number must be recognized and blocked via normalization"
    );

    let delimited_card_spaces = "4532 0151 1283 0366";
    let res_delimited_spaces = vault.classify(delimited_card_spaces).await;
    assert!(
        matches!(res_delimited_spaces, EgressClassification::Block(BlockReason::SensitivePattern(_))),
        "Spaced credit card number must be recognized and blocked via normalization"
    );

    // 16-digit number with invalid Luhn checksum -> NOT blocked
    let invalid_card = "1234567890123456";
    let res_invalid = vault.classify(invalid_card).await;
    assert_eq!(
        res_invalid,
        EgressClassification::Allow,
        "16-digit number with invalid Luhn checksum must NOT be blocked"
    );
}

#[tokio::test]
async fn test_phone_numbers_blocked() {
    let vault = EgressVault::try_default().expect("valid vault");

    let national_phone = "030 1234567";
    let res_nat = vault.classify(national_phone).await;
    assert!(
        matches!(res_nat, EgressClassification::Block(BlockReason::SensitivePattern(_))),
        "National phone number must be blocked"
    );

    let intl_phone = "+49 170 1234567";
    let res_intl = vault.classify(intl_phone).await;
    assert!(
        matches!(res_intl, EgressClassification::Block(BlockReason::SensitivePattern(_))),
        "International E.164 phone number must be blocked"
    );
}

#[tokio::test]
async fn test_ipv4_address_blocked() {
    let vault = EgressVault::try_default().expect("valid vault");

    let ipv4 = "192.168.1.1";
    let res = vault.classify(ipv4).await;
    assert!(
        matches!(res, EgressClassification::Block(BlockReason::SensitivePattern(_))),
        "IPv4 address must be blocked"
    );
}
