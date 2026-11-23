#!/usr/bin/env python3
"""Gegenprobe der Goldens (SPEC §19.4, A.3). Unabhängig vom Generator.

Liest die Golden-Dateien ohne `generator.py` und prüft:
  1. Anhang A.1 für CRC-32C mit einer bitweisen Implementierung (keine Tabelle).
  2. Pro Golden: lückenlose Offsets, Feldlängen, GESAMT-Zeile und Laenge-Zeile stimmen.
  3. Prüfwert chk = CRC-32C(seed 0) über alle Bytes vor chk, mit bitweiser Berechnung.
  4. Feste Längen aus der Spec (FORMAT 49 vor chk / 53 gesamt, Segmentkopf 32 / 36).
  5. Gleichheit mit den Werten aus dem Entwurf PR #6069 (optional, Pfad per Argument).

Aufruf: python3 tools/reference/verify_goldens.py [pfad_zu_draft_json]
Exit 0 nur, wenn alle Prüfungen bestehen.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
GOLDEN_DIR = REPO_ROOT / "crates" / "contextra-storage" / "tests" / "format" / "golden"
REFLECTED_POLY = 0x82F63B78


def crc32c_bitwise(seed: int, data: bytes) -> int:
    """CRC-32C, bitweise, Ergebnis = !update(!seed, data) (SPEC §4)."""
    crc = ~seed & 0xFFFFFFFF
    for byte in data:
        crc ^= byte
        for _ in range(8):
            crc = (crc >> 1) ^ REFLECTED_POLY if crc & 1 else crc >> 1
    return ~crc & 0xFFFFFFFF


def parse_golden(path: Path) -> dict:
    """Liest eine .golden-Datei: Felder als (offset, länge, name, bytes), GESAMT, Laenge."""
    fields, gesamt, laenge = [], None, None
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.split("#", 1)[0].strip()
        if raw.startswith("# laenge:"):
            laenge = int(raw.split(":", 1)[1])
        if not line:
            continue
        if line.startswith("GESAMT "):
            gesamt = bytes.fromhex(line.split(" ", 1)[1])
            continue
        off, ln, name, hexdata = line.split()
        fields.append((int(off), int(ln), name, bytes.fromhex(hexdata)))
    return {"fields": fields, "gesamt": gesamt, "laenge": laenge}


def check_golden(path: Path, fehler: list[str]) -> dict:
    g = parse_golden(path)
    name = path.stem
    pos = 0
    for off, ln, fname, data in g["fields"]:
        if off != pos:
            fehler.append(f"{name}: Feld {fname} bei Offset {off}, erwartet {pos}")
        if len(data) != ln:
            fehler.append(f"{name}: Feld {fname} hat {len(data)} statt {ln} Byte")
        pos += ln
    joined = b"".join(f[3] for f in g["fields"])
    if joined != g["gesamt"]:
        fehler.append(f"{name}: GESAMT weicht von der Summe der Felder ab")
    if g["laenge"] != pos:
        fehler.append(f"{name}: Laenge-Zeile {g['laenge']} statt {pos}")
    return g


def check_chk(name: str, g: dict, fehler: list[str]) -> bytes | None:
    """Prüft den letzten Feldwert 'chk' bitweise. Gibt die Bytes vor chk zurück."""
    if not g["fields"] or g["fields"][-1][2] != "chk":
        return None
    body = b"".join(f[3] for f in g["fields"][:-1])
    expected = crc32c_bitwise(0, body).to_bytes(4, "little")
    if g["fields"][-1][3] != expected:
        fehler.append(f"{name}: chk {g['fields'][-1][3].hex()} statt {expected.hex()}")
    return body


def check_a1(fehler: list[str]) -> None:
    cases = [(b"123456789", 0xE3069283), (b"a", 0xC1D04330)]
    for data, want in cases:
        got = crc32c_bitwise(0, data)
        if got != want:
            fehler.append(f"A.1 CRC-32C({data!r}) bitweise: {got:#x} statt {want:#x}")


def check_draft(path: Path, fehler: list[str]) -> None:
    """Vergleicht Hex-Werte mit einer JSON-Datei des Entwurfs (Abweichungen melden)."""
    draft = json.loads(path.read_text(encoding="utf-8"))
    for key in ("format_plain", "segment_header_plain"):
        ref = draft[key]["hex"]
        mine = (GOLDEN_DIR / f"{key}.golden")
        ours = parse_golden(mine)["gesamt"].hex()
        if ours != ref:
            fehler.append(f"Entwurf {key}: Hex weicht ab")
        else:
            print(f"Entwurf {key}: identisch ({len(ref) // 2} Byte)")


def main(argv: list[str]) -> int:
    fehler: list[str] = []
    check_a1(fehler)
    print("A.1 CRC-32C bitweise: geprüft")

    expected = {"format_plain": (49, 53), "format_plain_features": (49, 53),
                "segment_header_plain": (32, 36)}
    for name, (vor, gesamt) in expected.items():
        path = GOLDEN_DIR / f"{name}.golden"
        g = check_golden(path, fehler)
        body = check_chk(name, g, fehler)
        if body is None or len(body) != vor or len(g["gesamt"]) != gesamt:
            fehler.append(f"{name}: Länge vor chk {len(body or b'')} / gesamt {len(g['gesamt'])}, "
                          f"erwartet {vor} / {gesamt}")
        else:
            print(f"{name}: {gesamt} Byte, chk bitweise bestätigt")

    frame = check_golden(GOLDEN_DIR / "frame_body_plain.golden", fehler)
    if frame["fields"][0][2] != "frame_len" or int.from_bytes(frame["fields"][0][3], "little") != 117:
        fehler.append("frame_body_plain: frame_len ist nicht 117")
    if frame["fields"][-1][2] == "chk":
        fehler.append("frame_body_plain: unerwartet mit chk (Lücke L-2 wäre geschlossen, Golden prüfen)")
    else:
        print(f"frame_body_plain: {len(frame['gesamt'])} Byte vor frame_chk, frame_len 117 geprüft")

    if len(argv) > 0:
        check_draft(Path(argv[0]), fehler)

    if fehler:
        print("FEHLER:", *fehler, sep="\n  ", file=sys.stderr)
        return 1
    print("OK: Gegenprobe bestanden")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
