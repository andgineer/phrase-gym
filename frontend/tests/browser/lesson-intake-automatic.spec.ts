import { expect } from '@playwright/test'
import { test } from './preview-server'

for (const clipboard of ['text', 'empty', 'denied', 'late'] as const) {
  test(`mobile + previews ${clipboard} clipboard without sending, and respects edits/close`, async ({
    page,
    request,
  }) => {
    await request.put('/api/v1/settings', {
      data: { native_language: 'en', ui_language_mode: 'learning' },
    })
    await page.setViewportSize({ width: 390, height: 844 })
    await page.addInitScript((mode) => {
      Object.defineProperty(navigator, 'clipboard', {
        value: {
          readText() {
            if (mode === 'denied') return Promise.reject(new Error('Synthetic denied clipboard'))
            if (mode === 'late')
              return new Promise<string>((resolve) => {
                ;(
                  window as unknown as { clipboardRelease: (text: string) => void }
                ).clipboardRelease = resolve
              })
            return Promise.resolve(mode === 'text' ? "Let's get started." : '')
          },
        },
      })
    }, clipboard)
    const deliveries: unknown[] = []
    let uploads = 0
    page.on('request', (r) => {
      if (r.method() === 'POST' && r.url().endsWith('/api/v1/inbox/lessons'))
        deliveries.push(r.postDataJSON())
      if (r.method() === 'POST' && r.url().endsWith('/api/v1/media')) uploads++
    })
    await page.goto('/learn/en')
    await page.getByRole('button', { name: 'Add phrase', exact: true }).click()
    const phrase = page.getByLabel('Exact English phrase', { exact: true })
    await expect(phrase).toHaveValue(clipboard === 'text' ? "Let's get started." : '')
    expect(deliveries).toHaveLength(0)
    await phrase.fill('Could you repeat that, please?')
    if (clipboard === 'late')
      await page.evaluate(() =>
        (window as unknown as { clipboardRelease: (text: string) => void }).clipboardRelease(
          'Late clipboard must not replace edits',
        ),
      )
    await expect(phrase).toHaveValue('Could you repeat that, please?')
    await page.getByRole('button', { name: 'Close', exact: true }).click()
    expect(deliveries).toHaveLength(0)
    await page.getByRole('button', { name: 'Add phrase', exact: true }).click()
    // A new gesture may preview new clipboard text; review is always editable.
    await phrase.fill('Could you repeat that, please?')
    await page.getByLabel('Optional note', { exact: true }).fill('Ask someone to say it again.')
    await page.getByRole('button', { name: 'Send', exact: true }).dblclick()
    await expect(page.getByText(/Accepted — waiting for Coach/)).toBeVisible()
    expect(deliveries).toHaveLength(1)
    expect(uploads).toBe(0)
    expect(deliveries[0]).toMatchObject({
      note: 'Ask someone to say it again.',
      phrases: [
        { text: 'Could you repeat that, please?', reference_audio_id: null, audio_sha256: null },
      ],
    })
  })
}

test('closing while clipboard is pending never reopens or changes retained preview', async ({
  page,
  request,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'en', ui_language_mode: 'learning' },
  })
  await page.addInitScript(() =>
    Object.defineProperty(navigator, 'clipboard', {
      value: {
        readText: () =>
          new Promise<string>((resolve) => {
            ;(window as unknown as { clipboardRelease: (text: string) => void }).clipboardRelease =
              resolve
          }),
      },
    }),
  )
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Add phrase', exact: true }).click()
  await page.getByRole('button', { name: 'Close', exact: true }).click()
  await page.evaluate(() =>
    (window as unknown as { clipboardRelease: (text: string) => void }).clipboardRelease(
      'Late clipboard',
    ),
  )
  await expect(page.getByRole('dialog')).toHaveCount(0)
})
