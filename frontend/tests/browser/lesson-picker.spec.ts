import { expect, test } from './fixtures'
import { extraPackage } from './extra-package'
import { publishPackage } from './helpers'

test('lesson filtering supports Escape, Enter and arrow navigation without losing keyboard focus', async ({
  page,
}) => {
  await publishPackage(page, extraPackage, 3)
  await page.goto('/')
  const lesson = page.getByRole('combobox', { name: 'Lesson', exact: true })
  const options = page.getByRole('option')
  await lesson.click()
  await lesson.pressSequentially('Small')
  await expect(options).toHaveText(['All lessons', 'Small talk · Extra course'])
  await expect(lesson).toHaveAttribute('aria-activedescendant', /.+/)
  await lesson.press('Escape')
  await expect(options).toHaveCount(0)
  await expect(lesson).toHaveValue('')
  await lesson.pressSequentially('Sm')
  await expect(options).toHaveText(['All lessons', 'Small talk · Extra course'])
  await expect(lesson).toHaveAttribute('aria-activedescendant', /.+/)
  await lesson.press('Enter')
  await expect(lesson).toHaveValue('Small talk · Extra course')
  await expect(lesson).toBeFocused()
  await lesson.press('ArrowDown')
  await lesson.press('ArrowUp')
  await lesson.press('Enter')
  await expect(lesson).toHaveValue('')
  await expect(page.getByTestId('lesson-counts')).toHaveText('0 due · 3 new')
})
