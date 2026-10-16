use contextra_cognition::memory_consolidation::{
    intake_pre_condensed_inputs, PreCondensedInput, MAX_PRE_CONDENSED_DIGEST_LEN,
    MAX_PRE_CONDENSED_LABEL_LEN, MAX_PRE_CONDENSED_PROVENANCE_COUNT,
};
use contextra_ports::ResponseGroundingValidator;
use contextra_types::{ContextraError, DocId, Result, TenantId};

struct TestMockValidator {
    score: f32,
}

impl ResponseGroundingValidator for TestMockValidator {
    fn score_grounding(&self, _response: &str, _sources: &[&str]) -> Result<f32> {
        Ok(self.score)
    }
}

#[test]
fn test_pre_condensed_input_tenant_boundary_veto_f10() {
    let target_tenant = TenantId::try_new(101).unwrap();
    let wrong_tenant = TenantId::try_new(202).unwrap();

    let input_valid = PreCondensedInput {
        tenant_id: target_tenant,
        source_label: "scratchpad_checkpoint_1".to_string(),
        text_digest: "Pre-condensed context summary text A".to_string(),
        token_count: 25,
        provenance: vec![DocId::new(10), DocId::new(11)],
    };

    let input_wrong = PreCondensedInput {
        tenant_id: wrong_tenant,
        source_label: "scratchpad_checkpoint_2".to_string(),
        text_digest: "Pre-condensed context summary text B".to_string(),
        token_count: 30,
        provenance: vec![DocId::new(20)],
    };

    let res = intake_pre_condensed_inputs(&target_tenant, &[input_valid, input_wrong], None, None);

    assert!(
        res.is_err(),
        "Tenant mismatch must return PolicyViolation error under VETO-F10"
    );
    match res.unwrap_err() {
        ContextraError::PolicyViolation(msg) => {
            assert!(
                msg.contains("VETO-F10"),
                "Expected VETO-F10 message, got: {}",
                msg
            );
        }
        err => panic!("Expected PolicyViolation, got {:?}", err),
    }
}

#[test]
fn test_pre_condensed_input_provenance_retention() {
    let tenant = TenantId::try_new(101).unwrap();

    let prov1 = vec![DocId::new(100), DocId::new(101)];
    let prov2 = vec![DocId::new(200)];

    let input1 = PreCondensedInput {
        tenant_id: tenant,
        source_label: "src_1".to_string(),
        text_digest: "Digest 1".to_string(),
        token_count: 15,
        provenance: prov1.clone(),
    };

    let input2 = PreCondensedInput {
        tenant_id: tenant,
        source_label: "src_2".to_string(),
        text_digest: "Digest 2".to_string(),
        token_count: 20,
        provenance: prov2.clone(),
    };

    let segments = intake_pre_condensed_inputs(&tenant, &[input1, input2], None, None)
        .expect("intake should succeed");

    assert_eq!(segments.len(), 2);
    assert_eq!(
        segments[0].turn_ids, prov1,
        "Input 1 provenance must be preserved in turn_ids"
    );
    assert_eq!(
        segments[1].turn_ids, prov2,
        "Input 2 provenance must be preserved in turn_ids"
    );
}

#[test]
fn test_pre_condensed_input_length_and_bound_limits() {
    let tenant = TenantId::try_new(101).unwrap();

    // 1. Digest too long
    let overlong_digest = "x".repeat(MAX_PRE_CONDENSED_DIGEST_LEN + 1);
    let input_overlong_digest = PreCondensedInput {
        tenant_id: tenant,
        source_label: "label".to_string(),
        text_digest: overlong_digest,
        token_count: 10,
        provenance: vec![DocId::new(1)],
    };
    let res1 = intake_pre_condensed_inputs(&tenant, &[input_overlong_digest], None, None);
    assert!(res1.is_err());
    assert!(matches!(res1.unwrap_err(), ContextraError::InvalidInput(_)));

    // 2. Label too long
    let overlong_label = "y".repeat(MAX_PRE_CONDENSED_LABEL_LEN + 1);
    let input_overlong_label = PreCondensedInput {
        tenant_id: tenant,
        source_label: overlong_label,
        text_digest: "digest".to_string(),
        token_count: 10,
        provenance: vec![DocId::new(1)],
    };
    let res2 = intake_pre_condensed_inputs(&tenant, &[input_overlong_label], None, None);
    assert!(res2.is_err());
    assert!(matches!(res2.unwrap_err(), ContextraError::InvalidInput(_)));

    // 3. Provenance count too high
    let overlong_prov: Vec<DocId> = (0..(MAX_PRE_CONDENSED_PROVENANCE_COUNT + 1))
        .map(|i| DocId::new(i as u64))
        .collect();
    let input_overlong_prov = PreCondensedInput {
        tenant_id: tenant,
        source_label: "label".to_string(),
        text_digest: "digest".to_string(),
        token_count: 10,
        provenance: overlong_prov,
    };
    let res3 = intake_pre_condensed_inputs(&tenant, &[input_overlong_prov], None, None);
    assert!(res3.is_err());
    assert!(matches!(res3.unwrap_err(), ContextraError::InvalidInput(_)));
}

#[test]
fn test_pre_condensed_input_deterministic_sorting() {
    let tenant = TenantId::try_new(101).unwrap();

    let input_a = PreCondensedInput {
        tenant_id: tenant,
        source_label: "lbl_a".to_string(),
        text_digest: "AAA Digest".to_string(),
        token_count: 10,
        provenance: vec![DocId::new(1)],
    };

    let input_b = PreCondensedInput {
        tenant_id: tenant,
        source_label: "lbl_b".to_string(),
        text_digest: "BBB Digest".to_string(),
        token_count: 10,
        provenance: vec![DocId::new(2)],
    };

    let input_c = PreCondensedInput {
        tenant_id: tenant,
        source_label: "lbl_c".to_string(),
        text_digest: "CCC Digest".to_string(),
        token_count: 10,
        provenance: vec![DocId::new(3)],
    };

    // Order 1: C, A, B
    let segs1 = intake_pre_condensed_inputs(
        &tenant,
        &[input_c.clone(), input_a.clone(), input_b.clone()],
        None,
        None,
    )
    .unwrap();

    // Order 2: B, C, A
    let segs2 =
        intake_pre_condensed_inputs(&tenant, &[input_b, input_c, input_a], None, None).unwrap();

    assert_eq!(
        segs1, segs2,
        "Intake order must be strictly deterministic regardless of input slice ordering"
    );
    assert_eq!(
        segs1[0].turn_ids,
        vec![DocId::new(1)],
        "AAA Digest must be first"
    );
    assert_eq!(
        segs1[1].turn_ids,
        vec![DocId::new(2)],
        "BBB Digest must be second"
    );
    assert_eq!(
        segs1[2].turn_ids,
        vec![DocId::new(3)],
        "CCC Digest must be third"
    );
}

#[test]
fn test_pre_condensed_input_grounding_rejection() {
    let tenant = TenantId::try_new(101).unwrap();

    let input = PreCondensedInput {
        tenant_id: tenant,
        source_label: "source_evidence".to_string(),
        text_digest: "Ungrounded or low confidence claims".to_string(),
        token_count: 20,
        provenance: vec![DocId::new(50)],
    };

    // 1. Failing grounding validator (score 0.30 < threshold 0.70)
    let val_failing = TestMockValidator { score: 0.30 };
    let segs_failing =
        intake_pre_condensed_inputs(&tenant, &[input.clone()], Some(&val_failing), Some(0.70))
            .unwrap();

    assert_eq!(
        segs_failing.len(),
        0,
        "Low grounding score must reject input"
    );

    // 2. Passing grounding validator (score 0.85 >= threshold 0.70)
    let val_passing = TestMockValidator { score: 0.85 };
    let segs_passing =
        intake_pre_condensed_inputs(&tenant, &[input.clone()], Some(&val_passing), Some(0.70))
            .unwrap();

    assert_eq!(
        segs_passing.len(),
        1,
        "Passing grounding score must accept input"
    );

    // 3. Missing validator when threshold is set
    let segs_missing_val =
        intake_pre_condensed_inputs(&tenant, &[input], None, Some(0.70)).unwrap();

    assert_eq!(
        segs_missing_val.len(),
        0,
        "Missing validator when threshold is active must reject ungrounded input"
    );
}
