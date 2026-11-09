import { installFakeMedia } from './fake-media'
import { expect, test } from './fixtures'
import { card, details, extraPackage, openDetails } from './extra-package'
import { importPackage, middayTomorrow, revealAnswer } from './helpers'

test.beforeEach(async ({ page }) => installFakeMedia(page))

test('rating buttons show each return and hide it when the grade cannot reschedule', async ({
  page,
}) => {
  await page.goto('/')
  await importPackage(page, extraPackage, 3)
  await page.request.post(`/__test/clock/${middayTomorrow()}`)
  await page.getByRole('button', { name: 'Daily', exact: true }).click()
  const returns = card(page).locator('.rating-return')
  await expect(card(page).getByRole('button', { name: 'Show', exact: true })).toBeVisible()
  await expect(returns).toHaveCount(0)
  await revealAnswer(page)
  await expect(returns).toHaveText(['1d', '1d', '1d', '3d'])
  const easy = card(page).getByRole('button', { name: 'Easy', exact: true })
  await expect(easy).toHaveAttribute('title', 'Back in 3 days')
  const good = card(page).getByRole('button', { name: 'Good', exact: true })
  await expect(good).toHaveAttribute('title', 'Back tomorrow')
  const text = await page.getByTestId('practice-phrase').innerText()
  await good.click()
  await expect(page.getByTestId('practice-phrase')).toHaveCount(0)

  await openDetails(page, text)
  await details(page).getByRole('button', { name: 'Practice', exact: true }).click()
  await revealAnswer(page)
  await expect(good).toBeVisible()
  await expect(good).not.toHaveAttribute('title')
  await expect(returns).toHaveCount(0)
})
