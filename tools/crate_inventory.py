#!/usr/bin/env python3
"""
crate_inventory.py — Abhängigkeits- und Funktionsinventar pro Crate.

Ersetzt für die Entkopplungsarbeit die Einzel-Auswertungen durch ein einziges,
abhängigkeitsfreies Werkzeug (nur Python-Standardbibliothek, Python >= 3.11).

Liest für jedes Workspace-Mitglied (und jedes crates/*-Verzeichnis):
  * [dependencies], [dev-dependencies], [build-dependencies] inkl. target-Tabellen
  * Auflösung von `workspace = true` gegen [workspace.dependencies]
  * optional-Flag, Feature-Liste, Feature-Verweise auf interne Crates
  * package.metadata.contextra.ring
  * öffentliche API (pub fn/struct/enum/trait/type/const/static/mod/use) aus src/
  * unsafe-Vorkommen und #![forbid/deny(unsafe_code)]-Status
  * LOC der src/-Dateien

Abgeleitet:
  * Reverse-Abhängigkeiten (wer hängt von mir ab)
  * Hub-Kopplung: Kanten in Hub-Crates (Standard: contextra, contextra-engine,
    contextra-ports, contextra-types, contextra-db)
  * Zyklen im internen Normal-Abhängigkeitsgraphen

Unterbefehle:
  overview   Tabelle aller Crates (Standard)
  crate      Detailansicht eines Crates
  matrix     Abhängigkeitsmatrix (intern)
  hubs       Alle Kanten in/aus Hub-Crates
  cycles     Zyklen im internen Graphen
  json       Vollständiges Inventar als JSON
  agents     Schreibt den generierten Abschnitt in crates/<x>/AGENTS.md
  check      Exit 1 bei Zyklen oder Verstoß gegen --max-hub-edges

Beispiele:
  python3 tools/crate_inventory.py overview
  python3 tools/crate_inventory.py crate contextra-ports
  python3 tools/crate_inventory.py hubs --json
  python3 tools/crate_inventory.py agents --dry-run
  python3 tools/crate_inventory.py check --max-hub-edges 0
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import tomllib
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Set, Tuple

GEN_BEGIN = "<!-- BEGIN GENERATED: crate_inventory.py -->"
GEN_END = "<!-- END GENERATED: crate_inventory.py -->"

# Crates, an denen die Entkopplung Kanten zählt (Bausteinsammlung, kein Produkt).
DEFAULT_HUBS = ("contextra", "contextra-engine", "contextra-ports",
                "contextra-types", "contextra-db")

PUB_ITEM_RE = re.compile(
    r"^\s*pub\s+(?:async\s+)?(?:unsafe\s+)?(?:const\s+)?"
    r"(fn|struct|enum|trait|type|const|static|mod|use)\s+([A-Za-z_][A-Za-z0-9_]*)?",
    re.MULTILINE,
)
PUB_USE_RE = re.compile(r"^\s*pub\s+use\s+([^;]+);", re.MULTILINE)
UNSAFE_RE = re.compile(r"\bunsafe\s+(?:fn|impl|trait|\{)")
FORBID_RE = re.compile(r"#!\[(forbid|deny)\(unsafe_code\)\]")
COMMENT_LINE_RE = re.compile(r"^\s*//")


# --------------------------------------------------------------------------- #
# Datenmodell
# --------------------------------------------------------------------------- #

@dataclass
class Dep:
    name: str
    kind: str                 # normal | dev | build
    target: Optional[str]     # cfg-Ausdruck oder None
    optional: bool
    version: Optional[str]
    features: List[str]
    internal: bool
    via_workspace: bool


@dataclass
class Crate:
    name: str
    path: str                 # relativ zum Repo-Root
    version: str
    description: str
    ring: Optional[str]
    member: bool              # Workspace-Mitglied?
    deps: List[Dep] = field(default_factory=list)
    features: Dict[str, List[str]] = field(default_factory=dict)
    pub_items: Dict[str, List[str]] = field(default_factory=dict)
    pub_reexports: List[str] = field(default_factory=list)
    unsafe_hits: int = 0
    unsafe_policy: str = "unbekannt"   # forbid | deny | keiner
    loc_src: int = 0
    src_files: int = 0
    reverse: List[str] = field(default_factory=list)  # wer hängt von mir ab (normal)

    # Abgeleitet
    def deps_of(self, kind: str, internal_only: bool = False) -> List[Dep]:
        out = [d for d in self.deps if d.kind == kind]
        return [d for d in out if d.internal] if internal_only else out

    @property
    def internal_normal(self) -> Set[str]:
        return {d.name for d in self.deps if d.kind == "normal" and d.internal}

    @property
    def internal_dev(self) -> Set[str]:
        return {d.name for d in self.deps if d.kind in ("dev", "build") and d.internal}


# --------------------------------------------------------------------------- #
# Einlesen
# --------------------------------------------------------------------------- #

def repo_root(start: Optional[Path] = None) -> Path:
    p = (start or Path(__file__).resolve().parent).resolve()
    for cand in [p, *p.parents]:
        if (cand / "Cargo.toml").is_file() and (cand / "crates").is_dir():
            return cand
    raise SystemExit("Repo-Root nicht gefunden (Cargo.toml + crates/ erwartet).")


def _load_toml(path: Path) -> dict:
    with path.open("rb") as f:
        return tomllib.load(f)


def _workspace_deps(root_manifest: dict) -> Dict[str, dict]:
    return root_manifest.get("workspace", {}).get("dependencies", {}) or {}


def _parse_dep_table(name: str, spec, kind: str, target: Optional[str],
                     ws_deps: Dict[str, dict], internal_names: Set[str]) -> Dep:
    via_ws = False
    if isinstance(spec, str):
        version, optional, feats = spec, False, []
    else:
        spec = dict(spec)
        if spec.get("workspace") is True:
            via_ws = True
            base = ws_deps.get(name, {})
            base = {"version": base} if isinstance(base, str) else dict(base)
            merged = {**base, **{k: v for k, v in spec.items() if k != "workspace"}}
            spec = merged
        version = spec.get("version") or ("path" if "path" in spec else None)
        optional = bool(spec.get("optional", False))
        feats = list(spec.get("features", []) or [])
    return Dep(
        name=name, kind=kind, target=target, optional=optional,
        version=str(version) if version is not None else None,
        features=feats,
        internal=name in internal_names or name.startswith("contextra"),
        via_workspace=via_ws,
    )


def _collect_deps(manifest: dict, ws_deps: Dict[str, dict],
                  internal_names: Set[str]) -> List[Dep]:
    deps: List[Dep] = []
    sections = [("dependencies", "normal"), ("dev-dependencies", "dev"),
                ("build-dependencies", "build")]
    for sec, kind in sections:
        for name, spec in (manifest.get(sec) or {}).items():
            deps.append(_parse_dep_table(name, spec, kind, None, ws_deps, internal_names))
    for target, tdata in (manifest.get("target") or {}).items():
        for sec, kind in sections:
            for name, spec in (tdata.get(sec) or {}).items():
                deps.append(_parse_dep_table(name, spec, kind, target, ws_deps, internal_names))
    return deps


def _resolve_version(v, root_manifest: dict) -> str:
    if isinstance(v, dict) and v.get("workspace") is True:
        return str(root_manifest.get("workspace", {}).get("package", {}).get("version", "workspace"))
    return str(v) if v is not None else "–"


def _scan_src(crate_dir: Path) -> Tuple[Dict[str, List[str]], List[str], int, int, int, str]:
    pub: Dict[str, List[str]] = {}
    reexports: List[str] = []
    unsafe_hits = loc = files = 0
    policy = "keiner"
    src = crate_dir / "src"
    if not src.is_dir():
        return pub, reexports, unsafe_hits, loc, files, policy
    for rs in sorted(src.rglob("*.rs")):
        try:
            text = rs.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        files += 1
        lines = text.splitlines()
        loc += sum(1 for ln in lines if ln.strip() and not COMMENT_LINE_RE.match(ln))
        for m in PUB_ITEM_RE.finditer(text):
            kind, ident = m.group(1), m.group(2)
            if kind == "use":
                continue  # pub use wird separat gesammelt
            if ident:
                pub.setdefault(kind, []).append(ident)
        for m in PUB_USE_RE.finditer(text):
            reexports.append(" ".join(m.group(1).split()))
        code_lines = [ln for ln in lines if not COMMENT_LINE_RE.match(ln)]
        unsafe_hits += len(UNSAFE_RE.findall("\n".join(code_lines)))
        fm = FORBID_RE.search(text)
        if fm and rs.name in ("lib.rs", "main.rs"):
            policy = fm.group(1)
    for k in pub:
        pub[k] = sorted(set(pub[k]))
    return pub, sorted(set(reexports)), unsafe_hits, loc, files, policy


def load_inventory(root: Optional[Path] = None) -> Dict[str, Crate]:
    root = root or repo_root()
    root_manifest = _load_toml(root / "Cargo.toml")
    ws = root_manifest.get("workspace", {})
    members = {Path(m).as_posix() for m in ws.get("members", [])}
    ws_deps = _workspace_deps(root_manifest)

    manifests: Dict[str, Tuple[Path, dict]] = {}
    for mpath in sorted(root.glob("crates/*/Cargo.toml")) + sorted(
            root.glob("benchmarks/*/Cargo.toml")):
        manifests[mpath.parent.relative_to(root).as_posix()] = (mpath.parent, _load_toml(mpath))

    internal_names = {m["package"]["name"] for _, m in manifests.values()
                      if "package" in m}

    crates: Dict[str, Crate] = {}
    for rel, (cdir, man) in manifests.items():
        pkg = man.get("package", {})
        name = pkg.get("name", cdir.name)
        ring = ((pkg.get("metadata") or {}).get("contextra") or {}).get("ring")
        deps = _collect_deps(man, ws_deps, internal_names)
        pub, reex, unsafe_hits, loc, files, policy = _scan_src(cdir)
        feats = {k: list(v or []) for k, v in (man.get("features") or {}).items()}
        crates[name] = Crate(
            name=name,
            path=rel,
            version=_resolve_version(pkg.get("version"), root_manifest),
            description=str(pkg.get("description", "")).strip(),
            ring=str(ring) if ring is not None else None,
            member=rel in members,
            deps=deps,
            features=feats,
            pub_items=pub,
            pub_reexports=reex,
            unsafe_hits=unsafe_hits,
            unsafe_policy=policy,
            loc_src=loc,
            src_files=files,
        )

    # Reverse-Abhängigkeiten (nur normale Kanten, intern)
    for c in crates.values():
        for d in c.deps:
            if d.kind == "normal" and d.internal and d.name in crates:
                crates[d.name].reverse.append(c.name)
    for c in crates.values():
        c.reverse = sorted(set(c.reverse))
    return crates


# --------------------------------------------------------------------------- #
# Analysen
# --------------------------------------------------------------------------- #

def find_cycles(crates: Dict[str, Crate]) -> List[List[str]]:
    """Tarjan-SCC über den internen Normal-Graphen; gibt Zyklen (|SCC|>1) zurück."""
    graph = {n: sorted(c.internal_normal & crates.keys()) for n, c in crates.items()}
    index: Dict[str, int] = {}
    low: Dict[str, int] = {}
    on_stack: Set[str] = set()
    stack: List[str] = []
    sccs: List[List[str]] = []
    counter = [0]

    def strong(v: str) -> None:
        index[v] = low[v] = counter[0]
        counter[0] += 1
        stack.append(v)
        on_stack.add(v)
        for w in graph[v]:
            if w not in index:
                strong(w)
                low[v] = min(low[v], low[w])
            elif w in on_stack:
                low[v] = min(low[v], index[w])
        if low[v] == index[v]:
            comp = []
            while True:
                w = stack.pop()
                on_stack.discard(w)
                comp.append(w)
                if w == v:
                    break
            if len(comp) > 1 or v in graph[v]:
                sccs.append(sorted(comp))

    sys.setrecursionlimit(max(10000, sys.getrecursionlimit()))
    for v in graph:
        if v not in index:
            strong(v)
    return sorted(sccs)


def hub_edges(crates: Dict[str, Crate], hubs: Iterable[str]) -> List[Tuple[str, str, str]]:
    """Alle normalen internen Kanten, bei denen ein Ende eine Hub-Crate ist.
    Rückgabe: (von, nach, kind)."""
    hub_set = set(hubs)
    out = []
    for c in crates.values():
        for d in c.deps:
            if d.internal and d.name in crates and (c.name in hub_set or d.name in hub_set):
                out.append((c.name, d.name, d.kind))
    return sorted(set(out))


def matrix(crates: Dict[str, Crate]) -> Tuple[List[str], Dict[str, Dict[str, str]]]:
    names = sorted(crates)
    m: Dict[str, Dict[str, str]] = {a: {} for a in names}
    for a in names:
        for b in names:
            if a == b:
                m[a][b] = "·"
                continue
            kinds = {d.kind for d in crates[a].deps if d.name == b}
            if "normal" in kinds:
                m[a][b] = "N"
            elif kinds:
                m[a][b] = "D"
            else:
                m[a][b] = ""
    return names, m


# --------------------------------------------------------------------------- #
# Ausgabe
# --------------------------------------------------------------------------- #

def _short(xs: Iterable[str], limit: int = 12) -> str:
    xs = list(xs)
    if not xs:
        return "–"
    if len(xs) <= limit:
        return ", ".join(f"`{x}`" for x in xs)
    return ", ".join(f"`{x}`" for x in xs[:limit]) + f" … (+{len(xs) - limit})"


def render_overview(crates: Dict[str, Crate], hubs: Iterable[str]) -> str:
    hub_set = set(hubs)
    lines = ["| Crate | Ring | Mitglied | Intern normal | Intern dev | Extern normal | Reverse | Hub-Kanten | unsafe | Policy | LOC |",
             "|---|---|---|---|---|---|---|---|---|---|---|"]
    for n in sorted(crates):
        c = crates[n]
        ext = sum(1 for d in c.deps if d.kind == "normal" and not d.internal)
        hub_k = sum(1 for d in c.deps if d.internal and d.name in hub_set and d.kind == "normal")
        lines.append(
            f"| `{n}` | {c.ring or '–'} | {'ja' if c.member else 'nein'} | "
            f"{len(c.internal_normal)} | {len(c.internal_dev)} | {ext} | {len(c.reverse)} | "
            f"{hub_k} | {c.unsafe_hits} | {c.unsafe_policy} | {c.loc_src} |")
    return "\n".join(lines)


def render_crate(c: Crate, crates: Dict[str, Crate]) -> str:
    out = [f"# {c.name}  ({c.path})", ""]
    if c.description:
        out.append(c.description)
        out.append("")
    out.append(f"- Ring: {c.ring or '–'} · Workspace-Mitglied: {'ja' if c.member else 'nein'} · "
               f"src: {c.src_files} Dateien / {c.loc_src} LOC · unsafe: {c.unsafe_hits} ({c.unsafe_policy})")
    out.append("")
    for kind, label in (("normal", "Abhängigkeiten (normal)"), ("dev", "Dev-Abhängigkeiten"),
                        ("build", "Build-Abhängigkeiten")):
        ds = c.deps_of(kind)
        if not ds:
            continue
        out.append(f"## {label}")
        for d in sorted(ds, key=lambda d: (not d.internal, d.name)):
            tag = "intern" if d.internal else "extern"
            extra = []
            if d.optional:
                extra.append("optional")
            if d.target:
                extra.append(f"target={d.target}")
            if d.features:
                extra.append("features=" + ",".join(d.features))
            if d.via_workspace:
                extra.append("workspace")
            out.append(f"- `{d.name}` [{tag}] {d.version or ''} {'(' + '; '.join(extra) + ')' if extra else ''}".rstrip())
        out.append("")
    out.append("## Reverse-Abhängigkeiten (wer nutzt mich)")
    out.append(_short(c.reverse, 50) if c.reverse else "keine (Blattknoten)")
    out.append("")
    if c.features:
        out.append("## Features")
        for k in sorted(c.features):
            out.append(f"- `{k}` → {', '.join(c.features[k]) or '–'}")
        out.append("")
    out.append("## Öffentliche API (aus src/, grob)")
    if not c.pub_items and not c.pub_reexports:
        out.append("keine erkennbaren `pub`-Items")
    for k in ("struct", "enum", "trait", "fn", "type", "const", "static", "mod"):
        if k in c.pub_items:
            out.append(f"- {k} ({len(c.pub_items[k])}): {_short(c.pub_items[k], 25)}")
    if c.pub_reexports:
        out.append(f"- pub use: {_short(c.pub_reexports, 15)}")
    return "\n".join(out)


def render_agents_block(c: Crate, crates: Dict[str, Crate], hubs: Iterable[str]) -> str:
    hub_set = set(hubs)
    int_n = sorted(c.internal_normal)
    int_d = sorted(c.internal_dev)
    ext_n = sorted({d.name for d in c.deps if d.kind == "normal" and not d.internal})
    hub_touch = sorted({d.name for d in c.deps if d.internal and d.kind == "normal" and d.name in hub_set})
    hub_rev = sorted(set(c.reverse) & hub_set)
    unsafe_line = (f"{c.unsafe_hits} Vorkommen, Policy `{c.unsafe_policy}`"
                   if c.unsafe_hits or c.unsafe_policy != "keiner" else "keines, kein `forbid/deny(unsafe_code)`")
    lines = [
        GEN_BEGIN,
        "",
        "## Inventar (generiert)",
        "",
        f"- **Pfad:** `{c.path}` · **Ring:** {c.ring or '–'} · **Version:** {c.version}",
        f"- **Beschreibung:** {c.description or '–'}",
        f"- **src:** {c.src_files} Dateien, {c.loc_src} LOC · **unsafe:** {unsafe_line}",
        f"- **Interne Abhängigkeiten (normal, {len(int_n)}):** {_short(int_n, 40)}",
        f"- **Interne Dev/Build-Abhängigkeiten ({len(int_d)}):** {_short(int_d, 40)}",
        f"- **Externe Abhängigkeiten (normal, {len(ext_n)}):** {_short(ext_n, 40)}",
        f"- **Reverse-Abhängigkeiten ({len(c.reverse)}):** {_short(c.reverse, 40)}",
        f"- **Kopplung an Hub-Crates:** ausgehend {_short(hub_touch) if hub_touch else '–'}; "
        f"eingehend {_short(hub_rev) if hub_rev else '–'}",
    ]
    if c.features:
        lines.append(f"- **Features ({len(c.features)}):** {_short(sorted(c.features), 40)}")
    if c.pub_items or c.pub_reexports:
        lines.append("")
        lines.append("### Öffentliche API (grob, aus src/)")
        for k in ("struct", "enum", "trait", "fn", "type", "const", "static", "mod"):
            if k in c.pub_items:
                lines.append(f"- {k} ({len(c.pub_items[k])}): {_short(c.pub_items[k], 15)}")
        if c.pub_reexports:
            lines.append(f"- pub use: {_short(c.pub_reexports, 10)}")
    lines += ["", "Regeneriert mit `python3 tools/crate_inventory.py agents`. "
              "Manuelle Notizen stehen außerhalb dieses Blocks.", "", GEN_END]
    return "\n".join(lines)


def write_agents(crates: Dict[str, Crate], root: Path, hubs: Iterable[str],
                 dry_run: bool) -> List[str]:
    """Schreibt/aktualisiert den generierten Block. Bestehender manueller Inhalt
    außerhalb des Markerblocks bleibt erhalten; fehlt die Datei, entsteht sie neu."""
    changed: List[str] = []
    for n in sorted(crates):
        c = crates[n]
        target = root / c.path / "AGENTS.md"
        block = render_agents_block(c, crates, hubs)
        if target.is_file():
            old = target.read_text(encoding="utf-8")
            b = old.find(GEN_BEGIN)
            e = old.rfind(GEN_END) if b != -1 else -1  # letzter Marker: entfernt auch Altreste
            if b != -1 and e != -1:
                new = old[:b] + block + old[e + len(GEN_END):]
            else:
                new = old.rstrip() + "\n\n" + block + "\n"
        else:
            new = (f"# AGENTS.md — `{n}`\n\n> Werkzeug/Crate-Kontext für Agenten. "
                   f"Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.\n\n"
                   + block + "\n")
        prev = target.read_text(encoding="utf-8") if target.is_file() else None
        if new != prev:
            changed.append(c.path + "/AGENTS.md")
            if not dry_run:
                target.write_text(new, encoding="utf-8")
    return changed


# --------------------------------------------------------------------------- #
# CLI
# --------------------------------------------------------------------------- #

def main(argv: Optional[List[str]] = None) -> int:
    ap = argparse.ArgumentParser(description="Crate-Abhängigkeits- und Funktionsinventar")
    ap.add_argument("--root", type=Path, default=None, help="Repo-Root (Standard: automatisch)")
    ap.add_argument("--hubs", default=",".join(DEFAULT_HUBS),
                    help="Komma-getrennte Hub-Crates (Standard: %(default)s)")
    sub = ap.add_subparsers(dest="cmd")

    p = sub.add_parser("overview", help="Tabelle aller Crates")
    p.add_argument("--json", action="store_true")

    p = sub.add_parser("crate", help="Detail eines Crates")
    p.add_argument("name")
    p.add_argument("--json", action="store_true")

    sub.add_parser("matrix", help="Abhängigkeitsmatrix (N=normal, D=dev/build)")
    sub.add_parser("hubs", help="Kanten in/aus Hub-Crates").add_argument("--json", action="store_true")
    sub.add_parser("cycles", help="Zyklen im internen Graphen")
    sub.add_parser("json", help="Vollständiges Inventar als JSON")

    p = sub.add_parser("agents", help="AGENTS.md-Inventarblöcke schreiben")
    p.add_argument("--dry-run", action="store_true")
    p.add_argument("--only", help="nur dieses Crate")

    p = sub.add_parser("check", help="Gate: Zyklen und Hub-Kanten prüfen")
    p.add_argument("--max-hub-edges", type=int, default=None,
                   help="Maximal erlaubte Hub-Kanten (Standard: nur Zyklen prüfen)")

    args = ap.parse_args(argv)
    cmd = args.cmd or "overview"
    root = args.root.resolve() if args.root else repo_root()
    hubs = [h for h in args.hubs.split(",") if h]
    crates = load_inventory(root)

    if cmd == "overview":
        if getattr(args, "json", False):
            print(json.dumps({n: {"ring": c.ring, "member": c.member,
                                  "internal_normal": sorted(c.internal_normal),
                                  "reverse": c.reverse} for n, c in crates.items()},
                             ensure_ascii=False, indent=2))
        else:
            print(render_overview(crates, hubs))

    elif cmd == "crate":
        if args.name not in crates:
            print(f"Unbekanntes Crate: {args.name}", file=sys.stderr)
            return 2
        c = crates[args.name]
        if args.json:
            print(json.dumps(asdict(c), ensure_ascii=False, indent=2))
        else:
            print(render_crate(c, crates))

    elif cmd == "matrix":
        names, m = matrix(crates)
        w = max(len(n) for n in names)
        print(" " * (w + 1) + " ".join(str(i % 10) for i in range(len(names))))
        for i, a in enumerate(names):
            row = " ".join(m[a][b] or "." for b in names)
            print(f"{a:<{w}} {row}   [{i}]")
        print("\nLegende: N normal · D dev/build · · Diagonale · . keine Kante")

    elif cmd == "hubs":
        edges = hub_edges(crates, hubs)
        if args.json:
            print(json.dumps([{"from": a, "to": b, "kind": k} for a, b, k in edges],
                             ensure_ascii=False, indent=2))
        else:
            for a, b, k in edges:
                print(f"{a} -[{k}]-> {b}")
            print(f"\n{len(edges)} Kanten in/aus Hubs {hubs}")

    elif cmd == "cycles":
        cyc = find_cycles(crates)
        if not cyc:
            print("Keine Zyklen im internen Normal-Graphen.")
        for comp in cyc:
            print("Zyklus: " + " ↔ ".join(comp))

    elif cmd == "json":
        print(json.dumps({n: asdict(c) for n, c in crates.items()}, ensure_ascii=False, indent=2))

    elif cmd == "agents":
        targets = crates
        if args.only:
            if args.only not in crates:
                print(f"Unbekanntes Crate: {args.only}", file=sys.stderr)
                return 2
            targets = {args.only: crates[args.only]}
        changed = write_agents(targets, root, hubs, dry_run=args.dry_run)
        verb = "würde ändern" if args.dry_run else "geändert"
        print(f"{len(changed)} Datei(en) {verb}:")
        for p_ in changed:
            print(f"  {p_}")

    elif cmd == "check":
        status = 0
        cyc = find_cycles(crates)
        for comp in cyc:
            print("FEHLER Zyklus: " + " ↔ ".join(comp))
            status = 1
        if args.max_hub_edges is not None:
            n = len(hub_edges(crates, hubs))
            if n > args.max_hub_edges:
                print(f"FEHLER Hub-Kanten: {n} > erlaubt {args.max_hub_edges}")
                status = 1
            else:
                print(f"OK Hub-Kanten: {n} <= {args.max_hub_edges}")
        if status == 0:
            print("OK keine Zyklen")
        return status

    return 0


if __name__ == "__main__":
    sys.exit(main())
