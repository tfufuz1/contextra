#!/usr/bin/env python3
"""
port_contract_auditor.py - Audit von Port-Verträgen und Re-Exporten (M3 / M7).

Prüfungen:
- Regel M3: Öffentliche Signaturen nutzen nur Typen aus types, ports oder dem Crate selbst.
  Kein `pub use contextra_*` fremder Implementierungstypen!
- Regel M7: Datenhalter implementieren nur `Purger` und `LayerScanner`.
- Schnittstellen-Analyse: Welche Ports (contextra_ports::*) werden tatsächlich konsumiert?
"""

import os
import sys
import re
import json
import argparse
from pathlib import Path

DATA_HOLDER_CRATES = {
    "contextra-store",
    "contextra-vector",
    "contextra-text",
    "contextra-graph",
    "contextra-kvcache",
    "contextra-checkpoint",
}

FORBIDDEN_DOMAIN_IN_HOLDERS = [
    "DSGVO",
    "Saga",
    "DeletionProof",
    "ErasureReceipt",
    "RetentionAttestation",
]

PORT_TRAITS = [
    "Vfs", "Clock", "Rng", "IdGen", "MetricsSink",
    "StorageEngine", "TextIndex", "VectorIndex",
    "Purger", "LayerScanner", "DynSafePurger", "DynSafeLayerScanner"
]

PUB_USE_OTHER_CRATE = re.compile(r'^\s*pub\s+use\s+contextra_([a-zA-Z0-9_]+)::')

def scan_crate_contracts(crate_dir: Path):
    crate_name = crate_dir.name
    is_holder = crate_name in DATA_HOLDER_CRATES

    m3_leaks = []
    implemented_ports = set()
    holder_domain_violations = []

    src_dir = crate_dir / "src"
    if not src_dir.exists():
        return None

    # Check lib.rs / main.rs for M3 leaks
    root_files = [src_dir / "lib.rs", src_dir / "main.rs"]
    for rf in root_files:
        if not rf.exists():
            continue
        try:
            with open(rf, "r", encoding="utf-8", errors="ignore") as f:
                lines = f.readlines()
        except Exception:
            continue

        for idx, line in enumerate(lines):
            line_str = line.strip()
            if line_str.startswith("//"):
                continue
            m = PUB_USE_OTHER_CRATE.search(line_str)
            if m:
                target_crate = m.group(1)
                # Allowing re-export of types or ports if crate is contextra facade
                if crate_name != "contextra" and target_crate not in ("types", "ports"):
                    m3_leaks.append({
                        "file": str(rf),
                        "line": idx + 1,
                        "reexported_crate": f"contextra_{target_crate}",
                        "code": line_str
                    })

    # Scan all files for Port implementations & domain leaks
    for rs_file in src_dir.rglob("*.rs"):
        try:
            with open(rs_file, "r", encoding="utf-8", errors="ignore") as f:
                content = f.read()
                lines = content.splitlines()
        except Exception:
            continue

        for pt in PORT_TRAITS:
            if re.search(rf'\bimpl\b[^{{;]*\b{pt}\b[^{{;]*\s+for\b', content):
                implemented_ports.add(pt)

        if is_holder:
            for idx, line in enumerate(lines):
                if line.strip().startswith("//"):
                    continue
                for term in FORBIDDEN_DOMAIN_IN_HOLDERS:
                    if re.search(rf'\b{term}\b', line):
                        holder_domain_violations.append({
                            "file": str(rs_file),
                            "line": idx + 1,
                            "term": term,
                            "code": line.strip()[:100]
                        })

    return {
        "crate": crate_name,
        "is_data_holder": is_holder,
        "m3_leaks_count": len(m3_leaks),
        "m3_leaks": m3_leaks,
        "implemented_ports": sorted(list(implemented_ports)),
        "holder_domain_violations_count": len(holder_domain_violations),
        "holder_domain_violations": holder_domain_violations,
    }

def main():
    parser = argparse.ArgumentParser(description="Port Contract & API Leak Auditor (M3/M7)")
    parser.add_argument("--workspace", default=".", help="Workspace root")
    parser.add_argument("--json", action="store_true", help="Output JSON directly to stdout")
    parser.add_argument("--output-dir", default="docs/diagnostics", help="Output directory for reports")
    args = parser.parse_args()

    workspace_root = Path(args.workspace).resolve()
    crates_dir = workspace_root / "crates"
    output_dir = workspace_root / args.output_dir
    output_dir.mkdir(parents=True, exist_ok=True)

    crates = sorted([p for p in crates_dir.iterdir() if p.is_dir() and (p / "Cargo.toml").exists()])
    results = []

    for c in crates:
        res = scan_crate_contracts(c)
        if res:
            results.append(res)

    md_path = output_dir / "PORT_CONTRACT_AUDIT.md"
    json_path = output_dir / "port_contract_audit.json"

    with open(json_path, "w", encoding="utf-8") as jf:
        json.dump(results, jf, indent=2)

    with open(md_path, "w", encoding="utf-8") as mf:
        mf.write("# Port Contract & Facade Leak Audit (M3 / M7)\n\n")
        mf.write(f"> Generiert durch `tools/port_contract_auditor.py`.\n")
        mf.write(f"> M3: Kein Fremd-Reexport (`pub use contextra_*`). M7: Reine mechanische Bereinigung in Datenhaltern.\n\n")

        mf.write("## Crate Übersicht\n\n")
        mf.write("| Crate | Rolle | M3 API-Leaks | Implementierte Ports | M7 Domain-Leaks (Holders) |\n")
        mf.write("|---|---|---|---|---|\n")
        for r in results:
            role = "Datenhalter" if r["is_data_holder"] else "Baustein"
            ports_str = ", ".join(f"`{p}`" for p in r["implemented_ports"]) if r["implemented_ports"] else "-"
            mf.write(f"| `{r['crate']}` | {role} | {r['m3_leaks_count']} | {ports_str} | {r['holder_domain_violations_count']} |\n")

        total_leaks = sum(r["m3_leaks_count"] for r in results)
        total_m7 = sum(r["holder_domain_violations_count"] for r in results)

        if total_leaks > 0 or total_m7 > 0:
            mf.write("\n## Befunde & Verstöße\n\n")
            for r in results:
                if r["m3_leaks"]:
                    mf.write(f"### M3 Leaks in `{r['crate']}`\n\n")
                    for lk in r["m3_leaks"]:
                        mf.write(f"- `{lk['file']}:{lk['line']}`: `{lk['code']}`\n")
                if r["holder_domain_violations"]:
                    mf.write(f"### M7 Erasure/DSGVO Domänen-Leaks in Datenhalter `{r['crate']}`\n\n")
                    for dm in r["holder_domain_violations"][:15]:
                        mf.write(f"- `{dm['file']}:{dm['line']}` [{dm['term']}]: `{dm['code']}`\n")
                    if len(r["holder_domain_violations"]) > 15:
                        mf.write(f"- ... und {len(r['holder_domain_violations']) - 15} weitere Stellen.\n")

    if args.json:
        print(json.dumps(results, indent=2))
    else:
        print(f"Port Contract Audit abgeschlossen:")
        print(f" - Geprüfte Crates: {len(results)}")
        print(f" - M3 API-Leaks: {total_leaks}")
        print(f" - M7 Domain-Leaks: {total_m7}")
        print(f" - Markdown: {md_path}")
        print(f" - JSON: {json_path}")

if __name__ == "__main__":
    main()
