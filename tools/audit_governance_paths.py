#!/usr/bin/env python3
"""
tools/audit_governance_paths.py — Auditing script for governance path split
INV-DOC-SINGLE-SOURCE-1, SPEC §18.7
"""

import os
import subprocess
import glob
import re

def get_git_file_info(filepath):
    if not os.path.exists(filepath):
        return ("N/A", "N/A")
    size = os.path.getsize(filepath)
    cmd = ["git", "log", "-1", "--format=%h %cd", "--date=iso-short", "--", filepath]
    res = subprocess.run(cmd, capture_output=True, text=True)
    commit_info = res.stdout.strip() if res.returncode == 0 and res.stdout.strip() else "unknown"
    return (size, commit_info)

def generate_table_a():
    gov_dir = "governance"
    agent_gov_dir = ".agents/governance"

    files_gov = set(os.listdir(gov_dir)) if os.path.exists(gov_dir) else set()
    files_agent = set(os.listdir(agent_gov_dir)) if os.path.exists(agent_gov_dir) else set()

    all_files = sorted(list(files_gov | files_agent))

    lines = []
    lines.append("### Tabelle A: Dateien unter `governance/` und `.agents/governance/`\n")
    lines.append("| Dateiname | governance/ (Größe, Last Commit) | .agents/governance/ (Größe, Last Commit) | Diff-Befund |")
    lines.append("|---|---|---|---|")

    for f in all_files:
        p1 = os.path.join(gov_dir, f)
        p2 = os.path.join(agent_gov_dir, f)

        e1 = os.path.exists(p1)
        e2 = os.path.exists(p2)

        s1, c1 = get_git_file_info(p1) if e1 else ("-", "-")
        s2, c2 = get_git_file_info(p2) if e2 else ("-", "-")

        info1 = f"{s1} B (`{c1}`)" if e1 else "fehlt"
        info2 = f"{s2} B (`{c2}`)" if e2 else "fehlt"

        if e1 and e2:
            res = subprocess.run(["diff", "-q", p1, p2], capture_output=True, text=True)
            if res.returncode == 0:
                diff_status = "Identisch"
            else:
                diff_status = "**Abweichend**"
        elif e1 and not e2:
            diff_status = "Nur in `governance/`"
        else:
            diff_status = "Nur in `.agents/governance/`"

        lines.append(f"| `{f}` | {info1} | {info2} | {diff_status} |")
    return "\n".join(lines)

def search_paths_in_repo():
    targets = []
    for root_item in ["justfile", "AGENTS.md", "CLAUDE.md", "capabilities.toml"]:
        if os.path.exists(root_item):
            targets.append(root_item)

    for pattern in ["xtask/**", ".github/**", "docs/**"]:
        for filepath in glob.glob(pattern, recursive=True):
            if os.path.isfile(filepath):
                targets.append(filepath)

    targets = sorted(list(set(targets)))
    results = []

    path_regex = re.compile(r'(\.agents/governance(?:/[a-zA-Z0-9_\-\.]*)?|governance(?:/[a-zA-Z0-9_\-\.]*)?|\.jules(?:/[a-zA-Z0-9_\-\.]*)?)')

    for filepath in targets:
        try:
            with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
                for line_num, line in enumerate(f, 1):
                    matches = path_regex.findall(line)
                    for m in matches:
                        cleaned_path = m.rstrip(':,).`\'"')
                        if not cleaned_path:
                            continue
                        exists = os.path.exists(cleaned_path)
                        results.append((filepath, line_num, cleaned_path, "ja" if exists else "**nein**"))
        except Exception:
            pass

    lines = []
    lines.append("\n### Tabelle B: Pfadangaben in Konfigurations- und Dokumentationsdateien\n")
    lines.append("| Fundstelle (Datei:Zeile) | Referenzierter Pfad | Existiert? |")
    lines.append("|---|---|---|")
    for loc, line, path, exists in results:
        lines.append(f"| `{loc}:{line}` | `{path}` | {exists} |")
    return "\n".join(lines)

def get_git_history_diffs():
    differing = ["gates.toml", "ratchet.toml", "protected-paths.toml"]
    lines = []
    lines.append("\n### Analyse der abweichenden Dateien\n")

    for f in differing:
        p_gov = f"governance/{f}"
        p_agent = f".agents/governance/{f}"
        lines.append(f"#### `{f}`\n")
        lines.append("##### Git Log (governance/)\n```")
        res1 = subprocess.run(["git", "log", "-n", "3", "--format=%h %cd %s", "--", p_gov], capture_output=True, text=True)
        lines.append(res1.stdout.strip())
        lines.append("```\n")

        lines.append("##### Git Log (.agents/governance/)\n```")
        res2 = subprocess.run(["git", "log", "-n", "3", "--format=%h %cd %s", "--", p_agent], capture_output=True, text=True)
        lines.append(res2.stdout.strip())
        lines.append("```\n")

        lines.append("##### Inhaltlicher Diff (governance/ vs .agents/governance/)\n```diff")
        res_diff = subprocess.run(["diff", "-u", p_gov, p_agent], capture_output=True, text=True)
        lines.append(res_diff.stdout.strip())
        lines.append("```\n")

    return "\n".join(lines)

if __name__ == "__main__":
    print(generate_table_a())
    print(search_paths_in_repo())
    print(get_git_history_diffs())
