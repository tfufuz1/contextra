"""TLV (Type-Length-Value) parsing and canonical validation for DeletionProof v4."""

import struct
from typing import List, Tuple

MAX_INPUT_SIZE = 16 * 1024 * 1024  # 16 MiB max input limit
KNOWN_V4_CRITICAL_TAGS = set(range(0x8001, 0x8015))  # 0x8001..0x8014


class TlvFormatError(Exception):
    """Exception raised for TLV format or canonical ordering violations."""

    def __init__(self, message: str):
        super().__init__(message)
        self.message = message


class TlvField:
    """Represents a single TLV field element."""

    def __init__(self, tag: int, value: bytes):
        self.tag = tag
        self.value = value

    @property
    def is_critical(self) -> bool:
        """Returns True if top bit 0x8000 is set on tag."""
        return bool(self.tag & 0x8000)

    def encode(self) -> bytes:
        """Encodes this field to byte stream: tag(u16 BE) | len(u32 BE) | value."""
        return struct.pack(">HI", self.tag, len(self.value)) + self.value


def parse_tlv_stream(data: bytes, allow_sub_tlv: bool = False) -> List[TlvField]:
    """Parses a binary TLV byte stream and enforces canonical TLV invariants."""
    if len(data) > MAX_INPUT_SIZE:
        raise TlvFormatError(f"Input size {len(data)} exceeds limit of {MAX_INPUT_SIZE} bytes")
    if not data:
        raise TlvFormatError("Empty TLV stream input")

    offset = 0
    fields: List[TlvField] = []
    seen_tags = set()
    prev_tag = -1

    while offset < len(data):
        if len(data) - offset < 6:
            raise TlvFormatError("Truncated TLV header")

        tag, length = struct.unpack(">HI", data[offset : offset + 6])
        offset += 6

        if len(data) - offset < length:
            raise TlvFormatError(f"Truncated TLV payload for tag 0x{tag:04X} (expected {length} bytes, available {len(data) - offset})")

        value = data[offset : offset + length]
        offset += length

        if not allow_sub_tlv:
            if tag in seen_tags:
                raise TlvFormatError(f"Duplicate TLV tag encountered: 0x{tag:04X}")
            if tag <= prev_tag:
                raise TlvFormatError(f"TLV tags not in strictly ascending order: 0x{tag:04X} <= 0x{prev_tag:04X}")

        seen_tags.add(tag)
        prev_tag = tag
        fields.append(TlvField(tag, value))

    if offset != len(data):
        raise TlvFormatError(f"Trailing bytes after TLV stream: {len(data) - offset} bytes remaining")

    return fields


def check_unknown_critical_tags(fields: List[TlvField]) -> None:
    """Rejects any critical tag (bit 0x8000 set) not in KNOWN_V4_CRITICAL_TAGS."""
    for field in fields:
        if field.is_critical and field.tag not in KNOWN_V4_CRITICAL_TAGS:
            raise TlvFormatError(f"Unknown critical TLV tag encountered: 0x{field.tag:04X}")
