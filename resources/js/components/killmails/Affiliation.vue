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
        <a v-if="affiliation.alliance_id" :href="`https://zkillboard.com/alliance/${affiliation.alliance_id}/`" target="_blank" rel="noopener noreferrer">
            <AllianceLogo :alliance_id="affiliation.alliance_id" :alliance_name="alt" />
        </a>
        <a v-else-if="affiliation.faction_id" :href="`https://zkillboard.com/faction/${affiliation.faction_id}/`" target="_blank" rel="noopener noreferrer">
            <FactionLogo :faction_id="affiliation.faction_id" :faction_name="alt" />
        </a>
        <a v-else-if="affiliation.corporation_id" :href="`https://zkillboard.com/corporation/${affiliation.corporation_id}/`" target="_blank" rel="noopener noreferrer">
            <CorporationLogo :corporation_id="affiliation.corporation_id" :corporation_name="alt" />
        </a>
        <a v-else-if="affiliation.character_id" :href="`https://zkillboard.com/character/${affiliation.character_id}/`" target="_blank" rel="noopener noreferrer">
            <CharacterImage :character_id="affiliation.character_id" :character_name="alt" />
        </a>
        <a v-else-if="affiliation.type_id" :href="`https://zkillboard.com/ship/${affiliation.type_id}/`" target="_blank" rel="noopener noreferrer">
            <TypeImage :type_id="affiliation.type_id" :type_name="alt" variant="icon" />
        </a>
    </div>
</template>

<style scoped></style>
