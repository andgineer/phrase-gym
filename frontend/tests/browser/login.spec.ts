import { expect, test } from '@playwright/test'

test('learner login establishes a scoped cookie for practice and reference requests', async ({
  browser,
  baseURL,
  request,
}) => {
  expect((await request.post('/_test/reset')).ok()).toBeTruthy()
  const context = await browser.newContext({ baseURL, extraHTTPHeaders: {} })
  try {
    const page = await context.newPage()
    expect((await context.request.get('/api/v1/settings')).status()).toBe(401)
    await page.goto('/login')
    await page.getByLabel('Learner access token').fill('synthetic-learner-00000000000000000000')
    await page.getByRole('button', { name: 'Sign in' }).click()
    await expect(page.getByLabel('Learner access token')).toHaveCount(0)
    expect((await context.request.get('/api/v1/settings')).status()).toBe(200)
    expect((await context.request.post('/api/v1/media', { data: {} })).status()).toBe(403)
    expect((await context.request.put('/api/v1/profile', { data: {} })).status()).toBe(403)
    const cookie = (await context.cookies()).find((value) => value.name === 'phrase_gym_learner')
    expect(cookie).toMatchObject({ httpOnly: true, sameSite: 'Strict' })
    expect(await page.evaluate(() => document.cookie)).not.toContain('phrase_gym_learner')
    await page.reload()
    expect((await context.request.get('/api/v1/profile')).status()).toBe(200)
  } finally {
    await context.close()
  }
})
