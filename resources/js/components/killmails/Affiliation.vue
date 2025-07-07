<script setup lang="ts">
import { AllianceLogo, CharacterImage, CorporationLogo, FactionLogo, TypeImage } from '@/components/images';

defineProps<{
    alt: string;
    affiliation: {
        character_id?: number | null;
        corporation_id?: number | null;
        alliance_id?: number | null;
        faction_id?: number | null;
        type_id?: number | null;
        weapon_type_id?: number | null;
    };
}>();
</script>

<template>
    <div>
        <AllianceLogo v-if="affiliation.alliance_id" :alliance_id="affiliation.alliance_id" :alliance_name="alt" />
        <FactionLogo v-else-if="affiliation.faction_id" :faction_id="affiliation.faction_id" :faction_name="alt" />
        <CorporationLogo v-else-if="affiliation.corporation_id" :corporation_id="affiliation.corporation_id" :corporation_name="alt" />
        <CharacterImage v-else-if="affiliation.character_id" :character_id="affiliation.character_id" :character_name="alt" />
        <TypeImage v-else-if="affiliation.type_id" :type_id="affiliation.type_id" :type_name="alt" variant="icon" />
    </div>
</template>

<style scoped></style>
