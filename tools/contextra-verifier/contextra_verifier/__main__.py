"""Module execution entrypoint for python -m contextra_verifier."""

import sys
from .cli import main

if __name__ == "__main__":
    sys.exit(main())
