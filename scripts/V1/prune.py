"""`ctxrs prune` – Crates konsistent aus dem Workspace entfernen (WP N0).

Ändert (kommentarerhaltend, mit tomllib-Nachvalidierung):
  * Cargo.toml: members, default-members, [workspace.dependencies] (inkl. Alias-Schlüssel), Profil-Overrides
  * alle Mitglieds-Manifeste: Abhängigkeitseinträge in allen Tabellen (normal/dev/build/target.*)
  * [features]: Elemente `dep:X`, `X/f`, `X?/f`, Verweise auf entfernte Features; Workspace-weiter Fixpunkt
    (auch `anderes-crate/feature`)
  * capabilities.toml: [crates.X] und `may_depend_on`
Berichtet (ändert NICHT): Rust-Quelltextstellen, die entfernte Crates/Features noch benutzen, und Textverweise.
Grund für die Trennung: Quelltextchirurgie (Match-Arme, Struct-Felder, cfg-Blöcke) lässt sich ohne Compiler
nicht sicher automatisieren; die Liste ist die Arbeitsgrundlage für die verbleibenden manuellen PRs.
"""
from __future__ import annotations

import os
import re
from collections import defaultdict

from . import tomledit as te
from .doctor import check as doctor_check
from .lib import (ChangeSet, Crate, ToolError, Workspace, crate_idents_used, hit_class, mask_rust, md_table,
                  pkg_ident, read_text, rust_files, test_spans, walk_files, LineIndex)
from .doctor import masked_keep_strings

DEP_TABLE_NAMES = ("dependencies", "dev-dependencies", "build-dependencies")


def resolve_targets(ws: Workspace, names: list[str], missing: bool) -> tuple[set[str], set[str]]:
    """-> (Paketnamen, Verzeichnisse)"""
    pkgs: set[str] = set(names)
    dirs: set[str] = set()
    if missing:
        pkgs |= ws.dangling_pkgs()
        for m in ws.missing_members:
            dirs.add(m)
            pkgs.add(os.path.basename(m))
            for k in ws.ws_deps:
                if ws.ws_dep_path(k) == m:
                    pkgs.add(ws.ws_dep_pkg(k))
    for n in list(pkgs):
        c = ws.crates.get(n)
        if c:
            dirs.add(c.dir)
        for k in ws.ws_deps:
            if ws.ws_dep_pkg(k) == n and (p := ws.ws_dep_path(k)):
                dirs.add(p)
    unknown = [n for n in names if n not in ws.crates and not any(ws.ws_dep_pkg(k) == n for k in ws.ws_deps)]
    if unknown:
        raise ToolError(f"Unbekannte Crates: {', '.join(unknown)}")
    return pkgs, dirs


def _cfg_ref_count(ws: Workspace, c: Crate, feature: str, cache: dict) -> int:
    key = (c.name, feature)
    if key not in cache:
        pat = re.compile(r'feature\s*=\s*"%s"' % re.escape(feature))
        n = 0
        for rel in rust_files(ws.root, c.dir):
            n += len(pat.findall(masked_keep_strings(read_text(ws.root, rel))))
        cache[key] = n
    return cache[key]


def feature_fixpoint(ws: Workspace, targets: set[str], removed_keys: dict[str, set[str]], policy: str):
    """Workspace-weiter Fixpunkt über tote Features. -> (dropped, kept_empty) je Crate."""
    dropped: dict[str, set[str]] = defaultdict(set)
    kept: dict[str, set[str]] = defaultdict(set)
    cache: dict = {}
    live = {n: c for n, c in ws.crates.items() if n not in targets}

    def is_dead(c: Crate, item: str) -> bool:
        rk = removed_keys.get(c.name, set())
        if item.startswith("dep:"):
            return item[4:] in rk
        if "/" in item:
            left, right = item.split("/", 1)
            left = left.rstrip("?")
            if left in rk:
                return True
            for d in c.deps:
                if d.key == left and d.pkg in dropped and right in dropped[d.pkg]:
                    return True
            return False
        return item in rk and item not in c.features or item in dropped[c.name]

    changed = True
    while changed:
        changed = False
        for c in live.values():
            for f, items in c.features.items():
                if f in dropped[c.name] or f in kept[c.name]:
                    continue
                if items and all(is_dead(c, i) for i in items):
                    keep = policy == "keep-empty" or (policy == "auto" and _cfg_ref_count(ws, c, f, cache) > 0)
                    (kept if keep else dropped)[c.name].add(f)
                    changed = True
    return dropped, kept, is_dead


def plan(ws: Workspace, targets: set[str], target_dirs: set[str], policy: str, delete_dirs: bool):
    root = ws.root
    cs = ChangeSet(root)
    rep: dict = {"manifests": [], "features": [], "ws_keys": [], "dirs": sorted(target_dirs)}

    removed_ws_keys = {k for k in ws.ws_deps if ws.ws_dep_pkg(k) in targets}
    removed_keys: dict[str, set[str]] = {}
    for n, c in ws.crates.items():
        if n in targets:
            continue
        ks = {d.key for d in c.deps if d.pkg in targets}
        if ks:
            removed_keys[n] = ks
    dropped, kept, is_dead = feature_fixpoint(ws, targets, removed_keys, policy)

    # --- Root-Manifest ------------------------------------------------------------
    ed = te.Editor(ws.manifest_text)
    norm = lambda s: os.path.normpath(s).replace("\\", "/")
    for key in ("members", "default-members"):
        for s in ed.kvs(("workspace",)):
            if s.key == (key,):
                ed.remove_array_items(s, lambda v: norm(v) in target_dirs)
    for s in ed.kvs(("workspace", "dependencies")):
        if s.key[0] in removed_ws_keys:
            ed.delete_stmt(s)
            rep["ws_keys"].append(s.key[0])
    for k in removed_ws_keys:
        ed.delete_table(("workspace", "dependencies", k))
    for t in ed.table_names():
        if len(t) >= 3 and t[0] == "profile" and "package" in t and t[-1] in targets:
            ed.delete_table(t)
    # Root-Paket (falls vorhanden) behandeln wie Mitglied
    root_crate = ws.by_dir.get(".")
    if root_crate and root_crate.name not in targets:
        _edit_member(ed, root_crate, removed_keys.get(root_crate.name, set()), dropped, kept, is_dead, rep)
    new_root = ed.result()
    te.validate(new_root, "Cargo.toml")
    cs.write("Cargo.toml", new_root)

    # --- Mitglieder -----------------------------------------------------------------
    for n, c in sorted(ws.crates.items()):
        if n in targets or c.dir == ".":
            continue
        rk = removed_keys.get(n, set())
        if not rk and not dropped.get(n) and not kept.get(n) and not any(
                is_dead(c, i) for items in c.features.values() for i in items):
            continue
        text = read_text(root, c.manifest)
        ed = te.Editor(text)
        _edit_member(ed, c, rk, dropped, kept, is_dead, rep)
        new = ed.result()
        te.validate(new, c.manifest)
        cs.write(c.manifest, new)

    # --- capabilities.toml -----------------------------------------------------------
    if (root / "capabilities.toml").is_file():
        ed = te.Editor(read_text(root, "capabilities.toml"))
        for t in ed.table_names():
            if len(t) == 2 and t[0] == "crates" and t[1] in targets:
                ed.delete_table(t)
        for s in ed.stmts:
            if s.kind == "kv" and s.key == ("may_depend_on",):
                ed.remove_array_items(s, lambda v: v in targets)
        new = ed.result()
        te.validate(new, "capabilities.toml")
        cs.write("capabilities.toml", new)

    if delete_dirs:
        for d in sorted(target_dirs):
            if (root / d).is_dir():
                cs.delete_dir(d)
    rep["dropped"], rep["kept"] = dropped, kept
    return cs, rep, removed_keys, removed_ws_keys


def _edit_member(ed: te.Editor, c: Crate, rk: set[str], dropped, kept, is_dead, rep) -> None:
    removed_here = []
    for s in ed.stmts:
        if s.kind == "kv" and s.key and s.table and s.table[-1] in DEP_TABLE_NAMES and s.key[0] in rk:
            ed.delete_stmt(s)
            removed_here.append(s.key[0])
    for t in ed.table_names():
        if len(t) >= 2 and t[-2] in DEP_TABLE_NAMES and t[-1] in rk:
            ed.delete_table(t)
            removed_here.append(t[-1])
    for s in ed.kvs(("features",)):
        f = s.key[0]
        if f in dropped.get(c.name, set()):
            ed.delete_stmt(s)
            rep["features"].append((c.name, f, "entfernt"))
        elif f in kept.get(c.name, set()):
            ed.replace_value(s, "[]")
            rep["features"].append((c.name, f, "leer behalten (cfg-Verweise im Quelltext)"))
        else:
            gone = ed.remove_array_items(s, lambda v, c=c: is_dead(c, v))
            if gone:
                rep["features"].append((c.name, f, "Elemente entfernt: " + ", ".join(gone)))
    if removed_here:
        rep["manifests"].append((c.manifest, sorted(set(removed_here))))


def source_refs(ws: Workspace, targets: set[str], removed_keys, removed_ws_keys, dropped) -> list[dict]:
    """Quelltextstellen, die nach dem Prune nicht mehr kompilieren."""
    idents = {pkg_ident(t) for t in targets} | {pkg_ident(k) for k in removed_ws_keys}
    out = []
    for n, c in sorted(ws.crates.items()):
        if n in targets:
            continue
        dead_feats = dropped.get(n, set())
        for rel in rust_files(ws.root, c.dir):
            src = read_text(ws.root, rel)
            if not any(i in src for i in idents) and not dead_feats:
                continue
            masked = mask_rust(src)
            li = LineIndex(src)
            spans = test_spans(masked)
            for ident, off in crate_idents_used(masked, idents):
                out.append({"crate": n, "file": rel, "line": li.line(off), "what": ident,
                            "cls": hit_class(rel, c.dir, off, spans)})
            for f in dead_feats:
                for m in re.finditer(r'feature\s*=\s*"%s"' % re.escape(f), masked_keep_strings(src)):
                    out.append({"crate": n, "file": rel, "line": li.line(m.start()), "what": f"feature:{f}",
                                "cls": hit_class(rel, c.dir, m.start(), spans)})
    return out


def text_mentions(ws: Workspace, targets: set[str]) -> dict[str, int]:
    pat = re.compile(r"(?<![\w-])(%s)(?![\w-])" % "|".join(re.escape(t) for t in sorted(targets, key=len, reverse=True)))
    hits: dict[str, int] = {}
    for rel in walk_files(ws.root, {".md", ".yml", ".yaml", ".toml", ".json", ".py", ".nix", ".txt"}):
        if rel == "Cargo.toml" or rel.endswith("/Cargo.toml"):
            continue
        n = len(pat.findall(read_text(ws.root, rel)))
        if n:
            hits[rel] = n
    return hits


def render(ws, targets, cs, rep, refs, mentions, full: bool) -> str:
    out = [f"# ctxrs prune – Plan\n", f"Zu entfernen ({len(targets)}): " + ", ".join(f"`{t}`" for t in sorted(targets)) + "\n"]
    out.append("## Änderungen\n")
    out.append(cs.summary() + "\n")
    out.append(f"- Workspace-Abhängigkeiten entfernt: {', '.join(f'`{k}`' for k in sorted(rep['ws_keys'])) or '–'}\n")
    if rep["manifests"]:
        out.append(md_table(["Manifest", "entfernte Abhängigkeiten"], [[m, ", ".join(ks)] for m, ks in rep["manifests"]]))
    if rep["features"]:
        out.append("### Features\n")
        out.append(md_table(["Crate", "Feature", "Aktion"], [list(x) for x in rep["features"]]))
    out.append("## Verbleibende Quelltext-Arbeit (nicht automatisiert)\n")
    if not refs:
        out.append("Keine Quelltextverweise auf entfernte Crates/Features gefunden.\n")
    else:
        by: dict[str, list[dict]] = defaultdict(list)
        for r in refs:
            by[r["crate"]].append(r)
        rows = []
        for cr, items in sorted(by.items()):
            prod = sum(1 for i in items if i["cls"] == "prod")
            rows.append([cr, len({i['file'] for i in items}), prod, len(items) - prod])
        out.append(md_table(["Crate", "Dateien", "Treffer prod", "Treffer test/bench/example"], rows))
        lim = 10_000 if full else 40
        out.append("Treffer (Datei:Zeile, Klasse):\n")
        shown = 0
        for r in sorted(refs, key=lambda r: (r["cls"] != "prod", r["file"], r["line"])):
            if shown >= lim:
                out.append(f"- … {len(refs) - shown} weitere (`--full`)")
                break
            out.append(f"- `{r['file']}:{r['line']}` {r['what']} ({r['cls']})")
            shown += 1
        out.append("")
    if mentions:
        out.append("## Textverweise (Doku/CI/Governance, nur Information)\n")
        top = sorted(mentions.items(), key=lambda kv: -kv[1])[: (10_000 if full else 15)]
        out.append(md_table(["Datei", "Treffer"], [[f"`{f}`", n] for f, n in top]))
    return "\n".join(out)


def run(ws: Workspace, args) -> int:
    if not args.crates and not args.missing:
        raise ToolError("Crates angeben oder --missing verwenden.")
    targets, dirs = resolve_targets(ws, args.crates, args.missing)
    if not targets:
        print("Nichts zu entfernen.")
        return 0
    cs, rep, removed_keys, removed_ws = plan(ws, targets, dirs, args.features, args.delete_dirs)
    refs = source_refs(ws, targets, removed_keys, removed_ws, rep["dropped"])
    mentions = text_mentions(ws, targets)
    print(render(ws, targets, cs, rep, refs, mentions, args.full))
    if not args.apply:
        if not args.no_diff:
            print("## Diff (Dry-Run)\n\n```diff\n" + cs.diff() + "```\n")
        print("Dry-Run. Mit `--apply` schreiben.")
        return 0
    cs.apply()
    ws2 = Workspace(ws.root)
    left = [f for f in doctor_check(ws2, with_sources=False) if f.sev == "K" and f.id not in ("D07",)]
    print(f"\nAngewendet. Verbleibende K-Befunde (ohne Quelltext/xtask): {len(left)}")
    for f in left[:20]:
        print(f"- {f.id} `{f.where}` {f.detail}")
    print("\nNächste Schritte: `ctxrs ci-purge --apply`; Quelltextliste oben abarbeiten; "
          "`cargo metadata` (ohne --locked) zur Lock-Aktualisierung; `ctxrs doctor`.")
    return 0
