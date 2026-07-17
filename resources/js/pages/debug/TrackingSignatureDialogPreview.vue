<script setup lang="ts">
import TrackingSignatureDialog from '@/components/map/TrackingSignatureDialog.vue';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { TooltipProvider } from '@/components/ui/tooltip';
import Appearance from '@/layouts/Appearance.vue';
import { TMapConnection, TMapSolarsystem } from '@/pages/maps';
import { TLifetimeStatus, TMassStatus, TSignature, TStringedSolarsystemClass } from '@/types/models';
import { RotateCcw } from 'lucide-vue-next';
import { computed, ref } from 'vue';

/**
 * Local-only playground for the "which signature did you jump" prompt: renders
 * the real TrackingSignatureDialog against canned scenarios so layout changes
 * can be checked for every case without jumping systems in game.
 */

type TScenario = {
    key: string;
    title: string;
    description: string;
    targetName: string;
    targetClass: TStringedSolarsystemClass;
    suggestedAlias: string | null;
    signatures: TSignature[];
};

let nextId = 1;

function minutesAgo(minutes: number): string {
    return new Date(Date.now() - minutes * 60_000).toISOString();
}

function sig(input: {
    signatureId: string;
    category?: 'wormhole' | 'gas' | 'data' | 'relic' | 'combat' | 'ore';
    code?: string;
    targetClass?: TStringedSolarsystemClass | 'unknown';
    extra?: string | null;
    rawTypeName?: string;
    /** Marks the signature as already tied to a connection leading to this map solarsystem id. */
    connectedToId?: number;
    lifetime?: TLifetimeStatus;
    massStatus?: TMassStatus;
    createdMinutesAgo?: number;
}): TSignature {
    const id = nextId++;
    const isWormhole = input.category === 'wormhole';

    return {
        id,
        signature_id: input.signatureId,
        signature_type_id: input.code ? id : null,
        signature_category_id: input.category ? id : null,
        raw_type_name: input.rawTypeName ?? null,
        map_connection_id: input.connectedToId ? id : null,
        map_connection: input.connectedToId
            ? ({ id, from_map_solarsystem_id: ORIGIN_ID, to_map_solarsystem_id: input.connectedToId } as TMapConnection)
            : null,
        signature_type: input.code
            ? {
                  id,
                  name: `${input.code} - ${input.targetClass ?? '?'}`,
                  signature: input.code,
                  signature_category_id: id,
                  category_name: isWormhole ? 'Wormhole' : (input.category ?? null),
                  target_class: input.targetClass ?? null,
                  extra: input.extra ?? null,
                  spawn_areas: null,
              }
            : null,
        signature_category: input.category
            ? { id, name: input.category, code: input.category, created_at: minutesAgo(0), updated_at: minutesAgo(0) }
            : null,
        lifetime: input.lifetime ?? 'healthy',
        mass_status: input.massStatus ?? null,
        created_at: minutesAgo(input.createdMinutesAgo ?? 30),
        updated_at: minutesAgo(input.createdMinutesAgo ?? 30),
    } as TSignature;
}

const ORIGIN_ID = 1;

const ORIGIN = {
    id: ORIGIN_ID,
    alias: 'HOME',
    solarsystem: { name: 'J152820' },
} as TMapSolarsystem;

/** Map systems the connected fixtures can lead to. */
const MAP_SOLARSYSTEMS = [
    ORIGIN,
    { id: 2, alias: 'HOME-A', solarsystem: { name: 'J145510' } } as TMapSolarsystem,
    { id: 3, alias: null, solarsystem: { name: 'J104859' } } as TMapSolarsystem,
];

const scenarios: TScenario[] = [
    {
        key: 'typical-c4',
        title: 'Typical C4 jump',
        description: 'Mixed bag: matching types, wrong classes, a connected hole, unscanned signatures, and a gas site that must not appear.',
        targetName: 'J145510',
        targetClass: '4',
        suggestedAlias: 'HOME-A',
        signatures: [
            sig({ signatureId: 'ABC-123', category: 'wormhole', code: 'X877', targetClass: '4', createdMinutesAgo: 12 }),
            sig({ signatureId: 'DEF-456', category: 'wormhole', code: 'K162', targetClass: 'unknown', createdMinutesAgo: 45 }),
            sig({ signatureId: 'GHI-789', category: 'wormhole', code: 'U210', targetClass: 'l', createdMinutesAgo: 80 }),
            sig({ signatureId: 'JKL-012', category: 'wormhole', code: 'Z142', targetClass: 'n', createdMinutesAgo: 200 }),
            sig({ signatureId: 'MNO-345', category: 'gas', code: undefined, rawTypeName: 'Bountiful Frontier Reservoir' }),
            sig({ signatureId: 'PQR-678' }),
            sig({ signatureId: 'STU-901', category: 'wormhole', code: 'X877', targetClass: '4', connectedToId: 2, createdMinutesAgo: 300 }),
        ],
    },
    {
        key: 'highsec-exit',
        title: 'Highsec exit',
        description: 'Jump into highsec: exact class match on k-space classes.',
        targetName: 'Jita',
        targetClass: 'h',
        suggestedAlias: null,
        signatures: [
            sig({ signatureId: 'AAA-111', category: 'wormhole', code: 'B274', targetClass: 'h', createdMinutesAgo: 20 }),
            sig({ signatureId: 'BBB-222', category: 'wormhole', code: 'U210', targetClass: 'l', createdMinutesAgo: 60 }),
            sig({ signatureId: 'CCC-333', category: 'wormhole', code: 'D845', targetClass: 'h', lifetime: 'eol', createdMinutesAgo: 900 }),
            sig({ signatureId: 'DDD-444', category: 'relic', rawTypeName: 'Forgotten Frontier Quarantine Outpost' }),
        ],
    },
    {
        key: 'all-unlikely',
        title: 'Everything unlikely',
        description: 'No signature fits the destination: only the demoted section has options, and the sites are hidden entirely.',
        targetName: 'J105830',
        targetClass: '5',
        suggestedAlias: 'HOME-B',
        signatures: [
            sig({ signatureId: 'EEE-555', category: 'wormhole', code: 'B274', targetClass: 'h' }),
            sig({ signatureId: 'FFF-666', category: 'combat', rawTypeName: 'Fortification Frontier Stronghold' }),
            sig({ signatureId: 'GGG-777', category: 'data', rawTypeName: 'Unsecured Frontier Database' }),
        ],
    },
    {
        key: 'unscanned-only',
        title: 'Unscanned signatures',
        description: 'Nothing resolved yet: every option is a bare signature id.',
        targetName: 'J170211',
        targetClass: '3',
        suggestedAlias: null,
        signatures: [sig({ signatureId: 'HHH-888' }), sig({ signatureId: 'III-999' }), sig({ signatureId: 'JJJ-000' })],
    },
    {
        key: 'crowded',
        title: 'Crowded system',
        description: 'A C6 with two dozen signatures: scrolling, search, and all three sections populated.',
        targetName: 'J104859',
        targetClass: '6',
        suggestedAlias: 'DEEP-1',
        signatures: Array.from({ length: 24 }, (_, index) => {
            const signatureId = `${String.fromCharCode(75 + (index % 12))}${String.fromCharCode(65 + (index % 8))}Q-${String(100 + index * 7).slice(0, 3)}`;
            if (index % 5 === 4) return sig({ signatureId, category: 'gas', rawTypeName: 'Vast Frontier Reservoir' });
            if (index % 3 === 2) return sig({ signatureId, category: 'wormhole', code: 'H296', targetClass: '5', createdMinutesAgo: index * 30 });
            if (index % 3 === 1)
                return sig({
                    signatureId,
                    category: 'wormhole',
                    code: 'V753',
                    targetClass: '6',
                    createdMinutesAgo: index * 15,
                    connectedToId: index === 1 ? 3 : undefined,
                });
            return sig({ signatureId, createdMinutesAgo: index * 10 });
        }),
    },
    {
        key: 'eol-critical',
        title: 'EOL and critical holes',
        description: 'Signatures carrying lifetime and mass state that prefill the selectors.',
        targetName: 'J121856',
        targetClass: '3',
        suggestedAlias: null,
        signatures: [
            sig({ signatureId: 'KKK-111', category: 'wormhole', code: 'C247', targetClass: '3', lifetime: 'eol', massStatus: 'reduced' }),
            sig({ signatureId: 'LLL-222', category: 'wormhole', code: 'K162', targetClass: 'unknown', lifetime: 'critical', massStatus: 'critical' }),
        ],
    },
];

const preselectFirst = ref(false);

const activeScenario = ref<TScenario | null>(null);
const dialogOpen = ref(false);
const lastSelection = ref<Record<string, unknown> | null>(null);

const activeSignatures = computed(() => activeScenario.value?.signatures ?? []);

function openScenario(scenario: TScenario): void {
    activeScenario.value = scenario;
    lastSelection.value = null;
    dialogOpen.value = true;
}

function handleSelection(selection: Record<string, unknown>): void {
    lastSelection.value = selection;
    dialogOpen.value = false;
}
</script>

<template>
    <TooltipProvider :delay-duration="300">
        <div class="min-h-screen bg-background p-8 text-foreground">
            <div class="mx-auto max-w-3xl">
                <div class="flex items-start justify-between gap-4">
                    <p class="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">Debug · local only</p>
                    <Appearance />
                </div>
                <h1 class="mt-2 font-display text-2xl font-bold tracking-tight">Tracking signature dialog preview</h1>
                <p class="mt-2 text-sm text-muted-foreground">
                    Pick a scenario to open the real jump prompt with canned signatures. Closing or confirming shows the emitted selection below.
                </p>

                <div class="mt-8 grid gap-3 sm:grid-cols-2">
                    <button
                        v-for="scenario in scenarios"
                        :key="scenario.key"
                        type="button"
                        class="flex flex-col gap-1 rounded bg-card p-4 text-left ring-1 ring-border transition-shadow hover:ring-foreground/25"
                        @click="openScenario(scenario)"
                    >
                        <span class="font-display font-bold">{{ scenario.title }}</span>
                        <span class="text-sm leading-6 text-muted-foreground">{{ scenario.description }}</span>
                        <span class="mt-2 font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
                            → {{ scenario.targetName }} · class {{ scenario.targetClass }} · {{ scenario.signatures.length }} signatures
                        </span>
                    </button>
                </div>

                <label class="mt-6 flex w-fit items-center gap-2 text-sm text-muted-foreground">
                    <Checkbox :model-value="preselectFirst" @update:model-value="(value) => (preselectFirst = value === true)" />
                    Preselect the first likely signature
                </label>

                <div v-if="activeScenario" class="mt-8 flex items-center gap-3">
                    <Button variant="outline" size="sm" class="gap-2" @click="dialogOpen = true">
                        <RotateCcw class="size-3.5" />
                        Reopen {{ activeScenario.title }}
                    </Button>
                </div>

                <div v-if="lastSelection" class="mt-6 overflow-hidden rounded bg-card ring-1 ring-border">
                    <div class="border-b border-border/50 bg-muted/30 px-3 py-2 font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
                        Last emitted selection
                    </div>
                    <pre class="overflow-x-auto p-4 text-xs">{{ JSON.stringify(lastSelection, null, 2) }}</pre>
                </div>
            </div>

            <TrackingSignatureDialog
                v-model:open="dialogOpen"
                :origin-map-solarsystem="ORIGIN"
                :target-solarsystem-name="activeScenario?.targetName ?? null"
                :target-solarsystem-class="activeScenario?.targetClass ?? null"
                :signatures="activeSignatures"
                :suggested-alias="activeScenario?.suggestedAlias"
                :map-solarsystems="MAP_SOLARSYSTEMS"
                :preselect-first-signature="preselectFirst"
                @select-signature="handleSelection"
            />
        </div>
    </TooltipProvider>
</template>
