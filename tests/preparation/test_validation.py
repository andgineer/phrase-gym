import pytest

from phrase_gym.errors import DomainError
from phrase_gym.preparation.validation import assemble


def test_rotation_masks_translation_only_and_no_reliable_cue(source, draft):
    target = assemble(source, draft).targets[0]
    assert len(target.cues) == 2
    assert target.explanations["ru"].usage is None
    draft.phrases[0].candidates = []
    with pytest.raises(DomainError, match="No-cue"):
        assemble(source, draft)
    draft.phrases[0].no_reliable_cue_reason = "Insufficient identifying material"
    assert assemble(source, draft).targets[0].cues == []


def test_target_leaks_and_unreachable_masks_rejected(source, draft):
    source.phrases[0].kind = "word_sense"
    draft.phrases[0].required_chunks = ["a"]
    with pytest.raises((DomainError, ValueError)):
        assemble(source, draft)
    source.phrases[0].kind = "fixed_phrase"
    draft.phrases[0].required_chunks = ["a", "b", "c"]
    draft.phrases[0].candidates[1].visible_chunks = ["a"]
    with pytest.raises(DomainError, match="comparable"):
        assemble(source, draft)
