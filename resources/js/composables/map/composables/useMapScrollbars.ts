import { useEventListener, useThrottleFn } from '@vueuse/core';
import { computed, MaybeRefOrGetter, onBeforeUnmount, ref, toValue, watch } from 'vue';
import { mapState } from '../state';

const SCROLLBAR_SIZE = 8;
const MIN_THUMB_SIZE = 30;
const AUTO_HIDE_DELAY = 1500;
const MOUSEMOVE_THROTTLE = 300;

export function useMapScrollbars(container: MaybeRefOrGetter<HTMLElement | null>) {
    const scroll_left = ref(0);
    const scroll_top = ref(0);
    const viewport_width = ref(0);
    const viewport_height = ref(0);
    const content_width = ref(0);
    const content_height = ref(0);
    const scrollbars_visible = ref(false);
    const is_dragging = ref(false);

    let hide_timeout: ReturnType<typeof setTimeout> | null = null;

    function measureDimensions() {
        const el = toValue(container);
        if (!el) return;

        viewport_width.value = el.clientWidth;
        viewport_height.value = el.clientHeight;
        content_width.value = el.scrollWidth;
        content_height.value = el.scrollHeight;
    }

    function syncScroll() {
        const el = toValue(container);
        if (!el) return;

        scroll_left.value = el.scrollLeft;
        scroll_top.value = el.scrollTop;
        measureDimensions();
        showScrollbars();
    }

    // --- Visibility ---

    function showScrollbars() {
        scrollbars_visible.value = true;
        resetHideTimeout();
    }

    function resetHideTimeout() {
        if (hide_timeout) {
            clearTimeout(hide_timeout);
        }

        if (is_dragging.value) return;

        hide_timeout = setTimeout(() => {
            scrollbars_visible.value = false;
            hide_timeout = null;
        }, AUTO_HIDE_DELAY);
    }

    // --- Computed thumb dimensions ---

    const max_scroll_x = computed(() => Math.max(0, content_width.value - viewport_width.value));
    const max_scroll_y = computed(() => Math.max(0, content_height.value - viewport_height.value));

    const has_horizontal = computed(() => content_width.value > viewport_width.value);
    const has_vertical = computed(() => content_height.value > viewport_height.value);

    const v_track_height = computed(() => viewport_height.value - (has_horizontal.value ? SCROLLBAR_SIZE : 0));
    const h_track_width = computed(() => viewport_width.value - (has_vertical.value ? SCROLLBAR_SIZE : 0));

    const v_thumb_size = computed(() => {
        if (!has_vertical.value || v_track_height.value <= 0) return 0;
        return Math.max(MIN_THUMB_SIZE, (viewport_height.value / content_height.value) * v_track_height.value);
    });

    const h_thumb_size = computed(() => {
        if (!has_horizontal.value || h_track_width.value <= 0) return 0;
        return Math.max(MIN_THUMB_SIZE, (viewport_width.value / content_width.value) * h_track_width.value);
    });

    const v_thumb_offset = computed(() => {
        if (max_scroll_y.value <= 0) return 0;
        return (scroll_top.value / max_scroll_y.value) * (v_track_height.value - v_thumb_size.value);
    });

    const h_thumb_offset = computed(() => {
        if (max_scroll_x.value <= 0) return 0;
        return (scroll_left.value / max_scroll_x.value) * (h_track_width.value - h_thumb_size.value);
    });

    // --- Wheel handling ---

    function onWheel(event: WheelEvent) {
        if (event.ctrlKey || event.metaKey) {
            event.preventDefault();
            return;
        }

        // Shift+wheel → scroll the map (vertical by default, horizontal with deltaX)
        if (event.shiftKey) {
            event.preventDefault();
            const el = toValue(container);
            if (!el) return;
            el.scrollTop += event.deltaY;
            el.scrollLeft += event.deltaX;
            return;
        }

        // No modifier → let event bubble to page scroll (overflow: hidden prevents map scroll)
    }

    // --- Thumb drag ---

    let drag_axis: 'vertical' | 'horizontal' | null = null;
    let drag_start_mouse = 0;
    let drag_start_scroll = 0;

    function onThumbMousedown(axis: 'vertical' | 'horizontal', event: MouseEvent) {
        event.preventDefault();
        event.stopPropagation();

        is_dragging.value = true;
        drag_axis = axis;

        if (axis === 'vertical') {
            drag_start_mouse = event.clientY;
            drag_start_scroll = scroll_top.value;
        } else {
            drag_start_mouse = event.clientX;
            drag_start_scroll = scroll_left.value;
        }

        document.body.style.userSelect = 'none';
        document.addEventListener('mousemove', onDragMove);
        document.addEventListener('mouseup', onDragEnd);
    }

    function onDragMove(event: MouseEvent) {
        const el = toValue(container);
        if (!el || !drag_axis) return;

        if (drag_axis === 'vertical') {
            const delta_mouse = event.clientY - drag_start_mouse;
            const scroll_range = v_track_height.value - v_thumb_size.value;
            if (scroll_range <= 0) return;
            const scroll_ratio = delta_mouse / scroll_range;
            el.scrollTop = drag_start_scroll + scroll_ratio * max_scroll_y.value;
        } else {
            const delta_mouse = event.clientX - drag_start_mouse;
            const scroll_range = h_track_width.value - h_thumb_size.value;
            if (scroll_range <= 0) return;
            const scroll_ratio = delta_mouse / scroll_range;
            el.scrollLeft = drag_start_scroll + scroll_ratio * max_scroll_x.value;
        }
    }

    function onDragEnd() {
        is_dragging.value = false;
        drag_axis = null;
        document.body.style.userSelect = '';
        document.removeEventListener('mousemove', onDragMove);
        document.removeEventListener('mouseup', onDragEnd);
        resetHideTimeout();
    }

    // --- Track click ---

    function onTrackMousedown(axis: 'vertical' | 'horizontal', event: MouseEvent) {
        event.preventDefault();
        event.stopPropagation();

        const el = toValue(container);
        if (!el) return;

        const rect = el.getBoundingClientRect();

        if (axis === 'vertical') {
            const click_y = event.clientY - rect.top;
            const ratio = click_y / v_track_height.value;
            el.scrollTop = ratio * max_scroll_y.value - viewport_height.value / 2;
        } else {
            const click_x = event.clientX - rect.left;
            const ratio = click_x / h_track_width.value;
            el.scrollLeft = ratio * max_scroll_x.value - viewport_width.value / 2;
        }
    }

    // --- Scroll area hover ---

    function onScrollAreaEnter() {
        showScrollbars();
    }

    const onScrollAreaMousemove = useThrottleFn(() => {
        if (scrollbars_visible.value) {
            resetHideTimeout();
        } else {
            showScrollbars();
        }
    }, MOUSEMOVE_THROTTLE);

    // --- Resize observer ---

    let resize_observer: ResizeObserver | null = null;

    function setupResizeObserver() {
        const el = toValue(container);
        if (!el) return;

        resize_observer = new ResizeObserver(() => {
            measureDimensions();
        });

        resize_observer.observe(el);

        // Also observe the first child (content) for size changes
        if (el.firstElementChild) {
            resize_observer.observe(el.firstElementChild);
        }
    }

    function teardownResizeObserver() {
        if (resize_observer) {
            resize_observer.disconnect();
            resize_observer = null;
        }
    }

    // Watch for scale changes (content size changes)
    watch(
        () => mapState.scale,
        () => {
            requestAnimationFrame(() => {
                measureDimensions();
            });
        },
    );

    // Watch for container being available
    watch(
        () => toValue(container),
        (el) => {
            teardownResizeObserver();
            if (el) {
                measureDimensions();
                setupResizeObserver();
            }
        },
        { immediate: true },
    );

    // Listen to native scroll event
    useEventListener(container, 'scroll', syncScroll, { passive: true });

    // Listen to wheel event on the container
    useEventListener(container, 'wheel', onWheel, { passive: false });

    onBeforeUnmount(() => {
        teardownResizeObserver();
        if (hide_timeout) {
            clearTimeout(hide_timeout);
        }
        document.removeEventListener('mousemove', onDragMove);
        document.removeEventListener('mouseup', onDragEnd);
    });

    return {
        scrollbars_visible,
        has_vertical,
        has_horizontal,
        v_thumb_size,
        v_thumb_offset,
        v_track_height,
        h_thumb_size,
        h_thumb_offset,
        h_track_width,
        scrollbar_size: SCROLLBAR_SIZE,
        onThumbMousedown,
        onTrackMousedown,
        onScrollAreaEnter,
        onScrollAreaMousemove,
    };
}
