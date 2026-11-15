---
description: "Vollstaendige Qualitaetspruefung (check, gates, test, clippy, fmt) mit ehrlichem Bericht"
allowed-tools: Bash(cargo:*), Bash(bash scripts/bootstrap-toolchain.sh)
---
Fuehre den Skill `acta-gates` aus. Stelle zuerst sicher, dass `rustc --version` 1.89.0 zeigt (sonst `bash scripts/bootstrap-toolchain.sh`).
Starte lange Laeufe im Hintergrund mit Log in den Scratchpad, werte den Exit-Code je Schritt getrennt aus (nie ueber eine Pipe).
Berichte pro Schritt ok/FAIL mit Datei:Zeile bei Befund. Nicht Gelaufenes als "nicht geprueft" kennzeichnen. $ARGUMENTS
