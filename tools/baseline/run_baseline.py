#!/usr/bin/env python3
"""
tools/baseline/run_baseline.py

Reproducible Baseline Runner for Contextra Workspace (35 members).
Produces uncut raw logs, json summaries, root cause attribution for compiler/clippy errors,
and test execution metrics with timeout tracking.

Only uses Python 3 standard library and bash tools.
"""

import argparse
import gzip
import json
import os
import pathlib
import re
import shutil
import signal
import subprocess
import sys
import time

REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent.parent


def normalize_path(text: str, repo_root: pathlib.Path = REPO_ROOT) -> str:
    """
    Normalize absolute paths in text to relative paths to ensure reproducible and clean logs.
    Handles /app/, /home/jules/..., and local repo_root.
    """
    if not text:
        return text

    resolved_root = str(repo_root.resolve())
    text = text.replace(resolved_root + "/", "")
    text = text.replace(resolved_root, ".")

    # Common container or CI absolute prefixes
    text = re.sub(r"/app/", "", text)
    text = re.sub(r"/home/[^/]+/[^/]+/", "", text)

    return text


def check_worktree_clean(repo_root: pathlib.Path = REPO_ROOT, allow_dirty: bool = False):
    """Verify git worktree has no uncommitted changes."""
    res = subprocess.run(
        ["git", "status", "--porcelain"],
        cwd=repo_root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if res.returncode != 0:
        raise RuntimeError(f"Failed to check git status: {res.stderr}")

    dirty_lines = [line for line in res.stdout.splitlines() if line.strip()]
    if dirty_lines and not allow_dirty:
        print("ERROR: Git worktree is not clean! Aborting baseline run.", file=sys.stderr, flush=True)
        for line in dirty_lines[:10]:
            print(f"  {line}", file=sys.stderr, flush=True)
        if len(dirty_lines) > 10:
            print(f"  ... and {len(dirty_lines) - 10} more files.", file=sys.stderr, flush=True)
        print("Commit or stash your changes before running run_baseline.py.", file=sys.stderr, flush=True)
        sys.exit(1)


def get_git_commit(repo_root: pathlib.Path = REPO_ROOT) -> str:
    """Get current HEAD git commit hash."""
    res = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=repo_root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=True,
    )
    return res.stdout.strip()


def get_toolchain_version() -> str:
    """Get rustc version string."""
    res = subprocess.run(
        ["rustc", "--version"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=True,
    )
    return res.stdout.strip()


def get_workspace_metadata(repo_root: pathlib.Path = REPO_ROOT) -> dict:
    """
    Runs cargo metadata and returns a mapping of package_name -> info
    and crate_path_map (mapping directory path relative to repo -> crate_name).
    """
    res = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=repo_root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=True,
    )
    data = json.loads(res.stdout)
    packages = {}
    path_to_crate = {}

    for pkg in data.get("packages", []):
        name = pkg["name"]
        manifest_path = pathlib.Path(pkg["manifest_path"])
        pkg_dir = manifest_path.parent
        rel_dir = normalize_path(str(pkg_dir), repo_root)

        pkg_info = {
            "name": name,
            "version": pkg["version"],
            "manifest_path": manifest_path,
            "dir": pkg_dir,
            "rel_dir": rel_dir,
        }
        packages[name] = pkg_info
        path_to_crate[rel_dir] = name

    return {
        "packages": packages,
        "path_to_crate": path_to_crate,
    }


def write_log_file(log_dir: pathlib.Path, command_slug: str, content: str, repo_root: pathlib.Path = REPO_ROOT) -> str:
    """
    Writes content to a log file in log_dir.
    Normalizes paths in content.
    If the content size is >= 2MB (2,000,000 bytes), compresses to .log.gz using gzip.
    Returns relative log path string.
    """
    log_dir.mkdir(parents=True, exist_ok=True)
    content = normalize_path(content, repo_root)
    content_bytes = content.encode("utf-8", errors="replace")

    log_path = log_dir / f"{command_slug}.log"
    gz_path = log_dir / f"{command_slug}.log.gz"

    GZIP_THRESHOLD_BYTES = 2 * 1024 * 1024  # 2 MB

    if len(content_bytes) >= GZIP_THRESHOLD_BYTES:
        if log_path.exists():
            log_path.unlink()
        with gzip.open(gz_path, "wb") as f:
            f.write(content_bytes)
        final_path = gz_path
    else:
        if gz_path.exists():
            gz_path.unlink()
        with open(log_path, "wb") as f:
            f.write(content_bytes)
        final_path = log_path

    return normalize_path(str(final_path), repo_root)


def find_owning_crate(file_path: str, packages: dict, repo_root: pathlib.Path = REPO_ROOT) -> str:
    """
    Finds the workspace crate that owns a given relative file path.
    e.g., 'crates/contextra-types/src/lib.rs' -> 'contextra-types'
    """
    norm_file = normalize_path(file_path, repo_root)
    # Check package directories
    best_match = None
    best_len = -1

    for pkg_name, pkg_info in packages.items():
        rel_dir = pkg_info.get("rel_dir", "")
        if not rel_dir:
            continue
        if norm_file == rel_dir or norm_file.startswith(rel_dir + "/"):
            if len(rel_dir) > best_len:
                best_len = len(rel_dir)
                best_match = pkg_name

    if best_match:
        return best_match

    return "external"


def parse_diagnostics_from_json_stream(raw_output: str, current_crate: str, packages: dict, repo_root: pathlib.Path = REPO_ROOT) -> tuple:
    """
    Parses compiler-message JSON objects from raw_output.
    Returns (diagnostics_list, first_root_cause_crate, caused_by_dependency).
    """
    diagnostics = []

    for line in raw_output.splitlines():
        line = line.strip()
        if not (line.startswith("{") and line.endswith("}")):
            continue
        try:
            msg_obj = json.loads(line)
        except Exception:
            continue

        if msg_obj.get("reason") != "compiler-message":
            continue

        msg = msg_obj.get("message", {})
        level = msg.get("level", "warning")
        code_obj = msg.get("code")
        lint_code = code_obj.get("code") if isinstance(code_obj, dict) else (code_obj or "compiler_error")
        rendered = normalize_path(msg.get("rendered", ""), repo_root)

        spans = msg.get("spans", [])
        primary_span = None
        for span in spans:
            if span.get("is_primary"):
                primary_span = span
                break
        if not primary_span and spans:
            primary_span = spans[0]

        file_name = primary_span.get("file_name", "") if primary_span else ""
        file_name_norm = normalize_path(file_name, repo_root)
        line_start = primary_span.get("line_start", 0) if primary_span else 0
        col_start = primary_span.get("column_start", 0) if primary_span else 0

        location = f"{file_name_norm}:{line_start}:{col_start}" if file_name_norm else "unknown"
        root_cause_crate = find_owning_crate(file_name_norm, packages, repo_root) if file_name_norm else current_crate

        diag_item = {
            "level": level,
            "lint": lint_code,
            "root_cause_crate": root_cause_crate,
            "victim_crate": current_crate,
            "location": location,
            "rendered": rendered,
        }
        diagnostics.append(diag_item)

    # Determine first_root_cause_crate from errors first, then warnings
    first_root_cause = None
    for d in diagnostics:
        if d["level"] == "error":
            first_root_cause = d["root_cause_crate"]
            break
    if not first_root_cause and diagnostics:
        first_root_cause = diagnostics[0]["root_cause_crate"]

    caused_by_dependency = False
    if first_root_cause and first_root_cause != current_crate and first_root_cause != "external":
        caused_by_dependency = True

    return diagnostics, first_root_cause, caused_by_dependency


def parse_test_results(raw_output: str) -> dict:
    """
    Parses test output from cargo test or nextest.
    Returns dict with tests_total, tests_passed, tests_failed, tests_ignored,
    slowest_tests, last_running_tests, compile_duration, test_duration.
    """
    tests_passed = 0
    tests_failed = 0
    tests_ignored = 0
    tests_total = 0

    started_tests = []
    finished_tests = set()
    slowest_tests = []  # list of {"name": name, "duration_seconds": float}

    # Match standard cargo test result line
    res_match = re.search(
        r"test result: \w+\.\s+(\d+)\s+passed;\s+(\d+)\s+failed;\s+(\d+)\s+ignored",
        raw_output,
    )
    if res_match:
        tests_passed += int(res_match.group(1))
        tests_failed += int(res_match.group(2))
        tests_ignored += int(res_match.group(3))
        tests_total = tests_passed + tests_failed + tests_ignored

    # Nextest summary match
    nextest_match = re.search(
        r"Summary\s+\[[\s\d\.]+\w+\]\s+(\d+)\s+tests run:\s+(\d+)\s+passed(?:,\s+(\d+)\s+failed)?(?:,\s+(\d+)\s+skipped)?",
        raw_output,
    )
    if nextest_match and tests_total == 0:
        tests_total = int(nextest_match.group(1))
        tests_passed = int(nextest_match.group(2))
        tests_failed = int(nextest_match.group(3)) if nextest_match.group(3) else 0
        tests_ignored = int(nextest_match.group(4)) if nextest_match.group(4) else 0

    # Parse individual test executions
    for line in raw_output.splitlines():
        line_s = line.strip()

        # Cargo test start
        start_m = re.match(r"^test\s+([^\s]+)\s+\.\.\.\s+running$", line_s)
        if start_m:
            t_name = start_m.group(1)
            started_tests.append(t_name)
            continue

        # Cargo test end
        end_m = re.match(r"^test\s+([^\s]+)\s+\.\.\.\s+(\w+)(?:\s+\(([\d\.]+)s\))?$", line_s)
        if end_m:
            t_name = end_m.group(1)
            status = end_m.group(2)
            dur_str = end_m.group(3)
            finished_tests.add(t_name)
            if t_name not in started_tests:
                started_tests.append(t_name)
            if dur_str:
                slowest_tests.append({"name": t_name, "duration_seconds": float(dur_str)})
            continue

        # Nextest start: "START [   0.001s] ( 1/55) contextra agent_memory::tests::test_foo"
        n_start_m = re.match(r"^START\s+\[[\s\d\.]+\w*\](?:\s+\([\s\d/]+\))?(?:\s+\S+)?\s+(\S+)$", line_s)
        if n_start_m:
            t_name = n_start_m.group(1)
            if t_name not in started_tests:
                started_tests.append(t_name)
            continue

        # Nextest end: "PASS [   0.012s] ( 1/55) contextra agent_memory::tests::test_foo"
        n_end_m = re.match(r"^(PASS|FAIL|CANCEL)\s+\[\s*([\d\.]+)s\](?:\s+\([\s\d/]+\))?(?:\s+\S+)?\s+(\S+)$", line_s)
        if n_end_m:
            dur = float(n_end_m.group(2))
            t_name = n_end_m.group(3)
            finished_tests.add(t_name)
            if t_name not in started_tests:
                started_tests.append(t_name)
            slowest_tests.append({"name": t_name, "duration_seconds": dur})
            continue

    # Unfinished tests (running when crashed/timed out)
    last_running = [t for t in started_tests if t not in finished_tests]

    # Sort slowest tests descending by duration, take top 20
    slowest_tests.sort(key=lambda x: x["duration_seconds"], reverse=True)
    top_slowest = slowest_tests[:20]

    return {
        "tests_total": tests_total,
        "tests_passed": tests_passed,
        "tests_failed": tests_failed,
        "tests_ignored": tests_ignored,
        "slowest_tests": top_slowest,
        "last_running_tests": last_running[-10:],
    }


def run_command_with_timeout(
    cmd: list,
    cwd: pathlib.Path,
    env: dict = None,
    timeout: int = 1800,
) -> tuple:
    """
    Runs process with timeout limit in seconds.
    Returns (exit_code, stdout, stderr, duration_seconds, timed_out).
    """
    full_env = os.environ.copy()
    if env:
        full_env.update(env)

    start_time = time.time()
    try:
        proc = subprocess.Popen(
            cmd,
            cwd=cwd,
            env=full_env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            preexec_fn=os.setsid if hasattr(os, "setsid") else None,
        )
        stdout_data, stderr_data = proc.communicate(timeout=timeout)
        duration = time.time() - start_time
        return proc.returncode, stdout_data, stderr_data, duration, False
    except subprocess.TimeoutExpired:
        duration = time.time() - start_time
        try:
            if hasattr(os, "killpg"):
                os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
            else:
                proc.kill()
        except Exception:
            pass
        stdout_data, stderr_data = proc.communicate()
        return 124, stdout_data or "", stderr_data or "", duration, True


def generate_root_causes_md(all_diagnostics: list, commit: str, toolchain: str, output_path: pathlib.Path) -> str:
    """
    Generates ROOT_CAUSES.md summarizing diagnostics grouped by (root_cause_crate, lint).
    """
    groups = {}

    for diag in all_diagnostics:
        if diag["level"] not in ("error", "warning"):
            continue
        key = (diag["root_cause_crate"], diag["lint"])
        if key not in groups:
            groups[key] = {
                "root_cause_crate": diag["root_cause_crate"],
                "lint": diag["lint"],
                "victims": set(),
                "first_location": diag["location"],
            }
        groups[key]["victims"].add(diag["victim_crate"])

    lines = [
        "# Wurzelursachen-Analyse (Baseline Root Causes)",
        "",
        f"- **Commit:** `{commit}`",
        f"- **Toolchain:** `{toolchain}`",
        f"- **Stand:** {time.strftime('%Y-%m-%d %H:%M:%S UTC', time.gmtime())}",
        "",
        "| Root Cause Crate | Lint / Fehlercode | Betroffene Crates | Fundstelle | Betroffene Crates Liste |",
        "|---|---|---|---|---|",
    ]

    sorted_keys = sorted(groups.keys(), key=lambda k: (-len(groups[k]["victims"]), k[0], k[1]))

    for k in sorted_keys:
        g = groups[k]
        victims_sorted = sorted(list(g["victims"]))
        victims_str = ", ".join(victims_sorted)
        lines.append(
            f"| `{g['root_cause_crate']}` | `{g['lint']}` | {len(victims_sorted)} | `{g['first_location']}` | {victims_str} |"
        )

    lines.append("")
    content = "\n".join(lines)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w", encoding="utf-8") as f:
        f.write(content)

    return content


def count_warnings(text: str) -> int:
    """Counts warning occurrences in cargo build/check/clippy output."""
    matches = re.findall(r"warning:", text)
    return len(matches)


def run_crate_baseline(
    pkg_name: str,
    pkg_info: dict,
    packages: dict,
    commit: str,
    toolchain: str,
    timeout: int = 1800,
    repo_root: pathlib.Path = REPO_ROOT,
) -> tuple:
    """
    Runs baseline checks for a single crate and writes evidence/baseline/<crate>.json.
    Returns (crate_summary_dict, crate_diagnostics_list).
    """
    log_dir = repo_root / "evidence" / "baseline" / "logs" / pkg_name
    log_dir.mkdir(parents=True, exist_ok=True)

    commands_info = []
    all_raw_diags = []

    total_start = time.time()
    compile_duration = 0.0
    test_duration = 0.0

    tests_total = 0
    tests_passed = 0
    tests_failed = 0
    tests_ignored = 0
    slowest_tests = []
    last_running_tests = []
    warnings_count = 0

    timed_out_any = False
    compile_error_any = False
    test_fail_any = False
    overall_exit_code = 0

    # Command definitions - use nextest with -j 1 if available
    has_nextest = shutil.which("cargo-nextest") is not None
    test_cmd = (
        ["cargo", "nextest", "run", "-p", pkg_name, "--no-fail-fast", "-j", "1", "--locked"]
        if has_nextest
        else ["cargo", "test", "-p", pkg_name, "--no-fail-fast", "--locked", "--", "--test-threads=1", "--no-capture"]
    )

    cmds = [
        ("cargo_build", ["cargo", "build", "-p", pkg_name, "--no-default-features", "--locked"]),
        ("cargo_check", ["cargo", "check", "-p", pkg_name, "--all-targets", "--locked", "--message-format=json"]),
        ("cargo_clippy", ["cargo", "clippy", "-p", pkg_name, "--all-targets", "--locked", "--message-format=json", "--", "-D", "warnings"]),
        ("cargo_test", test_cmd),
    ]

    if pkg_name == "contextra":
        cmds.append(
            ("cargo_test_deletion_chain_e2e", ["cargo", "test", "-p", "contextra", "--test", "deletion_chain_e2e", "--", "--test-threads=1", "--no-capture"])
        )

    for slug, cmd in cmds:
        env_vars = {"RUST_BACKTRACE": "1"}
        exit_code, stdout, stderr, dur, is_timeout = run_command_with_timeout(
            cmd, cwd=repo_root, env=env_vars, timeout=timeout
        )

        raw_combined = f"--- STDOUT ---\n{stdout}\n--- STDERR ---\n{stderr}"
        log_rel_path = write_log_file(log_dir, slug, raw_combined, repo_root)

        warnings_count += count_warnings(stdout) + count_warnings(stderr)

        if exit_code != 0 and overall_exit_code == 0:
            overall_exit_code = exit_code

        if is_timeout:
            timed_out_any = True

        if slug in ("cargo_build", "cargo_check", "cargo_clippy"):
            compile_duration += dur
            if exit_code != 0:
                compile_error_any = True

            # Parse diagnostics from JSON in check/clippy
            if slug in ("cargo_check", "cargo_clippy"):
                diags, _, _ = parse_diagnostics_from_json_stream(stdout + "\n" + stderr, pkg_name, packages, repo_root)
                all_raw_diags.extend(diags)

        elif "test" in slug:
            comp_time_part = 0.0
            test_time_part = dur

            if "Finished" in stderr or "Running" in stderr:
                finished_pos = stderr.find("Finished")
                if finished_pos != -1:
                    frac = finished_pos / len(stderr) if len(stderr) > 0 else 0.5
                    comp_time_part = dur * frac
                    test_time_part = dur * (1 - frac)

            compile_duration += comp_time_part
            test_duration += test_time_part

            if exit_code != 0:
                test_fail_any = True

            t_res = parse_test_results(stdout + "\n" + stderr)
            tests_total += t_res["tests_total"]
            tests_passed += t_res["tests_passed"]
            tests_failed += t_res["tests_failed"]
            tests_ignored += t_res["tests_ignored"]
            slowest_tests.extend(t_res["slowest_tests"])
            if t_res["last_running_tests"]:
                last_running_tests.extend(t_res["last_running_tests"])

        cmd_str = " ".join(cmd)
        commands_info.append({
            "command": cmd_str,
            "exit_code": exit_code,
            "duration_seconds": round(dur, 3),
            "stdout_snippet": stdout,
            "stderr_snippet": stderr,
        })

    # Sort slowest tests and keep top 20
    slowest_tests.sort(key=lambda x: x["duration_seconds"], reverse=True)
    top_slowest = slowest_tests[:20]

    # Determine status
    if timed_out_any:
        status = "timeout"
    elif compile_error_any:
        status = "compile_error"
    elif test_fail_any or overall_exit_code != 0:
        status = "fail"
    else:
        status = "pass"

    # Root cause attribution for this crate
    _, first_root_cause, caused_by_dep = parse_diagnostics_from_json_stream(
        "\n".join([c["stdout_snippet"] + "\n" + c["stderr_snippet"] for c in commands_info]),
        pkg_name,
        packages,
        repo_root,
    )

    total_duration = time.time() - total_start

    crate_json_data = {
        "crate": pkg_name,
        "commit": commit,
        "toolchain": toolchain,
        "status": status,
        "overall_exit_code": overall_exit_code,
        "duration_seconds": round(total_duration, 3),
        "compile_duration_seconds": round(compile_duration, 3),
        "test_duration_seconds": round(test_duration, 3),
        "tests_total": tests_total,
        "tests_passed": tests_passed,
        "tests_failed": tests_failed,
        "tests_ignored": tests_ignored,
        "warnings_count": warnings_count,
        "root_cause_crate": first_root_cause,
        "caused_by_dependency": caused_by_dep,
        "log_path": normalize_path(str(log_dir), repo_root),
        "last_running_tests": list(set(last_running_tests)),
        "slowest_tests": top_slowest,
        "commands": commands_info,
    }

    # Write evidence/baseline/<crate>.json
    crate_json_path = repo_root / "evidence" / "baseline" / f"{pkg_name}.json"
    crate_json_path.parent.mkdir(parents=True, exist_ok=True)
    with open(crate_json_path, "w", encoding="utf-8") as f:
        json.dump(crate_json_data, f, indent=2, ensure_ascii=False)

    summary_entry = {
        "crate": pkg_name,
        "status": status,
        "overall_exit_code": overall_exit_code,
        "duration_seconds": round(total_duration, 3),
        "tests_passed": tests_passed,
        "tests_failed": tests_failed,
        "tests_total": tests_total,
        "root_cause_crate": first_root_cause,
        "caused_by_dependency": caused_by_dep,
        "log_path": normalize_path(str(log_dir), repo_root),
    }

    return summary_entry, all_raw_diags


def run_self_tests():
    """Runs internal unit tests for run_baseline.py."""
    print("Running run_baseline.py self-tests...", flush=True)

    # 1. Test path normalization
    dummy_root = pathlib.Path("/app")
    sample_text = "Error in /app/crates/contextra-types/src/lib.rs:10"
    normalized = normalize_path(sample_text, dummy_root)
    assert normalized == "Error in crates/contextra-types/src/lib.rs:10", f"Got: {normalized}"

    sample_text2 = "Error in /home/jules/contextra/crates/contextra-crypto/src/lib.rs:5"
    normalized2 = normalize_path(sample_text2, REPO_ROOT)
    assert "crates/contextra-crypto/src/lib.rs:5" in normalized2, f"Got: {normalized2}"

    print("Path normalization self-tests passed!", flush=True)

    # 2. Test log file writer and gzip threshold
    test_log_dir = REPO_ROOT / "target" / "test_logs"
    if test_log_dir.exists():
        shutil.rmtree(test_log_dir)

    small_content = "This is a small log content.\n"
    small_path = write_log_file(test_log_dir, "cargo_build", small_content)
    assert small_path.endswith("cargo_build.log"), f"Expected .log, got: {small_path}"
    assert (REPO_ROOT / small_path).exists()

    large_content = "X" * (2 * 1024 * 1024 + 100)
    large_path = write_log_file(test_log_dir, "cargo_check", large_content)
    assert large_path.endswith("cargo_check.log.gz"), f"Expected .log.gz, got: {large_path}"
    assert (REPO_ROOT / large_path).exists()

    # Cleanup test_logs
    if test_log_dir.exists():
        shutil.rmtree(test_log_dir)

    print("Log file writer and gzip self-tests passed!", flush=True)

    # 3. Test diagnostic JSON parsing & root cause attribution
    mock_packages = {
        "contextra-types": {"rel_dir": "crates/contextra-types"},
        "contextra": {"rel_dir": "crates/contextra"},
    }

    mock_json_line = json.dumps({
        "reason": "compiler-message",
        "package_id": "contextra-types 0.1.0",
        "message": {
            "level": "error",
            "code": {"code": "clippy::cast_possible_truncation"},
            "rendered": "error: casting u64 to u32 may truncate",
            "spans": [
                {
                    "file_name": "/app/crates/contextra-types/src/types/domain/document.rs",
                    "line_start": 143,
                    "column_start": 35,
                    "is_primary": True,
                }
            ],
        },
    })

    diags, root_cause, caused_by_dep = parse_diagnostics_from_json_stream(
        mock_json_line, "contextra", mock_packages, dummy_root
    )
    assert len(diags) == 1
    assert diags[0]["root_cause_crate"] == "contextra-types"
    assert diags[0]["victim_crate"] == "contextra"
    assert root_cause == "contextra-types"
    assert caused_by_dep is True, "contextra was victim of contextra-types error"

    # Test ROOT_CAUSES.md generation
    md_path = REPO_ROOT / "target" / "test_ROOT_CAUSES.md"
    generate_root_causes_md(diags, "commit123", "rustc 1.89.0", md_path)
    assert md_path.exists()
    md_content = md_path.read_text(encoding="utf-8")
    assert "`contextra-types`" in md_content
    assert "`clippy::cast_possible_truncation`" in md_content
    if md_path.exists():
        md_path.unlink()

    print("Diagnostic JSON parsing & ROOT_CAUSES.md self-tests passed!", flush=True)

    # 4. Test test result parsing & timeout extraction
    mock_test_output = """
running 2 tests
test tests::test_pass ... ok (0.01s)
test tests::test_fail ... FAILED
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
"""
    t_parsed = parse_test_results(mock_test_output)
    assert t_parsed["tests_total"] == 2
    assert t_parsed["tests_passed"] == 1
    assert t_parsed["tests_failed"] == 1

    # Nextest line format test
    mock_nextest_output = """
    Starting 2 tests across 1 binary
        START [   0.001s] ( 1/2) mycrate test_running
        PASS [   0.012s] ( 1/2) mycrate test_pass
        FAIL [   0.045s] ( 2/2) mycrate test_fail
     Summary [   0.050s] 2 tests run: 1 passed, 1 failed
"""
    nt_parsed = parse_test_results(mock_nextest_output)
    assert nt_parsed["tests_total"] == 2
    assert nt_parsed["tests_passed"] == 1
    assert nt_parsed["tests_failed"] == 1
    assert len(nt_parsed["slowest_tests"]) == 2
    assert nt_parsed["slowest_tests"][0]["name"] == "test_fail"
    assert nt_parsed["last_running_tests"] == ["test_running"]

    print("Test output parser self-tests passed!", flush=True)


def main():
    parser = argparse.ArgumentParser(description="Contextra Baseline Execution Tool")
    parser.add_argument("--self-test", action="store_true", help="Run internal self-tests and exit")
    parser.add_argument("--timeout", type=int, default=1800, help="Per-crate timeout in seconds (default: 1800)")
    parser.add_argument("--crate", type=str, help="Run baseline only for a single crate")
    parser.add_argument("--allow-dirty", action="store_true", help="Allow running on dirty worktree")
    args = parser.parse_args()

    if args.self_test:
        run_self_tests()
        sys.exit(0)

    check_worktree_clean(REPO_ROOT, allow_dirty=args.allow_dirty)
    commit = get_git_commit(REPO_ROOT)
    toolchain = get_toolchain_version()
    metadata = get_workspace_metadata(REPO_ROOT)
    packages = metadata["packages"]

    print(f"Commit: {commit}", flush=True)
    print(f"Toolchain: {toolchain}", flush=True)
    print(f"Found {len(packages)} workspace packages.", flush=True)

    target_crates = sorted(list(packages.keys()))
    if args.crate:
        if args.crate not in packages:
            print(f"ERROR: Crate '{args.crate}' not found in workspace!", file=sys.stderr, flush=True)
            sys.exit(1)
        target_crates = [args.crate]

    crate_summaries = []
    all_workspace_diagnostics = []

    # Per-crate runs
    for idx, crate_name in enumerate(target_crates, 1):
        print(f"[{idx}/{len(target_crates)}] Running baseline for {crate_name}...", flush=True)
        pkg_info = packages[crate_name]
        summary_item, diags = run_crate_baseline(
            crate_name, pkg_info, packages, commit, toolchain, timeout=args.timeout, repo_root=REPO_ROOT
        )
        crate_summaries.append(summary_item)
        all_workspace_diagnostics.extend(diags)

    # Workspace-wide runs
    print("Running workspace-wide commands...", flush=True)
    workspace_log_dir = REPO_ROOT / "evidence" / "baseline" / "logs" / "workspace"
    workspace_runs = {}

    ws_commands = [
        ("cargo_check_workspace", ["cargo", "check", "--workspace", "--all-targets", "--locked"]),
        ("cargo_clippy_workspace", ["cargo", "clippy", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings"]),
        ("cargo_test_workspace", ["cargo", "test", "--workspace", "--no-fail-fast", "--locked"]),
        ("cargo_fmt_check", ["cargo", "fmt", "--check"]),
        ("cargo_deny_check", ["cargo", "deny", "check"]),
        ("cargo_audit", ["cargo", "audit"]),
    ]

    for slug, cmd in ws_commands:
        print(f"  Running workspace command: {' '.join(cmd)}", flush=True)
        exit_code, stdout, stderr, dur, is_timeout = run_command_with_timeout(
            cmd, cwd=REPO_ROOT, timeout=args.timeout
        )
        combined = f"--- STDOUT ---\n{stdout}\n--- STDERR ---\n{stderr}"
        log_path = write_log_file(workspace_log_dir, slug, combined, REPO_ROOT)

        status_str = "pass" if exit_code == 0 else ("timeout" if is_timeout else "fail")
        workspace_runs[slug] = {
            "status": status_str,
            "exit_code": exit_code,
            "duration_seconds": round(dur, 3),
            "log_path": log_path,
        }

    # Generate ROOT_CAUSES.md
    root_causes_path = REPO_ROOT / "evidence" / "baseline" / "ROOT_CAUSES.md"
    generate_root_causes_md(all_workspace_diagnostics, commit, toolchain, root_causes_path)
    print(f"Wrote {root_causes_path}", flush=True)

    # Counts
    crates_passed = sum(1 for c in crate_summaries if c["status"] == "pass")
    crates_failed = sum(1 for c in crate_summaries if c["status"] == "fail")
    crates_compile_error = sum(1 for c in crate_summaries if c["status"] == "compile_error")
    crates_timeout = sum(1 for c in crate_summaries if c["status"] == "timeout")

    summary_json_data = {
        "commit": commit,
        "toolchain": toolchain,
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "crates_total": len(crate_summaries),
        "crates_passed": crates_passed,
        "crates_failed": crates_failed,
        "crates_compile_error": crates_compile_error,
        "crates_timeout": crates_timeout,
        "crates": crate_summaries,
        "workspace_runs": workspace_runs,
    }

    summary_json_path = REPO_ROOT / "evidence" / "baseline" / "SUMMARY.json"
    with open(summary_json_path, "w", encoding="utf-8") as f:
        json.dump(summary_json_data, f, indent=2, ensure_ascii=False)

    print(f"Wrote {summary_json_path}", flush=True)
    print("Baseline run completed!", flush=True)


if __name__ == "__main__":
    main()
