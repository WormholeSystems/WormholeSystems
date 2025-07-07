<script setup lang="ts">
import { CharacterImage, CorporationLogo } from '@/components/images';
import TypeImage from '@/components/images/TypeImage.vue';
import { TRawKillmail } from '@/types/models';

defineProps<{
    victim: TRawKillmail['victim'];
}>();
</script>

<template>
    <div>
        <CharacterImage v-if="victim.character_id" :character_id="victim.character_id" character_name="Victim" />
        <CorporationLogo v-else-if="victim.corporation_id" :corporation_id="victim.corporation_id" corporation_name="Victim Corporation" />
        <TypeImage v-else-if="victim.ship_type_id" :type_id="victim.ship_type_id" type_name="Victim Ship" variant="icon" />
    </div>
</template>

<style scoped></style>
