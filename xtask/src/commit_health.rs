use chrono::Utc;
use std::fs;
use std::path::Path;

pub fn run_commit_health_impl(
    root: &Path,
    since_days: u32,
    output: Option<&Path>,
) -> Result<String, String> {
    let shell_report =
        crate::shell_commit_audit::run_shell_commit_audit_impl(root, Some(since_days), None)?;
    let audit_gaps = crate::audit_integrity_check::run_audit_integrity_check_impl(
        root,
        Some(since_days),
        false,
    )?;
    let hotspots = crate::hotspot_report::run_hotspot_report_impl(root, since_days, 10, false)?;

    let today = Utc::now().format("%Y-%m-%d").to_string();

    let total_commits = shell_report.total_commits;
    let shell_count = shell_report.shell_commits.len();
    let shell_percentage = if total_commits > 0 {
        (shell_count as f64 / total_commits as f64) * 100.0
    } else {
        0.0
    };

    let audit_gaps_under_6h = audit_gaps
        .iter()
        .filter(|g| {
            matches!(
                g.severity,
                crate::audit_integrity_check::GapSeverity::Contradicted(_)
                    | crate::audit_integrity_check::GapSeverity::Suspicious(_)
            )
        })
        .count();

    let top_hotspot = hotspots
        .first()
        .map(|h| format!("{} ({} Änd.)", h.file, h.changes_in_window))
        .unwrap_or_else(|| "Keine".to_string());

    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# Commit-Gesundheitsbericht — Stand {}\n\n",
        today
    ));

    markdown.push_str("## Zusammenfassung\n\n");
    markdown.push_str("| Metrik | Wert |\n");
    markdown.push_str("|---|---|\n");
    markdown.push_str(&format!("| Commits gesamt | {} |\n", total_commits));
    markdown.push_str(&format!(
        "| Shell-Commits | {} ({:.1}%) |\n",
        shell_count, shell_percentage
    ));
    markdown.push_str(&format!(
        "| Audit→Fix-Gaps <6h | {} |\n",
        audit_gaps_under_6h
    ));
    markdown.push_str(&format!("| Top-Hotspot-Datei | {} |\n\n", top_hotspot));

    markdown.push_str("## Top 5 Shell-Commits mit substanziellem Diff (>50 Zeilen)\n\n");
    let mut substantial_shells: Vec<_> = shell_report
        .shell_commits
        .iter()
        .filter(|c| c.has_substantial_diff)
        .collect();
    substantial_shells
        .sort_by(|a, b| (b.insertions + b.deletions).cmp(&(a.insertions + a.deletions)));
    substantial_shells.truncate(5);

    if substantial_shells.is_empty() {
        markdown.push_str("Keine Shell-Commits mit substanziellem Diff gefunden.\n\n");
    } else {
        markdown.push_str("| Commit-Hash | Datum | Dateianzahl | Insertions | Deletions |\n");
        markdown.push_str("|---|---|---|---|---|\n");
        for c in substantial_shells {
            markdown.push_str(&format!(
                "| `{}` | {} | {} | {} | {} |\n",
                &c.hash[..c.hash.len().min(8)],
                c.date,
                c.files_changed,
                c.insertions,
                c.deletions
            ));
        }
        markdown.push_str("\n");
    }

    markdown.push_str("## Top 5 Auffällige Audit-Fix-Lücken\n\n");
    let mut suspicious_gaps: Vec<_> = audit_gaps
        .iter()
        .filter(|g| !matches!(g.severity, crate::audit_integrity_check::GapSeverity::Clean))
        .collect();
    suspicious_gaps.sort_by(|a, b| {
        a.gap_hours
            .partial_cmp(&b.gap_hours)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    suspicious_gaps.truncate(5);

    if suspicious_gaps.is_empty() {
        markdown.push_str("Keine auffälligen Audit-Fix-Lücken (<6h) gefunden.\n\n");
    } else {
        markdown.push_str(
            "| Audit-Commit | Audit-Datei | Verdict | Fix-Commit | Lücke (Std) | Einstufung |\n",
        );
        markdown.push_str("|---|---|---|---|---|---|\n");
        for g in suspicious_gaps {
            let gap_str = g
                .gap_hours
                .map(|gh| format!("{:.2}", gh))
                .unwrap_or_else(|| "N/A".to_string());
            let status_str = match g.severity {
                crate::audit_integrity_check::GapSeverity::Contradicted(_) => "Contradicted (<=2h)",
                crate::audit_integrity_check::GapSeverity::Suspicious(_) => "Suspicious (2-6h)",
                crate::audit_integrity_check::GapSeverity::Clean => "Clean",
            };
            markdown.push_str(&format!(
                "| `{}` | `{}` | {} | `{}` | {} | {} |\n",
                &g.audit_commit[..g.audit_commit.len().min(8)],
                g.audit_file,
                g.verdict,
                g.fix_commit
                    .as_deref()
                    .map(|c| &c[..c.len().min(8)])
                    .unwrap_or("N/A"),
                gap_str,
                status_str
            ));
        }
        markdown.push_str("\n");
    }

    markdown.push_str("## Top 5 Hotspots (meistgeänderte Dateien)\n\n");
    let top_5_hotspots = hotspots.iter().take(5);

    markdown.push_str("| Datei | Änderungen | Autoren (Sessions) | Risikostufe |\n");
    markdown.push_str("|---|---|---|---|\n");
    for h in top_5_hotspots {
        let risk_str = match h.risk_level {
            crate::hotspot_report::HotspotRisk::Normal => "Normal",
            crate::hotspot_report::HotspotRisk::Elevated => "Elevated (>20)",
            crate::hotspot_report::HotspotRisk::Critical => "Critical (>50)",
        };
        markdown.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            h.file, h.changes_in_window, h.distinct_authors, risk_str
        ));
    }
    markdown.push_str("\n");

    if let Some(out_path) = output {
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directories for {:?}: {}", out_path, e))?;
        }
        fs::write(out_path, &markdown).map_err(|e| {
            format!(
                "Failed to write commit health report to {:?}: {}",
                out_path, e
            )
        })?;
    }

    Ok(markdown)
}

pub fn run_commit_health(since_days: u32, output: Option<&Path>) -> Result<String, String> {
    let root = xtask::find_root_dir();
    run_commit_health_impl(&root, since_days, output)
}
