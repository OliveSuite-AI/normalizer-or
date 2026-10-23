"""Run the documented scripts as consumers of the installed native wheel."""

import json
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


def test_colab_notebook_examples():
    """Execute example cells locally; network/toolchain setup is syntax-checked."""
    root = Path(__file__).resolve().parents[3]
    notebook = json.loads(
        (root / "bindings/python/examples/normalizer_or_colab.ipynb").read_text(
            encoding="utf-8"
        )
    )
    assert notebook["nbformat"] == 4
    namespace = {"repo_dir": root}
    executed = 0
    setup_count = 0
    for cell in notebook["cells"]:
        if cell["cell_type"] != "code":
            continue
        assert cell["execution_count"] is None and cell["outputs"] == []
        compiled = compile("".join(cell["source"]), cell["id"], "exec")
        if "setup" in cell["metadata"].get("tags", []):
            setup_count += 1
            continue
        exec(compiled, namespace)
        executed += 1
    assert setup_count == 1 and executed == 8
