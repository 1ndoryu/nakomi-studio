/* [044A-38 Fase 4] API client para asignación, delegación y gestión de empleados.
 * Conecta con /api/orders/unassigned, /api/orders/:id/take, /api/delegations, /api/admin/employees.
 * [T2-assignment] Añadido apiAssignOrder para asignación admin. */
import instance from './axios-instance';
import type {OrderResponse} from './orders';

/*    TIPOS */

export type DelegationStatus = 'requested' | 'accepted' | 'rejected' | 'completed';

export interface DelegationResponse {
    id: string;
    order_id: string;
    order_number: number;
    service_title: string;
    from_employee_id: string;
    to_employee_id: string | null;
    reason: string;
    delegation_type: string;
    status: DelegationStatus;
    created_at: string;
    resolved_at: string | null;
}

export interface EmployeeListItem {
    user_id: string;
    email: string;
    specialties: string[];
    availability: string;
    current_orders: number;
    max_concurrent_orders: number;
    total_completed_orders: number;
    average_rating: number | null;
}

export interface CreateDelegationRequest {
    reason: string;
}

export interface RespondDelegationRequest {
    accept: boolean;
}

/*    API CALLS */

export async function apiListUnassigned(): Promise<OrderResponse[]> {
    const {data} = await instance.get<OrderResponse[]>('/api/orders/unassigned');
    return data;
}

export async function apiTakeOrder(orderId: string): Promise<OrderResponse> {
    const {data} = await instance.post<OrderResponse>(`/api/orders/${orderId}/take`);
    return data;
}

export async function apiListEmployees(): Promise<EmployeeListItem[]> {
    const {data} = await instance.get<EmployeeListItem[]>('/api/admin/employees');
    return data;
}

/* [T2-assignment] PUT /api/orders/:orderId/assign/:employeeId — admin asigna orden a empleado */
export async function apiAssignOrder(orderId: string, employeeId: string): Promise<{ status: string; assigned_employee_id: string }> {
    const {data} = await instance.put<{ status: string; assigned_employee_id: string }>(
        `/api/orders/${orderId}/assign/${employeeId}`,
    );
    return data;
}

export async function apiCreateDelegation(
    orderId: string,
    req: CreateDelegationRequest,
): Promise<DelegationResponse> {
    const {data} = await instance.post<DelegationResponse>(
        `/api/orders/${orderId}/delegate`,
        req,
    );
    return data;
}

export async function apiCreateHelpRequest(
    orderId: string,
    req: CreateDelegationRequest,
): Promise<DelegationResponse> {
    const {data} = await instance.post<DelegationResponse>(
        `/api/orders/${orderId}/help`,
        req,
    );
    return data;
}

export async function apiRespondDelegation(
    delegationId: string,
    accept: boolean,
): Promise<DelegationResponse> {
    const {data} = await instance.patch<DelegationResponse>(
        `/api/delegations/${delegationId}`,
        {accept},
    );
    return data;
}

export async function apiListDelegations(): Promise<DelegationResponse[]> {
    const {data} = await instance.get<DelegationResponse[]>('/api/delegations');
    return data;
}

/*    HELPERS */

export const DELEGATION_STATUS_LABELS: Record<DelegationStatus, string> = {
    requested: 'Pendiente',
    accepted: 'Aceptada',
    rejected: 'Rechazada',
    completed: 'Completada',
};

export const DELEGATION_STATUS_CLASS: Record<DelegationStatus, string> = {
    requested: 'delegEstado--pendiente',
    accepted: 'delegEstado--aceptada',
    rejected: 'delegEstado--rechazada',
    completed: 'delegEstado--completada',
};

export const AVAILABILITY_LABELS: Record<string, string> = {
    available: 'Disponible',
    busy: 'Ocupado',
    away: 'Ausente',
    offline: 'Desconectado',
};
