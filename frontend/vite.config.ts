import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

/* [074A-14] Bundle splitting + optimizaciones de build para Core Web Vitals.
 * manualChunks separa dependencias pesadas en archivos independientes para
 * mejorar caching (vendor rara vez cambia) y reducir main bundle size. */
export default defineConfig({
  plugins: [react()],
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          'react-core': ['react', 'react-dom', 'react-router-dom'],
          'query': ['@tanstack/react-query'],
          'editor': [
            '@tiptap/react', '@tiptap/starter-kit',
            '@tiptap/extension-image', '@tiptap/extension-link',
            '@tiptap/extension-placeholder',
          ],
          'stripe': ['@stripe/react-stripe-js', '@stripe/stripe-js'],
          'i18n': ['i18next', 'react-i18next', 'i18next-browser-languagedetector'],
        },
      },
    },
  },
  server: {
    port: 5173,
    /* Permitir servir fuentes desde App/Assets/fonts/ (fuera de frontend/) */
    fs: {
      allow: ['..'],
    },
    /* Proxy API requests al backend Rust en desarrollo */
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
      /* [064A-16] Proxy uploads para que avatares se sirvan desde el backend */
      '/uploads': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
      '/swagger-ui': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
      '/api-docs': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
      /* [104A-1] Proxy WebSocket en desarrollo */
      '/ws': {
        target: 'http://localhost:3000',
        changeOrigin: true,
        ws: true,
      },
    },
  },
});
