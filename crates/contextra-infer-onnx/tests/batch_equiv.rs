//! Test zur Äquivalenz von Batch-Inferenz und Einzel-Inferenz.

use contextra_infer_onnx::traits::Embedder;
use contextra_infer_onnx::FakeEmbedder;

#[test]
fn test_batch_equivalence_fake_embedder() {
    let embedder = FakeEmbedder::new(64).unwrap();
    let texts = vec![
        "erster testtext",
        "zweiter text für batch äquivalenz",
        "dritter langer text mit mehr inhalt",
    ];

    let batch_res = embedder.embed_batch(&texts).unwrap();
    assert_eq!(batch_res.len(), texts.len());

    for (i, &t) in texts.iter().enumerate() {
        let single_res = embedder.embed(t).unwrap();
        assert_eq!(batch_res[i], single_res);
    }
}
