"""Test-only canonical facade and shared fixture assertions; never creates goldens."""

from normalizer_or import CancellationToken, Hint, NormalizationError


def issue_record(issue):
    return {
        "range": [issue.start_byte, issue.end_byte],
        "category": issue.category,
        "explanation": issue.explanation,
    }


def run_request(normalizer, request):
    token = None
    if request.get("cancelled"):
        token = CancellationToken()
        token.cancel()
    try:
        if request.get("operation") == "clean":
            result = normalizer.clean(
                request["input"],
                digits=request.get("digits", "preserve"),
                whitespace=request.get("whitespace", "preserve"),
                cancellation=token,
            )
        else:
            result = normalizer.normalize(
                request["input"],
                ambiguity_policy=request.get("policy", "preserve"),
                hints=[
                    Hint(h["start"], h["end"], h["kind"])
                    for h in request.get("hints", [])
                ],
                cancellation=token,
            )
    except NormalizationError as exc:
        return {
            "error": exc.code,
            "limit_kind": exc.limit_kind,
            "issues": [issue_record(i) for i in exc.issues],
        }
    return {
        "normalized_text": result.normalized_text,
        "locale": result.locale,
        "normalizer_id": result.normalizer_id,
        "complete": result.complete,
        "fallback_used": result.fallback_used,
        "segments": [
            {
                "range": [s.start_byte, s.end_byte],
                "kind": s.kind,
                "text": s.text,
                "rule_id": s.rule_id,
            }
            for s in result.segments
        ],
        "issues": [issue_record(i) for i in result.issues],
        "fallbacks": [
            {
                "range": [f.start_byte, f.end_byte],
                "attempted_class": f.attempted_class,
                "reason": f.reason,
                "original_category": f.original_category,
                "strategy": f.strategy,
            }
            for f in result.fallbacks
        ],
    }


def subset(expected, actual, context):
    if isinstance(expected, dict):
        for key, value in expected.items():
            assert key in actual, (context, key, actual)
            subset(value, actual[key], f"{context}.{key}")
    elif isinstance(expected, list):
        assert isinstance(actual, list) and len(expected) == len(actual), (
            context,
            expected,
            actual,
        )
        for index, (left, right) in enumerate(zip(expected, actual)):
            subset(left, right, f"{context}[{index}]")
    else:
        assert expected == actual, (context, expected, actual)


def check_fixture_case(case, policy, actual, preserved):
    context = f"{case['id']}/{policy}"
    if "expect" in case:
        subset(case["expect"][policy], actual, context)
    elif not case.get("complete", True) and policy == "reject":
        subset({"error": "unresolved", "issues": preserved["issues"]}, actual, context)
    elif case.get("complete", True):
        subset(
            {
                "normalized_text": case["output"],
                "complete": True,
                "issues": [],
                "fallbacks": [],
            },
            actual,
            context,
        )
    else:
        spans = case.get("issue_spans", [[0, len(case["input"].encode("utf-8"))]])
        field = "issues" if policy == "preserve" else "fallbacks"
        assert [r["range"] for r in actual[field]] == spans, context
        if policy == "preserve":
            subset(
                {
                    "normalized_text": case.get("output", case["input"]),
                    "complete": False,
                },
                actual,
                context,
            )
        else:
            subset(
                {"complete": True, "issues": [], "fallback_used": True}, actual, context
            )
    if policy == "reject" and actual.get("error") == "unresolved":
        assert actual["issues"] == preserved["issues"], context
