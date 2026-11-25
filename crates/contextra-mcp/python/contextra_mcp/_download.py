"""
Downloader and platform detection for contextra-mcp-server release binaries.
Uses strictly Python standard library for HTTPS downloads, SHA256 verification, and archive extraction.
"""

import os
import sys
import platform
import hashlib
import tempfile
import urllib.request
import tarfile
import zipfile
from pathlib import Path
from typing import Tuple

def get_target_triple() -> Tuple[str, str]:
    """
    Detect system and machine architecture to return (target_triple, archive_extension).

    Returns:
        (target_triple, archive_extension) e.g. ("x86_64-unknown-linux-gnu", "tar.gz")
    """
    sys_name = platform.system()
    machine = platform.machine().lower()

    # Map architecture synonyms
    if machine in ("x86_64", "amd64", "x64"):
        arch = "x86_64"
    elif machine in ("aarch64", "arm64"):
        arch = "aarch64"
    else:
        arch = machine

    if sys_name == "Linux":
        if arch == "x86_64":
            return "x86_64-unknown-linux-gnu", "tar.gz"
        elif arch == "aarch64":
            return "aarch64-unknown-linux-gnu", "tar.gz"
    elif sys_name == "Darwin":
        if arch == "x86_64":
            return "x86_64-apple-darwin", "tar.gz"
        elif arch == "aarch64":
            return "aarch64-apple-darwin", "tar.gz"
    elif sys_name == "Windows":
        if arch == "x86_64":
            return "x86_64-pc-windows-msvc", "zip"

    raise RuntimeError(
        f"Unsupported platform '{sys_name} ({machine})'.\n"
        "Please set the CONTEXTRA_MCP_BINARY environment variable pointing to a precompiled "
        "`contextra-mcp-server` executable."
    )

def get_cache_dir(version: str) -> Path:
    """
    Determine local user cache directory for contextra-mcp without external dependencies.
    Creates directory with 0o700 permissions if it does not exist.
    """
    if platform.system() == "Windows":
        local_app_data = os.environ.get("LOCALAPPDATA")
        if local_app_data:
            base = Path(local_app_data)
        else:
            base = Path.home() / "AppData" / "Local"
    else:
        xdg_cache = os.environ.get("XDG_CACHE_HOME")
        if xdg_cache:
            base = Path(xdg_cache)
        else:
            base = Path.home() / ".cache"

    cache_dir = base / "contextra-mcp" / version
    cache_dir.mkdir(parents=True, exist_ok=True, mode=0o700)
    return cache_dir

def get_cached_binary_path(version: str) -> Path:
    """Return path to the cached contextra-mcp-server binary for the given version."""
    bin_name = "contextra-mcp-server.exe" if platform.system() == "Windows" else "contextra-mcp-server"
    return get_cache_dir(version) / bin_name

def validate_archive_member(name: str) -> None:
    """Check archive member path for path traversal vulnerabilities."""
    norm_name = os.path.normpath(name)
    if (
        norm_name.startswith("..")
        or norm_name.startswith("/")
        or norm_name.startswith("\\")
        or os.path.isabs(name)
        or ":" in name
    ):
        raise ValueError(f"Path traversal detected in archive member name: '{name}'")

def download_release_binary(version: str) -> Path:
    """
    Download release binary from GitHub, verify SHA256 checksum, extract securely,
    and return path to extracted executable.
    """
    triple, ext = get_target_triple()
    archive_filename = f"contextra-mcp-server-{triple}.{ext}"
    base_release_url = f"https://github.com/tfufuz1/contextra/releases/download/v{version}"
    archive_url = f"{base_release_url}/{archive_filename}"
    checksums_url = f"{base_release_url}/SHA256SUMS"

    if not archive_url.startswith("https://") or not checksums_url.startswith("https://"):
        raise ValueError("Download URLs must use HTTPS protocol.")

    cache_dir = get_cache_dir(version)
    target_bin_name = "contextra-mcp-server.exe" if platform.system() == "Windows" else "contextra-mcp-server"
    target_bin_path = cache_dir / target_bin_name

    headers = {"User-Agent": "contextra-mcp-launcher"}

    # 1. Fetch SHA256SUMS
    sys.stderr.write(f"[contextra-mcp] Fetching checksums from {checksums_url}...\n")
    sys.stderr.flush()

    req = urllib.request.Request(checksums_url, headers=headers)
    with urllib.request.urlopen(req) as resp:
        if not resp.geturl().startswith("https://"):
            raise ValueError("Insecure HTTP redirect detected during SHA256SUMS download.")
        checksum_content = resp.read().decode("utf-8")

    expected_sha256 = None
    for line in checksum_content.splitlines():
        line = line.strip()
        if not line:
            continue
        parts = line.split(maxsplit=1)
        if len(parts) == 2:
            sha, fname = parts[0], parts[1].lstrip("*")
            if fname.strip() == archive_filename:
                expected_sha256 = sha.lower()
                break

    if not expected_sha256:
        raise RuntimeError(
            f"Could not find SHA256 checksum for '{archive_filename}' in SHA256SUMS."
        )

    # 2. Download Archive to Temp File and Calculate SHA256
    sys.stderr.write(f"[contextra-mcp] Downloading release binary from {archive_url}...\n")
    sys.stderr.flush()

    req = urllib.request.Request(archive_url, headers=headers)
    hasher = hashlib.sha256()

    with tempfile.NamedTemporaryFile(delete=False, suffix=f".{ext}") as tmp_file:
        tmp_path = Path(tmp_file.name)
        try:
            with urllib.request.urlopen(req) as resp:
                if not resp.geturl().startswith("https://"):
                    raise ValueError("Insecure HTTP redirect detected during archive download.")
                while True:
                    chunk = resp.read(65536)
                    if not chunk:
                        break
                    hasher.update(chunk)
                    tmp_file.write(chunk)
            tmp_file.flush()
        except Exception:
            if tmp_path.exists():
                tmp_path.unlink()
            raise

    # 3. Verify SHA256
    actual_sha256 = hasher.hexdigest().lower()
    if actual_sha256 != expected_sha256:
        if tmp_path.exists():
            tmp_path.unlink()
        raise ValueError(
            f"SHA256 checksum mismatch for {archive_filename}.\n"
            f"Expected: {expected_sha256}\n"
            f"Actual:   {actual_sha256}"
        )

    # 4. Safe Extraction
    try:
        extracted = False
        if ext == "tar.gz":
            with tarfile.open(tmp_path, "r:gz") as tar:
                for member in tar.getmembers():
                    validate_archive_member(member.name)
                    member_basename = os.path.basename(member.name)
                    if member_basename == target_bin_name:
                        f = tar.extractfile(member)
                        if f is None:
                            raise RuntimeError(f"Could not read member {member.name} from tar archive.")
                        with open(target_bin_path, "wb") as out_f:
                            shutil_copyfile(f, out_f)
                        extracted = True
                        break
        elif ext == "zip":
            with zipfile.ZipFile(tmp_path, "r") as zf:
                for info in zf.infolist():
                    validate_archive_member(info.filename)
                    info_basename = os.path.basename(info.filename)
                    if info_basename == target_bin_name:
                        with zf.open(info) as f, open(target_bin_path, "wb") as out_f:
                            shutil_copyfile(f, out_f)
                        extracted = True
                        break

        if not extracted:
            raise RuntimeError(
                f"Binary '{target_bin_name}' was not found inside downloaded archive '{archive_filename}'."
            )

        if platform.system() != "Windows":
            os.chmod(target_bin_path, 0o755)

        sys.stderr.write(f"[contextra-mcp] Successfully installed binary to {target_bin_path}\n")
        sys.stderr.flush()
        return target_bin_path

    finally:
        if tmp_path.exists():
            tmp_path.unlink()

def shutil_copyfile(src, dst):
    """Helper to copy byte stream from src file-like object to dst file-like object."""
    while True:
        buf = src.read(65536)
        if not buf:
            break
        dst.write(buf)
