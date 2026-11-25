#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
crate_coupling_report.py
========================

Statische Kopplungsanalyse für Rust-Workspaces (nur Python-Standardbibliothek,
Python >= 3.8; für Python < 3.11 wird zusätzlich ``tomli`` benötigt).

Das Skript liest alle Crates eines Cargo-Workspaces und erzeugt ein
Markdown-Dokument, das pro Crate und pro Komponente (Modul) zeigt:

  * welche anderen Workspace-Crates importiert werden (use, Inline-Pfade,
    extern crate) und welche Symbole genau,
  * welche öffentliche Schnittstelle das Crate anbietet (pub-Items, die über
    eine Kette von ``pub mod`` oder per ``pub use`` erreichbar sind),
  * welche Symbole andere Crates tatsächlich nutzen (Eingehend),
  * wo die öffentliche API Typen fremder Crates enthüllt ("API-Leck"),
  * Abweichungen zwischen Cargo.toml und tatsächlichem Code,
  * Zyklen, schwache Kanten (billig zu trennen) und Hub-Symbole
    (Kandidaten zum Verschieben nach types/ports).

Aufruf
------
    python3 crate_coupling_report.py [WORKSPACE_ROOT] [-o report.md]
        [--depth 1] [--top-symbols 10] [--max-items 80]
        [--only crate-a,crate-b] [--json out.json] [--mermaid]

Grenzen (bewusst, da kein echter Rust-Parser):
  * Kein Makro-Expansions- und kein Name-Resolution-Lauf. Symbole werden
    textuell aus ``use``-Bäumen und ``crate_ident::pfad``-Vorkommen gelesen.
  * Glob-Importe (``use x::*``) werden als ``*`` ausgewiesen; welche Symbole
    wirklich genutzt werden, ist nicht auflösbar.
  * API-Lecks werden nur erkannt, wenn der fremde Typ in der gleichen Datei
    importiert oder voll qualifiziert geschrieben ist.
  * Items hinter ``#[cfg(...)]`` werden mitgezählt (cfg wird nur für
    ``test`` ausgewertet) und mit ihrem cfg-Ausdruck vermerkt.
  * Die Ergebnisse sind Hinweise für die Entkopplung, kein Compiler-Beweis.
"""
from __future__ import annotations

import argparse
import bisect
import datetime as _dt
import json
import re
import sys
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple

try:  # Python >= 3.11
    import tomllib as _toml
except ModuleNotFoundError:  # pragma: no cover
    try:
        import tomli as _toml  # type: ignore
    except ModuleNotFoundError:
        _toml = None

SKIP_DIRS = {"target", ".git", "node_modules", ".venv", "venv", "__pycache__"}


# ---------------------------------------------------------------------------
# Datenmodell
# ---------------------------------------------------------------------------
@dataclass
class Dep:
    alias: str
    package: str
    kind: str  # normal | dev | build
    optional: bool
    target: Optional[str]
    internal: bool = False
    features: List[str] = field(default_factory=list)


@dataclass
class Crate:
    name: str
    ident: str
    dir: Path
    manifest: dict
    deps: List[Dep] = field(default_factory=list)
    ring: Optional[str] = None
    loc: int = 0
    files: List[Path] = field(default_factory=list)
    features: Dict[str, List[str]] = field(default_factory=dict)


@dataclass
class Imp:
    src: str
    dst: str
    symbol: str
    mode: str  # use | inline | extern
    file: str
    comp: str
    scope: str  # src | bin | build | test | bench | example | fuzz
    cls: str  # prod | dev
    line: int
    cfg: str
    reexport: bool
    declared: bool


@dataclass
class Item:
    kind: str
    name: str
    sig: str
    file: str
    comp: str
    line: int
    inline_pub: bool
    body: str = ""
    methods: List[str] = field(default_factory=list)
    foreign: Set[str] = field(default_factory=set)
    status: str = "intern"  # pub | reexp | intern
    cfg: str = ""


@dataclass
class FileScan:
    rel: str
    comp: str
    scope: str
    loc: int
    imports: List[dict] = field(default_factory=list)
    items: List[Item] = field(default_factory=list)
    mod_decls: List[Tuple[str, bool]] = field(default_factory=list)
    reexports_internal: List[Tuple[str, str]] = field(default_factory=list)
    inline_hits: List[dict] = field(default_factory=list)


# ---------------------------------------------------------------------------
# Rust-Text-Hilfen (leichtgewichtig)
# ---------------------------------------------------------------------------
_RAW_STR = re.compile(r'b?r(#*)"')


def sanitize(src: str) -> str:
    """Ersetzt Kommentare sowie String-/Char-Inhalte durch Leerzeichen.

    Die Länge und alle Zeilenumbrüche bleiben exakt erhalten, damit Offsets
    und Zeilennummern zum Original passen.
    """
    out: List[str] = []
    i, n = 0, len(src)

    def blank(seg: str) -> str:
        return re.sub(r"[^\n]", " ", seg)

    while i < n:
        c = src[i]
        nxt = src[i + 1] if i + 1 < n else ""
        if c == "/" and nxt == "/":
            j = src.find("\n", i)
            j = n if j < 0 else j
            out.append(" " * (j - i))
            i = j
            continue
        if c == "/" and nxt == "*":
            depth, j = 1, i + 2
            while j < n and depth:
                if src.startswith("/*", j):
                    depth += 1
                    j += 2
                elif src.startswith("*/", j):
                    depth -= 1
                    j += 2
                else:
                    j += 1
            out.append(blank(src[i:j]))
            i = j
            continue
        prev_ident = i > 0 and (src[i - 1].isalnum() or src[i - 1] == "_")
        if c in "rb" and not prev_ident:
            m = _RAW_STR.match(src, i)
            if m:
                closing = '"' + m.group(1)
                j = src.find(closing, m.end())
                j = n if j < 0 else j + len(closing)
                out.append(blank(src[i:j]))
                i = j
                continue
        if c == '"':
            j = i + 1
            while j < n and src[j] != '"':
                j += 2 if src[j] == "\\" else 1
            j = min(j + 1, n)
            out.append('"' + blank(src[i + 1 : j - 1]) + '"' if j - i >= 2 else blank(src[i:j]))
            i = j
            continue
        if c == "'":
            if nxt == "\\":
                j = src.find("'", i + 3)
                j = n if j < 0 else j + 1
                out.append(blank(src[i:j]))
                i = j
                continue
            if i + 2 < n and src[i + 2] == "'" and nxt != "'":
                out.append("   ")
                i += 3
                continue
            # Lifetime -> unverändert lassen
        out.append(c)
        i += 1
    return "".join(out)


_STRIP_ATTR = re.compile(r"#!?\s*\[[^\[\]]*(?:\[[^\[\]]*\][^\[\]]*)*\]")
_CFG_RE = re.compile(r"#!?\s*\[\s*cfg\s*\((.*?)\)\s*\]", re.S)
_USE_RE = re.compile(r"^(pub(?:\s*\([^)]*\))?\s+)?use\s+(.*)$", re.S)
_EXTERN_CRATE = re.compile(r"^(?:pub\s+)?extern\s+crate\s+(\w+)")
_ITEM_RE = re.compile(
    r"^pub\s+(?:(?:unsafe|async|const|extern(?:\s*\"[^\"]*\")?|default|auto)\s+)*"
    r"(fn|struct|enum|trait|type|const|static|mod|union)\s+(?:mut\s+)?(?:r#)?(\w+)"
)
_FN_RE = re.compile(r"\bfn\s+(?:r#)?(\w+)")
_PUB_FIELD = re.compile(r"\bpub\s+(?!\()(\w+)\s*:\s*([^,]+)")
_IDENT = re.compile(r"[A-Za-z_]\w*")


def norm_ws(s: str) -> str:
    return re.sub(r"\s+", " ", s).strip()


def cfg_is_test(cfg: str) -> bool:
    if not cfg:
        return False
    if re.search(r"not\s*\(\s*test", cfg):
        return False
    return bool(re.search(r"\btest\b", cfg))


def strip_leading_generics(s: str) -> str:
    s = s.lstrip()
    if not s.startswith("<"):
        return s
    depth = 0
    for i, ch in enumerate(s):
        if ch == "<":
            depth += 1
        elif ch == ">":
            depth -= 1
            if depth == 0:
                return s[i + 1 :].lstrip()
    return s


def block_info(h: str) -> Tuple[str, str, bool, bool]:
    """-> (kind, name, is_pub, trait_impl)"""
    is_pub = bool(re.match(r"^pub\s*(?!\()", h)) and not re.match(r"^pub\s*\(", h)
    body = re.sub(r"^pub(?:\s*\([^)]*\))?\s+", "", h)
    m = re.match(r"^(?:unsafe\s+)?(?:extern\s*(?:\"[^\"]*\")?\s+)?mod\s+(?:r#)?(\w+)", body)
    if m:
        return "mod", m.group(1), is_pub, False
    m = re.match(r"^(?:unsafe\s+)?impl\b(.*)$", body, re.S)
    if m:
        rest = strip_leading_generics(m.group(1))
        if re.search(r"\bfor\b", rest):
            return "impl", "", False, True
        rest = re.split(r"\bwhere\b", rest)[0]
        t = re.match(r"[&\s]*(?:dyn\s+)?([\w:]+)", rest)
        name = t.group(1).split("::")[-1] if t else "?"
        return "impl", name, False, False
    m = re.match(r"^(?:unsafe\s+)?(?:auto\s+)?trait\s+(?:r#)?(\w+)", body)
    if m:
        return "trait", m.group(1), is_pub, False
    m = re.match(r"^(struct|enum|union)\s+(?:r#)?(\w+)", body)
    if m:
        return m.group(1), m.group(2), is_pub, False
    if re.match(r"^macro_rules\s*!", body):
        return "macro", "", False, False
    if re.search(r"(?:^|\s)fn\s+\w+", body) and "=" not in body.split("fn", 1)[0]:
        return "fn", "", is_pub, False
    return "other", "", False, False


# ---------------------------------------------------------------------------
# use-Baum-Parser
# ---------------------------------------------------------------------------
_TOK = re.compile(r"::|[{},*]|r#\w+|[A-Za-z_]\w*")


def expand_use(text: str) -> List[Tuple[List[str], Optional[str]]]:
    toks = _TOK.findall(text)

    def parse(i: int, prefix: List[str]):
        res: List[Tuple[List[str], Optional[str]]] = []
        path = list(prefix)
        if i < len(toks) and toks[i] == "::":
            i += 1
        while i < len(toks):
            t = toks[i]
            if t == "{":
                i += 1
                while i < len(toks):
                    sub, i = parse(i, path)
                    res += sub
                    if i < len(toks) and toks[i] == ",":
                        i += 1
                        continue
                    break
                if i < len(toks) and toks[i] == "}":
                    i += 1
                return res, i
            if t == "*":
                res.append((path + ["*"], None))
                return res, i + 1
            if t in (",", "}"):
                if path != list(prefix):
                    res.append((path, None))
                return res, i
            if t == "as":
                alias = toks[i + 1] if i + 1 < len(toks) else None
                res.append((path, alias))
                return res, i + 2
            if t == "::":
                i += 1
                continue
            path.append(t)
            i += 1
        if path != list(prefix):
            res.append((path, None))
        return res, i

    out, _ = parse(0, [])
    cleaned = []
    for p, a in out:
        if p and p[-1] == "self":
            p = p[:-1]
        if p:
            cleaned.append((p, a))
    return cleaned


# ---------------------------------------------------------------------------
# Datei-Scan
# ---------------------------------------------------------------------------
def scan_source(raw: str, rel: str, comp: str, scope: str, ident_resolver, local_mods_hint: Set[str]) -> FileScan:
    code = sanitize(raw)
    nl = [m.start() for m in re.finditer(r"\n", raw)]

    def line_of(off: int) -> int:
        return bisect.bisect_right(nl, off) + 1

    fs = FileScan(rel=rel, comp=comp, scope=scope, loc=sum(1 for ln in raw.splitlines() if ln.strip()))
    stack: List[dict] = []
    test_ranges: List[Tuple[int, int]] = []
    use_spans: List[Tuple[int, int]] = []
    file_cfg_test = bool(re.search(r"#!\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]", code))
    buf_start = 0

    def in_test() -> bool:
        return file_cfg_test or any(e["test"] for e in stack)

    def ctx() -> str:
        if all(e["kind"] == "mod" for e in stack):
            return "top"
        top = stack[-1]
        if top["kind"] in ("impl", "trait") and all(e["kind"] == "mod" for e in stack[:-1]):
            return top["kind"]
        return "skip"

    def inline_pub() -> bool:
        return all(e["pub"] for e in stack if e["kind"] == "mod")

    def handle(header: str, off: int, closing: str):
        cfgs = [c.strip() for c in _CFG_RE.findall(header)]
        cfg = " & ".join(cfgs)
        h = norm_ws(_STRIP_ATTR.sub(" ", header))
        if not h:
            return None, cfg
        mu = _USE_RE.match(h)
        if mu and closing == ";":
            use_spans.append((off, off + len(header)))
            is_pub = bool(mu.group(1)) and "(" not in mu.group(1)
            ln = line_of(off + (len(header) - len(header.lstrip())))
            for path, alias in expand_use(mu.group(2)):
                fs.imports.append(
                    dict(path=path, alias=alias, pub=is_pub, line=ln, cfg=cfg,
                         test=in_test() or cfg_is_test(cfg), top=(ctx() == "top"))
                )
            return None, cfg
        me = _EXTERN_CRATE.match(h)
        if me and closing == ";":
            fs.imports.append(dict(path=[me.group(1)], alias=None, pub=False, line=line_of(off), cfg=cfg,
                                   test=in_test() or cfg_is_test(cfg), top=True, extern=True))
            return None, cfg
        return h, cfg

    for m in re.finditer(r"[{};]", code):
        c = m.group()
        i = m.start()
        header = code[buf_start:i]
        buf_start = i + 1
        if c == "}":
            if not stack:
                continue
            e = stack.pop()
            if e["test"]:
                test_ranges.append((e["start"], i))
            if e.get("item") is not None and e["kind"] in ("struct", "enum", "union"):
                body = code[e["start"] + 1 : i]
                it: Item = e["item"]
                if e["kind"] == "enum":
                    it.body = norm_ws(body)
                else:
                    it.body = " ".join(t for _, t in _PUB_FIELD.findall(body))
            continue
        h, cfg = handle(header, i - len(header), c)
        if h is None:
            if c == "{":
                stack.append(dict(kind="other", name="", pub=False, test=False, start=i, item=None))
            continue
        test_here = cfg_is_test(cfg)
        pushed = None
        if c == "{":
            kind, name, is_pub, trait_impl = block_info(h)
            pushed = dict(kind=kind, name=name, pub=is_pub, test=test_here, start=i, item=None, trait_impl=trait_impl)
        context = ctx() if stack or True else "top"
        if not stack:
            context = "top"
        if in_test() or test_here:
            if pushed is not None:
                stack.append(pushed)
            continue
        mi = _ITEM_RE.match(h)
        item = None
        if context == "top" and mi and mi.group(2) != "_":
            kind, name = mi.group(1), mi.group(2)
            if kind == "mod" and c == ";":
                fs.mod_decls.append((name, True))
            elif kind != "mod":
                item = Item(kind=kind, name=name, sig=h, file=rel, comp=comp, line=line_of(i - len(header) + len(header) - len(header.lstrip())),
                            inline_pub=inline_pub(), cfg=cfg)
                fs.items.append(item)
        elif context == "top" and c == ";":
            mm = re.match(r"^(?:pub\s*\([^)]*\)\s+)?(?:unsafe\s+)?(?:extern\s*(?:\"[^\"]*\")?\s+)?mod\s+(?:r#)?(\w+)$", h)
            if mm:
                fs.mod_decls.append((mm.group(1), False))
        elif context == "impl" and mi and mi.group(1) == "fn":
            top = stack[-1]
            if not top.get("trait_impl"):
                it = next((x for x in reversed(fs.items) if x.name == top["name"] and x.kind in ("struct", "enum", "union", "type", "trait")), None)
                if it is not None:
                    it.methods.append(mi.group(2))
                else:
                    # Typ in anderer Datei definiert: eigener Pseudo-Eintrag
                    fs.items.append(Item(kind="impl", name=top["name"], sig=f"impl {top['name']}", file=rel, comp=comp,
                                         line=line_of(i), inline_pub=inline_pub(), methods=[mi.group(2)]))
        elif context == "trait":
            fm = _FN_RE.search(h)
            top = stack[-1]
            if fm:
                it = next((x for x in reversed(fs.items) if x.name == top["name"] and x.kind == "trait"), None)
                if it is not None:
                    it.methods.append(fm.group(1))
        if pushed is not None:
            pushed["item"] = item if (item and item.kind in ("struct", "enum", "union")) else None
            if item is not None and item.kind == "trait":
                pass
            stack.append(pushed)

    # Inline-Pfade (außerhalb von use-Anweisungen)
    blanked = list(code)
    for a, b in use_spans:
        for k in range(a, min(b, len(blanked))):
            if blanked[k] != "\n":
                blanked[k] = " "
    blanked_s = "".join(blanked)
    inline_re = ident_resolver["inline_re"]
    if inline_re is not None:
        for m in inline_re.finditer(blanked_s):
            ident = m.group(1)
            if ident in local_mods_hint:
                continue
            off = m.start()
            in_t = file_cfg_test or any(a <= off <= b for a, b in test_ranges)
            fs.inline_hits.append(dict(ident=ident, path=m.group(2), line=line_of(off), test=in_t))
    fs.test_ranges = test_ranges  # type: ignore[attr-defined]
    return fs


# ---------------------------------------------------------------------------
# Workspace-Erkennung
# ---------------------------------------------------------------------------
def load_toml(p: Path) -> dict:
    if _toml is None:
        sys.exit("Fehler: Python < 3.11 braucht das Paket 'tomli' (pip install tomli).")
    with p.open("rb") as fh:
        return _toml.load(fh)


def discover_crates(root: Path) -> List[Crate]:
    manifests: List[Path] = []
    rootm = root / "Cargo.toml"
    members: List[str] = []
    excludes: List[str] = []
    if rootm.exists():
        d = load_toml(rootm)
        ws = d.get("workspace", {})
        members = ws.get("members", [])
        excludes = ws.get("exclude", [])
        if "package" in d:
            manifests.append(rootm)
    if members:
        excl = set()
        for pat in excludes:
            for p in root.glob(pat):
                excl.add(p.resolve())
        for pat in members:
            for p in sorted(root.glob(pat)):
                if (p / "Cargo.toml").exists() and p.resolve() not in excl:
                    manifests.append(p / "Cargo.toml")
    else:
        for p in sorted(root.rglob("Cargo.toml")):
            if any(part in SKIP_DIRS for part in p.relative_to(root).parts):
                continue
            manifests.append(p)
    crates: List[Crate] = []
    seen: Set[str] = set()
    for mp in manifests:
        try:
            d = load_toml(mp)
        except Exception as exc:  # pragma: no cover
            print(f"Warnung: {mp}: {exc}", file=sys.stderr)
            continue
        pkg = d.get("package")
        if not pkg or "name" not in pkg:
            continue
        name = pkg["name"]
        if name in seen:
            continue
        seen.add(name)
        libname = d.get("lib", {}).get("name", name)
        ring = None
        meta = pkg.get("metadata", {}) if isinstance(pkg.get("metadata", {}), dict) else {}
        if isinstance(meta.get("contextra"), dict):
            ring = str(meta["contextra"].get("ring")) if "ring" in meta["contextra"] else None
        crates.append(Crate(name=name, ident=libname.replace("-", "_"), dir=mp.parent, manifest=d,
                            ring=ring, features=d.get("features", {})))
    names = {c.name for c in crates}
    for c in crates:
        c.deps = parse_deps(c.manifest, names)
        enabling: Dict[str, List[str]] = defaultdict(list)
        for feat, items in c.features.items():
            for it in items:
                base = it.split("/")[0]
                base = base[4:] if base.startswith("dep:") else base
                enabling[base].append(feat)
        for d in c.deps:
            if d.optional:
                d.features = sorted(set(enabling.get(d.alias, [])))
    return crates


def parse_deps(manifest: dict, internal_names: Set[str]) -> List[Dep]:
    out: List[Dep] = []

    def add(tbl: dict, kind: str, target: Optional[str]):
        for alias, spec in (tbl or {}).items():
            pkg, opt = alias, False
            if isinstance(spec, dict):
                pkg = spec.get("package", alias)
                opt = bool(spec.get("optional", False))
            out.append(Dep(alias=alias, package=pkg, kind=kind, optional=opt, target=target,
                           internal=pkg in internal_names))

    for key, kind in (("dependencies", "normal"), ("dev-dependencies", "dev"), ("build-dependencies", "build")):
        add(manifest.get(key, {}), kind, None)
    for tgt, tdoc in (manifest.get("target", {}) or {}).items():
        if isinstance(tdoc, dict):
            for key, kind in (("dependencies", "normal"), ("dev-dependencies", "dev"), ("build-dependencies", "build")):
                add(tdoc.get(key, {}), kind, tgt)
    return out


def collect_rs_files(c: Crate) -> List[Path]:
    files: List[Path] = []
    for p in sorted(c.dir.rglob("*.rs")):
        rel = p.relative_to(c.dir)
        if any(part in SKIP_DIRS for part in rel.parts):
            continue
        files.append(p)
    return files


def classify_file(rel: Path) -> Tuple[str, str]:
    """-> (scope, component)"""
    parts = rel.parts
    stem = rel.stem
    if parts[0] == "src":
        inner = list(parts[1:])
        if inner and inner[0] == "bin":
            return "bin", "bin::" + "::".join(Path(*inner[1:]).with_suffix("").parts) if len(inner) > 1 else "bin"
        mod = list(Path(*inner).with_suffix("").parts) if inner else []
        if mod and mod[-1] in ("lib", "main") and len(mod) == 1:
            comp = "(crate-root)"
        else:
            if mod and mod[-1] == "mod":
                mod = mod[:-1]
            comp = "::".join(mod) if mod else "(crate-root)"
        is_test = ("tests" in inner[:-1]) or stem in ("tests", "test") or stem.endswith("_tests") or stem.endswith("_test")
        return ("test" if is_test else "src"), comp
    if parts[0] in ("tests", "benches", "examples", "fuzz"):
        sc = {"tests": "test", "benches": "bench", "examples": "example", "fuzz": "fuzz"}[parts[0]]
        return sc, f"{parts[0]}/{'/'.join(parts[1:])}"
    if parts[0] == "build.rs":
        return "build", "build.rs"
    return "src", "/".join(parts)


# ---------------------------------------------------------------------------
# Analyse
# ---------------------------------------------------------------------------
class Analysis:
    def __init__(self, root: Path, crates: List[Crate], depth: int):
        self.root = root
        self.crates = {c.name: c for c in crates}
        self.depth = depth
        self.scans: Dict[str, List[FileScan]] = {}
        self.imps: List[Imp] = []
        self.global_idents = {c.ident: c.name for c in crates}
        alt = "|".join(sorted((re.escape(i) for i in self.global_idents), key=len, reverse=True))
        self.inline_re = re.compile(r"(?<![\w:])(" + alt + r")::(\w+(?:::\w+)*)") if alt else None

    # -- Scan -----------------------------------------------------------
    def run(self, only: Optional[Set[str]] = None):
        for c in self.crates.values():
            if only and c.name not in only:
                continue
            self.scan_crate(c)
        self.resolve_api()

    def alias_map(self, c: Crate) -> Dict[str, str]:
        m = {}
        for d in c.deps:
            if d.internal:
                m[d.alias.replace("-", "_")] = d.package
        return m

    def scan_crate(self, c: Crate):
        files = collect_rs_files(c)
        c.files = files
        alias = self.alias_map(c)
        resolver = {"inline_re": self.inline_re}
        scans: List[FileScan] = []
        test_mod_files: Set[str] = set()
        raw_cache: Dict[Path, str] = {}
        for p in files:
            try:
                raw_cache[p] = p.read_text(encoding="utf-8", errors="replace")
            except OSError:
                raw_cache[p] = ""
            for m in re.finditer(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*(?:pub(?:\s*\([^)]*\))?\s+)?mod\s+(\w+)\s*;",
                                 raw_cache[p]):
                base = p.parent if p.name in ("lib.rs", "main.rs", "mod.rs") else p.parent / p.stem
                test_mod_files.add((base / f"{m.group(1)}.rs").relative_to(c.dir).as_posix())
                test_mod_files.add((base / m.group(1) / "mod.rs").relative_to(c.dir).as_posix())
        local_mods: Set[str] = set()
        for p in files:
            for m in re.finditer(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)", raw_cache[p], re.M):
                local_mods.add(m.group(1))
        for p in files:
            rel = p.relative_to(c.dir)
            scope, comp = classify_file(rel)
            if rel.as_posix() in test_mod_files:
                scope = "test"
            fs = scan_source(raw_cache[p], rel.as_posix(), comp, scope, resolver, local_mods & set(self.global_idents))
            scans.append(fs)
            c.loc += fs.loc
            self.collect_imports(c, fs, alias)
        self.scans[c.name] = scans
        self.compute_status(c, scans)

    def collect_imports(self, c: Crate, fs: FileScan, alias: Dict[str, str]):
        declared_idents = set(alias)
        fs.name_map = {}  # type: ignore[attr-defined]
        fs.globs = set()  # type: ignore[attr-defined]
        scope_prod = fs.scope in ("src", "bin", "build")

        def pkg_of(ident: str) -> Tuple[Optional[str], bool]:
            if ident in alias:
                return alias[ident], True
            if ident in self.global_idents:
                return self.global_idents[ident], False
            return None, False

        for imp in fs.imports:
            path = imp["path"]
            first = path[0]
            if first in ("crate", "self", "super", "std", "core", "alloc"):
                if imp["pub"] and imp["top"] and first in ("crate", "self", "super"):
                    nm = imp["alias"] or path[-1]
                    fs.reexports_internal.append((nm, "::".join(path)))
                continue
            pkg, declared = pkg_of(first)
            if pkg is None or pkg == c.name:
                continue
            rest = path[1:]
            symbol = "::".join(rest) if rest else "(crate)"
            if rest and rest[-1] == "*":
                fs.globs.add(pkg)
            else:
                nm = imp["alias"] or (rest[-1] if rest else first)
                fs.name_map[nm] = pkg
            test = imp["test"] or not scope_prod
            self.imps.append(Imp(src=c.name, dst=pkg, symbol=symbol,
                                 mode="extern" if imp.get("extern") else "use",
                                 file=fs.rel, comp=fs.comp, scope=fs.scope if not imp["test"] else "test",
                                 cls="dev" if test else "prod", line=imp["line"], cfg=imp["cfg"],
                                 reexport=bool(imp["pub"] and imp["top"]), declared=declared))
        for h in fs.inline_hits:
            pkg, declared = pkg_of(h["ident"])
            if pkg is None or pkg == c.name:
                continue
            segs = h["path"].split("::")
            symbol = "::".join(segs[:3])
            test = h["test"] or not scope_prod
            fs.name_map.setdefault(segs[0], pkg)
            self.imps.append(Imp(src=c.name, dst=pkg, symbol=symbol, mode="inline", file=fs.rel, comp=fs.comp,
                                 scope=fs.scope if not h["test"] else "test", cls="dev" if test else "prod",
                                 line=h["line"], cfg="", reexport=False, declared=declared))

    # -- Öffentliche API ------------------------------------------------
    def compute_status(self, c: Crate, scans: List[FileScan]):
        by_rel = {s.rel: s for s in scans}
        root_rel = "src/lib.rs" if "src/lib.rs" in by_rel else ("src/main.rs" if "src/main.rs" in by_rel else None)
        public_files: Set[str] = set()
        if root_rel:
            todo = [(root_rel, True)]
            seen: Set[str] = set()
            while todo:
                rel, pub = todo.pop()
                if rel in seen:
                    continue
                seen.add(rel)
                if pub:
                    public_files.add(rel)
                s = by_rel.get(rel)
                if not s:
                    continue
                p = Path(rel)
                base = p.parent if p.name in ("lib.rs", "main.rs", "mod.rs") else p.parent / p.stem
                for name, is_pub in s.mod_decls:
                    for cand in ((base / f"{name}.rs").as_posix(), (base / name / "mod.rs").as_posix()):
                        if cand in by_rel:
                            todo.append((cand, pub and is_pub))
                            break
        names_reexp: Set[str] = set()
        glob_mods: Set[str] = set()
        for s in scans:
            if s.scope not in ("src",):
                continue
            for imp in s.imports:
                if imp["pub"] and imp["top"]:
                    p = imp["path"]
                    if p and p[-1] == "*":
                        if len(p) >= 2:
                            glob_mods.add(p[-2])
                    else:
                        names_reexp.add(imp["alias"] or p[-1])
        c.public_files = public_files  # type: ignore[attr-defined]
        for s in scans:
            if s.scope not in ("src", "bin"):
                continue
            for it in s.items:
                last = s.comp.split("::")[-1]
                if s.rel in public_files and it.inline_pub:
                    it.status = "pub"
                elif it.name in names_reexp or last in glob_mods:
                    it.status = "reexp"
                else:
                    it.status = "intern"

    def resolve_api(self):
        for cname, scans in self.scans.items():
            for s in scans:
                nm = getattr(s, "name_map", {})
                for it in s.items:
                    text = it.sig + " " + it.body
                    found: Set[str] = set()
                    if self.inline_re is not None:
                        for m in self.inline_re.finditer(text):
                            pk = self.global_idents.get(m.group(1))
                            if pk and pk != cname:
                                found.add(pk)
                    for tok in _IDENT.findall(text):
                        pk = nm.get(tok)
                        if pk and pk != cname:
                            found.add(pk)
                    it.foreign = found

    # -- Aggregation ----------------------------------------------------
    def comp_at_depth(self, comp: str) -> str:
        if "/" in comp or comp.startswith("("):
            return comp
        return "::".join(comp.split("::")[: self.depth])

    def edges(self) -> Dict[Tuple[str, str], dict]:
        e: Dict[Tuple[str, str], dict] = {}
        for i in self.imps:
            d = e.setdefault((i.src, i.dst), dict(prod=Counter(), dev=Counter(), files_prod=set(), files_dev=set(),
                                                  sites_prod=0, sites_dev=0, declared=True))
            d[i.cls][i.symbol] += 1
            d["sites_" + i.cls] += 1
            d["files_" + i.cls].add(i.file)
            d["declared"] = d["declared"] and i.declared
        return e


def scc(nodes: List[str], edges: Dict[str, Set[str]]) -> List[List[str]]:
    index = {}
    low = {}
    on = set()
    st: List[str] = []
    out: List[List[str]] = []
    counter = [0]
    sys.setrecursionlimit(10000)

    def strong(v):
        index[v] = low[v] = counter[0]
        counter[0] += 1
        st.append(v)
        on.add(v)
        for w in edges.get(v, ()):
            if w not in index:
                strong(w)
                low[v] = min(low[v], low[w])
            elif w in on:
                low[v] = min(low[v], index[w])
        if low[v] == index[v]:
            comp = []
            while True:
                w = st.pop()
                on.discard(w)
                comp.append(w)
                if w == v:
                    break
            if len(comp) > 1:
                out.append(sorted(comp))

    for v in nodes:
        if v not in index:
            strong(v)
    return out


# ---------------------------------------------------------------------------
# Markdown-Ausgabe
# ---------------------------------------------------------------------------
def esc(s: str) -> str:
    return s.replace("|", "\\|").replace("\n", " ")


def fmt_syms(cnt: Counter, top: int) -> str:
    items = cnt.most_common()
    shown = [f"`{esc(s)}`" + (f"×{n}" if n > 1 else "") for s, n in items[:top]]
    more = len(items) - top
    return ", ".join(shown) + (f" … (+{more})" if more > 0 else "")


def trunc(s: str, n: int = 130) -> str:
    s = norm_ws(s)
    return s if len(s) <= n else s[: n - 1] + "…"


def anchor(name: str) -> str:
    return re.sub(r"[^a-z0-9\- _]", "", name.lower()).replace(" ", "-")


def render(an: Analysis, args) -> Tuple[str, dict]:
    crates = [an.crates[n] for n in sorted(an.scans)]
    names = [c.name for c in crates]
    edges = an.edges()
    prod_out: Dict[str, Set[str]] = defaultdict(set)
    prod_in: Dict[str, Set[str]] = defaultdict(set)
    any_out: Dict[str, Set[str]] = defaultdict(set)
    for (s, d), v in edges.items():
        any_out[s].add(d)
        if v["sites_prod"]:
            prod_out[s].add(d)
            prod_in[d].add(s)
    cargo_out: Dict[str, Set[str]] = defaultdict(set)
    for c in crates:
        for d in c.deps:
            if d.internal and d.kind in ("normal", "build"):
                cargo_out[c.name].add(d.package)

    L: List[str] = []
    w = L.append
    w("# Crate-Kopplungsbericht")
    w("")
    w(f"- Erzeugt: {_dt.datetime.now().strftime('%Y-%m-%d %H:%M')}")
    w(f"- Workspace-Root: `{an.root}`")
    w(f"- Analysierte Crates: {len(crates)} · Komponenten-Tiefe: {an.depth} · Zeilen (ohne Leerzeilen): {sum(c.loc for c in crates):,}".replace(",", "."))
    w("- Legende: **prod** = src/bin/build ohne `cfg(test)`; **dev** = tests/benches/examples/fuzz und `cfg(test)`-Code.")
    w("- Hinweis: Textuelle Analyse ohne Compiler. Glob-Importe (`*`) sind nicht auflösbar. Siehe Kopf des Skripts für Grenzen.")
    w("")

    # ---- 1. Überblick
    w("## 1. Überblick")
    w("")
    w("| Crate | Ring | Zeilen | Fan-out prod | Fan-in prod | Cargo-Deps intern (n/d/b) | Pub-Items | API-Lecks | Zyklus |")
    w("|---|---|---:|---:|---:|---|---:|---:|---|")
    cyc_prod = scc(names, prod_out)
    cyc_cargo = scc(names, cargo_out)
    in_cycle = {n for g in cyc_prod for n in g}
    leak_count: Dict[str, int] = {}
    pub_count: Dict[str, int] = {}
    for c in crates:
        pubs = [it for s in an.scans[c.name] for it in s.items if it.status in ("pub", "reexp") and it.kind != "impl"]
        pub_count[c.name] = len(pubs)
        lk = sum(1 for it in pubs if it.foreign)
        lk += sum(1 for s in an.scans[c.name] for i in s.imports if i["pub"] and i["top"] and i["path"][0] in an.alias_map(c))
        leak_count[c.name] = lk
        dn = Counter(d.kind for d in c.deps if d.internal)
        w(f"| [{c.name}](#{anchor(c.name)}) | {c.ring or '–'} | {c.loc} | {len(prod_out[c.name])} | {len(prod_in[c.name])} | "
          f"{dn['normal']}/{dn['dev']}/{dn['build']} | {pub_count[c.name]} | {lk} | {'ja' if c.name in in_cycle else '–'} |")
    w("")

    # ---- 2. Auffälligkeiten
    w("## 2. Workspace-weite Auffälligkeiten")
    w("")
    w("### 2.1 Zyklen")
    w("")
    if cyc_prod:
        for g in cyc_prod:
            w(f"- Code-Zyklus (prod): {' ⇄ '.join(f'`{x}`' for x in g)}")
    else:
        w("- Kein Zyklus im prod-Code-Graphen gefunden.")
    if cyc_cargo:
        for g in cyc_cargo:
            w(f"- Cargo-Zyklus (normal/build): {' ⇄ '.join(f'`{x}`' for x in g)}")
    else:
        w("- Kein Zyklus in den normalen Cargo-Abhängigkeiten.")
    w("")

    w("### 2.2 Abweichungen Cargo.toml ↔ Code")
    w("")
    rows = []
    for c in crates:
        declared: Dict[str, Set[str]] = defaultdict(set)
        for d in c.deps:
            if d.internal:
                declared[d.package].add(d.kind + ("?" if d.optional else ""))
        used_prod = {d for (s, d), v in edges.items() if s == c.name and v["sites_prod"]}
        used_dev = {d for (s, d), v in edges.items() if s == c.name and v["sites_dev"]}
        for pkg, kinds in sorted(declared.items()):
            if pkg not in used_prod and pkg not in used_dev:
                rows.append((c.name, pkg, "/".join(sorted(kinds)), "kein Code-Zugriff gefunden (prüfen: Makro, Feature, cfg, oder entfernen)"))
            elif pkg not in used_prod and any(k.startswith(("normal", "build")) for k in kinds):
                rows.append((c.name, pkg, "/".join(sorted(kinds)), "nur in Tests/Benches genutzt, aber als normale Abhängigkeit deklariert → dev-dependency?"))
            elif pkg in used_prod and not any(k.startswith(("normal", "build")) for k in kinds):
                rows.append((c.name, pkg, "/".join(sorted(kinds)), "im prod-Code genutzt, aber nur als dev-Abhängigkeit deklariert"))
        for pkg in sorted((used_prod | used_dev) - set(declared)):
            rows.append((c.name, pkg, "–", "genutzt, aber nicht in Cargo.toml deklariert (cfg-/Feature-Pfad oder Fehler)"))
    if rows:
        w("| Crate | Ziel | Deklariert als | Befund |")
        w("|---|---|---|---|")
        for r in rows:
            w(f"| `{r[0]}` | `{r[1]}` | {r[2]} | {r[3]} |")
    else:
        w("Keine Abweichungen gefunden.")
    w("")

    w("### 2.3 Schwache Kanten (billig zu trennen)")
    w("")
    w("Kanten im prod-Code mit höchstens 3 verschiedenen Symbolen. Reihenfolge: wenigste Symbole, dann wenigste Fundstellen.")
    w("")
    weak = [(s, d, v) for (s, d), v in edges.items() if v["sites_prod"] and len(v["prod"]) <= 3]
    weak.sort(key=lambda t: (len(t[2]["prod"]), t[2]["sites_prod"]))
    if weak:
        w("| Von | Nach | Symbole | Fundstellen | Dateien | Symbole (prod) |")
        w("|---|---|---:|---:|---:|---|")
        for s, d, v in weak[:60]:
            w(f"| `{s}` | `{d}` | {len(v['prod'])} | {v['sites_prod']} | {len(v['files_prod'])} | {fmt_syms(v['prod'], 3)} |")
        if len(weak) > 60:
            w(f"\n… {len(weak) - 60} weitere Kanten (siehe `--json`).")
    else:
        w("Keine.")
    w("")
    dev_only = [(s, d, v) for (s, d), v in edges.items() if not v["sites_prod"]]
    if dev_only:
        w("**Nur in Tests/Benches/Beispielen** (für das Produkt irrelevant, aber sie koppeln den Test-Build):")
        w("")
        w("| Von | Nach | Fundstellen | Symbole |")
        w("|---|---|---:|---|")
        for s, d, v in sorted(dev_only, key=lambda t: (t[0], t[1])):
            w(f"| `{s}` | `{d}` | {v['sites_dev']} | {fmt_syms(v['dev'], 4)} |")
        w("")

    w("### 2.4 Hub-Symbole (Kandidaten für types/ports)")
    w("")
    w("Symbole, die aus einem Nicht-Basis-Crate von mindestens 3 verschiedenen Crates im prod-Code importiert werden. "
      "Wer sie in ein schmales Basis-Crate verschiebt, kann viele Kanten kappen.")
    w("")
    sym_users: Dict[Tuple[str, str], Set[str]] = defaultdict(set)
    for i in an.imps:
        if i.cls == "prod" and not i.symbol.endswith("*") and i.symbol != "(crate)":
            sym_users[(i.dst, i.symbol)].add(i.src)
    base = {n for n in names if n.endswith("-types") or n.endswith("-ports")}
    hubs = [(k, v) for k, v in sym_users.items() if len(v) >= 3 and k[0] not in base]
    hubs.sort(key=lambda t: (-len(t[1]), t[0]))
    if hubs:
        w("| Quelle | Symbol | Nutzer | Crates |")
        w("|---|---|---:|---|")
        for (d, sym), users in hubs[:40]:
            w(f"| `{d}` | `{esc(sym)}` | {len(users)} | {', '.join(f'`{u}`' for u in sorted(users))} |")
    else:
        w("Keine.")
    w("")

    w("### 2.5 API-Lecks")
    w("")
    w("Öffentliche Items, deren Signatur oder öffentliche Felder Typen anderer Workspace-Crates enthalten, sowie `pub use` fremder Crates. "
      "Diese Stellen machen eine Entkopplung am schwersten, weil Nutzer des Crates die fremde Abhängigkeit mitsehen.")
    w("")
    leak_rows = []
    for c in crates:
        for s in an.scans[c.name]:
            for it in s.items:
                if it.status in ("pub", "reexp") and it.foreign:
                    leak_rows.append((c.name, s.comp, f"{it.kind} {it.name}", sorted(it.foreign), s.rel, it.line))
            amap = an.alias_map(c)
            for imp in s.imports:
                if imp["pub"] and imp["top"] and imp["path"][0] in amap:
                    leak_rows.append((c.name, s.comp, "pub use " + "::".join(imp["path"]), [amap[imp["path"][0]]], s.rel, imp["line"]))
    if leak_rows:
        w("| Crate | Komponente | Item | Fremde Crates | Fundstelle |")
        w("|---|---|---|---|---|")
        for r in leak_rows[:150]:
            w(f"| `{r[0]}` | `{r[1]}` | `{esc(trunc(r[2], 90))}` | {', '.join(f'`{x}`' for x in r[3])} | `{r[4]}:{r[5]}` |")
        if len(leak_rows) > 150:
            w(f"\n… {len(leak_rows) - 150} weitere (siehe `--json`).")
    else:
        w("Keine erkannt.")
    w("")

    # ---- 3. Kantenliste
    w("## 3. Kantenliste")
    w("")
    w("| Von | Nach | prod Fundstellen | prod Symbole | dev Fundstellen | Cargo-Art |")
    w("|---|---|---:|---:|---:|---|")
    for (s, d), v in sorted(edges.items(), key=lambda kv: (-kv[1]["sites_prod"], kv[0])):
        c = an.crates[s]
        kinds = sorted({x.kind + ("?" if x.optional else "") for x in c.deps if x.package == d})
        w(f"| `{s}` | `{d}` | {v['sites_prod']} | {len(v['prod'])} | {v['sites_dev']} | {'/'.join(kinds) or '–'} |")
    w("")
    if args.mermaid:
        w("```mermaid")
        w("graph LR")
        short = lambda n: n.replace("contextra-", "") or n
        for (s, d), v in sorted(edges.items()):
            if v["sites_prod"]:
                w(f"  {short(s).replace('-', '_')}[{short(s)}] -->|{v['sites_prod']}| {short(d).replace('-', '_')}[{short(d)}]")
        w("```")
        w("")

    # ---- 4. Crates
    w("## 4. Crates im Detail")
    w("")
    inbound: Dict[str, Dict[str, Counter]] = defaultdict(lambda: defaultdict(Counter))
    for i in an.imps:
        inbound[i.dst][i.src][(i.symbol, i.cls)] += 1

    for c in crates:
        scans = an.scans[c.name]
        w(f"### {c.name}")
        w("")
        w(f"- Pfad: `{c.dir.relative_to(an.root).as_posix() if c.dir != an.root else '.'}` · Zeilen: {c.loc} · Rust-Dateien: {len(c.files)}"
          + (f" · Ring: {c.ring}" if c.ring else ""))
        idps = [d for d in c.deps if d.internal]
        if idps:
            parts = []
            for d in sorted(idps, key=lambda x: (x.kind, x.package)):
                tag = d.kind + ("/optional" if d.optional else "") + (f"/{d.target}" if d.target else "")
                if d.features:
                    tag += " via " + ",".join(d.features)
                parts.append(f"`{d.package}` ({tag})")
            w("- Interne Cargo-Abhängigkeiten: " + ", ".join(parts))
        else:
            w("- Interne Cargo-Abhängigkeiten: keine")
        w("")

        # Ausgehend
        w("**Ausgehend (Importe aus anderen Workspace-Crates)**")
        w("")
        outs = {d: v for (s, d), v in edges.items() if s == c.name}
        if outs:
            w("| Ziel | prod | dev | Dateien (prod) | Symbole (prod) | Symbole (nur dev) |")
            w("|---|---:|---:|---:|---|---|")
            for d, v in sorted(outs.items(), key=lambda kv: (-kv[1]["sites_prod"], kv[0])):
                devonly = Counter({k: n for k, n in v["dev"].items() if k not in v["prod"]})
                w(f"| `{d}` | {v['sites_prod']} | {v['sites_dev']} | {len(v['files_prod'])} | "
                  f"{fmt_syms(v['prod'], args.top_symbols) or '–'} | {fmt_syms(devonly, 5) or '–'} |")
        else:
            w("Keine.")
        w("")

        # Pro Komponente
        w("**Pro Komponente**")
        w("")
        comp_map: Dict[str, Dict[str, Dict[str, Counter]]] = defaultdict(lambda: defaultdict(lambda: defaultdict(Counter)))
        comp_loc: Dict[str, int] = Counter()
        for s in scans:
            comp_loc[an.comp_at_depth(s.comp)] += s.loc
        for i in an.imps:
            if i.src == c.name:
                comp_map[an.comp_at_depth(i.comp)][i.dst][i.cls][i.symbol] += 1
        if comp_map:
            for comp in sorted(comp_map, key=lambda x: (x.startswith(("tests/", "benches/", "examples/", "fuzz/")), x)):
                w(f"- `{comp}` ({comp_loc.get(comp, 0)} Zeilen)")
                for d in sorted(comp_map[comp]):
                    pr = comp_map[comp][d].get("prod", Counter())
                    dv = comp_map[comp][d].get("dev", Counter())
                    segs = []
                    if pr:
                        segs.append("prod: " + fmt_syms(pr, args.top_symbols))
                    if dv:
                        segs.append("dev: " + fmt_syms(dv, 4))
                    w(f"  - → `{d}`: " + " · ".join(segs))
        else:
            w("Keine Importe aus anderen Workspace-Crates.")
        w("")

        # Öffentliche Schnittstelle
        w("**Öffentliche Schnittstelle** (erreichbar über `pub mod`-Kette oder `pub use`)")
        w("")
        exposed = [(s, it) for s in scans for it in s.items if it.status in ("pub", "reexp") and it.kind != "impl"]
        intern_n = sum(1 for s in scans for it in s.items if it.status == "intern" and it.kind != "impl")
        if exposed:
            by_comp: Dict[str, List[Tuple[FileScan, Item]]] = defaultdict(list)
            for s, it in exposed:
                by_comp[an.comp_at_depth(s.comp)].append((s, it))
            shown = 0
            for comp in sorted(by_comp):
                w(f"- `{comp}`")
                kinds = Counter(it.kind for _, it in by_comp[comp])
                w("  - " + ", ".join(f"{n} {k}" for k, n in sorted(kinds.items())))
                for s, it in by_comp[comp]:
                    if shown >= args.max_items:
                        break
                    shown += 1
                    flag = "" if it.status == "pub" else " *(re-exportiert)*"
                    leak = f" ⚠ fremd: {', '.join(f'`{x}`' for x in sorted(it.foreign))}" if it.foreign else ""
                    meth = f" · {len(it.methods)} Methoden" if it.methods and it.kind != "trait" else (f" · Methoden: {', '.join(it.methods[:6])}{'…' if len(it.methods) > 6 else ''}" if it.methods else "")
                    w(f"    - `{it.kind} {it.name}`{flag}{meth}{leak}")
            if len(exposed) > args.max_items:
                w(f"  - … {len(exposed) - args.max_items} weitere Items nicht aufgelistet (`--max-items`)")
        else:
            w("Keine pub-Items erkannt (Bibliothek ohne lib.rs, nur Binary, oder alles intern).")
        if intern_n:
            w(f"- Zusätzlich {intern_n} `pub`-Items in privaten Modulen ohne Re-Export (nicht Teil der echten API).")
        reexps = [(s.comp, n, p) for s in scans if s.scope == "src" for n, p in s.reexports_internal]
        amap = an.alias_map(c)
        ext_reexps = [(s.comp, "::".join(i["path"])) for s in scans if s.scope == "src" for i in s.imports
                      if i["pub"] and i["top"] and i["path"][0] in amap]
        if reexps:
            w(f"- Interne Re-Exporte (`pub use`): " + ", ".join(f"`{n}`" for _, n, _ in reexps[:25]) + (" …" if len(reexps) > 25 else ""))
        if ext_reexps:
            w("- ⚠ Re-Exporte **fremder Crates**: " + ", ".join(f"`{p}`" for _, p in ext_reexps[:20]))
        w("")

        # Eingehend
        w("**Eingehend (wer nutzt dieses Crate)**")
        w("")
        inc = inbound.get(c.name, {})
        if inc:
            w("| Nutzer | Symbole prod | Symbole dev |")
            w("|---|---|---|")
            for src_name in sorted(inc, key=lambda x: (-sum(n for (sy, cl), n in inc[x].items() if cl == "prod"), x)):
                pr = Counter({sy: n for (sy, cl), n in inc[src_name].items() if cl == "prod"})
                dv = Counter({sy: n for (sy, cl), n in inc[src_name].items() if cl == "dev" and sy not in pr})
                w(f"| `{src_name}` | {fmt_syms(pr, args.top_symbols) or '–'} | {fmt_syms(dv, 4) or '–'} |")
        else:
            w("Wird von keinem anderen analysierten Workspace-Crate importiert.")
        w("")
        used_names: Set[str] = set()
        glob_used = False
        for src_name, cnt in inc.items():
            for (sy, cl), n in cnt.items():
                if sy.endswith("*"):
                    glob_used = True
                used_names.add(sy.split("::")[-1])
        unused = [it for s in scans for it in s.items
                  if it.status in ("pub", "reexp") and it.kind not in ("impl", "mod") and it.name not in used_names]
        if exposed and not glob_used and unused:
            w(f"**Nach außen ungenutzte öffentliche Items (Heuristik, {len(unused)} von {len(exposed)})**: "
              + ", ".join(f"`{it.name}`" for it in unused[:30]) + (" …" if len(unused) > 30 else ""))
            w("")
        elif glob_used and exposed:
            w("*Hinweis: Mindestens ein Crate importiert per Glob (`*`); ungenutzte Items sind deshalb nicht bestimmbar.*")
            w("")

        notes = []
        if c.name in in_cycle:
            notes.append("Teil eines Code-Zyklus (siehe 2.1)")
        for r in rows:
            if r[0] == c.name:
                notes.append(f"{r[1]}: {r[3]}")
        if notes:
            w("**Auffälligkeiten**")
            w("")
            for n in notes:
                w(f"- {n}")
            w("")

    data = dict(
        crates={c.name: dict(loc=c.loc, ring=c.ring,
                             deps=[dict(package=d.package, kind=d.kind, optional=d.optional, target=d.target, features=d.features)
                                   for d in c.deps if d.internal])
                for c in crates},
        edges=[dict(src=s, dst=d, prod_sites=v["sites_prod"], dev_sites=v["sites_dev"],
                    prod_symbols=dict(v["prod"]), dev_symbols=dict(v["dev"])) for (s, d), v in sorted(edges.items())],
        api_leaks=[dict(crate=r[0], comp=r[1], item=r[2], foreign=r[3], at=f"{r[4]}:{r[5]}") for r in leak_rows],
        cycles_prod=cyc_prod, cycles_cargo=cyc_cargo,
        weak_edges=[dict(src=s, dst=d, symbols=list(v["prod"]), sites=v["sites_prod"]) for s, d, v in weak],
    )
    return "\n".join(L) + "\n", data


# ---------------------------------------------------------------------------
def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description="Erzeugt einen Markdown-Bericht über Kopplungen zwischen Rust-Workspace-Crates.")
    ap.add_argument("root", nargs="?", default=".", help="Workspace-Wurzel (Standard: aktuelles Verzeichnis)")
    ap.add_argument("-o", "--out", default="crate-coupling-report.md", help="Ausgabedatei (Markdown)")
    ap.add_argument("--depth", type=int, default=1, help="Komponenten-Tiefe: 1 = Top-Level-Module, 2 = Untermodule …")
    ap.add_argument("--top-symbols", type=int, default=10, help="Max. Symbole pro Zelle")
    ap.add_argument("--max-items", type=int, default=80, help="Max. aufgelistete pub-Items pro Crate")
    ap.add_argument("--only", default="", help="Nur diese Crates detailliert analysieren (kommagetrennt)")
    ap.add_argument("--json", default="", help="Zusätzlich Rohdaten als JSON schreiben")
    ap.add_argument("--mermaid", action="store_true", help="Abhängigkeitsgraph als Mermaid-Block anhängen")
    args = ap.parse_args(argv)

    root = Path(args.root).resolve()
    crates = discover_crates(root)
    if not crates:
        print("Keine Crates gefunden (Cargo.toml mit [package] fehlt?).", file=sys.stderr)
        return 2
    only = {x.strip() for x in args.only.split(",") if x.strip()} or None
    an = Analysis(root, crates, max(1, args.depth))
    an.run(only)
    md, data = render(an, args)
    Path(args.out).write_text(md, encoding="utf-8")
    if args.json:
        Path(args.json).write_text(json.dumps(data, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"{len(an.scans)} Crates analysiert, {len(an.imps)} Importstellen → {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
