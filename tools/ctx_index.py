#!/usr/bin/env python3
"""Erzeugt Crate-Übersicht und prüft Konsistenz zwischen Quellen.

  python3 tools/ctx_index.py crates   -> schreibt docs/agent/CRATES.md
  python3 tools/ctx_index.py gates    -> Konsistenzprüfung (Exit 1 bei Abweichung)

Quellen: Cargo.toml (Workspace-Mitglieder), capabilities.toml (Gruppe/Ebene/Tier),
tests/layering.rs (Ring-Matrix, grob per Regex). Nur Standardbibliothek (tomllib).
"""
import re, sys, tomllib, pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent

def load():
    ws = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]
    members = [pathlib.Path(m).name for m in ws["members"]]
    default = {pathlib.Path(m).name for m in ws["default-members"]}
    cap = tomllib.loads((ROOT / "capabilities.toml").read_text())["crates"]
    lay = (ROOT / "tests/layering.rs").read_text()
    layer_names = set(re.findall(r'"(contextra[a-z0-9-]*)"', lay))
    return members, default, cap, layer_names

def crates():
    members, default, cap, _ = load()
    out = ["# Crate-Übersicht (generiert: python3 tools/ctx_index.py crates)", "",
           "| Crate | Gruppe | Ebene | Tier | Default-Member | in capabilities.toml |",
           "|---|---|---|---|---|---|"]
    for m in sorted(members):
        c = cap.get(m, {})
        out.append(f"| {m} | {c.get('group','-')} | {c.get('level','-')} | {c.get('tier','-')} "
                   f"| {'ja' if m in default else 'nein'} | {'ja' if m in cap else 'NEIN'} |")
    (ROOT / "docs/agent/CRATES.md").write_text("\n".join(out) + "\n")
    print(f"geschrieben: docs/agent/CRATES.md ({len(members)} Workspace-Mitglieder)")

def gates():
    members, default, cap, layer_names = load()
    mem = set(members)
    cap_set = set(cap)
    not_in_cap = sorted(mem - cap_set)
    cap_not_member = sorted(cap_set - mem)
    not_in_layer = sorted(mem - layer_names)
    print(f"Workspace-Mitglieder:            {len(mem)}")
    print(f"in capabilities.toml:            {len(mem & cap_set)}")
    print(f"Mitglied, aber nicht in capab.:  {len(not_in_cap)} {not_in_cap}")
    print(f"in capab., aber kein Mitglied:   {len(cap_not_member)} {cap_not_member}")
    print(f"Mitglied, nicht in layering.rs:  {len(not_in_layer)}")
    return 1 if (not_in_cap or cap_not_member) else 0

if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "gates"
    sys.exit({"crates": lambda: (crates(), 0)[1], "gates": gates}[cmd]())
