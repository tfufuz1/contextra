# GitHub als primäre Informationsquelle

Jules MUSS die folgenden GitHub-Quellen BEVORZUGEN gegenüber statischen Repo-Dateien:

## Lesen (bevor statische Docs konsultiert werden)
- `gh issue list --label "context" --json number,title,body,labels`
- `gh pr list --state open --json number,title,body,labels,status`
- `gh issue view <N>` für spezifische Aufgaben-Details
- `gh run list --workflow=<name>` für CI-Status
- `gh release list` für Version-History

## Schreiben (Protokollierung aller Agenten-Aktionen)
- Session-Start: `gh issue comment <session-issue> --body "🤖 Session gestartet: <timestamp>"`
- Task-Completion: `gh issue close <task-issue> --comment "✅ Erledigt: <summary>"`
- Blocker melden: `gh issue create --title "🚫 Blocker: <title>" --label "blocker,agent-reported"`
- PR öffnen: `gh pr create --title "<wp>: <summary>" --body "<details>"`

## Konventionen
- Alle Agenten-Aktionen werden in GitHub-Issues/PRs dokumentiert
- Labels: `agent-reported`, `context`, `task`, `blocker`, `session`
- Session-Issues: `gh issue create --title "🤖 Session <date>" --label "session,agent-reported"`
- Keine lokalen Log-Dateien mehr als primäre Protokolle (JULES_LOG.md ist sekundär)
