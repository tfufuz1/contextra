#!/usr/bin/env python3
"""CI-Gates G-1 … G-6 der Löschkette (docs/erasure/13-contextra-testkit.md §9).

Altlasten stehen als Ratsche in `erasure_gates_baseline.txt` (`Gate Datei Anzahl`):
mehr Verstöße als dort sind ein Fehler, weniger ebenfalls (Eintrag senken). Die
Baseline darf nur schrumpfen.

Nur Standardbibliothek. Ausgabe `datei:zeile: Gx: Text`, Exit-Code 1 bei Verstoß,
2 bei Konfigurationsfehlern.

Die Gates sind lexikalisch: Kommentare, String- und Zeichenliterale werden vor dem
Matching maskiert; Elemente unter `#[cfg(test)]` und Verzeichnisse `tests/`,
`benches/` sind ausgenommen. Makro-erzeugter Code wird nicht gesehen.
"""

from __future__ import annotations

import argparse
import fnmatch
import re
import sys
from pathlib import Path

NEW_LAYER_FILES = [
    "crates/contextra-text/src/erasure.rs",
    "crates/contextra-vector/src/hnsw/erasure.rs",
    "crates/contextra-graph/src/erasure.rs",
    "crates/contextra-checkpoint/src/erasure.rs",
]
ERASURE_DIRS = [
    "crates/contextra-store/src/erasure/*",
    "crates/contextra-engine/src/erasure/*",
]
G1_PATHS = ERASURE_DIRS + NEW_LAYER_FILES + [
    "crates/contextra-engine/src/background_workers/erasure_worker.rs",
    "crates/contextra-crypto/src/vault.rs",
    "crates/contextra-crypto/src/erasure_root.rs",
]
SCOPES = {
    "G-1": G1_PATHS,
    "G-2": G1_PATHS + ["crates/contextra-crypto/src/dek.rs"],
    "G-3": ["crates/*/src/*"],
    "G-4": ERASURE_DIRS + NEW_LAYER_FILES,
    # „Neue Dateien (Liste wie G-1)“: alle Pfade der G-1-Liste sind Neuanlagen der Löschkette.
    "G-5": G1_PATHS,
    "G-6": [
        "crates/contextra-ports/src/gate.rs",
        "crates/contextra-store/src/erasure/gate.rs",
        "crates/contextra-testkit/src/gates.rs",
    ],
}
TRAIT_FILE = "crates/contextra-ports/src/gate.rs"
BASELINE = "tools/erasure_gates_baseline.txt"
SENSITIVE_PARAMS = {"id", "key", "target", "doc_id", "term"}

TOKEN = re.compile(
    r"""(?P<line>//[^\n]*)
      |(?P<block>/\*)
      |(?P<raw>\b[bc]?r(?P<hashes>\#*)"(?:.|\n)*?"(?P=hashes))
      |(?P<str>"(?:\\(?:.|\n)|[^"\\])*")
      |(?P<char>'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^\\'\n])')""",
    re.VERBOSE,
)


def _blank(text: str) -> str:
    return re.sub(r"[^\n]", " ", text)


def mask(source: str) -> tuple[str, dict[int, str]]:
    """Maskiert Kommentare und Literale; liefert zusätzlich Zeile → Zeilenkommentar."""
    out: list[str] = []
    comments: dict[int, str] = {}
    pos = 0
    while True:
        m = TOKEN.search(source, pos)
        if not m:
            out.append(source[pos:])
            return "".join(out), comments
        out.append(source[pos : m.start()])
        end = m.end()
        if m.lastgroup == "line":
            comments[source.count("\n", 0, m.start()) + 1] = m.group()
        elif m.group("block"):
            depth = 1
            for d in re.compile(r"/\*|\*/").finditer(source, end):
                depth += 1 if d.group() == "/*" else -1
                if depth == 0:
                    end = d.end()
                    break
            else:
                end = len(source)
        out.append(_blank(source[m.start() : end]))
        pos = end


def match_close(code: str, start: int) -> int:
    """Index hinter der zu `code[start]` passenden schließenden Klammer."""
    pairs = {"{": "}", "(": ")", "[": "]"}
    open_, close = code[start], pairs[code[start]]
    depth = 0
    for i in range(start, len(code)):
        if code[i] == open_:
            depth += 1
        elif code[i] == close:
            depth -= 1
            if depth == 0:
                return i + 1
    return len(code)


def item_end(code: str, start: int) -> int:
    """Ende des Elements ab `start`: `;`/`,` auf Tiefe 0 oder der erste `{}`-Block."""
    i = start
    while i < len(code):
        c = code[i]
        if c in "([":
            i = match_close(code, i)
            continue
        if c == "{":
            return match_close(code, i)
        if c in ";,}":
            return i + 1
        i += 1
    return len(code)


def strip_test_items(code: str) -> str:
    if re.search(r"#!\[\s*cfg\s*\(\s*test\s*\)\s*\]", code):
        return _blank(code)
    attr = re.compile(r"#\[\s*(?:cfg\s*\(\s*(?:all\s*\(\s*)?test\b|(?:tokio::)?test\b)")
    pos = 0
    while m := attr.search(code, pos):
        i = match_close(code, m.start() + 1)
        # Weitere Attribute desselben Elements überspringen.
        while n := re.compile(r"\s*#\[").match(code, i):
            i = match_close(code, n.end() - 1)
        end = item_end(code, i)
        code = code[: m.start()] + _blank(code[m.start() : end]) + code[end:]
        pos = end
    return code


def line_of(code: str, pos: int) -> int:
    return code.count("\n", 0, pos) + 1


def find(code: str, pattern: str, message: str):
    for m in re.finditer(pattern, code):
        yield line_of(code, m.start()), message.format(m.group().strip())


def g1(code, comments, ctx):
    yield from find(code, r"\bblock_on\b", "`{}` blockiert die Laufzeit")


def g2(code, comments, ctx):
    patterns = [
        (r"\blet\s+_\s*(?::[^=;]+)?=(?!=)", "`let _ =` verwirft Ergebnisse"),
        (r"\bunwrap_or_default\s*\(", "`unwrap_or_default()` verschluckt Fehler"),
        (r"\.\s*ok\s*\(\s*\)\s*;", "`.ok();` verwirft Fehler"),
    ]
    for pattern, message in patterns:
        for m in re.finditer(pattern, code):
            line = line_of(code, m.start())
            last = line_of(code, m.end())
            allow = [
                a
                for n in range(line, last + 1)
                if (a := re.search(r"//\s*gate-allow:\s*G-2\b(.*)", comments.get(n, "")))
            ]
            if not allow:
                yield line, message
            elif not re.search(r"\w{3,}", allow[0].group(1)):
                yield line, "gate-allow: G-2 ohne Begründung"


def split_top_level(params: str) -> list[str]:
    parts, depth, last = [], 0, 0
    for i, c in enumerate(params):
        if c in "([{<":
            depth += 1
        elif c in ")]}" or (c == ">" and params[i - 1 : i] != "-"):
            depth -= 1
        elif c == "," and depth == 0:
            parts.append(params[last:i])
            last = i + 1
    parts.append(params[last:])
    return parts


def g3(code, comments, ctx):
    attr = re.compile(r"#\[\s*(?:(?:::)?tracing\s*::\s*)?(?:instrument\b|cfg_attr\b)")
    for m in attr.finditer(code):
        end = match_close(code, m.start() + 1)
        text = code[m.start() : end]
        inner = re.search(r"\binstrument\b", text)
        if not inner:
            continue
        after = text[inner.end() :]
        skipped: set[str] = set()
        if re.search(r"\bskip_all\b", after):
            continue
        if s := re.search(r"\bskip\s*\(([^)]*)\)", after):
            skipped = {n.strip() for n in s.group(1).split(",")}
        fn = re.compile(r"\bfn\s+\w+").search(code, end)
        if not fn:
            continue
        i = fn.end()
        # Generics überspringen (`->` in Bounds zählt nicht als Klammer).
        depth = 0
        while i < len(code) and (code[i] != "(" or depth):
            if code[i] == "<":
                depth += 1
            elif code[i] == ">" and code[i - 1] != "-":
                depth -= 1
            i += 1
        params = code[i + 1 : match_close(code, i) - 1] if i < len(code) else ""
        for part in split_top_level(params):
            pattern = re.split(r"(?<!:):(?!:)", part, maxsplit=1)[0]
            for name in re.findall(r"\b[a-z_][a-z0-9_]*\b", pattern):
                if name.lstrip("_") in SENSITIVE_PARAMS and name not in skipped:
                    yield (
                        line_of(code, m.start()),
                        f"`instrument` protokolliert sensiblen Parameter `{name}`",
                    )


def g4(code, comments, ctx):
    yield from find(code, r"\b(?:std|tokio)\s*::\s*fs\b", "direktes Dateisystem `{}`")
    for m in re.finditer(r"\buse\s+(?:std|tokio)\s*::\s*\{", code):
        end = match_close(code, m.end() - 1)
        if re.search(r"[{,]\s*fs\b", code[m.end() - 1 : end]):
            yield line_of(code, m.start()), "direktes Dateisystem `fs` im use-Baum"


def g5(code, comments, ctx):
    yield from find(
        code,
        r"\b(?:Instant|SystemTime)\s*::\s*now\b|\bHash(?:Map|Set)\b",
        "nichtdeterministisch: `{}`",
    )


def trait_body(code: str) -> tuple[int, str] | None:
    m = re.search(r"\btrait\s+ReadGate\b[^{;]*\{", code)
    if not m:
        return None
    return m.end(), code[m.end() : match_close(code, m.end() - 1) - 1]


def top_level_fns(body: str):
    """Yield (Name, Position, hat Körper) für Funktionen auf Tiefe 0."""
    i = 0
    fn = re.compile(r"\bfn\s+(\w+)")
    while m := fn.search(body, i):
        j = m.end()
        while j < len(body) and body[j] not in "{;":
            j = match_close(body, j) if body[j] in "([" else j + 1
        has_body = j < len(body) and body[j] == "{"
        yield m.group(1), m.start(), has_body
        i = match_close(body, j) if has_body else j + 1


def g6(code, comments, ctx):
    required = ctx["methods"]
    if found := trait_body(code):
        offset, body = found
        for name, pos, has_body in top_level_fns(body):
            if has_body:
                yield (
                    line_of(code, offset + pos),
                    f"`ReadGate::{name}` hat einen Default-Körper",
                )
    impl = re.compile(r"\bimpl\b[^{;]*?\bReadGate\s+for\b[^{;]*\{")
    for m in impl.finditer(code):
        body = code[m.end() : match_close(code, m.end() - 1) - 1]
        have = {name for name, _, _ in top_level_fns(body)}
        if missing := sorted(required - have):
            yield (
                line_of(code, m.start()),
                f"`impl ReadGate` ohne explizite Methode(n): {', '.join(missing)}",
            )


GATES = {"G-1": g1, "G-2": g2, "G-3": g3, "G-4": g4, "G-5": g5, "G-6": g6}


def gate_methods(source: str) -> set[str]:
    code, _ = mask(source)
    found = trait_body(code)
    return {name for name, _, _ in top_level_fns(found[1])} if found else set()


def check_source(source: str, gates, ctx):
    code, comments = mask(source)
    code = strip_test_items(code)
    for gate in gates:
        for line, message in GATES[gate](code, comments, ctx):
            yield line, gate, message


def load_baseline(path: Path) -> dict[tuple[str, str], int]:
    entries: dict[tuple[str, str], int] = {}
    if path.is_file():
        for raw in path.read_text(encoding="utf-8").splitlines():
            if raw.strip() and not raw.startswith("#"):
                gate, rel, count = raw.split()
                entries[(rel, gate)] = int(count)
    return entries


def apply_baseline(found, baseline):
    """Liefert die auszugebenden Verstöße unter Berücksichtigung der Ratsche."""
    report = []
    for key in sorted(set(found) | set(baseline)):
        rel, gate = key
        hits = found.get(key, [])
        allowed = baseline.get(key, 0)
        if len(hits) > allowed:
            note = f" (Baseline erlaubt {allowed})" if allowed else ""
            report += [f"{rel}:{line}: {gate}: {msg}{note}" for line, msg in hits]
        elif len(hits) < allowed:
            report.append(
                f"{rel}:1: {gate}: Baseline veraltet ({allowed} erlaubt, {len(hits)} gefunden)"
            )
    return report


def run(root: Path) -> int:
    trait_file = root / TRAIT_FILE
    if not trait_file.is_file():
        print(f"{TRAIT_FILE}:1: G-6: Trait-Definition fehlt", file=sys.stderr)
        return 2
    ctx = {"methods": gate_methods(trait_file.read_text(encoding="utf-8"))}
    if not ctx["methods"]:
        print(f"{TRAIT_FILE}:1: G-6: keine ReadGate-Methoden erkannt", file=sys.stderr)
        return 2
    found: dict[tuple[str, str], list[tuple[int, str]]] = {}
    files = 0
    for path in sorted((root / "crates").glob("*/src/**/*.rs")):
        rel = path.relative_to(root).as_posix()
        if path.name == "tests.rs" or "/tests/" in rel or "/benches/" in rel:
            continue
        gates = [g for g, globs in SCOPES.items() if any(fnmatch.fnmatch(rel, p) for p in globs)]
        if not gates:
            continue
        files += 1
        for line, gate, message in check_source(path.read_text(encoding="utf-8"), gates, ctx):
            found.setdefault((rel, gate), []).append((line, message))
    baseline = load_baseline(root / BASELINE)
    report = apply_baseline(found, baseline)
    for entry in report:
        print(entry)
    print(
        f"erasure gates: {files} Dateien, {len(report)} Verstöße, "
        f"{sum(baseline.values())} Altlasten in der Baseline",
        file=sys.stderr,
    )
    if files == 0:
        return 2
    return 1 if report else 0


def self_test(root: Path, fixtures: Path) -> int:
    ctx = {"methods": gate_methods((root / TRAIT_FILE).read_text(encoding="utf-8"))}
    failed = not ctx["methods"]
    for gate in GATES:
        n = gate.replace("-", "").lower()
        for color in ("red", "green"):
            path = fixtures / f"{n}_{color}.rs"
            if not path.is_file():
                print(f"self-test {gate}: Fixture {path} fehlt")
                failed = True
                continue
            source = path.read_text(encoding="utf-8")
            hits = {line for line, _, _ in check_source(source, [gate], ctx)}
            # Im Rot-Beispiel markiert `//~` jede Zeile, die gemeldet werden muss.
            expected = {i for i, l in enumerate(source.splitlines(), 1) if "//~" in l}
            ok = hits == expected and (bool(expected) == (color == "red"))
            if not ok:
                print(
                    f"self-test {gate} {color}: erwartet {sorted(expected)}, "
                    f"gemeldet {sorted(hits)}"
                )
                failed = True
    found = {("a.rs", "G-2"): [(1, "x"), (2, "x")]}
    ratchet = [
        (found, {("a.rs", "G-2"): 2}, 0),
        (found, {("a.rs", "G-2"): 1}, 2),
        (found, {("a.rs", "G-2"): 3}, 1),
        ({}, {("a.rs", "G-2"): 1}, 1),
    ]
    for hits, baseline, expected_lines in ratchet:
        if len(apply_baseline(hits, baseline)) != expected_lines:
            print(f"self-test Ratsche: Baseline {baseline} falsch ausgewertet")
            failed = True
    print("self-test: " + ("FEHLER" if failed else f"{len(GATES)} Gates rot/grün ok"))
    return 1 if failed else 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return self_test(args.root, Path(__file__).resolve().parent / "tests" / "erasure_gates")
    return run(args.root)


if __name__ == "__main__":
    sys.exit(main())
