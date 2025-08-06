import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    environment: 'happy-dom',
    setupFiles: ['../../../shared/test/setup.ts'],
    globals: true,
  },
})
