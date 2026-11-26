#!/usr/bin/env python3
"""
agent_tooling_common.py — Shared library for Contextra diagnostic & analysis tools.

Designed with Principal-Thinking, Agentic-Engineering, and Context-Engineering:
- Deterministic, zero-panic, robust parsing of Rust source files.
- Comment-safe and string-safe scanning and tokenizing.
- Workspace discovery from Cargo.toml and capabilities.toml.
- Unified Markdown and JSON artifact generation into docs/diagnostics/ or custom paths.
- Command-line argument standardization across all diagnostic tools.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable, Dict, Generator, List, Optional, Set, Tuple


def get_repo_root() -> Path:
    """Resolve the repository root directory deterministically."""
    current = Path(__file__).resolve().parent
    while current != current.parent:
        if (current / "Cargo.toml").is_file() and (current / "crates").is_dir():
            return current
        current = current.parent
    return Path.cwd()


def get_diagnostics_dir(repo_root: Optional[Path] = None) -> Path:
    """Get the standard directory for diagnostic documents."""
    root = repo_root or get_repo_root()
    diag_dir = root / "docs" / "diagnostics"
    diag_dir.mkdir(parents=True, exist_ok=True)
    return diag_dir


@dataclass
class CrateMeta:
    name: str
    path: Path
    ring: Optional[int] = None
    role: Optional[str] = None
    category: Optional[str] = None
    manifest_path: Path = field(init=False)

    def __post_init__(self):
        self.manifest_path = self.path / "Cargo.toml"

    @property
    def src_path(self) -> Path:
        return self.path / "src"

    @property
    def tests_path(self) -> Path:
        return self.path / "tests"

    @property
    def benches_path(self) -> Path:
        return self.path / "benches"


def parse_capabilities_toml(repo_root: Optional[Path] = None) -> Dict[str, Dict[str, Any]]:
    """Parse capabilities.toml to get ring, class, and may_depend_on for each crate."""
    root = repo_root or get_repo_root()
    cap_file = root / "capabilities.toml"
    capabilities: Dict[str, Dict[str, Any]] = {}
    if not cap_file.is_file():
        return capabilities

    current_crate = None
    with open(cap_file, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            if line.startswith("[[crate]]"):
                current_crate = {}
                continue
            if current_crate is not None:
                if line.startswith("name ="):
                    name = line.split("=", 1)[1].strip().strip('"').strip("'")
                    current_crate["name"] = name
                    capabilities[name] = current_crate
                elif line.startswith("ring ="):
                    val = line.split("=", 1)[1].strip()
                    try:
                        current_crate["ring"] = int(val)
                    except ValueError:
                        current_crate["ring"] = None
                elif line.startswith("class ="):
                    current_crate["class"] = line.split("=", 1)[1].strip().strip('"')
                elif line.startswith("maturity ="):
                    current_crate["maturity"] = line.split("=", 1)[1].strip().strip('"')
                elif line.startswith("may_depend_on ="):
                    raw_deps = line.split("=", 1)[1].strip()
                    deps = [
                        d.strip().strip('"').strip("'")
                        for d in raw_deps.strip("[]").split(",")
                        if d.strip()
                    ]
                    current_crate["may_depend_on"] = deps
    return capabilities


def discover_crates(repo_root: Optional[Path] = None) -> Dict[str, CrateMeta]:
    """Discover all crates in crates/ and benchmarks/."""
    root = repo_root or get_repo_root()
    caps = parse_capabilities_toml(root)
    crates: Dict[str, CrateMeta] = {}

    search_dirs = [root / "crates", root / "benchmarks"]
    for sdir in search_dirs:
        if not sdir.is_dir():
            continue
        for child in sorted(sdir.iterdir()):
            if child.is_dir() and (child / "Cargo.toml").is_file():
                crate_name = child.name
                # Normalize crate name if Cargo.toml has [package] name
                actual_name = crate_name
                try:
                    with open(child / "Cargo.toml", "r", encoding="utf-8") as f:
                        for line in f:
                            if line.strip().startswith("name ="):
                                actual_name = line.split("=", 1)[1].strip().strip('"').strip("'")
                                break
                except Exception:
                    pass

                cap_info = caps.get(actual_name, {})
                ring = cap_info.get("ring")
                role = cap_info.get("class")
                crates[actual_name] = CrateMeta(
                    name=actual_name,
                    path=child,
                    ring=ring,
                    role=role,
                    category=sdir.name,
                )
    return crates


def strip_comments_and_strings(rust_code: str) -> str:
    """
    Remove block comments, line comments, and string/byte/raw-string literals from Rust code,
    replacing them with equivalent whitespace/spaces to preserve line numbers and byte offsets.
    """
    result = list(rust_code)
    n = len(rust_code)
    i = 0

    while i < n:
        # Check raw string: r#"..."# or r"..."
        if rust_code[i : i + 2] == 'r"' or rust_code[i : i + 3] in ('r#"', 'r##'):
            # parse raw string
            match = re.match(r'r(#*)"', rust_code[i:])
            if match:
                hashes = match.group(1)
                closer = f'"{hashes}'
                start = i
                i += match.end()
                close_idx = rust_code.find(closer, i)
                if close_idx != -1:
                    end = close_idx + len(closer)
                    for k in range(start, end):
                        if result[k] != "\n":
                            result[k] = " "
                    i = end
                    continue
                else:
                    for k in range(start, n):
                        if result[k] != "\n":
                            result[k] = " "
                    break

        # Check normal string literal
        if rust_code[i] == '"' and (i == 0 or rust_code[i - 1] != "\\"):
            start = i
            i += 1
            while i < n:
                if rust_code[i] == "\\" and i + 1 < n:
                    i += 2
                    continue
                if rust_code[i] == '"':
                    i += 1
                    break
                i += 1
            for k in range(start, min(i, n)):
                if result[k] != "\n":
                    result[k] = " "
            continue

        # Check char literal
        if rust_code[i] == "'" and (i == 0 or rust_code[i - 1] not in (
            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m",
            "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
            "_", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "&"
        )):
            # Distinguish lifetime 'a from char 'c'
            if i + 2 < n and rust_code[i + 2] == "'":
                start = i
                i += 3
                for k in range(start, i):
                    result[k] = " "
                continue
            elif i + 3 < n and rust_code[i + 1] == "\\" and rust_code[i + 3] == "'":
                start = i
                i += 4
                for k in range(start, i):
                    result[k] = " "
                continue

        # Check line comment
        if rust_code[i : i + 2] == "//":
            start = i
            while i < n and rust_code[i] != "\n":
                i += 1
            for k in range(start, i):
                result[k] = " "
            continue

        # Check block comment (handles nested comments)
        if rust_code[i : i + 2] == "/*":
            start = i
            depth = 1
            i += 2
            while i < n and depth > 0:
                if rust_code[i : i + 2] == "/*":
                    depth += 1
                    i += 2
                elif rust_code[i : i + 2] == "*/":
                    depth -= 1
                    i += 2
                else:
                    i += 1
            for k in range(start, min(i, n)):
                if result[k] != "\n":
                    result[k] = " "
            continue

        i += 1

    return "".join(result)


def iter_rust_files(crate_path: Path, include_tests: bool = True, include_benches: bool = True) -> Generator[Path, None, None]:
    """Iterate over all .rs files in a crate path."""
    for root, _, files in os.walk(crate_path):
        rel = os.path.relpath(root, crate_path)
        if "target" in rel.split(os.sep) or ".git" in rel.split(os.sep):
            continue
        if not include_tests and (rel == "tests" or rel.startswith("tests" + os.sep)):
            continue
        if not include_benches and (rel == "benches" or rel.startswith("benches" + os.sep)):
            continue
        for file in sorted(files):
            if file.endswith(".rs"):
                yield Path(root) / file


class DiagnosticReport:
    """Standardized report container for Agentic & Context Engineering."""

    def __init__(self, tool_name: str, title: str, description: str):
        self.tool_name = tool_name
        self.title = title
        self.description = description
        self.timestamp = datetime.now(timezone.utc).isoformat()
        self.metrics: Dict[str, Any] = {}
        self.findings: List[Dict[str, Any]] = []
        self.sections: List[Tuple[str, str]] = []  # (Header, Markdown Body)

    def set_metric(self, key: str, value: Any) -> None:
        self.metrics[key] = value

    def add_finding(self, finding: Dict[str, Any]) -> None:
        self.findings.append(finding)

    def add_section(self, header: str, markdown_content: str) -> None:
        self.sections.append((header, markdown_content))

    def to_dict(self) -> Dict[str, Any]:
        return {
            "tool_name": self.tool_name,
            "title": self.title,
            "description": self.description,
            "timestamp": self.timestamp,
            "metrics": self.metrics,
            "findings_count": len(self.findings),
            "findings": self.findings,
            "sections": [{"header": h, "content": c} for h, c in self.sections],
        }

    def to_markdown(self) -> str:
        lines = [
            f"# {self.title}",
            "",
            f"> **Werkzeug:** `{self.tool_name}`  ",
            f"> **Erstellungszeit:** `{self.timestamp}`  ",
            f"> **Status/Befunde:** `{len(self.findings)} Befunde erfasst`  ",
            "",
            f"## Beschreibung",
            self.description,
            "",
        ]

        if self.metrics:
            lines.append("## Metriken-Übersicht")
            lines.append("| Metrik | Wert |")
            lines.append("|---|---|")
            for k, v in self.metrics.items():
                lines.append(f"| `{k}` | `{v}` |")
            lines.append("")

        for header, content in self.sections:
            lines.append(f"## {header}")
            lines.append(content)
            lines.append("")

        if self.findings:
            lines.append("## Detaillierte Befunde")
            # If findings have standard keys: crate, file, line, message/kind
            sample = self.findings[0]
            if "crate" in sample and "file" in sample and "line" in sample:
                lines.append("| Crate | Datei | Zeile | Art / Beschreibung |")
                lines.append("|---|---|---|---|")
                for f in self.findings[:500]:  # Limit table rows to avoid overflow
                    crate = f.get("crate", "-")
                    file_ = f.get("file", "-")
                    line = f.get("line", "-")
                    desc = f.get("description", f.get("kind", f.get("message", "-")))
                    lines.append(f"| `{crate}` | `{file_}` | `{line}` | {desc} |")
                if len(self.findings) > 500:
                    lines.append(f"\n*... und {len(self.findings) - 500} weitere Befunde (siehe JSON-Export).*")
            else:
                lines.append("```json")
                lines.append(json.dumps(self.findings[:100], indent=2))
                lines.append("```")
            lines.append("")

        lines.append("---")
        lines.append(f"*Automatisch generiert durch `{self.tool_name}` für Agentic Engineering & Context Auditing.*")
        return "\n".join(lines)

    def write_artifacts(self, base_name: str, output_dir: Optional[Path] = None, json_only: bool = False) -> Tuple[Path, Optional[Path]]:
        out_dir = output_dir or get_diagnostics_dir()
        out_dir.mkdir(parents=True, exist_ok=True)

        json_path = out_dir / f"{base_name}.json"
        with open(json_path, "w", encoding="utf-8") as f:
            json.dump(self.to_dict(), f, indent=2)

        md_path = None
        if not json_only:
            md_path = out_dir / f"{base_name}.md"
            with open(md_path, "w", encoding="utf-8") as f:
                f.write(self.to_markdown())

        return json_path, md_path


def create_standard_cli(description: str) -> argparse.ArgumentParser:
    """Create a standardized argument parser for all Contextra diagnostic tools."""
    parser = argparse.ArgumentParser(
        description=description,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--crate", "-c",
        type=str,
        help="Target a specific crate (e.g., contextra-store). Defaults to workspace-wide scan.",
    )
    parser.add_argument(
        "--output-dir", "-o",
        type=str,
        help="Custom output directory for diagnostic artifacts (defaults to docs/diagnostics/).",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output raw JSON to stdout for agent pipelines.",
    )
    parser.add_argument(
        "--no-save",
        action="store_true",
        help="Do not save markdown/json documents to disk (print to stdout only).",
    )
    return parser
