import { expect } from '@playwright/test'
import { test } from './fixtures'

for (const [language, settings, native] of [
  ['en', 'Settings', 'Your native language'],
  ['ru', 'Настройки', 'Ваш родной язык'],
  ['sr', 'Podešavanja', 'Tvoj maternji jezik'],
  ['de', 'Einstellungen', 'Deine Muttersprache'],
]) {
  test(`native-language settings control has its accessible name in ${language}`, async ({
    page,
    request,
  }) => {
    await request.put('/api/v1/settings', {
      data: { native_language: language, ui_language_mode: 'native', last_learning_language: 'en' },
    })
    await page.goto('/learn/en')
    await page.getByRole('button', { name: settings, exact: true }).click()
    await expect(page.getByRole('combobox', { name: native, exact: true })).toHaveValue(language)
  })
}
