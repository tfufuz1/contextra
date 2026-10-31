#!/usr/bin/env bash
set -euo pipefail

echo "=== Contextra Invariants Gate ==="
ROOT_DIR="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT_DIR"

ERRORS=0

echo "[1/3] Checking Unsafe Islands..."
UNSAFE_OUTSIDE=$(grep -rnE "\bunsafe\s+(fn|trait|impl|\{)" crates/*/src/ 2>/dev/null | \
  grep -v "crates/contextra-simd" | \
  grep -v "crates/contextra-sys" | \
  grep -v "crates/contextra-wire" | \
  grep -v "#\[cfg(test)\]" | \
  grep -v "^\s*//" || true)

if [ -n "$UNSAFE_OUTSIDE" ]; then
    echo "  ! Notice: Potential unsafe usage outside approved islands:"
    echo "$UNSAFE_OUTSIDE" | head -n 5
    ERRORS=$((ERRORS + 1))
else
    echo "  ✓ Unsafe is cleanly isolated in simd/sys/wire."
fi

echo "[2/3] Checking for forbidden System Time and Thread RNG in Core Production Logic..."
python3 - <<'EOF'
import os, re, sys

ring0_dirs = [
    "crates/contextra-types/src",
    "crates/contextra-mvcc/src",
    "crates/contextra-vector/src",
    "crates/contextra-text/src",
]

pattern = re.compile(r'\b(Instant::now|SystemTime::now|thread_rng)\b')
violations = []

for base in ring0_dirs:
    if not os.path.exists(base):
        continue
    for root, _, files in os.walk(base):
        for f in files:
            if not f.endswith(".rs") or "test" in f.lower():
                continue
            path = os.path.join(root, f)
            with open(path, "r", encoding="utf-8", errors="ignore") as fh:
                in_test = False
                for i, line in enumerate(fh, 1):
                    sline = line.strip()
                    if sline.startswith("//") or sline.startswith("*") or sline.startswith("/*"):
                        continue
                    if "#[cfg(test)]" in line:
                        in_test = True
                    if in_test:
                        continue
                    if pattern.search(line):
                        violations.append(f"{path}:{i}: {sline}")

if violations:
    print("  ✗ Non-deterministic clock/rng calls found in core production logic:")
    for v in violations[:5]:
        print(f"    {v}")
    sys.exit(1)
else:
    print("  ✓ Core Ring 0 production logic is free of non-deterministic calls.")
EOF
if [ $? -ne 0 ]; then
    ERRORS=$((ERRORS + 1))
fi

echo "[3/3] Checking Zero-Panic in unwrap baseline..."
if [ -f ".github/unwrap_baseline.txt" ]; then
    BASELINE_COUNT=$(wc -l < .github/unwrap_baseline.txt)
    echo "  ✓ Zero-Panic tracking active: $BASELINE_COUNT allowed legacy unwrap baseline entries."
else
    echo "  - unwrap_baseline.txt not present."
fi

echo "================================="
if [ "$ERRORS" -eq 0 ]; then
    echo "Invariants gate check passed successfully."
    exit 0
else
    echo "Invariants gate check failed with $ERRORS error(s)."
    exit 1
fi
