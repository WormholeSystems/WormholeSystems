<script setup lang="ts">
import { CharacterImage } from '@/components/images';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { TCharacter } from '@/types/models';

defineProps<{
    pilots: TCharacter[];
}>();
</script>

<template>
    <Tooltip :delay-duration="700">
        <TooltipTrigger as-child>
            <span
                class="flex h-[20px] items-center gap-1 truncate border-t border-b-neutral-300 px-2 pt-0.5 text-[10px] leading-0 dark:border-neutral-700"
            >
                <span class="size-1 animate-pulse rounded-full bg-green-500"></span>{{ pilots.at(0)?.name }}
                <span v-if="pilots.length > 1">and {{ pilots.length - 1 }} more</span>
            </span>
        </TooltipTrigger>
        <TooltipContent side="bottom" class="p-2">
            <div class="flex max-h-64 flex-col gap-1.5 overflow-y-auto">
                <div v-for="pilot in pilots" :key="pilot.id" class="flex items-center gap-2">
                    <CharacterImage :character_id="pilot.id" :character_name="pilot.name" :size="32" class="size-5 shrink-0 rounded-full" />
                    <span class="truncate font-medium">{{ pilot.name }}</span>
                    <span v-if="pilot.corporation" class="shrink-0 text-muted-foreground">[{{ pilot.corporation.ticker }}]</span>
                    <span v-if="pilot.status?.ship_type" class="ml-auto truncate pl-2 text-muted-foreground">{{ pilot.status.ship_type.name }}</span>
                </div>
            </div>
        </TooltipContent>
    </Tooltip>
</template>

<style scoped></style>
