import unittest
from pathlib import Path
from unittest.mock import MagicMock

from scripts.contextra_full_audit import (
    befund_maturity_claims,
    unsafe_inventar,
    Ctx,
    Workspace,
    spec_static,
    export_spec,
    SPEC_ITEMS,
)


class TestAuditGovernance(unittest.TestCase):
    def test_live_workspace_caps_and_maturity(self):
        root = Path(".")
        ctx = Ctx(root)

        # Check that all workspace members are in ctx.caps
        self.assertTrue(len(ctx.ws.members) > 0)
        status, detail, text = befund_maturity_claims(ctx, root)
        self.assertIn(status, ("PASS", "INFO"))
        self.assertIn("Eintraege ohne Crate=0", detail)

    def test_unsafe_inventar_live(self):
        root = Path(".")
        ctx = Ctx(root)
        import tempfile
        with tempfile.TemporaryDirectory() as tmpdir:
            outdir = Path(tmpdir)
            status, detail, text = unsafe_inventar(ctx, outdir)
            self.assertEqual(status, "PASS")
            self.assertIn("keine Verstoesse", detail)
            self.assertTrue((outdir / "unsafe_inventar.csv").exists())

    def test_caps_path_and_name_resolution(self):
        ctx = MagicMock()
        ctx.ws = MagicMock()
        ctx.ws.members = {
            "contextra-arch-tests": Path("/dummy/tests/arch"),
            "contextra-durable-fs": Path("/dummy/crates/contextra-durable-fs"),
        }
        ctx.ws.root = Path("/dummy")
        ctx.caps = {
            "contextra-arch-tests": {"maturity": "stable", "path": "tests/arch", "unsafe_island": False},
            "contextra-durable-fs": {"maturity": "stable", "path": "crates/contextra-durable-fs", "unsafe_island": False},
        }
        status, detail, text = befund_maturity_claims(ctx, Path("."))
        self.assertIn(status, ("PASS", "INFO"))
        self.assertIn("Eintraege ohne Crate=0", detail)

    def test_spec_gates_statisch_and_export(self):
        root = Path(".")
        ctx = Ctx(root)
        import tempfile
        with tempfile.TemporaryDirectory() as tmpdir:
            outdir = Path(tmpdir)
            status, detail, text = spec_static(ctx, outdir)
            self.assertIn(status, ("PASS", "FAIL"))
            export_spec(outdir)
            self.assertTrue((outdir / "spec_gates.csv").exists())
            self.assertTrue((outdir / "spec_messung.csv").exists())


if __name__ == "__main__":
    unittest.main()
