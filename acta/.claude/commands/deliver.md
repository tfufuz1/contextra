---
description: "ZIP mit allen bearbeiteten Dateien erzeugen und ausliefern (entfernte Dateien als leere Huellen)"
allowed-tools: Bash(bash scripts/make-delivery.sh:*), Bash(unzip -l:*), Bash(git status:*), Bash(cargo:*)
---
Fuehre den Skill `acta-delivery` aus: erst `/gates`, dann `bash scripts/make-delivery.sh`, Inhalt mit `unzip -l` kontrollieren, mit SendUserFile ausliefern. Nenne Anzahl, Pruefstand und Offenes. $ARGUMENTS
