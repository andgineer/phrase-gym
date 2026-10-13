import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { createHash } from 'node:crypto'
import { readFileSync, writeFileSync } from 'node:fs'
import type { OutputBundle } from 'rollup'

let shellAssets: string[] = []

export default defineConfig({
  plugins: [
    vue(),
    {
      name: 'prepared-offline-shell',
      generateBundle(_options, bundle: OutputBundle) {
        shellAssets = [
          '/index.html',
          '/manifest.webmanifest',
          '/icon.svg',
          '/icon-192.png',
          '/icon-512.png',
          '/apple-touch-icon.png',
          ...Object.keys(bundle)
            .filter((name) => name.startsWith('assets/'))
            .map((name) => `/${name}`),
        ]
      },
      closeBundle() {
        const version = createHash('sha256')
          .update(readFileSync('dist/index.html'))
          .digest('hex')
          .slice(0, 16)
        const worker = readFileSync('public/sw.js', 'utf8')
          .replaceAll('SHELL_VERSION', version)
          .replace('const assets = SHELL_ASSETS', `const assets = ${JSON.stringify(shellAssets)}`)
        writeFileSync('dist/sw.js', worker)
      },
    },
  ],
  server: { proxy: { '/api': 'http://127.0.0.1:8000' } },
})
