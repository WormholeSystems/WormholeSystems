<script setup lang="ts">
import { Badge } from '@/components/ui/badge';
import { Combobox, ComboboxAnchor, ComboboxInput, ComboboxItem, ComboboxVirtualList } from '@/components/ui/combobox';
import { Input } from '@/components/ui/input';
import { useEveSearch } from '@/composables/useEveSearch';
import { TEveSearchResult, TKillmailFilterSubject } from '@/types/models';
import { X } from 'lucide-vue-next';
import { computed, reactive, ref, watch } from 'vue';

const { subject } = defineProps<{ subject: TKillmailFilterSubject }>();
const ids = defineModel<number[]>('ids', { default: () => [] });

const isShip = computed(() => subject === 'ship_type' || subject === 'ship_group');
const kind = computed(() => (subject === 'ship_group' ? 'group' : 'type'));

const { results, search, resolveLabels } = useEveSearch();
const term = ref('');

// Hide entries the user has already picked.
const availableResults = computed(() => results.value.filter((result) => !ids.value.includes(result.id)));
const draft = ref('');
const labels = reactive<Record<number, string>>({});

function labelFor(id: number): string {
    return labels[id] ?? String(id);
}

function addId(id: number, name?: string) {
    if (id <= 0 || ids.value.includes(id)) return;
    if (name) labels[id] = name;
    ids.value = [...ids.value, id];
}

function removeId(id: number) {
    ids.value = ids.value.filter((existing) => existing !== id);
}

function pickResult(result: TEveSearchResult) {
    addId(result.id, result.name);
    term.value = '';
    results.value = [];
}

function resultName(result: TEveSearchResult): string {
    return result.name;
}

function commitDraft() {
    const parsed = draft.value
        .split(/[\s,]+/)
        .map((part) => parseInt(part, 10))
        .filter((id) => Number.isInteger(id) && id > 0);

    parsed.forEach((id) => addId(id));
    draft.value = '';
}

watch(term, (value) => {
    if (isShip.value) void search(kind.value, value);
});

// Resolve names for ids we don't have a label for yet (e.g. when editing a saved webhook).
async function resolveMissing() {
    if (!isShip.value) return;
    const missing = ids.value.filter((id) => !(id in labels));
    if (missing.length === 0) return;
    Object.assign(labels, await resolveLabels(kind.value, missing));
}

watch(ids, resolveMissing, { immediate: true });
</script>

<template>
    <div class="space-y-2">
        <div v-if="ids.length > 0" class="flex flex-wrap items-center gap-1">
            <Badge v-for="id in ids" :key="id" variant="secondary" class="gap-1">
                {{ labelFor(id) }}
                <button type="button" class="text-muted-foreground hover:text-foreground" @click="removeId(id)">
                    <X class="h-3 w-3" />
                </button>
            </Badge>
        </div>

        <Combobox v-if="isShip" :ignore-filter="true" class="rounded-md border">
            <ComboboxAnchor>
                <ComboboxInput v-model="term" :placeholder="`Search ${subject === 'ship_group' ? 'groups' : 'types'}…`" />
            </ComboboxAnchor>
            <ComboboxVirtualList align="start" :options="availableResults" :estimate-size="32" :text-content="resultName">
                <template #empty>{{ term.trim().length > 0 ? 'No matches' : 'Start typing to search…' }}</template>
                <template #default="{ option }">
                    <ComboboxItem
                        :value="option.name"
                        class="flex w-full items-center justify-between gap-3"
                        @select.prevent="() => pickResult(option)"
                    >
                        <span class="truncate">{{ option.name }}</span>
                        <span v-if="option.group_name || option.category_name" class="shrink-0 text-xs text-muted-foreground">
                            {{ option.group_name || option.category_name }}
                        </span>
                    </ComboboxItem>
                </template>
            </ComboboxVirtualList>
        </Combobox>

        <Input
            v-else
            v-model="draft"
            placeholder="Enter ID and press Enter…"
            inputmode="numeric"
            @keydown.enter.prevent="commitDraft"
            @blur="commitDraft"
        />
    </div>
</template>
