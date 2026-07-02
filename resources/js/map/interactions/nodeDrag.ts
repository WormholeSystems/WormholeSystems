import { getSelectedMapSolarsystems } from '@/map/actions/selectedMapSolarsystems';
import { updateMapSelection } from '@/map/actions/updateMapSelection';
import { clampToCanvas, snapToGrid } from '@/map/core/coords';
import type { Vec2 } from '@/map/core/types';
import type { MapStore } from '@/map/store/mapStore';
import type { TShowMapProps } from '@/pages/maps';
import MapSolarsystems from '@/routes/map-solarsystems';
import type { AppPageProps } from '@/types';
import { router, usePage } from '@inertiajs/vue3';
import type { Gesture, GestureContext } from './gestures';

/**
 * Dragging a node (or the selection it belongs to) by its drag handle.
 *
 * Matches the old useDraggable wiring: dragging starts ONLY from the dedicated
 * drag handle (the old code passed SolarsystemDragHandle as the `handle`
 * option), never from the card body, and is disabled for pinned systems and
 * locked (tree) layouts. When the grabbed node is part of the effective
 * selection (selected minus pinned minus home — the old
 * map_solarsystems_selected), the whole group moves; otherwise just the node
 * moves and any lingering selection is dropped, exactly like the old
 * setSystemPosition did on the first move.
 *
 * The commit mirrors the old updateMapSolarsystem branching: a single node
 * PUTs MapSolarsystems.update with suppress_notification, a group PUTs
 * MapSelection.update and clears the selection on success.
 */
export function createNodeDragGesture(store: MapStore): Gesture {
    const page = usePage<AppPageProps<TShowMapProps>>();

    let draggedId: number | null = null;
    let isGroupDrag = false;
    let moved = false;
    let startPoint: Vec2 = { x: 0, y: 0 };
    let startPositions = new Map<number, Vec2>();

    function resetState(): void {
        draggedId = null;
        isGroupDrag = false;
        moved = false;
        startPositions = new Map();
    }

    function unlockAll(): void {
        for (const id of startPositions.keys()) {
            store.unlockPosition(id);
        }
    }

    function commit(): void {
        if (draggedId === null) return;

        if (isGroupDrag) {
            updateMapSelection(
                [...startPositions.keys()].flatMap((id) => {
                    const position = store.positions.get(id);
                    return position ? [{ id, position_x: Math.round(position.x), position_y: Math.round(position.y) }] : [];
                }),
            );
            return;
        }

        const position = store.positions.get(draggedId);
        if (!position) return;

        // The dedicated updateMapSolarsystem action doesn't carry
        // suppress_notification, so the drag commit issues the PUT itself with
        // the same only/error semantics.
        const isDetailPanelTarget = page.props.selected_map_solarsystem?.id === draggedId;
        router.put(
            MapSolarsystems.update(draggedId).url,
            {
                position_x: Math.round(position.x),
                position_y: Math.round(position.y),
                suppress_notification: true,
            },
            {
                preserveState: true,
                preserveScroll: true,
                only: isDetailPanelTarget ? ['selected_map_solarsystem'] : ['errors'],
                onError: () => router.reload({ only: ['map'] }),
            },
        );
    }

    return {
        kind: 'node-drag',
        hysteresis: 4,

        onStart(ctx: GestureContext): boolean {
            if (ctx.event.button !== 0) return false;
            if (ctx.nodeId === null || !ctx.hitDragHandle) return false;
            if (store.isLayoutLocked.value) return false;

            const system = store.systems.get(ctx.nodeId);
            if (!system || system.pinned) return false;
            if (!store.positions.has(ctx.nodeId)) return false;

            draggedId = ctx.nodeId;
            startPoint = ctx.basePoint;
            moved = false;

            const selected = getSelectedMapSolarsystems(store);
            isGroupDrag = selected.some((candidate) => candidate.id === ctx.nodeId);
            const members = isGroupDrag ? selected : [system];

            startPositions = new Map();
            for (const member of members) {
                const position = store.positions.get(member.id);
                if (position) {
                    startPositions.set(member.id, { x: position.x, y: position.y });
                }
            }
            return true;
        },

        onMove(ctx: GestureContext): void {
            if (draggedId === null) return;

            if (!moved) {
                moved = true;
                for (const id of startPositions.keys()) {
                    store.lockPosition(id);
                }
                if (!isGroupDrag) {
                    store.clearSelection();
                }
            }

            const delta = { x: ctx.basePoint.x - startPoint.x, y: ctx.basePoint.y - startPoint.y };
            const { max_size, grid_size } = store.config.value;
            for (const [id, start] of startPositions) {
                store.moveSystem(id, clampToCanvas(snapToGrid({ x: start.x + delta.x, y: start.y + delta.y }, grid_size), max_size, grid_size));
            }
        },

        onEnd(): void {
            unlockAll();
            commit();
            resetState();
        },

        onCancel(): void {
            for (const [id, start] of startPositions) {
                store.moveSystem(id, start);
            }
            unlockAll();
            resetState();
        },
    };
}
