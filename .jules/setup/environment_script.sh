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

# Repo-Wurzel ermitteln (Jules klont nach /app; nicht hart verdrahten)
REPO="$(git rev-parse --show-toplevel 2>/dev/null || true)"
[ -z "$REPO" ] && for d in /app /home/jules/repo "$PWD"; do [ -f "$d/rust-toolchain.toml" ] && REPO="$d" && break; done
[ -n "$REPO" ] || { echo "❌ Repo-Wurzel nicht gefunden"; exit 1; }
cd "$REPO" || exit 1

PIN="$(sed -n 's/^channel *= *"\(.*\)"/\1/p' rust-toolchain.toml | head -1)"   # Single Source of Truth
PIN="${PIN:-1.89.0}"

echo "============================================================"
echo "  Contextra Jules Environment Setup  $(date -u '+%F %T UTC')"
echo "  Repo: $REPO   Rust-Pin: $PIN   Nutzer: $(id -un)"
echo "============================================================"

# sudo nur nicht-interaktiv
SUDO=""; if [ "$(id -u)" != 0 ]; then sudo -n true 2>/dev/null && SUDO="sudo -n" || SUDO="__none__"; fi

# ── 1. Systempakete (nur, was Contextra wirklich braucht; kein Tauri/GTK) ─────
step "[1/7] apt-Pakete"
if [ "$SUDO" = "__none__" ]; then warn "kein sudo -n — apt übersprungen"
else
  try $SUDO apt-get update -q
  try $SUDO apt-get install -y -q --no-install-recommends \
      build-essential pkg-config libssl-dev clang lld cmake \
      strace valgrind gdb python3-venv python3-dev
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

# ── 2. Rust: Pin + stable (Cargo-Tools) + nightly (Fuzz/Sanitizer) ───────────
step "[2/7] Rust-Toolchains"
. "$HOME/.cargo/env" 2>/dev/null || true
command -v rustup >/dev/null || { curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none >>"$LOG" 2>&1; . "$HOME/.cargo/env"; }
need rustup toolchain install "$PIN" --profile minimal -c rustfmt -c clippy -c llvm-tools-preview
try  rustup default "$PIN"
try  rustup target add wasm32-wasip1 wasm32-unknown-unknown --toolchain "$PIN"
if [ "${CONTEXTRA_SETUP_FULL:-0}" = 1 ]; then
  try rustup toolchain install stable --profile minimal
  try rustup toolchain install nightly --profile minimal -c rust-src -c llvm-tools-preview
fi
# Verifikation im Repo-Verzeichnis (rust-toolchain.toml greift dort)
RV="$(rustc --version 2>/dev/null)"
[[ "$RV" == *"$PIN"* ]] && ok "$RV" || { echo "  ❌ Toolchain-Mismatch: erwartet $PIN, ist: $RV"; FAILS=$((FAILS+1)); }

# ── 3. Cargo-Werkzeuge: vorkompiliert (binstall), Fallback cargo install ──────
step "[3/7] Cargo-Tools"
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
if [ "${CONTEXTRA_SETUP_FULL:-0}" = 1 ]; then      # Audit-/Test-Kampagnen
  install_tool "cargo llvm-cov --version" cargo-llvm-cov
  install_tool "cargo mutants --version"  cargo-mutants
  # NIE 'cargo install fuzz' (veralteter Crate) – nur cargo-fuzz, mit nightly
  cargo fuzz --version >/dev/null 2>&1 || try cargo +nightly install --locked cargo-fuzz
fi
# ast-grep: npm ist schneller als Kompilieren; 'sg' → 'ast-grep' verlinken
if ! command -v ast-grep >/dev/null; then
  command -v sg >/dev/null && try ln -sf "$(command -v sg)" "$HOME/.cargo/bin/ast-grep" \
    || try npm install -g @ast-grep/cli
fi
command -v ast-grep >/dev/null && ok "ast-grep" || warn "ast-grep fehlt (optional)"

# ── 4. Python-Umgebung (contextra-py / maturin) ──────────────────────────────
step "[4/7] Python-venv"
VENV="$HOME/.venv-contextra"
[ -x "$VENV/bin/python" ] || try python3 -m venv "$VENV"
try "$VENV/bin/pip" install -q --upgrade pip maturin numpy pytest && ok "venv: $VENV"

# ── 5. Persistente Umgebung (idempotent, Marker-Block) ───────────────────────
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
ok "git hooks unverändert (Jules setzt global core.hooksPath=/dev/null)"

# ── 6. Abhängigkeits-Cache (Snapshot-Gewinn) ─────────────────────────────────
step "[6/7] Cargo-Cache aufwärmen"
timeout 600 cargo fetch --locked >>"$LOG" 2>&1 && ok "cargo fetch --locked" || warn "cargo fetch fehlgeschlagen/Timeout"
if [ "${CONTEXTRA_SETUP_PREBUILD:-1}" = 1 ]; then
  timeout 1200 cargo check --workspace --locked >>"$LOG" 2>&1 && ok "cargo check --workspace" \
    || warn "cargo check Fehler/Timeout (Details: $LOG) — Snapshot trotzdem möglich"
fi

# ── 7. Repo-Invarianten nur MELDEN (ein roter Repo-Zustand darf den Snapshot nicht blockieren)
step "[7/7] Invarianten-Report"
[ -f AGENTS.md ] && ok "AGENTS.md ($(wc -l < AGENTS.md) Zeilen)" || { echo "  ❌ AGENTS.md fehlt"; FAILS=$((FAILS+1)); }
grep -q axum crates/contextra-mcp/Cargo.toml 2>/dev/null && warn "ADR-010 verletzt: axum in contextra-mcp" || ok "ADR-010: kein axum in contextra-mcp"
OPEN=$(grep -rn 'AI-TAG\[[A-Z]*\]\[\(CRITICAL\|BLOCKER\)\]' crates/ --include='*.rs' 2>/dev/null | grep -vc RESOLVED)
echo "  ℹ️  Offene CRITICAL/BLOCKER-Tags: ${OPEN:-0} (Ziel: 0)"
if cargo run -q --manifest-path xtask/Cargo.toml -- harness-list 2>/dev/null | grep -q env-attest; then
  cargo run -q --manifest-path xtask/Cargo.toml -- env-attest 2>&1 | tail -n 15 || warn "env-attest meldet Abweichungen"
else warn "xtask env-attest (noch) nicht registriert"; fi

# Kontext einmalig erzeugen (wird bei Task-Start erneut aktualisiert)
[ -f .jules/setup/refresh-context.sh ] && bash .jules/setup/refresh-context.sh || warn "refresh-context.sh fehlt"

# ── Working-Tree zurücksetzen (Jules verwirft sonst die Verifikation: "Working tree is dirty") ──
if [ -n "$(git status --porcelain)" ]; then
  warn "Working Tree nach Setup dirty:"; git status --short | head -10
  git reset --hard HEAD -q && git clean -fdq        # ohne -x: target/ und Caches bleiben
fi
git reset --hard HEAD 2>&1 | tail -1                # gibt "HEAD is now at …" aus
git status --porcelain | grep -q . && { echo "❌ Working Tree weiterhin dirty"; FAILS=$((FAILS+1)); } || ok "Working Tree sauber"

echo; echo "============================================================"
[ "$FAILS" = 0 ] && echo "  ✅ Contextra Jules Environment bereit" || echo "  ❌ $FAILS Pflichtschritt(e) fehlgeschlagen"
echo "  Log: $LOG"; echo "============================================================"
exit $(( FAILS > 0 ))
