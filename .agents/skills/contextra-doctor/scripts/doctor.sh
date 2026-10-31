#!/usr/bin/env bash
set -euo pipefail

echo "=== Contextra Workspace Doctor ==="
ROOT_DIR="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT_DIR"

ERRORS=0

echo "[1/5] Checking cargo metadata..."
if cargo metadata --no-deps --format-version 1 >/dev/null 2>&1; then
    echo "  ✓ Cargo metadata loads successfully."
else
    echo "  ✗ Cargo metadata failed! Check workspace members and dependencies."
    ERRORS=$((ERRORS + 1))
fi

echo "[2/5] Checking for missing crate directories listed in Cargo.toml..."
python3 - <<'EOF'
import tomllib, os, sys

try:
    with open('Cargo.toml', 'rb') as f:
        d = tomllib.load(f)
    members = d.get('workspace', {}).get('members', [])
    missing = [m for m in members if not os.path.isdir(m)]
    if missing:
        print(f"  ✗ Missing workspace member directories: {missing}")
        sys.exit(1)
    else:
        print("  ✓ All declared workspace members exist on disk.")
except Exception as e:
    print(f"  ✗ Error checking members: {e}")
    sys.exit(1)
EOF
if [ $? -ne 0 ]; then
    ERRORS=$((ERRORS + 1))
fi

echo "[3/5] Checking for B-01 bug in delete_segment..."
SEGMENT_FILE="crates/contextra-store/src/kv/segment.rs"
if [ -f "$SEGMENT_FILE" ]; then
    if grep -q "pub fn delete_segment(&self, group_id: u64) -> bool" "$SEGMENT_FILE"; then
        echo "  ✗ B-01 detected: delete_segment returns bool instead of Result<bool, ...>"
        ERRORS=$((ERRORS + 1))
    else
        echo "  ✓ delete_segment signature is clean."
    fi
else
    echo "  - segment.rs not found or moved."
fi

echo "[4/5] Checking for stale xtask references..."
XTASK_HITS=$(grep -rn "cargo xtask" .github/workflows/ justfile .cargo/config.toml 2>/dev/null | wc -l || true)
if [ "$XTASK_HITS" -gt 0 ]; then
    echo "  ! Notice: Found $XTASK_HITS references to cargo xtask in configs/workflows."
else
    echo "  ✓ No stale cargo xtask references in primary configs."
fi

echo "[5/5] Checking git status..."
STAGED=$(git diff --cached --name-only | wc -l)
UNSTAGED=$(git diff --name-only | wc -l)
echo "  - Staged files: $STAGED, Unstaged files: $UNSTAGED"

echo "==================================="
if [ "$ERRORS" -eq 0 ]; then
    echo "Summary: Doctor checks passed without critical blockers."
    exit 0
else
    echo "Summary: Doctor found $ERRORS blocker(s). Consult .agents/skills/contextra-doctor/SKILL.md"
    exit 1
fi
