import { execFile } from 'node:child_process'
import { mkdtemp, readFile, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'
import { promisify } from 'node:util'

import { prepareTextWithNativeFakes } from './text-intake'
const execute = promisify(execFile)
export async function prepareWithNativeFakeCoach(server: string) {
  const directory = await mkdtemp(join(tmpdir(), 'phrase-coach-browser-'))
  try {
    const bundle = process.env.PHRASE_GYM_COACH_BUNDLE
    if (bundle) {
      const report = join(directory, 'report.json')
      const input = join(directory, 'intake')
      await execute(
        '/usr/bin/open',
        [
          '-W',
          '-n',
          '-a',
          bundle,
          '--env',
          'PHRASE_GYM_LAUNCH_CANARY=synthetic',
          '--env',
          'ANTHROPIC_API_KEY=synthetic-injected-must-not-reach-child',
          '--env',
          'OPENAI_API_KEY=synthetic-injected-must-not-reach-child',
          '--args',
          '--smoke-test',
          report,
          '--smoke-input',
          server,
          input,
        ],
        { timeout: 30000 },
      )
      const checks = JSON.parse(await readFile(report, 'utf8'))
      if (!checks.passed || !checks.intake || !('Ok' in checks.intake))
        throw new Error(`Packaged intake failed: ${JSON.stringify(checks)}`)
      console.log(
        `Packaged synthetic Coach intake passed; renderer=${checks.renderer?.backend}, idle frames=${checks.idle_frames}; no provider called`,
      )
      return JSON.parse(await readFile(join(input, 'state', 'publication.json'), 'utf8')) as {
        target_ids: string[]
      }
    }
    return (await prepareTextWithNativeFakes(server, true)).publication
  } finally {
    await rm(directory, { recursive: true, force: true })
  }
}
