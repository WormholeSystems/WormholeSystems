<script setup lang="ts">
import type { TMaintainerEntry } from '@/types/models';

const { entries } = defineProps<{
    entries: TMaintainerEntry[];
}>();

const medals: Record<number, string> = { 1: '🥇', 2: '🥈', 3: '🥉' };

function medalFor(position: number): string {
    return medals[position] ?? `#${position}`;
}

function totals(entry: TMaintainerEntry): { added: number; edited: number; deleted: number } {
    return entry.characters.reduce(
        (sum, character) => ({
            added: sum.added + character.nb_added,
            edited: sum.edited + character.nb_edited,
            deleted: sum.deleted + character.nb_deleted,
        }),
        { added: 0, edited: 0, deleted: 0 },
    );
}
</script>

<template>
    <div v-if="entries.length === 0" class="py-8 text-center text-sm text-muted-foreground">
        Nobody has scored any points this period yet.
    </div>
    <ul v-else class="divide-y divide-border/60">
        <li v-for="entry in entries" :key="entry.position" class="flex flex-col gap-2 py-3 first:pt-0 last:pb-0">
            <div class="flex items-center gap-3">
                <span class="w-8 shrink-0 text-center text-lg" aria-hidden="true">{{ medalFor(entry.position) }}</span>
                <div class="min-w-0 flex-1">
                    <p class="truncate font-medium text-foreground">{{ entry.display_name }}</p>
                    <p class="text-xs text-muted-foreground">
                        <span class="text-emerald-600 dark:text-emerald-400">+{{ totals(entry).added }}</span>
                        ·
                        <span class="text-amber-600 dark:text-amber-400">~{{ totals(entry).edited }}</span>
                        ·
                        <span class="text-red-600 dark:text-red-400">-{{ totals(entry).deleted }}</span>
                    </p>
                </div>
                <span class="shrink-0 font-semibold text-foreground">{{ entry.points }} pts</span>
            </div>

            <details v-if="entry.characters.length > 1" class="ml-11 rounded-md border border-border/50 bg-muted/5">
                <summary class="cursor-pointer list-none px-3 py-1.5 text-xs font-medium text-muted-foreground">
                    {{ entry.characters.length }} characters
                </summary>
                <ul class="divide-y divide-border/50 border-t border-border/50 px-3">
                    <li
                        v-for="character in entry.characters"
                        :key="character.character_id"
                        class="flex items-center justify-between gap-3 py-1.5 text-xs text-muted-foreground"
                    >
                        <span class="truncate">{{ character.character_name }}</span>
                        <span class="shrink-0">{{ character.points }} pts</span>
                    </li>
                </ul>
            </details>
        </li>
    </ul>
</template>
