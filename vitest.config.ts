// vitest.config.ts
import { defineConfig } from 'vitest/config';
// import { sveltekit } from '@sveltejs/kit/vite';
// import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
    plugins: [
        // tailwindcss(),
        // sveltekit()
    ],
    test: {
        projects: [
            {
                test: {
                    name: 'client',
                    environment: 'browser',
                    browser: {
                        enabled: true,
                        provider: 'playwright',
                        instances: [{ browser: 'chromium' }]
                    },
                    include: ['src/**/*.svelte.{test,spec}.{js,ts}'],
                    exclude: ['src/lib/server/**'],
                    setupFiles: ['./vitest-setup-client.ts']
                }
            },
            {
                test: {
                    name: 'server',
                    environment: 'node',
                    include: ['src/**/*.{test,spec}.{js,ts}'],
                    exclude: ['src/**/*.svelte.{test,spec}.{js,ts}']
                }
            }
        ]
    }
});