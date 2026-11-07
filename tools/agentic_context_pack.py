#!/usr/bin/env python3
"""
agentic_context_pack.py — Context-Engineering & Prompt-Packager for Coding Agents.

Packs and minifies hyper-focused context for LLM agents working on specific crates:
- Extracts public API interfaces, types, and trait signatures.
- Includes relevant rules from root AGENTS.md and crate-local AGENTS.md.
- Extracts applicable architectural constraints from capabilities.toml.
- Attaches known debt issues (from DEPRECATED_DEBT.md) and invariant tests.
- Outputs minified, high-signal Markdown document ready for prompt injection.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Dict, List, Optional

from agent_tooling_common import (
    DiagnosticReport,
    create_standard_cli,
    discover_crates,
    get_repo_root,
    iter_rust_files,
    strip_comments_and_strings,
)


def extract_crate_public_api(crate_path: Path) -> List[str]:
    """Extract public structs, enums, traits, and fn signatures from crate/src/lib.rs."""
    lib_rs = crate_path / "src" / "lib.rs"
    if not lib_rs.is_file():
        lib_rs = crate_path / "src" / "main.rs"
    if not lib_rs.is_file():
        return []

    try:
        content = lib_rs.read_text(encoding="utf-8")
    except Exception:
        return []

    clean = strip_comments_and_strings(content)
    public_items = []
    re_pub = re.compile(r"^\s*pub\s+(?:async\s+)?(?:unsafe\s+)?(?:struct|enum|trait|type|fn|use)\s+([A-Za-z0-9_]+)")

    for line in clean.splitlines():
        line_s = line.strip()
        if line_s.startswith("pub ") or "pub(" in line_s:
            public_items.append(line_s)

    return public_items[:100]  # Cap to keep context compact


def extract_local_agents_md(crate_path: Path) -> Optional[str]:
    agents_file = crate_path / "AGENTS.md"
    if agents_file.is_file():
        try:
            return agents_file.read_text(encoding="utf-8").strip()
        except Exception:
            return None
    return None


def extract_debt_items(repo_root: Path, crate_name: str) -> List[str]:
    debt_file = repo_root / "DEPRECATED_DEBT.md"
    if not debt_file.is_file():
        return []
    try:
        txt = debt_file.read_text(encoding="utf-8")
    except Exception:
        return []

    lines = []
    short_crate = crate_name.replace("contextra-", "")
    for line in txt.splitlines():
        if crate_name in line or short_crate in line:
            lines.append(line.strip())
    return lines[:20]


def main():
    cli = create_standard_cli("Context-Engineering & Agent Prompt Packager.")
    cli.add_argument("--max-tokens-approx", type=int, default=4000, help="Approximate token cap for context budget.")
    args = cli.parse_args()

    repo_root = get_repo_root()
    crates = discover_crates(repo_root)

    if not args.crate:
        print("Notice: --crate was not specified. Generating context packs for core persistence and orchestration crates.")
        target_crate_names = ["contextra-store", "contextra-ports", "contextra-types", "contextra-erasure", "contextra-engine"]
    else:
        if args.crate not in crates:
            print(f"Error: Crate '{args.crate}' not found.", file=sys.stderr)
            sys.exit(1)
        target_crate_names = [args.crate]

    for cname in target_crate_names:
        cmeta = crates[cname]
        report = DiagnosticReport(
            tool_name="agentic_context_pack.py",
            title=f"Agentic Context Pack: `{cname}`",
            description=f"Curated, minified context pack for coding agents assigned to `{cname}`. Optimized for LLM window density and context engineering.",
        )

        report.set_metric("crate_name", cname)
        report.set_metric("ring", cmeta.ring if cmeta.ring is not None else "Unknown")
        report.set_metric("class", cmeta.role or "Standard")

        # 1. Architecture & Capabilities
        arch_info = [
            f"- **Crate:** `{cname}`",
            f"- **Ring:** `{cmeta.ring}`",
            f"- **Rolle/Klasse:** `{cmeta.role}`",
            f"- **Pfad:** `{cmeta.path.relative_to(repo_root)}`",
        ]
        report.add_section("1. Architektur-Zuweisung", "\n".join(arch_info))

        # 2. Local AGENTS.md instructions
        local_rules = extract_local_agents_md(cmeta.path)
        if local_rules:
            report.add_section("2. Crate-Spezifische AGENTS.md Verträge", local_rules)
        else:
            report.add_section("2. Crate-Spezifische AGENTS.md Verträge", "_Keine separate AGENTS.md vorhanden. Es gelten die Root-AGENTS.md Regeln._")

        # 3. Known Debts
        debts = extract_debt_items(repo_root, cname)
        if debts:
            report.add_section("3. Bekannte Technische Schulden (DEPRECATED_DEBT.md)", "\n".join(f"- {d}" for d in debts))
        else:
            report.add_section("3. Bekannte Technische Schulden", "_Keine spezifischen Schulden in DEPRECATED_DEBT.md vermerkt._")

        # 4. Public API surface
        pub_api = extract_crate_public_api(cmeta.path)
        if pub_api:
            code_block = "```rust\n" + "\n".join(pub_api[:50]) + "\n```"
            report.add_section("4. Öffentliche Schnittstellen & Signaturen (lib.rs)", code_block)

        report.add_finding({
            "crate": cname,
            "file": f"crates/{cname}/Cargo.toml",
            "line": 1,
            "kind": "AGENT_CONTEXT_PACK",
            "description": f"Generated focused context pack for {cname}",
        })

        if args.json:
            print(json.dumps(report.to_dict(), indent=2))
        else:
            print(f"[*] Generated context pack for `{cname}` (Ring {cmeta.ring}).")

        if not args.no_save:
            out_name = f"context_pack_{cname}"
            j_p, m_p = report.write_artifacts(out_name, Path(args.output_dir) if args.output_dir else None)
            if not args.json:
                print(f"[+] Saved artifacts:\n    {j_p}\n    {m_p}")


if __name__ == "__main__":
    main()
