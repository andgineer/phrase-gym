import json


def canonical_json(payload: str) -> str:
    return json.dumps(
        json.loads(payload),
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
    )
