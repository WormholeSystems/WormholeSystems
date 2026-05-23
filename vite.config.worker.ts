import path from 'path';
import { defineConfig } from 'vite';

export default defineConfig({
    build: {
        outDir: 'public/workers',
        emptyOutDir: true,
        copyPublicDir: false,
        sourcemap: true,
        target: 'es2022',
        lib: {
            entry: path.resolve(__dirname, 'resources/js/routing/routing.worker.ts'),
            formats: ['es'],
            fileName: () => 'routing.worker.js',
        },
        rollupOptions: {
            output: {
                codeSplitting: false,
            },
        },
    },
    resolve: {
        alias: {
            '@': path.resolve(__dirname, './resources/js'),
        },
    },
});
