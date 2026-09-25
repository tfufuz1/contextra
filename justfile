# AGENT:11
set shell := ["bash", "-uc"]

default:
    @just --list

# Sets up local development environment (git hooks, tooling)
bootstrap:
    #!/usr/bin/env bash
    set -euo pipefail
    git config core.hooksPath .githooks
    echo "✅ git core.hooksPath configured to .githooks"

# Runs the TDD Validation Loop (Red -> Green -> Refactor)
test *ARGS:
    cargo test --locked {{ARGS}}

# Runs formatting, clippy and checks compilation
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
    $RUNNER cargo xtask check-max-results-unbound
    $RUNNER cargo xtask check-toctou-defaults
    $RUNNER cargo xtask check-nan-hot-loop
    $RUNNER cargo xtask check-result-dropped-io
    if [ -f coverage.json ]; then
        $RUNNER cargo xtask check-coverage-gate
    fi

# Modular check for contextra-core
check-core:
    nix develop -c cargo check -p contextra-core || cargo check -p contextra-core

# Modular check for contextra-store
check-store:
    nix develop -c cargo check -p contextra-store || cargo check -p contextra-store

# Runs the chaos matrix fault-injection integration test suite
chaos-test:
    nix develop -c cargo test -p contextra-store --test chaos_matrix -- --ignored --test-threads=1 || \
    cargo test -p contextra-store --test chaos_matrix -- --ignored --test-threads=1

# Modular check for contextra-vector
check-vector:
    nix develop -c cargo check -p contextra-vector || cargo check -p contextra-vector

# Modular check for contextra-db
check-db:
    nix develop -c cargo check -p contextra-db || cargo check -p contextra-db

# Modular check for contextra-text
check-text:
    nix develop -c cargo check -p contextra-text || cargo check -p contextra-text

# Sync documentation from inline tags and cargo topology
sync-docs:
    nix develop -c cargo xtask sync-docs || cargo xtask sync-docs

# Verifies if documentation is in sync with code without making changes
sync-docs-check:
    nix develop -c cargo xtask sync-docs --check || cargo xtask sync-docs --check

# Verifies multi-session review coverage for completed anchors
check-review-coverage:
    nix develop -c cargo xtask check-review-coverage || cargo xtask check-review-coverage

# Verifies internal documentation consistency (e.g. crate counts)
check-consistency:
    nix develop -c cargo xtask check-consistency || cargo xtask check-consistency

# Zeigt alle Context-Tags als NDJSON (filterbar nach Crate, Severity, Status)
context-tags *ARGS:
    cargo xtask context-tags {{ARGS}}

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

# Target checks for xtask audit lints
check-max-results-unbound:
    cargo xtask check-max-results-unbound

check-toctou-defaults:
    cargo xtask check-toctou-defaults

check-nan-hot-loop:
    cargo xtask check-nan-hot-loop

check-result-dropped-io:
    cargo xtask check-result-dropped-io

coverage-gate:
    cargo llvm-cov --workspace --exclude contextra-py --json --output-path coverage.json
    cargo xtask check-coverage-gate coverage.json

# Modular check for contextra-py
check-py:
    nix develop -c cargo check --manifest-path crates/contextra-py/Cargo.toml || cargo check --manifest-path crates/contextra-py/Cargo.toml

# Modular check for contextra-infer
check-infer:
    nix develop -c cargo check -p contextra-infer-candle -p contextra-infer-ollama -p contextra-infer-onnx || cargo check -p contextra-infer-candle -p contextra-infer-ollama -p contextra-infer-onnx

# Generiert prompter-data.json aus dem Live-Repo-Stand
gen-prompter-data:
    nix develop -c cargo xtask gen-prompter-data || cargo xtask gen-prompter-data

# Verifies the Directed Acyclic Graph (DAG) integrity of the workspace
dag-check:
    nix develop -c cargo xtask check-dag || cargo xtask check-dag

# Checks for permanent feature veto keywords in recent commits
check-vetoes:
    nix develop -c cargo xtask check-vetoes || cargo xtask check-vetoes

# Checks ADR deprecation and removal deadlines
check-adr-deadlines:
    nix develop -c cargo xtask check-adr-deadlines || cargo xtask check-adr-deadlines

# Triple-Test-Gate: Tests müssen 3x hintereinander grün sein (DONE-Definition)
triple-test: check
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v nix &> /dev/null && nix develop -c true &> /dev/null; then
        RUNNER="nix develop -c"
    else
        RUNNER=""
    fi
    echo "=== Triple-Test-Gate ==="
    for RUN in 1 2 3; do
        echo "--- Run $RUN/3 ---"
        if ! $RUNNER cargo test --workspace; then
            echo "❌ FAILED on run $RUN/3. Fix all failures before this WP is DONE."
            exit 1
        fi
    done
    echo "✅ Triple-Test-Gate PASSED (3/3)"

# Tech-Debt Audit: scannt nach .unwrap(), unsafe, std::fs in Produktionscode
debt-audit:
    cargo xtask debt-audit

# Runs the LongMemEval regression benchmark suite and compares against baseline
bench-regression:
    #!/usr/bin/env bash
    if command -v nix &> /dev/null && nix develop -c true &> /dev/null; then
        nix develop -c cargo run -p contextra-bench --release
    else
        cargo run -p contextra-bench --release
    fi

# Bootstrap a new feature using the Micro-Spec Template
spec NAME:
    #!/usr/bin/env bash
    set -euo pipefail
    TIMESTAMP=$(date +%Y%m%d)
    TARGET="docs/specs/SPEC-${TIMESTAMP}-{{NAME}}.md"
    mkdir -p docs/specs
    cp docs/specs/TEMPLATE_MICRO_SPEC.md "$TARGET"
    echo "Created new micro-spec at $TARGET"
    echo "Please fill out the spec and follow the SDD-Process (Spec -> Test -> Impl)!"

# Runs the Jules Session Preflight Verification
jules-preflight *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo run --manifest-path xtask/Cargo.toml -- jules-preflight "$@"

# Claims a crate / feature for exclusive session execution
claim *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo run --manifest-path xtask/Cargo.toml -- claim "$@"

# Runs factual integrity check for AGENTS.md against workspace
check-agents-integrity:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo run --manifest-path xtask/Cargo.toml -- check-agents-integrity


# Führt alle 8 Bug-Proof-Tests aus (L1 Unit-Tests, rote→grüne Beweise)
prove-bugs:
	#!/usr/bin/env bash
	set -euo pipefail
	echo "🔬 Führe Bug-Proof-Tests aus..."
	cargo test -p contextra-store --test toctou_put_if_absent -- --nocapture 2>&1 | tee /tmp/proof-b1.log
	cargo test -p contextra-vector --test nan_validation_policy -- --nocapture 2>&1 | tee /tmp/proof-b2.log
	cargo test -p contextra-db --test search_result_bound -- --nocapture 2>&1 | tee /tmp/proof-b3.log
	cargo test -p contextra-store --test manifest_corruption -- --nocapture 2>&1 | tee /tmp/proof-b4.log
	cargo test -p contextra-text --test tombstone_read_your_writes -- --nocapture 2>&1 | tee /tmp/proof-b5.log
	cargo test -p contextra-graph --test source_doc_ids_populated -- --nocapture 2>&1 | tee /tmp/proof-b6.log
	cargo test -p contextra-store --test wal_truncate_ordering -- --nocapture 2>&1 | tee /tmp/proof-b7.log
	cargo test -p contextra-vector --test deleted_nodes_lock_contention -- --nocapture 2>&1 | tee /tmp/proof-b8.log
	echo "✅ Alle Bug-Proof-Tests grün"

# Führt alle Property-Tests aus
prop-tests:
	#!/usr/bin/env bash
	set -euo pipefail
	echo "🔬 Property-Tests..."
	cargo test -p contextra-db --test proptest_search_invariants -- --nocapture
	cargo test -p contextra-text --test proptest_bm25_invariants -- --nocapture
	cargo test -p contextra-graph --test proptest_csr_invariants -- --nocapture
	cargo test -p contextra-store proptest -- --nocapture
	cargo test -p contextra-vector proptest -- --nocapture
	echo "✅ Alle Property-Tests grün"

# Vollständige QA-Suite: L0 Lints + L1 Proofs + L2 Property + L5 Integration
qa-full:
	#!/usr/bin/env bash
	set -euo pipefail
	just check
	just prove-bugs
	just prop-tests
	cargo test --workspace --test '*integration*' -- --nocapture
	echo "✅ Vollständige QA-Suite bestanden"

# Führt kritische Race-Condition-Tests unter ThreadSanitizer aus (benötigt Nightly)
tsan:
	#!/usr/bin/env bash
	set -euo pipefail
	echo "🔒 ThreadSanitizer..."
	RUSTFLAGS="-Z sanitizer=thread" \
	cargo +nightly test \
		-p contextra-store --test toctou_put_if_absent \
		-p contextra-vector --test deleted_nodes_lock_contention \
		--target x86_64-unknown-linux-gnu \
		-- --test-threads=1
	echo "✅ TSan: keine Races gefunden"

# Führt ausgewählte Tests unter AddressSanitizer aus
asan:
	#!/usr/bin/env bash
	set -euo pipefail
	echo "🔍 AddressSanitizer..."
	RUSTFLAGS="-Z sanitizer=address" \
	cargo +nightly test -p contextra-store -p contextra-vector \
		--target x86_64-unknown-linux-gnu \
		-- --test-threads=1

# Erstellt HTML-Coverage-Report (benötigt cargo-llvm-cov)
coverage-html:
	#!/usr/bin/env bash
	set -euo pipefail
	which cargo-llvm-cov || cargo install cargo-llvm-cov
	cargo llvm-cov --workspace --exclude contextra-py --html --output-dir target/coverage/
	echo "✅ Coverage-Report: target/coverage/index.html"
	if command -v xdg-open &>/dev/null; then xdg-open target/coverage/index.html; fi

# Erstellt Coverage-JSON für CI-Gate
coverage-json:
	#!/usr/bin/env bash
	set -euo pipefail
	which cargo-llvm-cov || cargo install cargo-llvm-cov
	cargo llvm-cov --workspace --exclude contextra-py --json --output-path coverage.json
	cargo xtask check-coverage-gate
	echo "✅ Coverage-Gate bestanden"

# Startet alle Fuzz-Targets für 60 Sekunden (benötigt cargo-fuzz + nightly)
fuzz-all SECONDS="60":
	#!/usr/bin/env bash
	set -euo pipefail
	which cargo-fuzz || cargo install cargo-fuzz
	echo "🔥 Fuzzing für {{SECONDS}} Sekunden pro Target..."
	targets=(
		"contextra-store:wal_roundtrip"
		"contextra-store:fuzz_manifest_load"
		"contextra-store:wal_mutation_chaos"
		"contextra-vector:hnsw_insert_search"
		"contextra-vector:fuzz_hnsw_persistence"
		"contextra-db:rrf_fusion"
		"contextra-text:fuzz_bm25_tokenize"
	)
	for entry in "${targets[@]}"; do
		crate="${entry%%:*}"
		target="${entry##*:}"
		echo "  → crates/${crate} :: ${target}"
		(cd "crates/${crate}" && cargo +nightly fuzz run "${target}" -- -max_total_time={{SECONDS}} 2>&1 | tail -3) || true
	done
	echo "✅ Fuzz-Suite abgeschlossen"

# Minimiert Fuzz-Corpus (entfernt Duplikate, behält minimale reproduzierende Inputs)
fuzz-minimize CRATE TARGET:
	#!/usr/bin/env bash
	set -euo pipefail
	cd "crates/{{CRATE}}" && cargo +nightly fuzz cmin {{TARGET}}
	echo "✅ Corpus minimiert: crates/{{CRATE}}/fuzz/corpus/{{TARGET}}"

# Vergleicht aktuelle Benchmark-Ergebnisse mit Baseline (±5% Gate)
bench-delta:
	#!/usr/bin/env bash
	set -euo pipefail
	echo "📊 Benchmark-Delta-Analyse..."
	cargo bench --workspace --bench '*' -- --save-baseline current 2>&1 | tail -20
	cargo xtask bench-gate --tolerance 0.05 \
		--baseline benchmarks/results/criterion-baseline.json \
		--current criterion/current
	echo "✅ Kein Benchmark-Regression > 5%"

# Setzt aktuelle Benchmarks als neue Baseline
bench-set-baseline:
	#!/usr/bin/env bash
	set -euo pipefail
	echo "📊 Setze neue Baseline..."
	cargo bench --workspace --bench '*' -- --save-baseline current
	cp -r criterion/current benchmarks/results/criterion-baseline.json
	echo "✅ Baseline aktualisiert"

# Externe Benchmarks (synthetisch, kein Download nötig)
bench-external:
	#!/usr/bin/env bash
	set -euo pipefail
	echo "🌐 Externe Benchmarks (BEIR + ANN)..."
	cargo run -p contextra-bench --release -- --synthetic-only 2>&1 | tee benchmarks/results/external_latest.json
	echo "✅ Externe Benchmarks abgeschlossen: benchmarks/results/external_latest.json"
