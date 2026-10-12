/** Only small reference events fit here; binary takes require IndexedDB. */
import { api } from './api'
import type { ExposureInput } from './api'

const prefix = 'phrase-gym.reference-exposure.v1.'
export function retainExposure(event: ExposureInput) {
  // A separate key per immutable event avoids read/modify/write races between tabs.
  localStorage.setItem(prefix + event.id, JSON.stringify(event))
}
export async function sendExposure(event: ExposureInput): Promise<boolean> {
  try {
    await api('/exposures', { method: 'POST', body: JSON.stringify(event) })
    localStorage.removeItem(prefix + event.id)
    return true
  } catch {
    return false
  }
}
export async function flushExposures() {
  const keys = Object.keys(localStorage).filter((key) => key.startsWith(prefix))
  for (const key of keys) {
    const raw = localStorage.getItem(key)
    if (raw) await sendExposure(JSON.parse(raw) as ExposureInput)
  }
}
