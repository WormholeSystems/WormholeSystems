<script setup lang="ts">
import MapAccessController from '@/actions/App/Http/Controllers/MapAccessController';
import { Button } from '@/components/ui/button';
import { Link } from '@inertiajs/vue3';
import { AlertTriangle } from 'lucide-vue-next';

const { mapSlug, characterName, canManageAccess } = defineProps<{
    characterName: string;
    canManageAccess: boolean;
    mapSlug: string;
}>();
</script>

<template>
    <div class="mx-4 mt-4 flex items-center justify-between gap-3 rounded-lg border border-yellow-500/20 bg-yellow-500/10 px-4 py-3">
        <div class="flex items-center gap-3">
            <AlertTriangle class="h-5 w-5 shrink-0 text-yellow-500" />
            <p class="text-sm text-foreground">
                Your active character <strong>{{ characterName }}</strong> is not on this map's access list. You are viewing this map through another
                character's access.
            </p>
        </div>
        <Button v-if="canManageAccess" variant="outline" size="sm" as-child class="shrink-0">
            <Link :href="MapAccessController.show(mapSlug)">Manage Access</Link>
        </Button>
    </div>
</template>
