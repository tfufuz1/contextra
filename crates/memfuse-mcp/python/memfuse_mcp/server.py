#!/usr/bin/env python3
"""
Python wrapper entry-point for memfuse-mcp-server binary.
Enables running `memfuse-mcp` seamlessly via `uvx memfuse-mcp`.
"""

import os
import sys
import shutil
import subprocess
from pathlib import Path

from memfuse_mcp import __version__
from memfuse_mcp._download import get_cached_binary_path, download_release_binary

def find_repo_root() -> Path:
    """Recursively search upward from current file to find repository root containing Cargo.toml."""
    curr = Path(__file__).resolve().parent
    for p in [curr] + list(curr.parents):
        if (p / "Cargo.toml").exists() and (p / "crates" / "memfuse-mcp").exists():
            return p
    return curr

def is_executable(path: Path) -> bool:
    """Check whether a path exists, is a file, and is executable."""
    return path.is_file() and os.access(path, os.X_OK)

def find_mcp_binary() -> str:
    """
    Locate or download the memfuse-mcp-server binary following strict priority:
    1. MEMFUSE_MCP_BINARY environment variable (explicit binary path).
    2. memfuse-mcp-server in system PATH.
    3. Cached release binary (~/.cache/memfuse-mcp/<version>/ or %LOCALAPPDATA%).
    4. Download GitHub release binary (SHA256 verified).
    5. Repo tree & Cargo build fallback (ONLY if MEMFUSE_MCP_ALLOW_REPO_BUILD=1).
    """
    # 1. Check explicit environment override
    env_bin = os.environ.get("MEMFUSE_MCP_BINARY")
    if env_bin:
        env_path = Path(env_bin)
        if is_executable(env_path):
            return str(env_path)
        sys.stderr.write(
            f"[memfuse-mcp] Warning: MEMFUSE_MCP_BINARY set to '{env_bin}', but file is missing or not executable.\n"
        )
        sys.stderr.flush()

    # 2. Check system PATH
    which_bin = shutil.which("memfuse-mcp-server")
    if which_bin and is_executable(Path(which_bin)):
        return which_bin

    # 3. Check local cache directory
    cached_bin = get_cached_binary_path(__version__)
    if is_executable(cached_bin):
        return str(cached_bin)

    # 4. Download release binary from GitHub
    try:
        downloaded_bin = download_release_binary(__version__)
        if is_executable(downloaded_bin):
            return str(downloaded_bin)
    except Exception as exc:
        sys.stderr.write(f"[memfuse-mcp] Automatic binary download failed: {exc}\n")
        sys.stderr.flush()

    # 5. Repo tree / Cargo build fallback ONLY IF MEMFUSE_MCP_ALLOW_REPO_BUILD=1
    if os.environ.get("MEMFUSE_MCP_ALLOW_REPO_BUILD") == "1":
        repo_root = find_repo_root()
        candidates = [
            repo_root / "target" / "release" / "memfuse-mcp-server",
            repo_root / "target" / "debug" / "memfuse-mcp-server",
            Path.cwd() / "target" / "release" / "memfuse-mcp-server",
            Path.cwd() / "target" / "debug" / "memfuse-mcp-server",
        ]

        for candidate in candidates:
            if is_executable(candidate):
                return str(candidate)

        if (repo_root / "Cargo.toml").exists():
            sys.stderr.write(
                "[memfuse-mcp] MEMFUSE_MCP_ALLOW_REPO_BUILD=1 is set. Building release binary via Cargo...\n"
            )
            sys.stderr.flush()
            cmd = ["cargo", "build", "--release", "-p", "memfuse-mcp", "--bin", "memfuse-mcp-server"]
            res = subprocess.run(cmd, cwd=str(repo_root))
            if res.returncode == 0:
                target_bin = repo_root / "target" / "release" / "memfuse-mcp-server"
                if is_executable(target_bin):
                    return str(target_bin)

    raise FileNotFoundError(
        "Could not locate or download `memfuse-mcp-server` executable.\n"
        "Options to resolve:\n"
        "1. Set MEMFUSE_MCP_BINARY environment variable pointing to the precompiled binary.\n"
        "2. Ensure memfuse-mcp-server is available in your system PATH.\n"
        "3. Set MEMFUSE_MCP_ALLOW_REPO_BUILD=1 to allow building from source via Cargo in developer repositories."
    )

def main():
    try:
        binary_path = find_mcp_binary()
    except Exception as e:
        sys.stderr.write(f"Error: {e}\n")
        sys.exit(1)

    args = [binary_path] + sys.argv[1:]

    # Use os.execv on POSIX for zero-overhead process replacement
    if hasattr(os, "execv"):
        try:
            os.execv(binary_path, args)
        except OSError:
            pass

    # Fallback to subprocess for platforms/environments where execv fails or isn't available
    res = subprocess.run(args)
    sys.exit(res.returncode)

if __name__ == "__main__":
    main()
