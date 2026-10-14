import { mkdtemp, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { test as base } from './fixtures'
import { observeNativePlayback } from './native-playback'

export const test = base.extend({
  context: async ({ playwright, browserName, baseURL, launchOptions }, use) => {
    const profile = await mkdtemp(join(tmpdir(), 'phrase-gym-capture-'))
    const context = await playwright[browserName].launchPersistentContext(profile, {
      ...launchOptions,
      headless: true,
      baseURL,
      serviceWorkers: 'block',
    })
    try {
      await observeNativePlayback(context)
      await use(context)
    } finally {
      await context.close()
      await rm(profile, { recursive: true, force: true })
    }
  },
})
