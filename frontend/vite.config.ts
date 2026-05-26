import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    // Dev: proxy /api → PHP Gateway on host 9080 (8080 reserved for archivio-parlante-starter)
    proxy: {
      '/api': {
        target: 'http://localhost:9080',
        changeOrigin: true,
      },
    },
  },
})
