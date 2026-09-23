# Publishing Process for `contextra` (Python / PyPI)

This document describes the process for preparing, testing, and publishing new releases of the `contextra` Python package to [PyPI](https://pypi.org/project/contextra/).

---

## 1. Prerequisites & Version Synchronization

Before creating a release, ensure all checks pass and versions are synchronized:

1. **Version Synchronization**:
   - Update `version` in `crates/contextra-py/pyproject.toml` (e.g. `version = "0.2.0"`).
   - Ensure `Cargo.toml` in the workspace root or crate aligns with the targeted release.

2. **Quality Gates & Tests**:
   - Ensure local Rust and Python tests pass:
     ```bash
     cargo test -p contextra-py
     cd crates/contextra-py && ./run_tests.sh
     ```
   - Run workspace checks:
     ```bash
     cargo run -p xtask -- check-consistency
     cargo run -p xtask -- check-jules-context-freshness
     ```

---

## 2. CI/CD Dry-Run Verification

Every Pull Request and commit to `main` triggers the `maturin-dry-run` step in `.github/workflows/rust-ci.yml`.

To test local wheel compilation:
```bash
python -m maturin build --manifest-path crates/contextra-py/Cargo.toml --release
```

---

## 3. Creating a Release Tag

When ready to publish to PyPI:

1. Create a git tag following the format `contextra-py-v*`:
   ```bash
   git tag -a contextra-py-v0.2.0 -m "Release contextra-py v0.2.0"
   git push origin contextra-py-v0.2.0
   ```

2. Pushing this tag automatically triggers `.github/workflows/publish-pypi.yml`.

---

## 4. Manual Workflow Trigger (Alternative)

If you need to publish or test without pushing a tag:

1. Navigate to GitHub Actions -> **Publish contextra-py to PyPI**.
2. Click **Run workflow** -> Select branch (e.g., `main`).
3. Click **Run workflow**.

---

## 5. Environment & Credentials Configuration

The workflow uses `PyO3/maturin-action` to compile wheels for Linux (x86_64 manylinux), macOS (x86_64, aarch64), and Windows (x86_64).

Publishing authenticates via the Repository Secret:
- `PYPI_API_TOKEN` (configured under GitHub Repository Settings -> Secrets and variables -> Actions).

The secret is passed via `MATURIN_PYPI_TOKEN` securely without being printed in logs.

---

## 6. Package Architecture & MCP Extras

- **`contextra-mcp` (Rust Package)**: A standalone Rust binary crate delivering the high-performance native Contextra Model Context Protocol (MCP) server executable.
- **`contextra[mcp]` (Python Package Extra)**: The optional Python extra for `contextra-py`, enabling Python FastMCP server integration via `contextra.mcp.create_mcp_server(db_path, embed_fn=...)`.
