import { MaybeRefOrGetter, ref, toValue } from 'vue';

export function useMapPanning(container: MaybeRefOrGetter<HTMLElement | null>) {
    const is_panning = ref(false);
    const start_x = ref(0);
    const start_y = ref(0);
    const scroll_left = ref(0);
    const scroll_top = ref(0);

    function handleMouseDown(event: MouseEvent) {
        const containerElement = toValue(container);
        if (!containerElement) return;

        // Check if middle mouse button (button 1) is pressed
        if (event.button !== 1) return;

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

    function handleMouseUp(event: MouseEvent) {
        if (!is_panning.value) return;

        const containerElement = toValue(container);
        if (!containerElement) return;

        // Only handle middle mouse button release
        if (event.button !== 1) return;

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
