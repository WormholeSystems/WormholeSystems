<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, useTemplateRef } from 'vue';

const {
    to,
    suffix = '',
    decimals = 0,
    duration = 1600,
} = defineProps<{
    to: number;
    suffix?: string;
    decimals?: number;
    duration?: number;
}>();

const el = useTemplateRef<HTMLElement>('el');
const current = ref(0);

let raf = 0;
let observer: IntersectionObserver | null = null;
let started = false;

const formatter = new Intl.NumberFormat('en-US', { minimumFractionDigits: decimals, maximumFractionDigits: decimals });

// The final number is rendered invisibly to reserve an exact, fixed width, so
// the animating value is right-aligned inside it and the suffix never shifts.
const finalNumber = formatter.format(to);
const numberText = computed(() => formatter.format(current.value));

function run(): void {
    const start = performance.now();
    const easeOut = (t: number): number => 1 - Math.pow(1 - t, 3);
    const step = (now: number): void => {
        const progress = Math.min(1, (now - start) / duration);
        current.value = to * easeOut(progress);
        if (progress < 1) {
            raf = requestAnimationFrame(step);
        } else {
            current.value = to;
        }
    };
    raf = requestAnimationFrame(step);
}

onMounted(() => {
    if (typeof window === 'undefined') {
        current.value = to;
        return;
    }
    if (typeof IntersectionObserver === 'undefined' || window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
        current.value = to;
        return;
    }
    observer = new IntersectionObserver(
        (entries) => {
            for (const entry of entries) {
                if (entry.isIntersecting && !started) {
                    started = true;
                    run();
                    observer?.disconnect();
                }
            }
        },
        { threshold: 0.5 },
    );
    if (el.value) {
        observer.observe(el.value);
    }
});

onBeforeUnmount(() => {
    cancelAnimationFrame(raf);
    observer?.disconnect();
});
</script>

<template>
    <span ref="el" class="inline-flex items-baseline tabular-nums">
        <span class="relative inline-block text-right">
            <span class="invisible" aria-hidden="true">{{ finalNumber }}</span>
            <span class="absolute inset-0 text-right">{{ numberText }}</span>
        </span>
        <span v-if="suffix">{{ suffix }}</span>
    </span>
</template>
