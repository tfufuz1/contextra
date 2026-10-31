"""Gemeinsame Basis: Repo-Erkennung, Workspace-Modell, ChangeSet (Dry-Run/Apply), Rust-Hilfen."""
from __future__ import annotations

import bisect
import difflib
import glob as _glob
import os
import re
import shutil
import subprocess
import tomllib
from dataclasses import dataclass, field
from pathlib import Path

SKIP_DIRS = {".git", "target", "node_modules", ".venv", "__pycache__", ".jules", ".idea", ".mypy_cache"}
TEXT_EXT = {".rs", ".toml", ".md", ".yml", ".yaml", ".json", ".py", ".nix", ".txt", ".sh", ".lock", ""}
DEP_TABLES = {"dependencies": "normal", "dev-dependencies": "dev", "build-dependencies": "build"}
SEV_ORDER = {"K": 0, "H": 1, "M": 2, "L": 3, "I": 4}
SEV_NAME = {"K": "kritisch", "H": "hoch", "M": "mittel", "L": "niedrig", "I": "info"}


class ToolError(Exception):
    """Erwarteter Fehler mit verständlicher Meldung (Exit 1)."""


# ----------------------------------------------------------------------------- Repo
def find_root(start: str | os.PathLike | None = None) -> Path:
    p = Path(start or os.getcwd()).resolve()
    for d in [p, *p.parents]:
        f = d / "Cargo.toml"
        if f.is_file():
            try:
                if "workspace" in tomllib.loads(f.read_text(encoding="utf-8")):
                    return d
            except (tomllib.TOMLDecodeError, UnicodeDecodeError):
                continue
    raise ToolError(f"Kein Cargo-Workspace ab {p} gefunden (--repo angeben).")


def walk_files(root: Path, exts: set[str] | None = None, skip_extra: set[str] | None = None):
    """Alle Dateien (relativ, POSIX) unter root ohne SKIP_DIRS."""
    skip = SKIP_DIRS | (skip_extra or set())
    for dp, dns, fns in os.walk(root):
        dns[:] = sorted(d for d in dns if d not in skip)
        for fn in sorted(fns):
            if exts is None or Path(fn).suffix in exts:
                yield (Path(dp) / fn).relative_to(root).as_posix()


def read_text(root: Path, rel: str) -> str:
    return (root / rel).read_text(encoding="utf-8", errors="replace")


def git(root: Path, *args: str, check: bool = False) -> subprocess.CompletedProcess:
    return subprocess.run(["git", "-C", str(root), *args], capture_output=True, text=True, check=check)


def is_git(root: Path) -> bool:
    return (root / ".git").exists()


# ----------------------------------------------------------------------------- ChangeSet
class ChangeSet:
    """Gesammelte Änderungen: Dry-Run ist Standard, `apply()` schreibt.

    Reihenfolge beim Anwenden: Textänderungen (alte Pfade) -> Dateilöschungen -> Verzeichnis-Moves
    -> Verzeichnis-Löschungen. Damit bleiben Pfade in den Edits bis zuletzt gültig.
    """

    def __init__(self, root: Path):
        self.root = root
        self.texts: dict[str, str] = {}
        self.deleted_files: set[str] = set()
        self.moves: list[tuple[str, str]] = []
        self.deleted_dirs: list[str] = []
        self.notes: list[str] = []

    def read(self, rel: str) -> str:
        return self.texts[rel] if rel in self.texts else read_text(self.root, rel)

    def write(self, rel: str, text: str) -> None:
        if rel in self.texts or not (self.root / rel).exists() or read_text(self.root, rel) != text:
            self.texts[rel] = text

    def delete_file(self, rel: str) -> None:
        self.deleted_files.add(rel)

    def delete_dir(self, rel: str) -> None:
        self.deleted_dirs.append(rel)

    def move(self, src: str, dst: str) -> None:
        self.moves.append((src, dst))

    def empty(self) -> bool:
        return not (self.texts or self.deleted_files or self.moves or self.deleted_dirs)

    def changed_files(self) -> list[str]:
        out = []
        for rel, new in sorted(self.texts.items()):
            if (self.root / rel).exists() and read_text(self.root, rel) == new:
                continue
            out.append(rel)
        return out

    def diff(self, context: int = 1) -> str:
        chunks = []
        for rel in self.changed_files():
            old = read_text(self.root, rel) if (self.root / rel).exists() else ""
            chunks.append("".join(difflib.unified_diff(
                old.splitlines(keepends=True), self.texts[rel].splitlines(keepends=True),
                f"a/{rel}", f"b/{rel}", n=context)))
        return "".join(chunks)

    def summary(self) -> str:
        lines = [f"- {len(self.changed_files())} Datei(en) geändert",
                 f"- {len(self.deleted_files)} Datei(en) gelöscht",
                 f"- {len(self.moves)} Verzeichnis-Move(s)",
                 f"- {len(self.deleted_dirs)} Verzeichnis(se) gelöscht"]
        for s, d in self.moves:
            lines.append(f"  - move `{s}` -> `{d}`")
        for d in self.deleted_dirs:
            lines.append(f"  - delete `{d}/`")
        return "\n".join(lines)

    def apply(self) -> None:
        for rel in self.changed_files():
            p = self.root / rel
            p.parent.mkdir(parents=True, exist_ok=True)
            tmp = p.with_name(p.name + ".ctxrs.tmp")
            tmp.write_text(self.texts[rel], encoding="utf-8", newline="")
            os.replace(tmp, p)
        use_git = is_git(self.root)
        for rel in sorted(self.deleted_files):
            if use_git and git(self.root, "ls-files", "--error-unmatch", rel).returncode == 0:
                git(self.root, "rm", "-q", "-f", "--", rel)
            elif (self.root / rel).exists():
                (self.root / rel).unlink()
        for src, dst in self.moves:
            (self.root / dst).parent.mkdir(parents=True, exist_ok=True)
            r = git(self.root, "mv", src, dst) if use_git else None
            if r is None or r.returncode != 0:
                shutil.move(str(self.root / src), str(self.root / dst))
        for rel in self.deleted_dirs:
            if not (self.root / rel).exists():
                continue
            r = git(self.root, "rm", "-r", "-q", "-f", "--", rel) if use_git else None
            if (r is None or r.returncode != 0) and (self.root / rel).exists():
                shutil.rmtree(self.root / rel)


# ----------------------------------------------------------------------------- Findings
@dataclass
class Finding:
    id: str
    sev: str
    title: str
    where: str = ""
    detail: str = ""
    fix: str = ""

    def key(self) -> str:
        return f"{self.id}|{self.where}|{self.title}"


def render_findings(fs: list[Finding], limit_per_id: int = 25) -> str:
    if not fs:
        return "Keine Befunde.\n"
    out = []
    by: dict[str, list[Finding]] = {}
    for f in fs:
        by.setdefault(f.id, []).append(f)
    for fid, items in sorted(by.items(), key=lambda kv: (SEV_ORDER[kv[1][0].sev], kv[0])):
        f0 = items[0]
        out.append(f"### {fid} [{f0.sev}] {f0.title} ({len(items)})\n")
        if f0.fix:
            out.append(f"*Behebung:* {f0.fix}\n")
        for f in items[:limit_per_id]:
            tail = f" — {f.detail}" if f.detail else ""
            out.append(f"- `{f.where}`{tail}")
        if len(items) > limit_per_id:
            out.append(f"- … {len(items) - limit_per_id} weitere (`--full`)")
        out.append("")
    return "\n".join(out) + "\n"


def md_table(head: list[str], rows: list[list]) -> str:
    out = ["| " + " | ".join(head) + " |", "| " + " | ".join("---" for _ in head) + " |"]
    for r in rows:
        out.append("| " + " | ".join(str(c).replace("|", "\\|") for c in r) + " |")
    return "\n".join(out) + "\n"


# ----------------------------------------------------------------------------- Workspace
@dataclass
class Dep:
    key: str                  # Schlüssel in der Tabelle (= Rust-Identifier-Basis)
    pkg: str                  # echter Paketname (package = ... oder workspace-Auflösung)
    kind: str                 # normal | dev | build
    target: str | None        # cfg(...)-Plattform oder None
    optional: bool
    workspace: bool
    path: str | None          # aufgelöster Pfad relativ zum Repo-Root (nur interne Deps)
    table: tuple[str, ...]    # TOML-Tabelle, in der der Eintrag steht

    @property
    def ident(self) -> str:
        return self.key.replace("-", "_")


@dataclass
class Crate:
    name: str
    dir: str                  # relativ, POSIX
    manifest: str
    data: dict
    deps: list[Dep] = field(default_factory=list)

    @property
    def features(self) -> dict[str, list[str]]:
        return self.data.get("features", {}) or {}


class Workspace:
    def __init__(self, root: Path):
        self.root = root
        self.manifest_text = read_text(root, "Cargo.toml")
        self.data = tomllib.loads(self.manifest_text)
        ws = self.data.get("workspace", {})
        self.members_decl: list[str] = list(ws.get("members", []))
        self.default_members: list[str] = list(ws.get("default-members", []))
        self.exclude: list[str] = list(ws.get("exclude", []))
        self.ws_deps: dict[str, object] = dict(ws.get("dependencies", {}))
        self.members: list[str] = []
        for m in self.members_decl:
            if any(ch in m for ch in "*?["):
                self.members += sorted(
                    Path(p).relative_to(root).as_posix() for p in _glob.glob(str(root / m)) if Path(p).is_dir())
            else:
                self.members.append(m.rstrip("/"))
        self.missing_members = [m for m in self.members if not (root / m / "Cargo.toml").is_file()]
        self.crates: dict[str, Crate] = {}
        self.by_dir: dict[str, Crate] = {}
        for m in self.members:
            mf = root / m / "Cargo.toml"
            if not mf.is_file():
                continue
            data = tomllib.loads(mf.read_text(encoding="utf-8"))
            name = data.get("package", {}).get("name")
            if not name:
                continue
            c = Crate(name, m, f"{m}/Cargo.toml", data)
            self.crates[name] = c
            self.by_dir[m] = c
        # root-Paket (falls vorhanden)
        if "package" in self.data:
            c = Crate(self.data["package"]["name"], ".", "Cargo.toml", self.data)
            self.crates.setdefault(c.name, c)
            self.by_dir.setdefault(".", c)
        for c in self.crates.values():
            c.deps = list(self._deps_of(c))

    # -- Abhängigkeiten ------------------------------------------------------------
    def ws_dep_pkg(self, key: str) -> str:
        spec = self.ws_deps.get(key)
        if isinstance(spec, dict):
            return spec.get("package", key)
        return key

    def ws_dep_path(self, key: str) -> str | None:
        spec = self.ws_deps.get(key)
        if isinstance(spec, dict) and "path" in spec:
            return os.path.normpath(spec["path"]).replace("\\", "/")
        return None

    def _deps_of(self, c: Crate):
        d = c.data
        groups: list[tuple[str, str | None, dict, tuple[str, ...]]] = []
        for tname, kind in DEP_TABLES.items():
            if tname in d:
                groups.append((kind, None, d[tname], (tname,)))
        for tgt, tdata in (d.get("target") or {}).items():
            for tname, kind in DEP_TABLES.items():
                if tname in tdata:
                    groups.append((kind, tgt, tdata[tname], ("target", tgt, tname)))
        for kind, tgt, table, tpath in groups:
            for key, spec in table.items():
                s = spec if isinstance(spec, dict) else {}
                is_ws = bool(s.get("workspace"))
                pkg = s.get("package") or (self.ws_dep_pkg(key) if is_ws else key)
                path = None
                if is_ws:
                    path = self.ws_dep_path(key)
                elif "path" in s:
                    path = os.path.normpath(os.path.join(c.dir, s["path"])).replace("\\", "/")
                yield Dep(key, pkg, kind, tgt, bool(s.get("optional")), is_ws, path, tpath)

    def dep_is_dangling(self, dep: Dep) -> bool:
        if dep.path is None:
            return False
        return not (self.root / dep.path / "Cargo.toml").is_file()

    def dangling_pkgs(self) -> set[str]:
        """Paketnamen, auf die ein Workspace-/Pfad-Eintrag zeigt, deren Verzeichnis fehlt."""
        out = {self.ws_dep_pkg(k) for k in self.ws_deps if (p := self.ws_dep_path(k)) and not (self.root / p / "Cargo.toml").is_file()}
        for c in self.crates.values():
            out |= {d.pkg for d in c.deps if self.dep_is_dangling(d)}
        return out

    def graph(self, kinds: tuple[str, ...] = ("normal", "build")) -> dict[str, set[str]]:
        g: dict[str, set[str]] = {n: set() for n in self.crates}
        for n, c in self.crates.items():
            for d in c.deps:
                if d.kind in kinds and d.pkg in self.crates and d.pkg != n:
                    g[n].add(d.pkg)
        return g

    def crate_of_path(self, rel: str) -> Crate | None:
        best = None
        for c in self.crates.values():
            if c.dir != "." and (rel == c.dir or rel.startswith(c.dir + "/")):
                if best is None or len(c.dir) > len(best.dir):
                    best = c
        return best


def tarjan(g: dict[str, set[str]]) -> list[list[str]]:
    index: dict[str, int] = {}
    low: dict[str, int] = {}
    stack: list[str] = []
    on: set[str] = set()
    out: list[list[str]] = []
    counter = [0]

    def sc(v: str):
        # iterativ, um Rekursionstiefe zu vermeiden
        work = [(v, iter(sorted(g.get(v, ()))))]
        index[v] = low[v] = counter[0]; counter[0] += 1
        stack.append(v); on.add(v)
        while work:
            node, it = work[-1]
            adv = False
            for w in it:
                if w not in index:
                    index[w] = low[w] = counter[0]; counter[0] += 1
                    stack.append(w); on.add(w)
                    work.append((w, iter(sorted(g.get(w, ())))))
                    adv = True
                    break
                elif w in on:
                    low[node] = min(low[node], index[w])
            if adv:
                continue
            work.pop()
            if work:
                low[work[-1][0]] = min(low[work[-1][0]], low[node])
            if low[node] == index[node]:
                comp = []
                while True:
                    w = stack.pop(); on.discard(w); comp.append(w)
                    if w == node:
                        break
                if len(comp) > 1:
                    out.append(sorted(comp))

    for v in sorted(g):
        if v not in index:
            sc(v)
    return out


# ----------------------------------------------------------------------------- Rust-Hilfen
_LEX = re.compile(
    r"""//[^\n]*|/\*.*?\*/|(?<![\w])b?r(\#*)".*?"\1|(?<![\w])b?"(?:\\.|[^"\\])*"|"""
    r"""'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^\\'\n])'""", re.S)


def mask_rust(src: str) -> str:
    """Kommentare und Stringinhalte durch Leerzeichen ersetzen (Zeilen/Offsets bleiben stabil)."""
    return _LEX.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), src)


_CFG_TEST = re.compile(r"#\s*\[\s*(?:cfg\s*\(\s*test\s*\)|test|tokio::test[^\]]*|rstest[^\]]*)\s*\]")


def test_spans(masked: str) -> list[tuple[int, int]]:
    """Offsets von `#[cfg(test)]`-Items und `#[test]`-Funktionen (Klammerabgleich)."""
    spans = []
    for m in _CFG_TEST.finditer(masked):
        p = m.end()
        n = len(masked)
        while p < n:                                   # weitere Attribute überspringen
            while p < n and masked[p].isspace():
                p += 1
            if masked.startswith("#[", p):
                depth, q = 0, p + 1
                while q < n:
                    if masked[q] == "[":
                        depth += 1
                    elif masked[q] == "]":
                        depth -= 1
                        if depth == 0:
                            break
                    q += 1
                p = q + 1
                continue
            break
        q = p
        while q < n and masked[q] not in "{;":
            q += 1
        if q >= n or masked[q] == ";":
            spans.append((m.start(), min(q + 1, n)))
            continue
        depth = 0
        while q < n:
            if masked[q] == "{":
                depth += 1
            elif masked[q] == "}":
                depth -= 1
                if depth == 0:
                    break
            q += 1
        spans.append((m.start(), min(q + 1, n)))
    return spans


class LineIndex:
    def __init__(self, text: str):
        self.starts = [0] + [m.end() for m in re.finditer(r"\n", text)]

    def line(self, off: int) -> int:
        return bisect.bisect_right(self.starts, off)  # 1-basiert


def in_spans(off: int, spans: list[tuple[int, int]]) -> bool:
    return any(a <= off < b for a, b in spans)


def rust_files(root: Path, crate_dir: str):
    base = root / crate_dir
    if not base.is_dir():
        return
    for dp, dns, fns in os.walk(base):
        dns[:] = [d for d in dns if d not in SKIP_DIRS]
        for fn in fns:
            if fn.endswith(".rs"):
                yield (Path(dp) / fn).relative_to(root).as_posix()


def file_class(rel: str, crate_dir: str) -> str:
    """prod (src/), test (tests/, tests.rs, *_tests.rs), bench, example, other."""
    sub = rel[len(crate_dir) + 1:] if crate_dir != "." else rel
    parts = sub.split("/")
    head = parts[0]
    cls = {"src": "prod", "tests": "test", "benches": "bench", "examples": "example"}.get(head, "other")
    if cls == "prod":
        stem = parts[-1][:-3] if parts[-1].endswith(".rs") else parts[-1]
        if stem == "tests" or stem.endswith("_tests") or stem.endswith("_test") or "tests" in parts[1:-1]:
            return "test"
    return cls


def hit_class(rel: str, crate_dir: str, off: int, spans: list[tuple[int, int]]) -> str:
    """Wie file_class, zusätzlich: Treffer innerhalb #[cfg(test)]/#[test] zählt als test."""
    cls = file_class(rel, crate_dir)
    return "test" if cls == "prod" and in_spans(off, spans) else cls


_USE_IDENT = re.compile(r"(?<![\w:.$])(contextra_[A-Za-z0-9_]+)\s*::|(?<![\w])extern\s+crate\s+(\w+)")


def crate_idents_used(masked: str, known: set[str]) -> list[tuple[str, int]]:
    """Verwendungen bekannter Crate-Identifier (Name, Offset)."""
    out = []
    for m in _USE_IDENT.finditer(masked):
        ident = m.group(1) or m.group(2)
        if ident in known:
            out.append((ident, m.start(1) if m.group(1) else m.start(2)))
    return out


def pkg_ident(name: str) -> str:
    return name.replace("-", "_")


def load_data(name: str) -> dict:
    p = Path(__file__).parent / "data" / name
    if not p.is_file():
        p = Path(__file__).parent / name
    return tomllib.loads(p.read_text(encoding="utf-8"))
