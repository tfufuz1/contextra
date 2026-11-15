#!/usr/bin/env bash
# SessionStart-Hook: stellt Toolchain und Werkzeuge sicher und gibt einen kurzen Statusbericht aus
# (stdout landet im Kontext von Claude). Schnell, wenn alles vorhanden ist; sonst ca. 1 bis 2 Minuten.
cd "$(dirname "$0")/.." || exit 0
export PATH="$HOME/.cargo/bin:$PATH"
log() { printf '%s\n' "$*"; }
if ! rustc --version 2>/dev/null | grep -q '^rustc 1\.89\.0'; then
  if timeout 8 curl -fsS -o /dev/null -m 6 https://static.rust-lang.org/dist/channel-rust-1.89.0.toml.sha256 2>/dev/null; then
    log "[setup] Rust 1.89.0 fehlt, rustup kann es laden: rustup toolchain install 1.89.0 (nicht automatisch ausgefuehrt)"
  else
    log "[setup] Rust 1.89.0 fehlt und static.rust-lang.org ist blockiert, starte scripts/bootstrap-toolchain.sh"
    bash scripts/bootstrap-toolchain.sh >/tmp/acta-bootstrap-toolchain.log 2>&1 && log "[setup] Toolchain bereit" || log "[FEHLER] Toolchain-Bootstrap fehlgeschlagen oder nicht anwendbar, siehe /tmp/acta-bootstrap-toolchain.log"
  fi
fi
if ! command -v just >/dev/null 2>&1 || ! command -v cargo-deny >/dev/null 2>&1 || ! command -v cargo-nextest >/dev/null 2>&1; then
  log "[setup] Werkzeuge fehlen, starte scripts/bootstrap-tools.sh"
  bash scripts/bootstrap-tools.sh >/tmp/acta-bootstrap-tools.log 2>&1 && log "[setup] Werkzeuge bereit" || log "[FEHLER] Werkzeug-Bootstrap fehlgeschlagen, siehe /tmp/acta-bootstrap-tools.log"
fi
log "== Acta Umgebung =="
log "rustc: $(rustc --version 2>&1 | head -1)"
log "cargo: $(cargo --version 2>&1 | head -1)"
for t in just cargo-deny cargo-audit cargo-nextest cargo-mutants cargo-llvm-cov cargo-cyclonedx cargo-geiger; do
  if command -v "$t" >/dev/null 2>&1; then s=ok; else s=FEHLT; fi
  printf '%s=%s ' "$t" "$s"
done; log ""
if timeout 5 curl -fsS -o /dev/null -m 4 https://index.crates.io/config.json 2>/dev/null; then log "netz: crates.io erreichbar"; else log "netz: crates.io NICHT erreichbar (Offline-Build nur mit vorhandenem ~/.cargo/registry)"; fi
log "git: $(git rev-parse --abbrev-ref HEAD 2>/dev/null) $(git status --short 2>/dev/null | wc -l) geaenderte Dateien"
log "Nicht verfuegbar in dieser VM: nightly/miri/cargo-fuzz (static.rust-lang.org blockiert). Details: docs/UMGEBUNG.md"
log "Start: CLAUDE.md lesen; Pruefung: /gates; Experten: .claude/agents/ (Skill acta-experts)"
exit 0
