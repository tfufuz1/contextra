# Contextra — Jules Playbook
> Stand: 2026-10-19 | Zentrale Prozessanweisung für Jules-Session-Ausführung

## 📚 1. Context Ladeliste
Jules lädt zu Session-Beginn ausschließlich folgende Kontext-Dateien:
1. `AGENTS.md` (Repo-Root & Crate-spezifisch in `crates/<crate>/AGENTS.md`)
2. `.jules/context/SESSION_START.md` & `.jules/context/GITHUB_STATE.md` (falls vorhanden)
3. Zuweisende Task-Karte (z. B. `.jules/tasks/T-2026-9001.toml`)

## ⚡ 2. Session Phasen & Reale Harness-Befehle
| Phase | Phasen-Befehl | Untergeordnete Gates / Kommandos |
|---|---|---|
| **start** | `cargo xtask jules start --card <karte>` | `task-card lint`, `claim`, `context-pack` (`env-attest` optional) |
| **check** | `cargo xtask jules check` | `cargo fmt --check`, `cargo clippy`, `scope-guard`, `protected-paths`, `check-duplicate-symbols`, `symbol-exists` |
| **verify** | `cargo xtask jules verify` | `cargo test`, `check-dag`, `check-ring-layering`, `check-vetoes`, `ratchet check` (`determinism-check`, `unsafe-audit`, `lock-audit` optional) |
| **submit** | `cargo xtask jules submit` | `gate-weakening`, `test-integrity`, `session-report` (`diff-budget`, `claims-in-pr`, `ledger` optional) |
| **stop** | `cargo xtask jules stop --reason <grund>` | `loop-guard stop` |

## 📐 3. Strikte Arbeitsregeln
- **Scope-Regel**: Ändere ausschließlich Dateien im `scope` der Task-Karte. Nicht erlaubte Pfade werden von `scope-guard` & `protected-paths` blockiert. Out-of-scope Befunde unter "Out-of-scope Findings" im PR melden.
- **Thrash-Regel**: Derselbe Fehler zweimal → STOPP. Keine blinden Korrekturversuche. Breche ab mit `cargo xtask jules stop --reason thrash`.
- **Gate-Schutz**: Niemals Gates, Lints, Baselines oder Toolchain abschwächen oder umgehen. Behebe stets die Ursache im Code.

## 🔄 4. PR-Überarbeitung per Kommentar
1. **Feedback lesen**: Lies den sticky PR-Kommentar mit Marker `<!-- contextra-verdict -->` und offene Review-Threads.
2. **Ursache beheben**: Repariere die Ursache im Code/Dokument. Niemals ein Gate oder eine Baseline abschwächen!
3. **Lokal verifizieren**: Führe `cargo xtask jules check` und `cargo xtask jules verify` erneut lokal aus.
4. **Push & Antworten**: Pushe die Korrektur auf denselben Branch. Antworte pro erledigtem Review-Thread mit genau einem prägnanten Satz.
