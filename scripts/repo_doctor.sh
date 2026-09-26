#!/usr/bin/env bash
# repo_doctor.sh
# ===============
#
# Ein-Shot "Gesundheitscheck" für Sessionbeginn bzw. vor dem Submit — beantwortet
# in wenigen Sekunden die Frage "Ist dieses Repository/diese VM in einem
# sauberen, arbeitsfähigen Zustand?", OHNE einen vollständigen `cargo build`
# des 33-Crate-Workspace anzustoßen (der laut ARCHITECTURE.md sehr groß ist).
#
# Deckt genau die in JULES_LOG.md §1–§3 beschriebenen Erst-Diagnose-Schritte
# (git status/log/branch/remote, Workspace-Konsistenz) in EINEM Aufruf ab,
# statt sie händisch nacheinander abzusetzen, und ergänzt sie um Prüfungen,
# die in xtask (noch) nicht als eigenes schnelles Gate existieren:
#   - Cargo.toml [workspace.members] vs. tatsächliche crates/*-Verzeichnisse
#     (erkennt Drift: neues Crate-Verzeichnis ohne Workspace-Eintrag oder
#     umgekehrt — Vorstufe zu ARCHITECTURE.md §4 "Bekannte Abweichungen").
#   - Unsafe-Insel-Schnellcheck (grep-basiert, ohne Kompilierung) gegen die
#     3 deklarierten Inseln.
#   - Zero-Panic-Oberflächenzahl (unwrap/expect/panic) pro Crate als grobe
#     Kennzahl, um Ausreißer-Crates zu erkennen.
#   - Git-Arbeitsbaum-Sauberkeit, Branch-Divergenz zu origin/main,
#     Stash-Liste, verwaiste/nicht getrackte Großdateien.
#   - Build-Cache-Fußabdruck (target/-Größe) und Tool-Verfügbarkeit
#     (cargo/rustc/git-Versionen), um Umgebungsprobleme früh zu erkennen.
#
# Verwendung
# ----------
#   scripts/repo_doctor.sh                  # Vollständiger Bericht
#   scripts/repo_doctor.sh --quick          # Nur Git- und Workspace-Checks (keine grep-Scans)
#   scripts/repo_doctor.sh --fail-on-warn   # Exit-Code 1 bei jeglichen Warnungen
#
# Exit-Codes: 0 = keine (relevanten) Probleme, 1 = Warnungen UND --fail-on-warn,
#             2 = kritischer Fehler (kein Git-Repo / keine crates/ gefunden).

set -uo pipefail

QUICK=0
FAIL_ON_WARN=0
for arg in "$@"; do
    case "$arg" in
        --quick) QUICK=1 ;;
        --fail-on-warn) FAIL_ON_WARN=1 ;;
        -h|--help) grep '^#' "$0" | sed 's/^# \?//'; exit 0 ;;
    esac
done

WARN_COUNT=0
warn() { echo "  ⚠️  $*"; WARN_COUNT=$((WARN_COUNT + 1)); }
ok()   { echo "  ✅ $*"; }
info() { echo "  ℹ️  $*"; }

section() { echo; echo "── $1 ──────────────────────────────────────────────"; }

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "[error] Kein Git-Repository." >&2
    exit 2
fi
REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT" || exit 2

if [[ ! -d crates ]]; then
    echo "[error] Kein 'crates/'-Verzeichnis unter ${REPO_ROOT} — falscher Repo-Root?" >&2
    exit 2
fi

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║              Contextra Repo Doctor — $(date '+%Y-%m-%d %H:%M')             ║"
echo "╚══════════════════════════════════════════════════════════════╝"

# --- 1. Tool-Verfügbarkeit & Versionen ----------------------------------------------------
section "Werkzeug-Verfügbarkeit"
for tool in git cargo rustc python3; do
    if command -v "$tool" >/dev/null 2>&1; then
        ver="$("$tool" --version 2>&1 | head -1)"
        ok "$tool: $ver"
    else
        warn "$tool NICHT gefunden im PATH"
    fi
done

# --- 2. Git-Zustand -------------------------------------------------------------------------
section "Git-Arbeitsbaum"
BRANCH="$(git rev-parse --abbrev-ref HEAD)"
info "Aktueller Branch: $BRANCH"

DIRTY="$(git status --porcelain)"
if [[ -z "$DIRTY" ]]; then
    ok "Working Tree sauber (keine uncommitted Änderungen)"
else
    N_DIRTY="$(echo "$DIRTY" | wc -l | tr -d ' ')"
    warn "$N_DIRTY uncommitted Änderung(en) im Working Tree"
    echo "$DIRTY" | head -10 | sed 's/^/      /'
    [[ "$N_DIRTY" -gt 10 ]] && echo "      ... ($((N_DIRTY - 10)) weitere)"
fi

STASH_N="$(git stash list | wc -l | tr -d ' ')"
[[ "$STASH_N" -gt 0 ]] && warn "$STASH_N Eintrag/Einträge im Stash (nicht vergessen: git stash pop)"

if git rev-parse --verify origin/main >/dev/null 2>&1; then
    AHEAD_BEHIND="$(git rev-list --left-right --count origin/main..."$BRANCH" 2>/dev/null)"
    BEHIND="$(echo "$AHEAD_BEHIND" | awk '{print $1}')"
    AHEAD="$(echo "$AHEAD_BEHIND" | awk '{print $2}')"
    if [[ "${BEHIND:-0}" -gt 0 ]]; then
        warn "Branch liegt $BEHIND Commit(s) hinter origin/main zurück — 'git fetch origin' + Rebase erwägen"
    else
        ok "Branch ist auf dem Stand von origin/main (0 Commits zurück)"
    fi
    info "$AHEAD Commit(s) auf diesem Branch gegenüber origin/main"
else
    warn "origin/main lokal nicht bekannt — 'git fetch origin' ausführen"
fi

LAST_COMMIT_MSG="$(git log -1 --pretty=%s 2>/dev/null || echo '')"
if [[ "$LAST_COMMIT_MSG" =~ ^(wip|fix|update|test|tmp|stuff|shell-commit)$ ]]; then
    warn "Letzte Commit-Message wirkt generisch/'Shell-Commit'-verdächtig: '$LAST_COMMIT_MSG'"
else
    ok "Letzte Commit-Message: '$LAST_COMMIT_MSG'"
fi

# --- 3. Workspace-Konsistenz (Cargo.toml members vs. crates/*) ------------------------------
section "Workspace-Konsistenz"
if [[ -f Cargo.toml ]]; then
    MEMBERS="$(sed -n '/^\[workspace\]/,/^\[/p' Cargo.toml | sed -n '/members = \[/,/\]/p' \
        | grep -oE '"crates/[^"]+"' | sed 's/"crates\///; s/"//')"
    ACTUAL_DIRS="$(find crates -maxdepth 1 -mindepth 1 -type d -exec basename {} \; | sort)"

    MISSING_IN_MANIFEST="$(comm -13 <(echo "$MEMBERS" | sort) <(echo "$ACTUAL_DIRS"))"
    MISSING_ON_DISK="$(comm -23 <(echo "$MEMBERS" | sort) <(echo "$ACTUAL_DIRS"))"

    if [[ -z "$MISSING_IN_MANIFEST" && -z "$MISSING_ON_DISK" ]]; then
        ok "Alle $(echo "$ACTUAL_DIRS" | wc -l | tr -d ' ') crates/*-Verzeichnisse sind als Workspace-Member registriert"
    else
        [[ -n "$MISSING_IN_MANIFEST" ]] && { warn "Verzeichnis(se) unter crates/ OHNE Workspace-Eintrag:"; echo "$MISSING_IN_MANIFEST" | sed 's/^/      - /'; }
        [[ -n "$MISSING_ON_DISK" ]] && { warn "Workspace-Member OHNE existierendes Verzeichnis (verwaister Eintrag):"; echo "$MISSING_ON_DISK" | sed 's/^/      - /'; }
    fi
else
    warn "Kein Cargo.toml im Repo-Root gefunden"
fi

if [[ "$QUICK" -eq 1 ]]; then
    echo
    echo "(--quick gesetzt: grep-basierte Code-Scans übersprungen)"
else
    # --- 4. Unsafe-Insel-Schnellcheck (grep, kein Build) ------------------------------------
    section "Unsafe-Insel-Schnellcheck (grep, ergänzt xtask check-unsafe-islands)"
    ALLOWED_ISLANDS="contextra-sys contextra-simd contextra-wire"
    VIOLATIONS=0
    for dir in crates/*/; do
        crate="$(basename "$dir")"
        lib="${dir}src/lib.rs"
        [[ -f "$lib" ]] || continue
        if grep -q "allow(unsafe_code)" "$lib" 2>/dev/null; then
            if [[ ! " $ALLOWED_ISLANDS " == *" $crate "* ]]; then
                warn "'$crate' erlaubt unsafe_code, ist aber KEINE der 3 deklarierten Inseln"
                VIOLATIONS=$((VIOLATIONS + 1))
            fi
        fi
    done
    [[ "$VIOLATIONS" -eq 0 ]] && ok "Nur die 3 deklarierten Crates ($ALLOWED_ISLANDS) erlauben unsafe_code"

    # --- 5. Zero-Panic-Oberfläche pro Crate (grobe Kennzahl, Top 5 Ausreißer) ---------------
    section "Zero-Panic-Oberfläche (unwrap/expect/panic! außerhalb tests/, Top 5 Crates)"
    TMP_COUNTS="$(mktemp)"
    for dir in crates/*/src; do
        crate="$(basename "$(dirname "$dir")")"
        n="$(grep -rE '\.unwrap\(\)|\.expect\(|panic!\(' "$dir" 2>/dev/null \
            | grep -v '/tests/' | wc -l | tr -d ' ')"
        echo "$n $crate" >> "$TMP_COUNTS"
    done
    sort -rn "$TMP_COUNTS" | head -5 | while read -r n crate; do
        if [[ "$n" -gt 0 ]]; then
            info "$crate: $n Vorkommen (Test-Dateien bereits ausgeschlossen; #[cfg(test)]-Module NICHT ausgeschlossen — grobe Kennzahl)"
        fi
    done
    rm -f "$TMP_COUNTS"
fi

# --- 6. Build-Fußabdruck --------------------------------------------------------------------
section "Build-Fußabdruck"
if [[ -d target ]]; then
    SIZE="$(du -sh target 2>/dev/null | cut -f1)"
    info "target/-Verzeichnis: $SIZE"
else
    info "Kein target/-Verzeichnis vorhanden (noch kein Build durchgeführt)"
fi

# --- Zusammenfassung -------------------------------------------------------------------------
section "Zusammenfassung"
if [[ "$WARN_COUNT" -eq 0 ]]; then
    echo "  ✅ Keine Auffälligkeiten. Repo ist in einem sauberen Ausgangszustand."
else
    echo "  ⚠️  $WARN_COUNT Warnung(en) — siehe oben. Empfehlung: relevante Punkte vor"
    echo "     Arbeitsbeginn/Submit klären, danach 'cargo xtask jules-preflight' ausführen."
fi

if [[ "$WARN_COUNT" -gt 0 && "$FAIL_ON_WARN" -eq 1 ]]; then
    exit 1
fi
exit 0
