/// <reference types="vitest/config" />
import react from '@vitejs/plugin-react';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vite';

const apiTarget = process.env.API_PROXY_TARGET ?? 'http://localhost:8080';

export default defineConfig({
    plugins: [react()],
    resolve: {
        alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) },
    },
    server: {
        host: true,
        port: 5173,
        proxy: { '/api': { target: apiTarget, changeOrigin: true } },
    },
    test: {
        environment: 'jsdom',
        setupFiles: ['./test/setup.ts'],
        globals: true,
    },
});
