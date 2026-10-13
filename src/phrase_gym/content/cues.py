from phrase_gym.contracts.content import Cue, TargetInput


def has_visible_hint(content: TargetInput, cue: Cue) -> bool:
    return any(
        chunk.id in cue.visible_chunks and any(character.isalnum() for character in chunk.text)
        for chunk in content.chunks
    )
