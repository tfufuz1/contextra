// FILE-CONTEXT
// STAND: 2026-09-22T00:00:00Z
// ZWECK: Unit and integration tests for Stage B KvState and forked Llama KV cache integration.

#![cfg(feature = "kv-stage-b")]

use candle_core::{DType, Device, Tensor};
use contextra_types::ContextraError;
use contextra_infer_candle::kv_state::{KvState, LayerKv};

#[test]
fn test_kv_state_4d_model_tensor_export_import_truncation() -> Result<(), Box<dyn std::error::Error>>
{
    let device = Device::Cpu;
    // Model weights shape: (batch=1, n_kv_head=2, seq_len=32, head_dim=4)
    let k0 = Tensor::zeros((1, 2, 32, 4), DType::F32, &device)?;
    let v0 = Tensor::ones((1, 2, 32, 4), DType::F32, &device)?;

    let layers = vec![LayerKv::new(k0, v0)];
    let mut state = KvState::new(layers, 32);

    // Export range 0..16
    let block = state.export_block(0..16)?;
    let mut imported = KvState::new(vec![], 0);
    imported.import_block(&block)?;

    assert_eq!(imported.pos(), 16);
    assert_eq!(imported.layers[0].k.dims(), &[1, 2, 16, 4]);
    assert_eq!(imported.layers[0].v.dims(), &[1, 2, 16, 4]);

    // Truncate state to 10
    state.truncate(10)?;
    assert_eq!(state.pos(), 10);
    assert_eq!(state.layers[0].k.dims(), &[1, 2, 10, 4]);
    assert_eq!(state.layers[0].v.dims(), &[1, 2, 10, 4]);

    Ok(())
}

#[test]
fn test_kv_state_basic_properties() -> Result<(), Box<dyn std::error::Error>> {
    let device = Device::Cpu;
    let k0 = Tensor::zeros((1, 32, 4, 64), DType::F32, &device)?;
    let v0 = Tensor::zeros((1, 32, 4, 64), DType::F32, &device)?;
    let k1 = Tensor::ones((1, 32, 4, 64), DType::F32, &device)?;
    let v1 = Tensor::ones((1, 32, 4, 64), DType::F32, &device)?;

    let layers = vec![LayerKv::new(k0, v0), LayerKv::new(k1, v1)];
    let state = KvState::new(layers, 32);

    assert_eq!(state.layer_count(), 2);
    assert_eq!(state.pos(), 32);
    assert!(!state.is_empty());

    Ok(())
}

#[test]
fn test_kv_state_export_import_roundtrip_f32() -> Result<(), Box<dyn std::error::Error>> {
    let device = Device::Cpu;
    let k_data: Vec<f32> = (0..128).map(|i| i as f32).collect();
    let v_data: Vec<f32> = (0..128).map(|i| (i * 2) as f32).collect();

    let k0 = Tensor::from_vec(k_data.clone(), (1, 16, 2, 4), &device)?;
    let v0 = Tensor::from_vec(v_data.clone(), (1, 16, 2, 4), &device)?;

    let layers = vec![LayerKv::new(k0, v0)];
    let state = KvState::new(layers, 16);

    // Export range 0..8 (8 tokens)
    let block = state.export_block(0..8)?;
    assert_eq!(block.block_id, 0);
    assert!(!block.data.is_empty());

    // Import into fresh empty KvState
    let mut imported_state = KvState::new(vec![], 0);
    imported_state.import_block(&block)?;

    assert_eq!(imported_state.layer_count(), 1);
    assert_eq!(imported_state.pos(), 8);

    // Verify imported tensor contents
    let imp_k = &imported_state.layers[0].k;
    let imp_k_vec = imp_k.flatten_all()?.to_vec1::<f32>()?;
    assert_eq!(imp_k_vec.len(), 64); // 1 * 8 * 2 * 4
    assert_eq!(imp_k_vec[0..8], k_data[0..8]);

    Ok(())
}

#[test]
fn test_kv_state_export_import_roundtrip_f16() -> Result<(), Box<dyn std::error::Error>> {
    let device = Device::Cpu;
    let k0 = Tensor::zeros((1, 16, 2, 4), DType::F16, &device)?;
    let v0 = Tensor::ones((1, 16, 2, 4), DType::F16, &device)?;

    let layers = vec![LayerKv::new(k0, v0)];
    let state = KvState::new(layers, 16);

    let block = state.export_block(4..12)?;
    let mut imported_state = KvState::new(vec![], 0);
    imported_state.import_block(&block)?;

    assert_eq!(imported_state.layer_count(), 1);
    assert_eq!(imported_state.pos(), 8);
    assert_eq!(imported_state.layers[0].k.dtype(), DType::F16);

    Ok(())
}

#[test]
fn test_kv_state_truncation() -> Result<(), Box<dyn std::error::Error>> {
    let device = Device::Cpu;
    let k0 = Tensor::zeros((1, 32, 2, 4), DType::F32, &device)?;
    let v0 = Tensor::zeros((1, 32, 2, 4), DType::F32, &device)?;

    let layers = vec![LayerKv::new(k0, v0)];
    let mut state = KvState::new(layers, 32);

    // Truncate to 16 tokens
    state.truncate(16)?;
    assert_eq!(state.pos(), 16);
    assert_eq!(state.layers[0].k.dims()[1], 16);
    assert_eq!(state.layers[0].v.dims()[1], 16);

    // Truncate to 0 tokens
    state.truncate(0)?;
    assert_eq!(state.pos(), 0);
    assert_eq!(state.layers[0].k.dims()[1], 0);

    // Truncating to a larger position is a no-op
    state.truncate(10)?;
    assert_eq!(state.pos(), 0);

    Ok(())
}

#[test]
fn test_kv_state_out_of_bounds_errors() -> Result<(), Box<dyn std::error::Error>> {
    let device = Device::Cpu;
    let k0 = Tensor::zeros((1, 10, 2, 4), DType::F32, &device)?;
    let v0 = Tensor::zeros((1, 10, 2, 4), DType::F32, &device)?;

    let layers = vec![LayerKv::new(k0, v0)];
    let state = KvState::new(layers, 10);

    // Exporting past pos should fail cleanly with InvalidInput
    let err = state.export_block(0..20);
    assert!(matches!(err, Err(ContextraError::InvalidInput(_))));

    Ok(())
}

#[test]
fn test_kv_state_import_block_at() -> Result<(), Box<dyn std::error::Error>> {
    let device = Device::Cpu;
    let k0 = Tensor::zeros((1, 16, 2, 4), DType::F32, &device)?;
    let v0 = Tensor::zeros((1, 16, 2, 4), DType::F32, &device)?;

    let layers = vec![LayerKv::new(k0, v0)];
    let mut state = KvState::new(layers, 16);

    let block = state.export_block(0..8)?;

    // Import at truncated position 8
    state.import_block_at(&block, 8)?;
    assert_eq!(state.pos(), 16); // 8 + 8 = 16

    // Importing past current pos should fail
    let err = state.import_block_at(&block, 25);
    assert!(matches!(err, Err(ContextraError::InvalidInput(_))));

    Ok(())
}
