import pytest

from tests.integration.helpers import phrases, publish


@pytest.mark.parametrize("usage", [None, "Use to ask about the day so far."])
def test_native_meaning_accepts_translation_only_or_useful_explanation(client, package, usage):
    native = {"meaning": "Как проходит твой день?"}
    if usage is not None:
        native["usage"] = usage
    package["targets"][0]["explanations"]["ru"] = native
    publish(client, package)
    phrase = next(item for item in phrases(client) if item["content"]["source_id"] == "day")
    explanation = phrase["content"]["explanations"]["ru"]
    assert explanation["meaning"] == native["meaning"]
    assert explanation["usage"] == usage
    assert explanation["breakdown"] == []
    assert phrase["content"]["explanations"]["en"]["usage"]
