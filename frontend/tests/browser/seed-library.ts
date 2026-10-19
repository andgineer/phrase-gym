import { readFileSync } from 'node:fs'
import type { APIRequestContext } from '@playwright/test'

export function tone() {
  const bytes = Buffer.alloc(44 + 16000 * 2)
  bytes.write('RIFF')
  bytes.writeUInt32LE(bytes.length - 8, 4)
  bytes.write('WAVEfmt ', 8)
  bytes.writeUInt32LE(16, 16)
  bytes.writeUInt16LE(1, 20)
  bytes.writeUInt16LE(1, 22)
  bytes.writeUInt32LE(16000, 24)
  bytes.writeUInt32LE(32000, 28)
  bytes.writeUInt16LE(2, 32)
  bytes.writeUInt16LE(16, 34)
  bytes.write('data', 36)
  bytes.writeUInt32LE(bytes.length - 44, 40)
  for (let index = 0; index < 16000; index++)
    bytes.writeInt16LE(
      Math.trunc(6000 * Math.sin((2 * Math.PI * 440 * index) / 16000)),
      44 + index * 2,
    )
  return bytes.toString('base64')
}

export async function seedLibrary(request: APIRequestContext) {
  const audio_base64 = tone()
  for (const language of ['en', 'sr']) {
    const content = JSON.parse(readFileSync(`../fixtures/manual-${language}.json`, 'utf8'))
    for (const target of content.targets) {
      const response = await request.post('/api/v1/media', {
        data: {
          metadata: {
            language,
            text: target.text,
            role: 'reference',
            origin: 'test',
            source: 'Synthetic browser fixture',
            locale: `${language}-${language === 'sr' ? 'RS' : 'US'}`,
            preparation_version: '1',
          },
          mime_type: 'audio/wav',
          audio_base64,
        },
      })
      if (!response.ok()) throw new Error(`Media seed failed: ${await response.text()}`)
      target.reference_audio_id = (await response.json()).id
    }
    const staged = await request.post('/api/v1/imports', { data: content })
    const report = await staged.json()
    if (!staged.ok() || report.status !== 'staged')
      throw new Error(`Content seed failed: ${JSON.stringify(report)}`)
    const published = await request.post(`/api/v1/imports/${report.id}/publish`, { data: {} })
    if (!published.ok()) throw new Error(`Content publish failed: ${await published.text()}`)
  }
}
