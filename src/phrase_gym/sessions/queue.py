from phrase_gym.contracts.sessions import SessionManifest, SessionStep
from phrase_gym.errors import DomainError


def initial_queue(manifest: SessionManifest) -> list[SessionStep]:
    if not manifest.dialogue:
        return [SessionStep(target_index=i) for i in range(len(manifest.targets))]
    bindings = {target.phrase.id: i for i, target in enumerate(manifest.targets)}
    queue = []
    for index, turn in enumerate(manifest.dialogue.content.turns):
        if turn.target_source_id:
            target_id = manifest.dialogue.target_ids[turn.target_source_id]
            if target_id not in bindings:
                raise DomainError("Every dialogue target needs prepared cues and audio", 409)
            queue.append(SessionStep(target_index=bindings[target_id], dialogue_turn=index))
    return queue
