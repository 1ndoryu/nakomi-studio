#!/usr/bin/env node
/* [114A-SEO3] Pre-rendering de rutas públicas para crawlers.
 * Arranca vite preview → navega con Puppeteer → guarda HTML → cierra.
 * Ejecutar: node scripts/prerender.mjs
 * Requisito: npm i -D puppeteer (en frontend/)
 *
 * Lee slugs dinámicos de content/*.toml para generar /servicios/:slug, /proyectos/:slug, etc.
 * El resultado va a prerendered/ en la raíz del proyecto. */

import { execSync, spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const FRONTEND = path.join(ROOT, 'frontend');
const CONTENT = path.join(ROOT, 'content');
const OUT_DIR = path.join(ROOT, 'prerendered');
const PREVIEW_PORT = 4174;
const BASE_URL = `http://localhost:${PREVIEW_PORT}`;

/* Rutas estáticas públicas (excluyendo /panel, que es privado) */
const STATIC_ROUTES = [
    '/',
    '/servicios',
    '/proyectos',
    '/nosotros',
    '/blog',
    '/soluciones',
    '/soluciones/hosting',
];

/* Extrae slugs de un archivo TOML buscando líneas con `slug = "..."` */
function extractSlugs(tomlFile) {
    if (!fs.existsSync(tomlFile)) return [];
    const content = fs.readFileSync(tomlFile, 'utf-8');
    const matches = content.matchAll(/^slug\s*=\s*"([^"]+)"/gm);
    return Array.from(matches, m => m[1]);
}

/* Construye la lista completa de rutas a pre-renderizar */
function buildRoutes() {
    const routes = [...STATIC_ROUTES];

    const serviceSlugs = extractSlugs(path.join(CONTENT, 'services.toml'));
    for (const slug of serviceSlugs) {
        routes.push(`/servicios/${slug}`);
    }

    const projectSlugs = extractSlugs(path.join(CONTENT, 'projects.toml'));
    for (const slug of projectSlugs) {
        routes.push(`/proyectos/${slug}`);
    }

    /* Blog: si existe blog.toml futuro, leer slugs. Por ahora no hay fixture de blog. */
    return routes;
}

/* Espera a que el servidor responda */
async function waitForServer(url, maxAttempts = 30) {
    for (let i = 0; i < maxAttempts; i++) {
        try {
            const resp = await fetch(url);
            if (resp.ok) return;
        } catch { /* retry */ }
        await new Promise(r => setTimeout(r, 500));
    }
    throw new Error(`Servidor no respondió en ${maxAttempts * 500}ms`);
}

async function main() {
    /* 1. Build del frontend */
    console.log('[prerender] Construyendo frontend...');
    execSync('npx vite build', { cwd: FRONTEND, stdio: 'inherit' });

    /* 2. Arrancar vite preview */
    console.log('[prerender] Iniciando servidor de preview...');
    /* En Windows, spawn sin shell no resuelve npx/.cmd. Usar shell solo en Windows. */
    const isWin = process.platform === 'win32';
    const preview = spawn('npx', ['vite', 'preview', '--port', String(PREVIEW_PORT)], {
        cwd: FRONTEND,
        stdio: 'pipe',
        shell: isWin,
    });

    /* Capturar errores del proceso */
    preview.stderr.on('data', d => {
        const msg = d.toString();
        if (!msg.includes('ExperimentalWarning')) process.stderr.write(msg);
    });

    try {
        await waitForServer(BASE_URL);
        console.log('[prerender] Servidor listo');

        /* 3. Importar puppeteer dinámicamente */
        const puppeteer = await import('puppeteer');
        const browser = await puppeteer.default.launch({
            headless: true,
            args: ['--no-sandbox', '--disable-setuid-sandbox'],
        });

        const routes = buildRoutes();
        console.log(`[prerender] ${routes.length} rutas a pre-renderizar`);

        /* Limpiar directorio de salida */
        if (fs.existsSync(OUT_DIR)) {
            fs.rmSync(OUT_DIR, { recursive: true });
        }
        fs.mkdirSync(OUT_DIR, { recursive: true });

        for (const route of routes) {
            const page = await browser.newPage();
            const url = `${BASE_URL}${route}`;

            try {
                await page.goto(url, { waitUntil: 'networkidle0', timeout: 15000 });
                /* Esperar a que React monte (el #root no esté vacío) */
                await page.waitForFunction(
                    () => {
                        const root = document.getElementById('root');
                        return root && root.children.length > 0;
                    },
                    { timeout: 10000 }
                );

                const html = await page.content();

                /* Ruta del archivo de salida */
                const clean = route === '/' ? 'index' : route.slice(1);
                const filePath = path.join(OUT_DIR, `${clean}.html`);
                const dir = path.dirname(filePath);
                fs.mkdirSync(dir, { recursive: true });
                fs.writeFileSync(filePath, html, 'utf-8');
                console.log(`  ✓ ${route} → ${path.relative(ROOT, filePath)}`);
            } catch (err) {
                console.error(`  ✗ ${route}: ${err.message}`);
            } finally {
                await page.close();
            }
        }

        await browser.close();
        console.log(`[prerender] Completado. Archivos en ${path.relative(ROOT, OUT_DIR)}/`);
    } finally {
        preview.kill('SIGTERM');
    }
}

main().catch(err => {
    console.error('[prerender] Error fatal:', err);
    process.exit(1);
});
