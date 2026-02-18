<script setup lang="ts">
import MapAccessController from '@/actions/App/Http/Controllers/MapAccessController';
import AccessExpiryPicker from '@/components/access/AccessExpiryPicker.vue';
import EditIcon from '@/components/icons/EditIcon.vue';
import EyeIcon from '@/components/icons/EyeIcon.vue';
import InfoIcon from '@/components/icons/InfoIcon.vue';
import { AllianceLogo, CharacterImage, CorporationLogo } from '@/components/images';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger } from '@/components/ui/select';
import { Separator } from '@/components/ui/separator';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { useSearch } from '@/composables/useSearch';
import SettingsLayout from '@/layouts/SettingsLayout.vue';
import { TMapSummary } from '@/pages/maps';
import { router } from '@inertiajs/vue3';
import { Ban, Crown, Eye, Pencil, Settings, Shield } from 'lucide-vue-next';

type TSearchEntity = {
    id: number;
    name: string;
    type: 'character' | 'corporation' | 'alliance';
    permission: 'viewer' | 'member' | 'manager' | null;
    is_owner: boolean;
};

type TAccessEntity = TSearchEntity & {
    expires_at: string | null;
};

const { map, entities, entitiesWithAccess } = defineProps<{
    map: TMapSummary;
    entities: TSearchEntity[];
    entitiesWithAccess: TAccessEntity[];
}>();

const search = useSearch();

function toggleAccess(entity: TSearchEntity | TAccessEntity, permission: 'viewer' | 'member' | 'manager' | null, expires_at?: string | null) {
    router.post(
        MapAccessController.store(map.slug),
        {
            entity_id: entity.id,
            entity_type: entity.type,
            permission: permission,
            expires_at: expires_at !== undefined ? expires_at : 'expires_at' in entity ? entity.expires_at : null,
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
        <div class="space-y-8">
            <!-- Section 1: Entities with Access -->
            <div>
                <h3 class="text-lg font-semibold">Entities with Access</h3>
                <p class="text-sm text-muted-foreground">All entities that currently have access to this map</p>

                <div v-if="entitiesWithAccess.length === 0" class="mt-4 flex flex-col items-center justify-center py-8 text-muted-foreground">
                    <Shield class="mb-2 size-8" />
                    <p class="text-sm">No entities currently have access to this map.</p>
                </div>

                <div v-else class="mt-4 rounded-lg border border-border/50">
                    <Table>
                        <TableHeader>
                            <TableRow class="border-border/50">
                                <TableHead class="w-12"></TableHead>
                                <TableHead>Name</TableHead>
                                <TableHead>Type</TableHead>
                                <TableHead>Access Level</TableHead>
                                <TableHead>Expires</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            <TableRow v-for="entity in entitiesWithAccess" :key="entity.id" class="border-border/50">
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
                                <TableCell class="font-medium">{{ entity.name }}</TableCell>
                                <TableCell>
                                    <Badge variant="outline" class="capitalize">{{ entity.type }}</Badge>
                                </TableCell>
                                <TableCell>
                                    <Badge v-if="entity.is_owner" class="bg-amber-500/10 text-amber-500 hover:bg-amber-500/10">
                                        <Crown class="mr-1 size-3" />
                                        Owner
                                    </Badge>
                                    <Select
                                        v-else
                                        :model-value="entity.permission ?? 'none'"
                                        @update:model-value="
                                            toggleAccess(entity, $event === 'none' ? null : ($event as 'viewer' | 'member' | 'manager'))
                                        "
                                    >
                                        <SelectTrigger class="w-36" size="sm">
                                            <span class="flex items-center gap-2">
                                                <Ban v-if="!entity.permission" class="size-4 text-muted-foreground" />
                                                <Eye v-else-if="entity.permission === 'viewer'" class="size-4 text-blue-500" />
                                                <Pencil v-else-if="entity.permission === 'member'" class="size-4 text-green-500" />
                                                <Settings v-else-if="entity.permission === 'manager'" class="size-4 text-purple-500" />
                                                {{
                                                    entity.permission
                                                        ? entity.permission.charAt(0).toUpperCase() + entity.permission.slice(1)
                                                        : 'None'
                                                }}
                                            </span>
                                        </SelectTrigger>
                                        <SelectContent>
                                            <SelectItem value="none">
                                                <span class="flex items-center gap-2">
                                                    <Ban class="size-4 text-muted-foreground" />
                                                    None
                                                </span>
                                            </SelectItem>
                                            <SelectItem value="viewer">
                                                <span class="flex items-center gap-2">
                                                    <Eye class="size-4 text-blue-500" />
                                                    Viewer
                                                </span>
                                            </SelectItem>
                                            <SelectItem value="member">
                                                <span class="flex items-center gap-2">
                                                    <Pencil class="size-4 text-green-500" />
                                                    Member
                                                </span>
                                            </SelectItem>
                                            <SelectItem value="manager">
                                                <span class="flex items-center gap-2">
                                                    <Settings class="size-4 text-purple-500" />
                                                    Manager
                                                </span>
                                            </SelectItem>
                                        </SelectContent>
                                    </Select>
                                </TableCell>
                                <TableCell>
                                    <AccessExpiryPicker
                                        v-if="!entity.is_owner && entity.permission !== null"
                                        :expires-at="entity.expires_at"
                                        @update="toggleAccess(entity, entity.permission, $event)"
                                    />
                                </TableCell>
                            </TableRow>
                        </TableBody>
                    </Table>
                </div>
            </div>

            <Separator />

            <!-- Section 2: Permission Levels -->
            <div>
                <h3 class="text-lg font-semibold">Permission Levels</h3>
                <p class="text-sm text-muted-foreground">Understanding access levels</p>

                <div class="mt-4 grid grid-cols-2 gap-4">
                    <div class="border border-l-2 border-border/50 border-l-blue-500 bg-muted/30 p-4">
                        <div class="mb-3 flex items-center gap-2">
                            <EyeIcon />
                            <h4 class="font-medium">Viewer</h4>
                        </div>
                        <ul class="space-y-1 text-sm text-muted-foreground">
                            <li>• View map and systems</li>
                            <li>• See signatures and connections</li>
                            <li>• View killmails and EVE Scout</li>
                            <li>• Cannot see character locations</li>
                            <li>• Cannot see audits or ship history</li>
                            <li>• Cannot see or edit notes</li>
                            <li>• Cannot make any changes</li>
                        </ul>
                    </div>
                    <div class="border border-l-2 border-border/50 border-l-green-500 bg-muted/30 p-4">
                        <div class="mb-3 flex items-center gap-2">
                            <EditIcon />
                            <h4 class="font-medium">Member</h4>
                        </div>
                        <ul class="space-y-1 text-sm text-muted-foreground">
                            <li>• All Viewer permissions</li>
                            <li>• View character locations and intel</li>
                            <li>• Add/edit signatures</li>
                            <li>• Create/modify connections</li>
                            <li>• Add systems to the map</li>
                            <li>• Edit notes</li>
                        </ul>
                    </div>
                    <div class="border border-l-2 border-border/50 border-l-purple-500 bg-muted/30 p-4">
                        <div class="mb-3 flex items-center gap-2">
                            <Settings class="size-4" />
                            <h4 class="font-medium">Manager</h4>
                        </div>
                        <ul class="space-y-1 text-sm text-muted-foreground">
                            <li>• All Member permissions</li>
                            <li>• Grant and revoke permissions</li>
                            <li>• Configure map settings</li>
                            <li>• Manage public access</li>
                            <li>• Generate share links</li>
                        </ul>
                    </div>
                    <div class="border border-l-2 border-border/50 border-l-amber-500 bg-muted/30 p-4">
                        <div class="mb-3 flex items-center gap-2">
                            <Crown class="size-4" />
                            <h4 class="font-medium">Owner</h4>
                        </div>
                        <ul class="space-y-1 text-sm text-muted-foreground">
                            <li>• All Manager permissions</li>
                            <li>• Delete the map</li>
                            <li>• Cannot be removed</li>
                            <li>• One per map</li>
                        </ul>
                    </div>
                </div>
            </div>

            <Separator />

            <!-- Section 3: Manage Permissions -->
            <div>
                <h3 class="text-lg font-semibold">Manage Permissions</h3>
                <p class="text-sm text-muted-foreground">Search and grant access to characters, corporations, and alliances</p>

                <div class="mt-4 space-y-4">
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
                                            <CorporationLogo :corporation_id="entity.id" :corporation_name="entity.name" class="size-8 rounded-lg" />
                                        </template>
                                        <template v-else-if="entity.type === 'alliance'">
                                            <AllianceLogo :alliance_id="entity.id" :alliance_name="entity.name" class="size-8 rounded-lg" />
                                        </template>
                                    </TableCell>
                                    <TableCell class="font-medium">{{ entity.name }}</TableCell>
                                    <TableCell>
                                        <Badge variant="outline" class="capitalize">{{ entity.type }}</Badge>
                                    </TableCell>
                                    <TableCell>
                                        <Badge v-if="entity.is_owner" class="bg-amber-500/10 text-amber-500 hover:bg-amber-500/10">
                                            <Crown class="mr-1 size-3" />
                                            Owner
                                        </Badge>
                                        <Select
                                            v-else
                                            :model-value="entity.permission ?? 'none'"
                                            @update:model-value="
                                                toggleAccess(entity, $event === 'none' ? null : ($event as 'viewer' | 'member' | 'manager'))
                                            "
                                        >
                                            <SelectTrigger class="w-36" size="sm">
                                                <span class="flex items-center gap-2">
                                                    <Ban v-if="!entity.permission" class="size-4 text-muted-foreground" />
                                                    <Eye v-else-if="entity.permission === 'viewer'" class="size-4 text-blue-500" />
                                                    <Pencil v-else-if="entity.permission === 'member'" class="size-4 text-green-500" />
                                                    <Settings v-else-if="entity.permission === 'manager'" class="size-4 text-purple-500" />
                                                    {{
                                                        entity.permission
                                                            ? entity.permission.charAt(0).toUpperCase() + entity.permission.slice(1)
                                                            : 'None'
                                                    }}
                                                </span>
                                            </SelectTrigger>
                                            <SelectContent>
                                                <SelectItem value="none">
                                                    <span class="flex items-center gap-2">
                                                        <Ban class="size-4 text-muted-foreground" />
                                                        None
                                                    </span>
                                                </SelectItem>
                                                <SelectItem value="viewer">
                                                    <span class="flex items-center gap-2">
                                                        <Eye class="size-4 text-blue-500" />
                                                        Viewer
                                                    </span>
                                                </SelectItem>
                                                <SelectItem value="member">
                                                    <span class="flex items-center gap-2">
                                                        <Pencil class="size-4 text-green-500" />
                                                        Member
                                                    </span>
                                                </SelectItem>
                                                <SelectItem value="manager">
                                                    <span class="flex items-center gap-2">
                                                        <Settings class="size-4 text-purple-500" />
                                                        Manager
                                                    </span>
                                                </SelectItem>
                                            </SelectContent>
                                        </Select>
                                    </TableCell>
                                </TableRow>
                            </TableBody>
                        </Table>
                    </div>
                </div>
            </div>
        </div>
    </SettingsLayout>
</template>
