import { test, expect } from './fixtures'
import { installFakeMedia } from './fake-media'
import { extraPackage, texts } from './extra-package'
import { importPackage, playedMedia, revealAnswer } from './helpers'
import { seedTake } from './practice-api'

test.use({ timezoneId: 'UTC', locale: 'en-US', viewport: { width: 390, height: 844 } })

test('listen across days and return to the same reviewed card', async ({ page }, testInfo) => {
  await installFakeMedia(page)
  await page.goto('/')
  await importPackage(page, extraPackage, 3)
  await page.getByRole('button', { name: 'Start', exact: true }).click()
  await revealAnswer(page)
  const current = async () =>
    (await (await page.request.get('/api/practice/current?language=en')).json()).card
  const first = await current()
  const today = new Date().toISOString().slice(0, 10)
  for (let minute = 0; minute < 10; minute++)
    await seedTake(page, first, `${today}T10:0${minute}:00Z`, 6200)
  await seedTake(page, first, '2020-11-12T09:15:00Z', null)
  await page.getByRole('button', { name: 'Good', exact: true }).click()
  const previous = page.getByRole('button', { name: 'Previous card' })
  await expect(previous).toBeEnabled()
  const second = await current()
  await previous.click()
  await expect(page.getByTestId('practice-phrase')).toHaveText(texts[0])
  await page.getByRole('button', { name: 'More explanation' }).click()
  const menu = page.getByRole('button', { name: 'More actions' })
  await menu.click()
  await page.getByRole('button', { name: 'Recording history' }).click()

  const screen = page.getByRole('region', { name: 'Recording history' })
  await expect(screen.getByRole('heading', { name: 'Today' })).toBeVisible()
  await expect(screen.getByRole('listitem')).toHaveCount(10)
  await screen.getByRole('button', { name: 'Show older recordings' }).click()
  await expect(screen.getByRole('listitem')).toHaveCount(11)
  await expect(screen.getByRole('heading', { name: 'November 12, 2020' })).toBeVisible()
  await expect(screen.getByRole('button', { name: 'Show older recordings' })).toHaveCount(0)
  const rows = screen.getByRole('listitem').getByRole('button')
  const latest = rows.first()
  const oldest = rows.last()
  await expect(latest).toHaveAccessibleName(/Play recording at 10:09 AM 0:06/)
  await expect(oldest).toHaveAccessibleName('Play recording at 9:15 AM')
  await latest.click()
  await expect(latest).toHaveAccessibleName(/^Stop recording/)
  await oldest.click()
  await expect(latest).toHaveAccessibleName(/^Play recording/)
  await expect(oldest).toHaveAccessibleName(/^Stop recording/)
  const mediaSources = () =>
    page.evaluate(() =>
      (window as unknown as { __playedMedia: HTMLMediaElement[] }).__playedMedia.map((media) =>
        media.getAttribute('src'),
      ),
    )
  const urls = await playedMedia(page)
  expect(urls).toHaveLength(2)
  expect(urls[0]).not.toBe(urls[1])
  expect(await mediaSources()).toEqual(['', new URL(urls[1]!).pathname])
  await oldest.click()
  await expect(oldest).toHaveAccessibleName(/^Play recording/)
  expect(await mediaSources()).toEqual(['', ''])
  await latest.click()
  await page.screenshot({ path: testInfo.outputPath('recording-history.png'), fullPage: true })
  await screen.getByRole('button', { name: 'Back to practice' }).click()
  expect(await mediaSources()).toEqual(['', '', ''])
  await expect(menu).toBeFocused()
  await expect(page.getByTestId('practice-phrase')).toHaveText(texts[0])
  await expect(page.getByRole('button', { name: 'Good', pressed: true })).toBeDisabled()
  await expect(page.getByRole('button', { name: 'More explanation' })).toHaveAttribute(
    'aria-expanded',
    'true',
  )
  expect(await current()).toEqual(second)
})
