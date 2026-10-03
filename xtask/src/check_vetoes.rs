// Contextra — Feature-Veto-Register & CI Gate
//
// Modul zur Überprüfung von `VETOES.md` im CI-Workflow.
//
// Prüflogik:
// 1. Permanent Rejected: Durchsucht die geänderten Dateien / Commits nach Schlüsselwörtern abgelehnter Features.
// 2. Conditionally Accepted: Überwacht `conditional_review_due` Fristen.
//    - Frist in der Vergangenheit (< 0 Tage) UND PR berührt das Feature:
//      Harter CI-Fehler (Exit-Code != 0) mit Referenz auf `adr_ref`.
//    - Frist in der Vergangenheit (< 0 Tage) UND PR berührt das Feature NICHT:
//      Warnung und Hinweis "Review-Ticket anlegen" (Exit-Code 0).
//    - Datumsverlängerung in VETOES.md erfordert ein bereits auf dem Basis-Branch existierendes ADR.
// 3. Fail-closed: Falls `VETOES.md` fehlt, schlägt das Gate fehl.

use chrono::NaiveDate;
use std::fs;
use std::path::Path;
use std::process::Command;

pub const CONDITIONAL_REVIEW_WARNING_THRESHOLD_DAYS: i64 = 14;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VetoEntry {
    pub feature_id: String,
    pub status: String,
    pub keywords: Vec<String>,
    pub affected_paths: Vec<String>,
    pub reason: Option<String>,
    pub adr_ref: Option<String>,
    pub conditional_review_due: Option<String>,
}

impl VetoEntry {
    pub fn reason_summary(&self) -> &str {
        match &self.reason {
            Some(r) => {
                let trimmed = r.trim();
                if trimmed.is_empty() {
                    &self.feature_id
                } else {
                    trimmed.lines().next().unwrap_or(&self.feature_id).trim()
                }
            }
            None => &self.feature_id,
        }
    }

    pub fn is_file_affected(&self, file_path: &str) -> bool {
        let clean_path = file_path.trim_start_matches("./");
        for path in &self.affected_paths {
            let clean_affected = path.trim_start_matches("./").trim_start_matches('/');
            if clean_path.starts_with(clean_affected)
                || clean_path.contains(clean_affected)
                || clean_affected.contains(clean_path)
            {
                return true;
            }
        }
        for kw in &self.keywords {
            let kw_lower = kw.to_lowercase();
            if clean_path.to_lowercase().contains(&kw_lower) {
                return true;
            }
        }
        false
    }
}

pub fn parse_vetoes(content: &str) -> Result<Vec<VetoEntry>, String> {
    let mut entries = Vec::new();
    let blocks = content.split("## VETO-");

    for block in blocks.skip(1) {
        let mut feature_id = String::new();
        let mut status = String::new();
        let mut keywords = Vec::new();
        let mut affected_paths = Vec::new();
        let mut adr_ref = None;
        let mut conditional_review_due = None;
        let mut reason_lines = Vec::new();
        let mut parsing_reason = false;

        for line in block.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("feature_id:") {
                parsing_reason = false;
                feature_id = trimmed.trim_start_matches("feature_id:").trim().to_string();
            } else if trimmed.starts_with("status:") {
                parsing_reason = false;
                status = trimmed.trim_start_matches("status:").trim().to_string();
            } else if trimmed.starts_with("review_date:")
                || trimmed.starts_with("conditional_review_due:")
            {
                parsing_reason = false;
                let due = if trimmed.starts_with("review_date:") {
                    trimmed
                        .trim_start_matches("review_date:")
                        .trim()
                        .to_string()
                } else {
                    trimmed
                        .trim_start_matches("conditional_review_due:")
                        .trim()
                        .to_string()
                };
                if !due.is_empty() {
                    conditional_review_due = Some(due);
                }
            } else if trimmed.starts_with("adr_ref:") {
                parsing_reason = false;
                let ar = trimmed.trim_start_matches("adr_ref:").trim().to_string();
                if !ar.is_empty() && ar != "null" {
                    adr_ref = Some(ar);
                }
            } else if trimmed.starts_with("keywords:") {
                parsing_reason = false;
                let kw_str = trimmed.trim_start_matches("keywords:").trim();
                if kw_str.starts_with('[') && kw_str.ends_with(']') {
                    let inner = &kw_str[1..kw_str.len() - 1];
                    for item in inner.split(',') {
                        let cleaned = item.trim().trim_matches('"').trim_matches('\'').to_string();
                        if !cleaned.is_empty() {
                            keywords.push(cleaned);
                        }
                    }
                }
            } else if trimmed.starts_with("affected_paths:") {
                parsing_reason = false;
                let ap_str = trimmed.trim_start_matches("affected_paths:").trim();
                if ap_str.starts_with('[') && ap_str.ends_with(']') {
                    let inner = &ap_str[1..ap_str.len() - 1];
                    for item in inner.split(',') {
                        let cleaned = item.trim().trim_matches('"').trim_matches('\'').to_string();
                        if !cleaned.is_empty() {
                            affected_paths.push(cleaned);
                        }
                    }
                }
            } else if trimmed.starts_with("reason:") {
                parsing_reason = true;
                let rest = trimmed.trim_start_matches("reason:").trim();
                if !rest.is_empty() && rest != ">" {
                    reason_lines.push(rest.to_string());
                }
            } else if trimmed.starts_with("scope_note:") || trimmed.starts_with("last_verified:") {
                parsing_reason = false;
            } else if parsing_reason && !trimmed.is_empty() {
                reason_lines.push(trimmed.to_string());
            }
        }

        let reason = if reason_lines.is_empty() {
            None
        } else {
            Some(reason_lines.join(" "))
        };

        if !feature_id.is_empty() && !status.is_empty() {
            entries.push(VetoEntry {
                feature_id,
                status,
                keywords,
                affected_paths,
                reason,
                adr_ref,
                conditional_review_due,
            });
        }
    }

    Ok(entries)
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct DeadlineCheckResult {
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

pub fn check_conditional_review_deadlines_scoped(
    entries: &[VetoEntry],
    today_str: &str,
    changed_files: &[String],
) -> DeadlineCheckResult {
    let mut result = DeadlineCheckResult::default();
    let today = match NaiveDate::parse_from_str(today_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return result,
    };

    for entry in entries {
        if entry.status == "conditionally_accepted" {
            if let Some(due_str) = &entry.conditional_review_due {
                if let Ok(due_date) = NaiveDate::parse_from_str(due_str, "%Y-%m-%d") {
                    let days_until_due = (due_date - today).num_days();
                    let adr = entry.adr_ref.as_deref().unwrap_or("keine ADR angegeben");

                    if days_until_due < 0 {
                        let touches_feature = changed_files
                            .iter()
                            .any(|file| entry.is_file_affected(file));

                        if touches_feature
                            || (entry.affected_paths.is_empty() && changed_files.is_empty())
                        {
                            result.errors.push(format!(
                                "❌ VETO-FRIST ÜBERSCHRITTEN: {} — Wiedervorlage war am {}, geänderte Dateien berühren das Feature. PR blockiert (adr_ref: {}). Bitte ADR mit Entscheidung erstellen oder Frist per neuem ADR auf Basis-Branch verlängern.",
                                entry.feature_id,
                                due_str,
                                adr
                            ));
                        } else {
                            result.warnings.push(format!(
                                "⚠️ WARNUNG: VETO {} Review-Frist {} ist abgelaufen (adr_ref: {}), aber der PR berührt dieses Feature nicht. Hinweis: Bitte Review-Ticket anlegen.",
                                entry.feature_id,
                                due_str,
                                adr
                            ));
                        }
                    } else if days_until_due <= CONDITIONAL_REVIEW_WARNING_THRESHOLD_DAYS {
                        result.warnings.push(format!(
                            "⚠️ WARNUNG: VETO {} ('{}'): Review-Frist {} laeuft in {} Tag(en) ab — rechtzeitige Review erforderlich (adr_ref: {}).",
                            entry.feature_id,
                            entry.reason_summary(),
                            due_str,
                            days_until_due,
                            adr
                        ));
                    }
                }
            }
        }
    }

    result
}

pub fn check_veto_date_extensions(
    current_entries: &[VetoEntry],
    base_vetoes_content: Option<&str>,
    check_adr_exists_on_base: &dyn Fn(&str) -> bool,
) -> Vec<String> {
    let mut errors = Vec::new();

    let base_entries = match base_vetoes_content {
        Some(content) => match parse_vetoes(content) {
            Ok(entries) => entries,
            Err(_) => return errors,
        },
        None => return errors,
    };

    for current in current_entries {
        if let Some(base) = base_entries
            .iter()
            .find(|e| e.feature_id == current.feature_id)
        {
            if current.conditional_review_due != base.conditional_review_due {
                // Date was changed/extended
                if let Some(adr) = &current.adr_ref {
                    if !check_adr_exists_on_base(adr) {
                        errors.push(format!(
                            "❌ UNZULÄSSIGE VETO-VERLÄNGERUNG: VETO {} Datum wurde auf {:?} geändert, aber das referenzierte ADR '{}' existiert NICHT auf dem Basis-Branch. Verlängerungen sind nur über vorab auf dem Basis-Branch gemergete ADRs zulässig.",
                            current.feature_id,
                            current.conditional_review_due,
                            adr
                        ));
                    }
                } else {
                    errors.push(format!(
                        "❌ UNZULÄSSIGE VETO-VERLÄNGERUNG: VETO {} Datum geändert ohne adr_ref.",
                        current.feature_id
                    ));
                }
            }
        }
    }

    errors
}

pub fn check_vetoes_with_root_and_opts(
    root: &Path,
    today_str: &str,
    changed_files_override: Option<&[String]>,
    base_vetoes_override: Option<&str>,
    adr_exists_on_base_override: Option<&dyn Fn(&str) -> bool>,
) -> Result<(), String> {
    println!("=== Running xtask check-vetoes ===");
    let vetoes_path = root.join("VETOES.md");
    if !vetoes_path.exists() {
        return Err(format!(
            "❌ FAIL-CLOSED: VETOES.md ({}) existiert nicht. Handlungsanweisung: Erstelle VETOES.md im Repository-Root.",
            vetoes_path.display()
        ));
    }

    let vetoes_content = fs::read_to_string(&vetoes_path)
        .map_err(|e| format!("VETOES.md ({}) nicht lesbar: {e}", vetoes_path.display()))?;

    let entries = parse_vetoes(&vetoes_content)?;

    // Get list of changed files
    let git_changed_files;
    let changed_files = if let Some(override_files) = changed_files_override {
        override_files
    } else {
        let diff_output = Command::new("git")
            .args(["diff", "--name-only", "HEAD~1...HEAD"])
            .output();

        git_changed_files = match diff_output {
            Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            _ => Vec::new(),
        };
        &git_changed_files
    };

    let log_output = Command::new("git")
        .args(["log", "--oneline", "-50"])
        .output()
        .map_err(|e| format!("git log fehlgeschlagen: {e}"))?;

    let commit_log = String::from_utf8_lossy(&log_output.stdout).to_lowercase();

    let mut warnings = Vec::new();
    for entry in entries.iter().filter(|e| e.status == "permanent_rejected") {
        for keyword in &entry.keywords {
            if commit_log.contains(&keyword.to_lowercase()) {
                warnings.push(format!(
                    "⚠️ Möglicher VETO-Treffer: {} (Keyword: '{}'). Siehe VETOES.md. \
                     Falls legitim: PR-Beschreibung mit 'VETO-CHECK-OK: <Grund>' versehen.",
                    entry.feature_id, keyword
                ));
            }
        }
    }

    let deadline_res =
        check_conditional_review_deadlines_scoped(&entries, today_str, changed_files);
    warnings.extend(deadline_res.warnings);

    let mut errors = deadline_res.errors;

    // Check date extension validity against base branch
    if let (Some(base_content), Some(check_adr_fn)) =
        (base_vetoes_override, adr_exists_on_base_override)
    {
        let ext_errors = check_veto_date_extensions(&entries, Some(base_content), check_adr_fn);
        errors.extend(ext_errors);
    } else if changed_files_override.is_none() {
        // Attempt git show origin/main:VETOES.md or main:VETOES.md
        let base_vetoes_out = Command::new("git")
            .args(["show", "HEAD~1:VETOES.md"])
            .output();

        if let Ok(out) = base_vetoes_out {
            if out.status.success() {
                let base_content = String::from_utf8_lossy(&out.stdout);
                let check_adr_fn = |adr_ref: &str| -> bool {
                    let git_check = Command::new("git")
                        .args(["cat-file", "-e", &format!("HEAD~1:{adr_ref}")])
                        .output();
                    match git_check {
                        Ok(o) => o.status.success(),
                        Err(_) => false,
                    }
                };
                let ext_errors =
                    check_veto_date_extensions(&entries, Some(&base_content), &check_adr_fn);
                errors.extend(ext_errors);
            }
        }
    }

    if !warnings.is_empty() {
        for w in &warnings {
            eprintln!("{w}");
        }
    } else if errors.is_empty() {
        println!("✅ Keine Veto-Keyword-Treffer in den letzten 50 Commits.");
    }

    if !errors.is_empty() {
        for err in &errors {
            eprintln!("{err}");
        }
        return Err(format!(
            "Veto conditional review check failed with {} error(s):\n{}",
            errors.len(),
            errors.join("\n")
        ));
    }

    Ok(())
}

pub fn check_vetoes() -> Result<(), String> {
    let root = crate::find_root_dir();
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    check_vetoes_with_root_and_opts(&root, &today, None, None, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_parse_vetoes() {
        let content = r#"
# Contextra — Feature-Veto-Register

## VETO-F02

feature_id: F-02
status: conditionally_accepted
conditional_review_due: 2026-10-07
affected_paths: ["crates/contextra-vector/"]
keywords: ["partial hnsw rebuild", "nucleation", "rebuild_region", "F-02"]
reason: >
  Reason text

## VETO-F10

feature_id: F-10
status: permanent_rejected
keywords: ["cross-tenant", "osmotic knowledge exchange", "tenant knowledge sharing", "F-10"]
reason: >
  Reason text
"#;
        let entries = parse_vetoes(content).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].feature_id, "F-02");
        assert_eq!(entries[0].status, "conditionally_accepted");
        assert_eq!(
            entries[0].conditional_review_due.as_deref(),
            Some("2026-10-07")
        );
        assert_eq!(entries[0].affected_paths, vec!["crates/contextra-vector/"]);
        assert_eq!(
            entries[0].keywords,
            vec![
                "partial hnsw rebuild",
                "nucleation",
                "rebuild_region",
                "F-02"
            ]
        );

        assert_eq!(entries[1].feature_id, "F-10");
        assert_eq!(entries[1].status, "permanent_rejected");
        assert_eq!(entries[1].conditional_review_due, None);
    }

    #[test]
    fn test_conditional_review_deadline_expired_hard_error_when_feature_touched() {
        let entries = vec![VetoEntry {
            feature_id: "F-02".to_string(),
            status: "conditionally_accepted".to_string(),
            keywords: vec!["hnsw".to_string()],
            affected_paths: vec!["crates/contextra-vector/".to_string()],
            reason: Some("Test reason".to_string()),
            adr_ref: Some("docs/decisions/ADR-077-test.md".to_string()),
            conditional_review_due: Some("2026-10-07".to_string()),
        }];

        let changed_files = vec!["crates/contextra-vector/src/hnsw.rs".to_string()];
        let res = check_conditional_review_deadlines_scoped(&entries, "2026-10-08", &changed_files);
        assert!(res.warnings.is_empty());
        assert_eq!(res.errors.len(), 1);
        assert!(res.errors[0].contains("VETO-FRIST ÜBERSCHRITTEN"));
        assert!(res.errors[0].contains("F-02"));
    }

    #[test]
    fn test_conditional_review_deadline_expired_warning_when_feature_not_touched() {
        let entries = vec![VetoEntry {
            feature_id: "F-02".to_string(),
            status: "conditionally_accepted".to_string(),
            keywords: vec!["hnsw".to_string()],
            affected_paths: vec!["crates/contextra-vector/".to_string()],
            reason: Some("Test reason".to_string()),
            adr_ref: Some("docs/decisions/ADR-077-test.md".to_string()),
            conditional_review_due: Some("2026-10-07".to_string()),
        }];

        let changed_files = vec!["crates/contextra-store/src/lib.rs".to_string()];
        let res = check_conditional_review_deadlines_scoped(&entries, "2026-10-08", &changed_files);
        assert_eq!(res.errors.len(), 0);
        assert_eq!(res.warnings.len(), 1);
        assert!(res.warnings[0].contains("F-02"));
        assert!(res.warnings[0].contains("Review-Ticket anlegen"));
    }

    #[test]
    fn test_check_veto_date_extension_disallowed_when_adr_not_on_base() {
        let current_entries = vec![VetoEntry {
            feature_id: "F-02".to_string(),
            status: "conditionally_accepted".to_string(),
            keywords: vec![],
            affected_paths: vec![],
            reason: None,
            adr_ref: Some("docs/decisions/ADR-099-new.md".to_string()),
            conditional_review_due: Some("2026-12-01".to_string()),
        }];

        let base_content = r#"
## VETO-F02
feature_id: F-02
status: conditionally_accepted
conditional_review_due: 2026-10-07
adr_ref: docs/decisions/ADR-077-old.md
"#;

        let check_adr_fn = |_path: &str| -> bool { false }; // ADR does NOT exist on base branch

        let errors =
            check_veto_date_extensions(&current_entries, Some(base_content), &check_adr_fn);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("UNZULÄSSIGE VETO-VERLÄNGERUNG"));
        assert!(errors[0].contains("F-02"));
    }

    #[test]
    fn test_check_vetoes_fail_closed_missing_file() {
        let temp = tempdir().unwrap();
        let err = check_vetoes_with_root_and_opts(temp.path(), "2026-10-01", Some(&[]), None, None)
            .unwrap_err();
        assert!(err.contains("FAIL-CLOSED"));
        assert!(err.contains("VETOES.md"));
    }
}
