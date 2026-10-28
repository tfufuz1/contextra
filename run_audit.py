import subprocess
import time
import json
import re
import os

crates = [
    "contextra", "contextra-adapt", "contextra-agent", "contextra-audit-export",
    "contextra-avv-generator", "contextra-bench", "contextra-checkpoint",
    "contextra-cognition", "contextra-core", "contextra-crypto", "contextra-db",
    "contextra-engine", "contextra-graph", "contextra-infer-candle",
    "contextra-infer-ollama", "contextra-infer-onnx", "contextra-kvcache",
    "contextra-license", "contextra-mcp", "contextra-mvcc", "contextra-ports",
    "contextra-privacy", "contextra-py", "contextra-rank", "contextra-router",
    "contextra-sandbox", "contextra-simd", "contextra-store", "contextra-sys",
    "contextra-testkit", "contextra-text", "contextra-types", "contextra-vector",
    "contextra-wire", "xtask"
]

results = {}
os.makedirs("evidence/baseline-2026-10-28/per_crate", exist_ok=True)

for crate in crates:
    print(f"=== Processing {crate} ===", flush=True)
    crate_res = {}
    
    # Check
    log_file = f"evidence/baseline-2026-10-28/per_crate/{crate}_check.log"
    t0 = time.time()
    res = subprocess.run(["cargo", "check", "-p", crate, "--locked"], capture_output=True, text=True)
    t1 = time.time()
    with open(log_file, "w") as f:
        f.write(res.stdout + "\n" + res.stderr)
    crate_res["check"] = {"exit_code": res.returncode, "duration_s": round(t1 - t0, 2)}
    
    # Clippy
    log_file = f"evidence/baseline-2026-10-28/per_crate/{crate}_clippy.log"
    t0 = time.time()
    res = subprocess.run(["cargo", "clippy", "-p", crate, "--locked", "--", "-D", "warnings"], capture_output=True, text=True)
    t1 = time.time()
    with open(log_file, "w") as f:
        f.write(res.stdout + "\n" + res.stderr)
    crate_res["clippy"] = {"exit_code": res.returncode, "duration_s": round(t1 - t0, 2)}

    # Test
    log_file = f"evidence/baseline-2026-10-28/per_crate/{crate}_test.log"
    t0 = time.time()
    res = subprocess.run(["cargo", "test", "-p", crate, "--locked", "--no-fail-fast"], capture_output=True, text=True)
    t1 = time.time()
    output = res.stdout + "\n" + res.stderr
    with open(log_file, "w") as f:
        f.write(output)
        
    # parse test results
    passed = 0
    failed = 0
    ignored = 0
    for line in output.splitlines():
        m = re.search(r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", line)
        if m:
            passed += int(m.group(2))
            failed += int(m.group(3))
            ignored += int(m.group(4))
            
    crate_res["test"] = {
        "exit_code": res.returncode,
        "duration_s": round(t1 - t0, 2),
        "passed": passed,
        "failed": failed,
        "ignored": ignored
    }
    
    results[crate] = crate_res
    print(f"Finished {crate}: check={crate_res['check']['exit_code']}, clippy={crate_res['clippy']['exit_code']}, test={crate_res['test']['exit_code']} (passed={passed}, failed={failed})", flush=True)

with open("evidence/baseline-2026-10-28/per_crate_summary.json", "w") as f:
    json.dump(results, f, indent=2)

print("Finished all crates!", flush=True)
