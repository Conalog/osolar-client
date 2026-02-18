#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import sys
import types

# Avoid loading setproctitle C extension in this process. On macOS + Python 3.13
# this can make forked mutmut workers unstable during pytest runs.
_setproctitle_stub = types.ModuleType("setproctitle")
_setproctitle_stub.setproctitle = lambda *_args, **_kwargs: None
sys.modules.setdefault("setproctitle", _setproctitle_stub)

from mutmut import __main__ as mutmut_main  # noqa: E402


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run mutmut with a macOS-safe process-title hook."
    )
    parser.add_argument("--max-children", type=int, default=None)
    parser.add_argument("mutant_names", nargs="*")
    args = parser.parse_args()

    # Avoid macOS proxy sysconf lookups in forked workers (urllib.getproxies).
    os.environ.setdefault("NO_PROXY", "*")
    os.environ.setdefault("no_proxy", "*")

    # Defensive rebind in case mutmut internals import it differently.
    mutmut_main.setproctitle = _setproctitle_stub.setproctitle
    mutmut_main._run(args.mutant_names, args.max_children)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
