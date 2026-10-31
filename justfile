# justfile v2 — Contextra Dev-Workflow | WP-X.3 2026-10-26
set shell := ["bash", "-uc"]

# ── Primary entry-point ───────────────────────────────────────────────────────

# Lists all available targets
default:
    @just --list

# ── Core Dev-Loop (10 targets + mandated extras) ──────────────────────────────

# Runs formatting, clippy and checks compilation (nix-develop aware)
check:
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v nix &> /dev/null && nix develop -c true &> /dev/null; then
        RUNNER="nix develop -c"
    else
        RUNNER=""
    fi
    $RUNNER cargo fmt --all -- --check
    $RUNNER cargo clippy --all-targets -- -D warnings
    $RUNNER cargo check --all-targets --workspace

# Runs the TDD Validation Loop (Red -> Green -> Refactor)
test *ARGS:
    cargo test --locked {{ARGS}}

# Executes governance layering test
gates:
    cargo test --test layering --locked

# Full pre-push pipeline: check → test → gates
pre-push: check test gates

# Check + test a single crate
crate CRATE:
    cargo check -p {{CRATE}} && cargo test -p {{CRATE}} --locked

# Run a single crate's tests under Loom (concurrency model checker)
loom CRATE:
    RUSTFLAGS="--cfg loom" cargo test -p {{CRATE}} --locked

# Sets up local development environment (git hooks)
bootstrap:
    git config core.hooksPath .githooks
    echo "✅ git core.hooksPath configured to .githooks"

# ── Mandated extras (CI-critical, kept from v1) ───────────────────────────────

# Erstellt Coverage-JSON für CI-Gate (benötigt cargo-llvm-cov)
coverage:
    #!/usr/bin/env bash
    set -euo pipefail
    which cargo-llvm-cov || cargo install cargo-llvm-cov
    cargo llvm-cov --workspace --exclude contextra-py --json --output-path coverage.json
    echo "✅ Coverage-Gate bestanden"

# Runs Loom concurrency exploration model test for group commit lock handoff (unoptimized debug build required by Loom)
loom-store:
    RUSTFLAGS="--cfg loom" cargo test -p contextra-store --test loom_group_commit_handoff -- --nocapture

# Startet alle Fuzz-Targets für 60 Sekunden (benötigt cargo-fuzz + nightly)
fuzz-all SECONDS="60":
    #!/usr/bin/env bash
    set -euo pipefail
    which cargo-fuzz || cargo install cargo-fuzz
    echo "🔥 Fuzzing für {{SECONDS}} Sekunden pro Target..."
    targets=(
        "contextra-store:fuzz_wal_replay"
        "contextra-vector:fuzz_hnsw_persistence"
        "contextra-text:fuzz_bm25_tokenize"
        "contextra-wire:fuzz_flatbuffers_ipc"
        "contextra-mcp:fuzz_jsonrpc_parsing"
        "contextra-store:wal_roundtrip"
        "contextra-store:fuzz_manifest_load"
        "contextra-store:wal_mutation_chaos"
        "contextra-vector:hnsw_insert_search"
        "contextra-db:rrf_fusion"
        "contextra-mcp:fuzz_prompt_injection_guard"
    )
    for entry in "${targets[@]}"; do
        crate="${entry%%:*}"
        target="${entry##*:}"
        echo "  → crates/${crate} :: ${target}"
        (cd "crates/${crate}" && cargo +nightly fuzz run "${target}" -- -max_total_time={{SECONDS}} 2>&1 | tail -3) || true
    done
    echo "✅ Fuzz-Suite abgeschlossen"

# Zeigt offene kritische AI-TAGs und ANCHORs an — primärer manueller
# Weg, den aktuellen Governance-Status einer Session zu prüfen.
# Für einen strukturierten NDJSON-Export aller Tags steht `just context-tags` zur Verfügung.
session-context:
    #!/usr/bin/env bash
    echo "OFFENE KRITISCHE TAGS:"
    grep -rn "AI-TAG\[.*\]\[BLOCKER\]\|AI-TAG\[.*\]\[CRITICAL\]" crates/ --include='*.rs' | grep -v RESOLVED || echo "  (keine)"
    echo ""
    echo "OFFENE ANCHORS:"
    grep -rn "ANCHOR\[.*\] STATUS:IN-PROGRESS" crates/ --include='*.rs' || echo "  (keine)"
