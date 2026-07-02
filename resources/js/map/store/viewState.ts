import type { Vec2 } from '@/map/core/types';
import { TMapConfig } from '@/types/map';
import { ref, shallowRef, type Ref, type ShallowRef } from 'vue';

export type GestureKind = 'none' | 'pan' | 'marquee' | 'node-drag' | 'link-drag';

export type Marquee = { start: Vec2; end: Vec2 };

export type ViewState = ReturnType<typeof createViewState>;

/**
 * Interaction and viewport state. The selected set is replaced wholesale on every
 * change, so per-node `selectedIds.value.has(id)` computeds re-evaluate together
 * but only flip (and re-render) the nodes whose membership actually changed.
 */
export function createViewState() {
    const scale = ref(1);
    const config: ShallowRef<TMapConfig> = shallowRef({ max_size: { x: 4000, y: 2000 }, grid_size: 20 });

    const selectedIds: ShallowRef<ReadonlySet<number>> = shallowRef(new Set<number>());
    const hoveredSolarsystemId: Ref<number | null> = ref(null);
    /** The live marquee box in base units, or null when none is being drawn. */
    const marquee: ShallowRef<Marquee | null> = shallowRef(null);
    const activeGesture: Ref<GestureKind> = ref('none');
    const linkDragOriginId: Ref<number | null> = ref(null);
    /** The viewer's personal layout override (when the map allows it); null follows the map. */
    const userLayoutOverride: Ref<'manual' | 'tree' | null> = ref(null);

    function setSelection(ids: Iterable<number>): void {
        selectedIds.value = new Set(ids);
    }

    function clearSelection(): void {
        if (selectedIds.value.size === 0) return;
        selectedIds.value = new Set();
    }

    function pruneSelection(removedId: number): void {
        if (!selectedIds.value.has(removedId)) return;
        const next = new Set(selectedIds.value);
        next.delete(removedId);
        selectedIds.value = next;
    }

    function isSelected(id: number): boolean {
        return selectedIds.value.has(id);
    }

    function isHovered(id: number): boolean {
        return hoveredSolarsystemId.value === id;
    }

    return {
        scale,
        config,
        selectedIds,
        hoveredSolarsystemId,
        marquee,
        activeGesture,
        linkDragOriginId,
        userLayoutOverride,
        setSelection,
        clearSelection,
        pruneSelection,
        isSelected,
        isHovered,
    };
}
