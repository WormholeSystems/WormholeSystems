<script setup lang="ts">
import { AllianceLogo, CharacterImage, CorporationLogo } from '@/components/images';
import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { useSearch } from '@/composables/useSearch';
import AppLayout from '@/layouts/AppLayout.vue';
import MapAccess from '@/routes/map-access';
import { TMap } from '@/types/models';
import { Head, router } from '@inertiajs/vue3';

type TEntity = {
    id: number;
    name: string;
    type: 'character' | 'corporation' | 'alliance';
    permission: 'read' | 'write' | null;
};

const { map, entities } = defineProps<{
    map: TMap;
    entities: TEntity[];
}>();

const search = useSearch();

function toggleAccess(entity: TEntity, permission: 'read' | 'write' | null) {
    router.post(
        MapAccess.store(map.slug).url,
        {
            entity_id: entity.id,
            entity_type: entity.type,
            permission: permission,
        },
        {
            preserveState: true,
            preserveScroll: true,
        },
    );
}
</script>

<template>
    <AppLayout>
        <Head title="Manage Map Access" />
        <div class="mx-auto mt-8 grid max-w-4xl gap-4">
            <Input v-model="search" placeholder="Search..." />
            <div class="rounded-lg border">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead></TableHead>
                            <TableHead>Name</TableHead>
                            <TableHead>Type</TableHead>
                            <TableHead>Access</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        <TableRow v-for="entity in entities" :key="entity.id">
                            <TableCell>
                                <template v-if="entity.type === 'character'">
                                    <CharacterImage :character_id="entity.id" :character_name="entity.name" class="size-8 rounded-lg" />
                                </template>
                                <template v-else-if="entity.type === 'corporation'">
                                    <CorporationLogo :corporation_id="entity.id" :corporation_name="entity.name" class="size-8 rounded-lg" />
                                </template>
                                <template v-else-if="entity.type === 'alliance'">
                                    <AllianceLogo :alliance_id="entity.id" :alliance_name="entity.name" class="size-8 rounded-lg" />
                                </template>
                            </TableCell>
                            <TableCell>{{ entity.name }}</TableCell>
                            <TableCell class="capitalize">{{ entity.type }}</TableCell>
                            <TableCell>
                                <div class="flex items-center gap-4">
                                    <Label>
                                        <Checkbox
                                            :model-value="entity.permission === 'read' || entity.permission === 'write'"
                                            @update:modelValue="toggleAccess(entity, entity.permission === 'read' ? null : 'read')"
                                        />
                                        Read
                                    </Label>
                                    <Label>
                                        <Checkbox
                                            :model-value="entity.permission === 'write'"
                                            @update:modelValue="toggleAccess(entity, entity.permission === 'write' ? null : 'write')"
                                        />
                                        Write
                                    </Label>
                                </div>
                            </TableCell>
                        </TableRow>
                    </TableBody>
                </Table>
            </div>
        </div>
    </AppLayout>
</template>
