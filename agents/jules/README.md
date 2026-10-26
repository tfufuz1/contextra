# agents/jules/ — Jules Agenten-Konfiguration

<!-- doc-stamp: 2026-10-26 | Normativ: AGENTS.md §2a -->

> [!NOTE]
> Dieses Verzeichnis ist Root-Level (`agents/jules/`, ohne Punkt).
> Die Legacy-Kopie unter `.agents/jules/` bleibt bis zur vollständigen Migration erhalten.

## Inhalt

| Datei/Verzeichnis | Zweck |
|-------------------|-------|
| `SESSION_BOOTSTRAP.md` | Session-Ablauf für Jules (AGENTS.md §2a) |
| `GITHUB_WORKFLOW.md` | GitHub-Konventionen, Labels, Protokollpflicht |
| `context/SESSION_START.md` | Dynamisch generierter Kontext (von refresh-context.sh) |
| `setup/refresh-context.sh` | Generiert SESSION_START.md |

## Nutzung (Jules Session-Start)

```bash
# 1. Kontext lesen
cat agents/jules/context/SESSION_START.md

# 2. Konkreten Task von GitHub holen
gh issue list --state open --label "jules" --json number,title,body,milestone
gh issue view <N>

# 3. Build-Grundlage prüfen
cargo check --workspace
```

## Wichtige Dateien außerhalb dieses Verzeichnisses

- `AGENTS.md` — Vollständige Agenten-Betriebsanleitung (normativ)
- `docs/SPEC.md` — Einzige normative Architekturquelle
- `governance/gates.toml` — Gate-Definitionen
- `capabilities.toml` — Ring/Klassen-Zuordnung je Crate
