#!/usr/bin/env node
/* [114A-SEO3] Pre-rendering de rutas públicas para crawlers.
 * Construye frontend → sirve dist/ localmente → navega con Puppeteer → guarda HTML → cierra.
 * Ejecutar: node scripts/prerender.mjs
 * Requisito: npm i -D puppeteer (en raíz o frontend/)
 *
 * Lee slugs dinámicos de content/*.toml para generar /servicios/:slug, /proyectos/:slug, etc.
 * El resultado va a prerendered/ en la raíz del proyecto. */

import { execSync } from 'node:child_process';
import fs from 'node:fs';
import http from 'node:http';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const FRONTEND = path.join(ROOT, 'frontend');
const CONTENT = path.join(ROOT, 'content');
const DIST_DIR = path.join(FRONTEND, 'dist');
const OUT_DIR = path.join(ROOT, 'prerendered');
const PREVIEW_PORT = 4174;
const PREVIEW_HOST = '127.0.0.1';
const BASE_URL = `http://${PREVIEW_HOST}:${PREVIEW_PORT}`;

/* Rutas estáticas públicas (excluyendo /panel, que es privado) */
const STATIC_ROUTES = [
    '/',
    '/servicios',
    '/proyectos',
    '/nosotros',
    '/blog',
    '/soluciones',
    '/soluciones/hosting',
    '/soluciones/vps',
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

function getContentType(filePath) {
    switch (path.extname(filePath).toLowerCase()) {
    case '.html': return 'text/html; charset=utf-8';
    case '.js': return 'application/javascript; charset=utf-8';
    case '.css': return 'text/css; charset=utf-8';
    case '.json': return 'application/json; charset=utf-8';
    case '.svg': return 'image/svg+xml';
    case '.png': return 'image/png';
    case '.jpg':
    case '.jpeg': return 'image/jpeg';
    case '.webp': return 'image/webp';
    case '.woff2': return 'font/woff2';
    case '.woff': return 'font/woff';
    case '.ttf': return 'font/ttf';
    default: return 'application/octet-stream';
    }
}

function resolveDistPath(requestPath) {
    const pathname = decodeURIComponent(requestPath.split('?')[0]);
    if (pathname.startsWith('/api/')) {
        return null;
    }
    const relative = pathname === '/'
        ? 'index.html'
        : pathname.replace(/^\//, '');
    const candidate = path.resolve(DIST_DIR, relative);
    if (!candidate.startsWith(DIST_DIR)) {
        return path.join(DIST_DIR, 'index.html');
    }
    if (fs.existsSync(candidate) && fs.statSync(candidate).isFile()) {
        return candidate;
    }
    return path.join(DIST_DIR, 'index.html');
}

function startPreviewServer() {
    const server = http.createServer((req, res) => {
        const filePath = resolveDistPath(req.url || '/');
        if (!filePath) {
            res.writeHead(404, { 'Content-Type': 'application/json; charset=utf-8' });
            res.end(JSON.stringify({ error: 'Not found during prerender preview' }));
            return;
        }
        try {
            const body = fs.readFileSync(filePath);
            res.writeHead(200, { 'Content-Type': getContentType(filePath) });
            res.end(body);
        } catch (error) {
            res.writeHead(500, { 'Content-Type': 'text/plain; charset=utf-8' });
            res.end(`No se pudo servir ${filePath}: ${error.message}`);
        }
    });

    return new Promise((resolve, reject) => {
        server.once('error', reject);
        server.listen(PREVIEW_PORT, PREVIEW_HOST, () => resolve(server));
    });
}

async function main() {
    /* 1. Build del frontend */
    console.log('[prerender] Construyendo frontend...');
    execSync('npx vite build', { cwd: FRONTEND, stdio: 'inherit' });

    /* 2. Arrancar servidor estático local */
    console.log('[prerender] Iniciando servidor de preview...');
    const preview = await startPreviewServer();

    try {
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
                await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 15000 });
                /* Esperar a que React monte el layout base.
                 * No dependemos de networkidle0 porque varias rutas disparan fetches
                 * secundarios que no son requisito para emitir HTML SEO útil. */
                await page.waitForFunction(
                    () => {
                        const main = document.querySelector('main, [role="main"]');
                        const text = main?.textContent?.trim() || '';
                        return text.length > 0;
                    },
                    { timeout: 10000 }
                );
                await new Promise(resolve => setTimeout(resolve, 350));

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
        await new Promise(resolve => preview.close(resolve));
    }
}

main().catch(err => {
    console.error('[prerender] Error fatal:', err);
    process.exit(1);
});
