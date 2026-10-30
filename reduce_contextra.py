#!/usr/bin/env python3
"""
reduce_contextra.py — Contextra 1.0 Reduktion (Spezifikation: Abschnitte 4, 5, 7, Anhang A).

Statisch, ohne Compiler. Idempotent. Standard = Dry-Run; Änderungen nur mit --apply.

Gruppe B und C werden GELÖSCHT (git rm); Wiederherstellung über Tag pre-reduction-1.0.

Phasen
  1  Sicherung         Git-Tag pre-reduction-1.0 (nur --apply, nur wenn Arbeitsbaum sauber)
  2  Crates löschen     Alles außerhalb von v1 (Gruppen B, C, core) wird entfernt; Wiederherstellung per Git-Tag
  3  Dateien entfernen  Funktionsebene in Kern-Crates (Abschnitt 5.1)
  4  Manifeste          Root-Cargo.toml, Kern-Cargo.toml (Deps, Features, Test-/Example-Blöcke)
  5  Quelltext-Header   `mod x;`-Deklarationen entfernter Module löschen
  6  Tests prunen       Test-/Example-Dateien, die ausschließlich Entferntes referenzieren
  7  Ballast            evidence/ aus Git-Index, .gitignore
  8  Verifikation       TOML parsebar, Pfad-Deps vorhanden, Members vorhanden
  9  Residuen-Bericht   REDUCTION_RESIDUALS.md (Eingabe für die Bereinigungs-Prompts)

Aufruf
  python3 reduce_contextra.py [--root PATH] [--apply] [--keep-tests]
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys
import tomllib
from collections import defaultdict
from pathlib import Path

# ───────────────────────────── Konfiguration ─────────────────────────────

CORE = ["types", "ports", "sys", "simd", "mvcc", "crypto", "store",
        "vector", "text", "rank", "engine", "db"]            # Gruppe A (+ Fassade)
TOOLING = ["testkit"]                                          # Gruppe D
GROUP_B = ["infer-ollama", "privacy", "ingest", "mcp", "wire",
           "checkpoint", "audit-export", "avv-generator"]      # später wiederherstellbar
GROUP_C = ["adapt", "agent", "cognition", "graph", "infer-candle", "infer-onnx",
           "kvcache", "router", "sandbox", "py", "license"]    # später wiederherstellbar
DELETE = ["core"]                                              # deprecated

# Dateiebene (relativ zu crates/). Reihenfolge egal.
REMOVE_PATHS = [
    "contextra-vector/src/diskann",
    "contextra-crypto/src/deletion_proof",
    "contextra-crypto/src/kv_segment",
    "contextra-crypto/src/kv_shredding.rs",
    "contextra-crypto/src/revocation_log.rs",
    "contextra-crypto/src/ed25519_proof.rs",
    "contextra-crypto/src/wal_completeness.rs",
    "contextra-store/src/scanner",
    "contextra-engine/src/extraction",
    "contextra-engine/src/decay_controller.rs",
    "contextra-engine/src/contextra_impl/erasure.rs",
    "contextra-db/src/multistep.rs",
    "contextra-db/src/homeostat.rs",
    "contextra-db/examples/verify_deletion_proof_external.rs",
]

# Bench-Teile außerhalb v1 (werden gelöscht)
BENCH = "benchmarks/contextra-bench"
BENCH_EXTERNAL = [
    f"{BENCH}/benches/locomo.rs",
    f"{BENCH}/benches/long_mem_eval.rs",
    f"{BENCH}/src/locomo.rs",
    f"{BENCH}/src/long_mem_eval.rs",
    f"{BENCH}/tests/external_benchmarks_test.rs",
]

# Features je Crate, die entfallen (Abschnitt 5.1/5.3)
REMOVE_FEATURES: dict[str, set[str]] = {
    "contextra-engine": {"onnx", "reranking", "sandbox", "experimental-diskann",
                         "graph-connectivity-health", "edge-reinforcement-learning",
                         "adaptive-candidate-pool-sizing", "entity-extraction",
                         "auto-extraction-opt-out"},
    "contextra-db": {"sandbox", "reranking", "onnx", "experimental-diskann",
                     "graph-connectivity-health", "coherence-bonus-fusion",
                     "edge-reinforcement-learning", "adaptive-candidate-pool-sizing"},
    "contextra-vector": {"experimental-diskann"},
    "contextra": {"compliance", "audit-export", "avv-generator", "candle"},
    "contextra-bench": {"onnx-bench"},
}
ALL_REMOVED_FEATURES = set().union(*REMOVE_FEATURES.values())

# Modulnamen, deren `mod`-Zeile in der Elterndatei verschwindet (aus REMOVE_PATHS abgeleitet + Bench)
EXTRA_MOD_REMOVALS = {
    f"{BENCH}/src/lib.rs": ["locomo", "long_mem_eval"],
}

# Residuen-Muster (Abschnitt 9): Referenzen auf Entferntes in verbleibendem Code
MODULE_IDENTS = (r"diskann|DiskAnn\w*|deletion_proof|DeletionProof\w*|kv_shredding|kv_segment|"
                 r"revocation_log|ed25519_proof|wal_completeness|decay_controller|"
                 r"homeostat|Homeostat\w*|multistep|MultiStep\w*")
SCOPED_IDENTS = r"(?:crate|super|self|contextra_\w+)::(?:scanner|extraction|erasure)\b"


# ───────────────────────────── Hilfsfunktionen ─────────────────────────────

class Ctx:
    def __init__(self, root: Path, apply: bool, keep_tests: bool):
        self.root, self.apply, self.keep_tests = root, apply, keep_tests
        self.log: list[str] = []
        self.pruned_tests: list[str] = []

    def say(self, msg: str) -> None:
        print(msg)
        self.log.append(msg)

    def git(self, *args: str, check: bool = True) -> subprocess.CompletedProcess:
        return subprocess.run(["git", "-C", str(self.root), *args],
                              capture_output=True, text=True, check=check)

    def tracked(self, rel: str) -> bool:
        return bool(self.git("ls-files", "--", rel, check=False).stdout.strip())

    def mv(self, src: str, dst: str) -> None:
        s, d = self.root / src, self.root / dst
        if not s.exists():
            return
        self.say(f"  mv  {src} -> {dst}")
        if not self.apply:
            return
        d.parent.mkdir(parents=True, exist_ok=True)
        if self.tracked(src):
            self.git("mv", src, dst)
        else:
            s.rename(d)

    def rm(self, rel: str) -> None:
        p = self.root / rel
        if not p.exists():
            return
        self.say(f"  rm  {rel}")
        if not self.apply:
            return
        if self.tracked(rel):
            self.git("rm", "-r", "-q", "-f", "--", rel)
        if p.exists():
            if p.is_dir():
                import shutil
                shutil.rmtree(p)
            else:
                p.unlink()

    def write(self, rel: str, text: str, old: str) -> None:
        if text == old:
            return
        self.say(f"  edit {rel}")
        if self.apply:
            (self.root / rel).write_text(text, encoding="utf-8")


def crate_dir(name: str) -> str:
    return "crates/contextra" if name == "contextra" else f"crates/contextra-{name}"


def all_removed_crates() -> list[str]:
    return GROUP_B + GROUP_C + DELETE


# ───────────────────────────── Phase 1 ─────────────────────────────

def phase_backup(c: Ctx) -> None:
    c.say("\n[1] Sicherung")
    if c.git("rev-parse", "--verify", "-q", "refs/tags/pre-reduction-1.0", check=False).returncode == 0:
        c.say("  Tag pre-reduction-1.0 existiert bereits")
        return
    dirty = c.git("status", "--porcelain").stdout.strip()
    if dirty:
        c.say("  WARNUNG: Arbeitsbaum nicht sauber -> Tag wird trotzdem auf HEAD gesetzt, "
              "ungesicherte Änderungen sind NICHT enthalten.")
    c.say("  git tag pre-reduction-1.0")
    if c.apply:
        c.git("tag", "pre-reduction-1.0")


# ───────────────────────────── Phase 2 ─────────────────────────────

def phase_move_crates(c: Ctx) -> None:
    c.say("\n[2] Crates ENTFERNEN (Wiederherstellung: git checkout pre-reduction-1.0 -- <Pfad>)")
    for n in GROUP_C + GROUP_B + DELETE:
        c.rm(crate_dir(n))
    # Reste früherer Läufe (frühere Fassung verschob nach attic/ext/benchmarks/external)
    for d in ("attic", "ext", "benchmarks/external"):
        c.rm(d)
    for rel in BENCH_EXTERNAL:
        c.rm(rel)


# ───────────────────────────── Phase 3 ─────────────────────────────

def phase_remove_paths(c: Ctx) -> None:
    c.say("\n[3] Funktionsebene entfernen")
    for rel in REMOVE_PATHS:
        c.rm(f"crates/{rel}")


# ───────────────────────────── Phase 4: Manifeste ─────────────────────────────

def edit_root_manifest(c: Ctx) -> None:
    rel = "Cargo.toml"
    old = (c.root / rel).read_text(encoding="utf-8")
    text = old

    members = [f"crates/contextra-{n}" for n in CORE] + ["crates/contextra"] \
              + [f"crates/contextra-{n}" for n in TOOLING] + [BENCH, "xtask"]
    default = [f"crates/contextra-{n}" for n in CORE] + ["crates/contextra"]

    def arr(items: list[str]) -> str:
        return "[\n" + "".join(f'    "{i}",\n' for i in items) + "]"

    text = re.sub(r'(?m)^exclude\s*=\s*\["attic"[^\n]*\]\n', "", text)
    # Kommentar zu „Restored archived crates“ entfernen
    text = re.sub(r"# Restored archived crates[^\n]*\n#[^\n]*\n", "", text)
    text = re.sub(r"(?m)^members\s*=\s*\[.*?\]", "members = " + arr(members), text, count=1, flags=re.S)
    text = re.sub(r"(?m)^default-members\s*=\s*\[.*?\]", "default-members = " + arr(default),
                  text, count=1, flags=re.S)

    # [workspace.dependencies]: Pfad-Deps auf entfernte Crates (inkl. Alias-Zeilen) entfernen
    removed = "|".join(re.escape(f"contextra-{n}") for n in all_removed_crates())
    text = re.sub(rf'(?m)^[\w-]+\s*=\s*\{{[^}}\n]*path\s*=\s*"crates/(?:{removed})"[^\n]*\n', "", text)
    c.write(rel, text, old)


def _strip_feature_tokens(section: str, removed: set[str], removed_crates: list[str]) -> str:
    """Entfernt Tokens in Feature-Arrays: entfernte Features, dep:-/crate-Referenzen auf entfernte Crates."""
    crate_alt = "|".join(re.escape(f"contextra-{n}") for n in removed_crates)

    def drop(tok: str) -> bool:
        if tok in removed:
            return True
        if re.fullmatch(rf"dep:(?:{crate_alt})", tok):
            return True
        m = re.fullmatch(rf"(?:{crate_alt})(?:/(.+))?", tok)
        if m:
            return True
        m = re.fullmatch(r"contextra[\w-]*\??/(.+)", tok)
        return bool(m and m.group(1) in ALL_REMOVED_FEATURES)

    return re.sub(r'"([^"\n]+)"[ \t]*,?[ \t]*',
                  lambda m: "" if drop(m.group(1)) else m.group(0), section)


def _remove_feature_keys(section: str, keys: set[str]) -> str:
    out, skip = [], False
    for line in section.splitlines(keepends=True):
        if skip:
            if "]" in line:
                skip = False
            continue
        m = re.match(r"\s*([\w-]+)\s*=", line)
        if m and m.group(1) in keys:
            if "[" in line and "]" not in line:
                skip = True
            continue
        out.append(line)
    return "".join(out)


def edit_crate_manifest(c: Ctx, rel: str, pkg: str) -> None:
    path = c.root / rel
    if not path.exists():
        return
    old = path.read_text(encoding="utf-8")
    text = old
    removed_crates = all_removed_crates()
    crate_alt = "|".join(re.escape(f"contextra-{n}") for n in removed_crates)
    removed_feats = REMOVE_FEATURES.get(pkg, set())

    # 1. Dependency-Zeilen auf entfernte Crates (einzeilig) und Tabellenform
    text = re.sub(rf"(?m)^(?:{crate_alt})(?:\s*=|\.[\w-]+\s*=)[^\n]*\n", "", text)
    text = re.sub(rf"(?ms)^\[(?:[\w.-]+\.)?dependencies\.(?:{crate_alt})\]\n.*?(?=^\[|\Z)", "", text)

    # 2. [features]-Sektion: Schlüssel und Tokens
    m = re.search(r"(?ms)^\[features\]\n(.*?)(?=^\[|\Z)", text)
    if m:
        body = _remove_feature_keys(m.group(1), removed_feats)
        body = _strip_feature_tokens(body, removed_feats, removed_crates)
        text = text[:m.start(1)] + body + text[m.end(1):]

    # 3. Inline `features = [..]` in Dependency-Zeilen
    def inline(mm: re.Match) -> str:
        return mm.group(1) + _strip_feature_tokens(mm.group(2), ALL_REMOVED_FEATURES, removed_crates) + mm.group(3)
    text = re.sub(r"(features\s*=\s*\[)([^\]\n]*)(\])", inline, text)

    # 4. [[test]]/[[example]]/[[bench]]/[[bin]]-Blöcke
    text = _prune_target_blocks(c, path.parent, text, ALL_REMOVED_FEATURES)
    text = re.sub(r"\n{3,}", "\n\n", text)
    c.write(rel, text, old)


def _prune_target_blocks(c: Ctx, base: Path, text: str, removed_feats: set[str]) -> str:
    parts = re.split(r"(?m)^(?=\[)", text)
    kept = []
    defaults = {"test": "tests", "example": "examples", "bench": "benches", "bin": "src/bin"}
    for part in parts:
        m = re.match(r"\[\[(test|example|bench|bin)\]\]", part)
        if not m:
            kept.append(part)
            continue
        kind = m.group(1)
        name = re.search(r'(?m)^name\s*=\s*"([^"]+)"', part)
        pth = re.search(r'(?m)^path\s*=\s*"([^"]+)"', part)
        req = re.search(r"(?m)^required-features\s*=\s*\[([^\]]*)\]", part)
        drop = False
        if req and any(f in removed_feats for f in re.findall(r'"([^"]+)"', req.group(1))):
            drop = True
        if not drop and name:
            files = [base / pth.group(1)] if pth else [
                base / defaults[kind] / f"{name.group(1)}.rs",
                base / defaults[kind] / name.group(1) / "main.rs",
                base / "src/main.rs" if kind == "bin" else base / "__none__"]
            # Dateien, die in diesem Lauf noch existieren, zählen; im Dry-Run ist nichts gelöscht
            if not any(f.exists() for f in files):
                drop = True
        if drop:
            continue
        kept.append(part)
    return "".join(kept)


def phase_manifests(c: Ctx) -> None:
    c.say("\n[4] Manifeste")
    edit_root_manifest(c)
    for n in CORE + TOOLING:
        edit_crate_manifest(c, f"{crate_dir(n)}/Cargo.toml", f"contextra-{n}")
    edit_crate_manifest(c, "crates/contextra/Cargo.toml", "contextra")
    edit_crate_manifest(c, f"{BENCH}/Cargo.toml", "contextra-bench")
    edit_crate_manifest(c, "xtask/Cargo.toml", "xtask")


# ───────────────────────────── Phase 5: mod-Zeilen ─────────────────────────────

def remove_mod_decl(c: Ctx, parent_candidates: list[Path], mod_name: str) -> None:
    pat = re.compile(
        rf"(?m)^(?:[ \t]*(?:#\[[^\n]*\]|///[^\n]*|//![^\n]*)[ \t]*\n)*"
        rf"[ \t]*(?:pub(?:\([\w: ]+\))?[ \t]+)?mod[ \t]+{re.escape(mod_name)}[ \t]*;[ \t]*\n")
    for f in parent_candidates:
        if f.exists():
            old = f.read_text(encoding="utf-8")
            new = pat.sub("", old)
            if new != old:
                c.write(str(f.relative_to(c.root)), new, old)
                # zugehörige `pub use <mod>::…;` ebenfalls entfernen
                new2 = re.sub(rf"(?m)^(?:[ \t]*#\[[^\n]*\][ \t]*\n)*[ \t]*pub[ \t]+use[ \t]+{re.escape(mod_name)}::[^;]*;[ \t]*\n", "", new)
                if new2 != new:
                    c.write(str(f.relative_to(c.root)), new2, new)
            return


def phase_mod_decls(c: Ctx) -> None:
    c.say("\n[5] mod-Deklarationen")
    for rel in REMOVE_PATHS:
        p = Path("crates") / rel
        if p.parts[-2] == "examples":
            continue
        mod = p.stem if p.suffix == ".rs" else p.name
        parent = c.root / p.parent
        remove_mod_decl(c, [parent / "lib.rs", parent / "mod.rs", parent.with_suffix(".rs")], mod)
    for rel, mods in EXTRA_MOD_REMOVALS.items():
        for m in mods:
            remove_mod_decl(c, [c.root / rel], m)


# ───────────────────────────── Phase 6: Tests prunen ─────────────────────────────

def residual_regex(extra_features: set[str]) -> re.Pattern:
    crate_idents = "|".join(re.escape(f"contextra_{n.replace('-', '_')}") for n in all_removed_crates())
    feats = "|".join(re.escape(f) for f in sorted(extra_features))
    parts = [rf"\b(?:{crate_idents})\b", rf"\b(?:{MODULE_IDENTS})\b", SCOPED_IDENTS]
    if feats:
        parts.append(rf'feature\s*=\s*"(?:{feats})"')
    return re.compile("|".join(parts))


def kept_crate_roots(c: Ctx) -> list[Path]:
    roots = [c.root / crate_dir(n) for n in CORE + TOOLING] + [c.root / "crates/contextra", c.root / BENCH]
    return [r for r in roots if r.exists()]


# Prune-Policy: ganze Datei nur löschen, wenn ihr Zweck das Entfernte ist.
# Einzeltreffer (z. B. `use contextra_core::TenantId`) bleiben und gehen in den Residuen-Bericht (Prompts).
PRUNE_NAME = re.compile(r"deletion|erasure|diskann|graph|cognition|consolidat|license|multistep|homeostat|"
                        r"extraction|decay|compliance|agent_memory|sandbox|hyperedge|zettelkasten|pathrag|"
                        r"community|adapt|onnx|candle|checkpoint|locomo|long_mem", re.I)
PRUNE_MIN_HITS = 8


def phase_prune_tests(c: Ctx) -> None:
    c.say("\n[6] Tests/Examples prunen" + (" (übersprungen: --keep-tests)" if c.keep_tests else ""))
    if c.keep_tests:
        return
    rx_hard = residual_regex(set())      # Crate-/Modul-Idents, ohne cfg(feature)
    for root in kept_crate_roots(c):
        for sub in ("tests", "examples", "benches"):
            d = root / sub
            if not d.exists():
                continue
            for f in sorted(d.rglob("*.rs")):
                txt = f.read_text(encoding="utf-8", errors="replace")
                hits = sum(1 for ln in txt.splitlines() if rx_hard.search(ln))
                if PRUNE_NAME.search(f.stem) and hits or hits >= PRUNE_MIN_HITS:
                    rel = str(f.relative_to(c.root))
                    c.pruned_tests.append(rel)
                    c.rm(rel)
    # Manifeste nach dem Pruning erneut bereinigen ([[test]]-Blöcke ohne Datei)
    if c.apply:
        for n in CORE + TOOLING:
            edit_crate_manifest(c, f"{crate_dir(n)}/Cargo.toml", f"contextra-{n}")
        edit_crate_manifest(c, "crates/contextra/Cargo.toml", "contextra")
        edit_crate_manifest(c, f"{BENCH}/Cargo.toml", "contextra-bench")


# ───────────────────────────── Phase 7: Ballast ─────────────────────────────

def phase_ballast(c: Ctx) -> None:
    c.say("\n[7] Repo-Ballast")
    if (c.root / "evidence").exists():
        c.say("  git rm -r --cached evidence + .gitignore")
        if c.apply:
            if c.tracked("evidence"):
                c.git("rm", "-r", "-q", "--cached", "--", "evidence")
            gi = c.root / ".gitignore"
            txt = gi.read_text(encoding="utf-8") if gi.exists() else ""
            if not re.search(r"(?m)^evidence/\s*$", txt):
                gi.write_text(txt.rstrip("\n") + "\nevidence/\n", encoding="utf-8")


# ───────────────────────────── Phase 8: Verifikation ─────────────────────────────

def phase_verify(c: Ctx) -> int:
    c.say("\n[8] Verifikation (statisch)")
    if not c.apply:
        c.say("  Dry-Run: übersprungen")
        return 0
    problems = 0
    root_toml = tomllib.loads((c.root / "Cargo.toml").read_text(encoding="utf-8"))
    ws = root_toml["workspace"]
    for m in ws["members"]:
        if not (c.root / m / "Cargo.toml").exists():
            c.say(f"  FEHLT member: {m}")
            problems += 1
    for k, v in ws.get("dependencies", {}).items():
        if isinstance(v, dict) and "path" in v and not (c.root / v["path"] / "Cargo.toml").exists():
            c.say(f"  FEHLT workspace.dependency-Pfad: {k} -> {v['path']}")
            problems += 1
    names = {k for k, v in ws.get("dependencies", {}).items()}
    for m in ws["members"]:
        mp = c.root / m / "Cargo.toml"
        try:
            t = tomllib.loads(mp.read_text(encoding="utf-8"))
        except Exception as e:  # noqa: BLE001
            c.say(f"  TOML-Fehler {mp.relative_to(c.root)}: {e}")
            problems += 1
            continue
        feats = set(t.get("features", {}))
        for sect in ("dependencies", "dev-dependencies", "build-dependencies"):
            for dep, spec in t.get(sect, {}).items():
                if isinstance(spec, dict) and spec.get("workspace") and dep not in names:
                    c.say(f"  {m}: [{sect}] {dep} ohne workspace.dependency")
                    problems += 1
        for f, items in t.get("features", {}).items():
            for it in items:
                base = it.split("/")[0].removeprefix("dep:").rstrip("?")
                if "/" not in it and not it.startswith("dep:") and it not in feats:
                    c.say(f"  {m}: feature '{f}' referenziert unbekanntes Feature '{it}'")
                    problems += 1
                if base.startswith("contextra") and base not in names and base != t["package"]["name"]:
                    c.say(f"  {m}: feature '{f}' referenziert Crate '{base}' außerhalb des Workspace")
                    problems += 1
    c.say(f"  Probleme: {problems}")
    return problems


# ───────────────────────────── Phase 9: Residuen ─────────────────────────────

def phase_residuals(c: Ctx) -> None:
    c.say("\n[9] Residuen-Bericht")
    rx = residual_regex(ALL_REMOVED_FEATURES)
    by_crate: dict[str, list[tuple[str, int, str]]] = defaultdict(list)
    for root in kept_crate_roots(c):
        for f in sorted(root.rglob("*.rs")):
            if "target" in f.parts:
                continue
            try:
                lines = f.read_text(encoding="utf-8", errors="replace").splitlines()
            except OSError:
                continue
            for i, line in enumerate(lines, 1):
                if rx.search(line):
                    by_crate[root.name].append((str(f.relative_to(c.root)), i, line.strip()[:140]))
    # Manifest-/Dokumentreste
    doc_hits: list[str] = []
    for f in ["README.md", "CLAUDE.md", "AGENTS.md", "capabilities.toml", "justfile", "docs/SPEC.md"]:
        p = c.root / f
        if p.exists():
            n = len(re.findall("|".join(f"contextra-{x}" for x in all_removed_crates()),
                               p.read_text(encoding="utf-8", errors="replace")))
            if n:
                doc_hits.append(f"| `{f}` | {n} |")
    out = ["# Residuen nach Reduktion", "",
           "Treffer = Referenzen auf entfernte Crates, Module, Features in verbleibendem Code.",
           "Eingabe für die Prompts P1–P6 (`REDUCTION_PROMPTS.md`).", "",
           "## Zusammenfassung", "", "| Crate | Treffer | Dateien |", "|---|---:|---:|"]
    for crate, hits in sorted(by_crate.items(), key=lambda kv: -len(kv[1])):
        out.append(f"| `{crate}` | {len(hits)} | {len({h[0] for h in hits})} |")
    if not by_crate:
        out.append("| – | 0 | 0 |")
    out += ["", "## Gelöschte Test-/Example-Dateien (restorebar via `git checkout pre-reduction-1.0 -- <Pfad>`)", ""]
    out += [f"- `{t}`" for t in c.pruned_tests] or ["- keine"]
    out += ["", "## Dokumente mit Verweisen auf entfernte Crates", "",
            "| Datei | Verweise |", "|---|---:|"] + (doc_hits or ["| – | 0 |"])
    out += ["", "## Treffer je Crate", ""]
    for crate, hits in sorted(by_crate.items(), key=lambda kv: -len(kv[1])):
        out += [f"### `{crate}` ({len(hits)})", "", "```"]
        out += [f"{p}:{ln}: {txt}" for p, ln, txt in hits[:400]]
        if len(hits) > 400:
            out.append(f"… {len(hits) - 400} weitere")
        out += ["```", ""]
    report = c.root / "REDUCTION_RESIDUALS.md"
    c.say(f"  {sum(len(v) for v in by_crate.values())} Treffer in {len(by_crate)} Crates -> {report.name}"
          + ("" if c.apply else " (Dry-Run: Bericht beschreibt den NOCH nicht reduzierten Stand nicht; nicht geschrieben)"))
    if c.apply:
        report.write_text("\n".join(out) + "\n", encoding="utf-8")


# ───────────────────────────── main ─────────────────────────────

def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--root", default=".", help="Repository-Wurzel")
    ap.add_argument("--apply", action="store_true", help="Änderungen ausführen (sonst Dry-Run)")
    ap.add_argument("--keep-tests", action="store_true", help="Tests/Examples mit Entfernt-Referenzen nicht löschen")
    a = ap.parse_args()

    root = Path(a.root).resolve()
    if not (root / "Cargo.toml").exists() or not (root / "crates").is_dir():
        print("Keine Contextra-Wurzel (Cargo.toml/crates fehlt)", file=sys.stderr)
        return 2
    c = Ctx(root, a.apply, a.keep_tests)
    c.say(f"Modus: {'APPLY' if a.apply else 'DRY-RUN'}  Root: {root}")
    phase_backup(c)
    phase_move_crates(c)
    phase_remove_paths(c)
    phase_manifests(c)
    phase_mod_decls(c)
    phase_prune_tests(c)
    phase_ballast(c)
    problems = phase_verify(c)
    phase_residuals(c)
    c.say("\nNächste Schritte: REDUCTION_RESIDUALS.md lesen, Prompts P1–P6 abarbeiten, "
          "danach `cargo check --workspace --all-targets --locked`." if a.apply else
          "\nDry-Run beendet. Mit --apply ausführen.")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
