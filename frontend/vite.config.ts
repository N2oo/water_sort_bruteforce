import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'

/**
 * In development the app talks to the API through this proxy, so the browser
 * only ever sees one origin and `VITE_API_BASE_URL` can stay empty.
 *
 *   API_PROXY_TARGET=http://localhost:8080 npm run dev
 */
const target = process.env.API_PROXY_TARGET ?? 'http://localhost:8080'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      '/api': { target, changeOrigin: true },
      '/health': { target, changeOrigin: true },
    },
  },
  test: {
    environment: 'node',
    include: ['src/**/*.test.ts'],
  },
})
