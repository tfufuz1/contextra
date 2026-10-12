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

# Profil statt Flag: CONTEXTRA_SETUP_PROFILE=core|full, Default full; CONTEXTRA_SETUP_FULL=1 bleibt Alias
PROFILE="${CONTEXTRA_SETUP_PROFILE:-}"
if [ -z "$PROFILE" ]; then
  if [ "${CONTEXTRA_SETUP_FULL:-1}" = "1" ]; then
    PROFILE="full"
  else
    PROFILE="core"
  fi
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

# ── 1. Systempakete (nur, was Contextra wirklich braucht; kein Tauri/GTK) ─────
step "[1/8] apt-Pakete"
if [ "$SUDO" = "__none__" ]; then warn "kein sudo -n — apt übersprungen"
else
  try $SUDO apt-get update -q
  try $SUDO apt-get install -y -q --no-install-recommends \
      build-essential pkg-config libssl-dev clang lld cmake \
      strace valgrind gdb python3-venv python3-dev shellcheck
  ok "apt abgeschlossen"
fi

# flatc-SHIM: crates/contextra-wire/build.rs ruft bei vorhandenem `flatc` IMMER neu generieren auf und
# überschreibt die getrackte src/contextra_generated.rs. Ubuntus flatc (2.0.8) passt nicht zum
# flatbuffers-Crate (24.12.x) → 35 Compilerfehler (E0053) + dirty Working Tree.
# Ohne nutzbares flatc nimmt build.rs den eingecheckten Code. Daher: apt-flatc entfernen/verdecken.
mkdir -p "$HOME/.local/bin"
[ "$SUDO" != "__none__" ] && try $SUDO apt-get remove -y -q flatbuffers-compiler
cat > "$HOME/.local/bin/flatc" <<'SHIM'
#!/usr/bin/env bash
# Shim: nur mit CONTEXTRA_REAL_FLATC=/pfad/zu/passendem/flatc nutzbar (Version muss zum Crate passen).
[ -n "${CONTEXTRA_REAL_FLATC:-}" ] && exec "$CONTEXTRA_REAL_FLATC" "$@"
echo "flatc deaktiviert (Versions-Mismatch-Schutz)" >&2; exit 127
SHIM
chmod +x "$HOME/.local/bin/flatc"
export PATH="$HOME/.local/bin:$PATH"
flatc --version >/dev/null 2>&1 && warn "flatc ist noch aktiv!" || ok "flatc deaktiviert → build.rs nutzt eingechecktes contextra_generated.rs"

# ── 2. Rust Toolchains ────────────────────────────────────────────────────────
step "[2/8] Rust-Toolchains"
. "$HOME/.cargo/env" 2>/dev/null || true
command -v rustup >/dev/null || { curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none >>"$LOG" 2>&1; . "$HOME/.cargo/env"; }
need rustup toolchain install "$PIN" --profile minimal -c rustfmt -c clippy -c llvm-tools-preview
try  rustup default "$PIN"
try  rustup target add wasm32-wasip1 wasm32-unknown-unknown --toolchain "$PIN"

if [ "$PROFILE" = "full" ]; then
  try rustup toolchain install stable --profile minimal
  try rustup toolchain install nightly --profile minimal -c rust-src -c llvm-tools-preview -c miri
fi

# Verifikation im Repo-Verzeichnis (rust-toolchain.toml greift dort)
RV="$(rustc --version 2>/dev/null)"
[[ "$RV" == *"$PIN"* ]] && ok "$RV" || { echo "  ❌ Toolchain-Mismatch: erwartet $PIN, ist: $RV"; FAILS=$((FAILS+1)); }

# ── 3. Fuzzing-Toolchain via install-fuzz-toolchain.sh ───────────────────────
step "[3/8] Fuzzing-Toolchain"
if [ "$PROFILE" = "full" ]; then
  need bash .jules/setup/install-fuzz-toolchain.sh
else
  try bash .jules/setup/install-fuzz-toolchain.sh
fi

# ── 4. Cargo-Werkzeuge ────────────────────────────────────────────────────────
step "[4/8] Cargo-Tools"
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
  command -v actionlint >/dev/null 2>&1 && ok "actionlint" || warn "actionlint installation fehlgeschlagen"
  try cargo +nightly miri setup
fi

# ast-grep: npm ist schneller als Kompilieren; 'sg' → 'ast-grep' verlinken
if ! command -v ast-grep >/dev/null; then
  command -v sg >/dev/null && try ln -sf "$(command -v sg)" "$HOME/.cargo/bin/ast-grep" \
    || try npm install -g @ast-grep/cli
fi
command -v ast-grep >/dev/null && ok "ast-grep" || warn "ast-grep fehlt (optional)"

# ── 5. Python-Umgebung & Benchmark-Abhängigkeiten ─────────────────────────────
step "[5/8] Python-venv & Benchmark-Deps"
VENV="$HOME/.venv-contextra"
[ -x "$VENV/bin/python" ] || try python3 -m venv "$VENV"
try "$VENV/bin/pip" install -q --upgrade pip maturin numpy pytest && ok "venv: $VENV"
try bash .jules/setup/install-bench-deps.sh

# ── 6. Persistente Umgebung & Caches vorwärmen ────────────────────────────────
step "[6/8] Umgebungsvariablen & Cache-Vorwärmung"
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

timeout 600 cargo fetch --locked >>"$LOG" 2>&1 && ok "cargo fetch --locked" || warn "cargo fetch fehlgeschlagen/Timeout"

find crates -path '*/fuzz/Cargo.toml' | sort | while read -r fuzz_toml; do
  fuzz_dir="$(dirname "$fuzz_toml")"
  cname="$(basename "$(dirname "$fuzz_dir")")"
  echo "  ℹ️ Fuzz build for $cname..."
  (cd "$fuzz_dir" && timeout 60 cargo +nightly fuzz build >>"$LOG" 2>&1) || warn "Fuzz build for $cname unvollständig/Fehler (siehe $LOG)"
done

if [ "${CONTEXTRA_SETUP_PREBUILD:-1}" = 1 ]; then
  timeout 1200 cargo check --workspace --locked >>"$LOG" 2>&1 && ok "cargo check --workspace" \
    || warn "cargo check Fehler/Timeout (Details: $LOG) — Snapshot trotzdem möglich"
  timeout 600 cargo check -p contextra-bench --benches --locked >>"$LOG" 2>&1 && ok "cargo check -p contextra-bench --benches" \
    || warn "cargo check -p contextra-bench --benches Fehler/Timeout"
  timeout 600 cargo bench --no-run -p contextra-bench >>"$LOG" 2>&1 && ok "cargo bench --no-run -p contextra-bench" \
    || warn "cargo bench --no-run -p contextra-bench Fehler/Timeout"
fi

# ── 7. Baseline-Protokoll (.jules/context/BASELINE.md) ────────────────────────
step "[7/8] Baseline-Protokoll generieren"
mkdir -p .jules/context
HEAD_SHA="$(git rev-parse HEAD 2>/dev/null || echo "unknown")"
BUILD_DATE="$(date -u '+%Y-%m-%d %H:%M:%S UTC')"
RUSTC_VER="$(rustc --version 2>/dev/null || echo "unknown")"

CHECK_LOG="${TMPDIR:-/tmp}/baseline_check_$$.log"
FMT_LOG="${TMPDIR:-/tmp}/baseline_fmt_$$.log"
CLIPPY_LOG="${TMPDIR:-/tmp}/baseline_clippy_$$.log"
XTASK_LOG="${TMPDIR:-/tmp}/baseline_xtask_$$.log"

run_cmd_measure() {  # $1=cmd  $2=timeout_secs  $3=out_file
  local cmd="$1" timeout_secs="$2" out_file="$3"
  local start_t end_t elapsed
  start_t=$(date +%s)
  if timeout "$timeout_secs" bash -c "$cmd" >"$out_file" 2>&1; then
    end_t=$(date +%s)
    elapsed=$((end_t - start_t))
    local warns
    warns=$(grep -c "warning:" "$out_file" 2>/dev/null || echo "0")
    echo "pass|$elapsed|$warns"
  else
    local code=$?
    end_t=$(date +%s)
    elapsed=$((end_t - start_t))
    if [ $code -eq 124 ]; then
      echo "timeout|$elapsed|0"
    else
      local warns
      warns=$(grep -c "warning:" "$out_file" 2>/dev/null || echo "0")
      echo "fail|$elapsed|$warns"
    fi
  fi
}

CHECK_RES="$(run_cmd_measure "cargo check --workspace --locked" 1200 "$CHECK_LOG")"
FMT_RES="$(run_cmd_measure "cargo fmt --check" 1200 "$FMT_LOG")"
CLIPPY_RES="$(run_cmd_measure "cargo clippy --workspace --locked" 1200 "$CLIPPY_LOG")"
XTASK_RES="$(run_cmd_measure "cargo check -p xtask --locked" 1200 "$XTASK_LOG")"

IFS='|' read -r CHECK_STATUS CHECK_TIME CHECK_WARNS <<< "$CHECK_RES"
IFS='|' read -r FMT_STATUS FMT_TIME FMT_WARNS <<< "$FMT_RES"
IFS='|' read -r CLIPPY_STATUS CLIPPY_TIME CLIPPY_WARNS <<< "$CLIPPY_RES"
IFS='|' read -r XTASK_STATUS XTASK_TIME XTASK_WARNS <<< "$XTASK_RES"

ERR_LINES="$(grep -h -E -i 'error\[|error:' "$CHECK_LOG" "$FMT_LOG" "$CLIPPY_LOG" "$XTASK_LOG" 2>/dev/null | head -n 20 || echo "None")"
rm -f "$CHECK_LOG" "$FMT_LOG" "$CLIPPY_LOG" "$XTASK_LOG"

cat > .jules/context/BASELINE.md <<BASELINEOF
# Contextra Baseline Measurement

- **Generated:** $BUILD_DATE
- **HEAD SHA:** $HEAD_SHA
- **Rustc Version:** $RUSTC_VER
- **Profile:** $PROFILE

## Verification Summary

| Command | Status | Duration (s) | Warnings |
| --- | --- | --- | --- |
| \`cargo check --workspace --locked\` | $CHECK_STATUS | ${CHECK_TIME}s | $CHECK_WARNS |
| \`cargo fmt --check\` | $FMT_STATUS | ${FMT_TIME}s | N/A |
| \`cargo clippy --workspace --locked\` | $CLIPPY_STATUS | ${CLIPPY_TIME}s | $CLIPPY_WARNS |
| \`cargo check -p xtask --locked\` | $XTASK_STATUS | ${XTASK_TIME}s | N/A |

### Initial Errors (First 20 Lines)
\`\`\`text
${ERR_LINES:-None}
\`\`\`
BASELINEOF
ok ".jules/context/BASELINE.md erzeugt"

# ── 8. Invarianten-Report & Clean Tree ───────────────────────────────────────
step "[8/8] Invarianten-Report & Clean Tree"
[ -f AGENTS.md ] && ok "AGENTS.md ($(wc -l < AGENTS.md) Zeilen)" || { echo "  ❌ AGENTS.md fehlt"; FAILS=$((FAILS+1)); }
grep -q axum crates/contextra-mcp/Cargo.toml 2>/dev/null && warn "ADR-010 verletzt: axum in contextra-mcp" || ok "ADR-010: kein axum in contextra-mcp"
OPEN=$(grep -rn 'AI-TAG\[[A-Z]*\]\[\(CRITICAL\|BLOCKER\)\]' crates/ --include='*.rs' 2>/dev/null | grep -vc RESOLVED)
echo "  ℹ️  Offene CRITICAL/BLOCKER-Tags: ${OPEN:-0} (Ziel: 0)"

if cargo run -q --manifest-path xtask/Cargo.toml -- harness-list 2>/dev/null | grep -q env-attest; then
  cargo run -q --manifest-path xtask/Cargo.toml -- env-attest 2>&1 | tail -n 15 || warn "env-attest meldet Abweichungen"
else warn "xtask env-attest (noch) nicht registriert"; fi

# Kontext einmalig erzeugen (wird bei Task-Start erneut aktualisiert)
[ -f .jules/setup/refresh-context.sh ] && bash .jules/setup/refresh-context.sh || warn "refresh-context.sh fehlt"

# ── Working-Tree zurücksetzen (Jules verifiziert sauberen Tree) ───────────────
# Getrackte Dateien außerhalb .jules/setup/ und .jules/context/ zurücksetzen
git checkout HEAD -- . ':(exclude).jules/setup/' ':(exclude).jules/context/' 2>/dev/null || true
git clean -fdq -e '.jules/*' 2>/dev/null || true

git status --porcelain | grep -v '^[?? M]*\.jules/setup/' | grep -q . && { echo "❌ Working Tree weiterhin dirty außerhalb .jules/setup/"; FAILS=$((FAILS+1)); } || ok "Working Tree sauber"

echo; echo "============================================================"
[ "$FAILS" = 0 ] && echo "  ✅ Contextra Jules Environment bereit" || echo "  ❌ $FAILS Pflichtschritt(e) fehlgeschlagen"
echo "  Log: $LOG"; echo "============================================================"
exit $(( FAILS > 0 ))
