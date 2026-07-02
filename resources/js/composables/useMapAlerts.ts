import MapAlertController from '@/actions/App/Http/Controllers/MapAlertController';
import { AppPageProps } from '@/types';
import { TJumpShipType, TKillmailFilterMatch, TKillmailFilterRule, TMapAlert, TMapWebhookType } from '@/types/models';
import { VisitHelperOptions } from '@inertiajs/core';
import { router, usePage } from '@inertiajs/vue3';
import { computed } from 'vue';

export type TMapAlertPayload = {
    map_webhook_id: number;
    map_webhook_role_id: number | null;
    type: TMapWebhookType;
    target_solarsystem_id?: number | null;
    ship_type?: TJumpShipType | null;
    jdc_level?: number | null;
    include_highsec?: boolean;
    max_jumps: number | null;
    filter_match?: TKillmailFilterMatch;
    filters?: TKillmailFilterRule[];
    is_active: boolean;
};

const only = ['webhooks', 'roles', 'alerts'];

/**
 * Manager-only CRUD for a map's alerts, each pointing at a webhook and optional role.
 */
export function useMapAlerts() {
    const page = usePage<AppPageProps<{ map: { id: number; slug: string }; alerts?: TMapAlert[] }>>();

    const alerts = computed(() => page.props.alerts ?? []);

    function createAlert(payload: TMapAlertPayload, options: VisitHelperOptions = {}): void {
        router.post(
            MapAlertController.store().url,
            { map_id: page.props.map.id, ...payload },
            { preserveScroll: true, preserveState: true, only, ...options },
        );
    }

    function updateAlert(id: number, payload: TMapAlertPayload, options: VisitHelperOptions = {}): void {
        router.put(MapAlertController.update(id).url, payload, { preserveScroll: true, preserveState: true, only, ...options });
    }

    function deleteAlert(id: number, options: VisitHelperOptions = {}): void {
        router.delete(MapAlertController.destroy(id).url, { preserveScroll: true, preserveState: true, only, ...options });
    }

    return { alerts, createAlert, updateAlert, deleteAlert };
}
