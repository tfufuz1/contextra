#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""extract_interfaces.py — erzeugt je Crate eine feingranulare Schnittstellenspezifikation aus dem Quellcode.

Aufruf:  python3 -I extract_interfaces.py <repo-root> <ausgabeverzeichnis>

Eigenschaften
- Nur Standardbibliothek (Python >= 3.11, tomllib).
- Regelbasiertes Parsen: zeilenweise Erkennung von Items, Impl-/Trait-/Struct-/Enum-Kontexten
  mit Klammertiefe. Keine Makroexpansion, keine Typauflösung, keine cfg-Auswertung.
- Jede Angabe trägt Datei:Zeile.
- Gegenprobe: Grep-Zählung aller `pub`-Deklarationen gegen die geparsten. Die Abdeckung
  steht je Crate im Bericht. Abweichungen sind nicht stillschweigend akzeptiert.
- Erzeugt je Crate: Markdown (Mensch) und JSON (Maschine), dazu INDEX.md und README.md.
"""
import collections
import datetime
import json
import os
import pathlib
import re
import sys

try:
    import tomllib
except ImportError:  # pragma: no cover
    sys.exit("Python >= 3.11 (tomllib) erforderlich")

if len(sys.argv) != 3:
    sys.exit(__doc__)

ROOT = pathlib.Path(sys.argv[1]).resolve()
OUT = pathlib.Path(sys.argv[2]).resolve()
SKIP_DIRS = {"target", ".git", "node_modules", "__pycache__", ".venv"}

# ---------------------------------------------------------------- Regeln
STR_RE = re.compile(r'"(?:\\.|[^"\\])*"')
CHR_RE = re.compile(r"'(?:\\.|[^'\\])'")
ITEM_RE = re.compile(
    r"^(?P<vis>pub(?:\((?P<scope>[^)]*)\))?\s+)?"
    r"(?:default\s+)?(?:async\s+)?(?:unsafe\s+)?(?:extern\s+\"[^\"]*\"\s+)?"
    r"(?:const\s+)?(?P<kind>fn|struct|enum|trait|type|const|static|mod|use|union)\b(?P<rest>.*)$"
)
IMPL_RE = re.compile(r"^(?:unsafe\s+)?impl\b\s*(?P<gen><.*?>)?\s*(?P<body>[^{]+)")
MACRO_RE = re.compile(r"^macro_rules!\s*(?P<name>\w+)")
FIELD_RE = re.compile(r"^(?P<vis>pub(?:\([^)]*\))?\s+)?(?P<name>\w+)\s*:(?!:)\s*(?P<ty>[^,]+?)\s*,?$")
VARIANT_RE = re.compile(r"^(?P<name>[A-Z]\w*)\s*(?P<tail>[({=,].*)?$")
GREP_PUB = re.compile(r"^\s*pub\s+(?:(?:async|unsafe|const|default)\s+)*(?:fn|struct|enum|trait|type|const|static|mod|union)\b")

# Befundzähler (nur Produktionscode, nicht innerhalb #[cfg(test)])
PANIC_PATTERNS = {
    "unwrap": re.compile(r"\.unwrap\(\)"),
    "expect": re.compile(r"\.expect\("),
    "panic_macro": re.compile(r"\bpanic!\("),
    "todo_unimpl": re.compile(r"\b(?:todo|unimplemented|unreachable)!\("),
    "index_bracket": re.compile(r"\w\[[^\]\"]+\]"),
}

ROLE = {
    "contextra-storage": "Speicherkern (Kern, verschachtelt unter crates/contextra-storage)",
    "contextra-storage-ports": "Speicherkern: normative Ports",
    "contextra-storage-fs": "Speicherkern: Fs-Adapter",
    "contextra-crypto-ref": "Speicherkern: Krypto-Referenz",
    "contextra-keys-file": "Speicherkern: Schlüssel-Provider (Datei)",
    "contextra-keys-cohort": "Speicherkern: Schlüssel-Provider (Cohort)",
    "contextra-storage-bridge": "Speicherkern: Platzhalter-Brücke",
    "contextra-store": "Altstand LSM (bleibt bis Cutover)",
    "contextra-erasure": "Löschkern alt (Orchestrator-Ziel laut R10)",
    "contextra-crypto": "Krypto Legacy (Passphrase, AES-GCM-SIV)",
    "contextra-privacy": "Datenschutz-Egress und Surrogate",
    "contextra-vault": "RAM-Staging (excluded)",
    "contextra-prompt-guard": "Prompt-Injection-Schutz",
    "contextra-ports": "Domänen-Ports",
    "contextra-types": "Gemeinsame Typen",
    "contextra-ids": "Identifikatoren",
    "contextra-env": "Umgebung (Clock, Rng, FS-Abstraktion)",
    "contextra-durable-fs": "Dauerhafte Dateioperationen (alt)",
    "contextra": "Fassade",
    "contextra-mcp": "MCP-Server",
    "contextra-py": "Python-Bindings",
    "contextra-wire": "Wire-Protokoll",
    "contextra-db": "Datenbank-Fassade (Hauptkonsument Store)",
    "contextra-engine": "Engine (Re-Export db)",
    "contextra-mvcc": "MVCC-Hilfen",
    "contextra-vector": "Vektorindex",
    "contextra-text": "Volltext",
    "contextra-graph": "Wissensgraph",
    "contextra-cognition": "Konsolidierung und Gedächtnis",
    "contextra-rank": "Ranking und Fusion",
    "contextra-ingest": "Dokumentenerfassung",
    "contextra-agent": "Agenten-Laufzeit",
    "contextra-router": "Modell-Routing",
    "contextra-kvcache": "KV-Cache",
    "contextra-adapt": "Adaptive Regelung",
    "contextra-sandbox": "Sandbox (Wasm)",
    "contextra-checkpoint": "Checkpoints",
    "contextra-infer-onnx": "Inferenz ONNX",
    "contextra-infer-candle": "Inferenz Candle",
    "contextra-infer-ollama": "Inferenz Ollama",
    "contextra-simd": "SIMD-Kerne",
    "contextra-sys": "Plattform-nahe Funktionen",
    "contextra-testkit": "Testhilfen",
    "contextra-bench": "Benchmarks",
}


def clean_lines(raw_lines):
    """Entfernt Literale und Kommentare; liefert bereinigte Zeilen (Blockkommentare übergreifend)."""
    out, in_block = [], False
    for raw in raw_lines:
        s = STR_RE.sub('""', raw)
        s = CHR_RE.sub("''", s)
        res, i = "", 0
        while i < len(s):
            if in_block:
                j = s.find("*/", i)
                if j < 0:
                    i = len(s)
                    break
                in_block, i = False, j + 2
                continue
            if s.startswith("/*", i):
                in_block, i = True, i + 2
                continue
            if s.startswith("//", i):
                break
            res += s[i]
            i += 1
        out.append(res)
    return out


class Ctx:
    __slots__ = ("kind", "name", "depth", "attrs", "test", "rec")

    def __init__(self, kind, name, depth, attrs, test, rec=None):
        self.kind, self.name, self.depth, self.attrs, self.test, self.rec = kind, name, depth, attrs, test, rec


def collect_signature(clean, idx):
    parts = []
    for j in range(idx, min(idx + 14, len(clean))):
        t = clean[j].strip()
        if not t:
            continue
        cut = min([p for p in (t.find("{"), t.find(";")) if p >= 0] or [len(t)])
        parts.append(t[:cut])
        if cut < len(t):
            break
    sig = re.sub(r"\s+", " ", " ".join(parts)).strip()
    return sig.rstrip("{; ").strip()


def vis_of(m):
    v = m.group("vis")
    if not v:
        return "private"
    return "pub" if "(" not in v else "pub(restricted)"


def parse_file(path, rel, file_mod):
    raw_lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    clean = clean_lines(raw_lines)
    res = {"items": [], "impls": [], "uses": [], "macros": [], "crate_doc": [],
           "counts": collections.Counter(), "all_pub_decl": 0, "grep_pub_decl": 0}
    depth, stack = 0, []
    pend_doc, pend_attr = [], []

    for idx, raw in enumerate(raw_lines):
        ln = idx + 1
        s = raw.strip()
        grep_hit = bool(GREP_PUB.match(raw))
        if grep_hit:
            res["grep_pub_decl"] += 1
        if s.startswith("//!"):
            if depth == 0:
                res["crate_doc"].append(s[3:].strip())
            continue
        if s.startswith("///"):
            pend_doc.append(s[3:].strip())
            continue
        if s.startswith("#["):
            pend_attr.append(s)
            continue

        t = clean[idx].strip()
        ctx = stack[-1] if stack else None
        in_test = any(c.test for c in stack)
        mod_path = list(file_mod) + [c.name for c in stack if c.kind == "mod"]

        if t and not in_test:
            for key, pat in PANIC_PATTERNS.items():
                res["counts"][key] += len(pat.findall(t))
            if re.search(r"\bunsafe\b", t):
                res["counts"]["unsafe_tokens"] += 1
            if any("deprecated" in a for a in pend_attr):
                res["counts"]["deprecated_attr"] += 1
        if t and any("cfg(feature" in a for a in pend_attr):
            res["counts"]["cfg_feature_gates"] += 1

        at_module = ctx is None or (ctx.kind == "mod" and depth == ctx.depth)
        at_member = ctx is not None and ctx.kind in ("impl", "trait", "struct", "enum") and depth == ctx.depth
        header = None  # (kind, name, rec)

        if t:
            m = ITEM_RE.match(t)
            if at_module:
                im = IMPL_RE.match(t)
                mac = MACRO_RE.match(t)
                if im:
                    body = re.sub(r"\s+where\b.*$", "", im.group("body")).strip()
                    if " for " in body:
                        trait_part, type_part = body.split(" for ", 1)
                    else:
                        trait_part, type_part = None, body
                    tname = re.split(r"[<\s]", type_part.strip().lstrip("&"))[0].split("::")[-1]
                    rec = {"type": tname, "trait": trait_part.strip() if trait_part else None,
                           "generics": (im.group("gen") or "").strip(), "signature": collect_signature(clean, idx),
                           "file": rel, "line": ln, "methods": [], "in_test": in_test,
                           "doc": pend_doc[:], "attrs": pend_attr[:]}
                    res["impls"].append(rec)
                    header = ("impl", tname, rec)
                elif mac:
                    res["macros"].append({"name": mac.group("name"), "file": rel, "line": ln,
                                          "exported": any("macro_export" in a for a in pend_attr),
                                          "doc": pend_doc[:], "in_test": in_test})
                elif m:
                    kind = m.group("kind")
                    vis = vis_of(m)
                    if kind == "use":
                        res["uses"].append({"signature": collect_signature(clean, idx), "vis": vis,
                                            "file": rel, "line": ln, "in_test": in_test,
                                            "module": "::".join(mod_path) or "crate"})
                    else:
                        name_m = re.match(r"\s*(?P<name>\w+)", m.group("rest"))
                        name = name_m.group("name") if name_m else "?"
                        if kind == "mod" and name_m and name_m.group("name") == "" :
                            name = "?"
                        rec = {"kind": kind, "name": name, "vis": vis, "module": "::".join(mod_path) or "crate",
                               "signature": collect_signature(clean, idx), "file": rel, "line": ln,
                               "doc": pend_doc[:], "attrs": pend_attr[:], "in_test": in_test,
                               "methods": [], "fields": [], "variants": []}
                        res["items"].append(rec)
                        if vis == "pub":
                            res["all_pub_decl"] += 1  # gezählt wie die Grep-Gegenprobe
                        if kind in ("struct", "enum", "trait", "mod", "fn"):
                            is_test_mod = kind == "mod" and any("cfg(test)" in a for a in pend_attr)
                            header = (kind, name, rec)
                            if kind == "mod" and t.endswith(";"):
                                header = None
                            if is_test_mod:
                                header = ("mod", name, rec)
                                rec["test_module"] = True
            elif ctx is not None and ctx.kind in ("impl", "trait") and at_member:
                m = ITEM_RE.match(t)
                if m:
                    kind = m.group("kind")
                    if kind == "use":
                        pass
                    else:
                        name_m = re.match(r"\s*(?P<name>\w+)", m.group("rest"))
                        name = name_m.group("name") if name_m else "?"
                        vis = "pub" if ctx.kind == "trait" else vis_of(m)
                        meth = {"name": name, "kind": kind, "vis": vis, "signature": collect_signature(clean, idx),
                                "file": rel, "line": ln, "doc": pend_doc[:], "attrs": pend_attr[:],
                                "in_test": in_test, "via_trait": ctx.rec.get("trait") if ctx.kind == "impl" else ctx.name}
                        ctx.rec["methods"].append(meth)
                        if ctx.kind == "impl" and vis == "pub":
                            res["all_pub_decl"] += 1  # gezählt wie die Grep-Gegenprobe
                        if kind == "fn" and t.endswith(";") is False:
                            header = ("fn", name, None)
            elif ctx is not None and ctx.kind in ("struct", "enum") and at_member:
                if ctx.kind == "struct":
                    fm = FIELD_RE.match(t.rstrip(","))
                    if fm:
                        ctx.rec["fields"].append({"name": fm.group("name"), "type": fm.group("ty").strip(),
                                                  "vis": "pub" if fm.group("vis") else "private",
                                                  "file": rel, "line": ln})
                else:
                    vm = VARIANT_RE.match(t.rstrip(","))
                    if vm and vm.group("name") and not t.startswith("#"):
                        ctx.rec["variants"].append({"name": vm.group("name"), "shape": (vm.group("tail") or "").strip(),
                                                    "file": rel, "line": ln})

        # Klammerzählung mit Kontext-Stack
        pushed_header = False
        for ch in clean[idx]:
            if ch == "{":
                depth += 1
                if header and not pushed_header:
                    kind, name, rec = header
                    test = bool(stack and stack[-1].test) or (kind == "mod" and rec is not None and rec.get("test_module", False))
                    stack.append(Ctx(kind, name, depth, pend_attr[:], test, rec))
                    pushed_header = True
                else:
                    stack.append(Ctx("block", None, depth, [], bool(stack and stack[-1].test)))
            elif ch == "}":
                depth -= 1
                while stack and stack[-1].depth > depth:
                    stack.pop()

        # Attribute und Doku gelten nur für das unmittelbar folgende Element
        pend_doc, pend_attr = [], []
    return res


def parse_manifest(path):
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    pkg = data.get("package", {})
    deps = {}
    for section, kind in (("dependencies", "normal"), ("dev-dependencies", "dev"), ("build-dependencies", "build")):
        for name, spec in (data.get(section) or {}).items():
            how = "workspace" if isinstance(spec, dict) and spec.get("workspace") else \
                  "path" if isinstance(spec, dict) and "path" in spec else "registry"
            deps[name] = {"kind": kind, "source": how,
                          "optional": bool(isinstance(spec, dict) and spec.get("optional"))}
    feats = {k: v for k, v in (data.get("features") or {}).items()}
    return {"name": pkg.get("name", path.parent.name), "version": pkg.get("version", "?") if isinstance(pkg.get("version"), str) else "workspace",
            "deps": deps, "features": feats, "lib": (data.get("lib") or {}).get("path")}


def find_packages():
    pkgs = []
    for dirpath, dirnames, filenames in os.walk(ROOT):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        if "Cargo.toml" in filenames:
            cargo = pathlib.Path(dirpath) / "Cargo.toml"
            try:
                head = cargo.read_text(encoding="utf-8")
            except OSError:
                continue
            if re.search(r"^\[package\]", head, re.M):
                pkgs.append(cargo)
    return sorted(pkgs)


def module_of_file(src_root, path):
    rel = path.relative_to(src_root).with_suffix("")
    parts = list(rel.parts)
    if parts and parts[-1] in ("mod",):
        parts = parts[:-1]
    if parts in (["lib"], ["main"], []):
        return []
    if parts and parts[0] == "bin":
        return ["bin:" + "/".join(parts[1:])]
    return parts


def analyse_crate(cargo):
    crate_dir = cargo.parent
    manifest = parse_manifest(cargo)
    src = crate_dir / "src"
    result = {"name": manifest["name"], "version": manifest["version"], "path": str(crate_dir.relative_to(ROOT)),
              "role": ROLE.get(manifest["name"], "unklassifiziert"), "features": manifest["features"],
              "deps": manifest["deps"], "lib": manifest["lib"], "files": [], "items": [], "impls": [], "uses": [],
              "macros": [], "crate_doc": [], "counts": collections.Counter(), "grep_pub_decl": 0,
              "all_pub_decl": 0, "has_src": src.is_dir(), "unreachable_note": None}
    if not src.is_dir():
        return result
    for f in sorted(src.rglob("*.rs")):
        rel = str(f.relative_to(ROOT))
        mod = module_of_file(src, f)
        parsed = parse_file(f, rel, mod)
        result["files"].append({"file": rel, "module": "::".join(mod) or "crate",
                                "lines": sum(1 for _ in f.open(encoding="utf-8", errors="replace"))})
        result["items"] += parsed["items"]
        result["impls"] += parsed["impls"]
        result["uses"] += parsed["uses"]
        result["macros"] += parsed["macros"]
        result["counts"].update(parsed["counts"])
        result["grep_pub_decl"] += parsed["grep_pub_decl"]
        result["all_pub_decl"] += parsed["all_pub_decl"]
        if not result["crate_doc"] and f.name in ("lib.rs", "main.rs") and f.parent == src:
            result["crate_doc"] = parsed["crate_doc"]
    # Methoden an Typen hängen (gleicher Crate-Name, erstes Vorkommen)
    by_name = {}
    for it in result["items"]:
        if it["kind"] in ("struct", "enum", "trait", "union"):
            by_name.setdefault(it["name"], it)
    unattached = []
    for im in result["impls"]:
        tgt = by_name.get(im["type"])
        if tgt is not None:
            tgt.setdefault("impl_blocks", []).append({k: im[k] for k in ("trait", "generics", "signature", "file", "line", "in_test")})
            tgt["methods"] += im["methods"]
        else:
            unattached.append(im)
    result["unattached_impls"] = unattached
    return result


def public_api(res):
    """Öffentliche Oberfläche: pub-Items außerhalb von Testmodulen."""
    items = [i for i in res["items"] if i["vis"] == "pub" and not i["in_test"] and not i.get("test_module")]
    items = [i for i in items if i["kind"] != "use"]
    uses = [u for u in res["uses"] if u["vis"] == "pub" and not u["in_test"]]
    macros = [m for m in res["macros"] if m["exported"] and not m["in_test"]]
    for it in items:
        it["methods"] = [m for m in it["methods"] if m["vis"] == "pub" and not m["in_test"]]
        it["fields"] = [f for f in it["fields"] if f["vis"] == "pub"]
    return items, uses, macros


def md_escape(s):
    return s.replace("|", "\\|")


def render_crate(res, deps_used_by):
    items, uses, macros = public_api(res)
    name = res["name"]
    out = []
    w = out.append
    w(f"# Schnittstellenspezifikation: `{name}`\n")
    w("> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. "
      "Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.\n")
    w("## 1. Kopf\n")
    w(f"| Feld | Wert |\n|---|---|")
    w(f"| Name | `{name}` |")
    w(f"| Version | {res['version']} |")
    w(f"| Pfad | `{res['path']}` |")
    w(f"| Rolle | {res['role']} |")
    w(f"| Bibliothekspfad | {res['lib'] or 'src/lib.rs (Standard)'} |")
    w("")
    if res["crate_doc"]:
        w("**Crate-Dokumentation (//!):**\n")
        for line in res["crate_doc"][:12]:
            w(f"> {line}" if line else ">")
        w("")
    if not res["has_src"]:
        w("*Kein `src/`-Verzeichnis: keine Rust-Schnittstelle zu extrahieren (Fuzz-, Stub- oder Paketrumpf).*\n")
        return "\n".join(out)

    w("## 2. Abhängigkeiten\n")
    w("| Name | Art | Quelle | optional | verwendet von (normal/dev) |")
    w("|---|---|---|---|---|")
    for dn, d in sorted(res["deps"].items()):
        used = deps_used_by.get(dn, {})
        w(f"| `{dn}` | {d['kind']} | {d['source']} | {'ja' if d['optional'] else 'nein'} | "
          f"{'' if not used else ', '.join(sorted(used))} |")
    if not res["deps"]:
        w("| – | – | – | – | – |")
    used_by_me = sorted(deps_used_by.get("__self__", {}).keys())
    w(f"\n**Verwendet von (Crate-Ebene):** {', '.join('`'+u+'`' for u in used_by_me) or '–'}\n")

    w("## 3. Features\n")
    if res["features"]:
        w("| Feature | Aktiviert |\n|---|---|")
        for f, members in sorted(res["features"].items()):
            w(f"| `{f}` | {', '.join('`'+m+'`' for m in members) or '–'} |")
    else:
        w("Keine Features deklariert.")
    w("")

    w("## 4. Metriken (Quellcode)\n")
    c = res["counts"]
    kinds = collections.Counter(i["kind"] for i in items)
    w("| Kennzahl | Wert |\n|---|---|")
    w(f"| Quelldateien | {len(res['files'])} |")
    w(f"| Quellzeilen | {sum(f['lines'] for f in res['files'])} |")
    w(f"| Öffentliche Items (Modul-Ebene) | {len(items)} |")
    for k in ("fn", "struct", "enum", "trait", "type", "const", "static", "mod", "union"):
        if kinds.get(k):
            w(f"| davon `{k}` | {kinds[k]} |")
    w(f"| Öffentliche Makros (`#[macro_export]`) | {len(macros)} |")
    w(f"| Öffentliche Re-Exports (`pub use`) | {len(uses)} |")
    w(f"| `unsafe`-Vorkommen (Produktionscode) | {c.get('unsafe_tokens', 0)} |")
    w(f"| `.unwrap()` (Produktionscode) | {c.get('unwrap', 0)} |")
    w(f"| `.expect(` (Produktionscode) | {c.get('expect', 0)} |")
    w(f"| `panic!` (Produktionscode) | {c.get('panic_macro', 0)} |")
    w(f"| `todo!/unimplemented!/unreachable!` | {c.get('todo_unimpl', 0)} |")
    w(f"| Indexierungen `x[…]` (grob, Produktionscode) | {c.get('index_bracket', 0)} |")
    w(f"| `#[deprecated]` | {c.get('deprecated_attr', 0)} |")
    w(f"| `#[cfg(feature…)]`-Gates | {c.get('cfg_feature_gates', 0)} |")
    cover = (res["all_pub_decl"] / res["grep_pub_decl"] * 100) if res["grep_pub_decl"] else 100.0
    w(f"| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | {res['all_pub_decl']} / {res['grep_pub_decl']} = {cover:.0f} % |")
    w("")

    w("## 5. Modulbaum und Dateien\n")
    w("| Modul | Datei | Zeilen |\n|---|---|---|")
    for f in res["files"]:
        w(f"| `{f['module']}` | `{f['file']}` | {f['lines']} |")
    w("")

    if uses:
        w("## 6. Re-Exports (`pub use`)\n")
        for u in sorted(uses, key=lambda x: (x["file"], x["line"])):
            w(f"- `{u['signature']}` — `{u['file']}:{u['line']}` (Modul `{u['module']}`)")
        w("")

    if macros:
        w("## 7. Öffentliche Makros\n")
        for m in macros:
            w(f"- `macro_rules! {m['name']}` — `{m['file']}:{m['line']}`")
            for d in m["doc"][:3]:
                if d:
                    w(f"  - {d}")
        w("")

    w("## 8. Öffentliche Schnittstelle je Item\n")
    by_mod = collections.defaultdict(list)
    for it in items:
        by_mod[it["module"]].append(it)
    order = {"mod": 0, "struct": 1, "enum": 2, "trait": 3, "fn": 4, "type": 5, "const": 6, "static": 7, "union": 8}
    for mod in sorted(by_mod):
        w(f"### Modul `{mod}`\n")
        for it in sorted(by_mod[mod], key=lambda x: (order.get(x["kind"], 9), x["name"])):
            w(f"#### `{it['kind']} {it['name']}`")
            w(f"- Ort: `{it['file']}:{it['line']}`")
            w("```rust")
            w(it["signature"])
            w("```")
            if it["doc"]:
                w("Dokumentation: " + " ".join(d for d in it["doc"] if d)[:600])
            if any("deprecated" in a for a in it["attrs"]):
                w("- **deprecated**")
            if any("cfg(feature" in a for a in it["attrs"]):
                cfg = [a for a in it["attrs"] if "cfg(feature" in a]
                w(f"- Gate: `{cfg[0]}`")
            if it.get("impl_blocks"):
                for ib in it["impl_blocks"]:
                    tr = f" (Trait `{ib['trait']}`)" if ib["trait"] else ""
                    w(f"- Impl-Block{tr} `{ib['file']}:{ib['line']}`")
            if it["fields"]:
                w("\nFelder:\n\n| Feld | Typ | Ort |\n|---|---|---|")
                for f in it["fields"]:
                    w(f"| `{f['name']}` | `{md_escape(f['type'])}` | `{f['file']}:{f['line']}` |")
            if it["variants"]:
                w("\nVarianten:\n")
                for v in it["variants"]:
                    w(f"- `{v['name']}{v['shape']}` — `{v['file']}:{v['line']}`")
            meths = [m for m in it["methods"] if m["kind"] == "fn"]
            consts = [m for m in it["methods"] if m["kind"] != "fn"]
            if meths:
                w("\nMethoden / Assoziierte Funktionen:\n")
                w("| Signatur | Ort | Doku |\n|---|---|---|")
                for m in meths:
                    d = " ".join(x for x in m["doc"] if x)[:160]
                    w(f"| `{md_escape(m['signature'])}` | `{m['file']}:{m['line']}` | {md_escape(d)} |")
            if consts:
                w("\nAssoziierte Konstanten/Typen:\n")
                for m in consts:
                    w(f"- `{m['signature']}` — `{m['file']}:{m['line']}`")
            w("")
    if res["unattached_impls"]:
        w("## 9. Impl-Blöcke ohne Typ im selben Crate\n")
        for im in res["unattached_impls"]:
            if im["in_test"]:
                continue
            pub_m = [m for m in im["methods"] if m["vis"] == "pub"]
            if not pub_m and not im["trait"]:
                continue
            w(f"- `{im['signature'][:160]}` — `{im['file']}:{im['line']}` ({len(pub_m)} pub Methoden)")
        w("")
    return "\n".join(out)


def main():
    cargos = find_packages()
    results = []
    for cargo in cargos:
        try:
            results.append(analyse_crate(cargo))
        except Exception as exc:  # Befund statt Abbruch
            results.append({"name": cargo.parent.name, "error": repr(exc), "path": str(cargo.parent.relative_to(ROOT))})

    names = {r["name"] for r in results if "name" in r}
    used_by = collections.defaultdict(dict)
    for r in results:
        if "deps" not in r:
            continue
        for dn, d in r["deps"].items():
            if dn in names:
                used_by[dn][r["name"]] = d["kind"]
                used_by[dn].setdefault("__dummy__", None)
    deps_used_by = {}
    for r in results:
        if "name" not in r:
            continue
        ub = {}
        for consumer in results:
            if consumer.get("deps") and r["name"] in consumer["deps"]:
                ub[consumer["name"]] = consumer["deps"][r["name"]]["kind"]
        deps_used_by[r["name"]] = {k: v for k, v in ub.items()}
    # Pro Abhängigkeitszeile: Konsumenten; Crate-Ebene: Konsumenten dieses Crates
    for r in results:
        if "name" not in r:
            continue
        per_dep = {}
        for dn in r.get("deps", {}):
            if dn in names:
                per_dep[dn] = deps_used_by.get(dn, {})
        ub_self = deps_used_by.get(r["name"], {})
        per_dep["__self__"] = ub_self

    (OUT / "crates").mkdir(parents=True, exist_ok=True)
    (OUT / "api").mkdir(parents=True, exist_ok=True)
    index_rows = []
    for r in results:
        if "error" in r:
            index_rows.append(f"| `{r['path']}` | FEHLER | {r['error']} |")
            continue
        dub = {}
        for dn in r.get("deps", {}):
            if dn in names:
                dub[dn] = {k: v for k, v in deps_used_by.get(dn, {}).items()}
        dub["__self__"] = deps_used_by.get(r["name"], {})
        md = render_crate(r, dub)
        (OUT / "crates" / f"{r['name']}.md").write_text(md, encoding="utf-8")
        items, uses, macros = public_api(r)
        payload = {"name": r["name"], "version": r["version"], "path": r["path"], "role": r["role"],
                   "features": r["features"], "deps": r["deps"], "crate_doc": r["crate_doc"],
                   "files": r["files"], "counts": dict(r["counts"]),
                   "grep_pub_decl": r["grep_pub_decl"], "all_pub_decl": r["all_pub_decl"],
                   "public_items": items, "reexports": uses, "macros": macros,
                   "used_by": sorted(deps_used_by.get(r["name"], {}).keys())}
        (OUT / "api" / f"{r['name']}.json").write_text(json.dumps(payload, ensure_ascii=False, indent=1, default=str),
                                                       encoding="utf-8")
        cover = (r["all_pub_decl"] / r["grep_pub_decl"] * 100) if r["grep_pub_decl"] else 100.0
        pubcount = len(items)
        index_rows.append(f"| [`{r['name']}`](crates/{r['name']}.md) | {r['path']} | {r['role']} | "
                          f"{pubcount} | {len(uses)} | {r['counts'].get('unsafe_tokens', 0)} | "
                          f"{r['counts'].get('unwrap', 0) + r['counts'].get('expect', 0)} | "
                          f"{len(deps_used_by.get(r['name'], {}))} | {cover:.0f} % |")

    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    index = [f"# Schnittstellen-Index Contextra\n",
             f"Erzeugt: {stamp} · Quelle: `{ROOT}` · Crates mit Manifest: {len(cargos)}\n",
             "| Crate | Pfad | Rolle | Öff. Items | Re-Exports | unsafe | unwrap+expect | Konsumenten | Parser-Abdeckung |",
             "|---|---|---|---|---|---|---|---|---|"]
    index += index_rows
    (OUT / "INDEX.md").write_text("\n".join(index) + "\n", encoding="utf-8")

    readme = f"""# Methode und Grenzen

Erzeugt: {stamp}
Werkzeug: `tools/extract_interfaces.py` (nur Python-Standardbibliothek, Python >= 3.11)
Eingabe: `{ROOT}`

## Was extrahiert wird
- Alle Crates mit `[package]` in `Cargo.toml` (inkl. verschachtelter Pakete, Fuzz- und Benchmarkpakete).
- Modul-Ebene: `pub` Items (fn, struct, enum, trait, type, const, static, mod, union), `pub use`, `#[macro_export]`-Makros.
- Methoden und assoziierte Konstanten aus `impl`-Blöcken, Trait-Methoden aus `trait`-Körpern.
- Felder öffentlicher Strukturen, Varianten von Enums.
- Attribute (`deprecated`, `cfg(feature…)`), Dokumentation (`///`, `//!`), Ort (Datei:Zeile).
- Abhängigkeiten aus `Cargo.toml` mit Art, Quelle und Konsumentenliste.
- Feature-Tabelle, Befundzähler (`unsafe`, `unwrap`, `expect`, `panic!`, `todo!`, Indexierungen).

## Was nicht extrahiert wird
- Keine Makroexpansion (Items, die durch `macro_rules!` erzeugt werden, fehlen).
- Keine Typauflösung: Signaturen sind Text, Re-Exports werden nicht aufgelöst (`pub use x::Y` zeigt nur die Zeile).
- Keine `cfg`-Auswertung: alle Varianten eines Features werden gleichzeitig gelistet.
- Rohe Mehrzeilen-Strings (`r#"…"#`) über mehrere Zeilen werden nicht maskiert.
- Trait-Implementierungen werden dem Typ nur zugeordnet, wenn Typ und Impl im selben Crate liegen.

## Gegenprobe
Die Spalte „Parser-Abdeckung“ vergleicht die geparsten `pub`-Deklarationen mit einer Grep-Zählung
derselben Zeilen (`^\\s*pub\\s+(…)`). 100 % heißt Übereinstimmung der Zählung, nicht Vollständigkeit der
Semantik. Abweichungen weisen auf Mehrzeilen-Konstrukte oder Makros hin und sind im jeweiligen Bericht prüfbar.

## Aufruf
```
python3 -I tools/extract_interfaces.py <repo-root> <ausgabeverzeichnis>
```
`-I` isoliert den Interpreter vom Arbeitsverzeichnis (Schutz vor eingeschleusten Modulen).
"""
    (OUT / "README.md").write_text(readme, encoding="utf-8")
    print(f"crates={len(results)} ausgabe={OUT}")


if __name__ == "__main__":
    main()
