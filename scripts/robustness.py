"""Run independent expectations against Rust JSONL and the installed Python wheel."""

import argparse
import hashlib
import json
import os
import platform
import subprocess
import sys
import time
from pathlib import Path

from normalizer_or import NormalizationError, Normalizer
from .contracts import check_fixture_case, run_request, subset
from .robustness_cases import POLICIES, ROOT, requests


def write_json(path, value):
    path.write_text(
        json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )


def stress_worker():
    n = Normalizer()
    report = []
    patterns = ["wrappers", "markers", "numeric_run", "combining", "electronic"]
    for size in [256, 1024, 4096, 16384, 32768]:
        for pattern in patterns:
            if pattern == "markers":
                source = "% " * ((size - 2) // 2) + "12"
            elif pattern == "wrappers":
                depth = (size - 1) // 2
                source = "(" * depth + "5" + ")" * depth
            else:
                unit = {
                    "markers": "% ",
                    "numeric_run": "1 ",
                    "combining": "5\u0301",
                    "electronic": "a=1,2;",
                }[pattern]
                prefix = "https://example.com/?" if pattern == "electronic" else ""
                source = prefix + unit * (
                    (size - len(prefix.encode("utf-8"))) // len(unit.encode("utf-8"))
                )
            for operation in ["preserve", "fallback", "clean"]:
                start = time.monotonic()
                try:
                    result = (
                        n.clean(source)
                        if operation == "clean"
                        else n.normalize(source, ambiguity_policy=operation)
                    )
                    assert (
                        "".join(s.text for s in result.segments)
                        == result.normalized_text
                    )
                    assert result.segments[-1].end_byte == len(source.encode("utf-8"))
                    if operation == "clean":
                        assert result.normalized_text == source
                    elif (
                        pattern in ("markers", "combining", "electronic")
                        and operation == "preserve"
                    ):
                        assert not result.complete and len(result.issues) == 1
                        assert (
                            result.issues[0].start_byte,
                            result.issues[0].end_byte,
                        ) == (0, len(source.encode("utf-8")))
                    elif pattern == "wrappers":
                        assert result.complete and not result.fallback_used
                    outcome = "ok"
                except NormalizationError as exc:
                    assert exc.code == "limit_exceeded" and exc.limit_kind in (
                        "result",
                        "candidates",
                    ), (pattern, size, exc.code)
                    outcome = exc.limit_kind
                report.append(
                    {
                        "pattern": pattern,
                        "bytes": len(source.encode("utf-8")),
                        "operation": operation,
                        "outcome": outcome,
                        "seconds": time.monotonic() - start,
                    }
                )
    print(json.dumps(report))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--probe", type=Path)
    parser.add_argument("--seed", type=int, default=1729)
    parser.add_argument("--deep", action="store_true")
    parser.add_argument("--stress", action="store_true")
    parser.add_argument("--stress-worker", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument("--output", type=Path, default=ROOT / "target/robustness")
    args = parser.parse_args()
    if args.stress_worker:
        stress_worker()
        return
    args.output.mkdir(parents=True, exist_ok=True)
    metadata = {
        "seed": args.seed,
        "deep": args.deep,
        "python": sys.version,
        "platform": platform.platform(),
        "property_cases": os.environ.get("PROPTEST_CASES", "512"),
        "property_seed": os.environ.get("PROPTEST_RNG_SEED", "1729"),
    }
    metadata["rustc"] = subprocess.run(
        ["rustc", "--version"],
        check=True,
        stdout=subprocess.PIPE,
        text=True,
        encoding="utf-8",
    ).stdout.strip()
    import normalizer_or._native as native

    metadata["wheel_extension_sha256"] = hashlib.sha256(
        Path(native.__file__).read_bytes()
    ).hexdigest()
    write_json(args.output / "metadata.json", metadata)
    shared = json.loads((ROOT / "tests/fixtures/odia.json").read_text(encoding="utf-8"))
    fixtures = {c["id"]: c for c in shared}
    corpus = [
        {
            "id": f"{case['id']}/{policy}",
            "input": case["input"],
            "policy": policy,
            "fixture_id": case["id"],
        }
        for case in shared
        for policy in POLICIES
    ]
    corpus.extend(requests(args.seed, args.deep))
    assert len({r["id"] for r in corpus}) == len(corpus)
    serialized = "".join(json.dumps(r, ensure_ascii=False) + "\n" for r in corpus)
    (args.output / "requests.jsonl").write_text(serialized, encoding="utf-8")
    probe = args.probe or ROOT / "target/debug/examples" / (
        "contract-probe.exe" if os.name == "nt" else "contract-probe"
    )
    metadata["probe_path"] = str(probe.resolve())
    metadata["rust_probe_sha256"] = hashlib.sha256(probe.read_bytes()).hexdigest()
    write_json(args.output / "metadata.json", metadata)
    completed = subprocess.run(
        [str(probe.resolve())],
        input=serialized,
        text=True,
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=120,
        check=False,
    )
    (args.output / "rust-stderr.txt").write_text(completed.stderr, encoding="utf-8")
    (args.output / "rust-results.jsonl").write_text(completed.stdout, encoding="utf-8")
    if completed.returncode:
        print(completed.stderr, file=sys.stderr)
        raise SystemExit(f"Rust probe failed; see {args.output / 'rust-stderr.txt'}")
    results = [json.loads(line) for line in completed.stdout.splitlines()]
    assert len(results) == len(corpus), (len(results), len(corpus))
    normalizer = Normalizer()
    preserved = {}
    failures = []
    for request, exported in zip(corpus, results):
        python = None
        try:
            assert exported["id"] == request["id"]
            rust = exported["result"]
            python = run_request(normalizer, request)
            assert rust == python, (request["id"], "Rust/Python mismatch", rust, python)
            if "fixture_id" in request:
                case = fixtures[request["fixture_id"]]
                if request["policy"] == "preserve":
                    preserved[case["id"]] = python
                check_fixture_case(
                    case, request["policy"], python, preserved[case["id"]]
                )
            else:
                subset(request["expected"], python, request["id"])
            if "error" not in python:
                assert python["normalized_text"] == "".join(
                    s["text"] for s in python["segments"]
                )
                cursor = 0
                for segment in python["segments"]:
                    assert segment["range"][0] == cursor
                    cursor = segment["range"][1]
                assert cursor == len(request["input"].encode("utf-8"))
        except Exception as exc:
            failures.append(
                {
                    "id": request["id"],
                    "request": request,
                    "rust": exported["result"],
                    "python": python,
                    "detail": str(exc),
                }
            )
    write_json(args.output / "failures.json", failures)
    if failures:
        for failure in failures[:10]:
            print(f"FAIL {failure['id']}: {ascii(failure['detail'])}", file=sys.stderr)
        raise SystemExit(
            f"{len(failures)} failures; replay seed {args.seed}; see {args.output / 'failures.json'}"
        )
    if args.stress:
        stress = subprocess.run(
            [sys.executable, "-m", "scripts.robustness", "--stress-worker"],
            text=True,
            encoding="utf-8",
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=30,
            check=True,
        )
        write_json(args.output / "stress.json", json.loads(stress.stdout))
    write_json(
        args.output / "summary.json", dict(metadata, checked=len(corpus), failures=0)
    )
    print(
        f"{len(corpus)} complete Rust/Python records matched independent expectations; seed={args.seed}; deep={args.deep}"
    )


if __name__ == "__main__":
    main()
