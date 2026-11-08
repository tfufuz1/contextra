#!/usr/bin/env python3
"""
generate_function_inventory.py — Robust Function & Method Inventory Generator.

Extracts all function and method definitions from workspace Rust files with the canonical schema:
crate,ring,datei,zeile,ende,loc,qualifizierter_name,sichtbarkeit,async,unsafe,dokumentiert,generiert,kontext,signatur
"""

from __future__ import annotations

import argparse
import csv
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Tuple


@dataclass
class FunctionEntry:
    crate: str
    ring: str
    datei: str
    zeile: int
    ende: int
    loc: int
    qualifizierter_name: str
    sichtbarkeit: str
    async_flag: int
    unsafe_flag: int
    dokumentiert: int
    generiert: int
    kontext: str
    signatur: str


def get_ring_mapping(repo_root: Path) -> Dict[str, str]:
    cap_file = repo_root / "capabilities.toml"
    mapping: Dict[str, str] = {
        "(root)": "–",
        "xtask": "–",
        "contextra-bench": "–",
        "contextra-store-fuzz": "–",
    }
    if cap_file.is_file():
        content = cap_file.read_text(encoding="utf-8")
        matches = re.findall(
            r'\[crates\.([^\]]+)\][^\[]*?ring\s*=\s*[\x27"]([^\x27"]+)[\x27"]',
            content,
            re.DOTALL,
        )
        for crate_name, ring_val in matches:
            if ring_val == "Extension":
                mapping[crate_name] = "Ring 0"
            else:
                mapping[crate_name] = ring_val
    return mapping


def determine_crate_and_ring(rel_path: str, ring_map: Dict[str, str]) -> Tuple[str, str]:
    parts = rel_path.split("/")
    if parts[0] == "crates" and len(parts) > 1:
        c_name = parts[1]
    elif parts[0] == "benchmarks" and len(parts) > 1:
        if parts[1].endswith(".rs"):
            c_name = "(root)"
        else:
            c_name = parts[1]
    elif parts[0] == "xtask":
        c_name = "xtask"
    elif parts[0] in ("benches", "examples"):
        c_name = "(root)"
    else:
        c_name = "(root)"

    ring = ring_map.get(c_name, "Ring 0" if c_name.startswith("contextra") else "–")
    return c_name, ring


def clean_signature(sig_raw: str) -> str:
    sig = sig_raw.replace("\n", " ")
    sig = re.sub(r"\s+", " ", sig).strip()
    sig = sig.rstrip(";{ ").strip()
    return sig


def parse_rust_file_functions(
    content: str, rel_path: str, crate_name: str, ring: str
) -> List[FunctionEntry]:
    lines = content.splitlines()
    entries: List[FunctionEntry] = []
    is_generated = 1 if "generated" in rel_path.lower() else 0

    re_impl = re.compile(
        r"^(?:pub(?:\s*\([^)]+\))?\s+)?(?:unsafe\s+)?impl(?:<[^>]*>)?\s+(?:([A-Za-z0-9_:]+)(?:<[^>]*>)?\s+for\s+)?([A-Za-z0-9_:]+)"
    )
    re_trait = re.compile(
        r"^(?:pub(?:\s*\([^)]+\))?\s+)?(?:unsafe\s+)?trait\s+([A-Za-z0-9_]+)"
    )
    re_fn_start = re.compile(
        r"^(?:([a-z_0-9() ]*?)\s+)?(?:(async)\s+)?(?:(unsafe)\s+)?(?:extern(?:\s*\"[^\"]+\")?\s+)?fn\s+([A-Za-z0-9_]+)"
    )

    context_stack: List[Tuple[int, str, str]] = []
    current_depth = 0
    in_block_comment = False

    i = 0
    n = len(lines)
    while i < n:
        raw_line = lines[i]
        stripped = raw_line.strip()

        if in_block_comment:
            if "*/" in stripped:
                in_block_comment = False
            i += 1
            continue
        if stripped.startswith("/*"):
            if "*/" not in stripped:
                in_block_comment = True
            i += 1
            continue

        if stripped.startswith("#[cfg(test)]") or stripped.startswith("mod tests {") or stripped.startswith("mod test {") or stripped.startswith("mod tests;"):
            brace_scan = i
            depth = 0
            found_open = False
            while brace_scan < n:
                line_text = lines[brace_scan]
                for ch in line_text:
                    if ch == '{':
                        depth += 1
                        found_open = True
                    elif ch == '}':
                        depth -= 1
                        if found_open and depth == 0:
                            break
                if found_open and depth == 0:
                    break
                brace_scan += 1
            i = brace_scan + 1
            continue

        m_impl = re_impl.match(stripped)
        if m_impl:
            trait_name = m_impl.group(1)
            target_type = m_impl.group(2)
            target_clean = target_type.split("::")[-1]
            if trait_name:
                trait_clean = trait_name.split("::")[-1]
                ctx_str = f"impl {trait_clean} for {target_clean}"
            else:
                ctx_str = f"impl {target_clean}"

            brace_line_idx = i
            while brace_line_idx < n and "{" not in lines[brace_line_idx]:
                brace_line_idx += 1
            context_stack.append((current_depth + 1, ctx_str, target_clean))

        m_trait = re_trait.match(stripped)
        if m_trait:
            t_name = m_trait.group(1)
            ctx_str = f"trait {t_name}"
            context_stack.append((current_depth + 1, ctx_str, t_name))

        if "fn " in stripped and not stripped.startswith("//") and not stripped.startswith("#"):
            m_fn = re_fn_start.search(stripped)
            if m_fn:
                vis_part = (m_fn.group(1) or "").strip()
                fn_prefix = stripped[:m_fn.end()]
                is_async = 1 if "async" in fn_prefix.split() else 0
                is_unsafe = 1 if "unsafe" in fn_prefix.split() else 0
                fn_name = m_fn.group(4)

                is_real_fn = False
                prefix = stripped[: m_fn.start()].strip()
                if prefix == "" or prefix.endswith(";") or prefix.endswith("{") or prefix.endswith("}"):
                    is_real_fn = True

                if is_real_fn:
                    start_line = i + 1

                    if "pub(crate)" in vis_part or "pub(super)" in vis_part:
                        vis = "crate"
                    elif "pub" in vis_part:
                        vis = "pub"
                    else:
                        vis = "priv"

                    is_documented = 0
                    doc_scan = i - 1
                    while doc_scan >= 0:
                        prev = lines[doc_scan].strip()
                        if prev.startswith("#["):
                            doc_scan -= 1
                            continue
                        if prev.startswith("///") or prev.startswith("//!"):
                            is_documented = 1
                            break
                        break

                    sig_lines = []
                    sig_idx = i
                    has_body = False
                    while sig_idx < n:
                        s_line = lines[sig_idx].strip()
                        if "//" in s_line:
                            s_line = s_line.split("//")[0].strip()
                        sig_lines.append(s_line)
                        if "{" in s_line:
                            has_body = True
                            break
                        if ";" in s_line:
                            break
                        sig_idx += 1

                    sig_text = clean_signature(" ".join(sig_lines))

                    if has_body:
                        fn_depth = 0
                        end_line = start_line
                        for scan_i in range(i, n):
                            line_chars = lines[scan_i]
                            in_str = False
                            for c_idx, ch in enumerate(line_chars):
                                if ch == '"' and (c_idx == 0 or line_chars[c_idx - 1] != '\\'):
                                    in_str = not in_str
                                if not in_str:
                                    if ch == '{':
                                        fn_depth += 1
                                    elif ch == '}':
                                        fn_depth -= 1
                                        if fn_depth == 0:
                                            end_line = scan_i + 1
                                            break
                            if fn_depth == 0 and scan_i >= i:
                                break
                    else:
                        end_line = sig_idx + 1

                    loc = end_line - start_line + 1

                    cur_ctx = ""
                    target_name = ""
                    if context_stack:
                        cur_ctx = context_stack[-1][1]
                        target_name = context_stack[-1][2]

                    if target_name:
                        qual_name = f"{target_name}::{fn_name}"
                    else:
                        qual_name = fn_name

                    entries.append(
                        FunctionEntry(
                            crate=crate_name,
                            ring=ring,
                            datei=rel_path,
                            zeile=start_line,
                            ende=end_line,
                            loc=loc,
                            qualifizierter_name=qual_name,
                            sichtbarkeit=vis,
                            async_flag=is_async,
                            unsafe_flag=is_unsafe,
                            dokumentiert=is_documented,
                            generiert=is_generated,
                            kontext=cur_ctx,
                            signatur=sig_text,
                        )
                    )

        for ch in stripped:
            if ch == '{':
                current_depth += 1
            elif ch == '}':
                current_depth -= 1
                while context_stack and context_stack[-1][0] > current_depth:
                    context_stack.pop()

        i += 1

    return entries


def scan_workspace_functions(
    repo_root: Path, git_commit: Optional[str] = None
) -> List[FunctionEntry]:
    ring_map = get_ring_mapping(repo_root)
    all_entries: List[FunctionEntry] = []

    if git_commit:
        cmd = ["git", "ls-tree", "-r", git_commit, "--name-only"]
        files = subprocess.check_output(cmd, cwd=repo_root).decode().splitlines()
    else:
        files = []
        for root, dirs, fnames in os.walk(repo_root):
            if "target" in dirs:
                dirs.remove("target")
            if ".git" in dirs:
                dirs.remove(".git")
            for f in fnames:
                if f.endswith(".rs"):
                    rel = os.path.relpath(os.path.join(root, f), repo_root).replace("\\", "/")
                    files.append(rel)

    rs_files = [
        f for f in files
        if f.endswith(".rs")
        and not f.startswith("xtask/")
        and "/tests/" not in f
        and not f.endswith("_test.rs")
        and not f.endswith("_tests.rs")
        and not f.endswith("/tests.rs")
        and not (f.startswith("crates/") and "/benches/" in f)
    ]
    rs_files.sort()

    for rel_path in rs_files:
        c_name, ring = determine_crate_and_ring(rel_path, ring_map)

        if git_commit:
            try:
                content = subprocess.check_output(
                    ["git", "show", f"{git_commit}:{rel_path}"],
                    cwd=repo_root,
                    stderr=subprocess.DEVNULL,
                ).decode("utf-8", errors="replace")
            except Exception:
                continue
        else:
            try:
                content = (repo_root / rel_path).read_text(encoding="utf-8", errors="replace")
            except Exception:
                continue

        file_entries = parse_rust_file_functions(content, rel_path, c_name, ring)
        all_entries.extend(file_entries)

    return all_entries


def write_inventory_csv(entries: List[FunctionEntry], out_path: Path):
    fieldnames = [
        "crate",
        "ring",
        "datei",
        "zeile",
        "ende",
        "loc",
        "qualifizierter_name",
        "sichtbarkeit",
        "async",
        "unsafe",
        "dokumentiert",
        "generiert",
        "kontext",
        "signatur",
    ]
    with open(out_path, "w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        for e in entries:
            writer.writerow({
                "crate": e.crate,
                "ring": e.ring,
                "datei": e.datei,
                "zeile": e.zeile,
                "ende": e.ende,
                "loc": e.loc,
                "qualifizierter_name": e.qualifizierter_name,
                "sichtbarkeit": e.sichtbarkeit,
                "async": e.async_flag,
                "unsafe": e.unsafe_flag,
                "dokumentiert": e.dokumentiert,
                "generiert": e.generiert,
                "kontext": e.kontext,
                "signatur": e.signatur,
            })


def write_inventory_markdown(entries: List[FunctionEntry], out_path: Path, title: str):
    from collections import Counter

    total_funcs = len(entries)
    crates_count = Counter(e.crate for e in entries)
    vis_count = Counter(e.sichtbarkeit for e in entries)
    async_count = sum(1 for e in entries if e.async_flag == 1)
    unsafe_count = sum(1 for e in entries if e.unsafe_flag == 1)
    doc_count = sum(1 for e in entries if e.dokumentiert == 1)

    lines = [
        f"# {title}",
        "",
        f"> **Gesamtanzahl Funktionen:** {total_funcs}  ",
        f"> **Öffentlich (pub):** {vis_count.get('pub', 0)} | **Crate (pub(crate)):** {vis_count.get('crate', 0)} | **Privat:** {vis_count.get('priv', 0)}  ",
        f"> **Async:** {async_count} | **Unsafe:** {unsafe_count} | **Dokumentiert:** {doc_count}  ",
        "",
        "## Übersicht nach Crates",
        "",
        "| Crate | Ring | Funktionen | Pub | Privat | Async | Unsafe |",
        "|---|---|---|---|---|---|---|",
    ]

    crate_rings = {}
    for e in entries:
        crate_rings[e.crate] = e.ring

    for crate in sorted(crates_count.keys()):
        c_entries = [e for e in entries if e.crate == crate]
        c_pub = sum(1 for e in c_entries if e.sichtbarkeit == "pub")
        c_priv = sum(1 for e in c_entries if e.sichtbarkeit == "priv")
        c_async = sum(1 for e in c_entries if e.async_flag == 1)
        c_unsafe = sum(1 for e in c_entries if e.unsafe_flag == 1)
        ring = crate_rings.get(crate, "–")
        lines.append(
            f"| `{crate}` | {ring} | {len(c_entries)} | {c_pub} | {c_priv} | {c_async} | {c_unsafe} |"
        )

    lines.extend([
        "",
        "## Detailliertes Verzeichnis (Auszug / Kernfunktionen)",
        "",
        "| Crate | Datei | Funktion / Methode | Sichtbarkeit | Kontext | Signatur |",
        "|---|---|---|---|---|---|",
    ])

    pub_entries = [e for e in entries if e.sichtbarkeit == "pub"]
    for e in pub_entries[:500]:
        sig_escaped = e.signatur.replace("|", "\\|")
        lines.append(
            f"| `{e.crate}` | `{e.datei}:{e.zeile}` | `{e.qualifizierter_name}` | `{e.sichtbarkeit}` | {e.kontext} | `{sig_escaped}` |"
        )

    if len(pub_entries) > 500:
        lines.append(f"\n*(Weitere {len(pub_entries) - 500} öffentliche Funktionen in zugehöriger CSV-Datei)*\n")

    out_path.write_text("\n".join(lines), encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description="Generate function inventory.")
    parser.add_argument("--commit", default=None, help="Git commit / tag to inspect.")
    parser.add_argument("--head", action="store_true", help="Inspect current HEAD filesystem.")
    parser.add_argument("--out-csv", required=True, help="Path for output CSV file.")
    parser.add_argument("--out-md", required=True, help="Path for output Markdown file.")
    parser.add_argument("--title", default="Funktionsinventar Contextra", help="Markdown title.")
    args = parser.parse_args()

    repo_root = Path.cwd()
    entries = scan_workspace_functions(repo_root, git_commit=args.commit)
    print(f"Extracted {len(entries)} functions (commit={args.commit}).")

    out_csv = Path(args.out_csv)
    out_md = Path(args.out_md)
    out_csv.parent.mkdir(parents=True, exist_ok=True)
    out_md.parent.mkdir(parents=True, exist_ok=True)

    write_inventory_csv(entries, out_csv)
    write_inventory_markdown(entries, out_md, args.title)
    print(f"Saved: {out_csv} and {out_md}")


if __name__ == "__main__":
    main()
