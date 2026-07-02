import prettier from 'eslint-config-prettier';
import vue from 'eslint-plugin-vue';

import { defineConfigWithVueTs, vueTsConfigs } from '@vue/eslint-config-typescript';

export default defineConfigWithVueTs(
    vue.configs['flat/essential'],
    vueTsConfigs.recommended,
    {
        ignores: ['vendor', 'node_modules', 'public', 'bootstrap/ssr', 'tailwind.config.js', 'resources/js/components/ui/*'],
    },
    {
        rules: {
            'vue/multi-word-component-names': 'off',
            '@typescript-eslint/no-explicit-any': 'off',
            '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_', varsIgnorePattern: '^_' }],
            'no-console': 'error',
        },
    },
    // Map subsystem layering. Code outside resources/js/map may only touch the public
    // surface: the '@/map/api' facade and the two mountable root components.
    {
        files: ['resources/js/**/*.{ts,vue}'],
        ignores: ['resources/js/map/**'],
        rules: {
            'no-restricted-imports': [
                'error',
                {
                    patterns: [
                        {
                            group: [
                                '@/map/*',
                                '@/map/**',
                                '!@/map/api',
                                '!@/map/components',
                                '@/map/components/**',
                                '!@/map/components/MapRoot.vue',
                                '!@/map/components/MapReadonly.vue',
                            ],
                            message:
                                "Outside the map subsystem, import only '@/map/api' or mount '@/map/components/MapRoot.vue' / '@/map/components/MapReadonly.vue'.",
                        },
                    ],
                },
            ],
        },
    },
    // map/core is pure domain logic: framework-free and unaware of the outer layers.
    {
        files: ['resources/js/map/core/**/*.ts'],
        rules: {
            'no-restricted-imports': [
                'error',
                {
                    paths: [{ name: 'vue', message: 'map/core is framework-free; keep Vue out of the domain layer.' }],
                    patterns: [
                        {
                            group: [
                                '@/map/store',
                                '@/map/store/**',
                                '@/map/interactions',
                                '@/map/interactions/**',
                                '@/map/components',
                                '@/map/components/**',
                                '@/map/sync',
                                '@/map/sync/**',
                            ],
                            message: 'map/core must not depend on the store, interactions, components or sync layers.',
                        },
                    ],
                },
            ],
        },
    },
    // map/store holds state; it may use core but not the layers built on top of it.
    {
        files: ['resources/js/map/store/**/*.ts'],
        rules: {
            'no-restricted-imports': [
                'error',
                {
                    patterns: [
                        {
                            group: [
                                '@/map/interactions',
                                '@/map/interactions/**',
                                '@/map/components',
                                '@/map/components/**',
                                '@/map/sync',
                                '@/map/sync/**',
                                '@/map/actions',
                                '@/map/actions/**',
                            ],
                            message: 'map/store must not depend on the interactions, components, sync or actions layers.',
                        },
                    ],
                },
            ],
        },
    },
    prettier,
);
