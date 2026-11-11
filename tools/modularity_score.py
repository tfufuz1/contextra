#!/usr/bin/env python3
"""
tools/modularity_score.py — Modularitäts-Score M je Crate (SPEC §8.6 / Entkopplungsbericht §5).

Formel nach SPEC §8.6 / Entkopplungsbericht §5:
Start bei 100 Punkten.
Abzüge:
- Kern-Abhängigkeiten über dem Zielzustand aus SPEC §3.1 (12 Punkte je Abweichung, max. 35)
- Ungegatete Brückenreferenzen: 15 * log10(1 + n) (max. 40 Punkte)
- Verfehlte Null-Prüfung G-L2 bei E0/E1 (-15 Punkte)
- Fehlende compat/-Brücke oder fehlendes Workspace-Feature bei E0/E1 mit types/ports (je -10 Punkte)
- Fehlender eigener Fehlertyp bei E0/E1 (-5 Punkte)

Score M ist auf 0..100 normiert.
"""

from __future__ import annotations

import argparse
import json
import math
import re
import sys
import tomllib
from pathlib import Path
from typing import Any, Dict, List, Tuple

from gate_bridge_null import run_gate_bridge_null


TARGET_CORE_DEPS = {
    "contextra-ids": set(),
    "contextra-env": set(),
    "contextra-sys": set(),
    "contextra-durable-fs": set(),
    "contextra-simd": set(),
    "contextra-wire": set(),
    "contextra-adapt": set(),
    "contextra-ingest": set(),
    "contextra-privacy": set(),
    "contextra-sandbox": set(),
    "contextra-prompt-guard": set(),
    "contextra-infer-onnx": set(),
    "contextra-erasure": set(),
    "contextra-license": set(),
    "contextra-audit-export": set(),
    "contextra-avv-generator": set(),
    "contextra-mvcc": {"contextra-ids", "contextra-env"},
    "contextra-crypto": {"contextra-ids", "contextra-env"},
    "contextra-text": {"contextra-ids", "contextra-env"},
    "contextra-graph": {"contextra-ids", "contextra-env"},
    "contextra-rank": {"contextra-ids", "contextra-env"},
    "contextra-checkpoint": {"contextra-ids", "contextra-env"},
    "contextra-kvcache": {"contextra-env"},
    "contextra-vault": {"contextra-sys"},
    "contextra-infer-ollama": {"contextra-env"},
    "contextra-infer-candle": {"contextra-env"},
    "contextra-store": {"contextra-ids", "contextra-env", "contextra-sys", "contextra-mvcc", "contextra-crypto"},
    "contextra-vector": {"contextra-ids", "contextra-env", "contextra-mvcc"},
    "contextra-types": {"contextra-ids"},
    "contextra-ports": {"contextra-types", "contextra-ids", "contextra-env"},
    "contextra-router": {"contextra-types", "contextra-ports", "contextra-adapt"},
    "contextra-agent": {"contextra-types", "contextra-ports"},
    "contextra-cognition": {"contextra-types", "contextra-ports"},
    "contextra-engine": {
        "contextra-ids", "contextra-env", "contextra-types", "contextra-ports",
        "contextra-mvcc", "contextra-store", "contextra-vector", "contextra-text",
        "contextra-graph", "contextra-rank", "contextra-checkpoint",
    },
    "contextra-db": {"contextra-engine", "contextra-types", "contextra-ports", "contextra-vault"},
    "contextra": {"contextra-db", "contextra-types", "contextra-ports"},
    "contextra-mcp": {
        "contextra", "contextra-types", "contextra-ports", "contextra-wire",
        "contextra-rank", "contextra-crypto", "contextra-privacy", "contextra-router",
        "contextra-prompt-guard",
    },
    "contextra-py": {"contextra-types", "contextra", "contextra-wire"},
    "contextra-testkit": {"contextra-ports", "contextra-types", "contextra-env"},
    "contextra-bench": {"contextra-store", "contextra-vector", "contextra-text", "contextra-db"},
}


def parse_manifest_deps(manifest_path: Path) -> Tuple[set[str], set[str]]:
    """Gibt (normale interne deps, optionale interne deps) zurück."""
    if not manifest_path.is_file():
        return set(), set()
    with open(manifest_path, "rb") as f:
        data = tomllib.load(f)
    deps = data.get("dependencies", {})
    normal_internal = set()
    optional_internal = set()
    for name, spec in deps.items():
        if name.startswith("contextra-") or name == "contextra":
            if isinstance(spec, dict) and spec.get("optional", False):
                optional_internal.add(name)
            else:
                normal_internal.add(name)
    return normal_internal, optional_internal


def calculate_modularity_score(repo_root: Path, target_crate: str | None = None) -> List[Dict[str, Any]]:
    caps_file = repo_root / "capabilities.toml"
    with open(caps_file, "rb") as f:
        caps = tomllib.load(f)

    crates_spec: Dict[str, dict] = caps.get("crates", {})
    _, bridge_summary = run_gate_bridge_null(repo_root)

    results = []

    for cname, cspec in sorted(crates_spec.items()):
        if target_crate and cname != target_crate:
            continue

        level = cspec.get("level", 0)
        cpath = repo_root / cspec.get("path", f"crates/{cname}")
        manifest_path = cpath / "Cargo.toml"

        normal_deps, optional_deps = parse_manifest_deps(manifest_path)
        target_core = TARGET_CORE_DEPS.get(cname, set())

        score = 100
        deductions = []

        # 1. Kern-Abhängigkeiten über dem Zielzustand
        excess_deps = normal_deps - target_core
        if excess_deps:
            excess_penalty = min(35, len(excess_deps) * 12)
            score -= excess_penalty
            deductions.append(f"Excess core deps {sorted(excess_deps)}: -{excess_penalty}")

        # 2. Ungegatete Brückenreferenzen
        bridge_refs = bridge_summary.get(cname, 0)
        if bridge_refs > 0:
            bridge_penalty = min(40, int(round(15.0 * math.log10(1.0 + bridge_refs))))
            score -= bridge_penalty
            deductions.append(f"Bridge refs ({bridge_refs}): -{bridge_penalty}")

        # 3. Verfehlte Null-Prüfung G-L2 bei E0/E1
        if level in (0, 1) and bridge_refs > 0:
            score -= 15
            deductions.append("Failed G-L2 in E0/E1: -15")

        # 4. Fehlende compat/-Brücke bei types/ports Abhängigkeit
        has_types_ports = bool({"contextra-types", "contextra-ports"} & (normal_deps | optional_deps))
        if level in (0, 1) and has_types_ports:
            compat_dir = cpath / "src" / "compat"
            if not compat_dir.is_dir():
                score -= 10
                deductions.append("Missing src/compat/: -10")
            # Prüfe workspace Feature in Cargo.toml
            if manifest_path.is_file():
                with open(manifest_path, "rb") as f:
                    cdata = tomllib.load(f)
                features = cdata.get("features", {})
                if "workspace" not in features and "types-compat" not in features:
                    score -= 10
                    deductions.append("Missing workspace/types-compat feature: -10")

        # 5. Eigener Fehlertyp bei E0/E1
        # Heuristik: Suche nach *Error in src/
        if level in (0, 1):
            has_error_type = False
            src_dir = cpath / "src"
            if src_dir.is_dir():
                for p in src_dir.glob("*.rs"):
                    try:
                        content = p.read_text(encoding="utf-8", errors="ignore")
                        if re.search(r'pub\s+enum\s+\w+Error\b', content) or "thiserror" in content:
                            has_error_type = True
                            break
                    except Exception:
                        pass
            if not has_error_type and cname not in ("contextra-env", "contextra-durable-fs", "contextra-wire", "contextra-privacy", "contextra-sandbox", "contextra-avv-generator"):
                score -= 5
                deductions.append("Missing local Error type: -5")

        score = max(0, min(100, score))
        results.append({
            "crate": cname,
            "level": f"E{level}",
            "score": score,
            "bridge_refs": bridge_refs,
            "normal_deps_count": len(normal_deps),
            "deductions": deductions,
        })

    return results


def main() -> int:
    parser = argparse.ArgumentParser(description="Modularitäts-Score M je Crate (SPEC §8.6)")
    parser.add_argument("--crate", "-c", help="Spezifisches Crate prüfen")
    parser.add_argument("--root", default=".", help="Workspace-Root")
    parser.add_argument("--json", action="store_true", help="JSON-Ausgabe")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    results = calculate_modularity_score(root, args.crate)

    if args.json:
        print(json.dumps(results, indent=2))
        return 0

    print("=== MODULARITÄTS-SCORE M (SPEC §8.6) ===")
    print(f"{'Crate':<30} | {'Ebene':<5} | {'Score M':<7} | {'Bridge Refs':<11} | Abzüge")
    print("-" * 80)
    for r in results:
        ded_str = ", ".join(r["deductions"]) if r["deductions"] else "–"
        print(f"{r['crate']:<30} | {r['level']:<5} | {r['score']:<7} | {r['bridge_refs']:<11} | {ded_str}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
