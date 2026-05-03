#!/usr/bin/env node
/* [304A-1] El entorno local cambia de rama/proyecto con frecuencia.
 * npm run dev fuerza una BD PostgreSQL aislada por rama antes de que dotenv cargue
 * .env en Rust, evitando mezclar historiales _sqlx_migrations incompatibles. */

import { spawn, spawnSync } from 'node:child_process';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = dirname(fileURLToPath(import.meta.url));
const cwd = resolve(scriptDir, '..');
const cargoToml = resolve(cwd, 'Cargo.toml');
const frontendDir = resolve(cwd, 'frontend');
const envPath = resolve(cwd, '.env');
/* Cada rama usa su propio subdirectorio bajo glory-target para evitar
 * que .rmeta de una rama contaminen el build de otra (fix: stale extern location). */
const cargoTargetBase = process.env.CARGO_TARGET_DIR_BASE || (isWindowsPlatform() ? 'C:\\tmp\\glory-target' : resolve(tmpdir(), 'glory-target'));
const cargoTargetMaxMb = process.env.GLORY_CARGO_TARGET_MAX_MB || '4096';
const cargoCleanIntervalSeconds = process.env.GLORY_CARGO_CLEAN_INTERVAL_SECONDS || '120';


if (!existsSync(cargoToml)) {
    console.error('[glory-dev] No se encontro Cargo.toml en', cwd);
    process.exit(1);
}

if (!existsSync(frontendDir)) {
    console.error('[glory-dev] No se encontro frontend/ en', cwd);
    process.exit(1);
}

const isWin = isWindowsPlatform();
const spawnOpts = isWin ? { shell: true } : {};
const children = [];

function isWindowsPlatform() {
    return process.platform === 'win32';
}

function commandName(cmd) {
    if (!isWin) {
        return cmd;
    }

    if (cmd === 'npm') {
        return 'npm.cmd';
    }
    if (cmd === 'cargo') {
        return 'cargo.exe';
    }
    if (cmd === 'git') {
        return 'git.exe';
    }
    if (cmd === 'psql') {
        return 'psql.exe';
    }
    return cmd;
}

function runGit(args) {
    const result = spawnSync(commandName('git'), args, { cwd, encoding: 'utf8' });
    return result.status === 0 ? result.stdout.trim() : '';
}

function parseEnvFile(path) {
    if (!existsSync(path)) {
        return new Map();
    }

    const entries = new Map();
    for (const rawLine of readFileSync(path, 'utf8').split(/\r?\n/)) {
        const line = rawLine.trim();
        if (!line || line.startsWith('#') || !line.includes('=')) {
            continue;
        }

        const separator = line.indexOf('=');
        const key = line.slice(0, separator).trim();
        let value = line.slice(separator + 1).trim();
        if ((value.startsWith('"') && value.endsWith('"')) || (value.startsWith("'") && value.endsWith("'"))) {
            value = value.slice(1, -1);
        }
        entries.set(key, value);
    }
    return entries;
}

function slugifyBranchName(branch) {
    const slug = branch
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, '_')
        .replace(/^_+|_+$/g, '');
    return slug || 'local';
}

function detectBranch() {
    return runGit(['branch', '--show-current']) || runGit(['rev-parse', '--short', 'HEAD']) || 'local';
}

function detectPackageName() {
    const toml = readFileSync(cargoToml, 'utf8');
    const match = toml.match(/^name\s*=\s*"([^"]+)"/m);
    return (match ? match[1] : 'glory').replace(/-/g, '_');
}

function databaseNameForBranch(branch) {
    if (process.env.GLORY_DEV_DB_NAME) {
        return process.env.GLORY_DEV_DB_NAME;
    }
    const pkgName = detectPackageName();
    const isDefault = branch === 'main' || branch === 'master';
    return isDefault ? pkgName : `${pkgName}_${slugifyBranchName(branch)}`;
}

function databaseUrlForName(envValues, dbName) {
    const template = process.env.GLORY_DEV_DATABASE_URL_TEMPLATE || envValues.get('GLORY_DEV_DATABASE_URL_TEMPLATE');
    if (template) {
        return template.replaceAll('{db}', dbName);
    }

    const baseUrl = process.env.DATABASE_URL || envValues.get('DATABASE_URL') || 'postgres://postgres:root@localhost:5432/postgres';
    const parsed = new URL(baseUrl);
    parsed.pathname = `/${dbName}`;
    return parsed.toString();
}

function findPsql() {
    const command = spawnSync(commandName('psql'), ['--version'], { encoding: 'utf8' });
    if (command.status === 0) {
        return commandName('psql');
    }

    if (!isWin) {
        return null;
    }

    const postgresRoot = 'C:\\Program Files\\PostgreSQL';
    if (!existsSync(postgresRoot)) {
        return null;
    }

    const versions = readdirSync(postgresRoot).sort((a, b) => b.localeCompare(a, undefined, { numeric: true }));
    for (const version of versions) {
        const candidate = resolve(postgresRoot, version, 'bin', 'psql.exe');
        if (existsSync(candidate)) {
            return candidate;
        }
    }
    return null;
}

function quoteSqlIdentifier(value) {
    if (!/^[a-zA-Z0-9_]+$/.test(value)) {
        throw new Error(`Nombre de BD inseguro: ${value}`);
    }
    return `"${value.replaceAll('"', '""')}"`;
}

function runPsql(psql, url, sql) {
    const parsed = new URL(url);
    const env = {
        ...process.env,
        PGPASSWORD: decodeURIComponent(parsed.password),
    };
    const args = [
        '-h',
        parsed.hostname,
        '-p',
        parsed.port || '5432',
        '-U',
        decodeURIComponent(parsed.username),
        '-d',
        'postgres',
        '-tAc',
        sql,
    ];
    return spawnSync(psql, args, { cwd, env, encoding: 'utf8' });
}

function resetPublicSchema(psql, databaseUrl) {
    const parsed = new URL(databaseUrl);
    const env = {
        ...process.env,
        PGPASSWORD: decodeURIComponent(parsed.password),
    };
    const args = [
        '-h',
        parsed.hostname,
        '-p',
        parsed.port || '5432',
        '-U',
        decodeURIComponent(parsed.username),
        '-d',
        parsed.pathname.slice(1),
        '-c',
        'DROP SCHEMA public CASCADE; CREATE SCHEMA public; GRANT ALL ON SCHEMA public TO CURRENT_USER;',
    ];
    return spawnSync(psql, args, { cwd, env, encoding: 'utf8' });
}

function runSqlxMigrations(databaseUrl) {
    const env = {
        ...process.env,
        DATABASE_URL: databaseUrl,
        CARGO_TARGET_DIR: cargoTargetDir,
    };
    return spawnSync(commandName('cargo'), ['sqlx', 'migrate', 'run'], { cwd, env, encoding: 'utf8' });
}

function resolveRustcWrapper() {
    if (process.env.RUSTC_WRAPPER) {
        return process.env.RUSTC_WRAPPER;
    }

    if (isWin) {
        const userProfile = process.env.USERPROFILE;
        if (userProfile) {
            const sccachePath = resolve(userProfile, '.cargo', 'bin', 'sccache.exe');
            if (existsSync(sccachePath)) {
                return sccachePath;
            }
        }
    }

    const command = spawnSync('sccache', ['--version'], { encoding: 'utf8', shell: isWin });
    return command.status === 0 ? 'sccache' : null;
}

function spawnCargoTargetWatcher(env) {
    if (!isWin) {
        return;
    }

    const watcherScript = resolve(cwd, 'scripts', 'watch-cargo-target.ps1');
    if (!existsSync(watcherScript)) {
        return;
    }

    spawnProc(
        'cargo-target-watch',
        'powershell',
        [
            '-ExecutionPolicy',
            'Bypass',
            '-File',
            watcherScript,
            '-TargetDirs',
            cargoTargetDir,
            '-MaxTotalMB',
            cargoTargetMaxMb,
            '-IntervalSeconds',
            cargoCleanIntervalSeconds,
        ],
        { cwd, env, stdio: 'ignore' },
    );
}

function ensureMigrationsAreCompatible(databaseUrl, dbName) {
    const firstRun = runSqlxMigrations(databaseUrl);
    if (firstRun.status === 0) {
        return;
    }

    const output = `${firstRun.stdout}\n${firstRun.stderr}`;
    if (!/VersionMissing|VersionMismatch|previously applied but has been modified/.test(output)) {
        process.stdout.write(firstRun.stdout);
        process.stderr.write(firstRun.stderr);
        console.error('[glory-dev] No se pudieron aplicar las migraciones locales.');
        process.exit(firstRun.status ?? 1);
    }

    const psql = findPsql();
    if (!psql) {
        process.stderr.write(firstRun.stderr);
        console.error('[glory-dev] La BD local tiene migraciones incompatibles, pero psql no esta disponible para resetearla.');
        process.exit(firstRun.status ?? 1);
    }

    console.warn(`[glory-dev] Historial de migraciones incompatible en ${dbName}; reseteando schema public de desarrollo.`);
    const resetResult = resetPublicSchema(psql, databaseUrl);
    if (resetResult.status !== 0) {
        process.stderr.write(resetResult.stderr);
        console.error('[glory-dev] No se pudo resetear la BD local de desarrollo.');
        process.exit(resetResult.status ?? 1);
    }

    const secondRun = runSqlxMigrations(databaseUrl);
    if (secondRun.status !== 0) {
        process.stdout.write(secondRun.stdout);
        process.stderr.write(secondRun.stderr);
        console.error('[glory-dev] Las migraciones siguen fallando tras resetear la BD local.');
        process.exit(secondRun.status ?? 1);
    }
}

function ensureDatabaseExists(databaseUrl, dbName) {
    const psql = findPsql();
    if (!psql) {
        console.warn('[glory-dev] psql no esta disponible; si la BD no existe, el backend fallara al conectar.');
        return;
    }

    const existsResult = runPsql(psql, databaseUrl, `SELECT 1 FROM pg_database WHERE datname = '${dbName}'`);
    if (existsResult.status !== 0) {
        console.warn('[glory-dev] No se pudo verificar la BD local:', existsResult.stderr.trim());
        return;
    }

    if (existsResult.stdout.trim() === '1') {
        return;
    }

    const createResult = runPsql(psql, databaseUrl, `CREATE DATABASE ${quoteSqlIdentifier(dbName)}`);
    if (createResult.status !== 0) {
        console.warn('[glory-dev] No se pudo crear la BD local:', createResult.stderr.trim());
        return;
    }

    console.log(`[glory-dev] BD local creada: ${dbName}`);
}

function spawnProc(label, cmd, args, options) {
    const proc = spawn(cmd, args, {
        stdio: 'inherit',
        ...spawnOpts,
        ...options,
    });
    proc.on('error', (err) => console.error(`[${label}] Error: ${err.message}`));
    proc.on('exit', (code) => {
        console.log(`[${label}] Proceso terminado con codigo ${code}`);
        cleanup();
    });
    children.push(proc);
    return proc;
}

function cleanup() {
    for (const child of children) {
        if (!child.killed) {
            child.kill();
        }
    }
}

process.on('SIGINT', cleanup);
process.on('SIGTERM', cleanup);

function detectBinName() {
    const toml = readFileSync(cargoToml, 'utf8');
    const match = toml.match(/^name\s*=\s*"([^"]+)"/m);
    return match ? match[1] : null;
}

const envValues = parseEnvFile(envPath);
const branch = detectBranch();
/* Subdirectorio por proyecto+rama: aísla artefactos entre proyectos Y entre ramas.
 * Formato: {pkg_name}_{branch_slug}  →  C:\tmp\glory-target\glory_backend_glory_rust_nakomi */
const cargoTargetDir = process.env.CARGO_TARGET_DIR || resolve(cargoTargetBase, `${detectPackageName()}_${slugifyBranchName(branch)}`);
const dbName = databaseNameForBranch(branch);
if (!/^[a-zA-Z0-9_]+$/.test(dbName)) {
    console.error(`[glory-dev] Nombre de BD inseguro: ${dbName}`);
    process.exit(1);
}
const databaseUrl = databaseUrlForName(envValues, dbName);
const binName = detectBinName();

if (!binName) {
    console.error('[glory-dev] No se pudo detectar el nombre del binario en Cargo.toml');
    process.exit(1);
}

if (process.argv.includes('--print-db')) {
    console.log(databaseUrl);
    process.exit(0);
}

ensureDatabaseExists(databaseUrl, dbName);
ensureMigrationsAreCompatible(databaseUrl, dbName);

const childEnv = {
    ...process.env,
    DATABASE_URL: databaseUrl,
    CARGO_TARGET_DIR: cargoTargetDir,
    KAMPLES_PG_DBNAME: dbName,
    GLORY_DEV_BRANCH: branch,
    GLORY_DEV_DB_NAME: dbName,
};

const rustcWrapper = resolveRustcWrapper();
if (rustcWrapper) {
    childEnv.RUSTC_WRAPPER = rustcWrapper;
}

console.log(`[glory-dev] Rama: ${branch}`);
console.log(`[glory-dev] Base local: ${dbName}`);
console.log(`[glory-dev] Cargo target: ${cargoTargetDir}`);
if (rustcWrapper) {
    console.log(`[glory-dev] Rust cache: ${rustcWrapper}`);
}
console.log(`[glory-dev] Iniciando backend (cargo run --bin ${binName}) y frontend (vite)...\n`);

spawnCargoTargetWatcher(childEnv);
spawnProc('backend', 'cargo', ['run', '--bin', binName], { cwd, env: childEnv });
spawnProc('frontend', 'npm', ['run', 'dev'], { cwd: frontendDir, env: childEnv });