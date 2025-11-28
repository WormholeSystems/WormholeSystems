<script setup lang="ts">
import { CharacterImage } from '@/components/images';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuPortal,
    DropdownMenuSeparator,
    DropdownMenuSub,
    DropdownMenuSubContent,
    DropdownMenuSubTrigger,
    DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { useNavigationSystems } from '@/composables/useNavigationSystems';
import useUser from '@/composables/useUser';
import { useWaypoint } from '@/composables/useWaypoint';
import { TUniverseSolarsystem } from '@/types/universe-map';
import { computed, nextTick, ref, watch } from 'vue';
import { toast } from 'vue-sonner';

const props = defineProps<{
    system: TUniverseSolarsystem | null;
    position: { x: number; y: number };
}>();

const emit = defineEmits<{
    close: [];
}>();

const user = useUser();
const setWaypoint = useWaypoint();
const { setFromSystem, setToSystem } = useNavigationSystems();

const isOpen = ref(false);

// Watch for system changes to open menu
watch(
    () => props.system,
    async (system) => {
        if (system) {
            await nextTick();
            isOpen.value = true;
        }
    },
);

// Emit close when menu closes
watch(isOpen, (open) => {
    if (!open) {
        emit('close');
    }
});

const classString = computed(() => {
    if (!props.system) return '';
    if (props.system.class) return `C${props.system.class}`;
    if (props.system.security >= 0.5) return 'HS';
    if (props.system.security > 0.0) return 'LS';
    return 'NS';
});

const defaultName = computed(() => {
    if (!props.system) return '';
    return `${classString.value} ${props.system.name} ${props.system.region.name}`;
});

function copyNameToClipboard(value?: string) {
    navigator.clipboard.writeText(value || defaultName.value);
    toast.success('Copied to clipboard');
}

function copySystemName() {
    if (!props.system) return;
    navigator.clipboard.writeText(props.system.name);
    toast.success('Copied to clipboard');
}

const isWormhole = computed(() => props.system?.class !== null);

const dotlanUrl = computed(() => {
    if (!props.system) return '';
    const regionSlug = props.system.region.name.replaceAll(' ', '_');
    const systemSlug = props.system.name.replaceAll(' ', '_');
    return `https://evemaps.dotlan.net/map/${regionSlug}/${systemSlug}`;
});

const zkillUrl = computed(() => {
    if (!props.system) return '';
    return `https://zkillboard.com/system/${props.system.id}/`;
});
</script>

<template>
    <DropdownMenu v-model:open="isOpen" :modal="false">
        <DropdownMenuTrigger as-child>
            <span class="pointer-events-none fixed" :style="{ left: `${position.x}px`, top: `${position.y}px` }" />
        </DropdownMenuTrigger>
        <DropdownMenuPortal>
            <DropdownMenuContent v-if="system" class="w-56" :side-offset="0" align="start" side="bottom">
                <!-- External links -->
                <DropdownMenuSub>
                    <DropdownMenuSubTrigger>External</DropdownMenuSubTrigger>
                    <DropdownMenuSubContent>
                        <DropdownMenuItem as-child>
                            <a :href="dotlanUrl" target="_blank" rel="noopener noreferrer"> Dotlan </a>
                        </DropdownMenuItem>
                        <DropdownMenuItem as-child>
                            <a :href="zkillUrl" target="_blank" rel="noopener noreferrer"> zKillboard </a>
                        </DropdownMenuItem>
                    </DropdownMenuSubContent>
                </DropdownMenuSub>

                <!-- Set destination (only for k-space) -->
                <DropdownMenuSub v-if="!isWormhole && user.characters.length > 0">
                    <DropdownMenuSubTrigger>Set destination</DropdownMenuSubTrigger>
                    <DropdownMenuSubContent>
                        <DropdownMenuItem v-for="character in user.characters" :key="character.id" @select="setWaypoint(character.id, system.id)">
                            <CharacterImage :character_id="character.id" :character_name="character.name" class="size-5 rounded-lg" />
                            {{ character.name }}
                        </DropdownMenuItem>
                    </DropdownMenuSubContent>
                </DropdownMenuSub>

                <!-- Add waypoint (only for k-space) -->
                <DropdownMenuSub v-if="!isWormhole && user.characters.length > 0">
                    <DropdownMenuSubTrigger>Add waypoint</DropdownMenuSubTrigger>
                    <DropdownMenuSubContent>
                        <DropdownMenuItem
                            v-for="character in user.characters"
                            :key="character.id"
                            @select="setWaypoint(character.id, system.id, false)"
                        >
                            <CharacterImage :character_id="character.id" :character_name="character.name" class="size-5 rounded-lg" />
                            {{ character.name }}
                        </DropdownMenuItem>
                    </DropdownMenuSubContent>
                </DropdownMenuSub>

                <DropdownMenuSeparator />

                <!-- Copy name -->
                <DropdownMenuSub>
                    <DropdownMenuSubTrigger>Copy name</DropdownMenuSubTrigger>
                    <DropdownMenuSubContent>
                        <DropdownMenuItem @select="copySystemName">
                            {{ system.name }}
                        </DropdownMenuItem>
                        <DropdownMenuItem @select="copyNameToClipboard()">
                            {{ defaultName }}
                        </DropdownMenuItem>
                    </DropdownMenuSubContent>
                </DropdownMenuSub>

                <DropdownMenuSeparator />

                <!-- Route planner -->
                <DropdownMenuSub>
                    <DropdownMenuSubTrigger>Route planner</DropdownMenuSubTrigger>
                    <DropdownMenuSubContent>
                        <DropdownMenuItem @select="setFromSystem(system.id)">Set as origin</DropdownMenuItem>
                        <DropdownMenuItem @select="setToSystem(system.id)">Set as destination</DropdownMenuItem>
                    </DropdownMenuSubContent>
                </DropdownMenuSub>
            </DropdownMenuContent>
        </DropdownMenuPortal>
    </DropdownMenu>
</template>
