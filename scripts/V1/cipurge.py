"""`ctxrs ci-purge` – Verweise auf das nicht mehr vorhandene `xtask` neutralisieren (Befund B-02 / D07).

Vorgehen (Prinzip: nichts stillschweigend grün machen, jede Änderung reversibel und sichtbar):
  * `.cargo/config.toml`: Alias `xtask = ...` entfernen (leere [alias]-Tabelle mit)
  * `justfile`: xtask-Zeilen durch eine laute Warnung ersetzen (`@echo "[ctxrs] …" >&2`), Original im Kommentar
  * Workflows (zeilenbasiertes YAML-Lite, keine PyYAML-Abhängigkeit):
      - Step mit ausschließlich xtask-Befehlen  -> auskommentiert (`# `-Präfix, Einrückung bleibt)
      - Step mit gemischten Befehlen            -> NICHT angefasst, steht in der Liste „manuell“
      - Cache-Zeile `xtask` unter `workspaces:` / `paths:`-Eintrag `xtask/**` -> Zeile entfernt
      - Job ohne aktive Steps                   -> Job auskommentiert, `needs:`-Verweise bereinigt
      - Workflow ohne aktive Jobs               -> mit --delete-empty gelöscht, sonst nur gemeldet
Ersatz für entfernte Gates: `--emit-workflow` schreibt .github/workflows/ctxrs-gates.yml (doctor + archgate).
Hinweis: `.github/**` und `justfile` stehen in governance/protected-paths.toml – PR braucht ADR-Trailer/Label.
"""
from __future__ import annotations

import re
from pathlib import Path

from . import tomledit as te
from .lib import ChangeSet, Workspace, md_table, read_text

XT_CMD = re.compile(
    r"(?<![\w/.-])(?:cargo\s+xtask\b|cargo\s+run\s+(?:[^\n]*?\s)?(?:-p|--package)\s+xtask\b|"
    r"cargo\s+(?:build|test|clippy|check)\s+(?:[^\n]*?\s)?(?:-p|--package)\s+xtask\b|"
    r"(?:--manifest-path|working-directory:)\s*xtask\b|xtask/Cargo\.toml)")
XT_WORD = re.compile(r"(?<![\w-])xtask(?![\w-])")
CMD_START = re.compile(r"^\s*(?:-\s+)?(?:run:\s*)?(cargo|just|python3?|bash|sh|\./|make|npm|git|nix|rustup|curl)\b")
HINTS = {
    "check-ring-layering": "ctxrs archgate (R1/R2)", "panic-inventory": "ctxrs archgate --rules R6",
    "security-scan": "ctxrs archgate --rules R3,R5,R7", "check-unsafe-islands": "ctxrs archgate --rules R3,R3b",
    "check-fbs-drift": "entfällt mit contextra-wire", "check-flatbuffers-drift": "entfällt mit contextra-wire",
}


def _indent(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


def _cmd_hint(line: str) -> str:
    m = re.search(r"xtask\s+(?:--\s+)?([\w-]+)", line)
    return HINTS.get(m.group(1), "") if m else ""


def justfile(text: str):
    out, hits = [], []
    for i, ln in enumerate(text.splitlines(keepends=True), 1):
        body = ln.rstrip("\n")
        if XT_WORD.search(body) and XT_CMD.search(body) and not body.lstrip().startswith("#"):
            ind = body[: len(body) - len(body.lstrip())]
            hint = _cmd_hint(body)
            msg = f"[ctxrs] entfernt (xtask fehlt): {body.strip()}" + (f" -> {hint}" if hint else "")
            msg = msg.replace('"', "'")
            out.append(f'{ind}@echo "{msg[:200]}" >&2\n')
            hits.append((i, body.strip()))
        else:
            out.append(ln)
    return "".join(out), hits


def cargo_alias(text: str):
    ed = te.Editor(text)
    n = 0
    for s in ed.kvs(("alias",)):
        if s.key == ("xtask",):
            ed.delete_stmt(s)
            n += 1
    if n and len(ed.kvs(("alias",))) == n:
        ed.delete_table(("alias",))
    new = ed.result()
    te.validate(new, ".cargo/config.toml")
    return new, n


# ----------------------------------------------------------------------------- Workflows
class Wf:
    def __init__(self, text: str):
        self.lines = text.splitlines(keepends=True)
        self.comment: set[int] = set()
        self.delete: set[int] = set()
        self.manual: list[tuple[int, str]] = []
        self.commented_steps = 0
        self.commented_jobs: list[str] = []
        self.jobs_total = 0

    def _block_end(self, start: int, ind: int, stop: int) -> int:
        j = start + 1
        while j < stop:
            ln = self.lines[j]
            if ln.strip() and not ln.lstrip().startswith("#") and _indent(ln) <= ind:
                break
            j += 1
        return j

    def analyse(self) -> None:
        L = self.lines
        # Jobs finden
        jobs_i = next((i for i, l in enumerate(L) if re.match(r"^jobs:\s*(#.*)?$", l)), None)
        jobs: list[tuple[str, int, int]] = []
        if jobs_i is not None:
            end_jobs = next((i for i in range(jobs_i + 1, len(L)) if L[i].strip() and not L[i].startswith((" ", "#", "\t"))), len(L))
            jind = None
            starts = []
            for i in range(jobs_i + 1, end_jobs):
                l = L[i]
                if not l.strip() or l.lstrip().startswith("#"):
                    continue
                if jind is None:
                    jind = _indent(l)
                if _indent(l) == jind and re.match(r"\s*[\w.-]+:\s*(#.*)?$", l):
                    starts.append((re.match(r"\s*([\w.-]+):", l).group(1), i))
            for k, (name, s) in enumerate(starts):
                e = starts[k + 1][1] if k + 1 < len(starts) else end_jobs
                jobs.append((name, s, e))
        self.jobs_total = len(jobs)
        for name, js, je in jobs:
            steps_i = next((i for i in range(js + 1, je) if re.match(r"^\s*steps:\s*(#.*)?$", L[i])), None)
            active = 0
            had = 0
            if steps_i is not None:
                sind = _indent(L[steps_i])
                dash_ind = None
                starts = []
                for i in range(steps_i + 1, je):
                    l = L[i]
                    if not l.strip() or l.lstrip().startswith("#"):
                        continue
                    if _indent(l) <= sind:
                        break
                    if dash_ind is None:
                        dash_ind = _indent(l)
                    if _indent(l) == dash_ind and l.lstrip().startswith("- "):
                        starts.append(i)
                for k, s in enumerate(starts):
                    e = self._block_end(s, dash_ind, starts[k + 1] if k + 1 < len(starts) else je)
                    e = min(e, starts[k + 1] if k + 1 < len(starts) else je)
                    block = L[s:e]
                    had += 1
                    text = "".join(block)
                    if not XT_CMD.search(text):
                        # Cache-/paths-Zeilen
                        for off, bl in enumerate(block):
                            if re.match(r"^\s*(?:-\s*)?['\"]?xtask(?:/\*\*)?['\"]?\s*$", bl):
                                self.delete.add(s + off)
                        active += 1
                        continue
                    cmds = [bl for bl in block if CMD_START.match(bl)]
                    xt = [bl for bl in cmds if XT_CMD.search(bl)]
                    others = [bl for bl in cmds if bl not in xt]
                    uses = re.search(r"^\s*(?:-\s*)?uses:", text, re.M)
                    if others and xt:
                        self.manual.append((s + 1, f"Job `{name}`: Step mischt xtask- und andere Befehle"))
                        active += 1
                        continue
                    if uses and not xt:
                        active += 1
                        continue
                    self.comment.update(range(s, e))
                    self.commented_steps += 1
            if had and active == 0:
                self.comment.update(range(js, je))
                self.commented_jobs.append(name)
        # paths:-Filter: Zeilen `- xtask/**`
        for i, l in enumerate(L):
            if re.match(r"^\s*-\s*['\"]?xtask/", l) and i not in self.comment:
                self.delete.add(i)
        self._fix_needs()

    def _fix_needs(self) -> None:
        gone = set(self.commented_jobs)
        if not gone:
            return
        L = self.lines
        for i, l in enumerate(L):
            if i in self.comment:
                continue
            m = re.match(r"^(\s*)needs:\s*\[(.*)\]\s*(#.*)?$", l)
            if m:
                names = [x.strip() for x in m.group(2).split(",") if x.strip()]
                keep = [x for x in names if x not in gone]
                if len(keep) != len(names):
                    L[i] = (f"{m.group(1)}needs: [{', '.join(keep)}]\n" if keep else f"{m.group(1)}# needs: [{', '.join(names)}]\n")
                continue
            m = re.match(r"^(\s*)needs:\s*([\w.-]+)\s*(#.*)?$", l)
            if m and m.group(2) in gone:
                L[i] = f"{m.group(1)}# needs: {m.group(2)}\n"
                continue
            if re.match(r"^\s*needs:\s*(#.*)?$", l):
                ind = _indent(l)
                j = i + 1
                items = []
                while j < len(L) and L[j].strip().startswith("- ") and _indent(L[j]) > ind:
                    items.append(j); j += 1
                kept = [k for k in items if L[k].strip()[2:].split("#")[0].strip() not in gone]
                for k in items:
                    if k not in kept:
                        self.delete.add(k)
                if items and not kept:
                    L[i] = f"{L[i][:ind]}# needs:\n"

    def active_jobs(self) -> int:
        return self.jobs_total - len(self.commented_jobs)

    def result(self) -> str:
        out = []
        for i, l in enumerate(self.lines):
            if i in self.delete:
                continue
            if i in self.comment:
                if l.strip():
                    ind = _indent(l)
                    out.append(f"{l[:ind]}# {l[ind:]}")
                else:
                    out.append(l)
            else:
                out.append(l)
        return "".join(out)


GATE_WORKFLOW = """name: ctxrs gates

# Ersetzt die entfernten xtask-Gates (Workspace-Diagnose, Architektur-Ratchet).
# Reine Python-Standardbibliothek; kein Cargo erforderlich.
on:
  pull_request:
  push:
    branches: ["main", "master"]

permissions:
  contents: read

jobs:
  ctxrs:
    name: Workspace und Architektur (ctxrs)
    runs-on: ubuntu-latest
    timeout-minutes: 10
    steps:
      - uses: {checkout}
        with:
          fetch-depth: 1

      - name: Workspace-Diagnose (D01-D12)
        run: python3 {tool}/ctxrs.py doctor --show KH

      - name: Architektur-Ratchet (R1-R7, W1)
        run: python3 {tool}/ctxrs.py archgate

      - name: Befund-Tracker (Information)
        run: python3 {tool}/ctxrs.py findings --open-only
        continue-on-error: true
"""


def run(ws: Workspace, args) -> int:
    root = ws.root
    cs = ChangeSet(root)
    report: list[list] = []
    manual: list[tuple[str, int, str]] = []

    if (root / ".cargo/config.toml").is_file():
        new, n = cargo_alias(read_text(root, ".cargo/config.toml"))
        if n:
            cs.write(".cargo/config.toml", new)
            report.append([".cargo/config.toml", "Alias entfernt", n])
    if (root / "justfile").is_file():
        new, hits = justfile(read_text(root, "justfile"))
        if hits:
            cs.write("justfile", new)
            report.append(["justfile", "Zeilen durch Warnung ersetzt", len(hits)])
    wfdir = root / ".github" / "workflows"
    empty: list[str] = []
    if wfdir.is_dir():
        for f in sorted(wfdir.glob("*.y*ml")):
            rel = f.relative_to(root).as_posix()
            text = f.read_text(encoding="utf-8", errors="replace")
            if not XT_WORD.search(text):
                continue
            w = Wf(text)
            w.analyse()
            for ln, msg in w.manual:
                manual.append((rel, ln, msg))
            if w.active_jobs() <= 0 and w.jobs_total > 0:
                empty.append(rel)
                if args.delete_empty:
                    cs.delete_file(rel)
                    report.append([rel, "Workflow leer -> gelöscht", w.jobs_total])
                    continue
            new = w.result()
            if new != text:
                cs.write(rel, new)
                report.append([rel, f"{w.commented_steps} Step(s), {len(w.commented_jobs)} Job(s) auskommentiert; "
                                    f"{len(w.delete)} Zeile(n) entfernt", w.active_jobs()])
    if getattr(args, "emit_workflow", False):
        text = ""
        chk = "actions/checkout@v4"
        for f in sorted(wfdir.glob("*.y*ml")) if wfdir.is_dir() else []:
            m = re.search(r"uses:\s*(actions/checkout@[0-9a-f]{40})", f.read_text(encoding="utf-8", errors="replace"))
            if m:
                chk = m.group(1) + " # v4"
                break
        cs.write(".github/workflows/ctxrs-gates.yml",
                 GATE_WORKFLOW.format(checkout=chk, tool=args.tool_path.rstrip("/")))
        report.append([".github/workflows/ctxrs-gates.yml", "Ersatz-Gate angelegt", 1])

    print("# ctxrs ci-purge – Plan\n")
    print(md_table(["Datei", "Aktion", "Anzahl / aktive Jobs"], report) if report else "Keine xtask-Verweise gefunden.\n")
    if empty and not args.delete_empty:
        print("Workflows ohne verbleibende Jobs (ohne `--delete-empty` bleiben sie als ungültige Dateien zurück):\n")
        print("\n".join(f"- `{e}`" for e in empty) + "\n")
    if manual:
        print("## Manuell prüfen (gemischte Steps, nicht angefasst)\n")
        print("\n".join(f"- `{f}:{ln}` {m}" for f, ln, m in manual) + "\n")
    print(cs.summary() + "\n")
    print("Hinweis: `.github/**` und `justfile` sind geschützte Pfade (governance/protected-paths.toml).\n")
    if not args.apply:
        if not args.no_diff and not cs.empty():
            print("```diff\n" + cs.diff(0)[:60_000] + "```\n")
        print("Dry-Run. Mit `--apply` schreiben.")
        return 0
    cs.apply()
    print("Angewendet.")
    return 0
