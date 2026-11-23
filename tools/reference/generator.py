#!/usr/bin/env python3
"""Unabhängiger Referenzcodierer für contextra-storage (SPEC §19.4, Anhang A).

Dieses Skript ist die Pflichtquelle der Goldens. Es hängt nicht vom Rust-Kern ab
und schreibt die Golden-Dateien unter crates/contextra-storage/tests/format/golden/.

Abschnitt „Seal-Rahmen“ (SPEC §10.10, §10.1, §10.4, §10.5, §10.7, §16.3):
  - XChaCha20-Poly1305 (UE-10): HChaCha20-Ableitung plus ChaCha20-Poly1305
    (RFC 8439). HChaCha20 steht hier selbst; die AEAD-Stufe ist die Bibliothek
    `cryptography` (als unabhängige Referenz, nicht der Kern).
  - Seal-Rahmen: nonce[24] ‖ ciphertext ‖ tag[16]. Der Tag ersetzt das chk-Feld
    (§10.1); die Nonce steht davor (§10.7).
  - AAD-Kodierung: Art-Byte ‖ feste Felder, little-endian (siehe unten).

Quellen der Testvektoren (in Selbsttests geprüft, bevor etwas geschrieben wird):
  - RFC 8439 §2.8.2 (AEAD_CHACHA20_POLY1305).
  - draft-irtf-cfrg-xchacha-03 §2.2.1 (HChaCha20) und Anhang A.3.1
    (AEAD_XCHACHA20_POLY1305). Der Entwurf ist abgelaufen; die Vektoren sind
    öffentlich und stabil.

Aufruf:  python3 tools/reference/generator.py            (schreibt die Goldens)
         python3 tools/reference/generator.py --selftest (nur Selbsttests)
"""

from __future__ import annotations

import pathlib
import struct
import sys

from cryptography.hazmat.primitives.ciphers.aead import ChaCha20Poly1305

ROOT = pathlib.Path(__file__).resolve().parents[2]
GOLDEN_DIR = ROOT / "crates" / "contextra-storage" / "tests" / "format" / "golden"

MASK32 = 0xFFFFFFFF
SEAL_NONCE_LEN = 24
SEAL_TAG_LEN = 16
SEAL_LABEL = b"ctxs seal v1"


# --- ChaCha20 / HChaCha20 (pure Python, RFC 8439 §2.1 und draft §2.2) -------

def _rotl32(v: int, n: int) -> int:
    return ((v << n) & MASK32) | (v >> (32 - n))


def _quarter_round(s: list[int], a: int, b: int, c: int, d: int) -> None:
    s[a] = (s[a] + s[b]) & MASK32
    s[d] = _rotl32(s[d] ^ s[a], 16)
    s[c] = (s[c] + s[d]) & MASK32
    s[b] = _rotl32(s[b] ^ s[c], 12)
    s[a] = (s[a] + s[b]) & MASK32
    s[d] = _rotl32(s[d] ^ s[a], 8)
    s[c] = (s[c] + s[d]) & MASK32
    s[b] = _rotl32(s[b] ^ s[c], 7)


def _rounds20(state: list[int]) -> list[int]:
    """20 Runden (10 Doppelrunden), Spaltenrunde gefolgt von Diagonalrunde."""
    s = list(state)
    for _ in range(10):
        _quarter_round(s, 0, 4, 8, 12)
        _quarter_round(s, 1, 5, 9, 13)
        _quarter_round(s, 2, 6, 10, 14)
        _quarter_round(s, 3, 7, 11, 15)
        _quarter_round(s, 0, 5, 10, 15)
        _quarter_round(s, 1, 6, 11, 12)
        _quarter_round(s, 2, 7, 8, 13)
        _quarter_round(s, 3, 4, 9, 14)
    return s


_CONSTANTS = (0x61707865, 0x3320646E, 0x79622D32, 0x6B206574)


def hchacha20(key: bytes, nonce16: bytes) -> bytes:
    """HChaCha20 (draft-irtf-cfrg-xchacha-03 §2.2): 32-Byte-Unterschlüssel."""
    if len(key) != 32 or len(nonce16) != 16:
        raise ValueError("HChaCha20: Schlüssel 32 B, Nonce 16 B")
    words = list(_CONSTANTS) + list(struct.unpack("<8I", key)) + list(struct.unpack("<4I", nonce16))
    s = _rounds20(words)
    # Keine Addition des Anfangszustands; Ausgabe sind Wörter 0..3 und 12..15.
    return struct.pack("<8I", *(s[0:4] + s[12:16]))


# --- XChaCha20-Poly1305 ------------------------------------------------------

def xchacha20poly1305_seal(key: bytes, nonce24: bytes, aad: bytes, plain: bytes) -> bytes:
    """Liefert ciphertext ‖ tag16 (wie AEAD-Seal)."""
    if len(nonce24) != SEAL_NONCE_LEN:
        raise ValueError("XChaCha20: Nonce muss 24 B lang sein")
    subkey = hchacha20(key, nonce24[:16])
    inner_nonce = b"\x00\x00\x00\x00" + nonce24[16:24]
    return ChaCha20Poly1305(subkey).encrypt(inner_nonce, plain, aad)


def xchacha20poly1305_open(key: bytes, nonce24: bytes, aad: bytes, sealed: bytes) -> bytes:
    subkey = hchacha20(key, nonce24[:16])
    inner_nonce = b"\x00\x00\x00\x00" + nonce24[16:24]
    return ChaCha20Poly1305(subkey).decrypt(inner_nonce, sealed, aad)


# --- Seal-Rahmen und AAD (SPEC §10.10, §10.4, §10.5, §10.7) -----------------

AAD_KIND_REDO_OP = 1
AAD_KIND_BLOCK = 2
AAD_KIND_BLOB = 3


def aad_redo_op(space: int, cluster: bytes, op_uid: bytes, op_index: int) -> bytes:
    """AAD (space, cluster, op_uid, op_index) nach §10.4: 37 Byte."""
    _check_len(cluster, 16, "cluster")
    _check_len(op_uid, 16, "op_uid")
    return bytes([AAD_KIND_REDO_OP]) + struct.pack("<H", space) + cluster + op_uid + struct.pack("<H", op_index)


def aad_block(space: int, cluster: bytes, block_uid: bytes) -> bytes:
    """AAD (space, cluster, block_uid) nach §10.5: 35 Byte."""
    _check_len(cluster, 16, "cluster")
    _check_len(block_uid, 16, "block_uid")
    return bytes([AAD_KIND_BLOCK]) + struct.pack("<H", space) + cluster + block_uid


def aad_blob(space: int, cluster: bytes, blob_id: int, record_uid: bytes) -> bytes:
    """AAD (space, cluster, blob_id, record_uid) nach §10.7: 43 Byte."""
    _check_len(cluster, 16, "cluster")
    _check_len(record_uid, 16, "record_uid")
    return bytes([AAD_KIND_BLOB]) + struct.pack("<H", space) + cluster + struct.pack("<Q", blob_id) + record_uid


def seal_frame(key: bytes, nonce24: bytes, aad: bytes, plain: bytes) -> bytes:
    """Seal-Rahmen: nonce[24] ‖ ciphertext ‖ tag[16]."""
    return nonce24 + xchacha20poly1305_seal(key, nonce24, aad, plain)


def seal_key_context(key_epoch: int) -> bytes:
    """Kontext der Kdf-Ableitung: "ctxs seal v1" ‖ key_epoch u32 (§10.10)."""
    return SEAL_LABEL + struct.pack("<I", key_epoch)


def _check_len(value: bytes, want: int, name: str) -> None:
    if len(value) != want:
        raise ValueError(f"{name}: erwartet {want} Byte, erhalten {len(value)}")


# --- Selbsttests gegen öffentliche Vektoren ---------------------------------

def _h(s: str) -> bytes:
    return bytes.fromhex(s.replace(" ", "").replace(":", ""))


def selftest() -> None:
    # 1. HChaCha20: draft-irtf-cfrg-xchacha-03 §2.2.1
    got = hchacha20(
        _h("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"),
        _h("000000090000004a0000000031415927"),
    )
    want = _h("82413b4227b27bfed30e42508a877d73a0f9e4d58a74a853c12ec41326d3ecdc")
    _assert(got == want, "HChaCha20 (draft-irtf-cfrg-xchacha-03 §2.2.1)")

    # 2. ChaCha20-Poly1305: RFC 8439 §2.8.2 (Bibliotheksstufe)
    rfc_plain = (
        b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip "
        b"for the future, sunscreen would be it."
    )
    rfc_key = _h("808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f")
    rfc_nonce = _h("070000004041424344454647")
    rfc_aad = _h("50515253c0c1c2c3c4c5c6c7")
    rfc_ct = _h(
        "d31a8d34648e60db7b86afbc53ef7ec2a4aded51296e08fea9e2b5a736ee62d6"
        "3dbea45e8ca9671282fafb69da92728b1a71de0a9e060b2905d6a5b67ecd3b36"
        "92ddbd7f2d778b8c9803aee328091b58fab324e4fad675945585808b4831d7bc"
        "3ff4def08e4b7a9de576d26586cec64b6116"
        "1ae10b594f09e26a7e902ecbd0600691"
    )
    _assert(len(rfc_plain) == 114, "RFC 8439 §2.8.2: Klartext 114 B")
    got = ChaCha20Poly1305(rfc_key).encrypt(rfc_nonce, rfc_plain, rfc_aad)
    _assert(got == rfc_ct, "ChaCha20-Poly1305 (RFC 8439 §2.8.2, Ciphertext ‖ Tag)")

    # 3. XChaCha20-Poly1305: draft-irtf-cfrg-xchacha-03 Anhang A.3.1
    xkey = _h("808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f")
    xnonce = _h("404142434445464748494a4b4c4d4e4f5051525354555657")
    xaad = _h("50515253c0c1c2c3c4c5c6c7")
    xplain = _h(
        "4c616469657320616e642047656e746c656d656e206f662074686520636c6173"
        "73206f66202739393a204966204920636f756c64206f6666657220796f75206f"
        "6e6c79206f6e652074697020666f7220746865206675747572652c2073756e73"
        "637265656e20776f756c642062652069742e"
    )
    xsealed = _h(
        "bd6d179d3e83d43b9576579493c0e939572a1700252bfaccbed2902c21396cbb"
        "731c7f1b0b4aa6440bf3a82f4eda7e39ae64c6708c54c216cb96b72e1213b452"
        "2f8c9ba40db5d945b11b69b982c1bb9e3f3fac2bc369488f76b2383565d3fff9"
        "21f9664c97637da9768812f615c68b13b52e"
        "c0875924c1c7987947deafd8780acf49"
    )
    _assert(xchacha20poly1305_seal(xkey, xnonce, xaad, xplain) == xsealed,
            "XChaCha20-Poly1305 (draft-irtf-cfrg-xchacha-03 A.3.1)")
    _assert(xchacha20poly1305_open(xkey, xnonce, xaad, xsealed) == xplain,
            "XChaCha20-Poly1305 open (draft A.3.1)")

    # 4. Manipulationsprobe: jede Änderung wird abgelehnt (Referenzseite).
    from cryptography.exceptions import InvalidTag
    bad = bytearray(xsealed)
    bad[0] ^= 0x01
    try:
        xchacha20poly1305_open(xkey, xnonce, xaad, bytes(bad))
    except InvalidTag:
        pass
    else:
        _fail("Referenz akzeptiert manipulierten Text")
    print("selftest ok: HChaCha20, ChaCha20-Poly1305 (RFC 8439), XChaCha20-Poly1305 (draft A.3.1)")


def _assert(cond: bool, what: str) -> None:
    if not cond:
        _fail(what)
    print(f"  ok  {what}")


def _fail(what: str) -> None:
    print(f"SELFTEST FEHLGESCHLAGEN: {what}", file=sys.stderr)
    raise SystemExit(1)


# --- Golden: seal_frame ------------------------------------------------------

def _case_vectors() -> list[tuple[str, bytes, bytes, bytes, bytes]]:
    """(name, key, nonce24, aad, plain) — feste, nachvollziehbare Eingaben."""
    key = bytes(range(0x20))
    cluster = bytes(range(0x10, 0x20))
    nonce = bytes(range(0xA0, 0xB8))  # 24 Byte
    return [
        ("blob_record", key, nonce,
         aad_blob(1, cluster, 0x0102030405060708, bytes(range(0x30, 0x40))),
         b"ctxs blob body"),
        ("redo_op", key, nonce,
         aad_redo_op(2, cluster, bytes(range(0x50, 0x60)), 7),
         bytes(range(0x00, 0x40))),
        ("block", key, bytes(range(0x60, 0x78)),
         aad_block(3, cluster, bytes(range(0x70, 0x80))),
         b"block data block data block data"),
        ("block_empty_plain", key, bytes(range(0x90, 0xA8)),
         aad_block(0, bytes(16), bytes(16)),
         b""),
        ("block_odd_97", key, bytes(range(0xC0, 0xD8)),
         aad_block(4, cluster, bytes(range(0xE0, 0xF0))),
         bytes((i * 37 + 11) & 0xFF for i in range(97))),
        ("blob_multi_block_1000", key, bytes(range(0x10, 0x28)),
         aad_blob(5, cluster, 0xFFEEDDCCBBAA9988, bytes(range(0x40, 0x50))),
         bytes((i * 91 + 3) & 0xFF for i in range(1000))),
        ("draft_a31_raw", _h("808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f"),
         _h("404142434445464748494a4b4c4d4e4f5051525354555657"),
         _h("50515253c0c1c2c3c4c5c6c7"),
         _h("4c616469657320616e642047656e746c656d656e206f662074686520636c6173"
            "73206f66202739393a204966204920636f756c64206f6666657220796f75206f"
            "6e6c79206f6e652074697020666f7220746865206675747572652c2073756e73"
            "637265656e20776f756c642062652069742e")),
    ]


def write_golden() -> None:
    lines = [
        "# Golden seal_frame — tools/reference/generator.py (Abschnitt Seal-Rahmen)",
        "# SPEC §10.10, §10.1 (Tag ersetzt chk), §10.7 (Nonce vor Content).",
        "# Zeile frame|name|key|nonce24|aad|plain|sealed   (hex, ohne Trennzeichen)",
        "#   sealed = nonce24 ‖ ciphertext ‖ tag16",
        "# Zeile label|epoch|hex   (Kontext der Kdf: \"ctxs seal v1\" ‖ key_epoch u32 LE)",
        "# Quelle der Primitive: RFC 8439 §2.8.2; draft-irtf-cfrg-xchacha-03 §2.2.1, A.3.1",
    ]
    for name, key, nonce, aad, plain in _case_vectors():
        frame = seal_frame(key, nonce, aad, plain)
        lines.append("|".join(["frame", name, key.hex(), nonce.hex(), aad.hex(), plain.hex(), frame.hex()]))
    for epoch in (1, 0x01020304):
        lines.append(f"label|{epoch}|{seal_key_context(epoch).hex()}")
    GOLDEN_DIR.mkdir(parents=True, exist_ok=True)
    out = GOLDEN_DIR / "seal_frame.txt"
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"geschrieben: {out.relative_to(ROOT)} ({len(lines) - 6} Fälle)")


def main(argv: list[str]) -> int:
    selftest()
    if "--selftest" in argv:
        return 0
    write_golden()
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
