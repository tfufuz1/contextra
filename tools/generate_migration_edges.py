#!/usr/bin/env python3
"""
generate_migration_edges.py - Generates docs/spec/MIGRATION-EDGES.md inventory v2.

Rule definition:
- Scans all workspace crates for dependencies on the 9 Archive Crates:
  contextra-adapt, contextra-agent, contextra-cognition, contextra-core,
  contextra-graph, contextra-infer-ollama, contextra-infer-onnx,
  contextra-kvcache, contextra-router.
- Identifies manifest edges (normal, dev, build; optional flag) and source code references
  (number of files in src/ and tests/, top 10 imported symbols).
- Generates summary table by Dependent and per-archive-crate breakdown table.
- Python 3 standard library only.
"""

import os
import re
import subprocess
import sys
import tomllib
from collections import Counter

ARCHIVE_CRATES = {
    'contextra-adapt': 'contextra_adapt',
    'contextra-agent': 'contextra_agent',
    'contextra-cognition': 'contextra_cognition',
    'contextra-core': 'contextra_core',
    'contextra-graph': 'contextra_graph',
    'contextra-infer-ollama': 'contextra_infer_ollama',
    'contextra-infer-onnx': 'contextra_infer_onnx',
    'contextra-kvcache': 'contextra_kvcache',
    'contextra-router': 'contextra_router',
}

MASSNAHMEN = {
    'contextra-adapt': '`AdaptiveController` durch feste Startwerte ersetzen: `efSearch = 64`, `exact_scan_max = 20_000`, Overfetch `3 / 6 / 12` (PLAN.md §7, WP 2.2 Schritt 3).',
    'contextra-agent': 'Feature `agent-workflows` entfernen; optionale Kante aus `mcp/Cargo.toml` löschen.',
    'contextra-cognition': 'Kante aus `db/Cargo.toml` löschen; betroffene Module in `db/src/` entfernen oder stubben.',
    'contextra-core': 'Mechanische Ersetzung `use contextra_core::` → `use contextra_types::` (K-11: ~286 Dateien). Nach `cargo check` Crate und Eintrag in `Cargo.toml` löschen (D-05).',
    'contextra-graph': '`engine/src/collection/crud/links.rs` löschen; graph-bezogene Funktionen (`stage_graph_entity_delete`, `check_and_trigger_community_detection`, `CommunityDetectionConfig`, PPR-Pfade im Hybrid-Planner) entfernen; Modalität als `modalities_unavailable` markieren.',
    'contextra-infer-ollama': 'Feature `ollama` bleibt in Fassade, aber Kante nur noch über Feature-Gate.',
    'contextra-infer-onnx': 'Feature `onnx` bleibt in Fassade, aber Kante nur noch über Feature-Gate.',
    'contextra-kvcache': '`engine/src/kv_cache_integration/` löschen; `Collection::with_kv_store`, `set_kv_store`, `kv_store`, `start_kv_cache_eviction`, `TenantIsolatedKvStore`-Reste entfernen; Layer `KvCache` → `FeatureDisabled`.',
    'contextra-router': 'Feature `router` in Fassade entfernen; `contextra-py` anpassen.',
}

def get_target_wp(dep_name, archive_crate):
    if dep_name == 'contextra-engine':
        return 'WP 2.2'
    elif dep_name in ('contextra-bench', 'benchmarks/contextra-bench'):
        return 'WP 2.1b'
    elif dep_name in ARCHIVE_CRATES and archive_crate in ARCHIVE_CRATES:
        return 'WP 2.1b'
    elif dep_name in ('contextra-py', 'contextra-infer-candle', 'contextra-privacy', 'contextra-sandbox', 'contextra-license', 'contextra-audit-export', 'contextra-avv-generator'):
        return 'WP 2.3' if dep_name in ('contextra-py', 'contextra-core') else 'WP 2.4'
    else:
        return 'WP 2.3'

def get_commit_hash():
    try:
        res = subprocess.run(['git', 'rev-parse', '--short', 'HEAD'], capture_output=True, text=True, check=True)
        return res.stdout.strip()
    except Exception:
        return '014ee79f0'

def extract_symbols(content, mod_name):
    symbols = []
    # use mod::{a, b, c}
    use_group_pattern = re.compile(rf'use\s+{mod_name}::\{{([^}}]+)\}}')
    for m in use_group_pattern.finditer(content):
        group = m.group(1)
        for item in group.split(','):
            item = item.strip().split(' as ')[0].strip()
            if item:
                # clean up subpaths
                parts = [p for p in item.split('::') if p]
                if parts:
                    symbols.append(parts[-1])

    # use mod::path::symbol
    use_single_pattern = re.compile(rf'use\s+{mod_name}::([a-zA-Z0-9_:]+)')
    for m in use_single_pattern.finditer(content):
        path = m.group(1).strip()
        parts = [p for p in path.split('::') if p]
        if parts:
            symbols.append(parts[-1])

    # mod::path::symbol inline
    inline_pattern = re.compile(rf'{mod_name}::([a-zA-Z0-9_:]+)')
    for m in inline_pattern.finditer(content):
        path = m.group(1).strip()
        parts = [p for p in path.split('::') if p]
        if parts:
            symbols.append(parts[-1])

    return symbols

def main():
    root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
    os.chdir(root_dir)

    with open('Cargo.toml', 'rb') as f:
        root_cargo = tomllib.load(f)

    members = root_cargo['workspace']['members']

    crates = {}
    for m in members:
        cargo_path = os.path.join(m, 'Cargo.toml')
        if os.path.exists(cargo_path):
            with open(cargo_path, 'rb') as f:
                data = tomllib.load(f)
            name = data['package']['name']
            crates[name] = {
                'dir': m,
                'data': data
            }

    # Data structure:
    # edges_by_archive[archive_crate][dep_name] = { ... }
    edges_by_archive = {ac: {} for ac in ARCHIVE_CRATES}
    dep_summary = {}

    for archive_crate, mod_name in sorted(ARCHIVE_CRATES.items()):
        for dep_name, dep_info in sorted(crates.items()):
            if dep_name == archive_crate:
                continue
            data = dep_info['data']
            deps = data.get('dependencies', {})
            dev_deps = data.get('dev-dependencies', {})
            build_deps = data.get('build-dependencies', {})

            m_edge = None
            art = 'nein'
            opt_str = 'nein'
            if archive_crate in deps:
                spec = deps[archive_crate]
                is_opt = spec.get('optional', False) if isinstance(spec, dict) else False
                m_edge = ('normal', is_opt)
                art = 'normal'
                opt_str = 'ja' if is_opt else 'nein'
            elif archive_crate in dev_deps:
                spec = dev_deps[archive_crate]
                is_opt = spec.get('optional', False) if isinstance(spec, dict) else False
                m_edge = ('dev', is_opt)
                art = 'dev'
                opt_str = 'ja' if is_opt else 'nein'
            elif archive_crate in build_deps:
                spec = build_deps[archive_crate]
                is_opt = spec.get('optional', False) if isinstance(spec, dict) else False
                m_edge = ('build', is_opt)
                art = 'build'
                opt_str = 'ja' if is_opt else 'nein'

            crate_dir = dep_info['dir']
            src_files = []
            test_files = []
            symbols_counter = Counter()

            for r, dirs, files in os.walk(crate_dir):
                if 'target' in dirs:
                    dirs.remove('target')
                for file in files:
                    if file.endswith('.rs'):
                        fp = os.path.join(r, file)
                        rel = os.path.relpath(fp, crate_dir).replace('\\', '/')
                        try:
                            with open(fp, 'r', encoding='utf-8', errors='ignore') as f_rs:
                                content = f_rs.read()
                            if f'use {mod_name}' in content or f'{mod_name}::' in content:
                                if rel.startswith('src'):
                                    src_files.append(rel)
                                else:
                                    test_files.append(rel)
                                syms = extract_symbols(content, mod_name)
                                symbols_counter.update(syms)
                        except Exception:
                            pass

            total_files = len(src_files) + len(test_files)
            if m_edge or total_files > 0:
                target_wp = get_target_wp(dep_name, archive_crate)
                top_syms = [s for s, c in symbols_counter.most_common(10)]
                is_src_only = (m_edge is None and total_files > 0)

                edges_by_archive[archive_crate][dep_name] = {
                    'manifest_present': 'ja' if m_edge else 'nein',
                    'art': art,
                    'optional': opt_str,
                    'src_files_count': len(src_files),
                    'test_files_count': len(test_files),
                    'total_files_count': total_files,
                    'top_symbols': top_syms,
                    'target_wp': target_wp,
                    'src_only': is_src_only,
                }

                if dep_name not in dep_summary:
                    dep_summary[dep_name] = {
                        'manifest_edges': 0,
                        'src_files': 0,
                        'test_files': 0,
                        'total_files': 0,
                        'archive_edges': 0,
                        'target_wp': target_wp,
                    }
                if m_edge:
                    dep_summary[dep_name]['manifest_edges'] += 1
                dep_summary[dep_name]['src_files'] += len(src_files)
                dep_summary[dep_name]['test_files'] += len(test_files)
                dep_summary[dep_name]['total_files'] += total_files
                dep_summary[dep_name]['archive_edges'] += 1

    commit_hash = get_commit_hash()

    # Generate Markdown
    lines = []
    lines.append("# Kanten-Inventar Archiv-Crates — WP 2.1")
    lines.append("")
    lines.append(f"<!-- doc-stamp: {commit_hash} | Generiert: tools/generate_migration_edges.py -->")
    lines.append("")
    lines.append("> **Zweck.** Dieses Dokument listet alle Kanten von aktiven Crates zu Archiv-Crates.")
    lines.append("> Es ist die Abarbeitungsliste für WP 2.2, WP 2.3 und WP 2.4.")
    lines.append("> Erstellt und aktualisiert in WP 2.1 gemäß PLAN.md §7 (WP 2.1, Schritt 2 & 3).")
    lines.append("")
    lines.append("## Status")
    lines.append("")
    lines.append("| WP | Kanten-Ziel | Status |")
    lines.append("|---|---|---|")
    lines.append("| WP 2.2 | `engine` entkoppeln | 🔴 offen |")
    lines.append("| WP 2.3 | `db`, `mcp`, `contextra`, `core` entkoppeln | 🔴 offen |")
    lines.append("| WP 2.4 | Rand-Crates als optional markieren | 🔴 offen |")
    lines.append("| WP 2.1b | `bench` & Archiv-Crates (physisches Entfernen) | 🔴 offen |")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Ratchet-Baseline: Übersicht nach Dependent")
    lines.append("")
    lines.append("> **Ratchet-Regel:** Die Summen in dieser Tabelle dürfen in WP 2.2, 2.3 und 2.4 nur sinken.")
    lines.append("")
    lines.append("| Dependent | Manifest-Kanten | Quellcode-Dateien (src / tests / gesamt) | Archiv-Kanten | Ziel-WP |")
    lines.append("|---|---|---|---|---|")
    for dep_name, s in sorted(dep_summary.items()):
        lines.append(f"| {dep_name} | {s['manifest_edges']} | {s['src_files']} / {s['test_files']} / **{s['total_files']}** | {s['archive_edges']} | {s['target_wp']} |")
    lines.append("")
    lines.append("---")
    lines.append("")

    for archive_crate, mod_name in sorted(ARCHIVE_CRATES.items()):
        lines.append(f"## {archive_crate}")
        lines.append("")
        lines.append("| Dependent | Manifest | Art | Optional | Quellcode-Dateien (src / tests) | Top-10 Symbole | Zu erledigen in | Hinweis |")
        lines.append("|---|---|---|---|---|---|---|---|")

        deps_dict = edges_by_archive[archive_crate]
        if not deps_dict:
            lines.append("| (keine) | - | - | - | 0 / 0 | - | - | - |")
        else:
            for dep_name, info in sorted(deps_dict.items()):
                m_str = info['manifest_present']
                art_str = info['art']
                opt_str = info['optional']
                files_str = f"{info['src_files_count']} / {info['test_files_count']} ({info['total_files_count']})"
                syms_str = ", ".join(info['top_symbols']) if info['top_symbols'] else "-"
                wp_str = info['target_wp']
                note_str = "**nur Quellcode** (keine Manifest-Kante)" if info['src_only'] else "-"
                lines.append(f"| {dep_name} | {m_str} | {art_str} | {opt_str} | {files_str} | {syms_str} | {wp_str} | {note_str} |")

        lines.append("")
        lines.append(f"**Maßnahme:** {MASSNAHMEN.get(archive_crate, '-')}")
        lines.append("")
        lines.append("---")
        lines.append("")

    # Summary table of edges
    total_manifest = sum(s['manifest_edges'] for s in dep_summary.values())
    total_archive_edges = sum(s['archive_edges'] for s in dep_summary.values())
    total_files_sum = sum(s['total_files'] for s in dep_summary.values())

    lines.append("## Zusammenfassung")
    lines.append("")
    lines.append(f"- **Manifest-Kanten zu Archiv-Crates:** {total_manifest}")
    lines.append(f"- **Kanten gesamt (inkl. nur Quellcode):** {total_archive_edges}")
    lines.append(f"- **Betroffene Quellcode-Dateien gesamt:** {total_files_sum}")
    lines.append("")

    content = "\n".join(lines)
    with open('docs/spec/MIGRATION-EDGES.md', 'w', encoding='utf-8') as f_out:
        f_out.write(content)

    print(f"Erfolgreich docs/spec/MIGRATION-EDGES.md generiert (Commit: {commit_hash}).")

if __name__ == '__main__':
    main()
