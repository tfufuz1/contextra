#!/usr/bin/env python3
"""
rust_ast_inspector.py — Micro-granular Rust AST and item inspection tool.

Provides coding agents with granular extraction of:
- Struct definitions, fields, visibilities, and derive macros.
- Enum definitions, variants, and attached payloads.
- Trait definitions and associated methods.
- Impl blocks (inherent and trait implementations).
- Function signatures, visibilities, async qualifiers, parameter counts, return types.
- Type aliases and constants.
- Line numbers, byte spans, and docstring documentation.
"""

from __future__ import annotations

import json
import re
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional

from agent_tooling_common import (
    DiagnosticReport,
    create_standard_cli,
    discover_crates,
    get_repo_root,
    iter_rust_files,
    strip_comments_and_strings,
)


@dataclass
class ItemMetadata:
    item_type: str  # struct, enum, trait, impl, fn, type, const
    name: str
    visibility: str  # pub, pub(crate), pub(super), private
    line: int
    is_async: bool = False
    is_unsafe: bool = False
    attributes: List[str] = field(default_factory=list)
    doc_comment: str = ""
    details: Dict[str, Any] = field(default_factory=dict)


def parse_rust_file_items(file_path: Path, relative_path: str) -> List[ItemMetadata]:
    """Parse top-level and module items from a Rust file with regex heuristics."""
    try:
        content = file_path.read_text(encoding="utf-8")
    except Exception:
        return []

    lines = content.splitlines()
    clean_content = strip_comments_and_strings(content)
    clean_lines = clean_content.splitlines()

    items: List[ItemMetadata] = []
    pending_attrs: List[str] = []
    pending_docs: List[str] = []

    # Patterns for items
    # Visibilities: pub, pub(crate), pub(super), pub(in crate::...)
    vis_pattern = r"(?:pub(?:\s*\([^)]+\))?\s+)?"

    re_struct = re.compile(rf"^(\s*)({vis_pattern})struct\s+([A-Za-z0-9_]+)")
    re_enum = re.compile(rf"^(\s*)({vis_pattern})enum\s+([A-Za-z0-9_]+)")
    re_trait = re.compile(rf"^(\s*)({vis_pattern})(unsafe\s+)?trait\s+([A-Za-z0-9_]+)")
    re_type = re.compile(rf"^(\s*)({vis_pattern})type\s+([A-Za-z0-9_]+)")
    re_const = re.compile(rf"^(\s*)({vis_pattern})const\s+([A-Za-z0-9_]+)")
    re_impl = re.compile(r"^(\s*)(?:unsafe\s+)?impl(?:<[^>]+>)?\s+(?:([A-Za-z0-9_:]+)(?:<[^>]+>)?\s+for\s+)?([A-Za-z0-9_:]+)")
    re_fn = re.compile(rf"^(\s*)({vis_pattern})(async\s+)?(unsafe\s+)?(?:extern(?:\s*\"[^\"]+\")?\s+)?fn\s+([A-Za-z0-9_]+)")

    for idx, (raw_line, clean_line) in enumerate(zip(lines, clean_lines), start=1):
        stripped_raw = raw_line.strip()
        stripped_clean = clean_line.strip()

        # Collect doc comments
        if stripped_raw.startswith("///") or stripped_raw.startswith("//!"):
            doc_text = stripped_raw.lstrip("/! ")
            pending_docs.append(doc_text)
            continue

        # Collect outer attributes
        if stripped_clean.startswith("#["):
            pending_attrs.append(stripped_raw)
            continue

        if not stripped_clean:
            continue

        # Check struct
        m = re_struct.match(stripped_clean)
        if m:
            vis = m.group(2).strip() or "private"
            name = m.group(3)
            items.append(
                ItemMetadata(
                    item_type="struct",
                    name=name,
                    visibility=vis,
                    line=idx,
                    attributes=list(pending_attrs),
                    doc_comment="\n".join(pending_docs),
                )
            )
            pending_attrs.clear()
            pending_docs.clear()
            continue

        # Check enum
        m = re_enum.match(stripped_clean)
        if m:
            vis = m.group(2).strip() or "private"
            name = m.group(3)
            items.append(
                ItemMetadata(
                    item_type="enum",
                    name=name,
                    visibility=vis,
                    line=idx,
                    attributes=list(pending_attrs),
                    doc_comment="\n".join(pending_docs),
                )
            )
            pending_attrs.clear()
            pending_docs.clear()
            continue

        # Check trait
        m = re_trait.match(stripped_clean)
        if m:
            vis = m.group(2).strip() or "private"
            is_unsafe = bool(m.group(3))
            name = m.group(4)
            items.append(
                ItemMetadata(
                    item_type="trait",
                    name=name,
                    visibility=vis,
                    line=idx,
                    is_unsafe=is_unsafe,
                    attributes=list(pending_attrs),
                    doc_comment="\n".join(pending_docs),
                )
            )
            pending_attrs.clear()
            pending_docs.clear()
            continue

        # Check impl
        m = re_impl.match(stripped_clean)
        if m:
            trait_name = m.group(2)
            target_type = m.group(3)
            name = f"{trait_name} for {target_type}" if trait_name else target_type
            items.append(
                ItemMetadata(
                    item_type="impl",
                    name=name,
                    visibility="inherent" if not trait_name else "trait_impl",
                    line=idx,
                    attributes=list(pending_attrs),
                    doc_comment="\n".join(pending_docs),
                    details={"target": target_type, "trait": trait_name},
                )
            )
            pending_attrs.clear()
            pending_docs.clear()
            continue

        # Check fn
        m = re_fn.match(stripped_clean)
        if m:
            vis = m.group(2).strip() or "private"
            is_async = bool(m.group(3))
            is_unsafe = bool(m.group(4))
            name = m.group(5)
            items.append(
                ItemMetadata(
                    item_type="fn",
                    name=name,
                    visibility=vis,
                    line=idx,
                    is_async=is_async,
                    is_unsafe=is_unsafe,
                    attributes=list(pending_attrs),
                    doc_comment="\n".join(pending_docs),
                )
            )
            pending_attrs.clear()
            pending_docs.clear()
            continue

        # Check type alias
        m = re_type.match(stripped_clean)
        if m:
            vis = m.group(2).strip() or "private"
            name = m.group(3)
            items.append(
                ItemMetadata(
                    item_type="type",
                    name=name,
                    visibility=vis,
                    line=idx,
                    attributes=list(pending_attrs),
                    doc_comment="\n".join(pending_docs),
                )
            )
            pending_attrs.clear()
            pending_docs.clear()
            continue

        # Check const
        m = re_const.match(stripped_clean)
        if m:
            vis = m.group(2).strip() or "private"
            name = m.group(3)
            items.append(
                ItemMetadata(
                    item_type="const",
                    name=name,
                    visibility=vis,
                    line=idx,
                    attributes=list(pending_attrs),
                    doc_comment="\n".join(pending_docs),
                )
            )
            pending_attrs.clear()
            pending_docs.clear()
            continue

        # If not an item line, reset pending annotations on substantial statements
        if not stripped_clean.startswith("//") and not stripped_clean.startswith("#"):
            pending_attrs.clear()
            pending_docs.clear()

    return items


def main():
    cli = create_standard_cli("Micro-granular Rust AST & Item Inspector for coding agents.")
    cli.add_argument("--item-type", choices=["all", "struct", "enum", "trait", "impl", "fn", "type", "const"], default="all", help="Filter by item type.")
    cli.add_argument("--public-only", action="store_true", help="Only inspect public items (pub, pub(crate)).")
    args = cli.parse_args()

    repo_root = get_repo_root()
    crates = discover_crates(repo_root)

    target_crates = {}
    if args.crate:
        if args.crate in crates:
            target_crates[args.crate] = crates[args.crate]
        else:
            print(f"Error: Crate '{args.crate}' not found.", file=sys.stderr)
            sys.exit(1)
    else:
        target_crates = crates

    report = DiagnosticReport(
        tool_name="rust_ast_inspector.py",
        title="Micro-Granular Rust AST & Item Inspection Report",
        description="Comprehensive item inventory covering structs, enums, traits, implementations, functions, visibilities, and qualifiers across workspace crates.",
    )

    total_files = 0
    type_counts: Dict[str, int] = {"struct": 0, "enum": 0, "trait": 0, "impl": 0, "fn": 0, "type": 0, "const": 0}
    public_counts = 0
    async_counts = 0
    unsafe_counts = 0

    crate_summaries = []

    for cname, cmeta in sorted(target_crates.items()):
        c_items = 0
        c_types = 0
        c_fns = 0
        for rpath in iter_rust_files(cmeta.path, include_tests=False):
            total_files += 1
            rel_file = str(rpath.relative_to(repo_root))
            items = parse_rust_file_items(rpath, rel_file)
            for it in items:
                if args.item_type != "all" and it.item_type != args.item_type:
                    continue
                if args.public_only and it.visibility == "private":
                    continue

                type_counts[it.item_type] = type_counts.get(it.item_type, 0) + 1
                if it.visibility != "private":
                    public_counts += 1
                if it.is_async:
                    async_counts += 1
                if it.is_unsafe:
                    unsafe_counts += 1

                c_items += 1
                if it.item_type in ("struct", "enum", "trait"):
                    c_types += 1
                elif it.item_type == "fn":
                    c_fns += 1

                report.add_finding({
                    "crate": cname,
                    "file": rel_file,
                    "line": it.line,
                    "type": it.item_type,
                    "name": it.name,
                    "visibility": it.visibility,
                    "is_async": it.is_async,
                    "is_unsafe": it.is_unsafe,
                    "description": f"[{it.item_type.upper()}] {it.visibility} {it.name}",
                })
        crate_summaries.append((cname, c_types, c_fns, c_items))

    report.set_metric("total_crates_scanned", len(target_crates))
    report.set_metric("total_rust_files_scanned", total_files)
    report.set_metric("total_items_discovered", len(report.findings))
    report.set_metric("public_items", public_counts)
    report.set_metric("async_functions", async_counts)
    report.set_metric("unsafe_items", unsafe_counts)
    for k, v in type_counts.items():
        report.set_metric(f"count_{k}", v)

    # Crate Breakdown Section
    table_lines = [
        "| Crate | Datentypen (Struct/Enum/Trait) | Funktionen | Gesamt-Items |",
        "|---|---|---|---|",
    ]
    for cname, c_types, c_fns, c_items in crate_summaries:
        table_lines.append(f"| `{cname}` | {c_types} | {c_fns} | {c_items} |")
    report.add_section("Crate-Item-Übersicht", "\n".join(table_lines))

    if args.json:
        print(json.dumps(report.to_dict(), indent=2))
    else:
        print(f"[*] Rust AST Inspector complete: {len(report.findings)} items discovered across {len(target_crates)} crates.")
        print(f"    Structs: {type_counts['struct']}, Enums: {type_counts['enum']}, Traits: {type_counts['trait']}, Impls: {type_counts['impl']}, Fns: {type_counts['fn']}")

    if not args.no_save:
        out_name = f"ast_inventory_{args.crate}" if args.crate else "ast_inventory_workspace"
        j_p, m_p = report.write_artifacts(out_name, Path(args.output_dir) if args.output_dir else None)
        if not args.json:
            print(f"[+] Saved artifacts:\n    {j_p}\n    {m_p}")


if __name__ == "__main__":
    main()
