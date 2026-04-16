/* [154A-2] API client para el sistema de fixtures del CMS admin.
 * Conecta con /api/admin/fixtures (status) y /api/admin/fixtures/sync (trigger). */

import instance from './axios-instance';

export interface FixtureTableSummary {
    table_name: string;
    record_count: number;
}

export interface FixtureStatusResponse {
    tracked_records: number;
    tables: FixtureTableSummary[];
}

export interface FixtureSyncResult {
    inserted: number;
    updated: number;
    deleted: number;
    skipped: number;
    errors: string[];
    summary: string;
}

/** Obtiene el estado actual de los fixtures rastreados en BD */
export async function apiGetFixtureStatus(): Promise<FixtureStatusResponse> {
    const { data } = await instance.get<FixtureStatusResponse>('/api/admin/fixtures');
    return data;
}

/** Dispara una sincronización completa de los archivos TOML de content/ con la BD */
export async function apiTriggerFixtureSync(): Promise<FixtureSyncResult> {
    const { data } = await instance.post<FixtureSyncResult>('/api/admin/fixtures/sync');
    return data;
}
