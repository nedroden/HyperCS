import { getJson } from '@/shared/api';

export type Health = { status: 'ok'; version: string; database: 'up' | 'down' };

export const getHealth = () => getJson<Health>('/health');
