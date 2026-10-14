import { expect } from '@playwright/test'
import { test } from './fixtures'

test('dialogue rehearsal waits for the script exposure journal acknowledgement', async ({
  page,
  request,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  const dialogue = (await (await request.get('/api/v1/dialogues?language=en')).json()).items[0]
  const session = await (
    await request.post('/api/v1/sessions', {
      data: { id: crypto.randomUUID(), language: 'en', dialogue_id: dialogue.id },
    })
  ).json()
  await page.addInitScript(() => {
    const put = IDBObjectStore.prototype.put
    let held = false
    IDBObjectStore.prototype.put = function (value, key) {
      if (!held && value?.event?.kind === 'dialogue_exposure') {
        held = true
        let released = false
        this.transaction.addEventListener('complete', (event) => {
          if (released) return
          event.stopImmediatePropagation()
          const transaction = this.transaction
          Object.assign(window, {
            __releaseDialogueExposure: () => {
              released = true
              transaction.dispatchEvent(new Event('complete'))
            },
          })
        })
      }
      return put.call(this, value, key)
    }
  })
  await page.goto(`/learn/en/practice/${session.id}`)
  await expect.poll(() => page.evaluate(() => '__releaseDialogueExposure' in window)).toBe(true)
  try {
    await expect(
      page.getByRole('button', { name: 'Rehearse dialogue', exact: true }),
    ).toBeDisabled()
    expect((await (await request.get(`/api/v1/sessions/${session.id}`)).json()).last_sequence).toBe(
      0,
    )
  } finally {
    await page.evaluate(() =>
      (window as unknown as { __releaseDialogueExposure: () => void }).__releaseDialogueExposure(),
    )
  }
  await page.getByRole('button', { name: 'Rehearse dialogue', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.getByRole('button', { name: 'Isolate this turn' }).click()
  await page.getByRole('button', { name: 'Recalled' }).click()
  await expect(page.getByRole('heading', { name: 'Session complete' })).toBeVisible()
  const completed = await (await request.get(`/api/v1/sessions/${session.id}`)).json()
  expect(completed.attempts[0].original.supported).toBe(true)
  expect(completed.manifest.dialogue.content.turns).toHaveLength(2)
})
