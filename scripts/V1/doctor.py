"""`ctxrs doctor` – Workspace-Diagnose ohne Cargo (statisch, < 10 s).

Findet exakt die Fehlerklasse, die `cargo metadata` / `cargo check` blockiert, plus Folgeprobleme:

  D01 K  Workspace-Mitglied ohne Verzeichnis/Manifest
  D02 H  default-members: fehlt oder kein Mitglied
  D03 K  [workspace.dependencies]-Pfad existiert nicht
  D04 K/H/M  Mitglied hängt (normal/build | optional | dev) von fehlendem Crate
  D05 K  `workspace = true` ohne Eintrag in [workspace.dependencies]
  D06 H  [features] verweist auf unbekannte Abhängigkeit/Feature
  D07 K  xtask-Verweise (Alias, justfile, Workflows, Cargo.lock) ohne xtask-Verzeichnis
  D08 H/M  Abhängigkeitszyklus (normal/build bzw. nur dev)
  D09 M  Crate-Verzeichnis außerhalb von `members`
  D10 K/H  Rust-Quelltext nutzt Crate, das in der eigenen Cargo.toml nicht deklariert ist
  D11 H  `cfg(feature = "x")` im Quelltext, Feature x existiert nicht (unexpected_cfgs=deny bricht den Build)
  D12 L  Deklarierte interne Abhängigkeit wird im Quelltext nie verwendet (Näherung)
"""
from __future__ import annotations

import json
import re
from pathlib import Path

from .lib import (SEV_ORDER, Finding, Workspace, crate_idents_used, hit_class, mask_rust, pkg_ident,
                  read_text, render_findings, rust_files, tarjan, test_spans, md_table, LineIndex)

_CFG_FEATURE = re.compile(r'feature\s*=\s*"([^"]+)"')


def _xtask_exists(root: Path) -> bool:
    return (root / "xtask" / "Cargo.toml").is_file()


def xtask_refs(root: Path) -> list[Finding]:
    """D07 – alle Verweise auf das nicht (mehr) vorhandene xtask-Paket."""
    if _xtask_exists(root):
        return []
    out: list[Finding] = []
    pat = re.compile(r"(?<![\w-])xtask(?![\w])")
    for rel in ("justfile", ".cargo/config.toml", "Makefile"):
        if (root / rel).is_file():
            for i, ln in enumerate(read_text(root, rel).splitlines(), 1):
                if pat.search(ln) and not ln.lstrip().startswith("#"):
                    out.append(Finding("D07", "K", "Verweis auf fehlendes xtask", f"{rel}:{i}", ln.strip()[:110],
                                       "`ctxrs ci-purge` (Workflows/justfile/Alias) bzw. Ersatz über `ctxrs archgate`"))
    wf = root / ".github" / "workflows"
    if wf.is_dir():
        for f in sorted(wf.glob("*.y*ml")):
            rel = f.relative_to(root).as_posix()
            for i, ln in enumerate(f.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
                if pat.search(ln) and not ln.lstrip().startswith("#"):
                    out.append(Finding("D07", "K", "Verweis auf fehlendes xtask", f"{rel}:{i}", ln.strip()[:110],
                                       "`ctxrs ci-purge` (Workflows/justfile/Alias) bzw. Ersatz über `ctxrs archgate`"))
    lock = root / "Cargo.lock"
    if lock.is_file() and re.search(r'^name = "xtask"', lock.read_text(encoding="utf-8", errors="replace"), re.M):
        out.append(Finding("D07", "M", "Cargo.lock enthält xtask", "Cargo.lock", "`--locked` scheitert",
                           "Nach `prune --apply` einmal `cargo metadata` (ohne --locked) laufen lassen und Lock committen"))
    return out


def check(ws: Workspace, with_sources: bool = True) -> list[Finding]:
    root = ws.root
    fs: list[Finding] = []

    # D01 / D02 / D03 ----------------------------------------------------------------
    for m in ws.missing_members:
        fs.append(Finding("D01", "K", "Workspace-Mitglied ohne Verzeichnis", m, "",
                          f"`ctxrs prune --missing` entfernt Mitglied + Verweise"))
    mem = set(ws.members)
    for m in ws.default_members:
        if m not in mem:
            fs.append(Finding("D02", "H", "default-members-Eintrag ist kein Mitglied", m))
        elif m in ws.missing_members:
            fs.append(Finding("D02", "H", "default-members-Eintrag ohne Verzeichnis", m, "", "`ctxrs prune --missing`"))
    for k in ws.ws_deps:
        p = ws.ws_dep_path(k)
        if p and not (root / p / "Cargo.toml").is_file():
            fs.append(Finding("D03", "K", "[workspace.dependencies]-Pfad fehlt", f"Cargo.toml: {k}", p,
                              "`ctxrs prune --missing`"))

    # D04 / D05 / D06 ----------------------------------------------------------------
    for c in ws.crates.values():
        for d in c.deps:
            if d.workspace and d.key not in ws.ws_deps:
                fs.append(Finding("D05", "K", "`workspace = true` ohne [workspace.dependencies]-Eintrag",
                                  f"{c.manifest}: {d.key}"))
                continue
            if ws.dep_is_dangling(d):
                sev = "M" if d.kind == "dev" else ("H" if d.optional else "K")
                kind = d.kind + (", optional" if d.optional else "") + (f", target {d.target}" if d.target else "")
                fs.append(Finding("D04", sev, "Abhängigkeit auf fehlendes Crate", f"{c.manifest}: {d.key}",
                                  f"{kind} -> {d.pkg}", "`ctxrs prune --missing`"))
        feats = c.features
        optional_deps = {d.key for d in c.deps if d.optional}
        all_dep_keys = {d.key for d in c.deps}
        for fname, items in feats.items():
            for it in items:
                if it.startswith("dep:"):
                    ok = it[4:] in optional_deps
                elif "/" in it:
                    left = it.split("/", 1)[0].rstrip("?")
                    ok = left in all_dep_keys
                else:
                    ok = it in feats or it in optional_deps
                if not ok:
                    fs.append(Finding("D06", "H", "Feature verweist auf Unbekanntes", f"{c.manifest}: {fname}",
                                      f'"{it}"'))

    # D08 Zyklen ---------------------------------------------------------------------
    for comp in tarjan(ws.graph(("normal", "build"))):
        fs.append(Finding("D08", "H", "Abhängigkeitszyklus (normal/build)", " -> ".join(comp)))
    full = tarjan(ws.graph(("normal", "build", "dev")))
    base = {tuple(c) for c in tarjan(ws.graph(("normal", "build")))}
    for comp in full:
        if tuple(comp) not in base:
            fs.append(Finding("D08", "M", "Zyklus nur über dev-dependencies", " -> ".join(comp)))

    # D09 Verwaiste Crates -----------------------------------------------------------
    excluded = set(ws.exclude)
    for top in ("crates", "benchmarks", "xtask", "tools"):
        base_dir = root / top
        if not base_dir.is_dir():
            continue
        for mf in sorted(base_dir.glob("*/Cargo.toml")):
            rel = mf.parent.relative_to(root).as_posix()
            if rel not in mem and rel not in excluded:
                fs.append(Finding("D09", "M", "Crate-Verzeichnis nicht in members", rel))

    fs += xtask_refs(root)

    # D10 / D11 / D12 – Quelltext ------------------------------------------------------
    if with_sources:
        known_idents = {pkg_ident(n) for n in ws.crates} | {pkg_ident(k) for k in ws.ws_deps}
        for c in ws.crates.values():
            declared = {d.ident for d in c.deps} | {pkg_ident(c.name)}
            used: set[str] = set()
            feat_names = set(c.features) | {d.key for d in c.deps if d.optional}
            reported: set[tuple[str, str]] = set()
            for rel in rust_files(root, c.dir):
                src = read_text(root, rel)
                masked = mask_rust(src)
                li = None
                spans = test_spans(masked)
                for ident, off in crate_idents_used(masked, known_idents):
                    used.add(ident)
                    if ident not in declared and (rel, ident) not in reported:
                        reported.add((rel, ident))
                        li = li or LineIndex(src)
                        cls = hit_class(rel, c.dir, off, spans)
                        fs.append(Finding("D10", "K" if cls == "prod" else "H", "Quelltext nutzt nicht deklariertes Crate",
                                          f"{rel}:{li.line(off)}", f"{ident} ({cls})",
                                          "Abhängigkeit deklarieren oder Verwendung/Item entfernen (`ctxrs prune` listet Stellen)"))
                if "feature" in src:
                    for m in _CFG_FEATURE.finditer(masked_keep_strings(src)):
                        if m.group(1) not in feat_names:
                            li = li or LineIndex(src)
                            fs.append(Finding("D11", "H", "cfg(feature) ohne deklariertes Feature",
                                              f"{rel}:{li.line(m.start())}", m.group(1),
                                              "Feature (leer) deklarieren oder cfg-Code entfernen; unexpected_cfgs=deny"))
            for d in c.deps:
                if d.kind != "build" and d.pkg in ws.crates and d.ident not in used and not d.optional:
                    fs.append(Finding("D12", "L", "Interne Abhängigkeit ohne erkennbare Verwendung", f"{c.manifest}: {d.key}",
                                      d.kind, "mit `cargo machete` gegenprüfen"))
    fs.sort(key=lambda f: (SEV_ORDER[f.sev], f.id, f.where))
    return fs


def masked_keep_strings(src: str) -> str:
    """Kommentare maskieren, Strings behalten (für cfg(feature = "x"))."""
    return re.sub(r"//[^\n]*|/\*.*?\*/", lambda m: re.sub(r"[^\n]", " ", m.group(0)), src, flags=re.S)


def summarize(fs: list[Finding]) -> dict:
    by: dict[str, int] = {}
    for f in fs:
        by[f.id] = by.get(f.id, 0) + 1
    return {"total": len(fs), "blocking": sum(1 for f in fs if f.sev in ("K",)), "by_id": by,
            "by_sev": {s: sum(1 for f in fs if f.sev == s) for s in SEV_ORDER}}


def run(ws: Workspace, args) -> int:
    fs = check(ws, with_sources=not args.no_src)
    if args.json:
        print(json.dumps({"summary": summarize(fs), "findings": [f.__dict__ for f in fs]}, ensure_ascii=False, indent=1))
    else:
        s = summarize(fs)
        print(f"# ctxrs doctor – {ws.root.name}\n")
        print(f"Mitglieder: {len(ws.members)} (fehlend: {len(ws.missing_members)}) · geladene Crates: {len(ws.crates)} · "
              f"Befunde: {s['total']} (K={s['by_sev']['K']}, H={s['by_sev']['H']}, M={s['by_sev']['M']}, L={s['by_sev']['L']})\n")
        print(render_findings([f for f in fs if f.sev in args.show], 10_000 if args.full else 25))
        ok = s["by_sev"]["K"] == 0
        print("**Ergebnis:** " + ("Workspace ist ladbar; keine kritischen Befunde." if ok else
              "Build blockiert (mind. ein K-Befund). Reihenfolge: `prune --missing` → `ci-purge` → D10/D11 abarbeiten."))
    return 2 if any(f.sev == "K" for f in fs) else 0
