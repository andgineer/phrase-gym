import { expect, test as base } from '@playwright/test'
import { createSilentContext } from './silent-playback'

export const test = base.extend({
  context: async ({ browser }, use) => {
    const context = await createSilentContext(browser)
    await use(context)
    await context.close()
  },
})
export { expect }
