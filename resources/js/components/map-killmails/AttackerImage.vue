<script setup lang="ts">
import { CharacterImage, CorporationLogo } from '@/components/images';
import { TRawKillmailAttacker } from '@/types/models';
import TypeImage from '../images/TypeImage.vue';

defineProps<{
    attacker: TRawKillmailAttacker;
}>();
</script>

<template>
    <div>
        <CharacterImage v-if="attacker.character_id" :character_id="attacker.character_id" character_name="Attacker" />
        <CorporationLogo v-else-if="attacker.corporation_id" :corporation_id="attacker.corporation_id" corporation_name="Attacker Corporation" />
        <TypeImage v-else-if="attacker.ship_type_id" :type_id="attacker.ship_type_id" type_name="Attacker Ship" variant="icon" />
    </div>
</template>

<style scoped></style>
