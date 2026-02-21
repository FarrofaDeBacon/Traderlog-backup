
import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import path from 'path';

export default defineConfig({
    plugins: [svelte()],
    test: {
        environment: 'jsdom',
        globals: true,
        setupFiles: ['./src/setupTest.ts'],
        include: ['src/**/*.{test,spec}.{js,ts}'],
        alias: {
            $lib: path.resolve('./src/lib'),
        },
    },
});
