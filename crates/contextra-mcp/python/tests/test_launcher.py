"""
Offline unit tests for contextra-mcp Python launcher and release binary downloader.
Runs strictly without network calls via unittest.mock.
"""

import os
import io
import sys
import tarfile
import zipfile
import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch, MagicMock

from contextra_mcp._download import (
    get_target_triple,
    get_cache_dir,
    get_cached_binary_path,
    validate_archive_member,
    download_release_binary,
)
from contextra_mcp.server import find_mcp_binary

def create_mock_tar_gz_bytes(filename: str, content: bytes) -> bytes:
    """Helper to generate in-memory .tar.gz archive bytes containing a single file."""
    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w:gz") as tar:
        tarinfo = tarfile.TarInfo(name=filename)
        tarinfo.size = len(content)
        tar.addfile(tarinfo, io.BytesIO(content))
    return buf.getvalue()

def create_mock_zip_bytes(filename: str, content: bytes) -> bytes:
    """Helper to generate in-memory .zip archive bytes containing a single file."""
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, mode="w") as zf:
        zf.writestr(filename, content)
    return buf.getvalue()

class MockHttpResponse:
    """Mock HTTP response object for urllib.request.urlopen context manager."""
    def __init__(self, data: bytes, url: str):
        self.data = data
        self.url = url

    def geturl(self) -> str:
        return self.url

    def read(self, amt: int = -1) -> bytes:
        if amt == -1:
            res = self.data
            self.data = b""
            return res
        res = self.data[:amt]
        self.data = self.data[amt:]
        return res

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        pass

class TestLauncher(unittest.TestCase):

    def setUp(self):
        self.tmp_dir = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp_dir.cleanup)
        self.env_patch = patch.dict(
            os.environ,
            {
                "XDG_CACHE_HOME": self.tmp_dir.name,
                "LOCALAPPDATA": self.tmp_dir.name,
            },
            clear=False,
        )
        self.env_patch.start()
        self.addCleanup(self.env_patch.stop)

    def test_target_triple_detection(self):
        with patch("platform.system", return_value="Linux"), patch("platform.machine", return_value="x86_64"):
            triple, ext = get_target_triple()
            self.assertEqual(triple, "x86_64-unknown-linux-gnu")
            self.assertEqual(ext, "tar.gz")

        with patch("platform.system", return_value="Darwin"), patch("platform.machine", return_value="arm64"):
            triple, ext = get_target_triple()
            self.assertEqual(triple, "aarch64-apple-darwin")
            self.assertEqual(ext, "tar.gz")

        with patch("platform.system", return_value="Windows"), patch("platform.machine", return_value="AMD64"):
            triple, ext = get_target_triple()
            self.assertEqual(triple, "x86_64-pc-windows-msvc")
            self.assertEqual(ext, "zip")

    def test_unsupported_platform_raises_error(self):
        with patch("platform.system", return_value="FreeBSD"), patch("platform.machine", return_value="amd64"):
            with self.assertRaises(RuntimeError) as ctx:
                get_target_triple()
            self.assertIn("CONTEXTRA_MCP_BINARY", str(ctx.exception))

    def test_validate_archive_member_path_traversal(self):
        # Valid names
        validate_archive_member("contextra-mcp-server")
        validate_archive_member("bin/contextra-mcp-server")

        # Invalid path traversal names
        for bad_name in ["../bin/server", "/usr/bin/server", "..\\server", "C:\\server.exe"]:
            with self.assertRaises(ValueError):
                validate_archive_member(bad_name)

    def test_env_binary_priority(self):
        dummy_bin = Path(self.tmp_dir.name) / "custom-mcp-server"
        dummy_bin.write_bytes(b"#!binary\n")
        dummy_bin.chmod(0o755)

        with patch.dict(os.environ, {"CONTEXTRA_MCP_BINARY": str(dummy_bin)}):
            selected = find_mcp_binary()
            self.assertEqual(selected, str(dummy_bin))

    def test_path_binary_priority(self):
        dummy_path_bin = Path(self.tmp_dir.name) / "path-mcp-server"
        dummy_path_bin.write_bytes(b"#!binary\n")
        dummy_path_bin.chmod(0o755)

        env = {
            "XDG_CACHE_HOME": self.tmp_dir.name,
            "LOCALAPPDATA": self.tmp_dir.name,
        }
        with patch.dict(os.environ, env, clear=True), \
             patch("shutil.which", return_value=str(dummy_path_bin)):
            selected = find_mcp_binary()
            self.assertEqual(selected, str(dummy_path_bin))

    def test_cached_binary_priority(self):
        from contextra_mcp import __version__
        cached_path = get_cached_binary_path(__version__)
        cached_path.parent.mkdir(parents=True, exist_ok=True)
        cached_path.write_bytes(b"#!cached\n")
        cached_path.chmod(0o755)

        env = {
            "XDG_CACHE_HOME": self.tmp_dir.name,
            "LOCALAPPDATA": self.tmp_dir.name,
        }
        with patch.dict(os.environ, env, clear=True), \
             patch("shutil.which", return_value=None):
            selected = find_mcp_binary()
            self.assertEqual(selected, str(cached_path))

    def test_download_release_binary_success(self):
        version = "0.1.0"
        archive_content = b"fake-executable-binary-data"
        tar_gz_bytes = create_mock_tar_gz_bytes("contextra-mcp-server", archive_content)

        archive_hash = hashlib.sha256(tar_gz_bytes).hexdigest()
        sha256sums_text = f"{archive_hash}  contextra-mcp-server-x86_64-unknown-linux-gnu.tar.gz\n"

        def mock_urlopen(req, *args, **kwargs):
            url = req.full_url if hasattr(req, "full_url") else str(req)
            if "SHA256SUMS" in url:
                return MockHttpResponse(sha256sums_text.encode("utf-8"), url)
            else:
                return MockHttpResponse(tar_gz_bytes, url)

        with patch("platform.system", return_value="Linux"), \
             patch("platform.machine", return_value="x86_64"), \
             patch("urllib.request.urlopen", side_effect=mock_urlopen):
            bin_path = download_release_binary(version)
            self.assertTrue(bin_path.is_file())
            self.assertEqual(bin_path.read_bytes(), archive_content)

    def test_download_hash_mismatch_aborts_and_cleans_up(self):
        version = "0.1.0"
        tar_gz_bytes = create_mock_tar_gz_bytes("contextra-mcp-server", b"some-data")

        bad_sha256sums_text = "0000000000000000000000000000000000000000000000000000000000000000  contextra-mcp-server-x86_64-unknown-linux-gnu.tar.gz\n"

        def mock_urlopen(req, *args, **kwargs):
            url = req.full_url if hasattr(req, "full_url") else str(req)
            if "SHA256SUMS" in url:
                return MockHttpResponse(bad_sha256sums_text.encode("utf-8"), url)
            else:
                return MockHttpResponse(tar_gz_bytes, url)

        with patch("platform.system", return_value="Linux"), \
             patch("platform.machine", return_value="x86_64"), \
             patch("urllib.request.urlopen", side_effect=mock_urlopen):
            with self.assertRaises(ValueError) as ctx:
                download_release_binary(version)
            self.assertIn("SHA256 checksum mismatch", str(ctx.exception))

    def test_download_path_traversal_aborts(self):
        version = "0.1.0"
        tar_gz_bytes = create_mock_tar_gz_bytes("../../etc/passwd", b"root:x:0:0...")
        archive_hash = hashlib.sha256(tar_gz_bytes).hexdigest()
        sha256sums_text = f"{archive_hash}  contextra-mcp-server-x86_64-unknown-linux-gnu.tar.gz\n"

        def mock_urlopen(req, *args, **kwargs):
            url = req.full_url if hasattr(req, "full_url") else str(req)
            if "SHA256SUMS" in url:
                return MockHttpResponse(sha256sums_text.encode("utf-8"), url)
            else:
                return MockHttpResponse(tar_gz_bytes, url)

        with patch("platform.system", return_value="Linux"), \
             patch("platform.machine", return_value="x86_64"), \
             patch("urllib.request.urlopen", side_effect=mock_urlopen):
            with self.assertRaises(ValueError) as ctx:
                download_release_binary(version)
            self.assertIn("Path traversal detected", str(ctx.exception))

    def test_no_cargo_build_without_opt_in(self):
        env = {
            "XDG_CACHE_HOME": self.tmp_dir.name,
            "LOCALAPPDATA": self.tmp_dir.name,
        }
        with patch.dict(os.environ, env, clear=True), \
             patch("shutil.which", return_value=None), \
             patch("contextra_mcp.server.download_release_binary", side_effect=RuntimeError("Download disabled")):
            with self.assertRaises(FileNotFoundError) as ctx:
                find_mcp_binary()
            self.assertIn("CONTEXTRA_MCP_ALLOW_REPO_BUILD=1", str(ctx.exception))

    def test_cargo_build_with_opt_in(self):
        repo_bin = Path(self.tmp_dir.name) / "target" / "release" / "contextra-mcp-server"
        repo_bin.parent.mkdir(parents=True, exist_ok=True)
        repo_bin.write_bytes(b"#!binary\n")
        repo_bin.chmod(0o755)

        env = {
            "XDG_CACHE_HOME": self.tmp_dir.name,
            "LOCALAPPDATA": self.tmp_dir.name,
            "CONTEXTRA_MCP_ALLOW_REPO_BUILD": "1",
        }
        with patch.dict(os.environ, env, clear=True), \
             patch("shutil.which", return_value=None), \
             patch("contextra_mcp.server.download_release_binary", side_effect=RuntimeError("Download disabled")), \
             patch("contextra_mcp.server.find_repo_root", return_value=Path(self.tmp_dir.name)):
            selected = find_mcp_binary()
            self.assertEqual(selected, str(repo_bin))

if __name__ == "__main__":
    unittest.main()
