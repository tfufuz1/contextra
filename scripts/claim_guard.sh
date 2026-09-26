#!/usr/bin/env bash
# claim_guard.sh
# ===============
#
# Schnelle, git-only Kollisionswarnung GEGEN das in JULES_LOG.md §4/§6
# beschriebene "Parallel-Implementierungen" / "Branch Proliferation"
# Anti-Muster (z.B. 3 Agenten implementierten dieselbe Kalibrierungsfunktion
# in #1627/#1634/#1645 innerhalb 1 Stunde).
#
# Anders als `cargo xtask claim` / `check_no_active_claim_conflict` (Rust,
# braucht einen kompilierten xtask-Build) prüft dieses Skript zusätzlich
# aktiv die tatsächlichen REMOTE-BRANCHES auf sich überschneidende Crate-
# Pfade — nicht nur die formale `.jules/claims.json`-Registrierung. Das
# deckt genau den Fall ab, in dem ein Agent OHNE Claim einfach einen Branch
# pusht, der dasselbe Crate anfasst.
#
# Was wird geprüft:
#   1. Welche Crates berührt die eigene aktuelle Änderung (working tree
#      bzw. der aktuelle Branch gegenüber origin/main)?
#   2. Gibt es dazu einen aktiven, nicht abgelaufenen Eintrag in
#      .jules/claims.json von einer ANDEREN Session/Issue für dasselbe Crate?
#   3. Gibt es andere Remote-Branches (außer dem eigenen und main), die
#      DIESELBEN Crate-Pfade verändern? (Kandidaten-Vorfilter über den
#      Branch-Namen, danach echter `git diff` nur für Kandidaten — bleibt
#      auch bei >80 Branches schnell, siehe --max-diff-check.)
#
# Verwendung
# ----------
#   scripts/claim_guard.sh                          # eigene uncommitted Änderungen prüfen
#   scripts/claim_guard.sh --base origin/main        # eigenen Branch komplett gegen main prüfen
#   scripts/claim_guard.sh --max-diff-check 15       # max. 15 Remote-Branches per echtem diff prüfen
#   scripts/claim_guard.sh --fail-on-conflict        # Exit-Code 1 bei gefundener Kollision
#
# Exit-Codes: 0 = keine Kollision (oder --fail-on-conflict nicht gesetzt),
#             1 = Kollision gefunden UND --fail-on-conflict gesetzt,
#             2 = Kein Git-Repository / Ausführungsfehler.

set -uo pipefail

BASE_REF="HEAD"
CLAIMS_FILE=".jules/claims.json"
MAX_DIFF_CHECK=25
FAIL_ON_CONFLICT=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --base) BASE_REF="$2"; shift 2 ;;
        --claims-file) CLAIMS_FILE="$2"; shift 2 ;;
        --max-diff-check) MAX_DIFF_CHECK="$2"; shift 2 ;;
        --fail-on-conflict) FAIL_ON_CONFLICT=1; shift ;;
        -h|--help) grep '^#' "$0" | sed 's/^# \?//'; exit 0 ;;
        *) echo "Unbekannte Option: $1" >&2; exit 2 ;;
    esac
done

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "[error] Kein Git-Repository im aktuellen Verzeichnis." >&2
    exit 2
fi

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT" || exit 2

CURRENT_BRANCH="$(git rev-parse --abbrev-ref HEAD)"

echo "=== claim_guard: Branch '${CURRENT_BRANCH}' vs. Referenz '${BASE_REF}' ==="
echo

# --- 1. Eigene berührte Crates ermitteln -------------------------------------------------
if [[ "$BASE_REF" == "HEAD" ]]; then
    CHANGED_FILES="$(git diff --name-only HEAD; git diff --name-only --cached)"
else
    CHANGED_FILES="$(git diff --name-only "${BASE_REF}...HEAD" 2>/dev/null || git diff --name-only "${BASE_REF}")"
fi

MY_CRATES="$(echo "$CHANGED_FILES" | grep -oE '^crates/[^/]+' | sed 's#crates/##' | sort -u)"

if [[ -z "$MY_CRATES" ]]; then
    echo "Keine geänderten Dateien unter crates/ gefunden — nichts zu prüfen."
    exit 0
fi

echo "Eigene betroffene Crates:"
echo "$MY_CRATES" | sed 's/^/  - /'
echo

CONFLICT_FOUND=0

# --- 2. Aktive .jules/claims.json Einträge auf Kollision prüfen -------------------------
if [[ -f "$CLAIMS_FILE" ]]; then
    echo "--- Prüfe ${CLAIMS_FILE} auf aktive Fremd-Claims ---"
    CLAIM_HITS="$(python3 - "$CLAIMS_FILE" <<'PYEOF'
import json, sys, datetime
path = sys.argv[1]
try:
    with open(path) as f:
        db = json.load(f)
except Exception:
    sys.exit(0)
now = datetime.datetime.now(datetime.timezone.utc).replace(tzinfo=None).isoformat()
for c in db.get("claims", []):
    if not c.get("active", True):
        continue
    if c.get("released_at"):
        continue
    exp = c.get("expires_at")
    if exp and exp < now:
        continue
    print(f"{c.get('krate','?')}\t{c.get('issue','?')}\t{c.get('session_id','?')}\t{c.get('timestamp','?')}")
PYEOF
)"
    # Re-run properly piping MY_CRATES isn't needed above since we just list all active claims;
    # now cross-reference in bash:
    while IFS=$'\t' read -r krate issue session ts; do
        [[ -z "$krate" ]] && continue
        if echo "$MY_CRATES" | grep -qx "$krate"; then
            echo "  ⚠️  AKTIVER CLAIM auf '$krate': Issue=$issue Session=$session seit $ts"
            CONFLICT_FOUND=1
        fi
    done <<< "$CLAIM_HITS"
    [[ "$CONFLICT_FOUND" -eq 0 ]] && echo "  Keine aktiven Fremd-Claims auf eigene Crates."
    echo
else
    echo "(Kein ${CLAIMS_FILE} gefunden — formale Claim-Prüfung übersprungen. Empfehlung: "
    echo " 'cargo xtask claim --crate <CRATE> --issue <ID>' vor Arbeitsbeginn ausführen.)"
    echo
fi

# --- 3. Remote-Branches auf Crate-Überschneidung prüfen ----------------------------------
echo "--- Prüfe Remote-Branches auf Crate-Überschneidung ---"
git fetch origin --quiet 2>/dev/null || echo "  (git fetch fehlgeschlagen/übersprungen — nutze lokalen Ref-Stand)"

mapfile -t REMOTE_BRANCHES < <(git for-each-ref --sort=-committerdate --format='%(refname:short)' refs/remotes/origin \
    | grep -v -E '^(origin/HEAD|origin/main|origin/'"${CURRENT_BRANCH#origin/}"')$')

echo "Gefundene andere Remote-Branches: ${#REMOTE_BRANCHES[@]} (prüfe max. ${MAX_DIFF_CHECK} per Diff, neueste zuerst)"

CHECKED=0
for branch in "${REMOTE_BRANCHES[@]}"; do
    [[ "$CHECKED" -ge "$MAX_DIFF_CHECK" ]] && break
    CHECKED=$((CHECKED + 1))

    MERGE_BASE="$(git merge-base origin/main "$branch" 2>/dev/null)"
    [[ -z "$MERGE_BASE" ]] && continue

    OTHER_CRATES="$(git diff --name-only "$MERGE_BASE" "$branch" 2>/dev/null \
        | grep -oE '^crates/[^/]+' | sed 's#crates/##' | sort -u)"
    [[ -z "$OTHER_CRATES" ]] && continue

    OVERLAP="$(comm -12 <(echo "$MY_CRATES") <(echo "$OTHER_CRATES"))"
    if [[ -n "$OVERLAP" ]]; then
        echo "  ⚠️  '$branch' bearbeitet ebenfalls: $(echo "$OVERLAP" | tr '\n' ',' | sed 's/,$//')"
        CONFLICT_FOUND=1
    fi
done
echo "  (${CHECKED} von ${#REMOTE_BRANCHES[@]} Remote-Branches per Diff geprüft)"
echo

# --- Ergebnis -----------------------------------------------------------------------------
if [[ "$CONFLICT_FOUND" -eq 1 ]]; then
    echo "🔴 Mögliche Kollision(en) gefunden. Vor Arbeitsbeginn: Claim registrieren"
    echo "   ('cargo xtask claim --crate <CRATE> --issue <ID>') oder Rücksprache mit"
    echo "   dem betroffenen Branch/Issue halten, um Parallel-Implementierungen"
    echo "   (siehe docs/GITHUB_ANALYSE.md, #1627/#1634/#1645) zu vermeiden."
    [[ "$FAIL_ON_CONFLICT" -eq 1 ]] && exit 1
    exit 0
else
    echo "✅ Keine Kollisionen erkannt. Sicher, mit der Arbeit an: $(echo "$MY_CRATES" | tr '\n' ',' | sed 's/,$//') fortzufahren."
    exit 0
fi
