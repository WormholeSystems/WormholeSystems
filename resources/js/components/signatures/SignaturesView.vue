<script setup lang="ts">
import SignatureTimeDetails from '@/components/signatures/SignatureTimeDetails.vue';
import WormholeOption from '@/components/signatures/WormholeOption.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import type { TProcessedConnection } from '@/composables/map';
import { TSignature } from '@/types/models';
import { Cloud, Database, Fan, Gem, Landmark, Shield, Swords } from 'lucide-vue-next';
import { type Component, computed } from 'vue';

const { signatures, connections = [] } = defineProps<{
    signatures: TSignature[];
    connections?: TProcessedConnection[];
}>();

const categoryAbbrev: Record<string, string> = {
    Wormhole: 'WH',
    'Data Site': 'Data',
    'Relic Site': 'Relic',
    'Ore Site': 'Ore',
    'Gas Site': 'Gas',
    'Combat Site': 'Combat',
    'Homefront Operations': 'HF',
};

const categoryIcon: Record<string, Component> = {
    Wormhole: Fan,
    'Data Site': Database,
    'Relic Site': Landmark,
    'Ore Site': Gem,
    'Gas Site': Cloud,
    'Combat Site': Swords,
    'Homefront Operations': Shield,
};

const categoryColor: Record<string, string> = {
    Wormhole: 'text-sky-400',
    'Data Site': 'text-cyan-400',
    'Relic Site': 'text-amber-400',
    'Combat Site': 'text-green-400',
    'Gas Site': 'text-orange-400',
    'Ore Site': 'text-yellow-400',
    'Homefront Operations': 'text-rose-400',
};

function categoryName(signature: TSignature): string {
    return signature.signature_category?.name ?? '';
}

function isWormhole(signature: TSignature): boolean {
    return categoryName(signature) === 'Wormhole';
}

function selectedConnection(signature: TSignature): TProcessedConnection | null {
    if (!signature.map_connection_id) {
        return null;
    }
    return connections.find((connection) => connection.id === signature.map_connection_id) ?? null;
}

const sorted = computed(() => [...signatures].sort((a, b) => (a.signature_id ?? '').localeCompare(b.signature_id ?? '')));
</script>

<template>
    <div>
        <!-- Header -->
        <div
            class="flex items-center gap-2 border-b border-border/30 bg-muted/20 px-3 py-1.5 font-mono text-[10px] tracking-wider text-muted-foreground uppercase"
        >
            <span class="w-16 shrink-0">ID</span>
            <span class="w-24 shrink-0">Cat</span>
            <span class="min-w-0 flex-1">Type</span>
            <span class="min-w-0 flex-1">Conn</span>
            <span class="w-10 shrink-0 text-right">Age</span>
            <span class="w-6 shrink-0" />
        </div>

        <!-- Rows -->
        <template v-if="sorted.length">
            <div
                v-for="signature in sorted"
                :key="signature.id"
                class="flex items-center gap-2 border-b border-border/30 px-3 py-1.5 hover:bg-muted/30"
            >
                <!-- ID -->
                <span class="w-16 shrink-0 font-mono text-xs">{{ signature.signature_id || '---' }}</span>

                <!-- Category -->
                <span class="flex w-24 shrink-0 items-center gap-1 text-xs">
                    <component
                        :is="categoryIcon[categoryName(signature)]"
                        v-if="categoryIcon[categoryName(signature)]"
                        class="size-3 shrink-0"
                        :class="categoryColor[categoryName(signature)]"
                    />
                    {{ categoryAbbrev[categoryName(signature)] ?? '—' }}
                </span>

                <!-- Type -->
                <span class="min-w-0 flex-1 text-xs">
                    <WormholeOption v-if="isWormhole(signature) && signature.signature_type" :wormhole="signature.signature_type" />
                    <span v-else class="truncate">{{ signature.signature_type?.name ?? signature.raw_type_name ?? '·' }}</span>
                </span>

                <!-- Connection (wormholes only) -->
                <span class="min-w-0 flex-1 text-xs">
                    <span v-if="isWormhole(signature) && selectedConnection(signature)" class="inline-flex items-center gap-1">
                        <SolarsystemClass
                            :solarsystem_class="selectedConnection(signature)!.target.solarsystem.class"
                            class="w-5 shrink-0 text-center"
                        />
                        <span v-if="selectedConnection(signature)!.target.alias" class="shrink-0 font-medium">
                            {{ selectedConnection(signature)!.target.alias }}
                        </span>
                        <span class="truncate" :class="{ 'text-muted-foreground': selectedConnection(signature)!.target.alias }">
                            {{ selectedConnection(signature)!.target.solarsystem.name }}
                        </span>
                        <span class="shrink-0 text-muted-foreground/60">{{ selectedConnection(signature)!.target.solarsystem.region?.name }}</span>
                    </span>
                    <span v-else class="text-muted-foreground/40">·</span>
                </span>

                <!-- Age -->
                <span class="w-10 shrink-0 text-right">
                    <SignatureTimeDetails
                        :category="signature.signature_category?.name"
                        :selected_connection="selectedConnection(signature)"
                        :signature="signature"
                    />
                </span>
                <span class="w-6 shrink-0" />
            </div>
        </template>
        <div v-else class="flex h-full flex-col items-center justify-center gap-2 p-4">
            <p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">No signatures</p>
        </div>
    </div>
</template>

<style scoped></style>
