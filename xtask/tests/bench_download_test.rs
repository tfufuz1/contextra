#[path = "../src/bench_download.rs"]
mod bench_download;

use std::path::Path;

#[test]
fn test_get_dataset_targets() {
    let root = Path::new("/workspace");

    let sift_targets =
        bench_download::get_dataset_targets(bench_download::Dataset::AnnSift1m, root);
    assert_eq!(sift_targets.len(), 1);
    assert_eq!(sift_targets[0].url, bench_download::SIFT1M_URL);
    assert_eq!(
        sift_targets[0].target_path,
        root.join("benchmarks/data/sift-128-euclidean.hdf5")
    );

    let beir_targets = bench_download::get_dataset_targets(bench_download::Dataset::Beir, root);
    assert_eq!(beir_targets.len(), 1);
    assert_eq!(beir_targets[0].url, bench_download::BEIR_NFCORPUS_URL);
    assert_eq!(
        beir_targets[0].target_path,
        root.join("benchmarks/data/nfcorpus.zip")
    );

    let onnx_targets =
        bench_download::get_dataset_targets(bench_download::Dataset::OnnxTestModel, root);
    assert_eq!(onnx_targets.len(), 2);
    assert_eq!(onnx_targets[0].url, bench_download::ONNX_MODEL_URL);
    assert_eq!(
        onnx_targets[0].target_path,
        root.join("crates/contextra-infer-onnx/tests/fixtures/model.onnx")
    );
    assert_eq!(onnx_targets[1].url, bench_download::ONNX_TOKENIZER_URL);
    assert_eq!(
        onnx_targets[1].target_path,
        root.join("crates/contextra-infer-onnx/tests/fixtures/tokenizer.json")
    );

    let all_targets = bench_download::get_dataset_targets(bench_download::Dataset::All, root);
    assert_eq!(all_targets.len(), 4);
}

#[test]
fn test_build_curl_args() {
    let target = Path::new("/workspace/benchmarks/data/sift-128-euclidean.hdf5");
    let args = bench_download::build_curl_args(bench_download::SIFT1M_URL, target);

    assert_eq!(
        args,
        vec![
            "-fSL",
            "-o",
            "/workspace/benchmarks/data/sift-128-euclidean.hdf5",
            "http://ann-benchmarks.com/sift-128-euclidean.hdf5"
        ]
    );
}
