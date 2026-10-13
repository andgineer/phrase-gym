export const captureOwner = crypto.randomUUID()
const held = new Map<string, () => void>()
const lockName = (id: string) => `phrase-gym.take:${id}`

export async function holdCapture(id: string): Promise<void> {
  if (!navigator.locks)
    throw new Error('Recording is unavailable in this browser. You can still use self-check.')
  await new Promise<void>((resolve, reject) => {
    void navigator.locks
      .request(lockName(id), async () => {
        const released = new Promise<void>((finish) => held.set(id, finish))
        resolve()
        await released
      })
      .catch(reject)
  })
}
export function releaseCapture(id: string) {
  held.get(id)?.()
  held.delete(id)
}
export async function withAvailableTake(
  id: string,
  operation: () => Promise<void>,
): Promise<boolean> {
  if (!navigator.locks)
    throw new Error(
      'Recording recovery requires a compatible browser; audio remains on this device',
    )
  return navigator.locks.request(lockName(id), { ifAvailable: true }, async (lock) => {
    if (!lock) return false
    await operation()
    return true
  })
}
