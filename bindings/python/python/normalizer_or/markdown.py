"""Small, dependency-free Markdown preprocessing for speech normalization.

This handles common headings, lists, emphasis, inline code, and links. Paired
LaTeX math spans are copied byte-for-byte. Math-looking bold spans are wrapped
as inline math before emphasis is removed, preventing partial number expansion.
It is not a complete CommonMark renderer.
"""

from __future__ import annotations

import re

_MATH = re.compile(
    r"(?<!\\)(\$\$.*?\$\$|\\\[.*?\\\]|\\\(.*?\\\)|\$(?!\$)[^$\n]*?(?<!\\)\$)",
    re.DOTALL,
)
_HEADING = re.compile(r"(?m)^ {0,3}#{1,6}[ \t]+")
_HORIZONTAL_RULE = re.compile(r"(?m)^ {0,3}(?:-{3,}|_{3,}|\*{3,})[ \t]*$")
_BLOCK_PREFIX = re.compile(r"(?m)^[ \t]*(?:>[ \t]?|[-+*][ \t]+|\d+[.)][ \t]+)")
_INLINE_CODE = re.compile(r"(?<!`)`([^`\n]+)`(?!`)")
_LINK = re.compile(r"!?\[([^\]\n]*)\]\([^\)\n]*\)")
_STRONG = re.compile(r"(?<!\\)(\*\*|__)(?=\S)(.*?)(?<=\S)\1")
_EMPHASIS = re.compile(
    r"(?<![\\*])\*([^*\n]+)\*(?!\*)|(?<![\w\\_])_([^_\n]+)_(?![\w_])"
)
_MATH_TEXT = re.compile(r"(?:[A-Za-z]\s*(?:\^|[²³])|\d\s*[A-Za-z]|[=+*/−-])")


def _strong_text(match: re.Match[str]) -> str:
    content = match.group(2)
    # Preserve bold algebra as an atomic inline-math span before formatting is
    # removed; otherwise a numeric term inside it could be spoken by itself.
    return f"${content}$" if _MATH_TEXT.search(content) else content


def _strip_plain_markdown(text: str) -> str:
    text = _HEADING.sub("", text)
    text = _BLOCK_PREFIX.sub("", text)
    text = _HORIZONTAL_RULE.sub("", text)
    text = _INLINE_CODE.sub(r"\1", text)
    text = _LINK.sub(r"\1", text)
    text = _STRONG.sub(_strong_text, text)
    return _EMPHASIS.sub(lambda match: match.group(1) or match.group(2), text)


def strip_markdown(text: str) -> str:
    """Remove common Markdown markers while preserving text and LaTeX spans.

    Paired formula delimiters and their contents remain unchanged. Bold spans
    that look like algebra receive inline dollar delimiters so normalization
    can speak supported formulas without splitting their terms.
    """
    if not isinstance(text, str):
        raise TypeError("text must be str")
    pieces: list[str] = []
    cursor = 0
    for match in _MATH.finditer(text):
        pieces.append(_strip_plain_markdown(text[cursor : match.start()]))
        pieces.append(match.group(0))
        cursor = match.end()
    pieces.append(_strip_plain_markdown(text[cursor:]))
    return "".join(pieces)
