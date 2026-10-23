"""Reuse one normalizer across workers and write a UTF-8 JSONL result file."""

import json
from concurrent.futures import ThreadPoolExecutor
from dataclasses import asdict
from pathlib import Path
from tempfile import TemporaryDirectory

from normalizer_or import NormalizationError, Normalizer


def main():
    n = Normalizer()

    def normalize_record(source):
        try:
            result = n.normalize(source)
            return {
                "input": source,
                "result": asdict(result),
                "fallback_used": result.fallback_used,
            }
        except NormalizationError as exc:
            return {
                "input": source,
                "error": {"code": exc.code, "limit_kind": exc.limit_kind},
            }

    sources = ["୨୫ kg", "₹25.50", "AB12", ""]
    with ThreadPoolExecutor(max_workers=4) as pool:
        records = list(pool.map(normalize_record, sources))
    assert records == [normalize_record(source) for source in sources]
    assert records[0]["result"]["normalized_text"] == "ପଚିଶ କିଲୋଗ୍ରାମ"
    assert records[1]["result"]["normalized_text"] == "ପଚିଶ ଟଙ୍କା ପଚାଶ ପଇସା"
    assert not records[2]["result"]["complete"]
    assert records[3]["error"]["code"] == "invalid_input"
    # Replace the temporary path with your application output path to retain it.
    with TemporaryDirectory(prefix="odia-examples-") as directory:
        path = Path(directory) / "results.jsonl"
        path.write_text(
            "".join(json.dumps(r, ensure_ascii=False) + "\n" for r in records),
            encoding="utf-8",
        )
        loaded = [
            json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()
        ]
        assert len(loaded) == len(sources)
        assert loaded[0]["result"]["normalized_text"] == "ପଚିଶ କିଲୋଗ୍ରାମ"
        assert loaded[-1]["error"]["code"] == "invalid_input"
        print(path.read_text(encoding="utf-8"))
    print("PASS: shared normalizer, ordered batch, row errors, UTF-8 JSONL round trip")


if __name__ == "__main__":
    main()
