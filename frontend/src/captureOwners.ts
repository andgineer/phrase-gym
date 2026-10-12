export const captureOwner = crypto.randomUUID()
const channel =
  typeof BroadcastChannel === 'undefined' ? null : new BroadcastChannel('phrase-gym.capture-owners')
channel?.addEventListener('message', (event) => {
  if (event.data?.query) channel.postMessage({ reply: event.data.query, owner: captureOwner })
})
window.addEventListener('pagehide', () => channel?.close(), { once: true })

export async function liveCaptureOwners(): Promise<Set<string>> {
  const owners = new Set([captureOwner])
  if (!channel) return owners
  const query = crypto.randomUUID()
  function receive(event: MessageEvent) {
    if (event.data?.reply === query && typeof event.data.owner === 'string')
      owners.add(event.data.owner)
  }
  channel.addEventListener('message', receive)
  channel.postMessage({ query })
  await new Promise((resolve) => window.setTimeout(resolve, 200))
  channel.removeEventListener('message', receive)
  return owners
}
