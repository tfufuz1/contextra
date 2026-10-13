"""Cryptographic wrappers for BLAKE3, Ed25519, and HMAC-SHA256."""

import hmac
import hashlib
from typing import Optional

try:
    import blake3
    HAS_BLAKE3 = True
except ImportError:
    HAS_BLAKE3 = False

try:
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
    from cryptography.exceptions import InvalidSignature
    HAS_ED25519 = True
except ImportError:
    HAS_ED25519 = False


class CryptoError(Exception):
    """Exception raised for cryptographic processing failures."""
    pass


def blake3_hash(data: bytes) -> bytes:
    """Computes BLAKE3 hash of input data."""
    if not HAS_BLAKE3:
        raise CryptoError("Python 'blake3' module is not installed")
    return blake3.blake3(data).digest()


def verify_ed25519(pubkey_bytes: bytes, signature_bytes: bytes, message_bytes: bytes) -> bool:
    """Verifies Ed25519 signature (RFC 8032)."""
    if not HAS_ED25519:
        raise CryptoError("Python 'cryptography' module is not installed (Ed25519 unavailable)")

    if len(pubkey_bytes) != 32:
        raise CryptoError(f"Invalid public key length: {len(pubkey_bytes)} (expected 32 bytes)")
    if len(signature_bytes) != 64:
        raise CryptoError(f"Invalid signature length: {len(signature_bytes)} (expected 64 bytes)")

    try:
        public_key = Ed25519PublicKey.from_public_bytes(pubkey_bytes)
        public_key.verify(signature_bytes, message_bytes)
        return True
    except InvalidSignature:
        return False
    except Exception as e:
        raise CryptoError(f"Ed25519 verification error: {str(e)}") from e


def compute_hmac_sha256(key_bytes: bytes, data_bytes: bytes) -> bytes:
    """Computes HMAC-SHA256 checksum."""
    return hmac.new(key_bytes, data_bytes, hashlib.sha256).digest()


def verify_hmac_sha256(key_bytes: bytes, signature_bytes: bytes, data_bytes: bytes) -> bool:
    """Verifies HMAC-SHA256 checksum in constant time."""
    if not key_bytes:
        return False
    expected = compute_hmac_sha256(key_bytes, data_bytes)
    return hmac.compare_digest(expected, signature_bytes)
