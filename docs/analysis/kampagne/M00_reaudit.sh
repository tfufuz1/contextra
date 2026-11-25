#!/usr/bin/env bash
set -euo pipefail

echo "=== M00 RE-AUDIT SCRIPT ==="
echo "[GEMESSEN] Checking HEAD commit..."
git log -1 --oneline

echo "[GEMESSEN] Checking W0-13 compile check under --all-features..."
cargo check --tests --all-features -p contextra-vector || true

echo "[GEMESSEN] Checking W0-01 WAL flusher truncate size store..."
git grep -n "size.store(offset" crates/contextra-store/src/wal/flusher.rs || true

echo "[GEMESSEN] Checking W0-03 orphan workers and recover_pending_intents callers..."
git grep -n "start_orphan_cleanup_worker" crates/ benchmarks/ || echo "No callers in crates/ or benchmarks/"
git grep -n "recover_pending_intents" crates/ benchmarks/ || echo "No callers in crates/ or benchmarks/"

echo "[GEMESSEN] Checking W0-07 AuditChain verify_chain_against_anchor callers..."
git grep -n "verify_chain_against_anchor" crates/ benchmarks/ || echo "No callers in crates/ or benchmarks/"

echo "[GEMESSEN] Checking W0-08 with_revocation_log callers..."
git grep -n "with_revocation_log" crates/ benchmarks/ || echo "No callers in crates/ or benchmarks/"

echo "[GEMESSEN] Checking W0-10 CloudResponseRehydrator allowed_tokens default..."
git grep -n "allowed_tokens:" crates/contextra-privacy/src/egress_gateway.rs || true

echo "[GEMESSEN] Checking W0-12 DocIdRaw definition in posting_list.rs..."
git grep -n "DocIdRaw" crates/contextra-text/src/posting_list.rs || true

echo "=== RE-AUDIT COMPLETE ==="
