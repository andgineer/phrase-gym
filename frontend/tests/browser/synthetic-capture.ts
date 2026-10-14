import type { Page } from '@playwright/test'

export async function syntheticCapture(page: Page, holdPermission = false) {
  await page.addInitScript((holdPermission) => {
    const context = new AudioContext()
    const oscillator = context.createOscillator()
    oscillator.frequency.value = 440
    const gain = context.createGain()
    let destination = context.createMediaStreamDestination()
    oscillator.connect(gain)
    gain.connect(destination)
    oscillator.start()
    Object.assign(window, { __captureGain: gain })
    document.addEventListener(
      'click',
      () => {
        void context.resume()
      },
      { capture: true },
    )
    let requests = 0
    Object.defineProperty(navigator, 'mediaDevices', {
      configurable: true,
      value: {
        getUserMedia: async () => {
          requests++
          if (holdPermission)
            await new Promise<void>((resolve) => Object.assign(window, { __allowCapture: resolve }))
          if (destination.stream.getAudioTracks()[0]?.readyState === 'ended') {
            destination = context.createMediaStreamDestination()
            gain.connect(destination)
          }
          return destination.stream
        },
      },
    })
    Object.defineProperty(window, '__captureTrackState', {
      get: () => destination.stream.getAudioTracks()[0]?.readyState,
    })
    Object.defineProperty(window, '__captureRequests', { get: () => requests })
  }, holdPermission)
}
export async function retainedAudio(page: Page): Promise<number> {
  return page.evaluate(async () => {
    return new Promise<number>((resolve, reject) => {
      const opening = indexedDB.open('phrase-gym.practice.v1', 1)
      opening.onsuccess = () => {
        const request = opening.result.transaction('journal').objectStore('journal').getAll()
        request.onsuccess = () => resolve(request.result.filter((v) => v.audio?.byteLength).length)
        request.onerror = () => reject(request.error)
      }
    })
  })
}

export async function recordingPeak(page: Page, id: string): Promise<number> {
  return page.evaluate(async (id) => {
    const context = new OfflineAudioContext(1, 1, 48000)
    const response = await fetch(`/api/v1/recordings/${id}`)
    const decoded = await context.decodeAudioData(await response.arrayBuffer())
    let peak = 0
    for (let channel = 0; channel < decoded.numberOfChannels; channel++)
      for (const sample of decoded.getChannelData(channel)) peak = Math.max(peak, Math.abs(sample))
    return peak
  }, id)
}
