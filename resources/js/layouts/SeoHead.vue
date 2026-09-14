<script setup lang="ts">
import type { AppPageProps } from '@/types';
import { Head, usePage } from '@inertiajs/vue3';
import { computed } from 'vue';

interface SeoHeadProps {
    title?: string;
    description?: string;
    keywords?: string;
    url?: string;
    image?: {
        url: string;
        width?: number;
        height?: number;
        type?: string;
    };
    siteName?: string;
    twitterSite?: string;
    locale?: string;
    type?: string;
    themeColor?: string;
}

const props = withDefaults(defineProps<SeoHeadProps>(), {
    title: 'Wormhole Systems',
    description:
        'Advanced wormhole mapping and tracking system for EVE Online. Navigate dangerous wormhole space with real-time intel, signature tracking, and collaborative mapping tools.',
    keywords: 'EVE Online, wormhole, mapping, tracking, signatures, intel, navigation, space, gaming',
    locale: 'en_US',
    type: 'website',
    themeColor: '#ffffff',
});

const page = usePage<AppPageProps>();

const app_url = computed(() => page.props.app_url ?? '');

/** The bare host of this instance, e.g. "wormhole.systems". */
const app_host = computed(() => {
    try {
        return new URL(app_url.value).host;
    } catch {
        return app_url.value;
    }
});

const canonical_url = computed(() => props.url ?? app_url.value);
const site_name = computed(() => props.siteName ?? app_host.value);
const twitter_site = computed(() => props.twitterSite ?? app_host.value);

// Default image
const defaultImage = {
    url: '/img/og.png?v=1',
    width: 1024,
    height: 768,
    type: 'image/png',
};

const image = props.image || defaultImage;
</script>

<template>
    <Head>
        <!-- Basic Meta Tags -->
        <title>{{ title }}</title>
        <meta :content="description" name="description" />
        <meta :content="keywords" name="keywords" />
        <meta :content="themeColor" name="theme-color" />

        <!-- Open Graph Meta Tags -->
        <meta :content="title" property="og:title" />
        <meta :content="description" property="og:description" />
        <meta :content="canonical_url" property="og:url" />
        <meta :content="type" property="og:type" />
        <meta :content="site_name" property="og:site_name" />
        <meta :content="locale" property="og:locale" />
        <meta :content="image.url" property="og:image" />
        <meta v-if="image.type" :content="image.type" property="og:image:type" />
        <meta v-if="image.width" :content="image.width.toString()" property="og:image:width" />
        <meta v-if="image.height" :content="image.height.toString()" property="og:image:height" />

        <!-- Twitter Card Meta Tags -->
        <meta content="summary_large_image" name="twitter:card" />
        <meta :content="title" property="twitter:title" />
        <meta :content="description" property="twitter:description" />
        <meta :content="canonical_url" property="twitter:url" />
        <meta :content="image.url" property="twitter:image" />
        <meta :content="twitter_site" name="twitter:site" />
    </Head>
</template>
