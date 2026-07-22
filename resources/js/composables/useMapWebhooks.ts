import MapWebhookController from '@/actions/App/Http/Controllers/MapWebhookController';
import { AppPageProps } from '@/types';
import { TMapWebhook } from '@/types/models';
import { VisitHelperOptions } from '@inertiajs/core';
import { router, usePage } from '@inertiajs/vue3';
import { computed } from 'vue';

export type TMapWebhookPayload = {
    name: string;
    discord_webhook_url?: string | null;
};

const only = ['webhooks', 'roles', 'alerts'];

/**
 * Manager-only CRUD for a map's Discord webhook destinations (channel URLs).
 */
export function useMapWebhooks() {
    const page = usePage<AppPageProps<{ map: { id: number; slug: string }; webhooks?: TMapWebhook[] }>>();

    const webhooks = computed(() => page.props.webhooks ?? []);

    function createWebhook(payload: TMapWebhookPayload, options: VisitHelperOptions = {}): void {
        router.visit(MapWebhookController.store(), {
            data: { map_id: page.props.map.id, ...payload },
            preserveScroll: true,
            preserveState: true,
            only,
            ...options,
        });
    }

    function updateWebhook(id: number, payload: TMapWebhookPayload, options: VisitHelperOptions = {}): void {
        router.visit(MapWebhookController.update(id), { data: payload, preserveScroll: true, preserveState: true, only, ...options });
    }

    function deleteWebhook(id: number, options: VisitHelperOptions = {}): void {
        router.visit(MapWebhookController.destroy(id), { preserveScroll: true, preserveState: true, only, ...options });
    }

    return { webhooks, createWebhook, updateWebhook, deleteWebhook };
}
