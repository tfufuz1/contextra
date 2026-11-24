#!/usr/bin/env bash
# Zitierprüfung (SPEC §15, INDEX §4 Regel 3).
#
# §15 uses compact families such as INV-ERA-1…12, whereas a test must cite an
# individual, searchable invariant ID.  Expand the normative table itself so a
# future SPEC edit cannot silently leave the checker with a stale inventory.
set -euo pipefail
cd "$(dirname "$0")/.."

test -f SPEC.md
test -d crates/contextra-storage/tests

rows=$(awk '/^## 15\./ { in_section = 1; next } /^## 16\./ { in_section = 0 } in_section { print }' SPEC.md)
compact=$(grep -oE 'INV-[A-Z]+-[0-9]+([/…][0-9]+)*' <<<"$rows" | sort -u || true)
test -n "$compact" || { echo "FEHLER: keine INV-IDs in SPEC §15" >&2; exit 1; }

expected=''
while read -r token; do
    prefix=${token%%-[0-9]*}
    suffix=${token#"$prefix"-}
    if [[ "$suffix" == *…* ]]; then
        first=${suffix%%…*}
        last=${suffix##*…}
        for ((number = first; number <= last; number++)); do
            expected+=" $prefix-$number"
        done
    else
        IFS=/ read -r -a numbers <<<"$suffix"
        for number in "${numbers[@]}"; do
            expected+=" $prefix-$number"
        done
    fi
done <<<"$compact"

found=$(grep -rhoE 'INV-[A-Z]+-[0-9]+' crates/contextra-storage/tests 2>/dev/null | sort -u || true)
missing=0
for inv in $expected; do
    if ! grep -qx "$inv" <<<"$found"; then
        echo "FEHLER: fehlende Testreferenz: $inv" >&2
        missing=1
    fi
done

if [ "$missing" -ne 0 ]; then
    exit 1
fi

echo "OK: alle $(wc -w <<<"$expected") normativen INV-IDs aus SPEC §15 sind in Tests referenziert"
