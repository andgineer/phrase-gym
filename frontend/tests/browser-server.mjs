import { spawn, spawnSync } from 'node:child_process'
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

const port = process.argv[process.argv.indexOf('--port') + 1]
if (!/^\d+$/.test(port)) throw new Error('--port is required')
const build = spawnSync(
  'cargo',
  ['build', '--locked', '--package', 'phrase-gym-server', '--features', 'test-support'],
  { stdio: 'inherit' },
)
if (build.status !== 0) process.exit(build.status ?? 1)
const directory = mkdtempSync(join(tmpdir(), 'phrase-gym-browser-'))
const credentials = join(directory, 'credentials.json')
writeFileSync(
  credentials,
  JSON.stringify([
    {
      actor: 'synthetic-learner',
      token: 'synthetic-learner-00000000000000000000',
      capabilities: ['learner'],
    },
    {
      actor: 'synthetic-browser',
      token: 'synthetic-browser-00000000000000000000',
      capabilities: ['learner', 'material_submit', 'worker', 'profile_read', 'profile_update'],
    },
  ]),
  { mode: 0o600 },
)
const binary = `target/debug/phrase-gym-server${process.platform === 'win32' ? '.exe' : ''}`
const server = spawn(
  binary,
  [
    'serve',
    '--credentials-file',
    credentials,
    '--data-dir',
    directory,
    '--client-dir',
    'frontend/dist',
    '--port',
    port,
  ],
  { stdio: 'inherit' },
)
for (const signal of ['SIGINT', 'SIGTERM']) process.on(signal, () => server.kill(signal))
server.on('error', (error) => {
  console.error(error)
  rmSync(directory, { recursive: true, force: true })
  process.exit(1)
})
server.on('exit', (code, signal) => {
  rmSync(directory, { recursive: true, force: true })
  process.exit(signal ? 0 : (code ?? 1))
})
