import type { BrowserContext, Page } from '@playwright/test'

export async function installEventClock(context: BrowserContext) {
  await context.addInitScript(() => {
    window.Date = new Proxy(Date, {
      construct(target, args) {
        const offset = Number(localStorage.getItem('synthetic-clock-offset') ?? 0)
        return Reflect.construct(target, args.length ? args : [target.now() + offset])
      },
      get(target, key) {
        return key === 'now'
          ? () => target.now() + Number(localStorage.getItem('synthetic-clock-offset') ?? 0)
          : Reflect.get(target, key)
      },
    })
  })
}
export async function setEventClock(page: Page, seconds: number) {
  await page.evaluate(
    (offset) => localStorage.setItem('synthetic-clock-offset', String(offset)),
    seconds * 1000,
  )
}
