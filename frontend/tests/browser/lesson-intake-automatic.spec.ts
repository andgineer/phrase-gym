import { expect } from '@playwright/test'
import { test } from './preview-server'
import { fillLesson } from './lesson-intake-helpers'
import { tone } from './seed-library'

for (const kind of ['drop', 'paste'] as const) {
  test(`deliberate audio ${kind} transfers the original bytes without Submit`, async ({
    page,
    request,
  }) => {
    await request.put('/api/v1/settings', {
      data: { native_language: 'ru', ui_language_mode: 'learning' },
    })
    let uploads = 0
    page.on('request', (r) => {
      if (r.method() === 'POST' && r.url().endsWith('/api/v1/media')) {
        uploads++
        expect(r.postDataJSON().audio_base64).toBe(tone())
      }
    })
    await page.goto('/learn/en')
    await page.getByRole('button', { name: 'Add a first lesson', exact: true }).click()
    await fillLesson(page, 'Opening a meeting', false)
    await page.locator('form').evaluate(
      (form, { kind, bytes }) => {
        const transfer = new DataTransfer()
        transfer.items.add(
          new File([Uint8Array.from(atob(bytes), (c) => c.charCodeAt(0))], 'reference.wav', {
            type: 'audio/wav',
          }),
        )
        const event =
          kind === 'drop'
            ? new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: transfer })
            : new ClipboardEvent('paste', {
                bubbles: true,
                cancelable: true,
                clipboardData: transfer,
              })
        form.dispatchEvent(event)
      },
      { kind, bytes: tone() },
    )
    await expect(page.getByText(/Accepted — waiting for Coach/)).toBeVisible()
    expect(uploads).toBe(1)
    await expect(page.getByRole('button', { name: 'Submit lesson', exact: true })).toHaveCount(0)
  })
}
