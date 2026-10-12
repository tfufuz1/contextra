#!/usr/bin/env bash
# =============================================================================
# Contextra — Jules Environment Setup (schwer, snapshot-fähig, idempotent)
# Ziel: Jules-VM (Ubuntu 24.04, rustup/Node/Python/Go/Java/Docker vorinstalliert)
# Läuft bei "Run and Snapshot". Danach wird nur der SNAPSHOT wiederverwendet —
# alles Zeitabhängige (Commits, Tags, Claims, PRs) steckt deshalb in
# refresh-context.sh, das bei JEDEM Task-Start läuft.
# Diese Datei ändert KEINE getrackten Repo-Dateien (kein dirty tree).
# =============================================================================
set -uo pipefail                      # bewusst KEIN -e: ein optionales Tool darf den Snapshot nicht kippen
export DEBIAN_FRONTEND=noninteractive
LOG="${TMPDIR:-/tmp}/contextra-setup.log"; : > "$LOG"
FAILS=0

step() { echo; echo "=== $* ==="; }
ok()   { echo "  ✅ $*"; }
warn() { echo "  ⚠️  $*"; }
try()  { "$@" >>"$LOG" 2>&1 || { warn "fehlgeschlagen (optional): $* — siehe $LOG"; return 1; }; }
need() { "$@" >>"$LOG" 2>&1 || { echo "  ❌ PFLICHT fehlgeschlagen: $*"; tail -n 15 "$LOG"; FAILS=$((FAILS+1)); return 1; }; }

# Profile determination: core vs full (default: full)
PROFILE="${CONTEXTRA_SETUP_PROFILE:-full}"
if [ "${CONTEXTRA_SETUP_FULL:-0}" = "1" ]; then
  PROFILE="full"
fi

# Repo-Wurzel ermitteln (Jules klont nach /app; nicht hart verdrahten)
REPO="$(git rev-parse --show-toplevel 2>/dev/null || true)"
[ -z "$REPO" ] && for d in /app /home/jules/repo "$PWD"; do [ -f "$d/rust-toolchain.toml" ] && REPO="$d" && break; done
[ -n "$REPO" ] || { echo "❌ Repo-Wurzel nicht gefunden"; exit 1; }
cd "$REPO" || exit 1

PIN="$(sed -n 's/^channel *= *"\(.*\)"/\1/p' rust-toolchain.toml | head -1)"   # Single Source of Truth
PIN="${PIN:-1.89.0}"

echo "============================================================"
echo "  Contextra Jules Environment Setup  $(date -u '+%F %T UTC')"
echo "  Repo: $REPO   Rust-Pin: $PIN   Profil: $PROFILE   Nutzer: $(id -un)"
echo "============================================================"

# sudo nur nicht-interaktiv
SUDO=""; if [ "$(id -u)" != 0 ]; then sudo -n true 2>/dev/null && SUDO="sudo -n" || SUDO="__none__"; fi

# ── 1. Systempakete (inkl. shellcheck) ───────────────────────────────────────
step "[1/7] apt-Pakete"
if [ "$SUDO" = "__none__" ]; then warn "kein sudo -n — apt übersprungen"
else
  try $SUDO apt-get update -q
  try $SUDO apt-get install -y -q --no-install-recommends \
      build-essential pkg-config libssl-dev clang lld cmake \
      strace valgrind gdb python3-venv python3-dev shellcheck
  ok "apt abgeschlossen"
fi

# flatc-SHIM: crates/contextra-wire/build.rs ruft bei vorhandenem `flatc` IMMER neu generieren auf
mkdir -p "$HOME/.local/bin"
[ "$SUDO" != "__none__" ] && try $SUDO apt-get remove -y -q flatbuffers-compiler
cat > "$HOME/.local/bin/flatc" <<'SHIM'
#!/usr/bin/env bash
[ -n "${CONTEXTRA_REAL_FLATC:-}" ] && exec "$CONTEXTRA_REAL_FLATC" "$@"
echo "flatc deaktiviert (Versions-Mismatch-Schutz)" >&2; exit 127
SHIM
chmod +x "$HOME/.local/bin/flatc"
export PATH="$HOME/.local/bin:$PATH"
[ -f "$REPO/crates/contextra-wire/src/contextra_generated.rs" ] && touch "$REPO/crates/contextra-wire/src/contextra_generated.rs" 2>/dev/null || true
flatc --version >/dev/null 2>&1 && warn "flatc ist noch aktiv!" || ok "flatc deaktiviert → build.rs nutzt eingechecktes contextra_generated.rs"

# ── 2. Rust: Pin + stable + nightly (Fuzz/Miri/Sanitizer) ────────────────────
step "[2/7] Rust-Toolchains"
. "$HOME/.cargo/env" 2>/dev/null || true
command -v rustup >/dev/null || { curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none >>"$LOG" 2>&1; . "$HOME/.cargo/env"; }
need rustup toolchain install "$PIN" --profile minimal -c rustfmt -c clippy -c llvm-tools-preview
try  rustup default "$PIN"
try  rustup target add wasm32-wasip1 wasm32-unknown-unknown --toolchain "$PIN"

if [ "$PROFILE" = "full" ]; then
  try rustup toolchain install stable --profile minimal
  try rustup toolchain install nightly --profile minimal -c rust-src -c llvm-tools-preview -c miri
fi

RV="$(rustc --version 2>/dev/null)"
[[ "$RV" == *"$PIN"* ]] && ok "$RV" || { echo "  ❌ Toolchain-Mismatch: erwartet $PIN, ist: $RV"; FAILS=$((FAILS+1)); }

# ── 3. Cargo-Werkzeuge & Toolchains ──────────────────────────────────────────
step "[3/7] Cargo-Tools & Toolchains"
if ! command -v cargo-binstall >/dev/null; then
  try bash -c 'curl -L --proto "=https" --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash'
fi

install_tool() {   # $1=binary-check  $2=crate
  local chk="$1" crate="$2"
  if eval "$chk" >/dev/null 2>&1; then ok "$crate vorhanden"; return; fi
  if command -v cargo-binstall >/dev/null && try cargo binstall -y --locked "$crate"; then ok "$crate (binstall)"
  else try cargo install --locked "$crate" && ok "$crate (source)"; fi
}

install_tool "command -v just"            just
install_tool "cargo nextest --version"    cargo-nextest
install_tool "cargo deny --version"       cargo-deny
install_tool "cargo audit --version"      cargo-audit

if [ "$PROFILE" = "full" ]; then
  install_tool "cargo llvm-cov --version" cargo-llvm-cov
  install_tool "cargo mutants --version"  cargo-mutants
  install_tool "cargo insta --version"    cargo-insta
  install_tool "cargo hack --version"     cargo-hack

  if ! command -v actionlint >/dev/null 2>&1; then
    try bash -c 'curl -sSfL https://raw.githubusercontent.com/rhysd/actionlint/main/scripts/download-actionlint.bash | bash -s -- 1.7.7 "$HOME/.local/bin"'
  fi
  command -v actionlint >/dev/null 2>&1 && ok "actionlint vorhanden" || warn "actionlint fehlt (optional)"
fi

# Fuzz toolchain via install-fuzz-toolchain.sh (Warning in core profile, Required in full profile)
step "[3b/7] Fuzz-Toolchain"
if [ -x "$REPO/.jules/setup/install-fuzz-toolchain.sh" ]; then
  if [ "$PROFILE" = "full" ]; then
    need bash "$REPO/.jules/setup/install-fuzz-toolchain.sh"
  else
    try bash "$REPO/.jules/setup/install-fuzz-toolchain.sh"
  fi
else
  warn "install-fuzz-toolchain.sh fehlt oder nicht ausführbar"
fi

if [ "$PROFILE" = "full" ]; then
  # Miri setup
  step "[3c/7] Miri-Setup"
  try cargo +nightly miri setup
fi

if ! command -v ast-grep >/dev/null; then
  command -v sg >/dev/null && try ln -sf "$(command -v sg)" "$HOME/.cargo/bin/ast-grep" \
    || try npm install -g @ast-grep/cli
fi
command -v ast-grep >/dev/null && ok "ast-grep" || warn "ast-grep fehlt (optional)"

# ── 4. Benchmark- & Python-Umgebung ───────────────────────────────────────────
step "[4/7] Benchmark- & Python-Umgebung"
if [ -f "$REPO/.jules/setup/install-bench-deps.sh" ]; then
  try bash "$REPO/.jules/setup/install-bench-deps.sh"
fi

VENV="$HOME/.venv-contextra"
if [ -x "$VENV/bin/pip" ]; then
  try "$VENV/bin/pip" install -q --upgrade maturin numpy pytest && ok "venv tools: maturin, numpy, pytest"
fi

# ── 5. Persistente Umgebung (idempotent) ─────────────────────────────────────
step "[5/7] Umgebungsvariablen"
for RC in "$HOME/.bashrc" "$HOME/.profile"; do
  touch "$RC"
  sed -i '/# >>> contextra-env >>>/,/# <<< contextra-env <<</d' "$RC"
  cat >> "$RC" <<ENVEOF
# >>> contextra-env >>>
export CARGO_TERM_COLOR=never
export CARGO_INCREMENTAL=0
export CARGO_NET_RETRY=5
export RUST_BACKTRACE=1
export CONTEXTRA_EMBEDDING_PROVIDER=mock   # kein Modell-Download in der VM
export PATH="\$HOME/.local/bin:$VENV/bin:\$HOME/.cargo/bin:\$PATH"
# <<< contextra-env <<<
ENVEOF
done
ok ".bashrc/.profile aktualisiert"

# ── 6. Caches & Prebuilds ────────────────────────────────────────────────────
step "[6/7] Cargo-Cache & Prebuilds"
timeout 600 cargo fetch --locked >>"$LOG" 2>&1 && ok "cargo fetch --locked" || warn "cargo fetch fehlgeschlagen/Timeout"

if [ "${CONTEXTRA_SETUP_PREBUILD:-1}" = 1 ]; then
  timeout 1200 cargo check --workspace --locked >>"$LOG" 2>&1 && ok "cargo check --workspace" \
    || warn "cargo check Fehler/Timeout (Details: $LOG) — Snapshot trotzdem möglich"

  # Bench prebuild
  timeout 600 cargo check -p contextra-bench --benches --locked >>"$LOG" 2>&1 && ok "bench check" || warn "bench check Fehler/Timeout"
  timeout 600 cargo bench --no-run -p contextra-bench >>"$LOG" 2>&1 && ok "bench prebuild" || warn "bench prebuild Fehler/Timeout"

  # Fuzz targets pre-warming
  if [ "$PROFILE" = "full" ]; then
    echo "  🔥 Vorwärmung der Fuzz-Crates..."
    while IFS= read -r f; do
      [ -f "$f" ] || continue
      fuzz_dir="$(dirname "$f")"
      crate_dir="$(dirname "$fuzz_dir")"
      cname="$(basename "$crate_dir")"
      echo "  → Fuzz-Crate $cname..."
      (cd "$crate_dir" && timeout 300 cargo +nightly fuzz build >>"$LOG" 2>&1) || warn "fuzz build für $cname fehlgeschlagen/Timeout"
    done < <(find crates -path '*/fuzz/Cargo.toml' | sort)
  fi
fi

# ── 7. Baseline-Erfassung & Invarianten-Report ────────────────────────────────
step "[7/7] Baseline-Erfassung & Invarianten-Report"

BASELINE_FILE="$REPO/.jules/context/BASELINE.md"
mkdir -p "$REPO/.jules/context"

run_baseline_cmd() {
  local name="$1" cmd="$2"
  local log_file="${TMPDIR:-/tmp}/baseline_${name// /_}.log"
  local start_sec duration status warn_cnt exit_code
  start_sec=$(date +%s)

  eval "timeout 1200 $cmd" >"$log_file" 2>&1
  exit_code=$?
  duration=$(($(date +%s) - start_sec))

  if [ $exit_code -eq 124 ]; then
    status="NICHT_ERMITTELT (Timeout 1200s)"
  elif [ $exit_code -eq 0 ]; then
    status="GRÜN"
  else
    status="ROT"
  fi

  warn_cnt=$(grep -c -i "warning:" "$log_file" || true)

  {
    echo "### $name"
    echo "- **Status**: $status"
    echo "- **Dauer**: ${duration}s"
    echo "- **Warnungen**: $warn_cnt"
    if [ $exit_code -ne 0 ] && [ $exit_code -ne 124 ]; then
      echo "- **Erste 20 Fehlerzeilen**:"
      echo '```'
      grep -i "error" "$log_file" | head -n 20 || head -n 20 "$log_file"
      echo '```'
    fi
    echo ""
  } >> "$BASELINE_FILE"
}

cat > "$BASELINE_FILE" <<BASELINEOF
# Contextra — Setup Baseline Report
- **HEAD SHA**: $(git rev-parse HEAD 2>/dev/null || echo "UNKNOWN")
- **Datum**: $(date -u '+%Y-%m-%d %H:%M:%S UTC')
- **rustc**: $(rustc --version 2>/dev/null || echo "UNKNOWN")

BASELINEOF

run_baseline_cmd "cargo check" "cargo check --workspace --locked"
run_baseline_cmd "cargo fmt" "cargo fmt --check"
run_baseline_cmd "cargo clippy" "cargo clippy --workspace --locked"

if timeout 300 cargo check -p xtask --locked >>"$LOG" 2>&1; then
  echo "- **xtask Build Status**: GRÜN" >> "$BASELINE_FILE"
else
  echo "- **xtask Build Status**: ROT" >> "$BASELINE_FILE"
fi

ok "BASELINE.md erzeugt unter .jules/context/BASELINE.md"

[ -f AGENTS.md ] && ok "AGENTS.md ($(wc -l < AGENTS.md) Zeilen)" || { echo "  ❌ AGENTS.md fehlt"; FAILS=$((FAILS+1)); }
grep -q axum crates/contextra-mcp/Cargo.toml 2>/dev/null && warn "ADR-010 verletzt: axum in contextra-mcp" || ok "ADR-010: kein axum in contextra-mcp"
OPEN=$(grep -rn 'AI-TAG\[[A-Z]*\]\[\(CRITICAL\|BLOCKER\)\]' crates/ --include='*.rs' 2>/dev/null | grep -vc RESOLVED)
echo "  ℹ️  Offene CRITICAL/BLOCKER-Tags: ${OPEN:-0} (Ziel: 0)"

if cargo run -q --manifest-path xtask/Cargo.toml -- harness-list 2>/dev/null | grep -q env-attest; then
  cargo run -q --manifest-path xtask/Cargo.toml -- env-attest 2>&1 | tail -n 15 || warn "env-attest meldet Abweichungen"
else warn "xtask env-attest (noch) nicht registriert"; fi

# Kontext erzeugen (refresh-context.sh unberührt aufrufen)
[ -f "$REPO/.jules/setup/refresh-context.sh" ] && bash "$REPO/.jules/setup/refresh-context.sh" || warn "refresh-context.sh fehlt"

# Clean working tree check
if [ -n "$(git status --porcelain)" ]; then
  warn "Working Tree nach Setup dirty:"
  git status --short | head -10
  git reset --hard HEAD -q && git clean -fdq
fi
git status --porcelain | grep -q . && { echo "❌ Working Tree weiterhin dirty"; FAILS=$((FAILS+1)); } || ok "Working Tree sauber"

echo; echo "============================================================"
[ "$FAILS" = 0 ] && echo "  ✅ Contextra Jules Environment bereit" || echo "  ❌ $FAILS Pflichtschritt(e) fehlgeschlagen"
echo "  Log: $LOG"; echo "============================================================"
exit $(( FAILS > 0 ))
