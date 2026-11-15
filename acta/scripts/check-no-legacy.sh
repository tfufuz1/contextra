#!/usr/bin/env bash
# Prueft, dass keine Alt-Kennungen im Baum stehen (Projektname, Alt-Magics, Alt-IDs, Alt-Pfade). Exit 1 bei Treffern.
set -u
cd "$(dirname "$0")/.."
fail=0
chk(){ local n="$1" p="$2"; shift 2
  if grep -rIlE "$p" . --exclude-dir=.git --exclude-dir=target "$@" 2>/dev/null | grep -v '^./scripts/check-no-legacy.sh$' | head -5 | grep -q .; then
    echo "TREFFER [$n]:"; grep -rInE "$p" . --exclude-dir=.git --exclude-dir=target "$@" | grep -v 'scripts/check-no-legacy.sh' | head -5; fail=1; fi; }
chk projektname 'contextra|kontextra' -i
chk alt-magic   '\bMF[A-Z0-9]{2}[A-Z0-9]*\b' --include=*.rs
chk alt-ids     '\bINV-[A-Z0-9]+|\bADR-N[0-9]+' --include=*.rs
# bereits mit TODO(acta-altlast) markierte Dateien sind bekannte Altlasten (ALTLASTEN.md)
chk alt-schluessel 'CONTEXTRA_|MF_KEY'
for f in $(grep -rIl 'wal_integrity_key' . --exclude-dir=.git --exclude-dir=target --exclude=check-no-legacy.sh 2>/dev/null); do grep -q 'TODO(acta-altlast)' "$f" || { echo "TREFFER [schluesseldatei]: $f"; fail=1; }; done
find . -path ./.git -prune -o \( -iname '*contextra*' -o -name '*.bin' -o -name '.wal_integrity_key' \) -print | grep -q . && { echo "TREFFER [dateien]"; fail=1; }
[ $fail -eq 0 ] && echo "OK: keine Altspuren gefunden."
exit $fail
