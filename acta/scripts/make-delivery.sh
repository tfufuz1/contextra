#!/usr/bin/env bash
# Baut die Lieferung: ZIP nur mit bearbeiteten/neuen Dateien (Pfade ab Repo-Wurzel) und
# geloeschten Dateien als LEERE Dateien (damit der Empfaenger die Huellen lokal entfernen kann).
# Aufruf: scripts/make-delivery.sh [ziel.zip]   (Standard: /root/acta-lieferung.zip, ausserhalb des Repos)
set -euo pipefail
cd "$(dirname "$0")/.."
OUT="${1:-/root/acta-lieferung.zip}"
STAGE="$(mktemp -d)"; trap 'rm -rf "$STAGE"' EXIT
mapfile -t CHANGED < <(git ls-files -m -o --exclude-standard | sort -u)
n_ch=0; n_del=0
for f in "${CHANGED[@]}"; do
  case "$f" in target/*|*/target/*|.git/*) continue;; esac
  if [ -e "$f" ]; then
    mkdir -p "$STAGE/$(dirname "$f")"; cp -p "$f" "$STAGE/$f"; n_ch=$((n_ch+1))
  else
    mkdir -p "$STAGE/$(dirname "$f")"; : > "$STAGE/$f"; n_del=$((n_del+1))   # entfernt -> leere Huelle
  fi
done
# Verzeichnisse mit leer gemachten Dateien: auch zuvor entfernte, noch nicht committete (git ls-files -d)
while IFS= read -r f; do
  [ -e "$STAGE/$f" ] || { mkdir -p "$STAGE/$(dirname "$f")"; : > "$STAGE/$f"; n_del=$((n_del+1)); }
done < <(git ls-files -d)
rm -f "$OUT"
( cd "$STAGE" && [ -n "$(ls -A)" ] && zip -qr "$OUT" . ) || { echo "Keine Aenderungen, keine Lieferung."; exit 0; }
echo "Lieferung: $OUT  (bearbeitet/neu: $n_ch, geleert/entfernt: $n_del)"
unzip -l "$OUT" | tail -1
