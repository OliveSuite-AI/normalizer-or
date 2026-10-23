"""Independent corpus coverage and concurrent native facade behavior."""

import itertools
from concurrent.futures import ThreadPoolExecutor
from threading import Barrier

import pytest
from normalizer_or import CancellationToken, NormalizationError, Normalizer
from scripts.robustness_cases import (
    DIMENSIONS,
    NUMBERS,
    generated_cases,
    pairs,
    pairwise,
)


def test_pairwise_coverage_and_replay():
    selected = pairwise(1729)
    assert selected == pairwise(1729)
    expected = set().union(*(pairs(row) for row in itertools.product(*DIMENSIONS)))
    actual = set().union(*(pairs(row) for row in selected))
    assert actual == expected
    assert len(selected) < len(list(itertools.product(*DIMENSIONS)))
    cases = generated_cases(1729)
    assert len(NUMBERS) == 100 and len(set(NUMBERS)) == 100
    assert len({c["id"] for c in cases}) == len(cases)
    assert sum(c["id"].startswith("clock-") for c in cases) == 1440


def test_concurrent_calls_and_cancelled_token_reuse():
    n = Normalizer()
    expected = n.normalize("୨୫ kg")
    barrier = Barrier(9)
    token = CancellationToken()

    def worker(index):
        barrier.wait()
        try:
            result = n.normalize("୨୫ kg", cancellation=token)
            assert result == expected
            return "completed"
        except NormalizationError as exc:
            assert exc.code == "cancelled"
            return "cancelled"

    with ThreadPoolExecutor(max_workers=8) as pool:
        futures = [pool.submit(worker, index) for index in range(8)]
        barrier.wait()
        token.cancel()
        assert {f.result() for f in futures} <= {"completed", "cancelled"}
    for operation in [n.normalize, n.clean]:
        with pytest.raises(NormalizationError) as caught:
            operation("5", cancellation=token)
        assert caught.value.code == "cancelled"
    assert n.normalize("୨୫ kg") == expected


@pytest.mark.parametrize("value", [-1, 0, 60001, 2**64])
@pytest.mark.parametrize("operation", ["normalize", "clean"])
def test_deadline_boundaries(value, operation):
    with pytest.raises(ValueError):
        getattr(Normalizer(), operation)("5", deadline_ms=value)


@pytest.mark.parametrize("value", [1, 60000])
@pytest.mark.parametrize("operation", ["normalize", "clean"])
def test_valid_deadline_endpoints(value, operation):
    try:
        assert getattr(Normalizer(), operation)("5", deadline_ms=value).complete
    except NormalizationError as exc:
        # A valid 1 ms deadline can expire on a busy runner; rejection as an
        # invalid argument is the failure this endpoint test guards against.
        assert exc.code == "cancelled"


@pytest.mark.parametrize(
    "value,exception",
    [
        (-1, ValueError),
        (2**63, ValueError),
        (True, TypeError),
        ("0", TypeError),
        (1.5, TypeError),
    ],
)
def test_hint_coordinate_type_boundaries(value, exception):
    from normalizer_or import Hint

    with pytest.raises(exception):
        Hint(value, 1, "digits")
    with pytest.raises(exception):
        Hint(0, value, "digits")
