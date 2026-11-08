#!/usr/bin/env python3
"""
tests/test_gate_g8_negative.py — Gate G8: Negativtest für Gate G1 (SPEC v4.0 §8.1 / Gate G8).

Prüft, dass absichtliche Verletzungen in einer synthetischen Konfiguration
vom Gate G1 zuverlässig erkannt werden und zum Fehlschlag (Exit-Code != 0) führen.
"""

import sys
import tempfile
import unittest
from pathlib import Path

tools_dir = Path(__file__).resolve().parent.parent / "tools"
sys.path.insert(0, str(tools_dir))

from gate_g1_layering import run_gate_g1


class TestGateG8Negative(unittest.TestCase):
    def setUp(self):
        self.root = Path(__file__).resolve().parent.parent

    def test_gate_g1_fails_on_illegal_dependency(self):
        # Synthetische capabilities.toml erzeugen, in der Klasse A strikt ohne deps definiert ist
        content = """
[version]
schema = "2"
spec = "v4"

[crates.contextra-simd]
klasse = 'A'
may_depend_on = []

[crates.contextra-types]
klasse = 'B'
may_depend_on = []
"""
        with tempfile.NamedTemporaryFile("w", suffix=".toml", delete=False) as f:
            f.write(content)
            temp_caps = Path(f.name)

        try:
            violations, warnings = run_gate_g1(temp_caps, self.root)
            # Da contextra-simd von contextra-types abhängt, muss dies als Verstoß gemeldet werden
            simd_violations = [v for v in violations if "contextra-simd" in v and "contextra-types" in v]
            self.assertTrue(
                len(simd_violations) > 0,
                f"Gate G1 hätte illegalen Klasse-A-Import von contextra-simd auf types erkennen müssen. Verstöße: {violations}"
            )
        finally:
            temp_caps.unlink(missing_ok=True)

    def test_gate_g1_fails_on_forbidden_testkit(self):
        # Synthetische capabilities.toml mit Klasse A, die testkit als dev hat
        content = """
[version]
schema = "2"
spec = "v4"

[crates.contextra-crypto]
klasse = 'A'
may_depend_on = []
"""
        with tempfile.NamedTemporaryFile("w", suffix=".toml", delete=False) as f:
            f.write(content)
            temp_caps = Path(f.name)

        try:
            violations, warnings = run_gate_g1(temp_caps, self.root)
            testkit_violations = [v for v in violations if "contextra-crypto" in v and "contextra-testkit" in v]
            self.assertTrue(
                len(testkit_violations) > 0,
                f"Gate G1 hätte verbotenes testkit für Klasse A melden müssen. Verstöße: {violations}"
            )
        finally:
            temp_caps.unlink(missing_ok=True)


if __name__ == "__main__":
    unittest.main()
