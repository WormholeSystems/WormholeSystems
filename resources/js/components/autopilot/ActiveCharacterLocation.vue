<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import RoutePopover from '@/components/autopilot/RoutePopover.vue';
import { CharacterImage } from '@/components/images';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { usePath } from '@/composables/usePath';
import { useStaticSolarsystem } from '@/composables/useStaticSolarsystems';
import { TSolarsystem } from '@/pages/maps';
import { TCharacter, TCharacterStatus } from '@/types/models';
import { vElementHover } from '@vueuse/components';
import TypeImage from '../images/TypeImage.vue';

const { activeCharacter, characterStatus } = defineProps<{
    activeCharacter: TCharacter;
    characterStatus: TCharacterStatus;
}>();

const { setPath } = usePath();

const statusSolarsystem = useStaticSolarsystem(() => characterStatus?.solarsystem_id ?? null);

function handleHover(hovered: boolean, route: TSolarsystem[] | null) {
    if (hovered) {
        setPath(route);
    } else {
        setPath(null);
    }
}
</script>

<template>
    <DestinationContextMenu :solarsystem_id="characterStatus?.solarsystem_id">
        <div
            class="flex items-center gap-2.5 border-b border-border/50 px-3 py-2"
            v-element-hover="(e) => handleHover(e, activeCharacter?.route ?? null)"
        >
            <div class="relative shrink-0">
                <CharacterImage
                    v-if="activeCharacter"
                    :character_id="activeCharacter.id"
                    :character_name="activeCharacter.name"
                    class="size-10 rounded"
                />
                <TypeImage
                    v-if="characterStatus?.ship_type"
                    :type_id="characterStatus.ship_type.id"
                    :type_name="characterStatus.ship_type.name"
                    class="absolute -right-1 -bottom-1 size-6 rounded border-2 border-card"
                />
            </div>
            <div class="min-w-0 flex-1">
                <div class="flex items-center gap-1.5">
                    <span class="truncate text-xs font-medium">{{ activeCharacter?.name || 'Unknown' }}</span>
                </div>
                <div v-if="statusSolarsystem" class="flex items-baseline gap-1 text-[11px] text-muted-foreground">
                    <SolarsystemClass
                        :wormhole_class="statusSolarsystem.class"
                        :security="statusSolarsystem.security"
                        class="shrink-0 leading-none font-bold"
                    />
                    <span class="truncate">{{ statusSolarsystem.name }}</span>
                    <span class="text-muted-foreground/50">·</span>
                    <span class="truncate">{{ statusSolarsystem.region?.name }}</span>
                </div>
                <div class="flex items-center gap-1 text-[11px] text-muted-foreground">
                    <span v-if="characterStatus?.ship_type" class="truncate">{{ characterStatus.ship_type.name }}</span>
                    <span v-if="characterStatus?.ship_name && characterStatus?.ship_type" class="text-muted-foreground/50">·</span>
                    <span v-if="characterStatus?.ship_name" class="truncate italic">{{ characterStatus.ship_name }}</span>
                    <span v-if="!characterStatus?.ship_type && !characterStatus?.ship_name">Unknown Ship</span>
                </div>
            </div>
            <RoutePopover v-if="activeCharacter?.route" :route="activeCharacter.route">
                <Button variant="ghost" size="sm" class="h-6 px-1.5 font-mono text-[10px] text-muted-foreground">
                    {{ activeCharacter.route.length > 0 ? activeCharacter.route.length - 1 : '0' }}j
                </Button>
            </RoutePopover>
        </div>
    </DestinationContextMenu>
</template>

<style scoped></style>
