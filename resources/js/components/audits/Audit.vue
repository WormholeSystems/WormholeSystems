<script setup lang="ts">
import { CharacterImage } from '@/components/images';
import { useSelectedMapSolarsystem } from '@/composables/useSelectedMapSolarsystem';
import { useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import { TMapSolarsystem } from '@/pages/maps';
import { TAudit } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { useNow } from '@vueuse/core';
import { differenceInDays, differenceInHours, differenceInMinutes, format } from 'date-fns';
import { computed } from 'vue';

const { audit } = defineProps<{
    audit: TAudit;
}>();

const selectedMapSolarsystem = useSelectedMapSolarsystem();

const { resolveSolarsystem } = useStaticSolarsystems();

const resolvedSelectedSolarsystem = computed(() => {
    const selected = selectedMapSolarsystem.value;
    if (!selected) {
        return null;
    }

    const resolved = resolveSolarsystem({ id: selected.solarsystem_id });
    return {
        ...selected,
        solarsystem: resolved,
    };
});

const action = computed(() => {
    if (audit.event === 'created') {
        return 'added';
    }

    if (audit.old_values.position_x === null && audit.new_values.position_x !== null) {
        return 'added';
    }

    if (audit.old_values.position_x !== null && audit.new_values.position_x === null) {
        return 'removed';
    }

    if ('position_x' in audit.new_values || 'position_y' in audit.new_values) {
        return 'moved';
    }

    return 'updated';
});

function truncateNotes(notes: string = ''): string {
    if (!notes) return '';
    if (notes.length > 50) {
        return notes.slice(0, 50) + '...';
    }
    return notes;
}

const updated_values = computed(() => {
    const column = Object.keys(audit.new_values).at(0) as keyof (TMapSolarsystem & {
        position_x: number;
        position_y: number;
    });
    if (!column) return null;
    if (column === 'position_x' || column === 'position_y') return null;

    if (column === 'status') {
        return `changed the status ${fromToString(audit.old_values.status, audit.new_values.status)}`;
    }
    if (column === 'alias') {
        return `changed the alias ${fromToString(audit.old_values.alias, audit.new_values.alias)}`;
    }
    if (column === 'occupier_alias') {
        return `changed the occupier alias ${fromToString(audit.old_values.occupier_alias, audit.new_values.occupier_alias)}`;
    }

    if (column === 'pinned') {
        const name = resolvedSelectedSolarsystem.value?.solarsystem?.name ?? 'the system';
        return audit.new_values.pinned ? `pinned ${name}` : `unpinned ${name}`;
    }

    if (column === 'notes') {
        return `changed the notes ${fromToString(truncateNotes(audit.old_values.notes), truncateNotes(audit.new_values.notes))}`;
    }

    return `changed the ${column} ${fromToString(audit.old_values[column], audit.new_values[column])}`;

    function fromToString(from: string | null, to: string | null): string {
        if (!from) {
            return `to ${to}`;
        }
        if (!to) {
            return `to nothing`;
        }

        return `from ${from} to ${to}`;
    }
});

const now = useNow({ interval: 60_000 });

const date = computed(() => new UTCDate(audit.created_at));

const formattedDate = computed(() => format(date.value, 'dd MMM yyyy HH:mm'));
const timeAgo = computed(() => {
    const diffInMinutes = differenceInMinutes(now.value, date.value);
    const diffInHours = differenceInHours(now.value, date.value);
    const diffInDays = differenceInDays(now.value, date.value);

    if (diffInDays > 0) {
        return `${diffInDays}d ago`;
    }
    if (diffInHours > 0) {
        return `${diffInHours}h ago`;
    }
    if (diffInMinutes > 0) {
        return `${diffInMinutes}m ago`;
    }
    return 'just now';
});

const actor = computed(() => {
    if (audit.character?.name) {
        return audit.character.name;
    }

    return 'System';
});
</script>

<template>
    <div class="flex items-center gap-2 border-b border-border/30 px-3 py-1.5 hover:bg-muted/30" :key="audit.id">
        <CharacterImage
            class="size-5 shrink-0 rounded"
            :character_id="audit.character.id"
            :character_name="audit.character?.name"
            v-if="audit.character"
        />
        <div v-else class="flex size-5 shrink-0 items-center justify-center rounded bg-muted text-[10px] text-muted-foreground">S</div>

        <span class="flex-1 truncate text-xs text-muted-foreground" v-if="action === 'added'">
            <span class="text-foreground">{{ actor }}</span> added {{ resolvedSelectedSolarsystem?.solarsystem?.name ?? 'system' }}
        </span>
        <span class="flex-1 truncate text-xs text-muted-foreground" v-else-if="action === 'removed'">
            <span class="text-foreground">{{ actor }}</span> removed {{ resolvedSelectedSolarsystem?.solarsystem?.name ?? 'system' }}
        </span>
        <span class="flex-1 truncate text-xs text-muted-foreground" v-else-if="action === 'moved'">
            <span class="text-foreground">{{ actor }}</span> moved {{ resolvedSelectedSolarsystem?.solarsystem?.name ?? 'system' }}
        </span>
        <span class="flex-1 truncate text-xs text-muted-foreground" v-else-if="action === 'updated'">
            <span class="text-foreground">{{ actor }}</span> {{ updated_values }}
        </span>

        <span class="font-mono text-[10px] text-muted-foreground" :title="formattedDate">{{ timeAgo }}</span>
    </div>
</template>

<style scoped></style>
