#!/usr/bin/env python3
"""Unabhängiger Referenzcodierer für contextra-storage (SPEC §19.4, §22 G-2, WP-2.1).

Umfang dieser Stufe (Modus Plain, nur Standardbibliothek):
  - CRC-32C (Port Checksum, SPEC §4)
  - XXH64 und Bloom-Positionen (SPEC §10.5.1, Anhang A.1)
  - kanonisches varint (SPEC §10.1)
  - FORMAT-Kopf (SPEC §10.2) und Segmentkopf (SPEC §10.4), Modus Plain

Nicht Teil dieser Stufe: Mac16, AEAD, Kdf, KEYS, Gruppen, Frames, Catalog.
Diese Modi brauchen BLAKE3 und XChaCha20-Poly1305 (siehe Offene Punkte in
tests/format/README.md) und kommen mit eigenen Goldens.

Aufruf:
  python3 tools/reference/generator.py            # schreibt die Goldens
  python3 tools/reference/generator.py --check    # prüft Anhang A.1 und die Goldens
"""

from __future__ import annotations

import json
import struct
import sys
from pathlib import Path

MASK64 = (1 << 64) - 1
MASK32 = (1 << 32) - 1

# ---------------------------------------------------------------------------
# CRC-32C (Castagnoli), reflektiertes Polynom 0x82F63B78
# SPEC §4: crc32c(seed, data) setzt einen Strom fort;
#          Ergebnis = !update(!seed, data), seed = 0 liefert den üblichen CRC.
# ---------------------------------------------------------------------------

_CRC_TABLE = []
for _i in range(256):
    _c = _i
    for _ in range(8):
        _c = (_c >> 1) ^ 0x82F63B78 if _c & 1 else _c >> 1
    _CRC_TABLE.append(_c)


def _crc_update(crc: int, data: bytes) -> int:
    for b in data:
        crc = _CRC_TABLE[(crc ^ b) & 0xFF] ^ (crc >> 8)
    return crc & MASK32


def crc32c(seed: int, data: bytes) -> int:
    return (~_crc_update((~seed) & MASK32, data)) & MASK32


# ---------------------------------------------------------------------------
# XXH64 (reine Implementierung nach der XXH64-Spezifikation)
# ---------------------------------------------------------------------------

P1 = 0x9E3779B185EBCA87
P2 = 0xC2B2AE3D27D4EB4F
P3 = 0x165667B19E3779F9
P4 = 0x85EBCA77C2B2AE63
P5 = 0x27D4EB2F165667C5


def _rotl(x: int, r: int) -> int:
    return ((x << r) | (x >> (64 - r))) & MASK64


def _read64(b: bytes, i: int) -> int:
    return struct.unpack_from("<Q", b, i)[0]


def _read32(b: bytes, i: int) -> int:
    return struct.unpack_from("<I", b, i)[0]


def _xxh_round(acc: int, inp: int) -> int:
    acc = (acc + inp * P2) & MASK64
    acc = _rotl(acc, 31)
    return (acc * P1) & MASK64


def _xxh_merge(acc: int, val: int) -> int:
    val = _xxh_round(0, val)
    acc ^= val
    return (acc * P1 + P4) & MASK64


def xxh64(data: bytes, seed: int = 0) -> int:
    n = len(data)
    i = 0
    if n >= 32:
        v1 = (seed + P1 + P2) & MASK64
        v2 = (seed + P2) & MASK64
        v3 = seed & MASK64
        v4 = (seed - P1) & MASK64
        while i + 32 <= n:
            v1 = _xxh_round(v1, _read64(data, i))
            v2 = _xxh_round(v2, _read64(data, i + 8))
            v3 = _xxh_round(v3, _read64(data, i + 16))
            v4 = _xxh_round(v4, _read64(data, i + 24))
            i += 32
        h = (_rotl(v1, 1) + _rotl(v2, 7) + _rotl(v3, 12) + _rotl(v4, 18)) & MASK64
        h = _xxh_merge(h, v1)
        h = _xxh_merge(h, v2)
        h = _xxh_merge(h, v3)
        h = _xxh_merge(h, v4)
    else:
        h = (seed + P5) & MASK64
    h = (h + n) & MASK64
    while i + 8 <= n:
        k1 = _xxh_round(0, _read64(data, i))
        h ^= k1
        h = (_rotl(h, 27) * P1 + P4) & MASK64
        i += 8
    if i + 4 <= n:
        h ^= (_read32(data, i) * P1) & MASK64
        h = (_rotl(h, 23) * P2 + P3) & MASK64
        i += 4
    while i < n:
        h ^= (data[i] * P5) & MASK64
        h = (_rotl(h, 11) * P1) & MASK64
        i += 1
    h ^= h >> 33
    h = (h * P2) & MASK64
    h ^= h >> 29
    h = (h * P3) & MASK64
    h ^= h >> 32
    return h


# ---------------------------------------------------------------------------
# Bloom-Positionen (SPEC §10.5.1)
#   h1 = XXH64(key, 0); h2 = XXH64(key, 0x9E3779B97F4A7C15) | 1
#   pos_i = (h1 + i*h2) mod 2^64, dann mod nbits, für i = 0..k
# ---------------------------------------------------------------------------

BLOOM_SEED2 = 0x9E3779B97F4A7C15


def bloom_hashes(key: bytes) -> tuple[int, int]:
    h1 = xxh64(key, 0)
    h2 = xxh64(key, BLOOM_SEED2) | 1
    return h1, h2


def bloom_positions(key: bytes, nbits: int, k: int) -> list[int]:
    h1, h2 = bloom_hashes(key)
    out = []
    for i in range(k):
        pos = (h1 + i * h2) & MASK64
        out.append(pos % nbits)
    return out


# ---------------------------------------------------------------------------
# Kanonisches varint (LEB128 für u64, SPEC §10.1)
# ---------------------------------------------------------------------------

def varint_encode(v: int) -> bytes:
    if not 0 <= v <= MASK64:
        raise ValueError("varint außerhalb u64")
    out = bytearray()
    while True:
        b = v & 0x7F
        v >>= 7
        if v:
            out.append(b | 0x80)
        else:
            out.append(b)
            return bytes(out)


# ---------------------------------------------------------------------------
# FORMAT-Kopf (SPEC §10.2), Modus Plain
#   magic "CTXS" [4] | format_ver u16 (=1) | feature_req u64 | feature_opt u64 |
#   integrity_mode u8 | instance_id [16] | algo_set u16 (1) | created_ms u64 | chk
#   Vor chk: 49 B. Plain: chk = CRC-32C über die 49 B (Annahme A-1, s. README).
# ---------------------------------------------------------------------------

FORMAT_MAGIC = b"CTXS"
FORMAT_VER = 1
INTEGRITY_PLAIN = 0


def format_plain(feature_req: int, feature_opt: int, instance_id: bytes,
                 algo_set: int, created_ms: int) -> bytes:
    if len(instance_id) != 16:
        raise ValueError("instance_id braucht 16 Byte")
    body = (
        FORMAT_MAGIC
        + struct.pack("<H", FORMAT_VER)
        + struct.pack("<Q", feature_req)
        + struct.pack("<Q", feature_opt)
        + struct.pack("<B", INTEGRITY_PLAIN)
        + instance_id
        + struct.pack("<H", algo_set)
        + struct.pack("<Q", created_ms)
    )
    assert len(body) == 49, len(body)
    return body + struct.pack("<I", crc32c(0, body))


# ---------------------------------------------------------------------------
# Segmentkopf (SPEC §10.4), Modus Plain (chain_seed 4 B)
#   "CTXL" [4] | ver u16 | flags u16 | segment_id u64 | first_stamp u64 |
#   root_gen u32 | chain_seed (4 B) | chk u32
#   Gesamt 36 B.  chk = CRC-32C über die 32 B davor (Annahme A-1).
# ---------------------------------------------------------------------------

SEG_MAGIC = b"CTXL"
SEG_VER = 1


def segment_header_plain(flags: int, segment_id: int, first_stamp: int,
                         root_gen: int, chain_seed: int) -> bytes:
    body = (
        SEG_MAGIC
        + struct.pack("<H", SEG_VER)
        + struct.pack("<H", flags)
        + struct.pack("<Q", segment_id)
        + struct.pack("<Q", first_stamp)
        + struct.pack("<I", root_gen)
        + struct.pack("<I", chain_seed)
    )
    assert len(body) == 32, len(body)
    return body + struct.pack("<I", crc32c(0, body))


# ---------------------------------------------------------------------------
# Anhang A.1 (SPEC) und Selbsttest
# ---------------------------------------------------------------------------

ANHANG_A1 = [
    ("crc32c", b"123456789", 0xE3069283),
    ("crc32c", b"a", 0xC1D04330),
    ("xxh64", b"", 0xEF46DB3751D8E999),
    ("xxh64", b"a", 0xD24EC4F1A98C6E5B),
    ("xxh64", b"abc", 0x44BC2CF5AD770999),
    ("bloom_hash1", b"k", 0xC3D31922C50B1B63),
    ("bloom_hash2", b"k", 0x6CBC77349206F859),
]


def selbsttest() -> list[str]:
    fehler: list[str] = []
    for name, eingabe, erwartet in ANHANG_A1:
        if name == "crc32c":
            ist = crc32c(0, eingabe)
        elif name == "xxh64":
            ist = xxh64(eingabe, 0)
        elif name == "bloom_hash1":
            ist = bloom_hashes(eingabe)[0]
        elif name == "bloom_hash2":
            ist = bloom_hashes(eingabe)[1]
        else:  # pragma: no cover
            raise AssertionError(name)
        if ist != erwartet:
            fehler.append(f"{name}({eingabe!r}): erwartet {erwartet:#x}, ist {ist:#x}")
    if bloom_positions(b"k", 64, 7) != [35, 60, 21, 46, 7, 32, 57]:
        fehler.append("bloom_positions(k, 64, 7) weicht von A.1 ab")
    if varint_encode(300) != bytes([0xAC, 0x02]):
        fehler.append("varint(300) falsch")
    return fehler


# Testvektoren für die Goldens. Eingaben sind fest, damit der Lauf reproduzierbar ist.
VEKTOR_INSTANCE_ID = bytes(range(16))
VEKTOR_CREATED_MS = 0x0102030405060708

GOLDENS = {
    "format_plain": format_plain(0, 0, VEKTOR_INSTANCE_ID, 1, VEKTOR_CREATED_MS),
    "segment_header_plain": segment_header_plain(
        0, 0x1122334455667788, 1, 7, 0xAABBCCDD
    ),
}

GOLDEN_PFAD = Path(__file__).resolve().parents[2] / "tests" / "format" / "goldens_plain.json"


def golden_dokument() -> dict:
    return {
        "_quelle": "tools/reference/generator.py, unabhängig vom Kern",
        "_annahmen": [
            "A-1: chk ist CRC-32C mit seed 0 über alle Bytes des Kopfes vor chk.",
            "A-2: Ein Golden gilt nur zusammen mit Anhang A.1 (Primitive, geprüft).",
        ],
        "format_plain": {
            "eingabe": {
                "feature_req": 0,
                "feature_opt": 0,
                "instance_id": VEKTOR_INSTANCE_ID.hex(),
                "algo_set": 1,
                "created_ms": VEKTOR_CREATED_MS,
            },
            "laenge": len(GOLDENS["format_plain"]),
            "hex": GOLDENS["format_plain"].hex(),
        },
        "segment_header_plain": {
            "eingabe": {
                "flags": 0,
                "segment_id": 0x1122334455667788,
                "first_stamp": 1,
                "root_gen": 7,
                "chain_seed": 0xAABBCCDD,
            },
            "laenge": len(GOLDENS["segment_header_plain"]),
            "hex": GOLDENS["segment_header_plain"].hex(),
        },
    }


def main(argv: list[str]) -> int:
    fehler = selbsttest()
    if fehler:
        print("Selbsttest fehlgeschlagen:", file=sys.stderr)
        for f in fehler:
            print("  " + f, file=sys.stderr)
        return 1
    dok = golden_dokument()
    text = json.dumps(dok, indent=2, ensure_ascii=False) + "\n"
    if "--check" in argv:
        if not GOLDEN_PFAD.exists():
            print(f"fehlt: {GOLDEN_PFAD}", file=sys.stderr)
            return 1
        if GOLDEN_PFAD.read_text(encoding="utf-8") != text:
            print("Goldens weichen vom Generator ab", file=sys.stderr)
            return 1
        print("OK: Anhang A.1 und Goldens stimmen überein")
        return 0
    GOLDEN_PFAD.parent.mkdir(parents=True, exist_ok=True)
    GOLDEN_PFAD.write_text(text, encoding="utf-8")
    print(f"geschrieben: {GOLDEN_PFAD}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
