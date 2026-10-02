import fsd from '@feature-sliced/steiger-plugin';
import { defineConfig } from 'steiger';

export default defineConfig([
    ...fsd.configs.recommended,
    {
        // The project is young: many slices have a single consumer for now.
        // Re-enable once the forum, auth and settings slices exist.
        files: ['./src/**'],
        rules: { 'fsd/insignificant-slice': 'off' },
    },
]);
