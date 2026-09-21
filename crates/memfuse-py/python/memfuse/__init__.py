try:
    from ._memfuse import *
    from ._memfuse import __version__
except ImportError:
    __version__ = "0.1.0"
