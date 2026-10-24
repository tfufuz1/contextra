use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dataset {
    AnnSift1m,
    Beir,
    OnnxTestModel,
    All,
}

pub const SIFT1M_URL: &str = "http://ann-benchmarks.com/sift-128-euclidean.hdf5";
pub const BEIR_NFCORPUS_URL: &str =
    "https://public.ukp.informatik.tu-darmstadt.de/thakur/BEIR/datasets/nfcorpus.zip";
pub const ONNX_MODEL_URL: &str =
    "https://huggingface.co/hf-internal-testing/tiny-random-BertModel/resolve/main/onnx/model.onnx";
pub const ONNX_TOKENIZER_URL: &str =
    "https://huggingface.co/hf-internal-testing/tiny-random-BertModel/resolve/main/tokenizer.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadTarget {
    pub url: String,
    pub target_path: PathBuf,
}

/// Computes the list of download targets for a given dataset choice and workspace root.
pub fn get_dataset_targets(dataset: Dataset, root: &Path) -> Vec<DownloadTarget> {
    let bench_data_dir = root.join("benchmarks").join("data");
    let onnx_fixtures_dir = root
        .join("crates")
        .join("contextra-infer-onnx")
        .join("tests")
        .join("fixtures");

    let mut targets = Vec::new();

    match dataset {
        Dataset::AnnSift1m => {
            targets.push(DownloadTarget {
                url: SIFT1M_URL.to_string(),
                target_path: bench_data_dir.join("sift-128-euclidean.hdf5"),
            });
        }
        Dataset::Beir => {
            targets.push(DownloadTarget {
                url: BEIR_NFCORPUS_URL.to_string(),
                target_path: bench_data_dir.join("nfcorpus.zip"),
            });
        }
        Dataset::OnnxTestModel => {
            targets.push(DownloadTarget {
                url: ONNX_MODEL_URL.to_string(),
                target_path: onnx_fixtures_dir.join("model.onnx"),
            });
            targets.push(DownloadTarget {
                url: ONNX_TOKENIZER_URL.to_string(),
                target_path: onnx_fixtures_dir.join("tokenizer.json"),
            });
        }
        Dataset::All => {
            targets.extend(get_dataset_targets(Dataset::AnnSift1m, root));
            targets.extend(get_dataset_targets(Dataset::Beir, root));
            targets.extend(get_dataset_targets(Dataset::OnnxTestModel, root));
        }
    }

    targets
}

/// Helper to build curl command arguments for downloading a target file.
pub fn build_curl_args(url: &str, target_path: &Path) -> Vec<String> {
    vec![
        "-fSL".to_string(),
        "-o".to_string(),
        target_path.to_string_lossy().to_string(),
        url.to_string(),
    ]
}

/// Downloads benchmark datasets / test model fixtures using curl.
pub fn run_bench_download(dataset: Dataset, root: &Path) -> Result<(), String> {
    let targets = get_dataset_targets(dataset, root);

    for target in targets {
        if target.target_path.exists() {
            println!(
                "✅ Target file already exists, skipping download: {}",
                target.target_path.display()
            );
            continue;
        }

        if let Some(parent) = target.target_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                return Err(format!(
                    "Failed to create directory '{}': {e}",
                    parent.display()
                ));
            }
        }

        println!(
            "📥 Downloading dataset from {} to {}...",
            target.url,
            target.target_path.display()
        );

        let args = build_curl_args(&target.url, &target.target_path);
        let mut cmd = Command::new("curl");
        cmd.args(&args);

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute curl command: {e}"))?;

        if !output.status.success() {
            let stderr_str = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "curl failed downloading '{}' to '{}': {stderr_str}",
                target.url,
                target.target_path.display()
            ));
        }

        println!("✅ Download completed: {}", target.target_path.display());
    }

    Ok(())
}
