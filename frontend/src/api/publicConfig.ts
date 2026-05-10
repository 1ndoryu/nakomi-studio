import api from './axios-instance';

interface PublicConfigResponse {
    stripe_publishable_key?: string | null;
}

export async function apiGetPublicConfig(): Promise<PublicConfigResponse> {
    const {data} = await api.get<PublicConfigResponse>('/api/public-config');
    return data;
}
