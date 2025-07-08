<script setup lang="ts">
import { CharacterImage } from '@/components/images';
import type { User } from '@/types';

interface Props {
    user: User;
}

const { user } = defineProps<Props>();
</script>

<template>
    <CharacterImage
        v-if="user.active_character"
        :character_id="user.active_character.id"
        :character_name="user.active_character.name"
        class="size-8 rounded-lg"
    />

    <div class="grid flex-1 text-left text-sm leading-tight">
        <span class="truncate font-medium">{{ user.active_character?.name }}</span>
        <span class="truncate text-xs text-muted-foreground">{{ user.active_character?.corporation?.name }}</span>
    </div>
</template>
