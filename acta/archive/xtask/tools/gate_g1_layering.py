#!/usr/bin/env python3
"""
tools/gate_g1_layering.py — Alias / Wrapper für tools/gate_dependencies.py (Gate G1).
"""
import sys
from gate_dependencies import main, run_gate_dependencies

# Rückwärtskompatibles Alias
def run_gate_g1(caps_path, root):
    violations, warnings, _ = run_gate_dependencies(caps_path, root)
    return violations, warnings

if __name__ == "__main__":
    sys.exit(main())
