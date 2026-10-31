"""Kommentar- und formaterhaltender TOML-Texteditor.

Zweck: Manifeste gezielt ändern (Array-Elemente, Schlüssel, Tabellen löschen), ohne die
Datei durch einen Serializer neu zu formatieren. Gelesen wird semantisch mit `tomllib`,
geändert wird auf Textebene anhand eines Statement-Lexers. Jede Änderung wird vom Aufrufer
mit `validate` (tomllib) nachgeprüft.
"""
from __future__ import annotations

import bisect
import re
import tomllib
from dataclasses import dataclass

_BARE = re.compile(r"[A-Za-z0-9_-]+")


@dataclass
class Stmt:
    kind: str                     # "header" | "kv"
    start: int                    # erste Zeile (0-basiert, inklusiv)
    end: int                      # letzte Zeile (inklusiv)
    table: tuple[str, ...]        # aktuelle Tabelle (bei header: die neue)
    key: tuple[str, ...] = ()
    vstart: int = 0               # Offset Wertbeginn (nur kv)
    vend: int = 0                 # Offset Wertende, exklusiv (nur kv)
    aot: bool = False             # [[array-of-tables]]


def _skip_ws(t: str, p: int) -> int:
    while p < len(t) and t[p] in " \t":
        p += 1
    return p


def _parse_key(t: str, p: int) -> tuple[tuple[str, ...], int]:
    """Dotted key ab p. Liefert (Teile, Offset des ersten Zeichens nach dem Schlüssel)."""
    parts: list[str] = []
    n = len(t)
    while p < n:
        p = _skip_ws(t, p)
        if p >= n:
            break
        c = t[p]
        if c == '"':
            q = p + 1
            buf = []
            while q < n and t[q] != '"':
                if t[q] == "\\" and q + 1 < n:
                    buf.append(t[q + 1]); q += 2
                else:
                    buf.append(t[q]); q += 1
            parts.append("".join(buf)); p = q + 1
        elif c == "'":
            q = t.index("'", p + 1)
            parts.append(t[p + 1:q]); p = q + 1
        else:
            m = _BARE.match(t, p)
            if not m:
                raise ValueError(f"Schlüssel nicht lesbar bei Offset {p}: {t[p:p+30]!r}")
            parts.append(m.group(0)); p = m.end()
        p = _skip_ws(t, p)
        if p < n and t[p] == ".":
            p += 1
            continue
        break
    return tuple(parts), p


def _scan_value(t: str, p: int) -> int:
    """Offset (exklusiv) des Wertendes: bis Zeilenende bei Klammertiefe 0, inkl. Mehrzeiler."""
    n = len(t)
    depth = 0
    while p < n:
        c = t[p]
        if t.startswith('"""', p):
            q = p + 3
            while True:
                q = t.find('"""', q)
                if q < 0:
                    return n
                bs, k = 0, q - 1
                while k >= 0 and t[k] == "\\":
                    bs += 1; k -= 1
                if bs % 2 == 0:
                    break
                q += 1
            p = q + 3
            while p < n and t[p] == '"':
                p += 1
            continue
        if t.startswith("'''", p):
            q = t.find("'''", p + 3)
            p = n if q < 0 else q + 3
            continue
        if c == '"':
            p += 1
            while p < n and t[p] != '"':
                p += 2 if t[p] == "\\" else 1
            p += 1
            continue
        if c == "'":
            q = t.find("'", p + 1)
            p = n if q < 0 else q + 1
            continue
        if c == "#":
            q = t.find("\n", p)
            p = n if q < 0 else q
            continue
        if c in "[{":
            depth += 1
        elif c in "]}":
            depth -= 1
        elif c == "\n" and depth <= 0:
            return p
        p += 1
    return n


def parse(text: str) -> list[Stmt]:
    starts = [0] + [m.end() for m in re.finditer(r"\n", text)]

    def line_of(off: int) -> int:
        return bisect.bisect_right(starts, off) - 1

    stmts: list[Stmt] = []
    table: tuple[str, ...] = ()
    i = 0
    while i < len(starts):
        off = starts[i]
        if off >= len(text):
            break
        p = _skip_ws(text, off)
        if p >= len(text) or text[p] in "\r\n#":
            i += 1
            continue
        if text[p] == "[":
            aot = text.startswith("[[", p)
            key, _ = _parse_key(text, p + (2 if aot else 1))
            table = key
            stmts.append(Stmt("header", i, i, table, aot=aot))
            i += 1
            continue
        key, q = _parse_key(text, p)
        if q >= len(text) or text[q] != "=":
            raise ValueError(f"Zeile {i + 1}: '=' erwartet")
        vs = _skip_ws(text, q + 1)
        ve = _scan_value(text, vs)
        end_line = line_of(max(ve - 1, vs))
        stmts.append(Stmt("kv", i, end_line, table, key, vs, ve))
        i = end_line + 1
    return stmts


def array_items(text: str, vstart: int, vend: int) -> list[tuple[str, int, int]]:
    """Top-Level-String-Elemente eines Arrays: (Wert, Start, Ende exklusiv), absolute Offsets."""
    out = []
    p = vstart
    while p < vend and text[p] != "[":
        p += 1
    p += 1
    depth = 1
    while p < vend and depth > 0:
        c = text[p]
        if c == "#":
            q = text.find("\n", p)
            p = vend if q < 0 else q
        elif c in "[{":
            depth += 1; p += 1
        elif c in "]}":
            depth -= 1; p += 1
        elif c in "\"'" and depth == 1:
            q = p + 1
            buf = []
            while q < vend and text[q] != c:
                if c == '"' and text[q] == "\\":
                    buf.append(text[q + 1]); q += 2
                else:
                    buf.append(text[q]); q += 1
            out.append(("".join(buf), p, q + 1))
            p = q + 1
        elif c in "\"'":
            q = text.find(c, p + 1)
            p = vend if q < 0 else q + 1
        else:
            p += 1
    return out


class Editor:
    """Sammelt Textoperationen und wendet sie atomar an (`result()`)."""

    def __init__(self, text: str):
        self.text = text
        self.stmts = parse(text)
        self.lines = text.splitlines(keepends=True)
        self.starts = [0] + [m.end() for m in re.finditer(r"\n", text)]
        self._del_lines: set[int] = set()
        self._cuts: list[tuple[int, int]] = []
        self._repl: list[tuple[int, int, str]] = []

    # -- Abfragen -----------------------------------------------------------------
    def kvs(self, table: tuple[str, ...]) -> list[Stmt]:
        return [s for s in self.stmts if s.kind == "kv" and s.table == table]

    def header(self, table: tuple[str, ...]) -> Stmt | None:
        return next((s for s in self.stmts if s.kind == "header" and s.table == table), None)

    def table_names(self) -> list[tuple[str, ...]]:
        seen: list[tuple[str, ...]] = []
        for s in self.stmts:
            if s.table not in seen:
                seen.append(s.table)
        return seen

    # -- Operationen --------------------------------------------------------------
    def delete_stmt(self, s: Stmt, with_comments: bool = True) -> None:
        """Löscht das Statement; unmittelbar vorangehende Kommentarzeilen, die den Schlüssel nennen, gehen mit."""
        self._del_lines.update(range(s.start, s.end + 1))
        if with_comments and s.key:
            i = s.start - 1
            while i >= 0 and self.lines[i].lstrip().startswith("#") and s.key[0] in self.lines[i]:
                self._del_lines.add(i)
                i -= 1

    def delete_table(self, table: tuple[str, ...]) -> None:
        """Löscht Header, alle zugehörigen kv-Zeilen und eine folgende Leerzeile."""
        h = self.header(table)
        if h is None:
            return
        last = h.end
        for s in self.stmts:
            if s.start > h.start and s.table == table and s.kind == "kv":
                last = max(last, s.end)
        self._del_lines.update(range(h.start, last + 1))
        nxt = last + 1
        if nxt < len(self.lines) and self.lines[nxt].strip() == "":
            self._del_lines.add(nxt)

    def remove_array_items(self, s: Stmt, pred) -> list[str]:
        """Entfernt Elemente mit pred(wert)==True. Liefert die entfernten Werte (in Reihenfolge)."""
        removed: list[str] = []
        first_line = self.text.rfind("\n", 0, s.vstart) + 1
        for val, a, b in reversed(array_items(self.text, s.vstart, s.vend)):
            if not pred(val):
                continue
            removed.append(val)
            ls = self.text.rfind("\n", 0, a) + 1
            le = self.text.find("\n", b)
            le = len(self.text) if le < 0 else le + 1
            before = self.text[ls:a].replace(",", "").strip()
            after = re.sub(r"#.*", "", self.text[b:le]).replace(",", "").strip()
            lone = before == "" and after == "" and ls != first_line
            if lone:
                self._cuts.append((ls, le))
                continue
            m = re.match(r"[ \t]*,[ \t]*", self.text[b:])
            if m:
                self._cuts.append((a, b + m.end()))
            else:
                pm = re.search(r",[ \t]*$", self.text[:a])
                self._cuts.append((pm.start() if pm else a, b))
        removed.reverse()
        return removed

    def replace_value(self, s: Stmt, new_value: str) -> None:
        self._repl.append((s.vstart, s.vend, new_value))

    # -- Anwenden -----------------------------------------------------------------
    def result(self) -> str:
        ops: list[tuple[int, int, str]] = []
        for ln in sorted(self._del_lines):
            ops.append((self.starts[ln], self.starts[ln] + len(self.lines[ln]), ""))
        ops += [(a, b, "") for a, b in self._cuts]
        ops += self._repl
        ops.sort(key=lambda o: (o[0], o[1]))
        merged: list[tuple[int, int, str]] = []
        for o in ops:
            if merged and o[0] < merged[-1][1]:
                prev = merged[-1]
                if o[2] == "" and prev[2] == "":
                    merged[-1] = (prev[0], max(prev[1], o[1]), "")
                    continue
                raise ValueError("Überlappende Ersetzungen")
            merged.append(o)
        out = self.text
        for a, b, r in reversed(merged):
            out = out[:a] + r + out[b:]
        return out


def validate(text: str, what: str = "") -> dict:
    try:
        return tomllib.loads(text)
    except tomllib.TOMLDecodeError as e:
        raise ValueError(f"Ergebnis ist kein gültiges TOML ({what}): {e}") from e
