<script setup lang="ts">
import MapAccessController from '@/actions/App/Http/Controllers/MapAccessController';
import EditIcon from '@/components/icons/EditIcon.vue';
import EyeIcon from '@/components/icons/EyeIcon.vue';
import InfoIcon from '@/components/icons/InfoIcon.vue';
import { AllianceLogo, CharacterImage, CorporationLogo } from '@/components/images';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { useSearch } from '@/composables/useSearch';
import SettingsLayout from '@/layouts/SettingsLayout.vue';
import { TMap } from '@/types/models';
import { router } from '@inertiajs/vue3';

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
        MapAccessController.store(map.slug),
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
    <SettingsLayout :map="map" title="Access Control" description="Manage who can view and edit this map">
        <div class="space-y-6">
            <Card>
                <CardHeader>
                    <CardTitle>Permission Levels</CardTitle>
                    <CardDescription>Understanding access levels</CardDescription>
                </CardHeader>
                <CardContent>
                    <div class="space-y-4">
                        <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                            <div class="rounded-lg border border-border/50 bg-muted/30 p-4">
                                <div class="mb-3 flex items-center gap-2">
                                    <EyeIcon />
                                    <h4 class="font-medium">Read Access</h4>
                                </div>
                                <ul class="space-y-1 text-sm text-muted-foreground">
                                    <li>• View map and systems</li>
                                    <li>• See signatures and connections</li>
                                    <li>• View killmails and character locations</li>
                                </ul>
                            </div>
                            <div class="rounded-lg border border-border/50 bg-muted/30 p-4">
                                <div class="mb-3 flex items-center gap-2">
                                    <EditIcon />
                                    <h4 class="font-medium">Write Access</h4>
                                </div>
                                <ul class="space-y-1 text-sm text-muted-foreground">
                                    <li>• All read permissions</li>
                                    <li>• Add/edit signatures</li>
                                    <li>• Create/modify connections</li>
                                    <li>• Add systems to the map</li>
                                </ul>
                            </div>
                        </div>
                    </div>
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>Map Permissions</CardTitle>
                    <CardDescription>Control who can access and modify this map</CardDescription>
                </CardHeader>
                <CardContent>
                    <div class="space-y-4">
                        <Input v-model="search" placeholder="Search characters, corporations, or alliances..." />

                        <div class="flex items-center gap-2 border-l-2 border-l-orange-500 bg-orange-500/10 p-4 text-sm text-orange-500">
                            <InfoIcon />
                            <p>Only characters, corporations, and alliances that have registered at least once will be shown here.</p>
                        </div>

                        <div class="rounded-lg border border-border/50">
                            <Table>
                                <TableHeader>
                                    <TableRow class="border-border/50">
                                        <TableHead class="w-12"></TableHead>
                                        <TableHead>Name</TableHead>
                                        <TableHead>Type</TableHead>
                                        <TableHead>Access</TableHead>
                                    </TableRow>
                                </TableHeader>
                                <TableBody>
                                    <TableRow v-for="entity in entities" :key="entity.id" class="border-border/50">
                                        <TableCell>
                                            <template v-if="entity.type === 'character'">
                                                <CharacterImage :character_id="entity.id" :character_name="entity.name" class="size-8 rounded-lg" />
                                            </template>
                                            <template v-else-if="entity.type === 'corporation'">
                                                <CorporationLogo
                                                    :corporation_id="entity.id"
                                                    :corporation_name="entity.name"
                                                    class="size-8 rounded-lg"
                                                />
                                            </template>
                                            <template v-else-if="entity.type === 'alliance'">
                                                <AllianceLogo :alliance_id="entity.id" :alliance_name="entity.name" class="size-8 rounded-lg" />
                                            </template>
                                        </TableCell>
                                        <TableCell>{{ entity.name }}</TableCell>
                                        <TableCell class="capitalize">{{ entity.type }}</TableCell>
                                        <TableCell>
                                            <div class="flex items-center gap-6">
                                                <Label class="flex cursor-pointer items-center gap-2 text-sm">
                                                    <Checkbox
                                                        :model-value="entity.permission === 'read' || entity.permission === 'write'"
                                                        @update:modelValue="toggleAccess(entity, entity.permission === 'read' ? null : 'read')"
                                                    />
                                                    Read
                                                </Label>
                                                <Label class="flex cursor-pointer items-center gap-2 text-sm">
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
                </CardContent>
            </Card>
        </div>
    </SettingsLayout>
</template>
