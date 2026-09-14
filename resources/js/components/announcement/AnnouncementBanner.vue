<script setup lang="ts">
import { setCookie } from '@/lib/utils';
import type { AppPageProps, TAnnouncementLevel } from '@/types';
import { usePage } from '@inertiajs/vue3';
import { AlertTriangle, ArrowRight, Info, OctagonAlert, X } from 'lucide-vue-next';
import { computed, ref, type Component } from 'vue';

const page = usePage<AppPageProps>();

const announcement = computed(() => page.props.announcement ?? null);

/**
 * The server already withholds a dismissed announcement, so this only covers
 * the current page until the cookie is sent with the next request.
 */
const dismissed_id = ref<string | null>(null);

const is_visible = computed(() => announcement.value !== null && dismissed_id.value !== announcement.value.id);

const level = computed<TAnnouncementLevel>(() => announcement.value?.level ?? 'info');

const icons: Record<TAnnouncementLevel, Component> = {
    info: Info,
    warning: AlertTriangle,
    critical: OctagonAlert,
};

const classes: Record<TAnnouncementLevel, string> = {
    info: 'border-sky-500/30 bg-sky-500/10 text-sky-950 dark:text-sky-100',
    warning: 'border-amber-500/30 bg-amber-500/10 text-amber-950 dark:text-amber-100',
    critical: 'border-red-500/40 bg-red-500/15 text-red-950 dark:text-red-100',
};

function dismiss(): void {
    if (!announcement.value) {
        return;
    }

    setCookie('announcement_dismissed', announcement.value.id);
    dismissed_id.value = announcement.value.id;
}
</script>

<template>
    <div v-if="announcement && is_visible" role="status" class="border-b px-4 py-2.5" :class="classes[level]">
        <div class="mx-auto flex max-w-7xl items-start gap-3 text-sm">
            <component :is="icons[level]" class="mt-0.5 size-4 shrink-0" />
            <div class="min-w-0 flex-1">
                <p v-if="announcement.title" class="font-semibold">{{ announcement.title }}</p>
                <p class="text-pretty opacity-90">{{ announcement.message }}</p>
                <a
                    v-if="announcement.link"
                    :href="announcement.link.url"
                    class="mt-1 inline-flex items-center gap-1 font-medium underline underline-offset-4 hover:no-underline"
                >
                    {{ announcement.link.label }}
                    <ArrowRight class="size-3.5" />
                </a>
            </div>
            <button
                v-if="announcement.dismissible"
                type="button"
                aria-label="Dismiss announcement"
                class="-m-1 shrink-0 rounded p-1 opacity-70 transition hover:opacity-100"
                @click="dismiss"
            >
                <X class="size-4" />
            </button>
        </div>
    </div>
</template>
