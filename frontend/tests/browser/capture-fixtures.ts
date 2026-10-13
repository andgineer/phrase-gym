import { mkdtemp, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { test as base } from './fixtures'

export const test = base.extend({
  context: async ({ playwright, browserName, baseURL }, use) => {
    const profile = await mkdtemp(join(tmpdir(), 'phrase-gym-capture-'))
    const context = await playwright[browserName].launchPersistentContext(profile, {
      headless: true,
      baseURL,
      serviceWorkers: 'block',
    })
    try {
      await use(context)
    } finally {
      await context.close()
      await rm(profile, { recursive: true, force: true })
    }
  },
})
