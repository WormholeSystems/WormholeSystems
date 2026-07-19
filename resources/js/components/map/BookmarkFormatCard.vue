<script setup lang="ts">
import MapBookmarkFormatController from '@/actions/App/Http/Controllers/MapBookmarkFormatController';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { TAliasScheme } from '@/lib/alias';
import {
    BOOKMARK_TOKENS,
    DEFAULT_BOOKMARK_FORMAT_KSPACE,
    DEFAULT_BOOKMARK_FORMAT_RETURN,
    DEFAULT_BOOKMARK_FORMAT_WORMHOLE,
    renderBookmarkTemplate,
    TBookmarkToken,
} from '@/lib/bookmark';
import { TMapSummary } from '@/pages/maps';
import { router } from '@inertiajs/vue3';
import { computed, reactive, ref } from 'vue';

const { map, canEdit } = defineProps<{
    map: TMapSummary;
    canEdit: boolean;
}>();

const aliasSchemeOptions: { value: TAliasScheme; label: string; example: string }[] = [
    { value: 'numeric', label: 'Numeric', example: '11, 12, 13' },
    { value: 'alphabetical', label: 'Alphabetical', example: 'AA, AB, AH1' },
];

const aliasScheme = ref<TAliasScheme>(map.bookmark_alias_scheme);
const aliasSchemeError = ref<string | null>(null);

const ignoredAlias = ref<string>(map.bookmark_ignored_alias);
const ignoredAliasError = ref<string | null>(null);

type FieldKey = 'wormhole' | 'kspace' | 'return';

type Field = {
    key: FieldKey;
    column: 'bookmark_format_wormhole' | 'bookmark_format_kspace' | 'bookmark_format_return';
    label: string;
    description?: string;
    fallback: string;
    sample: Record<TBookmarkToken, string>;
};

const fields: Field[] = [
    {
        key: 'wormhole',
        column: 'bookmark_format_wormhole',
        label: 'Wormhole systems',
        fallback: DEFAULT_BOOKMARK_FORMAT_WORMHOLE,
        sample: {
            alias: 'Home',
            sig: 'ABC',
            class: 'C3',
            name: 'J123456',
            region: 'B-R00004',
            occupier: 'HK',
            size: 'XM',
            wh: 'K162',
            mass: 'reduced',
            life: 'EOL',
        },
    },
    {
        key: 'kspace',
        column: 'bookmark_format_kspace',
        label: 'K-space systems',
        fallback: DEFAULT_BOOKMARK_FORMAT_KSPACE,
        sample: {
            alias: 'Home',
            sig: 'ABC',
            class: 'HS',
            name: 'Jita',
            region: 'The Forge',
            occupier: 'CalNavy',
            size: 'MD',
            wh: 'B274',
            mass: 'reduced',
            life: 'EOL',
        },
    },
    {
        key: 'return',
        column: 'bookmark_format_return',
        label: 'Return connections',
        description: 'Used instead of the templates above for a bookmark that leads back up-chain, or to the home system.',
        fallback: DEFAULT_BOOKMARK_FORMAT_RETURN,
        sample: {
            alias: 'HOME',
            sig: 'ABC',
            class: 'C3',
            name: 'J123456',
            region: 'B-R00004',
            occupier: 'HK',
            size: 'XM',
            wh: 'K162',
            mass: 'reduced',
            life: 'EOL',
        },
    },
];

const drafts = reactive<Record<FieldKey, string>>({
    wormhole: map.bookmark_format_wormhole,
    kspace: map.bookmark_format_kspace,
    return: map.bookmark_format_return,
});

const errors = reactive<Record<FieldKey, string | null>>({ wormhole: null, kspace: null, return: null });

const saving = ref(false);

const dirty = computed(
    () =>
        drafts.wormhole !== map.bookmark_format_wormhole ||
        drafts.kspace !== map.bookmark_format_kspace ||
        drafts.return !== map.bookmark_format_return ||
        aliasScheme.value !== map.bookmark_alias_scheme ||
        ignoredAlias.value !== map.bookmark_ignored_alias,
);

function preview(field: Field): string {
    const rendered = renderBookmarkTemplate(drafts[field.key], field.sample);
    return rendered || '—';
}

function save(): void {
    if (!canEdit || !dirty.value || saving.value) {
        return;
    }

    saving.value = true;

    router.put(
        MapBookmarkFormatController.update(map.slug).url,
        {
            bookmark_format_wormhole: drafts.wormhole,
            bookmark_format_kspace: drafts.kspace,
            bookmark_format_return: drafts.return,
            bookmark_alias_scheme: aliasScheme.value,
            bookmark_ignored_alias: ignoredAlias.value,
        },
        {
            preserveState: true,
            preserveScroll: true,
            only: ['map'],
            onSuccess: () => {
                errors.wormhole = null;
                errors.kspace = null;
                errors.return = null;
                aliasSchemeError.value = null;
                ignoredAliasError.value = null;
            },
            onError: (bag) => {
                errors.wormhole = bag.bookmark_format_wormhole ?? null;
                errors.kspace = bag.bookmark_format_kspace ?? null;
                errors.return = bag.bookmark_format_return ?? null;
                aliasSchemeError.value = bag.bookmark_alias_scheme ?? null;
                ignoredAliasError.value = bag.bookmark_ignored_alias ?? null;
            },
            onFinish: () => {
                saving.value = false;
            },
        },
    );
}

function setAliasScheme(scheme: TAliasScheme): void {
    if (!canEdit) {
        return;
    }

    aliasScheme.value = scheme;
}

function insertToken(field: Field, token: TBookmarkToken): void {
    if (!canEdit) {
        return;
    }

    const value = drafts[field.key];
    const separator = value.length > 0 && !value.endsWith(' ') ? ' ' : '';
    drafts[field.key] = `${value}${separator}{${token}}`;
}

function resetToDefault(field: Field): void {
    if (!canEdit) {
        return;
    }

    drafts[field.key] = field.fallback;
}
</script>

<template>
    <Card>
        <CardHeader>
            <CardTitle class="text-xl font-semibold">Bookmark Format</CardTitle>
            <CardDescription>
                Customize how connection bookmark names are built when copying them to your clipboard. This format is shared by everyone on the map.
                Insert a token to add a value, or type your own text around them.
                <a
                    href="/documentation/bookmarking/customizing-the-format"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="font-medium underline underline-offset-4 hover:text-foreground"
                >
                    Learn what each placeholder does.
                </a>
            </CardDescription>
        </CardHeader>
        <CardContent class="space-y-8">
            <div class="space-y-2">
                <Label class="text-sm font-medium">Alias scheme</Label>

                <div class="grid gap-2 sm:grid-cols-2">
                    <button
                        v-for="option in aliasSchemeOptions"
                        :key="option.value"
                        type="button"
                        class="flex items-center justify-between gap-3 rounded-lg border p-3 text-left transition-colors"
                        :class="aliasScheme === option.value ? 'border-primary bg-primary/5' : 'border-border hover:bg-muted/40'"
                        :disabled="!canEdit"
                        @click="setAliasScheme(option.value)"
                    >
                        <span class="text-sm font-medium">{{ option.label }}</span>
                        <span class="font-mono text-xs text-muted-foreground">{{ option.example }}</span>
                    </button>
                </div>

                <p v-if="aliasSchemeError" class="text-sm text-destructive">{{ aliasSchemeError }}</p>
                <p class="text-xs text-muted-foreground">How the mapper suggests the next chain alias when you jump to a new system.</p>
            </div>

            <div class="space-y-2">
                <Label class="text-sm font-medium" for="bookmark-ignored-alias">Ignored alias</Label>

                <Input
                    id="bookmark-ignored-alias"
                    :model-value="ignoredAlias"
                    :disabled="!canEdit"
                    spellcheck="false"
                    autocomplete="off"
                    @update:model-value="(value) => (ignoredAlias = String(value))"
                    @keydown.enter.prevent="save"
                />

                <p v-if="ignoredAliasError" class="text-sm text-destructive">{{ ignoredAliasError }}</p>
                <p class="text-xs text-muted-foreground">
                    Excluded as a prefix when suggesting child aliases, e.g. HOME → A, AB. Leave empty to disable.
                </p>
            </div>

            <div v-for="field in fields" :key="field.key" class="space-y-2">
                <Label class="text-sm font-medium">{{ field.label }}</Label>
                <p v-if="field.description" class="text-xs text-muted-foreground">{{ field.description }}</p>

                <Input
                    :model-value="drafts[field.key]"
                    :disabled="!canEdit"
                    spellcheck="false"
                    autocomplete="off"
                    @update:model-value="(value) => (drafts[field.key] = String(value))"
                    @keydown.enter.prevent="save"
                />

                <div v-if="canEdit" class="flex flex-wrap items-center gap-1.5">
                    <button
                        v-for="token in BOOKMARK_TOKENS"
                        :key="token"
                        type="button"
                        class="rounded-md border border-border bg-muted px-2 py-0.5 font-mono text-xs text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
                        @click="insertToken(field, token)"
                    >
                        {{ '{' + token + '}' }}
                    </button>
                    <Button variant="ghost" size="sm" class="ml-auto h-6 text-xs text-muted-foreground" @click="resetToDefault(field)">
                        Reset to default
                    </Button>
                </div>

                <p v-if="errors[field.key]" class="text-sm text-destructive">{{ errors[field.key] }}</p>

                <div class="flex items-center gap-2 text-sm">
                    <span class="text-muted-foreground">Preview:</span>
                    <span class="font-mono">{{ preview(field) }}</span>
                </div>
            </div>

            <div v-if="canEdit" class="flex items-center justify-end gap-3 border-t border-border pt-4">
                <span v-if="dirty" class="text-sm text-muted-foreground">Unsaved changes</span>
                <Button :disabled="!dirty || saving" @click="save">Save</Button>
            </div>
        </CardContent>
    </Card>
</template>
