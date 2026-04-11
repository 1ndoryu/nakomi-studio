/* [114A-12] API client para configuración del sistema.
 * Toggle de rotación de API keys — solo admin. */

import axiosInstance from './axios-instance';

export interface RotacionStatus {
  enabled: boolean;
  total_keys: number;
  current_index: number;
  model: string;
  has_fallback: boolean;
}

export interface ToggleRotacionBody {
  enabled: boolean;
}

/** Obtener estado actual de la rotación de API keys */
export async function apiGetRotacionStatus(): Promise<RotacionStatus> {
  const { data } = await axiosInstance.get<RotacionStatus>('/api/admin/configuracion/rotacion');
  return data;
}

/** Activar o desactivar rotación de API keys */
export async function apiToggleRotacion(body: ToggleRotacionBody): Promise<RotacionStatus> {
  const { data } = await axiosInstance.patch<RotacionStatus>('/api/admin/configuracion/rotacion', body);
  return data;
}
