import { createHash } from 'node:crypto'
import { expect } from '@playwright/test'
import { test } from './offline-fixtures'
import { downloadSession } from './offline-actions'
import { readTake, takeFixture, writeTake } from './take-race-helpers'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})
for (const size of [0, 2048]) {
  test(`recovery rereads a finalized take instead of replacing its ${size}-byte snapshot`, async ({
    page,
    request,
  }) => {
    const id = await downloadSession(page)
    const { take, bytes } = await takeFixture(request, id)
    await writeTake(page, take, bytes.slice(0, size))
    await page.evaluate(
      ({ take, bytes }) => {
        const manager = navigator.locks
        Object.defineProperty(navigator, 'locks', { value: manager, configurable: true })
        const native = manager.request.bind(manager)
        let replaced = false
        manager.request = (async (
          name: string,
          options: LockOptions,
          callback: LockGrantedCallback<unknown>,
        ) => {
          if (!replaced && name === `phrase-gym.take:${take.id}` && options?.ifAvailable) {
            replaced = true
            await new Promise<void>((resolve) => {
              const opening = indexedDB.open('phrase-gym.practice.v1', 1)
              opening.onsuccess = () => {
                const transaction = opening.result.transaction('journal', 'readwrite')
                transaction.objectStore('journal').put(
                  {
                    ...take,
                    status: 'pending',
                    complete: true,
                    audio: Uint8Array.from(bytes).buffer,
                  },
                  `take-v2:${take.id}`,
                )
                transaction.oncomplete = () => resolve()
              }
            })
          }
          return native(name, options, callback)
        }) as typeof navigator.locks.request
      },
      { take, bytes },
    )
    await page.bringToFront()
    await page.evaluate(() => window.dispatchEvent(new Event('online')))
    await expect
      .poll(
        async () =>
          (await (await request.get(`/api/v1/phrases/${take.target_id}/recordings`)).json()).length,
      )
      .toBe(1)
    const rows = await (await request.get(`/api/v1/phrases/${take.target_id}/recordings`)).json()
    expect(rows[0]).toMatchObject({
      id: take.id,
      complete: true,
      byte_length: bytes.length,
      sha256: createHash('sha256').update(Buffer.from(bytes)).digest('hex'),
    })
    await expect.poll(() => readTake(page, take.id)).toBeNull()
  })
}
test('an unresponsive live writer lease prevents partial recovery and upload in another tab', async ({
  page,
  context,
  request,
}) => {
  const id = await downloadSession(page)
  const writer = await context.newPage()
  await writer.goto('/learn/en')
  const { take, bytes } = await takeFixture(request, id)
  await writer.evaluate(async (id) => {
    await new Promise<void>((acquired) => {
      const finished = navigator.locks.request(`phrase-gym.take:${id}`, async () => {
        const hold = new Promise<void>((release) =>
          Object.assign(window, { __releaseTake: release }),
        )
        acquired()
        await hold
      })
      Object.assign(window, { __leaseEnded: finished })
    })
  }, take.id)
  await writeTake(page, take, bytes.slice(0, 2048))
  await page.evaluate((id) => {
    const manager = navigator.locks
    Object.defineProperty(navigator, 'locks', { value: manager, configurable: true })
    const native = manager.request.bind(manager)
    Object.assign(window, { __recoveryFinished: false })
    manager.request = (async (
      name: string,
      options: LockOptions,
      callback: LockGrantedCallback<unknown>,
    ) => {
      const result = await native(name, options, callback)
      if (name === `phrase-gym.take:${id}`) Object.assign(window, { __recoveryFinished: true })
      return result
    }) as typeof navigator.locks.request
  }, take.id)
  await page.bringToFront()
  await page.evaluate(() => window.dispatchEvent(new Event('online')))
  await expect
    .poll(() =>
      page.evaluate(
        () => (window as unknown as { __recoveryFinished: boolean }).__recoveryFinished,
      ),
    )
    .toBe(true)
  expect(await readTake(page, take.id)).toEqual({
    complete: false,
    status: 'capturing',
    size: 2048,
  })
  expect(
    await (await request.get(`/api/v1/phrases/${take.target_id}/recordings`)).json(),
  ).toHaveLength(0)
  await writeTake(writer, { ...take, complete: true, status: 'pending' }, bytes)
  await writer.evaluate(async () => {
    const owner = window as unknown as { __releaseTake: () => void; __leaseEnded: Promise<void> }
    owner.__releaseTake()
    await owner.__leaseEnded
  })
  await page.evaluate(() => window.dispatchEvent(new Event('online')))
  await expect
    .poll(
      async () =>
        (await (await request.get(`/api/v1/phrases/${take.target_id}/recordings`)).json()).length,
    )
    .toBe(1)
  const rows = await (await request.get(`/api/v1/phrases/${take.target_id}/recordings`)).json()
  expect(rows[0]).toMatchObject({ complete: true, byte_length: bytes.length })
  await writer.close()
})

test('a lost ownership protocol cannot let acknowledgement deletion erase newer same-size audio', async ({
  page,
  request,
}) => {
  const id = await downloadSession(page)
  const { take, bytes } = await takeFixture(request, id)
  await request.post('/_test/hold-recording-ack')
  await writeTake(page, { ...take, complete: true, status: 'pending' }, bytes)
  await page.evaluate(() => window.dispatchEvent(new Event('online')))
  await expect
    .poll(
      async () =>
        (await (await request.get(`/api/v1/phrases/${take.target_id}/recordings`)).json()).length,
    )
    .toBe(1)
  const newer = bytes.slice()
  newer[newer.length - 4] = newer[newer.length - 4]! ^ 1
  // Simulate an older, non-cooperating writer; normal writers hold the take lease.
  await writeTake(page, { ...take, complete: true, status: 'pending' }, newer)
  await request.post('/_test/release-recording-ack')
  await expect(page.getByRole('alert').filter({ hasText: 'Newer audio remains' })).toBeVisible()
  expect(await readTake(page, take.id)).toEqual({
    complete: true,
    status: 'pending',
    size: bytes.length,
  })
  const retainedHash = await page.evaluate(
    (id) =>
      new Promise<string>((resolve) => {
        const opening = indexedDB.open('phrase-gym.practice.v1', 1)
        opening.onsuccess = () => {
          const read = opening.result
            .transaction('journal')
            .objectStore('journal')
            .get(`take-v2:${id}`)
          read.onsuccess = async () =>
            resolve(
              Array.from(
                new Uint8Array(await crypto.subtle.digest('SHA-256', read.result.audio)),
                (b) => b.toString(16).padStart(2, '0'),
              ).join(''),
            )
        }
      }),
    take.id,
  )
  expect(retainedHash).toBe(createHash('sha256').update(Buffer.from(newer)).digest('hex'))
})

test('legacy captures without exclusive ownership stay retained with compatible recovery guidance', async ({
  page,
  request,
}) => {
  const id = await downloadSession(page)
  const { take, bytes } = await takeFixture(request, id)
  const { capture_lease: _lease, ...legacy } = take
  await writeTake(page, legacy, bytes.slice(0, 2048))
  await page.evaluate(() => window.dispatchEvent(new Event('online')))
  await expect(page.getByRole('alert').filter({ hasText: 'older capture' })).toBeVisible()
  expect(await readTake(page, take.id)).toEqual({
    complete: false,
    status: 'capturing',
    size: 2048,
  })
  expect(
    await (await request.get(`/api/v1/phrases/${take.target_id}/recordings`)).json(),
  ).toHaveLength(0)
})

test('new captures use a separate journal namespace that an older unleased recovery cannot mutate', async ({
  page,
  request,
}) => {
  const id = await downloadSession(page)
  const { take, bytes } = await takeFixture(request, id)
  await writeTake(page, take, bytes.slice(0, 2048))
  const legacyKeys = await page.evaluate(
    () =>
      new Promise<string[]>((resolve) => {
        const opening = indexedDB.open('phrase-gym.practice.v1', 1)
        opening.onsuccess = () => {
          const keys = opening.result.transaction('journal').objectStore('journal').getAllKeys()
          keys.onsuccess = () =>
            resolve(keys.result.map(String).filter((key) => key.startsWith('take:')))
        }
      }),
  )
  expect(legacyKeys).toHaveLength(0)
  expect(await readTake(page, take.id)).toEqual({
    complete: false,
    status: 'capturing',
    size: 2048,
  })
})
