#!/usr/bin/env python3
"""Convenience entry point for ctxrs CLI."""
import sys
from pathlib import Path

# Ensure repo root is on sys.path
root = Path(__file__).resolve().parent
if str(root) not in sys.path:
    sys.path.insert(0, str(root))

from scripts.V1.cli import main

if __name__ == "__main__":
    sys.exit(main())
