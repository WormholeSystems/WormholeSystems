<script setup lang="ts">
type Position = {
    x: number;
    y: number;
};

type Props = {
    from: Position;
    to: Position;
};

const { from, to } = defineProps<Props>();

const emit = defineEmits<{
    (e: 'click', event: MouseEvent): void;
}>();
</script>

<template>
    <g pointer-events="visiblePainted" class="group">
        <path
            :d="`M ${from.x} ${from.y} L ${to.x} ${to.y}`"
            stroke="currentColor"
            fill="none"
            stroke-width="4"
            class="cursor-pointer transition-colors duration-200 ease-in-out group-hover:text-neutral-600"
        />
        <path
            :d="`M ${from.x} ${from.y} L ${to.x} ${to.y}`"
            stroke="transparent"
            fill="none"
            stroke-width="24"
            class="cursor-pointer transition-colors duration-200"
            @pointerdown.stop
            @click.stop="(event) => emit('click', event)"
        />
        <circle :cx="from.x" :cy="from.y" r="8" fill="currentColor" />
        <circle :cx="to.x" :cy="to.y" r="8" fill="currentColor" />
    </g>
</template>

<style scoped></style>
