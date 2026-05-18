import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import type { Plugin } from 'vite';

/* [175A-1] Critical CSS: convierte <link rel="stylesheet"> en carga no-bloqueante.
 * Técnica "media=print + onload": el navegador descarga el CSS en paralelo con el JS
 * pero NO bloquea el primer paint. El spinner (inline styles) aparece inmediatamente.
 * Esto elimina la penalización de 2400ms de render-blocking en PageSpeed.
 * Para SPAs esto es seguro: el CSS carga antes de que React termine de arrancar (~2-3s mobile). */
function viteDeferCss(): Plugin {
  return {
    name: 'defer-css',
    enforce: 'post',
    apply: 'build',
    transformIndexHtml(html: string): string {
      return html.replace(
        /<link rel="stylesheet"([^>]*)href="(\/assets\/[^"]+\.css)"([^>]*)>/g,
        (_match, before: string, href: string, after: string) => {
          const attrs = (before + after).trim();
          const crossorigin = attrs.includes('crossorigin') ? ' crossorigin' : '';
          /* Al cargar el CSS: remover overlay FOUC + revelar #root. Timeout 4s como fallback
           * por si onload no dispara (navegadores viejos, JS bloqueado parcialmente). */
          const onloadHandler =
            `var o=document.getElementById('fouc-overlay');if(o){o.style.transition='opacity 0.15s';o.style.opacity='0';setTimeout(function(){o.remove()},200);}` +
            `var r=document.getElementById('root');if(r)r.style.removeProperty('opacity');` +
            `this.onload=null;this.media='all'`;
          return (
            `<link rel="preload" as="style"${crossorigin} href="${href}">` +
            `<link rel="stylesheet" media="print"${crossorigin} onload="${onloadHandler}" href="${href}">` +
            `<noscript><link rel="stylesheet"${crossorigin} href="${href}"></noscript>` +
            `<script>setTimeout(function(){var o=document.getElementById('fouc-overlay');if(o)o.remove();var r=document.getElementById('root');if(r)r.style.removeProperty('opacity')},4000)<\/script>`
          );
        },
      );
    },
  };
}

/* [074A-14] Bundle splitting + optimizaciones de build para Core Web Vitals.
 * manualChunks separa dependencias pesadas en archivos independientes para
 * mejorar caching (vendor rara vez cambia) y reducir main bundle size.
 * [114A-19] modulePreload filtrado: editor y stripe son lazy (solo admin panel);
 * precargarlos en TODAS las páginas desperdicia ~393KB en mobile. */
export default defineConfig({
  plugins: [react(), viteDeferCss()],
  build: {
    modulePreload: {
      resolveDependencies: (_filename, deps) => {
        return deps.filter(dep => !dep.includes('editor') && !dep.includes('stripe'));
      },
    },
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
