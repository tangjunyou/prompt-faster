import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (!id.includes('node_modules')) return undefined
          if (id.includes('monaco-editor') || id.includes('@monaco-editor')) {
            return 'monaco'
          }
          if (id.includes('react-router')) {
            return 'react-router'
          }
          if (id.includes('react-dom') || id.includes('react')) {
            return 'react'
          }
          if (id.includes('@radix-ui')) {
            return 'radix'
          }
          if (id.includes('@tanstack')) {
            return 'tanstack'
          }
          if (id.includes('@xyflow')) {
            return 'xyflow'
          }
          return 'vendor'
        },
      },
    },
  },
})
