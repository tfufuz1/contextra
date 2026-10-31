#!/usr/bin/env python3
"""
audit_governance_paths.py — Audits governance files and path references across contextra repo.
Generates Table A (file inventory), Table B (path reference audit), git history, diffs, and section 4.
"""

import os
import re
import glob
import subprocess
import sys

def run_cmd(cmd):
    try:
        res = subprocess.run(cmd, capture_output=True, text=True)
        return res.stdout.strip()
    except Exception:
        return ""

def get_file_info(filepath):
    if not os.path.exists(filepath):
        return None, "fehlt", "fehlt"
    size = os.path.getsize(filepath)
    commit = run_cmd(["git", "log", "-1", "--format=%h (%ad)", "--date=iso", "--", filepath])
    if not commit:
        commit = "uncommitted"
    return size, f"{size} B", commit

def generate_table_a():
    gov_dir = "governance"
    agents_gov_dir = os.path.join(".agents", "governance")

    gov_files = set(os.listdir(gov_dir)) if os.path.exists(gov_dir) else set()
    agents_gov_files = set(os.listdir(agents_gov_dir)) if os.path.exists(agents_gov_dir) else set()

    gov_files = {f for f in gov_files if os.path.isfile(os.path.join(gov_dir, f))}
    agents_gov_files = {f for f in agents_gov_files if os.path.isfile(os.path.join(agents_gov_dir, f))}

    all_files = sorted(gov_files | agents_gov_files)

    table_rows = []
    for f in all_files:
        p1 = os.path.join(gov_dir, f)
        p2 = os.path.join(agents_gov_dir, f)

        s1, s1_str, c1 = get_file_info(p1)
        s2, s2_str, c2 = get_file_info(p2)

        if s1 is not None and s2 is not None:
            if s1 == s2:
                with open(p1, 'rb') as f1, open(p2, 'rb') as f2:
                    diff_finding = "Identisch" if f1.read() == f2.read() else "Abweichend"
            else:
                diff_finding = "Abweichend"
        elif s1 is not None:
            diff_finding = "Fehlt in .agents/governance/"
        else:
            diff_finding = "Fehlt in governance/"

        table_rows.append({
            "filename": f,
            "gov_size": s1_str,
            "gov_commit": c1,
            "agents_size": s2_str,
            "agents_commit": c2,
            "finding": diff_finding
        })

    return table_rows

def get_target_files():
    targets = []
    scopes = [".github", "docs"]
    output_doc_path = os.path.normpath("docs/spec/GOVERNANCE-PATHS.md")

    for s in scopes:
        if os.path.exists(s):
            for root, _, files in os.walk(s):
                for f in files:
                    full_p = os.path.normpath(os.path.join(root, f))
                    # Exclude the report output file itself to prevent self-referential duplication
                    if full_p == output_doc_path:
                        continue
                    targets.append(full_p)

    single_files = ["justfile", "AGENTS.md", "CLAUDE.md", "capabilities.toml"]
    for f in single_files:
        if os.path.exists(f):
            targets.append(os.path.normpath(f))

    return sorted(set(targets))

def check_path_exists(path_str):
    clean = path_str[2:] if path_str.startswith("./") else path_str

    if "*" in clean:
        matches = glob.glob(clean)
        if matches:
            return "ja"
        prefix = clean.split("*")[0].rstrip("/")
        if os.path.exists(prefix):
            return "ja"
        return "nein"

    if os.path.exists(clean):
        return "ja"
    return "nein"

def generate_table_b():
    targets = get_target_files()
    path_pattern = re.compile(
        r'((?:\.agents/governance/|\.agents/jules/|governance/|\.jules/|\.agents/)[a-zA-Z0-9_\-\./\*]+)'
    )

    results = []
    for filepath in targets:
        try:
            with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
                for line_no, line in enumerate(f, 1):
                    for match in path_pattern.findall(line):
                        clean_path = match.rstrip('\'":;,.)]}`>\\/ ')
                        if not clean_path:
                            continue
                        exists = check_path_exists(clean_path)
                        results.append({
                            "location": f"{filepath}:{line_no}",
                            "path": clean_path,
                            "exists": exists
                        })
        except Exception:
            pass

    return results

def get_three_files_git_logs_and_diffs():
    files = ["gates.toml", "ratchet.toml", "protected-paths.toml"]
    data = {}
    for f in files:
        p_gov = os.path.join("governance", f)
        p_agents = os.path.join(".agents", "governance", f)

        log_gov = run_cmd(["git", "log", "-5", "--follow", "--format=%h %ad %s", "--date=iso", "--", p_gov])
        log_agents = run_cmd(["git", "log", "-5", "--follow", "--format=%h %ad %s", "--date=iso", "--", p_agents])

        diff_content = run_cmd(["diff", "-u", p_gov, p_agents])

        data[f] = {
            "p_gov": p_gov,
            "p_agents": p_agents,
            "log_gov": log_gov,
            "log_agents": log_agents,
            "diff": diff_content
        }
    return data

def build_report():
    table_a = generate_table_a()
    table_b = generate_table_b()
    three_files = get_three_files_git_logs_and_diffs()

    output = []
    output.append("# AUDIT REPORT: GOVERNANCE UND HARNESS PFADSTRUKTUR")
    output.append("Normativ: SPEC §18.7 (INV-DOC-SINGLE-SOURCE-1) | Audit-Dateien: governance/ vs .agents/governance/\n")

    output.append("## 1. Tabelle A: Governance-Dateien Inventar")
    output.append("| Datei | governance/ (Größe, Commit) | .agents/governance/ (Größe, Commit) | Diff-Befund |")
    output.append("| --- | --- | --- | --- |")
    for row in table_a:
        gov_str = f"{row['gov_size']}, `{row['gov_commit']}`" if row['gov_size'] != "fehlt" else "fehlt"
        agents_str = f"{row['agents_size']}, `{row['agents_commit']}`" if row['agents_size'] != "fehlt" else "fehlt"
        output.append(f"| `{row['filename']}` | {gov_str} | {agents_str} | {row['finding']} |")

    output.append("\n## 2. Tabelle B: Audit aller Pfadverweise auf governance/, .agents/governance/, .jules/")
    missing_count = sum(1 for r in table_b if r['exists'] == "nein")
    output.append(f"Gesamtanzahl angemessener Fundstellen: {len(table_b)}")
    output.append(f"Davon nicht existierend auf HEAD (existiert: nein): {missing_count}\n")
    output.append("| Fundstelle | Pfadangabe | Existiert |")
    output.append("| --- | --- | --- |")
    for row in table_b:
        output.append(f"| `{row['location']}` | `{row['path']}` | existiert: {row['exists']} |")

    output.append("\n## 3. Detail-Analyse der drei abweichenden Dateien")
    for f, d in three_files.items():
        output.append(f"\n### 3.1 `{f}`")
        output.append(f"**Git-Historie `{d['p_gov']}`:**")
        output.append("```text")
        output.append(d['log_gov'] or "(keine Historie)")
        output.append("```")
        output.append(f"**Git-Historie `{d['p_agents']}`:**")
        output.append("```text")
        output.append(d['log_agents'] or "(keine Historie)")
        output.append("```")
        output.append(f"**Inhaltlicher Diff (`diff -u {d['p_gov']} {d['p_agents']}`):**")
        output.append("```diff")
        output.append(d['diff'] or "(keine Inhaltsunterschiede)")
        output.append("```")

    output.append("\n## 4. Gemessene Gate-Auswirkungen und ins Leere laufende Checks")
    output.append("Alle nicht explizit mit Messwerten aufgeführten Benchmarks, Miri-Läufe, Loom-Prover und Heavy-Chaos-Tests wurden in PR 1 **nicht geprüft** (Nicht-Ziel PR 1).\n")

    return "\n".join(output)

def main():
    report = build_report()
    target_doc = "docs/spec/GOVERNANCE-PATHS.md"
    os.makedirs(os.path.dirname(target_doc), exist_ok=True)
    with open(target_doc, "w", encoding="utf-8") as f:
        f.write(report)
    print(f"Generated {target_doc} successfully.")

if __name__ == "__main__":
    main()
