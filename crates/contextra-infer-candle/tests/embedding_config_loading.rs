use contextra_infer_candle::embedding::load_bert_config;
use contextra_infer_candle::model_registry::CandleQuantization;
use contextra_infer_candle::CandleEmbedClient;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_from_dir_missing_safetensors_returns_err() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let dir_path = dir.path().to_str().ok_or("invalid path string")?;
    let res = CandleEmbedClient::from_dir(dir.path(), CandleQuantization::Q4KM);
    assert!(
        res.is_err(),
        "from_dir must fail when model.safetensors is missing"
    );
    if let Err(err) = res {
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("model.safetensors") || err_msg.contains(dir_path),
            "Error message should mention model.safetensors or path: {err_msg}"
        );
    }
    Ok(())
}

#[test]
fn test_load_bert_config_valid_json() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config_path = dir.path().join("config.json");
    let json_content = r#"{
        "architectures": [
            "BertModel"
        ],
        "attention_probs_dropout_prob": 0.1,
        "hidden_act": "gelu",
        "hidden_dropout_prob": 0.1,
        "hidden_size": 384,
        "initializer_range": 0.02,
        "intermediate_size": 1536,
        "layer_norm_eps": 1e-12,
        "max_position_embeddings": 512,
        "model_type": "bert",
        "num_attention_heads": 6,
        "num_hidden_layers": 6,
        "pad_token_id": 0,
        "type_vocab_size": 2,
        "vocab_size": 30522,
        "use_cache": true
    }"#;
    let mut file = File::create(&config_path)?;
    file.write_all(json_content.as_bytes())?;

    let config = load_bert_config(&config_path)?;
    assert_eq!(config.hidden_size, 384);
    assert_eq!(config.vocab_size, 30522);
    assert_eq!(config.num_hidden_layers, 6);
    assert_eq!(config.num_attention_heads, 6);
    assert_eq!(config.intermediate_size, 1536);
    assert_eq!(config.layer_norm_eps, 1e-12);
    assert_eq!(config.initializer_range, 0.02);
    assert_eq!(config.model_type, Some("bert".to_string()));
    assert!(config.use_cache);
    Ok(())
}

#[test]
fn test_load_bert_config_broken_json_returns_err() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config_path = dir.path().join("config.json");
    let config_path_str = config_path.to_str().ok_or("invalid path string")?;
    let mut file = File::create(&config_path)?;
    file.write_all(b"{ broken json content: 123 }")?;

    let res = load_bert_config(&config_path);
    assert!(res.is_err(), "broken json must return Err");
    if let Err(err) = res {
        let err_msg = err.to_string();
        assert!(
            err_msg.contains(config_path_str) || err_msg.contains("config.json"),
            "Error message should contain path: {err_msg}"
        );
    }
    Ok(())
}

#[test]
fn test_load_bert_config_missing_file_returns_err_containing_path(
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let missing_path = dir.path().join("nonexistent_config.json");
    let missing_path_str = missing_path.to_str().ok_or("invalid path string")?;

    let res = load_bert_config(&missing_path);
    assert!(res.is_err(), "missing config file must return Err");
    if let Err(err) = res {
        let err_msg = err.to_string();
        assert!(
            err_msg.contains(missing_path_str),
            "Error message must contain missing file path. Got: {err_msg}"
        );
    }
    Ok(())
}
