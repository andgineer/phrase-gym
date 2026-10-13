import type { components } from './api.generated'
import { api } from './api'
import { journalEntries, journalPut } from './practiceJournal'
import type { PracticeSession } from './practiceTypes'
import { verifyOfflineShell } from './pwa'

export const mediaCacheName = 'phrase-gym.selected-media.v1'
export type SessionPack = {
  version: 3
  session: PracticeSession
  media: components['schemas']['MediaAsset'][]
}
export async function checksum(bytes: ArrayBuffer) {
  return Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', bytes)), (b) =>
    b.toString(16).padStart(2, '0'),
  ).join('')
}
export async function verifyPack(id: string): Promise<SessionPack | null> {
  const pack = (await journalEntries<SessionPack>(`pack:${id}`)).find(
    (e) => e.key === `pack:${id}`,
  )?.value
  if (!pack) return null
  if (
    pack.version !== 3 ||
    pack.session.id !== id ||
    pack.session.manifest.schema_version !== 1 ||
    pack.session.manifest.policy_version !== 'fsrs-6.3.2-retention90-v1' ||
    !['due-new-v1', 'due-new-v2'].includes(pack.session.manifest.planner_version)
  )
    throw new Error(
      'This download needs a compatible app and a fresh download while connected. Pending actions and audio remain retained.',
    )
  const cache = await caches.open(mediaCacheName)
  for (const asset of pack.media) {
    const response = await cache.match(`/api/v1/media/${asset.id}`)
    const bytes = await response?.arrayBuffer()
    if (
      !bytes ||
      bytes.byteLength !== asset.byte_length ||
      (await checksum(bytes)) !== asset.sha256
    )
      throw new Error(
        'Downloaded audio is unavailable or evicted. Reconnect and download again. Server receipts can restore acknowledged progress; unacknowledged device data cannot be restored from the server.',
      )
  }
  if (pack.session.manifest.required_media_ids.some((id) => !pack.media.some((m) => m.id === id)))
    throw new Error('Download is missing required audio')
  return pack
}
export async function downloadPack(
  session: PracticeSession,
  progress: (done: number, total: number) => void,
) {
  await verifyOfflineShell(true)
  await journalPut('preferences', await api<components['schemas']['Settings']>('/settings'))
  const cache = await caches.open(mediaCacheName)
  const media: SessionPack['media'] = []
  progress(0, session.manifest.required_media_ids.length)
  for (const id of session.manifest.required_media_ids) {
    const asset = await api<components['schemas']['MediaAsset']>(`/media/${id}/metadata`)
    const response = await fetch(`/api/v1/media/${id}`, { cache: 'no-store' })
    if (!response.ok) throw new Error(`Missing audio ${id}: ${response.status}`)
    const bytes = await response.clone().arrayBuffer()
    if (
      !asset.available ||
      bytes.byteLength !== asset.byte_length ||
      (await checksum(bytes)) !== asset.sha256
    )
      throw new Error(`Audio verification failed: ${id}`)
    await cache.put(`/api/v1/media/${id}`, response)
    media.push(asset)
    progress(media.length, session.manifest.required_media_ids.length)
  }
  await journalPut(`pack:${session.id}`, {
    version: 3,
    session: JSON.parse(JSON.stringify(session)),
    media,
  } satisfies SessionPack)
  await verifyPack(session.id)
  await navigator.storage?.persist?.().catch(() => false)
}
