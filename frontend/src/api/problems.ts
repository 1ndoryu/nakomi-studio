/* [104A-28] API client para el sistema de problemas reportados en órdenes.
 * Endpoints: report, list (admin), list by order, resolve. */
import instance from './axios-instance';

/* TIPOS */

export type ProblemStatus = 'open' | 'in_review' | 'resolved' | 'dismissed';
export type ProblemAction = 'resolve' | 'dismiss';

export interface ProblemResponse {
    id: string;
    order_id: string;
    order_number: number;
    reporter_id: string;
    reporter_name: string;
    reporter_role: string;
    reason: string;
    status: ProblemStatus;
    admin_response: string | null;
    resolved_by: string | null;
    resolved_at: string | null;
    created_at: string;
}

export interface ReportProblemRequest {
    reason: string;
}

export interface ResolveProblemRequest {
    action: ProblemAction;
    response?: string;
}

/* LABELS Y COLORES */

export const PROBLEM_STATUS_LABELS: Record<ProblemStatus, string> = {
    open: 'Abierto',
    in_review: 'En revisión',
    resolved: 'Resuelto',
    dismissed: 'Descartado',
};

export const PROBLEM_STATUS_COLORS: Record<ProblemStatus, string> = {
    open: '#e74c3c',
    in_review: '#f39c12',
    resolved: '#27ae60',
    dismissed: '#95a5a6',
};

/* API CALLS */

export async function apiReportProblem(
    orderId: string,
    req: ReportProblemRequest,
): Promise<ProblemResponse> {
    const {data} = await instance.post<ProblemResponse>(
        `/api/orders/${orderId}/report-problem`,
        req,
    );
    return data;
}

export async function apiListProblems(): Promise<ProblemResponse[]> {
    const {data} = await instance.get<ProblemResponse[]>('/api/admin/problems');
    return data;
}

export async function apiListOrderProblems(orderId: string): Promise<ProblemResponse[]> {
    const {data} = await instance.get<ProblemResponse[]>(
        `/api/orders/${orderId}/problems`,
    );
    return data;
}

export async function apiResolveProblem(
    problemId: string,
    req: ResolveProblemRequest,
): Promise<ProblemResponse> {
    const {data} = await instance.patch<ProblemResponse>(
        `/api/admin/problems/${problemId}/resolve`,
        req,
    );
    return data;
}
