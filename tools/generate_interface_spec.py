#!/usr/bin/env python3
"""
generate_interface_spec.py
===========================

Erzeugt die "Globale Mikrofeingranulare Schnittstellenspezifikation" für alle
Contextra-Workspace-Crates neu, direkt aus dem aktuellen Quellcode-Stand.

Funktionsweise
--------------
1. Liest `crates/*/Cargo.toml` und `crates/*/src/**/*.rs`.
2. Extrahiert per Regex/Klammer-Matching alle `pub`-Symbole:
   traits, structs, enums, fn, type-Aliase, const (Test-Module werden
   übersprungen: `#[cfg(test)] mod tests { ... }` bzw. `mod tests { ... }`).
3. Liest die `lib.rs`-Kopfdokumentation sowie `pub mod` / `pub use`
   Deklarationen (= öffentliche Modul-/Re-Export-Grenze des Crates).
4. Liest Abhängigkeiten und Feature-Flags aus `Cargo.toml`.
5. Rendert daraus ein einziges, in sich geschlossenes Markdown-Dokument,
   gegliedert nach dem Ring-0..4-Schichtenmodell aus ARCHITECTURE.md.
6. Schreibt das Ergebnis mit Datumsstempel nach `docs/`.

Verwendung
----------
    python3 scripts/generate_interface_spec.py
    python3 scripts/generate_interface_spec.py --repo-root /pfad/zu/contextra
    python3 scripts/generate_interface_spec.py --out-dir docs/specs --no-timestamp

Empfohlener Ablageort im Repo: `scripts/generate_interface_spec.py`
(rein lesend — verändert keine Quelldateien, keine externen Abhängigkeiten
außer der Python-3-Standardbibliothek).

Exit-Codes: 0 = Erfolg, 1 = Repo-Struktur nicht gefunden / Fehler.
"""

from __future__ import annotations

import argparse
import datetime as _dt
import json
import os
import re
import sys
from collections import Counter

# --------------------------------------------------------------------------- #
# 1. Ring-/Status-Metadaten (kuratiert aus ARCHITECTURE.md Crate-Inventar)
# --------------------------------------------------------------------------- #

RING_META: dict[str, tuple[int, str, str]] = {
    "contextra-types": (0, "Fertig", "Kanonische Domänenmodelle, IDs, Budgets, Filter-AST und Fehlertypen."),
    "contextra-ports": (0, "Fertig", "dyn-kompatible Trait-Definitionen (Ports) für Storage, Indizes, Embedder und System-Uhren."),
    "contextra-mvcc": (0, "Fertig", "SeqLog, SnapshotRegistry und transaktionaler TxBuffer (MVCC-Isolation)."),
    "contextra-wire": (0, "Fertig", "FlatBuffers-IPC-Generat und Zero-Copy-Adapter (Unsafe-Insel)."),
    "contextra-sys": (0, "Fertig", "System-Abstraktionen für mmap, mlock und Win32-ACLs (Unsafe-Insel)."),
    "contextra-simd": (0, "Fertig", "AVX2/AVX-512/NEON SIMD-Distanzberechnungskerne mit Laufzeit-Dispatch (Unsafe-Insel)."),
    "contextra-crypto": (0, "In Migration", "AES-256-GCM-SIV, WAL-HMAC-Integritätsketten, Zeroize und Deletion-Proofs."),
    "contextra-vector": (0, "In Migration", "HNSW- und DiskANN-Vektorindex-Implementierungen."),
    "contextra-text": (0, "Fertig", "BM25/BM25F-Volltextindexierung und deutsche Kompositazerlegung."),
    "contextra-graph": (0, "Fertig", "Compressed Sparse Row (CSR) Graph, Forward-Push PPR, Leiden und Hyperkanten."),
    "contextra-rank": (0, "In Migration", "4-Signal-Retrieval-Fusion, Score-Kalibrierung und Drift-Messung."),
    "contextra-adapt": (0, "Fertig", "LinUCB-Bandit-Mathematik, Lyapunov-Drift-Wächter und PID-Regler."),
    "contextra-core": (0, "Fertig (Strangler Facade)", "Re-Export-Fassade über types, ports, mvcc und wire zur Abwärtskompatibilität."),
    "contextra-audit-export": (0, "Fertig", "Generator für Verarbeitungsverzeichnisse gemäß DSGVO Art. 30."),
    "contextra-avv-generator": (0, "Fertig", "Template-Generator für Auftragsverarbeitungsverträge (AVV)."),
    "contextra-license": (0, "Fertig", "Lizenzierungs- und Aktivierungsprüfung für Feature-Ringe."),
    "contextra-store": (1, "Fertig", "LSM-Tree Storage Engine mit WAL-Group-Commit und HMAC-Integritätsprüfung."),
    "contextra-kvcache": (1, "Fertig", "Verschlüsselter Prefix-Radix-Baum und KV-Cache-Segment-Verwaltung."),
    "contextra-checkpoint": (1, "Fertig", "Time-Travel-Registry und Checkpoint-Verwaltung ohne globalen Zustand."),
    "contextra-infer-candle": (2, "In Migration", "Native GGUF-Modell-Inferenz via Candle."),
    "contextra-infer-ollama": (2, "In Migration", "HTTP-Inferenz-Client für Ollama mit Contextual-Chunk-Prefixing."),
    "contextra-infer-onnx": (2, "In Migration", "ONNX-Embeddings und Reranking via ort."),
    "contextra-sandbox": (2, "Fertig", "WASM-Ausführungs-Isolation mit Fuel- und Wall-Clock-Limits via Wasmtime."),
    "contextra-engine": (3, "In Migration", "Collection-LSM-Anbindung, Multi-Index-Pläne und Schreibtransaktionen."),
    "contextra-cognition": (3, "In Migration", "Hintergrund-Kompaktierung, Synthese und Konsolidierungs-Scheduler."),
    "contextra-privacy": (3, "In Migration", "Cloud-Egress Privacy Gateway, PII-Vault, Surrogat-Tokenisierung und DLP."),
    "contextra-router": (3, "In Migration", "SLM-Profil-Routing und MCP-Dispatch."),
    "contextra-agent": (3, "Fertig", "Agenten-Workflow-Engine mit auditierbarer State-Machine und DLQ."),
    "contextra-db": (3, "In Migration (Legacy-Fassade)", "Monolithische Übergangsfassade für Speicher-, Such- und Indexzugriffe."),
    "contextra": (4, "Fertig", "Öffentliche Haupt-Fassade und Composition Root für Rust-Anwendungen."),
    "contextra-mcp": (4, "In Migration", "Stdio-JSON-RPC MCP-Server-Protokoll-Adapter."),
    "contextra-py": (4, "In Migration", "PyO3 Python-FFI-Bindings."),
    "contextra-testkit": (99, "Fertig (Tooling)", "Deterministische Test-Utilities (ManualClock, InMemoryStorageEngine, FaultVfs)."),
}

RING_NAMES = {
    0: "Ring 0 — Foundation & Pure Numerics/Types",
    1: "Ring 1 — Persistence & Cache",
    2: "Ring 2 — External Adapters & Sandboxing (Leaf Engines)",
    3: "Ring 3 — Orchestration & Cognition",
    4: "Ring 4 — Boundary & Composition Roots",
    99: "Tooling (Cross-Ring)",
}

# Reihenfolge der Darstellung im Dokument (Ring-für-Ring, dependency-nah sortiert)
ORDER = [
    "contextra-types", "contextra-ports", "contextra-mvcc", "contextra-wire", "contextra-sys",
    "contextra-simd", "contextra-crypto", "contextra-vector", "contextra-text", "contextra-graph",
    "contextra-rank", "contextra-adapt", "contextra-core", "contextra-audit-export",
    "contextra-avv-generator", "contextra-license",
    "contextra-store", "contextra-kvcache", "contextra-checkpoint",
    "contextra-infer-candle", "contextra-infer-ollama", "contextra-infer-onnx", "contextra-sandbox",
    "contextra-engine", "contextra-cognition", "contextra-privacy", "contextra-router",
    "contextra-agent", "contextra-db",
    "contextra", "contextra-mcp", "contextra-py",
    "contextra-testkit",
]

UNSAFE_ISLANDS = {"contextra-simd", "contextra-sys", "contextra-wire"}


# --------------------------------------------------------------------------- #
# 2. Rust-Quellcode-Extraktion (Regex/Klammer-Matching, keine externen Deps)
# --------------------------------------------------------------------------- #

def strip_tests(src: str) -> str:
    """Schneidet den üblichen `#[cfg(test)] mod tests { ... }`-Block am Dateiende ab."""
    m = re.search(r"\n#\[cfg\(test\)\]\s*\nmod\s+tests\b", src)
    if m:
        return src[: m.start()]
    m = re.search(r"\nmod\s+tests\s*\{", src)
    if m:
        return src[: m.start()]
    return src


def find_matching_brace(src: str, start_idx: int) -> int:
    depth = 0
    i = start_idx
    while i < len(src):
        c = src[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return -1


def get_preceding_doc(src: str, start_idx: int) -> tuple[list[str], list[str]]:
    lines_before = src[:start_idx].split("\n")
    doc_lines: list[str] = []
    attr_lines: list[str] = []
    i = len(lines_before) - 1
    while i >= 0:
        line = lines_before[i].strip()
        if line == "":
            i -= 1
            continue
        if line.startswith("///"):
            doc_lines.insert(0, line[3:].strip())
            i -= 1
            continue
        if line.startswith("#["):
            attr_lines.insert(0, line)
            i -= 1
            continue
        if line.startswith("//!"):
            i -= 1
            continue
        break
    return doc_lines, attr_lines


def extract_items(filepath: str, crate_root: str) -> list[dict]:
    with open(filepath, "r", errors="ignore") as f:
        src = f.read()
    src = strip_tests(src)
    rel = os.path.relpath(filepath, crate_root)
    items: list[dict] = []

    # PUB TRAITS (voller Methodenkörper = eigentliche Schnittstelle)
    for m in re.finditer(r"\n(\s*)pub\s+trait\s+(\w+)((?:<[^{]*?>)?)\s*((?::[^{]*?)?)\s*\{", src):
        indent, name, generics, bounds = m.groups()
        brace_start = m.end() - 1
        brace_end = find_matching_brace(src, brace_start)
        if brace_end == -1:
            continue
        body = src[brace_start + 1: brace_end]
        doc, _ = get_preceding_doc(src, m.start(1) + len(indent))
        items.append({"kind": "trait", "name": name, "generics": generics.strip(),
                      "bounds": bounds.strip(), "body": body.strip(), "doc": " ".join(doc), "file": rel})

    # PUB STRUCT
    for m in re.finditer(r"\n(\s*)pub\s+struct\s+(\w+)([^;{]*)", src):
        indent, name, rest = m.groups()
        doc, attrs = get_preceding_doc(src, m.start(1) + len(indent))
        forward = src[m.end(): m.end() + 2000]
        semi_idx = forward.find(";")
        brace_idx = forward.find("{")
        if brace_idx != -1 and (semi_idx == -1 or brace_idx < semi_idx):
            abs_brace = m.end() + brace_idx
            brace_end = find_matching_brace(src, abs_brace)
            body = src[abs_brace + 1: brace_end] if brace_end != -1 else ""
        else:
            body = None
        items.append({"kind": "struct", "name": name, "sig": rest.strip(), "body": body,
                      "doc": " ".join(doc), "file": rel, "attrs": attrs})

    # PUB ENUM
    for m in re.finditer(r"\n(\s*)pub\s+enum\s+(\w+)([^{]*)\{", src):
        indent, name, generics = m.groups()
        doc, attrs = get_preceding_doc(src, m.start(1) + len(indent))
        brace_start = m.end() - 1
        brace_end = find_matching_brace(src, brace_start)
        body = src[brace_start + 1: brace_end] if brace_end != -1 else ""
        items.append({"kind": "enum", "name": name, "generics": generics.strip(),
                      "body": body.strip(), "doc": " ".join(doc), "file": rel, "attrs": attrs})

    # PUB FN (Top-Level und impl-Ebene; pub(crate) wird ausgeschlossen)
    for m in re.finditer(r"\n(\s*)pub(?:\(crate\))?\s+(?:async\s+)?(?:unsafe\s+)?fn\s+(\w+)", src):
        indent = m.group(1)
        if "pub(crate)" in src[m.start(): m.start() + 20]:
            continue
        name = m.group(2)
        doc, _ = get_preceding_doc(src, m.start(1) + len(indent))
        paren_start = src.find("(", m.end())
        if paren_start == -1:
            continue
        depth = 0
        i = paren_start
        while i < len(src):
            if src[i] == "(":
                depth += 1
            elif src[i] == ")":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        sig_end_search = src[i + 1: i + 400]
        term = re.search(r"[{;]", sig_end_search)
        term_idx = i + 1 + term.start() if term else i + 1
        signature = src[m.start(1) + len(indent): term_idx].strip()
        signature = re.sub(r"\s+", " ", signature)
        items.append({"kind": "fn", "name": name, "sig": signature, "doc": " ".join(doc), "file": rel})

    # PUB TYPE alias
    for m in re.finditer(r"\n(\s*)pub\s+type\s+(\w+)([^;]*);", src):
        indent, name, rest = m.groups()
        doc, _ = get_preceding_doc(src, m.start(1) + len(indent))
        items.append({"kind": "type", "name": name, "sig": re.sub(r"\s+", " ", rest.strip()),
                      "doc": " ".join(doc), "file": rel})

    # PUB CONST
    for m in re.finditer(r"\n(\s*)pub\s+const\s+(\w+)\s*:\s*([^=;]+)", src):
        indent, name, ty = m.groups()
        doc, _ = get_preceding_doc(src, m.start(1) + len(indent))
        items.append({"kind": "const", "name": name, "sig": ty.strip(), "doc": " ".join(doc), "file": rel})

    return items


def collect_crate_items(crate_dir: str) -> list[dict]:
    src_dir = os.path.join(crate_dir, "src")
    all_items: list[dict] = []
    if not os.path.isdir(src_dir):
        return all_items
    for root, dirs, files in os.walk(src_dir):
        dirs[:] = [d for d in dirs if d not in ("tests", "test")]
        for fn in files:
            if fn.endswith(".rs"):
                fp = os.path.join(root, fn)
                try:
                    all_items.extend(extract_items(fp, crate_dir))
                except Exception as e:  # pragma: no cover - defensive, keep script running
                    print(f"[warn] extraction failed for {fp}: {e}", file=sys.stderr)
    return all_items


def extract_lib_map(crate_dir: str) -> dict:
    lib = os.path.join(crate_dir, "src", "lib.rs")
    if not os.path.isfile(lib):
        return {"doc": "", "mods": [], "mods_inline": [], "uses": []}
    with open(lib, errors="ignore") as f:
        src = f.read()
    doc_lines: list[str] = []
    for line in src.split("\n"):
        s = line.strip()
        if s.startswith("//!"):
            doc_lines.append(s[3:].strip())
        elif s == "" and not doc_lines:
            continue
        else:
            if doc_lines:
                break
            if s.startswith("//") or s == "":
                continue
            break
    mods = re.findall(r"^\s*pub mod (\w+);", src, re.M)
    mods_inline = re.findall(r"^\s*pub mod (\w+)\s*\{", src, re.M)
    uses = []
    for m in re.finditer(r"pub use ([^;]+);", src, re.S):
        uses.append(re.sub(r"\s+", " ", m.group(1)).strip())
    return {"doc": " ".join(doc_lines), "mods": mods, "mods_inline": mods_inline, "uses": uses}


def parse_cargo_toml(crate_dir: str) -> tuple[list[str], list[str], list[tuple[str, str]]]:
    path = os.path.join(crate_dir, "Cargo.toml")
    with open(path) as f:
        content = f.read()

    def section(sec_name: str) -> str:
        m = re.search(rf"\n\[{re.escape(sec_name)}\]\n(.*?)(?=\n\[|\Z)", content, re.S)
        return m.group(1) if m else ""

    deps_sec = section("dependencies")
    feat_sec = section("features")
    internal, external = [], []
    for line in deps_sec.split("\n"):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        m = re.match(r"([A-Za-z0-9_\-]+)\s*=", line)
        if not m:
            continue
        dep = m.group(1)
        (internal if dep.startswith("contextra") else external).append(dep)
    internal, external = sorted(set(internal)), sorted(set(external))
    features = []
    for line in feat_sec.split("\n"):
        line = line.strip()
        m = re.match(r"([A-Za-z0-9_\-]+)\s*=\s*(\[.*\])", line)
        if m:
            features.append((m.group(1), m.group(2)))
    return internal, external, features


# --------------------------------------------------------------------------- #
# 3. Markdown-Rendering
# --------------------------------------------------------------------------- #

def esc(s: str | None) -> str:
    if s is None:
        return ""
    return s.replace("|", "\\|").replace("\n", " ").strip()


def clip(s: str | None, n: int = 220) -> str:
    s = s or ""
    s = re.sub(r"\s+", " ", s).strip()
    return s if len(s) <= n else s[: n - 1].rstrip() + "…"


def fmt_trait(t: dict) -> str:
    body = t["body"].strip("\n")
    sig = f"pub trait {t['name']}{t['generics']}{(' ' + t['bounds']) if t['bounds'] else ''} {{\n"
    sig += body + "\n}"
    return sig


def render_crate(name: str, crate_dir: str, f) -> Counter:
    ring, status, purpose = RING_META[name]
    items = collect_crate_items(crate_dir)
    lm = extract_lib_map(crate_dir)
    internal, external, features = parse_cargo_toml(crate_dir)
    unsafe_flag = "Ja (deklarierte Unsafe-Insel)" if name in UNSAFE_ISLANDS else "Nein (`#![forbid(unsafe_code)]`)"

    f.write(f"\n---\n\n## `{name}`\n\n")
    f.write("| Attribut | Wert |\n|---|---|\n")
    f.write(f"| **Ring** | {ring} — {RING_NAMES[ring]} |\n")
    f.write(f"| **Status** | {status} |\n")
    f.write(f"| **Zweck** | {purpose} |\n")
    f.write(f"| **Unsafe-Code** | {unsafe_flag} |\n")
    f.write(f"| **Interne Abhängigkeiten** | {', '.join(f'`{i}`' for i in internal) if internal else '—'} |\n")
    f.write(f"| **Externe Abhängigkeiten (Auswahl)** | {', '.join(f'`{e}`' for e in external) if external else '—'} |\n\n")

    if lm.get("doc"):
        f.write(f"### Architekturbeschreibung\n\n{lm['doc']}\n\n")

    f.write("### Modul- und Re-Export-Oberfläche (lib.rs)\n\n```rust\n")
    for m in lm.get("mods", []):
        f.write(f"pub mod {m};\n")
    for m in lm.get("mods_inline", []):
        f.write(f"pub mod {m} {{ .. }}\n")
    for u in lm.get("uses", []):
        f.write(f"pub use {u};\n")
    f.write("```\n\n")

    if features:
        f.write("### Feature-Flags\n\n| Flag | Aktiviert |\n|---|---|\n")
        for fn_, fv in features:
            f.write(f"| `{fn_}` | `{esc(fv)}` |\n")
        f.write("\n")

    traits = [i for i in items if i["kind"] == "trait"]
    structs = [i for i in items if i["kind"] == "struct"]
    enums = [i for i in items if i["kind"] == "enum"]
    fns = [i for i in items if i["kind"] == "fn"]
    types = [i for i in items if i["kind"] == "type"]
    consts = [i for i in items if i["kind"] == "const"]

    if traits:
        f.write(f"### Schnittstellen — Public Traits ({len(traits)})\n\n")
        for t in sorted(traits, key=lambda x: x["name"]):
            f.write(f"**`{t['name']}`** — *{clip(t['doc'], 260)}* (`{t['file']}`)\n\n")
            f.write("```rust\n" + fmt_trait(t) + "\n```\n\n")

    if structs:
        f.write(f"### Öffentliche Datentypen — Structs ({len(structs)})\n\n")
        f.write("| Struct | Felder / Form | Beschreibung | Datei |\n|---|---|---|---|\n")
        for s in sorted(structs, key=lambda x: x["name"]):
            body = s.get("body")
            if body is not None:
                bshort = clip(body.replace("\n", " "), 160)
                form = f"{{ {bshort} }}" if bshort else "{ }"
            else:
                form = "(tuple/unit)"
            f.write(f"| `{s['name']}` | `{esc(form)}` | {clip(s['doc'], 150) or '—'} | `{s['file']}` |\n")
        f.write("\n")

    if enums:
        f.write(f"### Öffentliche Datentypen — Enums ({len(enums)})\n\n")
        f.write("| Enum | Varianten | Beschreibung | Datei |\n|---|---|---|---|\n")
        for e in sorted(enums, key=lambda x: x["name"]):
            variants = clip(e["body"].replace("\n", " "), 200)
            f.write(f"| `{e['name']}` | `{esc(variants)}` | {clip(e['doc'], 150) or '—'} | `{e['file']}` |\n")
        f.write("\n")

    if types:
        f.write(f"### Typ-Aliase ({len(types)})\n\n")
        f.write("| Alias | Definition | Beschreibung |\n|---|---|---|\n")
        for t in sorted(types, key=lambda x: x["name"]):
            f.write(f"| `{t['name']}` | `type {t['name']}{esc(t['sig'])}` | {clip(t['doc'], 150) or '—'} |\n")
        f.write("\n")

    if consts:
        f.write(f"### Konstanten ({len(consts)})\n\n")
        f.write("| Konstante | Typ | Beschreibung |\n|---|---|---|\n")
        for c in sorted(consts, key=lambda x: x["name"]):
            f.write(f"| `{c['name']}` | `{esc(c['sig'])}` | {clip(c['doc'], 150) or '—'} |\n")
        f.write("\n")

    if fns:
        f.write(f"### Öffentliche Funktionen ({len(fns)})\n\n")
        f.write("| Signatur | Beschreibung | Datei |\n|---|---|---|\n")
        for fi in sorted(fns, key=lambda x: (x["file"], x["name"])):
            f.write(f"| `{esc(clip(fi['sig'], 260))}` | {clip(fi['doc'], 140) or '—'} | `{fi['file']}` |\n")
        f.write("\n")

    return Counter(i["kind"] for i in items)


def render_frontmatter(f, generated_at: str, total_stats: Counter, n_crates: int) -> None:
    f.write(f"""# Contextra — Globale Mikrofeingranulare Schnittstellenspezifikation

**Repository:** `contextra` (Workspace-Root) · **Generiert am:** {generated_at}
**Erzeugt durch:** `scripts/generate_interface_spec.py` (automatisierte, reproduzierbare Vollanalyse aller Workspace-Crates)
**Methodik:** Statische Extraktion aller `pub`-Symbole (Traits, Structs, Enums, Funktionen, Typ-Aliase, Konstanten) aus jedem `src/**/*.rs` je Crate, angereichert mit den Original-Doc-Kommentaren (`///`, `//!`), den `Cargo.toml`-Metadaten (Abhängigkeiten, Feature-Flags) und der `lib.rs`-Re-Export-Oberfläche. Test-Module (`#[cfg(test)] mod tests`) sind ausgeschlossen.

> **Hinweis zur Reproduzierbarkeit:** Dieses Dokument wird bei jedem Lauf von `scripts/generate_interface_spec.py` vollständig neu aus dem aktuellen Quellcode-Stand erzeugt. Es spiegelt exakt den Commit wider, gegen den es generiert wurde — bei Architekturänderungen (neue Crates, geänderte Traits/Ports) einfach das Skript erneut ausführen.

## Umfang der Analyse

| Kennzahl | Wert |
|---|---|
| Analysierte Fach-Crates (`crates/*`) | {n_crates} |
| Extrahierte öffentliche Symbole gesamt | **{sum(total_stats.values())}** |
| — davon Public Traits (Schnittstellen im engeren Sinn) | {total_stats.get('trait', 0)} |
| — davon Public Structs | {total_stats.get('struct', 0)} |
| — davon Public Enums | {total_stats.get('enum', 0)} |
| — davon Public Funktionen | {total_stats.get('fn', 0)} |
| — davon Typ-Aliase | {total_stats.get('type', 0)} |
| — davon Konstanten | {total_stats.get('const', 0)} |

---

## 1. Ring-Architektur (Layering-Modell, DAG-Invariante)

Contextra erzwingt ein striktes 5-Ring-Schichtenmodell. Abhängigkeiten dürfen **ausschließlich von höheren auf niedrigere Ringe** verlaufen (keine Aufwärtskanten, P5-Invariante).

| Ring | Name | Verantwortung | Merkmal |
|---|---|---|---|
| **0** | Foundation & Pure Numerics/Types | Domänentypen, Traits/Ports, Kryptographie, Mathematik, Vektor-/Text-/Graph-Primitive | **Kein `tokio`** — Sync-Kern, async-Schale (P26) |
| **1** | Persistence & Cache | LSM-Tree-Storage, WAL, Block-Cache, KV-Cache-Segmente, Time-Travel-Checkpoints | Async-I/O nur an den Rändern |
| **2** | External Adapters & Sandboxing | Candle/GGUF-, Ollama-, ONNX-Inferenz, WASM-Sandbox | Blätter im Abhängigkeitsgraphen |
| **3** | Orchestration & Cognition | Transaktions-Engine, Collection-Engine, Kompaktierung, Privacy-Gateway, Routing, Agent-Workflows | Geschäfts-/Orchestrierungslogik |
| **4** | Boundary & Composition Roots | Öffentliche Fassade (`contextra`), MCP-Server, Python-Bindings | Einzige Composition Root |

**Unsafe-Inseln:** `contextra-sys`, `contextra-simd`, `contextra-wire` (einzige Crates ohne `#![forbid(unsafe_code)]`).

**Locking-Disziplin:** `collections (RwLock) → kv_locks (KvKeyLocks) → embedder (RwLock)`, Shard-Indizes strikt aufsteigend akquirieren, kein `.await` unter gehaltenem Sync-Lock.

---

## 2. Crate-Inventar nach Ring (Navigationsübersicht)

**Ring 0 — Foundation:** {', '.join(f'`{c}`' for c in ORDER if RING_META[c][0] == 0)}

**Ring 1 — Persistence & Cache:** {', '.join(f'`{c}`' for c in ORDER if RING_META[c][0] == 1)}

**Ring 2 — External Adapters & Sandboxing:** {', '.join(f'`{c}`' for c in ORDER if RING_META[c][0] == 2)}

**Ring 3 — Orchestration & Cognition:** {', '.join(f'`{c}`' for c in ORDER if RING_META[c][0] == 3)}

**Ring 4 — Boundary & Composition Roots:** {', '.join(f'`{c}`' for c in ORDER if RING_META[c][0] == 4)}

**Tooling:** {', '.join(f'`{c}`' for c in ORDER if RING_META[c][0] == 99)}

---

## 3. Legende der Crate-Abschnitte

1. **Metadatentabelle** — Ring, Migrationsstatus, Zweck, Unsafe-Deklaration, Abhängigkeiten.
2. **Architekturbeschreibung** — Original-`//!`-Kopfkommentar aus `lib.rs`.
3. **Modul- und Re-Export-Oberfläche** — exakte `pub mod`/`pub use`-Deklarationen; definiert die von außen sichtbare API-Grenze.
4. **Feature-Flags** — Cargo-Features und Aktivierungsketten.
5. **Schnittstellen (Public Traits)** — vollständige Methodensignaturen.
6. **Öffentliche Datentypen** — Structs (Felder) und Enums (Varianten).
7. **Typ-Aliase / Konstanten**.
8. **Öffentliche Funktionen** — vollständige Signaturen mit Kurzbeschreibung.
""")


def render_closing(f) -> None:
    f.write("""
---

## Anhang A — Querschnittliche Invarianten (gelten crateübergreifend)

* **Zero-Panic-Doktrin:** `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable` sind workspace-weit als Clippy-Lint verboten.
* **`unsafe_code = "deny"`** ist Workspace-Default; nur `contextra-sys`, `contextra-simd`, `contextra-wire` heben dies explizit auf.
* **Sync-Kern, Async-Schale (P26):** Ring-0-Crates verwenden kein `tokio`.
* **DAG-Integrität (P5):** Aufwärtskanten sind verboten; einzige dokumentierte Restausnahme: `contextra-router → contextra-db` (Migrationsartefakt).
* **`#[non_exhaustive]`-Disziplin** bei zentralen Fehler-Enums (u. a. `ContextraError`).
* **Deletion-Proof-Kette:** kryptographisch extern verifizierbare Löschbeweise (`contextra-crypto`).
* **Feature-Ring-Trennung:** `fast` / `sovereign` / `compliance` (Crate `contextra`) trennen Lizenzstufen auf Kompilierungsebene.

## Anhang B — Nicht tiefenanalysierte Hilfs-Targets

| Target | Pfad | Rolle |
|---|---|---|
| `contextra-bench` | `benchmarks/contextra-bench` | Criterion-Benchmark-Harness, kein Teil der Produktions-API. |
| `xtask` | `xtask/` | Cargo-`xtask`-Automatisierungsbinary, von `[workspace] exclude` ausgenommen. |

## Anhang C — Referenzierte Primärquellen im Repository

* `README.md`, `ARCHITECTURE.md`, `CONSTITUTION.md`, `VETOES.md`, `AGENTS.md`
* `capabilities.toml`, `deny.toml`, `supply-chain/`
* `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`
* `docs/refactor/router-db-edge-audit.md`, `docs/refactor/mcp-upward-edges-audit.md`

---

*Ende der globalen Schnittstellenspezifikation — automatisch generiert. Für Implementierungsdetails unterhalb der öffentlichen API-Grenze (private Module, `pub(crate)`-Symbole) den Quellcode direkt konsultieren.*
""")


# --------------------------------------------------------------------------- #
# 4. Orchestrierung / CLI
# --------------------------------------------------------------------------- #

def main() -> int:
    parser = argparse.ArgumentParser(description="Regeneriert die Contextra-Schnittstellenspezifikation aus dem Quellcode.")
    parser.add_argument("--repo-root", default=".", help="Pfad zum Contextra-Repository-Root (Standard: aktuelles Verzeichnis).")
    parser.add_argument("--out-dir", default="docs", help="Zielverzeichnis für die generierte Datei (Standard: docs/).")
    parser.add_argument("--no-timestamp", action="store_true",
                         help="Schreibt zusätzlich KEINE datierte Kopie, nur die stabile Datei ohne Datum im Namen.")
    parser.add_argument("--stable-name", default="CONTEXTRA_INTERFACE_SPECIFICATION.md",
                         help="Dateiname der stets aktuellen, undatierten Kopie (wird zusätzlich zur datierten Version geschrieben, außer bei --no-timestamp-only).")
    parser.add_argument("--no-timestamp-only", action="store_true",
                         help="Schreibt NUR die stabile Datei ohne Datumsstempel im Namen (kein <name>_YYYYMMDD.md).")
    args = parser.parse_args()

    repo_root = os.path.abspath(args.repo_root)
    crates_dir = os.path.join(repo_root, "crates")
    if not os.path.isdir(crates_dir):
        print(f"[error] '{crates_dir}' nicht gefunden. --repo-root muss auf den Contextra-Workspace-Root zeigen.",
              file=sys.stderr)
        return 1

    missing = [c for c in ORDER if not os.path.isdir(os.path.join(crates_dir, c))]
    present = [c for c in ORDER if c not in missing]
    if missing:
        print(f"[warn] {len(missing)} im Skript bekannte Crate(s) nicht im Repo gefunden (übersprungen): {missing}",
              file=sys.stderr)

    now = _dt.datetime.now()
    date_str = now.strftime("%Y-%m-%d")
    stamp = now.strftime("%Y%m%d")

    out_dir = os.path.join(repo_root, args.out_dir)
    os.makedirs(out_dir, exist_ok=True)

    body_buf = []
    total_stats: Counter = Counter()

    class _Writer:
        """Kleiner Puffer-Adapter, damit render_crate() mit .write() arbeiten kann."""
        def write(self, s: str) -> None:
            body_buf.append(s)

    writer = _Writer()
    for name in present:
        crate_dir = os.path.join(crates_dir, name)
        stats = render_crate(name, crate_dir, writer)
        total_stats.update(stats)
        print(f"[ok] {name}: {sum(stats.values())} öffentliche Symbole extrahiert", file=sys.stderr)

    stable_base, ext = os.path.splitext(args.stable_name)
    targets = []
    if not args.no_timestamp_only:
        targets.append(os.path.join(out_dir, f"{stable_base}_{stamp}{ext}"))
    targets.append(os.path.join(out_dir, args.stable_name))

    for target_path in targets:
        with open(target_path, "w") as f:
            render_frontmatter(f, generated_at=date_str, total_stats=total_stats, n_crates=len(present))
            f.write("".join(body_buf))
            render_closing(f)
        print(f"[done] geschrieben: {target_path}", file=sys.stderr)

    print(f"\n{len(present)} Crates, {sum(total_stats.values())} öffentliche Symbole dokumentiert.", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
