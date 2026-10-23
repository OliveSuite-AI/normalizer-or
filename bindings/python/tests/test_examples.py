"""Run the documented scripts as consumers of the installed native wheel."""

import os
from pathlib import Path
import subprocess
import sys


def test_all_examples_from_another_directory(tmp_path):
    runner = Path(__file__).resolve().parents[1] / "examples/run_all.py"
    env = dict(os.environ, PYTHONUTF8="1", PYTHONIOENCODING="utf-8", PYTHONOPTIMIZE="0")
    completed = subprocess.run(
        [sys.executable, str(runner)],
        cwd=tmp_path,
        env=env,
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
        timeout=120,
    )
    assert "PASS: all 7 example scripts" in completed.stdout
