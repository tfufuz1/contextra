use super::common::not_implemented;

pub fn run_heavy_command(cmd: &str, _args: &[String]) -> Result<(), String> {
    match cmd {
        "bench-compile"
        | "bench-download"
        | "bench-gate"
        | "bench-trend"
        | "check-bandit-latency-budget"
        | "check-coverage-gate"
        | "check-mutation-score-gate"
        | "mutation-score-record"
        | "check-recall-stability"
        | "loom-run"
        | "py-test"
        | "forensic-test"
        | "ann-sift1m"
        | "beir"
        | "onnx-test-model"
        | "run-community-detection"
        | "check-flatbuffers-drift"
        | "check-fbs-drift"
        | "regenerate-flatbuffers"
        | "migrate-docid-128"
        | "fuzz-smoke"
        | "flake-report"
        | "wal-replay-verify"
        | "mutants-diff" => not_implemented(cmd),
        _ => Err(format!("Unknown command in heavy cluster: {}", cmd)),
    }
}
