import MapWebhookController from '@/actions/App/Http/Controllers/MapWebhookController';
import { AppPageProps } from '@/types';
import { TKillmailFilterMatch, TKillmailFilterRule, TMapWebhook, TMapWebhookType } from '@/types/models';
import { VisitHelperOptions } from '@inertiajs/core';
import { router, usePage } from '@inertiajs/vue3';
import { computed } from 'vue';

export type TMapWebhookPayload = {
    name: string;
    discord_webhook_url?: string | null;
    discord_role_id?: string | null;
    type: TMapWebhookType;
    target_solarsystem_id?: number | null;
    max_jumps: number;
    filter_match?: TKillmailFilterMatch;
    filters?: TKillmailFilterRule[];
    is_active: boolean;
};

/**
 * Manager-only CRUD for a map's Discord proximity-alert webhooks.
 */
export function useMapWebhooks() {
    const page = usePage<AppPageProps<{ map: { id: number; slug: string }; webhooks?: TMapWebhook[] }>>();

    const webhooks = computed(() => page.props.webhooks ?? []);

    function createWebhook(payload: TMapWebhookPayload, options: VisitHelperOptions = {}): void {
        router.post(
            MapWebhookController.store().url,
            { map_id: page.props.map.id, ...payload },
            {
                preserveScroll: true,
                preserveState: true,
                only: ['webhooks'],
                ...options,
            },
        );
    }

    function updateWebhook(id: number, payload: TMapWebhookPayload, options: VisitHelperOptions = {}): void {
        router.put(MapWebhookController.update(id).url, payload, {
            preserveScroll: true,
            preserveState: true,
            only: ['webhooks'],
            ...options,
        });
    }

    function deleteWebhook(id: number, options: VisitHelperOptions = {}): void {
        router.delete(MapWebhookController.destroy(id).url, {
            preserveScroll: true,
            preserveState: true,
            only: ['webhooks'],
            ...options,
        });
    }

    return {
        webhooks,
        createWebhook,
        updateWebhook,
        deleteWebhook,
    };
}
