import { expect } from '@playwright/test'
import { test } from './preview-server'
import { englishLesson } from './english-lesson'
import { syntheticCapture, retainedAudio } from './synthetic-capture'
import { expectComparison } from './native-playback'

for (const reveal of ['explain', 'answer', 'reference', 'comparison'] as const) {
  test(`pinned translation follows ${reveal} reveal independently of expanded English help`, async ({
    page,
    request,
  }) => {
    await request.put('/api/v1/settings', {
      data: { native_language: 'ru', ui_language_mode: 'learning' },
    })
    const profile = await (await request.get('/api/v1/profile')).json()
    profile.profile.languages.en.translation_with_answer = true
    const changed = await request.put('/api/v1/profile', {
      data: {
        mutation_id: crypto.randomUUID(),
        expected_revision: profile.revision,
        profile: profile.profile,
        reason: 'Synthetic translation preference test',
      },
    })
    expect(changed.ok(), await changed.text()).toBe(true)
    await englishLesson(request, 'Давайте начнём.')
    if (reveal === 'comparison') await syntheticCapture(page)
    await page.goto('/learn/en')
    await page.getByRole('button', { name: 'Start session', exact: true }).click()
    await expect(page.getByText('Begin a shared activity.', { exact: true })).toBeVisible()
    await expect(
      page.getByText('Begin a shared activity.', { exact: true }).locator('..'),
    ).toHaveAttribute('lang', 'en')
    await expect(page.getByText('Давайте начнём.', { exact: true })).toHaveCount(1)
    await expect(page.getByText('Давайте начнём.', { exact: true })).toBeVisible()
    await page.getByRole('button', { name: 'Next', exact: true }).click()
    await page.getByRole('button', { name: 'Library', exact: true }).click()
    await page.getByRole('button', { name: 'Start session', exact: true }).click()
    await expect(page.getByTestId('practice-phrase')).toHaveCount(0)
    await expect(page.getByText('Begin a shared activity.', { exact: true })).toHaveCount(0)
    await expect(page.getByText('Давайте начнём.', { exact: true })).toHaveCount(0)
    await expect
      .poll(
        async () =>
          (await (await request.get('/api/v1/sessions/current')).json())?.attempts.length ?? 0,
      )
      .toBe(1)
    const before = await (await request.get('/api/v1/sessions/current')).json()
    expect(before.attempts[0].original).toBeNull()
    if (reveal === 'comparison') {
      await page.getByRole('button', { name: 'Record', exact: true }).click()
      await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
      await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
      await expectComparison(page, 1)
    } else {
      await page
        .getByRole('button', {
          name:
            reveal === 'explain'
              ? /Show & explain/
              : reveal === 'answer'
                ? 'Show answer'
                : /Reference$/,
        })
        .click()
    }
    await expect(page.getByTestId('practice-phrase')).toHaveText("Let's get started.")
    await expect(page.locator('.answer-translation')).toHaveText('Давайте начнём.')
    await expect(page.getByText('Давайте начнём.', { exact: true })).toHaveCount(1)
    if (reveal === 'explain') {
      await expect(page.locator('.explanation')).toHaveAttribute('lang', 'en')
      await expect(page.locator('.explanation')).toContainText('Begin a shared activity.')
    } else {
      await expect(page.locator('.explanation')).toHaveCount(0)
    }
    await expect(page.getByRole('button', { name: /Recalled$/ })).toBeEnabled()
    await page.reload()
    await expect(page.locator('.answer-translation')).toHaveText('Давайте начнём.')
    await page.getByRole('button', { name: /Recalled$/ }).click()
    await expect(page.getByRole('heading', { name: 'Session complete', exact: true })).toBeVisible()
    const after = await (await request.get(`/api/v1/sessions/${before.id}`)).json()
    const supported = reveal === 'explain' || reveal === 'reference'
    expect(after.attempts[0].original.supported).toBe(supported)
    const reason = supported ? 'supported_learning_check' : 'eligible'
    expect(after.attempts[0].eligibility_reason).toBe(reason)
    const receipts = await (await request.get(`/api/v1/sessions/${before.id}/events`)).json()
    expect(receipts.at(-1).eligibility_reason).toBe(reason)
  })
}
