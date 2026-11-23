#!/usr/bin/env python3
"""Unabhängiger Referenzcodierer für contextra-storage (SPEC §19.4, §22 G-2, WP-2.1).

Erzeugt Byte-Goldens aus den Formatbeschreibungen in SPEC §10. Nur Standardbibliothek,
kein Import und kein Aufruf von Kerncode. Die Ergebnisse gelten erst zusammen mit dem
Selbsttest gegen Anhang A.1 und der Gegenprobe in `verify_goldens.py`.

Umfang (Modus Plain):
  - FORMAT-Kopf (§10.2), zwei Vektoren
  - Segmentkopf (§10.4)
  - Frame ohne frame_chk (§10.4): Layout vollständig, Prüfwert nicht spezifiziert (Lücke L-2)
Nicht Teil dieser Stufe: Run-Kopf (§10.5, nicht spezifiziert, Lücke L-3), Gruppenkopf
(§10.4, group_chk nicht spezifiziert), Seal-Rahmen (P9, braucht XChaCha20), Authenticated
und Encrypted (Mac16 braucht BLAKE3, AEAD braucht XChaCha20).

Aufruf:
  python3 tools/reference/generator.py             schreibt die Golden-Dateien
  python3 tools/reference/generator.py --check     vergleicht Erzeugung mit den Dateien
  python3 tools/reference/generator.py --selftest  prüft Anhang A.1 und Längen
"""

from __future__ import annotations

import sys
from dataclasses import dataclass, field
from pathlib import Path

MASK32 = 0xFFFFFFFF
MASK64 = (1 << 64) - 1
REPO_ROOT = Path(__file__).resolve().parents[2]
GOLDEN_DIR = REPO_ROOT / "crates" / "contextra-storage" / "tests" / "format" / "golden"

# ---------------------------------------------------------------------------
# Ganzzahlen little-endian (SPEC §10.1), ohne struct, mit Bereichsprüfung
# ---------------------------------------------------------------------------


def le(value: int, width: int) -> bytes:
    if not 0 <= value < (1 << (8 * width)):
        raise ValueError(f"{value} passt nicht in {width} Byte")
    return value.to_bytes(width, "little")


# ---------------------------------------------------------------------------
# CRC-32C (Castagnoli, reflektiert 0x82F63B78), tabellenbasiert.
# SPEC §4: crc32c(seed, data) = !update(!seed, data); seed 0 = üblicher CRC.
# Die bitweise Gegenimplementierung steht in verify_goldens.py.
# ---------------------------------------------------------------------------

_TABLE = []
for _i in range(256):
    _c = _i
    for _ in range(8):
        _c = (_c >> 1) ^ 0x82F63B78 if _c & 1 else _c >> 1
    _TABLE.append(_c)


def crc32c(seed: int, data: bytes) -> int:
    crc = (~seed) & MASK32
    for b in data:
        crc = _TABLE[(crc ^ b) & 0xFF] ^ (crc >> 8)
    return (~crc) & MASK32


# ---------------------------------------------------------------------------
# XXH64 und Bloom-Hashes: nur für den Selbsttest gegen Anhang A.1 (SPEC §10.5.1)
# ---------------------------------------------------------------------------

_P1, _P2, _P3 = 0x9E3779B185EBCA87, 0xC2B2AE3D27D4EB4F, 0x165667B19E3779F9
_P4, _P5 = 0x85EBCA77C2B2AE63, 0x27D4EB2F165667C5


def _rotl(x: int, r: int) -> int:
    return ((x << r) | (x >> (64 - r))) & MASK64


def _round(acc: int, inp: int) -> int:
    return (_rotl((acc + inp * _P2) & MASK64, 31) * _P1) & MASK64


def _merge(acc: int, val: int) -> int:
    return ((acc ^ _round(0, val)) * _P1 + _P4) & MASK64


def xxh64(data: bytes, seed: int = 0) -> int:
    n, i = len(data), 0
    if n >= 32:
        v = [(seed + _P1 + _P2) & MASK64, (seed + _P2) & MASK64, seed & MASK64, (seed - _P1) & MASK64]
        while i + 32 <= n:
            for j in range(4):
                v[j] = _round(v[j], int.from_bytes(data[i + 8 * j : i + 8 * j + 8], "little"))
            i += 32
        h = (_rotl(v[0], 1) + _rotl(v[1], 7) + _rotl(v[2], 12) + _rotl(v[3], 18)) & MASK64
        for vj in v:
            h = _merge(h, vj)
    else:
        h = (seed + _P5) & MASK64
    h = (h + n) & MASK64
    while i + 8 <= n:
        h ^= _round(0, int.from_bytes(data[i : i + 8], "little"))
        h = (_rotl(h, 27) * _P1 + _P4) & MASK64
        i += 8
    if i + 4 <= n:
        h ^= (int.from_bytes(data[i : i + 4], "little") * _P1) & MASK64
        h = (_rotl(h, 23) * _P2 + _P3) & MASK64
        i += 4
    while i < n:
        h ^= (data[i] * _P5) & MASK64
        h = (_rotl(h, 11) * _P1) & MASK64
        i += 1
    h ^= h >> 33
    h = (h * _P2) & MASK64
    h ^= h >> 29
    h = (h * _P3) & MASK64
    h ^= h >> 32
    return h


BLOOM_SEED2 = 0x9E3779B97F4A7C15


def bloom_positions(key: bytes, nbits: int, k: int) -> list[int]:
    h1 = xxh64(key, 0)
    h2 = xxh64(key, BLOOM_SEED2) | 1
    return [((h1 + i * h2) & MASK64) % nbits for i in range(k)]


# ---------------------------------------------------------------------------
# Golden-Modell: Felder mit Offset, Länge, Name, Hex und Bedeutung
# ---------------------------------------------------------------------------


@dataclass
class Field:
    name: str
    data: bytes
    meaning: str


@dataclass
class Golden:
    name: str
    spec: str
    assumptions: list[str]
    complete: bool = True
    note: str = ""
    fields: list[Field] = field(default_factory=list)

    def add(self, name: str, data: bytes, meaning: str) -> None:
        self.fields.append(Field(name, data, meaning))

    def body(self) -> bytes:
        return b"".join(f.data for f in self.fields)

    def add_crc(self, meaning: str) -> None:
        """Hängt chk = CRC-32C(seed 0) über alle bisherigen Bytes an (Annahme A-1)."""
        self.add("chk", le(crc32c(0, self.body()), 4), meaning)

    def text(self) -> str:
        lines = [
            f"# golden: {self.name}",
            "# quelle: tools/reference/generator.py (unabhängig vom Kern, nur Standardbibliothek)",
            f"# spec: {self.spec}",
            f"# annahmen: {', '.join(self.assumptions) or '-'}",
            f"# vollstaendig: {'ja' if self.complete else 'nein'}",
        ]
        if self.note:
            lines.append(f"# hinweis: {self.note}")
        lines.append("# zeilen: OFFSET LAENGE FELD HEX  # BEDEUTUNG (Offsets dezimal, Hex little-endian)")
        offset = 0
        for f in self.fields:
            lines.append(f"{offset} {len(f.data)} {f.name} {f.data.hex()}  # {f.meaning}")
            offset += len(f.data)
        lines.append(f"# laenge: {offset}")
        lines.append(f"GESAMT {self.body().hex()}")
        return "\n".join(lines) + "\n"


# ---------------------------------------------------------------------------
# Formatbuilder
# ---------------------------------------------------------------------------


def format_plain(name: str, feature_req: int, feature_opt: int, instance_id: bytes,
                 algo_set: int, created_ms: int) -> Golden:
    """FORMAT-Kopf, SPEC §10.2, Modus Plain. Vor chk genau 49 Byte, mit chk 53 Byte."""
    if len(instance_id) != 16:
        raise ValueError("instance_id braucht 16 Byte")
    g = Golden(name, "SPEC §10.2 (FORMAT)", ["A-1"])
    g.add("magic", b"CTXS", "Magic, ASCII")
    g.add("format_ver", le(1, 2), "Version 1")
    g.add("feature_req", le(feature_req, 8), "Pflicht-Bits")
    g.add("feature_opt", le(feature_opt, 8), "optionale Bits")
    g.add("integrity_mode", le(0, 1), "0 = Plain (Offset 22)")
    g.add("instance_id", instance_id, "Instanzkennung, 16 Byte")
    g.add("algo_set", le(algo_set, 2), "Algorithmensatz")
    g.add("created_ms", le(created_ms, 8), "Anlagezeitpunkt, Millisekunden")
    if len(g.body()) != 49:
        raise AssertionError(f"FORMAT vor chk: {len(g.body())} statt 49 Byte")
    g.add_crc("CRC-32C seed 0 über die 49 Byte davor (A-1)")
    return g


def segment_header_plain(name: str, flags: int, segment_id: int, first_stamp: int,
                         root_gen: int, chain_seed: int) -> Golden:
    """Segmentkopf, SPEC §10.4, Modus Plain (chain_seed 4 Byte). Gesamt 36 Byte."""
    g = Golden(name, "SPEC §10.4 (Segmentkopf, Plain)", ["A-1", "A-3"])
    g.add("magic", b"CTXL", "Magic, ASCII")
    g.add("ver", le(1, 2), "Version 1")
    g.add("flags", le(flags, 2), "Flags, Bedeutung nicht spezifiziert (0 gesetzt)")
    g.add("segment_id", le(segment_id, 8), "Segmentkennung")
    g.add("first_stamp", le(first_stamp, 8), "erster Stamp")
    g.add("root_gen", le(root_gen, 4), "Wurzelgeneration")
    g.add("chain_seed", le(chain_seed, 4), "Kettenstart, 4 Byte bei Plain (A-3: u32 little-endian)")
    if len(g.body()) != 32:
        raise AssertionError(f"Segmentkopf vor chk: {len(g.body())} statt 32 Byte")
    g.add_crc("CRC-32C seed 0 über die 32 Byte davor (A-1)")
    return g


def frame_body_plain(name: str, stamp: int, kind: int, idem_token: bytes, key_epoch: int,
                     frame_flags: int, ops: list[tuple[int, bytes, int, bytes, bytes]]) -> Golden:
    """Frame ohne frame_chk, SPEC §10.4. Layout vollständig; frame_chk nicht spezifiziert (L-2).

    ops: je Op (space, cluster[16], op_kind, op_uid[16], payload). Der Inhalt von
    ops_payload ist in dieser Stufe reine Byte-Eingabe; seine Semantik steht nicht in §10.
    """
    if len(idem_token) != 16:
        raise ValueError("idem_token braucht 16 Byte")
    g = Golden(name, "SPEC §10.4 (Frame, Plain)", ["A-4"], complete=False,
               note="frame_chk fehlt: Abdeckung und Kettenwert sind in §10.4 nicht festgelegt (Lücke L-2)")
    payload_total = sum(len(p) for *_, p in ops)
    frame_len = 8 + 1 + 2 + 16 + 4 + 1 + 39 * len(ops) + payload_total
    g.add("frame_len", le(frame_len, 4), f"Länge ab stamp bis ops_payload: {frame_len}")
    g.add("stamp", le(stamp, 8), "Stamp")
    g.add("kind", le(kind, 1), "1 = Change")
    g.add("n_ops", le(len(ops), 2), "Anzahl Ops")
    g.add("idem_token", idem_token, "Idempotenz-Token, 16 Byte")
    g.add("key_epoch", le(key_epoch, 4), "Schlüsselepoche")
    g.add("flags", le(frame_flags, 1), "Frame-Flags, Bit 7 = Siegel (hier 0)")
    for i, (space, cluster, op_kind, op_uid, _) in enumerate(ops):
        g.add(f"op{i}.space", le(space, 2), f"Op {i}: Space")
        g.add(f"op{i}.cluster", cluster, f"Op {i}: Cluster, 16 Byte")
        g.add(f"op{i}.op_kind", le(op_kind, 1), f"Op {i}: Art")
        g.add(f"op{i}.op_uid", op_uid, f"Op {i}: op_uid, 16 Byte")
        g.add(f"op{i}.payload_len", le(len(ops[i][4]), 4), f"Op {i}: Länge der Nutzlast")
    for i, (*_, payload) in enumerate(ops):
        g.add(f"ops_payload.{i}", payload, f"Op {i}: Nutzlast, Byte-Eingabe")
    if len(g.body()) != 4 + frame_len:
        raise AssertionError("Frame-Länge und Feldsumme stimmen nicht überein")
    return g


# ---------------------------------------------------------------------------
# Golden-Vektoren (fest, damit jeder Lauf reproduzierbar ist)
# ---------------------------------------------------------------------------

INSTANCE_ID = bytes(range(16))
INSTANCE_ID_B = bytes.fromhex("ffeeddccbbaa99887766554433221100")


def all_goldens() -> list[Golden]:
    return [
        format_plain("format_plain", 0, 0, INSTANCE_ID, 1, 0x0102030405060708),
        format_plain("format_plain_features", 0x8, 1 << 32, INSTANCE_ID_B, 1, 1700000000000),
        segment_header_plain("segment_header_plain", 0, 0x1122334455667788, 1, 7, 0xAABBCCDD),
        frame_body_plain(
            "frame_body_plain", stamp=100, kind=1, idem_token=bytes(range(0xA0, 0xB0)),
            key_epoch=3, frame_flags=0,
            ops=[
                (1, bytes(range(0x10, 0x20)), 1, bytes(range(0x20, 0x30)), b"abc"),
                (2, bytes(range(0x30, 0x40)), 2, bytes(range(0x40, 0x50)), bytes.fromhex("deadbeef")),
            ],
        ),
    ]


# ---------------------------------------------------------------------------
# Selbsttest (Anhang A.1) und Ablaufsteuerung
# ---------------------------------------------------------------------------

A1_CASES = [
    ("CRC-32C", "123456789", lambda: crc32c(0, b"123456789"), 0xE3069283),
    ("CRC-32C", "a", lambda: crc32c(0, b"a"), 0xC1D04330),
    ("XXH64", "", lambda: xxh64(b""), 0xEF46DB3751D8E999),
    ("XXH64", "a", lambda: xxh64(b"a"), 0xD24EC4F1A98C6E5B),
    ("XXH64", "abc", lambda: xxh64(b"abc"), 0x44BC2CF5AD770999),
    ("Bloom-Hash1", "k", lambda: xxh64(b"k", 0), 0xC3D31922C50B1B63),
    ("Bloom-Hash2", "k", lambda: xxh64(b"k", BLOOM_SEED2) | 1, 0x6CBC77349206F859),
]


def selftest() -> bool:
    passed = 0
    for name, inp, fn, expected in A1_CASES:
        got = fn()
        if got == expected:
            passed += 1
        else:
            print(f"FEHLER {name}({inp!r}): erwartet {expected:#018x}, ist {got:#018x}")
    positions = bloom_positions(b"k", 64, 7)
    if positions == [35, 60, 21, 46, 7, 32, 57]:
        passed += 1
    else:
        print(f"FEHLER Bloom-Positionen {positions} weichen von A.1 ab")
    total = len(A1_CASES) + 1
    ok = passed == total
    print(f"Anhang A.1: {passed}/{total} Vektoren ok")
    expected_len = {"format_plain": 53, "format_plain_features": 53,
                    "segment_header_plain": 36, "frame_body_plain": None}
    for g in all_goldens():
        want = expected_len[g.name]
        length = len(g.body())
        if want is not None and length != want:
            ok = False
            print(f"FEHLER {g.name}: Länge {length}, erwartet {want}")
        else:
            print(f"Länge {g.name}: {length} Byte")
    return ok


def main(argv: list[str]) -> int:
    if "--selftest" in argv:
        return 0 if selftest() else 1
    if not selftest():
        print("Selbsttest fehlgeschlagen, keine Goldens erzeugt.", file=sys.stderr)
        return 1
    if "--check" in argv:
        bad = []
        for g in all_goldens():
            path = GOLDEN_DIR / f"{g.name}.golden"
            if not path.exists() or path.read_text(encoding="utf-8") != g.text():
                bad.append(str(path))
        if bad:
            print("Abweichung:", *bad, sep="\n  ", file=sys.stderr)
            return 1
        print(f"OK: {len(all_goldens())} Goldens stimmen mit der Erzeugung überein")
        return 0
    GOLDEN_DIR.mkdir(parents=True, exist_ok=True)
    for g in all_goldens():
        (GOLDEN_DIR / f"{g.name}.golden").write_text(g.text(), encoding="utf-8")
        print(f"geschrieben: {GOLDEN_DIR / (g.name + '.golden')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
