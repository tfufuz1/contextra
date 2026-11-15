#!/usr/bin/env python3
"""
extract_acta.py - extrahiert die fuer Acta relevanten Crates aus dem Contextra-Repository,
benennt sie um (contextra -> acta) und legt sie in einem Acta-Verzeichnis ab.

Nur Python-3-Standardbibliothek. Benoetigt `git` (empfohlen) und `cargo`
(fuer die Abhaengigkeitsanalyse via `cargo metadata`).

Beispiele (im Contextra-Repository ausfuehren):
    python3 extract_acta.py --dry-run                 # nur anzeigen, was passieren wuerde
    python3 extract_acta.py --dest ../acta            # Profil "spec" (Ringmodell der SPEC 3.0)
    python3 extract_acta.py --profile skeleton        # nur der Walking Skeleton (R2)
    python3 extract_acta.py --crates ids,env,crypto   # eigene Auswahl (Abhaengigkeiten kommen automatisch dazu)
    python3 extract_acta.py --verify                  # danach `cargo check --workspace` im Ergebnis

Das Skript aendert das Contextra-Repository NICHT. Es liest nur (git ls-files / Dateien).
Im Ziel werden nur `crates/`, die Workspace-Dateien und EXTRACT_REPORT.md angelegt; `docs/` und `.git`
(z. B. docs/SPEC.md eines bestehenden Acta-Repos) bleiben unangetastet.
"""
from __future__ import annotations

import argparse
import datetime as dt
import json
import re
import shutil
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

OLD, NEW = "contextra", "acta"

# Reihenfolge ist wichtig (von spezifisch zu allgemein).
RENAME_RULES = [("CONTEXTRA", "ACTA"), ("Contextra", "Acta"), ("contextra", "acta")]

# Profile: Basisnamen ohne Praefix "contextra-".
PROFILES: dict[str, list[str]] = {
    # Alle Crates des Ringmodells der SPEC 3.0 (Abschnitt 3.1) ausser acta-mcp:
    # contextra-mcp zieht Facade, Candle, Ollama, ONNX, Privacy, Router usw. nach und ist nicht 1:1 uebertragbar.
    "spec": [
        "ids", "env", "sys", "durable-fs", "simd", "wire", "crypto",  # R0
        "types", "ports",                                             # R2
        "erasure",                                                    # R3
        "store", "mvcc", "vector", "text", "graph", "rank", "checkpoint",  # R1
    ],
    # Walking Skeleton (SPEC 3.0, Stufe R2): Ingest -> WAL -> ein Index -> Loeschkern.
    "skeleton": [
        "ids", "env", "sys", "durable-fs", "crypto", "types", "ports",
        "erasure", "store", "mvcc", "text",
    ],
}

# Pro Crate nicht uebernommen (SPEC 3.0: Crate-spezifische SPEC.md/AGENTS.md sind ungueltig).
EXCLUDE_TOP_DIRS = {"fuzz", "docs", "tools", "target", ".git"}
EXCLUDE_TOP_FILES = {
    "AGENTS.md", "SPEC.md", "PLAN.md", "MODULE.md", "API.md", "CHANGELOG.md",
    "capabilities.fragment.toml",
}

ROOT_FILES = [
    "rust-toolchain.toml", "clippy.toml", "deny.toml", ".gitignore", "Cargo.lock",
]

# Muster aus der SPEC 3.0 (Befundregister), die nach der Extraktion gemeldet werden.
AUDIT_PATTERNS = [
    ("V-01/V-05", r"\[0x(?:55|42)u8;\s*32\]", "Konstanter Schluessel im Code"),
    ("V-01", r"keyed_hash\(&self\.k_unit", "Fallback-DEK aus k_unit (maskiert fehlende Slots)"),
    ("V-02", r"\bb\s*\^\s*mask_byte\b", "XOR-Stromchiffre statt AEAD"),
    ("V-05", r"unwrap_or_default\(\)", "Fehler im Krypto-/Siegelpfad verschluckt"),
    ("V-05", r"let\s+_\s*=\s*hkdf_expand", "HKDF-Fehler verworfen"),
    ("V-07", r"\bDEFAULT_PARTITION\b", "Keine Klassentrennung N/P/D im Store"),
]
AUDIT_PATH_FILTER = ("crypto", "erasure", "store")  # nur Crates, deren Name dies enthaelt


# --------------------------------------------------------------------------- Hilfsfunktionen
def rename_text(s: str) -> tuple[str, int]:
    n = 0
    for a, b in RENAME_RULES:
        n += s.count(a)
        s = s.replace(a, b)
    return s, n


def is_text(data: bytes) -> bool:
    if b"\0" in data[:8192]:
        return False
    try:
        data.decode("utf-8")
    except UnicodeDecodeError:
        return False
    return True


def die(msg: str) -> "None":
    print(f"FEHLER: {msg}", file=sys.stderr)
    sys.exit(1)


def run(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)


def git_files(repo: Path, rel_dir: Path) -> list[Path]:
    """Nur versionierte Dateien (kein target/, keine Altlasten). Fallback: Dateisystem."""
    r = subprocess.run(["git", "-C", str(repo), "ls-files", "-z", "--", str(rel_dir)],
                       capture_output=True)
    if r.returncode == 0 and r.stdout:
        return [Path(p) for p in r.stdout.decode().split("\0") if p]
    out = []
    for p in (repo / rel_dir).rglob("*"):
        if p.is_file() and "target" not in p.relative_to(repo).parts:
            out.append(p.relative_to(repo))
    return out


def load_workspace(src: Path) -> dict[str, dict]:
    r = run(["cargo", "metadata", "--no-deps", "--format-version", "1"], src)
    if r.returncode != 0:
        die("`cargo metadata` fehlgeschlagen (Rust 1.89 installiert?):\n" + r.stderr[-800:])
    meta = json.loads(r.stdout)
    pk: dict[str, dict] = {}
    for p in meta["packages"]:
        pk[p["name"]] = {
            "dir": Path(p["manifest_path"]).parent.relative_to(src.resolve()),
            "deps": [(d["name"], d.get("kind") or "normal") for d in p["dependencies"]
                     if d.get("path") is not None],
        }
    return pk


def close_over_deps(pk: dict[str, dict], wanted: list[str]) -> tuple[list[str], dict[str, str]]:
    """Fuegt alle internen Abhaengigkeiten (normal, dev, build) hinzu. Liefert (Liste, Grund je Zusatz-Crate)."""
    chosen = list(dict.fromkeys(wanted))
    reasons: dict[str, str] = {}
    queue = list(chosen)
    while queue:
        cur = queue.pop(0)
        for dep, kind in pk[cur]["deps"]:
            if dep == cur or dep not in pk:
                continue
            if dep not in chosen:
                chosen.append(dep)
                reasons[dep] = f"{'Dev-' if kind == 'dev' else ''}Abhaengigkeit von {cur}"
                queue.append(dep)
    return chosen, reasons


def build_root_cargo(src: Path, selected_dirs: list[Path], selected_names: set[str]) -> str:
    text = (src / "Cargo.toml").read_text(encoding="utf-8")
    lines = text.split("\n")
    out: list[str] = []
    section = ""
    i = 0
    member_block = "\n".join(f'    "{d.as_posix()}",' for d in sorted(selected_dirs))
    while i < len(lines):
        line = lines[i]
        m = re.match(r"^\[(.+)\]\s*$", line)
        if m:
            section = m.group(1)
        # Arrays members / default-members / exclude im Abschnitt [workspace] ersetzen
        k = re.match(r"^(members|default-members|exclude)\s*=\s*\[", line)
        if section == "workspace" and k:
            key = k.group(1)
            if not line.rstrip().endswith("]"):  # mehrzeiliges Array: bis zur schliessenden Zeile ueberspringen
                while lines[i].strip() != "]":
                    i += 1
            body = "" if key == "exclude" else member_block + "\n"
            out.append(f"{key} = [\n{body}]")
            i += 1
            continue
        if section == "workspace" and line.lstrip().startswith("#"):
            i += 1  # Kommentare ueber nicht mehr vorhandene Crates verwerfen
            continue
        if section == "workspace.dependencies":
            d = re.match(r'^[\w.-]+\s*=\s*\{[^}]*path\s*=\s*"([^"]+)"', line)
            if d:
                tgt = Path(d.group(1)).name
                if tgt not in selected_names:
                    i += 1
                    continue
        out.append(line)
        i += 1
    return "\n".join(out)


def enable_workspace_feature(toml: str) -> tuple[str, bool]:
    """Contextra aktiviert das Feature `workspace` der Crates indirekt ueber db/agent/cognition
    (Cargo-Feature-Unification). Ohne diese Crates fehlen types/ports in Lib und Tests.
    Wir bilden das nach, indem `workspace` in die Default-Features aufgenommen wird.
    (SPEC 3.0 V-16 nennt genau dieses Muster als Altlast; es wird im Bericht ausgewiesen.)"""
    lines = toml.split("\n")
    section, has_ws, default_idx, feat_start = "", False, None, None
    for idx, ln in enumerate(lines):
        m = re.match(r"^\[(.+)\]\s*$", ln)
        if m:
            section = m.group(1)
            if section == "features":
                feat_start = idx
            continue
        if section == "features":
            if re.match(r"^workspace\s*=", ln):
                has_ws = True
            if re.match(r"^default\s*=\s*\[", ln):
                default_idx = idx
    if not has_ws:
        return toml, False
    if default_idx is not None:
        ln = lines[default_idx]
        if not ln.rstrip().endswith("]"):
            return toml, False  # mehrzeilig: nicht automatisch anfassen
        if '"workspace"' in ln:
            return toml, False
        inner = re.match(r"^default\s*=\s*\[(.*)\]\s*(#.*)?$", ln)
        if not inner:
            return toml, False
        items = [x.strip() for x in inner.group(1).split(",") if x.strip()]
        items.append('"workspace"')
        lines[default_idx] = f'default = [{", ".join(items)}]'
    elif feat_start is not None:
        lines.insert(feat_start + 1, 'default = ["workspace"]')
    else:
        return toml, False
    return "\n".join(lines), True


def prepare_dest(dest: Path, force: bool) -> None:
    allowed = {".git", "docs"}
    if dest.exists():
        extra = [p.name for p in dest.iterdir() if p.name not in allowed]
        if extra and not force:
            die(f"Ziel {dest} ist nicht leer ({', '.join(sorted(extra)[:6])} ...). "
                f"Mit --force werden nur crates/ und die erzeugten Root-Dateien ersetzt "
                f"(docs/ und .git bleiben).")
        if force:
            for name in ["crates", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "clippy.toml",
                         "deny.toml", ".gitignore", ".cargo", "EXTRACT_REPORT.md"]:
                p = dest / name
                if p.is_dir():
                    shutil.rmtree(p)
                elif p.exists():
                    p.unlink()
    dest.mkdir(parents=True, exist_ok=True)


# --------------------------------------------------------------------------- Hauptprogramm
def main() -> None:
    ap = argparse.ArgumentParser(description="Contextra -> Acta Extraktion")
    ap.add_argument("--src", default=".", help="Contextra-Repository (Standard: aktuelles Verzeichnis)")
    ap.add_argument("--dest", default=None, help="Zielverzeichnis (Standard: <src>/../acta)")
    ap.add_argument("--profile", choices=sorted(PROFILES), default="spec")
    ap.add_argument("--crates", default=None,
                    help="Kommagetrennte Basisnamen (z. B. ids,env,crypto); ersetzt das Profil")
    ap.add_argument("--keep-docs", action="store_true", help="Crate-Dokumente (SPEC.md, AGENTS.md, docs/ ...) mitnehmen")
    ap.add_argument("--drop-golden", action="store_true", help="tests/golden-Verzeichnisse nicht uebernehmen")
    ap.add_argument("--no-feature-patch", action="store_true",
                    help="Feature `workspace` NICHT in die Default-Features aufnehmen (Tests/Libs brauchen es dann explizit)")
    ap.add_argument("--force", action="store_true", help="vorhandene crates/ und Root-Dateien im Ziel ersetzen")
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--verify", action="store_true", help="danach `cargo check --workspace` im Ziel ausfuehren")
    a = ap.parse_args()

    src = Path(a.src).resolve()
    if a.src == ".":  # Aufruf aus einem Unterordner (z. B. docs/) ist erlaubt: Repo-Root per git suchen
        top = run(["git", "rev-parse", "--show-toplevel"], src).stdout.strip()
        if top:
            src = Path(top).resolve()
    if not (src / "Cargo.toml").exists() or not (src / "crates").is_dir():
        die(f"{src} sieht nicht nach dem Contextra-Repository aus (Cargo.toml / crates/ fehlt).")
    dest = Path(a.dest).resolve() if a.dest else (src.parent / NEW).resolve()
    if dest == src or src in dest.parents and (dest / "Cargo.toml").exists():
        die("Ziel darf nicht das Contextra-Repository selbst sein.")

    pk = load_workspace(src)
    base = [b.strip() for b in a.crates.split(",")] if a.crates else PROFILES[a.profile]
    wanted = [f"{OLD}-{b}" for b in base]
    unknown = [w for w in wanted if w not in pk]
    if unknown:
        die("Unbekannte Crates: " + ", ".join(unknown) + "\nVerfuegbar: " +
            ", ".join(sorted(n[len(OLD) + 1:] for n in pk if n.startswith(OLD + "-"))))

    chosen, reasons = close_over_deps(pk, wanted)
    sel_names = set(chosen)
    sel_dirs = [pk[n]["dir"] for n in chosen]
    new_name = lambda n: rename_text(n)[0]

    commit = run(["git", "rev-parse", "--short", "HEAD"], src).stdout.strip() or "unbekannt"
    print(f"Quelle : {src}  (Commit {commit})")
    print(f"Ziel   : {dest}")
    print(f"Crates : {len(chosen)} ({'Profil ' + a.profile if not a.crates else 'eigene Auswahl'})")
    for n in chosen:
        tag = f"   <- {reasons[n]}" if n in reasons else ""
        print(f"  {n:28s} -> {new_name(n)}{tag}")
    if a.dry_run:
        print("\n(--dry-run: nichts geschrieben)")
        return

    prepare_dest(dest, a.force)

    # ---- Crates kopieren und umbenennen
    stats = defaultdict(int)
    golden_changed: list[str] = []
    feature_patched: list[str] = []
    collisions: dict[str, int] = {}
    audit_hits: dict[str, list[str]] = defaultdict(list)
    skipped_exclude = 0
    for name in chosen:
        crate_rel = pk[name]["dir"]
        for rel in git_files(src, crate_rel):
            inner = rel.relative_to(crate_rel)
            top = inner.parts[0]
            if not a.keep_docs and ((len(inner.parts) > 1 and top in EXCLUDE_TOP_DIRS)
                                    or (len(inner.parts) == 1 and top in EXCLUDE_TOP_FILES)):
                skipped_exclude += 1
                continue
            if a.keep_docs and len(inner.parts) > 1 and top in {"fuzz", "target", ".git"}:
                continue
            if a.drop_golden and "golden" in inner.parts:
                skipped_exclude += 1
                continue
            sp = src / rel
            if not sp.is_file():
                continue
            out_rel = Path(*[new_name(part) for part in rel.parts])
            dp = dest / out_rel
            dp.parent.mkdir(parents=True, exist_ok=True)
            data = sp.read_bytes()
            if len(data) < 8_000_000 and is_text(data):
                text = data.decode("utf-8")
                hits = len(re.findall(r"(?i)acta", text))
                if hits:
                    collisions[str(rel)] = hits
                new, n = rename_text(text)
                if inner == Path("Cargo.toml") and not a.no_feature_patch:
                    new, patched = enable_workspace_feature(new)
                    if patched:
                        feature_patched.append(new_name(name))
                dp.write_text(new, encoding="utf-8", newline="")
                stats["text"] += 1
                stats["ersetzungen"] += n
                if n and "golden" in inner.parts:
                    golden_changed.append(out_rel.as_posix())
                if inner.parts[0] == "src" and any(k in name for k in AUDIT_PATH_FILTER):
                    for sid, pat, why in AUDIT_PATTERNS:
                        for ln, line in enumerate(new.split("\n"), 1):
                            if re.search(pat, line):
                                audit_hits[f"{sid} - {why}"].append(f"{out_rel.as_posix()}:{ln}")
            else:
                dp.write_bytes(data)
                stats["binaer"] += 1
            stats["dateien"] += 1

    # ---- Workspace-Dateien
    root_cargo, n_root = rename_text(build_root_cargo(src, sel_dirs, sel_names))
    (dest / "Cargo.toml").write_text(root_cargo, encoding="utf-8")
    for f in ROOT_FILES:
        p = src / f
        if p.exists():
            data = p.read_text(encoding="utf-8")
            (dest / f).write_text(rename_text(data)[0], encoding="utf-8")
    if (src / ".cargo").is_dir():
        for rel in git_files(src, Path(".cargo")):
            (dest / rel).parent.mkdir(parents=True, exist_ok=True)
            (dest / rel).write_text(rename_text((src / rel).read_text(encoding="utf-8"))[0], encoding="utf-8")

    # ---- Bericht (Markdown)
    now = dt.datetime.now().strftime("%Y-%m-%d %H:%M")
    rep = [f"# Extraktionsbericht Contextra -> Acta", "",
           f"- Erstellt: {now}", f"- Quelle: `{src}` (Commit `{commit}`)",
           f"- Profil: `{a.profile if not a.crates else 'eigene Auswahl'}`",
           f"- Dateien: {stats['dateien']} ({stats['text']} Text, {stats['binaer']} binaer), "
           f"{stats['ersetzungen']} Namensersetzungen, {skipped_exclude} Dateien bewusst ausgelassen", "",
           "## Uebernommene Crates", "", "| Contextra | Acta | Grund |", "|---|---|---|"]
    for n in chosen:
        rep.append(f"| {n} | {new_name(n)} | {reasons.get(n, 'gewaehlt')} |")
    rep += ["", "## Bewusst nicht uebernommen", "",
            "- Crate-spezifische `SPEC.md`, `AGENTS.md`, `PLAN.md`, `MODULE.md`, `API.md`, `CHANGELOG.md`, `docs/`, "
            "`tools/`, `capabilities.fragment.toml` (SPEC 3.0: ungueltig) - mit `--keep-docs` aenderbar.",
            "- `fuzz/`-Verzeichnisse (eigene Cargo-Projekte mit Pfaden ausserhalb des Workspace).",
            "- Alle Crates, die nicht gewaehlt und nicht benoetigt wurden (z. B. db, ingest, privacy, router, agent, mcp, infer-*)."]
    rep += ["", "## Umbenennung", "",
            "Ersetzt wurde in Dateiinhalten und Pfaden: `CONTEXTRA`->`ACTA`, `Contextra`->`Acta`, `contextra`->`acta`. "
            "Das betrifft auch Domaenen-Strings der Kryptografie (z. B. `contextra-wal-v1` -> `acta-wal-v1`). "
            "Bestehende Contextra-Daten sind damit nicht mehr lesbar; fuer einen Neustart ist das gewollt."]
    if feature_patched:
        rep += ["", "## Automatische Anpassung: Feature `workspace`", "",
                "In Contextra aktivieren `db`, `agent` und `cognition` das Feature `workspace` der unten genannten "
                "Crates fuer den gesamten Workspace (Cargo-Feature-Unification). Da diese Crates hier fehlen, wurde "
                "`workspace` in deren `default`-Features aufgenommen, damit sich der Code wie in Contextra verhaelt. "
                "Das ist die Altlast V-16 der SPEC 3.0 (Features muessen rein additiv sein) und sollte spaeter "
                "bereinigt werden, indem `acta-types`/`acta-ports` feste Abhaengigkeiten werden.", ""]
        rep += [f"- `{c}`" for c in sorted(feature_patched)]
    if golden_changed:
        rep += ["", "## Golden-Dateien, die durch die Umbenennung veraendert wurden", "",
                "Diese Dateien muessen mit der jeweiligen `regen.sh` bzw. dem Generator-Test neu erzeugt werden, "
                "sonst schlagen die Golden-Tests fehl:", ""]
        rep += [f"- `{g}`" for g in sorted(set(golden_changed))[:60]]
    if collisions:
        rep += ["", "## Hinweis: Dateien, die schon vor der Umbenennung `acta` enthielten", "",
                "Diese Treffer koennen zu Namenskollisionen fuehren. Bitte stichprobenartig pruefen:", ""]
        rep += [f"- `{k}` ({v}x)" for k, v in sorted(collisions.items())[:30]]
    rep += ["", "## Bekannte Befunde der SPEC 3.0 im uebernommenen Code", "",
            "Das Skript uebernimmt den Contextra-Stand unveraendert. Folgende Muster aus dem Befundregister "
            "(SPEC Abschnitt 0.2) sind im Ergebnis noch vorhanden (Treffer in `src/`, Tests nicht gesondert gefiltert):", ""]
    if audit_hits:
        for k, v in sorted(audit_hits.items()):
            rep.append(f"- **{k}** - {len(v)} Treffer, z. B. " + ", ".join(f"`{x}`" for x in v[:4]))
    else:
        rep.append("- keine Treffer")
    uses_xchacha = any("chacha20poly1305" in p.read_text(encoding="utf-8") for p in (dest / "crates").rglob("Cargo.toml"))
    if not uses_xchacha:
        rep += ["- **SPEC 4.1** - keine Abhaengigkeit auf `chacha20poly1305` gefunden; die SPEC verlangt XChaCha20-Poly1305 "
                "(im Code heute AES-256-GCM-SIV)."]
    rep += ["", "## Naechste Schritte", "",
            "1. `cargo check --workspace` im Zielverzeichnis (Rust 1.89, siehe `rust-toolchain.toml`).",
            "2. Befunde oben gemaess SPEC 3.0 beheben, bevor etwas Sicherheitsrelevantes darauf aufgebaut wird.",
            "3. `cargo test --workspace --no-fail-fast`; nicht kompilierende oder rote Tests einzeln triagieren.",
            "4. Golden-Dateien neu erzeugen (siehe oben), falls aufgefuehrt."]
    (dest / "EXTRACT_REPORT.md").write_text("\n".join(rep) + "\n", encoding="utf-8")

    print(f"\nFertig: {stats['dateien']} Dateien nach {dest}")
    print(f"Bericht: {dest / 'EXTRACT_REPORT.md'}")
    for k, v in sorted(audit_hits.items()):
        print(f"  Befund {k}: {len(v)} Treffer")

    if a.verify:
        print("\n== cargo check --workspace (kann dauern) ==")
        r = subprocess.run(["cargo", "check", "--workspace"], cwd=dest)
        sys.exit(r.returncode)


if __name__ == "__main__":
    main()
