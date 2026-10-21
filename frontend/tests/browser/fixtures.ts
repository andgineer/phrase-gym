import { test as base } from '@playwright/test'
import { seedLibrary } from './seed-library'
import { silencePlayback } from './silent-playback'

export const test = base.extend<{ isolatedLibrary: void; silentPlayback: void }>({
  // Page-level fault routes cannot intercept requests owned by a service worker.
  serviceWorkers: 'block',
  silentPlayback: [
    async ({ context }, use) => {
      await silencePlayback(context)
      await use()
    },
    { auto: true },
  ],
  isolatedLibrary: [
    async ({ request }, use, testInfo) => {
      const response = await request.post('/_test/reset')
      if (!response.ok()) throw new Error(`Fixture reset failed (${response.status()})`)
      if (!testInfo.project.name.endsWith('-python-preparation')) await seedLibrary(request)
      await use()
    },
    { auto: true },
  ],
})
