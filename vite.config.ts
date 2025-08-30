import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'
import { resolve } from 'path'

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },

  // Build configuration
  build: {
    // Output directory (Tauri expects ../dist from src-tauri)
    outDir: 'dist',
    
    // Clear output directory before build
    emptyOutDir: true,
    
    // Generate sourcemaps for debugging
    sourcemap: true,
    
    // Target modern browsers (Tauri uses modern webview)
    target: 'esnext',
    
    // Optimize chunks
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
          tauri: ['@tauri-apps/api'],
        },
      },
    },
  },

  // Define global variables
  define: {
    __APP_VERSION__: JSON.stringify(process.env.npm_package_version || '1.0.0'),
  },

  // Resolve configuration
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
      '@components': resolve(__dirname, 'src/components'),
      '@assets': resolve(__dirname, 'src/assets'),
    },
  },

  // CSS configuration
  css: {
    devSourcemap: true,
  },

  // Environment variables
  envPrefix: ['VITE_', 'TAURI_'],
}))