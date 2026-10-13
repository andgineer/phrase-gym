import type { APIRequestContext, Page } from '@playwright/test'
import type { StoredTake } from '../../src/practiceTypes'

export async function takeFixture(request: APIRequestContext, sessionId: string) {
  const state = await (await request.get(`/api/v1/sessions/${sessionId}`)).json()
  const captureId = crypto.randomUUID()
  const receipt = await (
    await request.post(`/api/v1/sessions/${sessionId}/events`, {
      data: {
        events: [
          {
            id: captureId,
            sequence: state.last_sequence + 1,
            step: state.cursor,
            occurred_at: new Date().toISOString(),
            kind: 'respond',
            modality: 'recorded',
          },
        ],
      },
    })
  ).json()
  const target = state.manifest.targets[state.queue[state.cursor].target_index].phrase
  const bytes = await (
    await request.get(`/api/v1/media/${target.content.reference_audio_id}`)
  ).body()
  const take: Omit<StoredTake, 'audio'> = {
    id: crypto.randomUUID(),
    target_id: target.id,
    revision: target.revision,
    attempt_id: receipt.receipts[0].attempt_id,
    capture_event_id: captureId,
    captured_at: new Date().toISOString(),
    mime_type: 'audio/wav',
    duration_ms: 1000,
    complete: false,
    capture_settings: {},
    owner: crypto.randomUUID(),
    capture_lease: 1,
    status: 'capturing',
  }
  return { take, bytes: Array.from(bytes) }
}
export async function writeTake(page: Page, take: Omit<StoredTake, 'audio'>, bytes: number[]) {
  await page.evaluate(
    ({ take, bytes }) =>
      new Promise<void>((resolve, reject) => {
        const opening = indexedDB.open('phrase-gym.practice.v1', 1)
        opening.onsuccess = () => {
          const transaction = opening.result.transaction('journal', 'readwrite')
          transaction
            .objectStore('journal')
            .put(
              { ...take, audio: Uint8Array.from(bytes).buffer },
              `${take.capture_lease === 1 ? 'take-v2:' : 'take:'}${take.id}`,
            )
          transaction.oncomplete = () => resolve()
          transaction.onerror = () => reject(transaction.error)
        }
      }),
    { take, bytes },
  )
}
export async function readTake(page: Page, id: string) {
  return page.evaluate(
    (id) =>
      new Promise<{ complete: boolean; status: string; size: number } | null>((resolve) => {
        const opening = indexedDB.open('phrase-gym.practice.v1', 1)
        opening.onsuccess = () => {
          const store = opening.result.transaction('journal').objectStore('journal')
          const read = store.get(`take-v2:${id}`)
          function finish(
            value: { complete: boolean; status: string; audio: ArrayBuffer } | undefined,
          ) {
            resolve(
              value
                ? { complete: value.complete, status: value.status, size: value.audio.byteLength }
                : null,
            )
          }
          read.onsuccess = () => {
            if (read.result) finish(read.result)
            else {
              const legacy = store.get(`take:${id}`)
              legacy.onsuccess = () => finish(legacy.result)
            }
          }
        }
      }),
    id,
  )
}
