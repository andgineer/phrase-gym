import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { expect, test } from 'vitest'

const root = join(import.meta.dirname, 'browser')

test('browser tests cannot bypass the silent context factory', () => {
  const direct = readdirSync(root)
    .filter((name) => name.endsWith('.ts') && name !== 'silent-playback.ts')
    .filter((name) => /\.newContext\s*\(/.test(readFileSync(join(root, name), 'utf8')))
  expect(direct).toEqual([])
})

test('persistent context adapters install silence before exposing the context', () => {
  const adapters = readdirSync(root)
    .filter((name) => name.endsWith('.ts'))
    .filter((name) => /\.launchPersistentContext\s*\(/.test(readFileSync(join(root, name), 'utf8')))
  for (const name of adapters) {
    const source = readFileSync(join(root, name), 'utf8')
    expect(source.indexOf('await silencePlayback(context)')).toBeGreaterThan(-1)
    expect(source.indexOf('await silencePlayback(context)')).toBeLessThan(
      source.indexOf('await use(context)'),
    )
  }
})

test('browser specs inherit the silent test fixtures', () => {
  const bypasses = readdirSync(root)
    .filter((name) => name.endsWith('.spec.ts'))
    .filter((name) =>
      /import\s*\{[^}]*\btest\b[^}]*\}\s*from\s*['"]@playwright\/test['"]/.test(
        readFileSync(join(root, name), 'utf8'),
      ),
    )
  expect(bypasses).toEqual([])
})
