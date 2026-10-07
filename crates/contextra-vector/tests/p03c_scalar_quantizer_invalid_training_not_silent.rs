use contextra_core::ContextraError;
use contextra_vector::quantize::ScalarQuantizer;

#[test]
fn test_try_train_validates_nan_inf_and_dimension_mismatch() {
    let valid_vec = vec![1.0, 2.0, 3.0];
    let nan_vec = vec![1.0, f32::NAN, 3.0];
    let inf_vec = vec![1.0, f32::INFINITY, 3.0];
    let short_vec = vec![1.0, 2.0];

    // NaN in training batch -> Err
    let batch_nan = vec![valid_vec.as_slice(), nan_vec.as_slice()];
    let res_nan = ScalarQuantizer::try_train(&batch_nan, 3);
    assert!(res_nan.is_err());
    assert!(res_nan.unwrap_err().to_string().contains("NaN or infinite values"));

    // Infinity in training batch -> Err
    let batch_inf = vec![valid_vec.as_slice(), inf_vec.as_slice()];
    let res_inf = ScalarQuantizer::try_train(&batch_inf, 3);
    assert!(res_inf.is_err());
    assert!(res_inf.unwrap_err().to_string().contains("NaN or infinite values"));

    // Dimension mismatch in batch -> Err
    let batch_mismatch = vec![valid_vec.as_slice(), short_vec.as_slice()];
    let res_mismatch = ScalarQuantizer::try_train(&batch_mismatch, 3);
    assert!(res_mismatch.is_err());
    assert!(res_mismatch.unwrap_err().to_string().contains("expected 3"));
}

#[test]
fn test_train_on_invalid_input_returns_untrained_quantizer() {
    let valid_vec = vec![1.0, 2.0, 3.0];
    let nan_vec = vec![1.0, f32::NAN, 3.0];

    let batch = vec![valid_vec.as_slice(), nan_vec.as_slice()];

    // Call train (fallback mode)
    let sq = ScalarQuantizer::train(&batch, 3);

    // Is trained must report false
    assert!(!sq.is_trained(), "Quantizer trained on invalid batch must report is_trained() == false");

    // Operations must fail with ContextraError::InvalidInput
    let vec_to_quant = vec![1.0, 2.0, 3.0];
    let res_quant = sq.quantize(&vec_to_quant);
    assert!(
        matches!(res_quant, Err(ContextraError::InvalidInput(_))),
        "quantize on untrained quantizer must return InvalidInput error"
    );

    let dummy_quant = vec![0u8, 128u8, 255u8];
    let res_dequant = sq.dequantize(&dummy_quant);
    assert!(
        matches!(res_dequant, Err(ContextraError::InvalidInput(_))),
        "dequantize on untrained quantizer must return InvalidInput error"
    );

    let res_asym = sq.asymmetric_dist(&vec_to_quant, &dummy_quant, contextra_core::DistanceMetric::Euclidean);
    assert!(
        matches!(res_asym, Err(ContextraError::InvalidInput(_))),
        "asymmetric_dist on untrained quantizer must return InvalidInput error"
    );

    let res_sym = sq.symmetric_dist(&dummy_quant, &dummy_quant, contextra_core::DistanceMetric::Euclidean);
    assert!(
        matches!(res_sym, Err(ContextraError::InvalidInput(_))),
        "symmetric_dist on untrained quantizer must return InvalidInput error"
    );
}
