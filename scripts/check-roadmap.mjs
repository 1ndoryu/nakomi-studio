#!/usr/bin/env node
/* Script que parsea roadmap.md y reporta tareas pendientes.
 * Diseñado para correr como VS Code task con probleMatcher,
 * generando warnings estilo compilador que aparecen en el panel de Problemas.
 * Formato: archivo:linea:columna: warning: mensaje */

import { readFileSync, existsSync, watchFile } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(__dirname, '..');

function findRoadmaps() {
    const paths = [
        resolve(projectRoot, 'roadmap.md'),
        resolve(projectRoot, 'App', 'roadmap.md'),
    ];
    return paths.filter(p => existsSync(p));
}

function parsePendingTasks(filePath) {
    const content = readFileSync(filePath, 'utf-8');
    const lines = content.split('\n');
    const tasks = [];
    let inPendientes = false;

    for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        const lower = line.toLowerCase();

        /* Detectar sección de pendientes */
        if (lower.includes('pendiente') && line.startsWith('#')) {
            inPendientes = true;
            continue;
        }

        /* Otra sección con # termina la zona de pendientes */
        if (inPendientes && line.startsWith('#') && !lower.includes('pendiente')) {
            inPendientes = false;
            continue;
        }

        if (!inPendientes) continue;

        /* Ignorar líneas vacías y marcadores de "sin tareas" */
        const trimmed = line.trim();
        if (!trimmed) continue;
        if (trimmed.includes('sin tareas pendientes')) continue;
        if (trimmed.startsWith('(') && trimmed.endsWith(')')) continue;

        /* Líneas que inician con - o con ### son tareas */
        if (trimmed.startsWith('-') || trimmed.startsWith('###') || trimmed.startsWith('--')) {
            /* Ignorar tareas marcadas como completadas (✅, ~~tachado~~) */
            if (trimmed.includes('✅') || trimmed.includes('COMPLETADO') || /^~~.*~~$/.test(trimmed.replace(/^[-#\s]+/, ''))) continue;
            tasks.push({ line: i + 1, text: trimmed });
        }
    }

    return tasks;
}

function report() {
    const roadmaps = findRoadmaps();
    let totalTasks = 0;

    for (const roadmap of roadmaps) {
        const tasks = parsePendingTasks(roadmap);
        totalTasks += tasks.length;

        for (const task of tasks) {
            /* Formato reconocido por VS Code problemMatcher */
            console.log(`${roadmap}:${task.line}:1: warning: TAREA PENDIENTE: ${task.text}`);
        }
    }

    if (totalTasks === 0) {
        console.log('[roadmap-watcher] Sin tareas pendientes.');
    } else {
        console.log(`\n[roadmap-watcher] ${totalTasks} tarea(s) pendiente(s) detectada(s).`);
    }

    return totalTasks;
}

/* Modo: --watch para vigilar cambios continuamente, normal para una sola ejecucion */
const watchMode = process.argv.includes('--watch');

if (watchMode) {
    console.log('[roadmap-watcher] Vigilando roadmap.md...');
    report();

    const roadmaps = findRoadmaps();
    for (const roadmap of roadmaps) {
        watchFile(roadmap, { interval: 2000 }, () => {
            console.log(`\n[roadmap-watcher] Cambio detectado en ${roadmap}`);
            report();
        });
    }
} else {
    const count = report();
    process.exit(count > 0 ? 1 : 0);
}
