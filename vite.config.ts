import { wayfinder } from '@laravel/vite-plugin-wayfinder';
import tailwindcss from '@tailwindcss/vite';
import vue from '@vitejs/plugin-vue';
import laravel from 'laravel-vite-plugin';
import path from 'path';
import { defineConfig } from 'vite';
import { VitePWA } from 'vite-plugin-pwa';

export default defineConfig({
    plugins: [
        laravel({
            input: ['resources/js/app.ts'],
            ssr: 'resources/js/ssr.ts',
            refresh: true,
        }),
        tailwindcss(),
        vue({
            template: {
                transformAssetUrls: {
                    base: null,
                    includeAbsolute: false,
                },
            },
        }),
        wayfinder(),
        VitePWA({
            registerType: 'autoUpdate',
            workbox: {
                globPatterns: ['**/*.{js,css,html,ico,png,svg}'],
            },
            manifest: {
                name: 'wormhole.systems',
                short_name: 'wormhole.systems',
                description:
                    'Advanced wormhole mapping and tracking system for EVE Online. Navigate dangerous wormhole space with real-time intel, signature tracking, and collaborative mapping tools.',
                start_url: '/maps',
                display: 'standalone',
                background_color: '#000000',
                theme_color: '#000000',
                orientation: 'any',
                scope: '/',
                lang: 'en',
                categories: ['games', 'utilities', 'productivity'],
                id: 'com.wormhole.systems',
                launch_handler: {
                    client_mode: 'navigate-existing',
                },
                screenshots: [
                    {
                        src: '/img/features/map.png',
                        sizes: '1280x800',
                        type: 'image/png',
                        form_factor: 'wide',
                        label: 'Interactive wormhole mapping interface',
                    },
                    {
                        src: '/img/features/signatures.png',
                        sizes: '640x800',
                        type: 'image/png',
                        form_factor: 'narrow',
                        label: 'Signature tracking and management',
                    },
                ],
                icons: [
                    { src: '/icons/icon-72x72.png', sizes: '72x72', type: 'image/png', purpose: 'any' },
                    { src: '/icons/icon-96x96.png', sizes: '96x96', type: 'image/png', purpose: 'any' },
                    { src: '/icons/icon-128x128.png', sizes: '128x128', type: 'image/png', purpose: 'any' },
                    { src: '/icons/icon-144x144.png', sizes: '144x144', type: 'image/png', purpose: 'any' },
                    { src: '/icons/icon-152x152.png', sizes: '152x152', type: 'image/png', purpose: 'any' },
                    { src: '/icons/icon-192x192.png', sizes: '192x192', type: 'image/png', purpose: 'any' },
                    { src: '/icons/icon-384x384.png', sizes: '384x384', type: 'image/png', purpose: 'any' },
                    { src: '/icons/icon-512x512.png', sizes: '512x512', type: 'image/png', purpose: 'any' },
                    {
                        src: '/icons/maskable-icon-192x192.png',
                        sizes: '192x192',
                        type: 'image/png',
                        purpose: 'maskable',
                    },
                    {
                        src: '/icons/maskable-icon-512x512.png',
                        sizes: '512x512',
                        type: 'image/png',
                        purpose: 'maskable',
                    },
                ],
            },
        }),
    ],
    resolve: {
        alias: {
            '@': path.resolve(__dirname, './resources/js'),
        },
    },
});
