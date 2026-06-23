import { MaybeRefOrGetter, ref, toValue } from 'vue';
import { is_map_dragging } from '../dragState';
import { is_layout_locked } from './useMapViewMode';

export function useMapPanning(container: MaybeRefOrGetter<HTMLElement | null>) {
    const is_panning = ref(false);
    const start_x = ref(0);
    const start_y = ref(0);
    const scroll_left = ref(0);
    const scroll_top = ref(0);

    function handleMouseDown(event: MouseEvent) {
        const containerElement = toValue(container);
        if (!containerElement) return;

        // Never hijack an in-progress drag (moving a node, drawing a new connection).
        if (is_map_dragging.value) return;

        // Middle mouse always pans; in the locked tree layout a left drag pans too,
        // unless a selection modifier (Shift/Ctrl/Cmd) is held, which box-selects.
        const wants_marquee = event.shiftKey || event.ctrlKey || event.metaKey;
        const can_pan = event.button === 1 || (event.button === 0 && is_layout_locked.value && !wants_marquee);
        if (!can_pan) return;

        event.preventDefault();
        is_panning.value = true;
        start_x.value = event.clientX;
        start_y.value = event.clientY;
        scroll_left.value = containerElement.scrollLeft;
        scroll_top.value = containerElement.scrollTop;

        // Change cursor to indicate panning mode
        containerElement.style.cursor = 'grabbing';
    }

    function handleMouseMove(event: MouseEvent) {
        if (!is_panning.value) return;

        const containerElement = toValue(container);
        if (!containerElement) return;

        event.preventDefault();

        // Calculate the distance moved
        const dx = event.clientX - start_x.value;
        const dy = event.clientY - start_y.value;

        // Update scroll position (subtract because we want to pan in the opposite direction of mouse movement)
        containerElement.scrollLeft = scroll_left.value - dx;
        containerElement.scrollTop = scroll_top.value - dy;
    }

    function handleMouseUp() {
        if (!is_panning.value) return;

        const containerElement = toValue(container);
        if (!containerElement) return;

        is_panning.value = false;
        containerElement.style.cursor = '';
    }

    function handleMouseLeave() {
        if (!is_panning.value) return;

        const containerElement = toValue(container);
        if (!containerElement) return;

        is_panning.value = false;
        containerElement.style.cursor = '';
    }

    // Prevent default context menu on middle click
    function handleContextMenu(event: MouseEvent) {
        if (event.button === 1) {
            event.preventDefault();
        }
    }

    return {
        is_panning,
        handleMouseDown,
        handleMouseMove,
        handleMouseUp,
        handleMouseLeave,
        handleContextMenu,
    };
}
