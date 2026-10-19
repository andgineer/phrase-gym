import { expect, test } from 'vitest'
import { audioInput } from '../src/inboxDraft'

test('supplied audio preserves exact bytes and normalizes supported file extensions', async () => {
  const bytes = new Uint8Array([0, 128, 255, 65])
  expect(await audioInput(new File([bytes], 'Reference.WAV'))).toEqual({
    mime: 'audio/wav',
    base64: 'AID/QQ==',
  })
  expect((await audioInput(new File([bytes], 'Reference.m4a'))).mime).toBe('audio/mp4')
})

test('empty, unsupported and oversized files never reach upload encoding', async () => {
  for (const file of [
    new File([], 'empty.wav'),
    new File(['text'], 'not-audio.txt'),
    new File([new Uint8Array(2 * 1024 * 1024 + 1)], 'large.wav'),
  ]) {
    await expect(audioInput(file)).rejects.toThrow('audio')
  }
})
