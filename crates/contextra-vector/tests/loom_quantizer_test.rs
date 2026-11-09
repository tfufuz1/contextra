// FILE-CONTEXT
// ZWECK: Concurrency & Unveränderlichkeitstest für Quantisierer unter parallelem Zugriff.

use contextra_vector::quantize::ScalarQuantizer;
use std::sync::Arc;
use std::thread;

#[test]
fn test_quantizer_concurrent_sharing() {
    let dim = 16;
    let v1 = vec![0.0f32; dim];
    let v2 = vec![10.0f32; dim];
    let quantizer = Arc::new(ScalarQuantizer::try_train(&[&v1, &v2], dim).unwrap());

    let mut handles = Vec::new();
    for thread_id in 0..8 {
        let q = Arc::clone(&quantizer);
        handles.push(thread::spawn(move || {
            let vec = vec![thread_id as f32; dim];
            for _ in 0..100 {
                let code = q.quantize(&vec).unwrap();
                let deq = q.dequantize(&code).unwrap();
                assert_eq!(deq.len(), dim);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}
