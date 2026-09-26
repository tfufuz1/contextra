#!/usr/bin/env bash
# scripts/jules_hard_gate.sh — Maschinelles, beweispflichtiges Pre-Submit-Gate für Jules
#
# ZWECK:
#   Ersetzt "ich vertraue der Selbstauskunft des Agenten" durch einen
#   tamper-evidenten Ledger-Eintrag (.jules/gate_ledger.json). Der
#   Pre-Push-Hook (.githooks/pre-push) verweigert den Push, wenn dieses
#   Skript nicht seit der letzten Working-Tree-Änderung erfolgreich
#   gelaufen ist (Hash-Bindung an `git stash create`).
#
# NUTZUNG:
#   ./scripts/jules_hard_gate.sh                 # volle Suite (Definition of Done)
#   ./scripts/jules_hard_gate.sh --fast          # nur fmt+clippy+check (Iterationsmodus, NICHT submit-fähig)
#   ./scripts/jules_hard_gate.sh --crate NAME    # zusätzlich crate-scope Test
#
# ABHÄNGIGKEITEN: jq (JSON-Ledger), cargo, just
#
# EXIT-CODE: 0 nur wenn ALLE ausgeführten Schritte bestehen (bzw. --fast-Teilmenge).
#            Jeder andere Code bedeutet: mindestens ein Schritt ist fehlgeschlagen,
#            der Ledger-Status ist "FAIL", und der pre-push-Hook wird den Push blockieren.

set -uo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

command -v jq >/dev/null 2>&1 || {
    echo "❌ 'jq' ist nicht installiert. Installiere es (z. B. 'apt-get install -y jq') — der Ledger kann sonst nicht geschrieben werden."
    exit 2
}

TS="$(date -u +%Y%m%dT%H%M%SZ)"
EPOCH="$(date -u +%s)"
EVIDENCE_DIR="target/jules_gate/${TS}"
mkdir -p "$EVIDENCE_DIR"
LEDGER=".jules/gate_ledger.json"

FAST=false
CRATE=""
while [[ $# -gt 0 ]]; do
    case "$1" in
        --fast) FAST=true; shift ;;
        --crate) CRATE="${2:-}"; shift 2 ;;
        *) shift ;;
    esac
done

STEPS_JSON="[]"
OVERALL_OK=true

run_step() {
    local name="$1"; shift
    local log="${EVIDENCE_DIR}/${name}.log"
    echo "→ [${name}] $*"
    if "$@" >"$log" 2>&1; then
        echo "  ✅ ${name}"
        STEPS_JSON=$(echo "$STEPS_JSON" | jq --arg n "$name" --arg l "$log" '. + [{"step":$n,"status":"PASS","log":$l}]')
    else
        echo "  ❌ ${name} — siehe $log"
        tail -n 20 "$log" || true
        STEPS_JSON=$(echo "$STEPS_JSON" | jq --arg n "$name" --arg l "$log" '. + [{"step":$n,"status":"FAIL","log":$l}]')
        OVERALL_OK=false
    fi
}

# ── Schnelle Schicht (immer, auch im --fast Modus) ──────────────────────────
run_step "fmt"    cargo fmt --all -- --check
run_step "clippy" cargo clippy --workspace --all-targets --locked -- -D warnings
run_step "check"  cargo check --workspace --all-targets --locked

# ── Volle Schicht (Definition-of-Done, submit-relevant) ─────────────────────
if [ "$FAST" = false ]; then
    run_step "test-workspace"          cargo test --workspace --locked
    run_step "debt-audit"              just debt-audit
    run_step "dag-check"               cargo run --manifest-path xtask/Cargo.toml -- check-dag
    run_step "ring-layering-full"      cargo run --manifest-path xtask/Cargo.toml -- check-ring-layering-full
    run_step "check-vetoes"            cargo run --manifest-path xtask/Cargo.toml -- check-vetoes
    run_step "check-agents-integrity"  cargo run --manifest-path xtask/Cargo.toml -- check-agents-integrity
    run_step "sync-docs-check"         cargo run --manifest-path xtask/Cargo.toml -- sync-docs --check
    run_step "check-max-results-unbound"  cargo run --manifest-path xtask/Cargo.toml -- check-max-results-unbound
    run_step "check-toctou-defaults"      cargo run --manifest-path xtask/Cargo.toml -- check-toctou-defaults
    run_step "check-nan-hot-loop"         cargo run --manifest-path xtask/Cargo.toml -- check-nan-hot-loop
    run_step "check-result-dropped-io"    cargo run --manifest-path xtask/Cargo.toml -- check-result-dropped-io
    run_step "check-duplicate-core-primitives" cargo run --manifest-path xtask/Cargo.toml -- check-duplicate-core-primitives
    run_step "check-commit-diff-integrity"     cargo run --manifest-path xtask/Cargo.toml -- check-commit-diff-integrity

    if [ -n "$CRATE" ]; then
        run_step "test-crate-${CRATE}" cargo test -p "$CRATE" --locked
    fi
fi

# ── Tamper-evidenter Worktree-Hash (bindet den Ledger an EXAKT diesen Stand) ─
# `git stash create` erzeugt einen Commit-Objekt-Hash aus Index+Worktree, OHNE
# den Zustand zu verändern. Bei sauberem Tree liefert es leere Ausgabe -> Fallback auf HEAD.
CURRENT_HASH="$(git stash create 2>/dev/null || true)"
if [ -z "$CURRENT_HASH" ]; then
    CURRENT_HASH="CLEAN:$(git rev-parse HEAD)"
fi

if [ "$OVERALL_OK" = true ]; then STATUS="PASS"; else STATUS="FAIL"; fi
MODE_STR="full"; [ "$FAST" = true ] && MODE_STR="fast"

mkdir -p .jules
jq -n \
    --arg status "$STATUS" \
    --arg ts "$TS" \
    --argjson epoch "$EPOCH" \
    --arg hash "$CURRENT_HASH" \
    --arg evidence "$EVIDENCE_DIR" \
    --arg mode "$MODE_STR" \
    --argjson steps "$STEPS_JSON" \
    '{status:$status, timestamp:$ts, timestamp_epoch:$epoch, worktree_hash:$hash, evidence_dir:$evidence, mode:$mode, steps:$steps}' \
    > "$LEDGER"

echo ""
echo "════════════════════════════════════════════════════"
if [ "$STATUS" = "PASS" ] && [ "$MODE_STR" = "full" ]; then
    echo "✅ HARD GATE (full): PASS — Ledger geschrieben: $LEDGER"
    echo "   Push ist über den pre-push-Hook zulässig, solange sich der"
    echo "   Working-Tree nicht ändert (Hash: ${CURRENT_HASH:0:12}...)."
elif [ "$STATUS" = "PASS" ] && [ "$MODE_STR" = "fast" ]; then
    echo "⚠️  HARD GATE (fast): Teil-PASS — NICHT submit-fähig."
    echo "   Fast-Modus deckt weder Tests noch Governance-Gates ab."
    echo "   Vor Push zwingend './scripts/jules_hard_gate.sh' (voller Modus) ausführen."
else
    echo "❌ HARD GATE: FAIL — Push wird vom pre-push-Hook blockiert."
    echo "   Details in: $EVIDENCE_DIR"
fi
echo "════════════════════════════════════════════════════"

[ "$STATUS" = "PASS" ] && [ "$MODE_STR" = "full" ]
