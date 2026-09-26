#![forbid(unsafe_code)]

use contextra_crypto::CryptoKey;
use contextra_kvcache::{
    KiviQuantizeConfig, KvSegment, KvSegmentContent, KvTensorView,
};
use contextra_types::{ContextraError, Result, TenantId};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct TrackingCipher {
    key: CryptoKey,
    seal_calls: Arc<AtomicUsize>,
    open_calls: Arc<AtomicUsize>,
}

impl TrackingCipher {
    fn new() -> Self {
        let key = CryptoKey::try_new("test-passphrase-12345", b"test-salt-67890").unwrap();
        Self {
            key,
            seal_calls: Arc::new(AtomicUsize::new(0)),
            open_calls: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl contextra_crypto::KvCipher for TrackingCipher {
    fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        self.seal_calls.fetch_add(1, Ordering::SeqCst);
        let (ct, nonce) = self
            .key.encrypt_auto_nonce(plaintext)
            .map_err(|e| ContextraError::Crypto(e.to_string()))?;
        let mut out = Vec::with_capacity(12 + ct.len());
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ct);
        Ok(out)
    }

    fn open(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        self.open_calls.fetch_add(1, Ordering::SeqCst);
        if ciphertext.len() < 12 {
            return Err(ContextraError::Crypto("Ciphertext too short".into()));
        }
        let nonce: &[u8; 12] = ciphertext[..12].try_into().unwrap();
        let ct = &ciphertext[12..];
        self.key
            .decrypt_auto_nonce(ct, nonce)
            .map_err(|e| ContextraError::Crypto(e.to_string()))
    }
}

struct FailingCipher;

impl contextra_crypto::KvCipher for FailingCipher {
    fn seal(&self, _plaintext: &[u8]) -> Result<Vec<u8>> {
        Err(ContextraError::Crypto("AEAD seal forced failure".into()))
    }

    fn open(&self, _ciphertext: &[u8]) -> Result<Vec<u8>> {
        Err(ContextraError::Crypto("AEAD open authentication failure".into()))
    }
}

#[test]
fn test_inv_kivi_aead_order_enforced_write_and_read() {
    let tenant = TenantId::try_new(101).unwrap();
    let mut segment = KvSegment::new(tenant, 1, vec![0u8; 16]);

    let raw = KvTensorView::new(
        vec![1.0, 2.0, 3.0, 4.0],
        vec![10.0, 20.0, 30.0, 40.0],
        2,
        2,
    )
    .unwrap();

    let cipher = TrackingCipher::new();
    let config = KiviQuantizeConfig {
        key_group_size: 16,
        quantize_values: true,
    };

    // 1. Write quantized: Quantization FIRST, AEAD Seal SECOND
    assert_eq!(cipher.seal_calls.load(Ordering::SeqCst), 0);
    segment.write_quantized(&raw, config, &cipher).unwrap();
    assert_eq!(cipher.seal_calls.load(Ordering::SeqCst), 1);

    // Segment data MUST now contain encrypted ciphertext, NOT raw or plain quantized floats
    assert_ne!(segment.as_bytes(), &vec![0u8; 16]);
    assert!(segment.encrypted);
    assert!(matches!(segment.content, KvSegmentContent::KiviQuantized(_)));

    // 2. Read dequantized: AEAD Open FIRST, Dequantization SECOND
    assert_eq!(cipher.open_calls.load(Ordering::SeqCst), 0);
    let reconstructed = segment.read_dequantized(&cipher).unwrap();
    assert_eq!(cipher.open_calls.load(Ordering::SeqCst), 1);

    assert_eq!(reconstructed.num_tokens, 2);
    assert_eq!(reconstructed.num_channels, 2);
    assert_eq!(reconstructed.keys.len(), 4);
    assert_eq!(reconstructed.values.len(), 4);
}

#[test]
fn test_inv_kivi_aead_order_enforced_decryption_failure_blocks_dequantization() {
    let tenant = TenantId::try_new(101).unwrap();
    let mut segment = KvSegment::new(tenant, 1, vec![0u8; 16]);

    let raw = KvTensorView::new(
        vec![1.0, 2.0, 3.0, 4.0],
        vec![10.0, 20.0, 30.0, 40.0],
        2,
        2,
    )
    .unwrap();

    let valid_cipher = TrackingCipher::new();
    let failing_cipher = FailingCipher;
    let config = KiviQuantizeConfig::default();

    segment.write_quantized(&raw, config, &valid_cipher).unwrap();

    // Reading with failing cipher MUST fail at step 1 (AEAD Open)
    let res = segment.read_dequantized(&failing_cipher);
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("AEAD open authentication failure"));
}
