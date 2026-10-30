import re

WORD = re.compile(r"\w+")


def search_key(*parts: str | None) -> str:
    words = [word for part in parts if part for word in WORD.findall(part.casefold())]
    return f" {' '.join(words)} "


def query_terms(query: str) -> list[str]:
    return list(dict.fromkeys(WORD.findall(query.casefold())))


def word_prefix_filter(column: str, terms: list[str]) -> tuple[str, list[str]]:
    # search_key pads every word with a leading space, so " term" matches word starts only.
    clause = " AND ".join(f"instr({column}, ?) > 0" for _ in terms) or "1"
    return clause, [f" {term}" for term in terms]
