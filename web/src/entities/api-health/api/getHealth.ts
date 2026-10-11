import { api } from '@/shared/api';

export async function getHealth() {
    const { data, error } = await api.GET('/api/v1/health');
    if (error || !data) {
        throw new Error('health check failed');
    }
    return data;
}
