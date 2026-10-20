import { execFile } from 'node:child_process'
import { mkdtemp, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'
import { promisify } from 'node:util'

const execute = promisify(execFile)

export async function prepareTextWithNativeFakes(server: string, accepted = false) {
  const directory = await mkdtemp(join(tmpdir(), 'phrase-coach-text-browser-'))
  try {
    const { stdout } = await execute(
      resolve('../target/debug/examples/fake_text_worker'),
      [server, directory, ...(accepted ? ['accepted'] : [])],
      { timeout: 15000 },
    )
    return JSON.parse(stdout) as {
      publication: { target_ids: string[]; lesson_id: string; ready_count: number }
      reference: string
    }
  } finally {
    await rm(directory, { recursive: true, force: true })
  }
}
