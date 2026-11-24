#!/usr/bin/env bash
# Einziger Einstiegspunkt für Build, Test und Navigation. Ausgaben sind begrenzt.
#   bash tools/ctx.sh status            Git, Toolchain, Speicher, Platte
#   bash tools/ctx.sh check <crate>     cargo check --locked -p <crate>  (nur Fehler/Warnungen)
#   bash tools/ctx.sh test <crate> [f]  cargo test --locked -p <crate> [f] (nur Ergebnisse)
#   bash tools/ctx.sh gates             Konsistenz capabilities.toml <-> Workspace
#   bash tools/ctx.sh find <regex> [p]  Suche in .rs/.md/.toml, max CTX_MAX_LINES Treffer
#   bash tools/ctx.sh space             regenerierbare Artefakte löschen, Page-Cache freigeben
# CTX_MAX_LINES (Standard 60) begrenzt jede Ausgabe.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"; cd "$ROOT"
MAX="${CTX_MAX_LINES:-60}"
env_setup(){ eval "$(bash tools/env/bootstrap.sh 2>/dev/null)" || { echo "bootstrap fehlgeschlagen"; exit 1; }; }
cargo_run(){  # $1 = Logdatei, Rest = cargo-Argumente
  local log="$1"; shift
  env_setup
  cargo "$@" >"$log" 2>&1; echo "exit=$?  (vollständiges Log: $log)"
}
case "${1:-status}" in
  status)
    echo "branch: $(git rev-parse --abbrev-ref HEAD 2>/dev/null)  head: $(git rev-parse --short HEAD 2>/dev/null)"
    echo "geändert: $(git status --porcelain | wc -l) Dateien"
    if [ -x "$HOME/rust189/bin/cargo" ]; then echo "toolchain: $($HOME/rust189/bin/cargo --version)"; else echo "toolchain: nicht installiert (tools/env/bootstrap.sh)"; fi
    free -m | awk '/Mem/{printf "ram: frei %s MB, verfügbar %s MB\n",$4,$7}'
    df -h / | awk 'NR==2{printf "platte: %s frei von %s\n",$4,$2}'
    ;;
  check)
    [ -n "${2:-}" ] || { echo "usage: check <crate>"; exit 2; }
    cargo_run /tmp/ctx_check_"$2".log check --locked -p "$2"
    grep -E '^(error|warning)' /tmp/ctx_check_"$2".log | sort | uniq -c | sort -rn | head -n "$MAX"
    ;;
  test)
    [ -n "${2:-}" ] || { echo "usage: test <crate> [filter]"; exit 2; }
    cargo_run /tmp/ctx_test_"$2".log test --locked -p "$2" ${3:-}
    grep -E 'test result|FAILED|panicked|^error' /tmp/ctx_test_"$2".log | head -n "$MAX"
    ;;
  gates)
    python3 tools/ctx_index.py gates ;;
  find)
    [ -n "${2:-}" ] || { echo "usage: find <regex> [pfad]"; exit 2; }
    P="${3:-crates}"
    if command -v rg >/dev/null 2>&1; then rg -n --max-count 3 -g '*.{rs,md,toml}' -- "$2" "$P"
    else grep -rnE --include='*.rs' --include='*.md' --include='*.toml' -- "$2" "$P"; fi | head -n "$MAX" ;;
  space)
    rm -rf target/debug/incremental target/release/incremental 2>/dev/null
    rm -rf "$HOME/micromamba/pkgs" 2>/dev/null
    sync; echo 1 > /proc/sys/vm/drop_caches 2>/dev/null && echo "page-cache freigegeben" || echo "drop_caches nicht möglich (kein root)"
    df -h / | awk 'NR==2{printf "platte jetzt: %s frei\n",$4}' ;;
  *) echo "unbekannter Befehl: $1"; sed -n '2,11p' "$0"; exit 2 ;;
esac
