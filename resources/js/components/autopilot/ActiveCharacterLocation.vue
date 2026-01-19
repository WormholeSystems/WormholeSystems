<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import RoutePopover from '@/components/autopilot/RoutePopover.vue';
import { CharacterImage } from '@/components/images';
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
            class="mb-2 flex items-center gap-2 rounded border bg-white p-2 dark:bg-neutral-900/40"
            v-element-hover="(e) => handleHover(e, activeCharacter?.route ?? null)"
        >
            <CharacterImage
                v-if="activeCharacter"
                :character_id="activeCharacter.id"
                :character_name="activeCharacter.name"
                class="size-8 rounded-lg"
            />
            <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                    <span class="truncate text-sm font-medium">{{ activeCharacter?.name || 'Unknown' }}</span>
                </div>
                <div class="flex items-center gap-1 text-xs text-muted-foreground">
                    <TypeImage
                        v-if="characterStatus?.ship_type"
                        :type_id="characterStatus.ship_type.id"
                        :type_name="characterStatus.ship_type.name"
                        class="size-3 rounded"
                    />
                    <span class="truncate">{{ characterStatus?.ship_name || characterStatus?.ship_type?.name || 'Unknown Ship' }}</span>
                    <span v-if="statusSolarsystem" class="text-muted-foreground">• {{ statusSolarsystem.name }}</span>
                </div>
            </div>
            <div v-if="activeCharacter?.route" class="flex-shrink-0">
                <RoutePopover :route="activeCharacter.route">
                    <Button variant="secondary" size="sm" class="font-mono text-xs">
                        {{ activeCharacter.route.length > 0 ? activeCharacter.route.length - 1 : '0' }}
                    </Button>
                </RoutePopover>
            </div>
        </div>
    </DestinationContextMenu>
</template>

<style scoped></style>
