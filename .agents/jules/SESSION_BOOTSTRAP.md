# Contextra – Ablauf einer Jules-Sitzung

Nicht normativ. Maßgeblich: [`docs/SPEC.md`](../../docs/SPEC.md) (Version 4.2.0). Repository: `tfufuz1/contextra`.

## Schritt 0 – Kontext prüfen

Vor dem Code den aktuellen Zustand auf GitHub prüfen. Die gh-CLI ist die bevorzugte Quelle.

```bash
# Offene Aufgaben für Jules
gh issue list --state open --label "jules" --json number,title,body,milestone

# Details zu einer Aufgabe
gh issue view <N>

# Rote CI-Läufe und ihre Ursache
gh run list --limit 5
gh run view <RUN-ID> --log-failed

# Neueste Pull Requests
gh pr list --state all --limit 10
```

**Stopbedingung:** Ist `main` rot, wird zuerst die Ursache behoben. Anderer Code wird vorher nicht geschrieben.

## Schritt 1 – Regeln lesen

`AGENTS.md`, `.agents/jules/PREAMBLE.md`, und der Abschnitt der SPEC zur Aufgabe. Für Änderungen am Löschkern zusätzlich `.agents/rules/deletion_pipeline.md`. Für WAL und Krypto `.agents/rules/wal_crypto.md`.

## Schritt 2 – Arbeitspaket bestimmen

Die Aufgabe muss einem Arbeitspaket aus SPEC 15.4 entsprechen (`WP-x.y`). Passt keines, wird die Aufgabe nicht begonnen. Rückfrage im Issue.

## Schritt 3 – Rot

Fehlschlagenden Test mit SPEC-ID schreiben (`#[ctx_test(id = "T-…")]`) und die Ausgabe festhalten.

## Schritt 4 – Grün

Minimale Änderung. Keine Änderung an Gates, Lints, Baselines, `governance/` oder Toolchain-Pins.

## Schritt 5 – Prüfen

Befehle aus `docs/COMPILATION.md`. Bei geändertem `Cargo.toml` wird `Cargo.lock` im selben Commit erneuert (D-93) und mit `--locked` geprüft.

## Schritt 6 – Pull Request

Die Beschreibung enthält: Arbeitspaket, betroffene SPEC-IDs, Testliste, Prüfergebnisse, Abschnitt „Gefundene Probleme außerhalb des Umfangs“. Die Vorlage für PRs ist `.github/pull_request_template.md`.

## Abbruchregel

Tritt derselbe Fehler zweimal auf, wird das Vorgehen überprüft und die Sitzung angehalten. Die Ursache wird im Issue beschrieben.
