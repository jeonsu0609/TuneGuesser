import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { resolve } from '@tauri-apps/api/path'
import mkcert from 'vite-plugin-mkcert'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      '/login': {
        target: 'http://127.0.0.1:12345',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
    },
  },
  build: {
    outDir: 'dist',
    rollupOptions: {
      output: {
        entryFileNames: `dist/name.js`,
        chunkFileNames: `dist/name.js`,
        // assetFileNames: `dist/name.[ext]`,
      },
    },
  },
})
