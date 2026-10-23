"""Run every standalone example against the installed package; stop on failure."""

import os
from pathlib import Path
import subprocess
import sys


def main():
    if sys.flags.optimize:
        raise SystemExit(
            "Run without -O: these examples use assertions to verify output."
        )
    directory = Path(__file__).resolve().parent
    examples = sorted(
        p for p in directory.glob("*.py") if p.name != Path(__file__).name
    )
    env = dict(os.environ, PYTHONUTF8="1", PYTHONIOENCODING="utf-8", PYTHONOPTIMIZE="0")
    for example in examples:
        print(f"\nRunning {example.name}", flush=True)
        subprocess.run([sys.executable, str(example)], env=env, check=True, timeout=60)
    print(f"\nPASS: all {len(examples)} example scripts")


if __name__ == "__main__":
    main()
