"""`ctxrs archgate` – SOLL-Architekturregeln (Spezifikation V1 §4.2) als Ratchet-Gate, ohne Cargo.

Regeln (nur für Crates mit zone = "core" laut data/soll.toml, sofern nicht anders angegeben):
  R1  Ringe nur abwärts (normal/build-Abhängigkeiten; gleicher Ring erlaubt)
  R2  Kern hängt nicht von Enterprise/Labor ab
  RD  Kern hängt von zu streichendem Crate (Information: Löschblockade, nicht Teil des Ratchets)
  R3  unsafe nur in unsafe_allowed; Crate-Wurzel ohne #![forbid(unsafe_code)] (R3b)
  R4  kein tokio unter [dependencies] in Ring 0..2
  R5  direkter Dateizugriff (std::fs, tokio::fs, File/OpenOptions) in Ring 1..3 außerhalb der Adapter
  R6  unwrap/expect/panic!/todo!/unimplemented!/unreachable! im Produktionscode (Zählung je Crate/Art)
  R7  Netzwerkcode/-abhängigkeiten außerhalb der erlaubten Crates
  W1  verbotene Außenaussagen (Wording-Lint) in Markdown und Rust-Doc-Kommentaren

Ratchet: Die Baseline (governance/arch_baseline.json) hält den Ist-Zustand fest und darf nur schrumpfen.
Neue oder größere Zähler => Exit 2. Zähler, die gesunken sind, werden mit `--shrink` in die Baseline übernommen.
"""
from __future__ import annotations

import json
import re
from collections import defaultdict
from pathlib import Path

from .lib import (ToolError, Workspace, load_data, mask_rust, md_table, read_text, rust_files, test_spans, in_spans,
                  walk_files, file_class, LineIndex)

PANICS = {
    "unwrap": re.compile(r"\.\s*unwrap\s*\(\s*\)"),
    "expect": re.compile(r"\.\s*expect\s*\("),
    "panic!": re.compile(r"(?<![\w])panic\s*!\s*[\(\[{]"),
    "todo!": re.compile(r"(?<![\w])todo\s*!\s*[\(\[{]"),
    "unimplemented!": re.compile(r"(?<![\w])unimplemented\s*!\s*[\(\[{]"),
    "unreachable!": re.compile(r"(?<![\w])unreachable\s*!\s*[\(\[{]"),
}
UNSAFE = re.compile(r"(?<![\w])unsafe\s*(fn\b|impl\b|trait\b|extern\b|\{)")
FORBID = re.compile(r"#!\s*\[\s*(?:forbid|deny)\s*\(\s*unsafe_code\s*\)\s*\]")
FS = re.compile(
    r"\b(?:std|tokio)::fs\b|(?<![\w:])fs::(?:read|write|remove_file|remove_dir|create_dir|rename|copy|File|OpenOptions|"
    r"metadata|read_dir|read_to_string|hard_link|set_permissions)\w*|\b(?:File|OpenOptions)::(?:open|create|new|options)\b")
RULE_TITLES = {
    "R1": "Ring-Aufwärtskante", "R2": "Kern → Enterprise/Labor", "R3": "unsafe außerhalb contextra-sys",
    "R3b": "Crate-Wurzel ohne forbid(unsafe_code)", "R4": "tokio in [dependencies] (Ring 0–2)",
    "R5": "Direkter Dateizugriff außerhalb Adapter", "R6": "Panic-Konstrukte im Produktionscode",
    "R7": "Netzwerkcode außerhalb erlaubter Crates", "W1": "Verbotene Außenaussage",
}


class Soll:
    def __init__(self):
        d = load_data("soll.toml")
        self.crates: dict[str, dict] = d["crate"]
        self.rules: dict = d["rules"]
        self.wording: list[str] = d["wording"]["forbidden"]

    def zone(self, n: str) -> str | None:
        return self.crates.get(n, {}).get("zone")

    def ring(self, n: str) -> int | None:
        return self.crates.get(n, {}).get("ring")


def collect(ws: Workspace, soll: Soll, only: set[str]) -> tuple[dict[str, int], dict[str, list[str]], list[dict]]:
    """-> (Zähler je Schlüssel, Details je Schlüssel, Blockaden)"""
    counts: dict[str, int] = defaultdict(int)
    detail: dict[str, list[str]] = defaultdict(list)
    blockers: list[dict] = []
    want = lambda r: not only or r in only

    def add(key: str, n: int = 1, d: str = "") -> None:
        counts[key] += n
        if d and len(detail[key]) < 5:
            detail[key].append(d)

    core = {n for n in ws.crates if soll.zone(n) == "core"}
    rules = soll.rules

    # --- Graphkanten ---------------------------------------------------------------
    for n, c in ws.crates.items():
        for d in c.deps:
            if d.kind == "dev" or d.pkg not in ws.crates or d.pkg == n:
                continue
            za, zb = soll.zone(n), soll.zone(d.pkg)
            if za == "core":
                if zb in ("enterprise", "labs") and want("R2"):
                    add(f"R2|{n}->{d.pkg}", 1, f"{n} → {d.pkg} ({zb})")
                if zb == "delete":
                    blockers.append({"from": n, "to": d.pkg, "kind": d.kind + (",opt" if d.optional else "")})
                if zb in ("enterprise", "labs") :
                    blockers.append({"from": n, "to": d.pkg, "kind": d.kind + (",opt" if d.optional else "")})
                ra, rb = soll.ring(n), soll.ring(d.pkg)
                if zb == "core" and ra is not None and rb is not None and ra < rb and want("R1"):
                    add(f"R1|{n}->{d.pkg}", 1, f"{n} (Ring {ra}) → {d.pkg} (Ring {rb})")

    # --- Quelltext-/Manifestregeln je Kern-Crate ------------------------------------
    for n in sorted(core):
        c = ws.crates[n]
        ring = soll.ring(n)
        # R4
        if want("R4") and ring in rules["tokio_free_rings"]:
            for d in c.deps:
                if d.kind in ("normal", "build") and d.pkg == "tokio":
                    add(f"R4|{n}", 1, f"{c.manifest}: tokio ({d.kind}{', optional' if d.optional else ''})")
        # R7 Manifest
        if want("R7") and n not in rules["network_allowed"]:
            for d in c.deps:
                if d.kind in ("normal", "build") and d.pkg in rules["network_dep_names"]:
                    add(f"R7|{n}|dep:{d.pkg}", 1, f"{c.manifest}: {d.pkg}")
        needs_src = any(want(r) for r in ("R3", "R3b", "R5", "R6", "R7"))
        if not needs_src:
            continue
        files = list(rust_files(ws.root, c.dir))
        roots = [f"{c.dir}/src/lib.rs", f"{c.dir}/src/main.rs"]
        has_root = [r for r in roots if (ws.root / r).is_file()]
        forbid_ok = False
        for rel in files:
            cls = file_class(rel, c.dir)
            src = read_text(ws.root, rel)
            masked = mask_rust(src)
            raw_for_forbid = src if rel in has_root else ""
            if rel in has_root and FORBID.search(mask_rust(raw_for_forbid)):
                forbid_ok = True
            spans = test_spans(masked)
            prod = cls == "prod"
            li = None
            # R3
            if want("R3") and n not in rules["unsafe_allowed"]:
                k = len(UNSAFE.findall(masked))
                if k:
                    add(f"R3|{rel}", k, rel)
            if not prod:
                continue
            # R6
            if want("R6"):
                for kind, rx in PANICS.items():
                    k = sum(1 for m in rx.finditer(masked) if not in_spans(m.start(), spans))
                    if k:
                        add(f"R6|{n}|{kind}", k)
            # R5
            if want("R5") and soll.ring(n) is not None and rules["fs_ring_min"] <= ring <= rules["fs_ring_max"] \
                    and n not in rules["fs_adapters"]:
                hits = [m for m in FS.finditer(masked) if not in_spans(m.start(), spans)]
                if hits:
                    li = LineIndex(src)
                    add(f"R5|{rel}", len(hits), f"{rel}:{li.line(hits[0].start())}")
            # R7 Quelltext
            if want("R7") and n not in rules["network_allowed"]:
                rx = re.compile(r"\b(%s)\b" % "|".join(map(re.escape, rules["network_tokens"])))
                hits = [m for m in rx.finditer(masked) if not in_spans(m.start(), spans)]
                if hits:
                    li = li or LineIndex(src)
                    add(f"R7|{rel}", len(hits), f"{rel}:{li.line(hits[0].start())} {hits[0].group(1)}")
        if want("R3b") and n not in rules["unsafe_allowed"] and has_root and not forbid_ok:
            add(f"R3b|{n}", 1, has_root[0])

    # --- Wording -------------------------------------------------------------------------
    if want("W1"):
        rx = re.compile("|".join(f"(?:{p})" for p in soll.wording), re.I)
        for rel in walk_files(ws.root, {".md", ".rs"}):
            if rel.startswith(("docs/postmortems/", "reports/")):
                continue
            text = read_text(ws.root, rel)
            if not rx.search(text):
                continue
            for i, ln in enumerate(text.splitlines(), 1):
                if "wording-ignore" in ln:
                    continue
                if rel.endswith(".rs") and not ln.lstrip().startswith(("///", "//!", "/**", "*")):
                    continue
                m = rx.search(ln)
                if m:
                    add(f"W1|{rel}", 1, f"{rel}:{i} «{m.group(0)}»")
    return counts, detail, blockers


def load_baseline(path: Path) -> dict[str, int] | None:
    if not path.is_file():
        return None
    return json.loads(path.read_text(encoding="utf-8")).get("entries", {})


def compare(counts: dict[str, int], base: dict[str, int]):
    new = {k: (v, base.get(k, 0)) for k, v in counts.items() if v > base.get(k, 0)}
    shrunk = {k: (counts.get(k, 0), v) for k, v in base.items() if counts.get(k, 0) < v}
    return new, shrunk


def write_baseline(path: Path, entries: dict[str, int]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = {"version": 1,
               "note": "Ratchet-Baseline (ctxrs archgate). Darf nur schrumpfen; nie von Hand erhöhen.",
               "entries": dict(sorted((k, v) for k, v in entries.items() if v > 0))}
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")


def rule_of(key: str) -> str:
    return key.split("|", 1)[0]


def blockers_md(ws: Workspace, soll: Soll, blockers: list[dict]) -> str:
    by: dict[str, list[dict]] = defaultdict(list)
    for b in blockers:
        by[b["to"]].append(b)
    if not by:
        return "Keine Kern-Abhängigkeiten auf zu streichende/ausgelagerte Crates.\n"
    rows = []
    for to, items in sorted(by.items(), key=lambda kv: (soll.zone(kv[0]), kv[0])):
        rows.append([f"`{to}`", soll.zone(to), len(items), ", ".join(f"`{b['from']}` ({b['kind']})" for b in items)])
    return md_table(["Ziel-Crate", "Zone", "Kanten", "Abhängig sind"], rows)


def run(ws: Workspace, args) -> int:
    soll = Soll()
    only = {r.strip() for r in args.rules.split(",") if r.strip()}
    counts, detail, blockers = collect(ws, soll, only)
    bpath = ws.root / args.baseline
    base = load_baseline(bpath)

    unmapped = sorted(n for n in ws.crates if n not in soll.crates)
    missing_new = sorted(n for n, v in soll.crates.items() if v.get("new") and n not in ws.crates)

    if args.init_baseline:
        if base is not None:
            raise ToolError(f"{args.baseline} existiert bereits (Ratchet: nicht überschreiben; --shrink verwenden).")
        write_baseline(bpath, counts)
        print(f"Baseline geschrieben: {args.baseline} ({len(counts)} Einträge, Summe {sum(counts.values())}).")
        return 0

    new, shrunk = compare(counts, base) if base is not None else ({}, {})
    if args.shrink:
        if base is None:
            raise ToolError("Keine Baseline vorhanden (--init-baseline).")
        if new:
            raise ToolError(f"{len(new)} neue/größere Zähler – Baseline darf nicht wachsen. Erst beheben.")
        merged = {k: min(v, counts.get(k, 0)) for k, v in base.items()}
        write_baseline(bpath, merged)
        print(f"Baseline abgesenkt: {sum(base.values())} → {sum(merged.values())}.")
        return 0

    if args.json:
        print(json.dumps({"counts": counts, "new": {k: v[0] for k, v in new.items()}, "unmapped": unmapped,
                          "blockers": blockers}, ensure_ascii=False, indent=1))
        return 2 if new else 0

    print("# ctxrs archgate\n")
    rules = ["R1", "R2", "R3", "R3b", "R4", "R5", "R6", "R7", "W1"]
    rows = []
    for r in rules:
        if only and r not in only:
            continue
        cur = sum(v for k, v in counts.items() if rule_of(k) == r)
        b = sum(v for k, v in (base or {}).items() if rule_of(k) == r) if base is not None else "–"
        nn = sum(1 for k in new if rule_of(k) == r)
        rows.append([r, RULE_TITLES[r], cur, b, nn])
    print(md_table(["Regel", "Inhalt", "Ist", "Baseline", "neu/größer"], rows))
    if base is None:
        print(f"*Keine Baseline* (`{args.baseline}`). Ist-Zustand oben; einmalig `--init-baseline`, danach gilt der Ratchet.\n")
    if new:
        print("## NEU oder größer als Baseline (Gate schlägt fehl)\n")
        for k, (cur, b) in sorted(new.items()):
            ex = "; ".join(detail.get(k, [])[:2])
            print(f"- `{k}`: {b} → {cur}" + (f" — {ex}" if ex else ""))
        print()
    if shrunk:
        print(f"## Verbessert: {len(shrunk)} Zähler unter Baseline – `--shrink` übernimmt das.\n")
    if args.full:
        print("## Alle Zähler\n")
        for k, v in sorted(counts.items()):
            print(f"- `{k}`: {v}" + (f" — {'; '.join(detail[k][:2])}" if detail.get(k) else ""))
        print()
    if args.blockers:
        print("## Löschblockaden: Kern-Crates mit Kanten auf auszulagernde/zu streichende Crates\n")
        print(blockers_md(ws, soll, blockers))
    if unmapped:
        print(f"Nicht in soll.toml zugeordnet ({len(unmapped)}): " + ", ".join(f"`{u}`" for u in unmapped) + "\n")
    if missing_new:
        print("SOLL-Crates noch nicht angelegt: " + ", ".join(f"`{u}`" for u in missing_new) + " (`ctxrs scaffold NAME`)\n")
    return 2 if new else 0
