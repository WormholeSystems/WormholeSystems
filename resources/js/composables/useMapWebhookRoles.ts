import MapWebhookRoleController from '@/actions/App/Http/Controllers/MapWebhookRoleController';
import { AppPageProps } from '@/types';
import { TMapWebhookRole } from '@/types/models';
import { VisitHelperOptions } from '@inertiajs/core';
import { router, usePage } from '@inertiajs/vue3';
import { computed } from 'vue';

export type TMapWebhookRolePayload = {
    name: string;
    discord_role_id: string;
};

const only = ['webhooks', 'roles', 'alerts'];

/**
 * Manager-only CRUD for a map's reusable Discord roles.
 */
export function useMapWebhookRoles() {
    const page = usePage<AppPageProps<{ map: { id: number; slug: string }; roles?: TMapWebhookRole[] }>>();

    const roles = computed(() => page.props.roles ?? []);

    function createRole(payload: TMapWebhookRolePayload, options: VisitHelperOptions = {}): void {
        router.visit(MapWebhookRoleController.store(), {
            data: { map_id: page.props.map.id, ...payload },
            preserveScroll: true,
            preserveState: true,
            only,
            ...options,
        });
    }

    function updateRole(id: number, payload: TMapWebhookRolePayload, options: VisitHelperOptions = {}): void {
        router.visit(MapWebhookRoleController.update(id), { data: payload, preserveScroll: true, preserveState: true, only, ...options });
    }

    function deleteRole(id: number, options: VisitHelperOptions = {}): void {
        router.visit(MapWebhookRoleController.destroy(id), { preserveScroll: true, preserveState: true, only, ...options });
    }

    return { roles, createRole, updateRole, deleteRole };
}
