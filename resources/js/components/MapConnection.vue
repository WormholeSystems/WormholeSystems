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
    <path
        :d="`M ${from.x} ${from.y} L ${to.x} ${to.y}`"
        stroke="currentColor"
        fill="none"
        stroke-width="4"
        class="hover:text-neutral-500"
        @click="(event) => emit('click', event)"
    />
    <circle :cx="from.x" :cy="from.y" r="8" fill="currentColor" />
    <circle :cx="to.x" :cy="to.y" r="8" fill="currentColor" />
</template>

<style scoped></style>
