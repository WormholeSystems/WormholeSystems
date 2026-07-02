import type { Vec2 } from '@/map/core/types';
import type { MapStore } from '@/map/store/mapStore';
import { isWormholeClass } from '@/const/solarsystemClasses';
import type { TMapSolarsystem } from '@/pages/maps';
import MapConnections from '@/routes/map-connections';
import { router } from '@inertiajs/vue3';
import { ref, type Ref } from 'vue';
import type { Gesture, GestureContext } from './gestures';

export type LinkDrag = {
    gesture: Gesture;
    /** The live pointer position in base units while a link is being drawn. */
    pendingTo: Ref<Vec2 | null>;
};

/**
 * Drag-to-connect from a node's connection handle. Claims the pointer as soon
 * as the handle is pressed (the old useNewConnection set its origin on the
 * handle's raw pointerdown, so hysteresis stays at 0), feeds the ghost edge's
 * endpoint through pendingTo, and on release resolves the drop target through
 * the data-node-id hook under the pointer — the equivalent of the old
 * per-node container pointerup listeners.
 */
export function createLinkDragGesture(store: MapStore): LinkDrag {
    const pendingTo = ref<Vec2 | null>(null);
    let originId: number | null = null;

    function clear(): void {
        originId = null;
        pendingTo.value = null;
        store.linkDragOriginId.value = null;
    }

    const gesture: Gesture = {
        kind: 'link-drag',
        hysteresis: 0,

        onStart(ctx: GestureContext): boolean {
            if (!ctx.hitConnectHandle || ctx.nodeId === null) return false;
            if (!store.systems.has(ctx.nodeId)) return false;

            originId = ctx.nodeId;
            store.linkDragOriginId.value = ctx.nodeId;
            pendingTo.value = ctx.basePoint;
            return true;
        },

        onMove(ctx: GestureContext): void {
            pendingTo.value = ctx.basePoint;
        },

        onEnd(ctx: GestureContext): void {
            const from = originId === null ? null : (store.systems.get(originId) ?? null);
            const dropId = resolveDropNodeId(ctx.event);
            const to = dropId === null ? null : (store.systems.get(dropId) ?? null);
            clear();

            if (!from || !to || from.id === to.id) return;

            router.post(
                MapConnections.store().url,
                {
                    from_map_solarsystem_id: from.id,
                    to_map_solarsystem_id: to.id,
                    ship_size: getMaximumShipSizeForConnection(from, to),
                },
                {
                    preserveState: true,
                    preserveScroll: true,
                    only: ['map', 'selected_map_solarsystem', 'eve_scout_connections'],
                },
            );
        },

        onCancel(): void {
            clear();
        },
    };

    return { gesture, pendingTo };
}

/** The node the pointer was released over, via the topmost element under it. */
function resolveDropNodeId(event: PointerEvent): number | null {
    const element = document.elementFromPoint(event.clientX, event.clientY);
    const node = element?.closest('[data-node-id]');
    if (!(node instanceof HTMLElement) || node.dataset.nodeId === undefined) {
        return null;
    }
    const id = Number(node.dataset.nodeId);
    return Number.isFinite(id) ? id : null;
}

/**
 * The default ship-size restriction for a new connection, ported verbatim from
 * the old useNewConnection: C1s and Turnur/Thera edge cases cap at medium,
 * C13s are frigate holes, everything else stays unset (large).
 */
export function getMaximumShipSizeForConnection(from: TMapSolarsystem, to: TMapSolarsystem): string | undefined {
    const classes = [from.solarsystem.class, to.solarsystem.class].filter(isWormholeClass);
    if (classes.includes('1')) return 'medium';

    // Check if Turnur connects to JSpace
    const names = [from.solarsystem.name, to.solarsystem.name];
    if (names.includes('Turnur') && classes.length) return 'medium';

    // Check if Thera connects to Highsec
    const highsec = [from.solarsystem?.security, to.solarsystem?.security].filter((security) => security && security >= 0.5);
    if (names.includes('Thera') && highsec.length) return 'medium';

    // Check if it involves a frigate-only system
    if (classes.includes('13')) return 'frigate';

    return undefined;
}
