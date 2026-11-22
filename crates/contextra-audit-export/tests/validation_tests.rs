//! Mandatory field validation tests for GDPR Article 30 requirements.

use contextra_audit_export::{
    testkit::InMemoryProcessingRegisterSource, AuditExportError, ProcessingRegisterEntry,
    ProcessingRegisterSource, RetentionPeriod, ThirdCountryTransfer,
};
use contextra_types::TenantId;

fn valid_sample_entry() -> Result<ProcessingRegisterEntry, Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(1)?;
    let source = InMemoryProcessingRegisterSource::with_sample_data_for(tenant_id);
    let mut entries = source.collect_entries(tenant_id)?;
    entries.pop().ok_or_else(|| "No sample entry found".into())
}

#[test]
fn test_validation_valid_entry_passes() -> Result<(), Box<dyn std::error::Error>> {
    let entry = valid_sample_entry()?;
    assert!(entry.validate().is_ok());
    Ok(())
}

#[test]
fn test_validation_missing_controller_name() -> Result<(), Box<dyn std::error::Error>> {
    let mut entry = valid_sample_entry()?;
    entry.controller_contact.controller_name = "   ".to_string();
    let Err(err) = entry.validate() else {
        return Err("Expected validation error".into());
    };
    match err {
        AuditExportError::MissingField { field } => {
            assert_eq!(field, "controller_contact.controller_name");
            Ok(())
        }
        _ => Err("Expected MissingField error".into()),
    }
}

#[test]
fn test_validation_missing_controller_contact() -> Result<(), Box<dyn std::error::Error>> {
    let mut entry = valid_sample_entry()?;
    entry.controller_contact.controller_contact = "".to_string();
    let Err(err) = entry.validate() else {
        return Err("Expected validation error".into());
    };
    match err {
        AuditExportError::MissingField { field } => {
            assert_eq!(field, "controller_contact.controller_contact");
            Ok(())
        }
        _ => Err("Expected MissingField error".into()),
    }
}

#[test]
fn test_validation_missing_processing_purpose() -> Result<(), Box<dyn std::error::Error>> {
    let mut entry = valid_sample_entry()?;
    entry.processing_purpose = "  ".to_string();
    let Err(err) = entry.validate() else {
        return Err("Expected validation error".into());
    };
    match err {
        AuditExportError::MissingField { field } => {
            assert_eq!(field, "processing_purpose");
            Ok(())
        }
        _ => Err("Expected MissingField error".into()),
    }
}

#[test]
fn test_validation_missing_legal_basis() -> Result<(), Box<dyn std::error::Error>> {
    let mut entry = valid_sample_entry()?;
    entry.legal_basis = "".to_string();
    let Err(err) = entry.validate() else {
        return Err("Expected validation error".into());
    };
    match err {
        AuditExportError::MissingField { field } => {
            assert_eq!(field, "legal_basis");
            Ok(())
        }
        _ => Err("Expected MissingField error".into()),
    }
}

#[test]
fn test_validation_missing_data_subject_categories() -> Result<(), Box<dyn std::error::Error>> {
    let mut entry = valid_sample_entry()?;
    entry.data_subject_categories = vec![];
    let Err(err) = entry.validate() else {
        return Err("Expected validation error".into());
    };
    match err {
        AuditExportError::MissingField { field } => {
            assert_eq!(field, "data_subject_categories");
            Ok(())
        }
        _ => Err("Expected MissingField error".into()),
    }
}

#[test]
fn test_validation_missing_data_categories() -> Result<(), Box<dyn std::error::Error>> {
    let mut entry = valid_sample_entry()?;
    entry.data_categories = vec!["  ".to_string()];
    let Err(err) = entry.validate() else {
        return Err("Expected validation error".into());
    };
    match err {
        AuditExportError::MissingField { field } => {
            assert_eq!(field, "data_categories");
            Ok(())
        }
        _ => Err("Expected MissingField error".into()),
    }
}

#[test]
fn test_validation_missing_recipient_categories() -> Result<(), Box<dyn std::error::Error>> {
    let mut entry = valid_sample_entry()?;
    entry.recipient_categories = vec![];
    let Err(err) = entry.validate() else {
        return Err("Expected validation error".into());
    };
    match err {
        AuditExportError::MissingField { field } => {
            assert_eq!(field, "recipient_categories");
            Ok(())
        }
        _ => Err("Expected MissingField error".into()),
    }
}

#[test]
fn test_validation_missing_retention_periods() -> Result<(), Box<dyn std::error::Error>> {
    let mut entry = valid_sample_entry()?;
    entry.retention_periods = vec![];
    let Err(err) = entry.validate() else {
        return Err("Expected validation error".into());
    };
    match err {
        AuditExportError::MissingField { field } => {
            assert_eq!(field, "retention_periods");
            Ok(())
        }
        _ => Err("Expected MissingField error".into()),
    }
}

#[test]
fn test_validation_missing_retention_period_fields() -> Result<(), Box<dyn std::error::Error>> {
    let mut entry = valid_sample_entry()?;
    entry.retention_periods = vec![RetentionPeriod {
        data_category: "Kategorie".to_string(),
        retention_period: "".to_string(),
    }];
    let Err(err) = entry.validate() else {
        return Err("Expected validation error".into());
    };
    match err {
        AuditExportError::MissingField { field } => {
            assert_eq!(field, "retention_period.retention_period");
            Ok(())
        }
        _ => Err("Expected MissingField error".into()),
    }
}

#[test]
fn test_validation_missing_toms() -> Result<(), Box<dyn std::error::Error>> {
    let mut entry = valid_sample_entry()?;
    entry.technical_organizational_measures = vec![];
    let Err(err) = entry.validate() else {
        return Err("Expected validation error".into());
    };
    match err {
        AuditExportError::MissingField { field } => {
            assert_eq!(field, "technical_organizational_measures");
            Ok(())
        }
        _ => Err("Expected MissingField error".into()),
    }
}

#[test]
fn test_validation_missing_third_country_fields() -> Result<(), Box<dyn std::error::Error>> {
    let mut entry = valid_sample_entry()?;
    entry.third_country_transfers = vec![ThirdCountryTransfer {
        destination_country: "USA".to_string(),
        safeguard_guarantee: "".to_string(),
    }];
    let Err(err) = entry.validate() else {
        return Err("Expected validation error".into());
    };
    match err {
        AuditExportError::MissingField { field } => {
            assert_eq!(field, "third_country_transfer.safeguard_guarantee");
            Ok(())
        }
        _ => Err("Expected MissingField error".into()),
    }
}
