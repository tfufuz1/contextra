use contextra_audit_export::{
    calculate_deadline_kpi, Art17_3Ground, AuditExportError, CivilDate, ErasureDecision,
    ErasureRequest, NonEmptyJustification, RetentionClass, RetentionMatrix, RetentionPeriod,
    TriggerEvent,
};
use contextra_ports::Clock;
use contextra_types::DeterministicTimestamp;

struct FixedClock {
    nanos: u64,
}

impl Clock for FixedClock {
    fn now(&self) -> DeterministicTimestamp {
        let wall_ms = self.nanos / 1_000_000;
        DeterministicTimestamp::new(wall_ms, 0)
    }

    fn monotonic_ticks(&self) -> u64 {
        0
    }

    fn now_unix_nanos(&self) -> u64 {
        self.nanos
    }
}

#[test]
fn test_deadline_calculation_month_end_and_extension_limits() {
    // 1. Entry on Jan 31, 2026 -> base deadline Feb 28, 2026 (non-leap year)
    let received = CivilDate::new(2026, 1, 31).expect("valid date");
    let req_base = ErasureRequest::try_new(
        "REQ-001",
        received,
        "scope_hash_1",
        false, // no extension notice sent
        None,
        0,
        None,
        None,
    )
    .expect("valid request");

    // Base deadline must be Feb 28, 2026
    // Anti-mirroring: independently constructed target civil date
    let expected_base_deadline = CivilDate {
        year: 2026,
        month: 2,
        day: 28,
    };
    assert_eq!(req_base.base_deadline(), expected_base_deadline);
    assert_eq!(req_base.effective_deadline(), expected_base_deadline);

    // 2. Extension notice sent with 2 additional months -> effective deadline Apr 30, 2026
    let ext_justification =
        NonEmptyJustification::try_new("Komplexität der Datenverarbeitung bei Drittanbietern")
            .unwrap();
    let req_extended = ErasureRequest::try_new(
        "REQ-002",
        received,
        "scope_hash_2",
        true, // extension notice sent
        Some(ext_justification.clone()),
        2, // max extension
        None,
        None,
    )
    .expect("valid extended request");

    let expected_extended_deadline = CivilDate {
        year: 2026,
        month: 4,
        day: 30,
    };
    assert_eq!(
        req_extended.effective_deadline(),
        expected_extended_deadline
    );

    // 3. Attempting > 2 extension months must fail with error
    let err_over_extended = ErasureRequest::try_new(
        "REQ-ERR-1",
        received,
        "scope_hash_err",
        true,
        Some(ext_justification.clone()),
        3, // invalid: > 2 months
        None,
        None,
    );
    assert!(matches!(
        err_over_extended,
        Err(AuditExportError::InvalidExtension(_))
    ));

    // 4. Claiming extended months without sending extension notice must fail
    let err_no_notice = ErasureRequest::try_new(
        "REQ-ERR-2",
        received,
        "scope_hash_err",
        false, // notice not sent
        Some(ext_justification),
        1,
        None,
        None,
    );
    assert!(matches!(
        err_no_notice,
        Err(AuditExportError::InvalidExtension(_))
    ));
}

#[test]
fn test_refusal_requires_non_empty_justification_and_ground() {
    // 1. Empty string or whitespace justification must fail
    let empty_res = NonEmptyJustification::try_new("   ");
    assert!(matches!(empty_res, Err(AuditExportError::EmptyJustification)));

    let empty_str_res = NonEmptyJustification::try_new("");
    assert!(matches!(
        empty_str_res,
        Err(AuditExportError::EmptyJustification)
    ));

    // 2. Valid refusal with justification and Art. 17(3) exception ground
    let valid_just = NonEmptyJustification::try_new(
        "Aufbewahrungspflicht nach § 147 AO für steuerlich relevante Unterlagen",
    )
    .unwrap();

    let refusal_decision = ErasureDecision::Refused {
        exception: Art17_3Ground::LegalObligation,
        justification: valid_just.clone(),
    };

    let received = CivilDate::new(2026, 2, 1).unwrap();
    let completed = CivilDate::new(2026, 2, 15).unwrap();

    let req = ErasureRequest::try_new(
        "REQ-REFUSED-1",
        received,
        "hash_refused",
        false,
        None,
        0,
        Some(refusal_decision),
        Some(completed),
    )
    .expect("valid refusal request");

    if let Some(ErasureDecision::Refused {
        exception,
        justification,
    }) = req.decision
    {
        assert_eq!(exception, Art17_3Ground::LegalObligation);
        assert!(!justification.as_str().trim().is_empty());
    } else {
        panic!("Expected refusal decision");
    }
}

#[test]
fn test_kpi_deadline_compliance_metrics() {
    // Clock set to 2026-03-15 UTC end of day
    let clock_date = CivilDate::new(2026, 3, 15).unwrap();
    let clock_nanos = clock_date.to_unix_seconds_end_of_day() * 1_000_000_000;
    let clock = FixedClock { nanos: clock_nanos };

    // 1. Empty set of requests -> 100% compliance default
    let empty_kpi = calculate_deadline_kpi(&[], &clock);
    assert_eq!(empty_kpi.total_requests, 0);
    assert_eq!(empty_kpi.completed_compliance_rate_percent, 100.0);
    assert_eq!(empty_kpi.overall_compliance_rate_percent, 100.0);

    // 2. Single completed request on time
    let received = CivilDate::new(2026, 1, 10).unwrap(); // deadline 2026-02-10
    let completed_on_time = CivilDate::new(2026, 2, 5).unwrap();
    let req_on_time = ErasureRequest::try_new(
        "REQ-KPI-1",
        received,
        "hash_kpi_1",
        false,
        None,
        0,
        Some(ErasureDecision::Granted),
        Some(completed_on_time),
    )
    .unwrap();

    let single_kpi = calculate_deadline_kpi(&[req_on_time.clone()], &clock);
    assert_eq!(single_kpi.total_requests, 1);
    assert_eq!(single_kpi.completed_count, 1);
    assert_eq!(single_kpi.completed_on_time_count, 1);
    assert_eq!(single_kpi.completed_overdue_count, 0);
    assert_eq!(single_kpi.completed_compliance_rate_percent, 100.0);

    // 3. Mixed batch: 1 completed on time, 1 completed overdue, 1 pending on time, 1 pending overdue
    // Overdue completed: received Jan 1, deadline Feb 1, completed Feb 15
    let req_completed_overdue = ErasureRequest::try_new(
        "REQ-KPI-2",
        CivilDate::new(2026, 1, 1).unwrap(),
        "hash_kpi_2",
        false,
        None,
        0,
        Some(ErasureDecision::Granted),
        Some(CivilDate::new(2026, 2, 15).unwrap()),
    )
    .unwrap();

    // Pending on time: received Feb 20, deadline Mar 20 (clock is Mar 15)
    let req_pending_on_time = ErasureRequest::try_new(
        "REQ-KPI-3",
        CivilDate::new(2026, 2, 20).unwrap(),
        "hash_kpi_3",
        false,
        None,
        0,
        None,
        None,
    )
    .unwrap();

    // Pending overdue: received Jan 10, deadline Feb 10 (clock is Mar 15)
    let req_pending_overdue = ErasureRequest::try_new(
        "REQ-KPI-4",
        CivilDate::new(2026, 1, 10).unwrap(),
        "hash_kpi_4",
        false,
        None,
        0,
        None,
        None,
    )
    .unwrap();

    let batch = vec![
        req_on_time,
        req_completed_overdue,
        req_pending_on_time,
        req_pending_overdue,
    ];
    let batch_kpi = calculate_deadline_kpi(&batch, &clock);

    assert_eq!(batch_kpi.total_requests, 4);
    assert_eq!(batch_kpi.completed_count, 2);
    assert_eq!(batch_kpi.completed_on_time_count, 1);
    assert_eq!(batch_kpi.completed_overdue_count, 1);
    assert_eq!(batch_kpi.pending_count, 2);
    assert_eq!(batch_kpi.pending_on_time_count, 1);
    assert_eq!(batch_kpi.pending_overdue_count, 1);
    assert_eq!(batch_kpi.completed_compliance_rate_percent, 50.0);
    assert_eq!(batch_kpi.overall_compliance_rate_percent, 50.0); // 2 on time out of 4 total
}

#[test]
fn test_retention_matrix_snapshot_output() {
    let rk1 = RetentionClass {
        class_id: "RK-101".to_string(),
        name: "Nutzerkonto- und Profilstammdaten".to_string(),
        data_categories: vec!["Name".to_string(), "E-Mail".to_string()],
        legal_basis_ref: "Art. 6 Abs. 1 lit. b DSGVO".to_string(),
        trigger_event: TriggerEvent::ContractTermination,
        retention_period: RetentionPeriod::Immediate,
        note: "Sofortige Anonymisierung / Crypto-Erase nach Konto-Schließung".to_string(),
    };

    let rk2 = RetentionClass {
        class_id: "RK-102".to_string(),
        name: "Transaktions- & Rechnungsbelege".to_string(),
        data_categories: vec!["Abrechnungsdaten".to_string(), "Zahlungshistorie".to_string()],
        legal_basis_ref: "§ 147 AO, § 257 HGB".to_string(),
        trigger_event: TriggerEvent::CalendarYearEnd,
        retention_period: RetentionPeriod::Years(10),
        note: "Gesetzliche Aufbewahrungspflicht für Buchungsbelege".to_string(),
    };

    let matrix = RetentionMatrix::new(vec![rk1, rk2]);

    let markdown = matrix.render_markdown();
    let expected_markdown_snapshot = r#"# Datenschutz-Löschkonzept & Retention-Matrix (Art. 30 / Art. 17 DSGVO)

> **Hinweis zur Normung:** Die Norm-Bezeichnung (DIN 66398 bzw. DIN EN ISO/IEC 27555:2025-09) ist noch zu verifizieren.
> **RECHTLICHER HINWEIS:** Vorlage, keine Rechtsberatung.

| Löschklasse-ID | Name / Zweck | Datenkategorien | Rechtsgrundlage | Startereignis | Regelfrist | Anmerkungen |
| --- | --- | --- | --- | --- | --- | --- |
| RK-101 | Nutzerkonto- und Profilstammdaten | Name, E-Mail | Art. 6 Abs. 1 lit. b DSGVO | Vertragsende / Konto-Schließung | Unverzüglich | Sofortige Anonymisierung / Crypto-Erase nach Konto-Schließung |
| RK-102 | Transaktions- & Rechnungsbelege | Abrechnungsdaten, Zahlungshistorie | § 147 AO, § 257 HGB | Ablauf des Kalenderjahres der Erfassung | 10 Jahre | Gesetzliche Aufbewahrungspflicht für Buchungsbelege |
"#;

    assert_eq!(markdown, expected_markdown_snapshot);

    let json = matrix.render_json().unwrap();
    assert!(json.contains("RK-101"));
    assert!(json.contains("RK-102"));
    assert!(json.contains("ContractTermination"));
}
