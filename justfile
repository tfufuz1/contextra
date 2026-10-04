# AGENT:11
set shell := ["bash", "-uc"]

default:
    @just --list

[private]
nix-run *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v nix &> /dev/null && nix develop -c true &> /dev/null; then
        nix develop -c {{ARGS}}
    else
        {{ARGS}}
    fi

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
    just nix-run cargo fmt --all -- --check
    just nix-run cargo clippy --all-targets -- -D warnings
    just nix-run cargo check --all-targets --workspace
    just nix-run cargo xtask check-max-results-unbound
    just nix-run cargo xtask check-toctou-defaults
    just nix-run cargo xtask check-nan-hot-loop
    just nix-run cargo xtask check-result-dropped-io
    if [ -f coverage.json ]; then
        just nix-run cargo xtask check-coverage-gate
    fi

# Agent entry points (Phase runner / xtask jules is the source of truth)
agent-check *ARGS:
    cargo xtask jules check {{ARGS}}

agent-verify *ARGS:
    cargo xtask jules verify {{ARGS}}

agent-submit *ARGS:
    cargo xtask jules submit {{ARGS}}

# Modular check for any crate
check-crate CRATE *ARGS:
    just nix-run cargo check -p {{CRATE}} {{ARGS}}

# Modular check for contextra-core
check-core *ARGS:
    just check-crate contextra-core {{ARGS}}

# Modular check for contextra-store
check-store *ARGS:
    just check-crate contextra-store {{ARGS}}

# Runs Loom concurrency exploration model test for group commit lock handoff (unoptimized debug build required by Loom)
loom-store:
    RUSTFLAGS="--cfg loom" cargo test -p contextra-store --test loom_group_commit_handoff -- --nocapture

# Runs the chaos matrix fault-injection integration test suite
chaos-test:
    just nix-run cargo test -p contextra-store --test chaos_matrix -- --ignored --test-threads=1

# Modular check for contextra-vector
check-vector *ARGS:
    just check-crate contextra-vector {{ARGS}}

# Modular check for contextra-db
check-db *ARGS:
    just check-crate contextra-db {{ARGS}}

# Modular check for contextra-text
check-text *ARGS:
    just check-crate contextra-text {{ARGS}}

# Sync documentation from inline tags and cargo topology
sync-docs:
    just nix-run cargo xtask sync-docs

# Verifies if documentation is in sync with code without making changes
sync-docs-check:
    just nix-run cargo xtask sync-docs --check

# Verifies multi-session review coverage for completed anchors
check-review-coverage:
    just nix-run cargo xtask check-review-coverage

# Verifies internal documentation consistency (e.g. crate counts)
check-consistency:
    just nix-run cargo xtask check-consistency

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

# Executes new architecture gates: check-ring-layering-full and check-duplicate-core-primitives
check-arch-gates:
    cargo run --manifest-path xtask/Cargo.toml -- check-ring-layering-full
    cargo run --manifest-path xtask/Cargo.toml -- check-duplicate-core-primitives

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
    just nix-run cargo check --manifest-path crates/contextra-py/Cargo.toml

# Modular check for contextra-infer <!-- crate-ref-ignore -->
check-infer:
    just nix-run cargo check -p contextra-infer-candle -p contextra-infer-ollama -p contextra-infer-onnx

# Generiert prompter-data.json aus dem Live-Repo-Stand
gen-prompter-data:
    just nix-run cargo xtask gen-prompter-data

# Verifies the Directed Acyclic Graph (DAG) integrity of the workspace
dag-check:
    just nix-run cargo xtask check-dag

# Checks for permanent feature veto keywords in recent commits
check-vetoes:
    just nix-run cargo xtask check-vetoes

# Checks ADR deprecation and removal deadlines
check-adr-deadlines:
    just nix-run cargo xtask check-adr-deadlines

# Triple-Test-Gate: Tests müssen 3x hintereinander grün sein (DONE-Definition)
triple-test: check
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Triple-Test-Gate ==="
    for RUN in 1 2 3; do
        echo "--- Run $RUN/3 ---"
        if ! just nix-run cargo test --workspace; then
            echo "❌ FAILED on run $RUN/3. Fix all failures before this WP is DONE."
            exit 1
        fi
    done
    echo "✅ Triple-Test-Gate PASSED (3/3)"

# Security Advisory Audit: scannt Abhängigkeiten auf bekannte Schwachstellen via cargo-audit
security-audit:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Security Advisory Audit ==="
    if ! cargo audit --version &>/dev/null 2>&1; then
        echo "❌ cargo-audit ist nicht installiert."
        echo "   Für lokale Verifikation: cargo install cargo-audit --locked"
        echo "   In CI-Umgebungen muss cargo-audit im Runner-Environment bzw. Setup bereitgestellt werden."
        echo "   Siehe Dokumentation unter docs/ci/cargo-audit.md"
        exit 1
    fi
    cargo audit --deny warnings

# Tech-Debt Audit: scannt nach .unwrap(), unsafe, std::fs in Produktionscode
# Hinweis: Security-Advisory-Scans wurden entkoppelt und laufen separat über `just security-audit`.
debt-audit:
    #!/usr/bin/env bash
    set -euo pipefail
    FAIL=0
    echo "=== Tech-Debt Audit ==="

    echo "--- [1/4] .unwrap() außerhalb von Test-Code ---"
    UNWRAP=$(grep -rn "\.unwrap()" crates/ --include="*.rs" \
        | grep -v "_test\.rs:" \
        | grep -v "/tests/" \
        | grep -v "/tests\.rs:" \
        | grep -v "/benches/" \
        | grep -v "benches\.rs:" \
        | grep -v "contextra_generated\.rs:" \
        | grep -v "::tests::" \
        | grep -v "//.*unwrap" \
        | grep -v "// expect" \
        || true)
    if [ -n "$UNWRAP" ]; then
        UNWRAP_COUNT=$(echo "$UNWRAP" | wc -l)
        echo "❌ UNWRAP VIOLATIONS ($UNWRAP_COUNT Treffer — fix in WP-0.0):"
        echo "$UNWRAP" | head -15
        FAIL=1
    else echo "✅ Kein .unwrap() in Produktionscode"; fi

    echo "--- [2/4] unsafe außerhalb distance.rs ---"
    UNSAFE=$(grep -rn "unsafe " crates/ --include="*.rs" \
        | grep -v "crates/contextra-vector/src/distance\.rs" \
        | grep -v "#\[allow(unsafe_code)\]" \
        | grep -v "//.*unsafe" \
        || true)
    if [ -n "$UNSAFE" ]; then
        echo "❌ UNSAFE VIOLATIONS:"; echo "$UNSAFE"; FAIL=1
    else echo "✅ Kein unsafe außerhalb distance.rs"; fi

    echo "--- [3/4] std::fs in Produktionscode (Soft-Warning) ---"
    STDFS=$(grep -rn "std::fs::" crates/ --include="*.rs" \
        | grep -v "/tests/" | grep -v "mod tests" || true)
    if [ -n "$STDFS" ]; then
        echo "⚠️  std::fs:: Treffer (nach tokio::fs migrieren):"
        echo "$STDFS"
    else echo "✅ Kein std::fs:: in Produktionscode"; fi

    echo "--- [4/4] Lock-Hierarchy & Async-Safety (AST Analysis) ---"
    # Prüfe auf verschachtelte Locks (potenzielle Deadlocks) mittels ast-grep
    if command -v sg > /dev/null; then
        if sg scan --rule rules/detect_nested_locks.yml crates/; then
            echo "❌ Graceful Deadlock Risiko erkannt! Verschachtelte Locks gefunden:"
            FAIL=1
        else
            echo "✅ Keine kritischen Deadlock-Zustände im AST gefunden."
        fi
    else
        echo "⚠️  ast-grep (sg) nicht installiert, überspringe AST-Lock-Analyse."
    fi

    if [ $FAIL -eq 1 ]; then
        echo ""; echo "❌ Debt-Audit FAILED — WP-0.0 zuerst abschließen!"; exit 1
    fi
    echo ""; echo "✅ Debt-Audit PASSED"

# Runs the LongMemEval regression benchmark suite and compares against baseline
bench-regression:
    just nix-run cargo run -p contextra-bench --release

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

# Führt Mutation-Testing für contextra-crypto (Tier-0) aus
mutants-crypto *ARGS:
	cargo mutants --package contextra-crypto {{ARGS}}

# Alias für mutants-crypto
mutants *ARGS:
	cargo mutants --package contextra-crypto {{ARGS}}

check-marker-drift:
    cargo xtask check-marker-drift

shell-commit-audit *ARGS:
    cargo xtask shell-commit-audit {{ARGS}}

workspace-verify *ARGS:
    cargo xtask workspace-verify {{ARGS}}

panic-inventory *ARGS:
    cargo xtask panic-inventory {{ARGS}}

security-scan *ARGS:
    cargo xtask security-scan {{ARGS}}

feature-matrix *ARGS:
    cargo xtask feature-matrix {{ARGS}}

check-ring-capabilities-consistency:
    cargo xtask check-ring-capabilities-consistency

loom-run *ARGS:
    cargo xtask loom-run {{ARGS}}

bench-compile *ARGS:
    cargo xtask bench-compile {{ARGS}}

bench-download *ARGS:
    cargo xtask bench-download {{ARGS}}

context-pack *ARGS:
    cargo xtask context-pack {{ARGS}}

session-init *ARGS:
    cargo xtask session-init {{ARGS}}

env-validate:
    cargo xtask env-validate

crate-context CRATE *ARGS:
    cargo xtask crate-context {{CRATE}} {{ARGS}}

audit-integrity-check *ARGS:
    cargo xtask audit-integrity-check {{ARGS}}

tag-health *ARGS:
    cargo xtask tag-health {{ARGS}}

hotspot-report *ARGS:
    cargo xtask hotspot-report {{ARGS}}

commit-health *ARGS:
    cargo xtask commit-health {{ARGS}}

py-test *ARGS:
    cargo xtask py-test {{ARGS}}

check-veto-deadlines:
    cargo xtask check-veto-deadlines

full-audit:
    cargo xtask workspace-verify --mode=audit

harness-list *ARGS:
    cargo xtask harness-list {{ARGS}}

registry-check *ARGS:
    cargo xtask registry-check {{ARGS}}
