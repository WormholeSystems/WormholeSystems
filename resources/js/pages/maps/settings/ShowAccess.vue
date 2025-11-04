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
import { TMapSummary } from '@/pages/maps';
import { router } from '@inertiajs/vue3';

type TEntity = {
    id: number;
    name: string;
    type: 'character' | 'corporation' | 'alliance';
    permission: 'guest' | 'read' | 'write' | null;
};

const { map, entities, entitiesWithAccess } = defineProps<{
    map: TMapSummary;
    entities: TEntity[];
    entitiesWithAccess: TEntity[];
}>();

const search = useSearch();

function toggleAccess(entity: TEntity, permission: 'guest' | 'read' | 'write' | null) {
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
                    <CardTitle>Entities with Access</CardTitle>
                    <CardDescription>All entities that currently have access to this map</CardDescription>
                </CardHeader>
                <CardContent>
                    <div class="space-y-4">
                        <div
                            v-if="entitiesWithAccess.length === 0"
                            class="flex items-center gap-2 border-l-2 border-l-blue-500 bg-blue-500/10 p-4 text-sm text-blue-500"
                        >
                            <InfoIcon />
                            <p>No entities currently have access to this map.</p>
                        </div>

                        <div v-else class="rounded-lg border border-border/50">
                            <Table>
                                <TableHeader>
                                    <TableRow class="border-border/50">
                                        <TableHead class="w-12"></TableHead>
                                        <TableHead>Name</TableHead>
                                        <TableHead>Type</TableHead>
                                        <TableHead>Access Level</TableHead>
                                    </TableRow>
                                </TableHeader>
                                <TableBody>
                                    <TableRow v-for="entity in entitiesWithAccess" :key="entity.id" class="border-border/50">
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
                                            <div class="flex items-center gap-2">
                                                <div
                                                    v-if="entity.permission === 'write'"
                                                    class="inline-flex items-center gap-1 rounded-full bg-green-500/10 px-2 py-1 text-xs font-medium text-green-500"
                                                >
                                                    <EditIcon class="size-3" />
                                                    Write Access
                                                </div>
                                                <div
                                                    v-else-if="entity.permission === 'read'"
                                                    class="inline-flex items-center gap-1 rounded-full bg-blue-500/10 px-2 py-1 text-xs font-medium text-blue-500"
                                                >
                                                    <EyeIcon class="size-3" />
                                                    Read Access
                                                </div>
                                                <div
                                                    v-else-if="entity.permission === 'guest'"
                                                    class="inline-flex items-center gap-1 rounded-full bg-gray-500/10 px-2 py-1 text-xs font-medium text-gray-500"
                                                >
                                                    <EyeIcon class="size-3" />
                                                    Guest Access
                                                </div>
                                            </div>
                                        </TableCell>
                                    </TableRow>
                                </TableBody>
                            </Table>
                        </div>
                    </div>
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>Permission Levels</CardTitle>
                    <CardDescription>Understanding access levels</CardDescription>
                </CardHeader>
                <CardContent>
                    <div class="space-y-4">
                        <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
                            <div class="rounded-lg border border-border/50 bg-muted/30 p-4">
                                <div class="mb-3 flex items-center gap-2">
                                    <EyeIcon />
                                    <h4 class="font-medium">Guest Access</h4>
                                </div>
                                <ul class="space-y-1 text-sm text-muted-foreground">
                                    <li>• View map and systems</li>
                                    <li>• See signatures and connections</li>
                                    <li>• Cannot see character locations</li>
                                    <li>• Cannot see audits or ship history</li>
                                    <li>• Cannot see or edit notes</li>
                                    <li>• Cannot make any changes</li>
                                </ul>
                            </div>
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
                    <CardTitle>Manage Permissions</CardTitle>
                    <CardDescription>Search and grant access to characters, corporations, and alliances</CardDescription>
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
                                                        :model-value="entity.permission === 'guest'"
                                                        @update:modelValue="toggleAccess(entity, entity.permission === 'guest' ? null : 'guest')"
                                                    />
                                                    Guest
                                                </Label>
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
