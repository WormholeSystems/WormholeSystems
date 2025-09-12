<script setup lang="ts">
import CharacterImage from '@/components/images/CharacterImage.vue';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import AppLayout from '@/layouts/AppLayout.vue';

// Import custom icons
import ScopeController from '@/actions/App/Http/Controllers/ScopeController';
import CheckIcon from '@/components/icons/CheckIcon.vue';
import { Link } from '@inertiajs/vue3';

interface Character {
    id: number;
    name: string;
    esi_scopes: string[];
}

interface Props {
    characters: Character[];
}

const props = defineProps<Props>();

// Available scopes we manage
const managedScopes = [
    {
        scope: 'esi-location.read_online.v1',
        name: 'Online Status',
        description: 'Allows the application to see if your character is online',
    },
    {
        scope: 'esi-location.read_location.v1',
        name: 'Character Location',
        description: "Allows the application to see your character's current location",
    },
    {
        scope: 'esi-location.read_ship_type.v1',
        name: 'Ship Information',
        description: 'Allows the application to see what ship your character is flying',
    },
    {
        scope: 'esi-ui.write_waypoint.v1',
        name: 'Waypoint Setting',
        description: 'Allows the application to set waypoints in your EVE client',
    },
];

function hasScope(character: Character, scopeName: string): boolean {
    return character.esi_scopes.includes(scopeName);
}
</script>

<template>
    <AppLayout>
        <div class="container mx-auto space-y-6 p-6">
            <div class="mb-8">
                <h1 class="text-3xl font-bold">ESI Scope Management</h1>
                <p class="mt-2 max-w-xl text-muted-foreground">
                    Manage EVE Online API permissions for your characters. Grant only the permissions you need for the features you want to use.
                </p>
            </div>

            <div class="space-y-6">
                <Card v-for="character in props.characters" :key="character.id">
                    <CardHeader>
                        <div class="flex items-center justify-between">
                            <div class="flex items-center space-x-3">
                                <CharacterImage :character_id="character.id" :character_name="character.name" class="size-10 rounded-full" />
                                <CardTitle>{{ character.name }}</CardTitle>
                            </div>
                            <Button v-if="managedScopes.some((scope) => !hasScope(character, scope.scope))" size="sm" as-child>
                                <a :href="ScopeController.show({ query: { scopes: managedScopes.map((scope) => scope.scope).join(',') } }).url">
                                    Grant all
                                </a>
                            </Button>
                        </div>
                    </CardHeader>
                    <CardContent>
                        <!-- Individual Scopes -->
                        <div
                            v-for="scopeData in managedScopes"
                            :key="scopeData.scope"
                            class="flex items-center justify-between border-b py-4 last:border-b-0"
                        >
                            <div>
                                <div class="font-medium">{{ scopeData.name }}</div>
                                <div class="text-sm text-muted-foreground">{{ scopeData.description }}</div>
                            </div>
                            <div class="flex items-center space-x-2">
                                <CheckIcon v-if="hasScope(character, scopeData.scope)" class="size-5 text-green-600" />
                                <Button v-else size="sm" variant="outline" as-child>
                                    <a
                                        :href="
                                            ScopeController.show({
                                                query: { scopes: scopeData.scope },
                                            }).url
                                        "
                                    >
                                        Grant scope
                                    </a>
                                </Button>
                            </div>
                        </div>
                    </CardContent>
                    <CardFooter class="flex justify-end">
                        <Button variant="link" as-child class="text-destructive">
                            <Link :href="ScopeController.destroy(character)" method="delete"> Revoke all scopes </Link>
                        </Button>
                    </CardFooter>
                </Card>
            </div>
        </div>
    </AppLayout>
</template>
