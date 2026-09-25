//! Gate: check-commit-diff-integrity (Phantom Commit Protection Gate)
//!
//! Prüft, ob Commit-Messages mit mehreren konkreten Point-Änderungsbehauptungen (≥ 3 Punkte)
//! tatsächlich entsprechende Änderungen im `git diff --stat` aufweisen.
//!
//! # Anlass & Rationale
//! In der Vergangenheit gab es nachgewiesene Fälle von LEEREN Commits mit erfundenen,
//! fünf Punkte umfassenden Commit-Messages ("Phantom Commits"). Dieses Gate verhindert,
//! dass leere oder stark unvollständige Commits mit umfangreichen Änderungsbehauptungen
//! gemerged werden.
//!
//! # Heuristik
//! - **Conventional Commit Prefix Stripping**: Falls eine Zeile mit einem Conventional Commit
//!   Header beginnt (z. B. `feat(scope): `, `fix: `), wird das Präfix vor der Auswertung entfernt.
//! - **Claim Points Count**: Eine Zeile gilt als eigene Änderungsbehauptung, wenn sie:
//!   - mit Stichpunkten beginnt: `-`, `*`, `+`, `•` oder `1.`, `2.` etc.
//!   - mit einem der definierten deutschen/englischen Aktionsverben beginnt (case-insensitive, als eigenständiges Wort):
//!     "implementiert", "fügt hinzu", "behebt", "entfernt", "ändert", "refaktoriert",
//!     "implements", "adds", "fixes", "removes", "changes", "refactors".
//! - **Prüfregel**:
//!   - Wenn `claim_points >= 3` UND `changed_files == 0`: **Harter FEHLER** (Phantom Commit).
//!   - Wenn `claim_points >= 3` UND `changed_files > 0` aber `changed_files < min_plausible_files`: **WARNUNG**
//!     (Konservativ: im Zweifel WARNEN statt hart fehlschlagen, um False-Positives bei legitimen
//!     Ein-Datei-Commits mit mehreren Teilpunkten zu vermeiden).
//!
//! # Known Limitations / False Positives & False Negatives
//! - **False Positives (Mögliche Über-Blockierung)**:
//!   - Ein sehr detaillierter Commit, der 3+ logische Punkte in genau einer einzigen Datei ändert
//!     (z. B. Refactoring in `main.rs`), erzeugt eine Warnung, aber KEINEN harten Fehler, da `changed_files > 0`.
//! - **False Negatives (Mögliche Unter-Blockierung)**:
//!   - Prosa-Texte ohne Stichpunkte oder Schlüsselverben werden eventuell nicht als ≥ 3 Punkte gezählt.
//!   - Ein Commit mit 2 erfundenen Punkten entgeht dem Schwellenwert 3.

use std::path::Path;
use std::process::Command;

/// Repräsentiert die Diff-Statistiken eines Commits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitDiffStats {
    pub changed_files: usize,
    pub insertions: usize,
    pub deletions: usize,
}

impl CommitDiffStats {
    pub fn total_lines_changed(&self) -> usize {
        self.insertions + self.deletions
    }
}

/// Ermittelt die Diff-Statistiken für einen gegebenen Commit-SHA mittels `git show --stat --format="" <sha>`.
pub fn get_commit_diff_stats(sha: &str, repo_dir: Option<&Path>) -> Result<CommitDiffStats, String> {
    let mut cmd = Command::new("git");
    if let Some(dir) = repo_dir {
        cmd.current_dir(dir);
    }
    cmd.args(["show", "--stat", "--format=", sha]);

    let output = cmd
        .output()
        .map_err(|e| format!("Fehler beim Ausführen von 'git show --stat': {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git show für SHA '{sha}' fehlgeschlagen: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_git_stat_output(&stdout)
}

/// Parst die Summary-Zeile der `git show --stat` Ausgabe.
/// Erwartetes Format z.B.: ` 3 files changed, 12 insertions(+), 4 deletions(-)`
/// Oder bei leeren Commits: ` 0 files changed` oder leere Ausgabe.
pub fn parse_git_stat_output(stat_output: &str) -> Result<CommitDiffStats, String> {
    let mut changed_files = 0;
    let mut insertions = 0;
    let mut deletions = 0;

    for line in stat_output.lines().rev() {
        let trimmed = line.trim();
        if trimmed.contains("file changed") || trimmed.contains("files changed") {
            let parts: Vec<&str> = trimmed.split(',').collect();
            for part in parts {
                let p = part.trim();
                if p.contains("file changed") || p.contains("files changed") {
                    if let Some(num_str) = p.split_whitespace().next() {
                        changed_files = num_str.parse::<usize>().map_err(|e| {
                            format!("Fehler beim Parsen der geänderten Dateien ('{num_str}'): {e}")
                        })?;
                    }
                } else if p.contains("insertion") {
                    if let Some(num_str) = p.split_whitespace().next() {
                        insertions = num_str.parse::<usize>().map_err(|e| {
                            format!("Fehler beim Parsen der Einfügungen ('{num_str}'): {e}")
                        })?;
                    }
                } else if p.contains("deletion") {
                    if let Some(num_str) = p.split_whitespace().next() {
                        deletions = num_str.parse::<usize>().map_err(|e| {
                            format!("Fehler beim Parsen der Löschungen ('{num_str}'): {e}")
                        })?;
                    }
                }
            }
            break;
        }
    }

    Ok(CommitDiffStats {
        changed_files,
        insertions,
        deletions,
    })
}

/// Liest die vollständige Commit-Message für den gegebenen SHA via `git log -1 --format=%B <sha>`.
pub fn get_commit_message(sha: &str, repo_dir: Option<&Path>) -> Result<String, String> {
    let mut cmd = Command::new("git");
    if let Some(dir) = repo_dir {
        cmd.current_dir(dir);
    }
    cmd.args(["log", "-1", "--format=%B", sha]);

    let output = cmd
        .output()
        .map_err(|e| format!("Fehler beim Ausführen von 'git log': {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git log für SHA '{sha}' fehlgeschlagen: {stderr}"));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Strippt ein evtl. vorhandenes Conventional Commit Präfix (z. B. `feat(scope): ` oder `fix: `).
fn strip_conventional_commit_prefix(line: &str) -> &str {
    let trimmed = line.trim();
    if let Some(colon_pos) = trimmed.find(':') {
        let prefix = &trimmed[..colon_pos];
        let cc_types = [
            "feat", "fix", "docs", "refactor", "test", "chore", "bench", "perf", "style", "build",
            "ci",
        ];

        let base_type = if let Some(open_paren) = prefix.find('(') {
            &prefix[..open_paren]
        } else {
            prefix
        };

        if cc_types.contains(&base_type.trim().to_lowercase().as_str()) {
            return trimmed[colon_pos + 1..].trim();
        }
    }
    trimmed
}

/// Zählt heuristisch, wie viele klar abgegrenzte Änderungsbehauptungen in der Commit-Message enthalten sind.
///
/// # Heuristik-Details
/// Eine Zeile wird als eigene stamped-Änderungsbehauptung gewertet, wenn:
/// 1. Sie ein Stichpunkt ist: beginnt nach Trim mit `- `, `* `, `+ `, `• `, oder `1. `, `2. ` etc.
/// 2. Oder sie beginnt nach Abschneiden eines evtl. Conventional-Commit-Präfix mit einem Verb/Aktionswort (case-insensitive):
///    "implementiert", "fügt hinzu", "behebt", "entfernt", "ändert", "refaktoriert",
///    "implements", "adds", "fixes", "removes", "changes", "refactors".
pub fn count_claim_points(commit_msg: &str) -> usize {
    let verbs = [
        "implementiert",
        "fügt hinzu",
        "behebt",
        "entfernt",
        "ändert",
        "refaktoriert",
        "implements",
        "adds",
        "fixes",
        "removes",
        "changes",
        "refactors",
    ];

    let mut points = 0;

    for raw_line in commit_msg.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        // 1. Check for bullet list format: "-", "*", "+", "•"
        if line.starts_with("- ")
            || line.starts_with("* ")
            || line.starts_with("+ ")
            || line.starts_with("• ")
        {
            points += 1;
            continue;
        }

        // 2. Check for numbered list format: e.g. "1. ", "2) "
        if let Some(first_word) = line.split_whitespace().next() {
            let mut chars = first_word.chars();
            if let Some(first_char) = chars.next() {
                if first_char.is_ascii_digit() {
                    let rest: String = chars.collect();
                    if rest == "." || rest == ")" || rest.ends_with('.') || rest.ends_with(')') {
                        points += 1;
                        continue;
                    }
                }
            }
        }

        // 3. Strip conventional commit prefix if present before checking for action verbs
        let stripped = strip_conventional_commit_prefix(line);
        let lower = stripped.to_lowercase();

        for verb in &verbs {
            if lower.starts_with(verb) {
                // Ensure word boundary after verb (space or end of string)
                let rest = &lower[verb.len()..];
                if rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t') {
                    points += 1;
                    break;
                }
            }
        }
    }

    points
}

/// Prüft die Integrität eines einzelnen Commits.
pub fn check_single_commit(
    sha: &str,
    repo_dir: Option<&Path>,
) -> Result<(), String> {
    let msg = get_commit_message(sha, repo_dir)?;
    let stats = get_commit_diff_stats(sha, repo_dir)?;
    let claims = count_claim_points(&msg);

    println!(
        "Commit {}: {} behauptete Änderungspunkte, {} geänderte Datei(en), {} Zeilen geändert",
        sha,
        claims,
        stats.changed_files,
        stats.total_lines_changed()
    );

    // Harter Fehlerfall: ≥ 3 Behauptungen aber 0 geänderte Dateien
    if claims >= 3 && stats.changed_files == 0 {
        return Err(format!(
            "PHANTOM COMMIT DETECTED! Commit SHA {} behauptet {} konkrete Änderungspunkte in der Message, hat aber 0 geänderte Dateien im Git-Diff!\nCommit Message:\n{}",
            sha, claims, msg
        ));
    }

    // Warnung: ≥ 3 Behauptungen bei genau 1 geänderten Datei (konservativ)
    if claims >= 3 && stats.changed_files == 1 {
        println!(
            "⚠️ WARNUNG [Commit {}]: Commit behauptet {} Änderungspunkte, betrifft aber nur 1 Datei. Bitte prüfen, ob alle Punkte enthalten sind.",
            sha, claims
        );
    }

    Ok(())
}

/// Löst einen Commit-Range (z. B. `base..head`) in eine Liste von Commit-SHAs auf via `git rev-list <range>`.
pub fn resolve_commit_range(range: &str, repo_dir: Option<&Path>) -> Result<Vec<String>, String> {
    let mut cmd = Command::new("git");
    if let Some(dir) = repo_dir {
        cmd.current_dir(dir);
    }
    cmd.args(["rev-list", range]);

    let output = cmd
        .output()
        .map_err(|e| format!("Fehler beim Ausführen von 'git rev-list {range}': {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "git rev-list für Range '{range}' fehlgeschlagen: {stderr}"
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let shas: Vec<String> = stdout
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(shas)
}

/// Hauptfunktion für das Subkommando `check-commit-diff-integrity`.
pub fn run_check_commit_diff_integrity(args: &[String]) -> Result<(), String> {
    println!("=== Running xtask check-commit-diff-integrity ===");

    let mut range_arg = None;
    let mut i = 0;
    while i < args.len() {
        if let Some(val) = args[i].strip_prefix("--range=") {
            range_arg = Some(val.to_string());
        } else if args[i] == "--range" && i + 1 < args.len() {
            range_arg = Some(args[i + 1].clone());
            i += 1;
        }
        i += 1;
    }

    let commits_to_check = if let Some(ref range) = range_arg {
        println!("Prüfe Commit-Range: {}", range);
        let shas = resolve_commit_range(range, None)?;
        if shas.is_empty() {
            println!("ℹ️ Keine Commits in Range '{}' gefunden.", range);
            return Ok(());
        }
        shas
    } else {
        println!("Prüfe Einzel-Commit: HEAD");
        vec!["HEAD".to_string()]
    };

    let mut violations = Vec::new();

    for sha in &commits_to_check {
        if let Err(err) = check_single_commit(sha, None) {
            violations.push(err);
        }
    }

    if violations.is_empty() {
        println!("✅ check-commit-diff-integrity: Alle Commits bestanden!");
        Ok(())
    } else {
        eprintln!(
            "❌ check-commit-diff-integrity FAILED: {} Phantom-Commit(s) gefunden!",
            violations.len()
        );
        for v in &violations {
            eprintln!("{}", v);
        }
        Err(format!(
            "Commit-Diff Integrity check failed: {} violation(s)",
            violations.len()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_git_stat_output_normal() {
        let stat = " src/lib.rs | 10 +++++-----\n src/main.rs | 2 ++\n 2 files changed, 7 insertions(+), 5 deletions(-)\n";
        let res = parse_git_stat_output(stat).unwrap();
        assert_eq!(res.changed_files, 2);
        assert_eq!(res.insertions, 7);
        assert_eq!(res.deletions, 5);
        assert_eq!(res.total_lines_changed(), 12);
    }

    #[test]
    fn test_parse_git_stat_output_empty() {
        let stat = "";
        let res = parse_git_stat_output(stat).unwrap();
        assert_eq!(res.changed_files, 0);
        assert_eq!(res.insertions, 0);
        assert_eq!(res.deletions, 0);
        assert_eq!(res.total_lines_changed(), 0);
    }

    #[test]
    fn test_count_claim_points_bullet_list() {
        let msg = r#"feat(core): update pipeline

- Implementiert Feature X
- Behebt Fehler Y in Crate Z
- Fügt Unit-Tests hinzu
- Dokumentiert API in README
- Refaktoriert Modul A"#;

        assert_eq!(count_claim_points(msg), 5);
    }

    #[test]
    fn test_count_claim_points_numbered_list() {
        let msg = r#"feat: major rewrite

1. Implementiert Trait A
2. Behebt TOCTOU Bug
3. Refaktoriert Modul B
"#;

        assert_eq!(count_claim_points(msg), 3);
    }

    #[test]
    fn test_count_claim_points_simple_message() {
        let msg = "fix: simple fix";
        assert_eq!(count_claim_points(msg), 0);
    }
}
