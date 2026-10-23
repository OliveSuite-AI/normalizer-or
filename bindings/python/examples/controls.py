"""Input errors, option errors, cancellation, deadlines, and resource limits."""

from normalizer_or import CancellationToken, NormalizationError, Normalizer


def expect_error(call, code, limit_kind=None):
    try:
        call()
    except NormalizationError as exc:
        assert (exc.code, exc.limit_kind) == (code, limit_kind)
        print(f"Handled: {exc.code}, limit={exc.limit_kind}")
    else:
        raise AssertionError(f"expected {code}")


def main():
    n = Normalizer()
    for operation in [n.normalize, n.clean]:
        for source in ["", " \t", "a\0", "a\u202e", "\ud800"]:
            expect_error(lambda: operation(source), "invalid_input")
        expect_error(lambda: operation("a" * 32769), "limit_exceeded", "input")
        token = CancellationToken()
        token.cancel()  # Permanently cancelled; use a new token for later work.
        expect_error(lambda: operation("୫", cancellation=token), "cancelled")
        assert operation(
            "୫", cancellation=CancellationToken(), deadline_ms=60000
        ).complete
    # Deadlines are cooperative. Do not assume a short task will time out.
    for call, error_type in [
        (lambda: n.normalize("5", deadline_ms=0), ValueError),
        (lambda: n.normalize("5", deadline_ms=True), TypeError),
        (lambda: n.normalize(5), TypeError),
        (lambda: n.normalize("5", ambiguity_policy="guess"), ValueError),
        (lambda: n.clean("5", digits="latin"), ValueError),
        (lambda: n.clean("5", whitespace="trim"), ValueError),
    ]:
        try:
            call()
        except error_type as exc:
            print(f"Handled {error_type.__name__}: {exc}")
        else:
            raise AssertionError(f"expected {error_type.__name__}")
    print("PASS: errors, cooperative controls, and input limits")


if __name__ == "__main__":
    main()
