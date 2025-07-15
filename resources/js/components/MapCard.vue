<script setup lang="ts">
import SatelliteDish from '@/components/icons/SatelliteDish.vue';
import TelescopeIcon from '@/components/icons/TelescopeIcon.vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import {
    Dialog,
    DialogClose,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from '@/components/ui/dialog';
import { TMap } from '@/types/models';
import { Link } from '@inertiajs/vue3';
import { ref } from 'vue';

const { map } = defineProps<{
    map: TMap;
}>();

const open = ref(false);
</script>

<template>
    <Card>
        <CardHeader>
            <Link class="group flex items-center gap-3" :href="route('maps.show', map.slug)">
                <div class="rounded-lg bg-primary/10 p-2">
                    <TelescopeIcon class="h-5 w-5 text-primary" />
                </div>
                <div class="min-w-0 flex-1">
                    <CardTitle class="truncate transition-colors group-hover:text-primary">
                        {{ map.name }}
                    </CardTitle>
                </div>
            </Link>
        </CardHeader>

        <CardContent>
            <div class="grid grid-cols-2 gap-4">
                <div class="flex items-center gap-2">
                    <SatelliteDish class="h-4 w-4 text-muted-foreground" />
                    <span class="text-sm text-muted-foreground">Systems</span>
                    <Badge>
                        {{ map.map_solarsystems_count }}
                    </Badge>
                </div>
            </div>
        </CardContent>
        <CardFooter class="justify-between px-4">
            <Dialog v-model:open="open">
                <DialogTrigger>
                    <Button variant="outline" v-if="map.map_user_setting?.tracking_allowed" class="flex items-center gap-2">
                        <div class="h-2 w-2 rounded-full bg-green-500"></div>
                        <span class="text-sm text-muted-foreground">Active</span>
                    </Button>
                    <Button variant="outline" v-else class="flex items-center gap-2">
                        <div class="h-2 w-2 rounded-full bg-red-500"></div>
                        <span class="text-sm text-muted-foreground">Inactive</span>
                    </Button>
                </DialogTrigger>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle> Manage Map Tracking Consent</DialogTitle>
                        <DialogDescription>
                            <p v-if="map.map_user_setting?.tracking_allowed">
                                You have consented to allow tracking of your location on this map. This is necessary for accurate map features and
                                updates. If you change your mind, you can revoke this consent by clicking the button below. Keep in mind that revoking
                                consent will result in you no longer being able to use this map until you provide consent again.
                            </p>
                            <p v-else>
                                You have not consented to allow tracking of your location on this map. This is necessary for accurate map features and
                                updates. If you want to use this map, you must provide consent by clicking the button below.
                            </p>
                        </DialogDescription>
                    </DialogHeader>
                    <DialogFooter>
                        <DialogClose as-child>
                            <Button variant="secondary">Close</Button>
                        </DialogClose>
                        <Button as-child>
                            <Link
                                :href="route('map-user-settings.update', map.map_user_setting!.id)"
                                :data="{ tracking_allowed: !map.map_user_setting?.tracking_allowed }"
                                method="put"
                                @click="open = false"
                            >
                                {{ map.map_user_setting?.tracking_allowed ? 'Revoke Consent' : 'Provide Consent' }}
                            </Link>
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
            <Button variant="ghost" as-child>
                <Link :href="route('maps.show', map.slug)">View Map →</Link>
            </Button>
        </CardFooter>
    </Card>
</template>

<style scoped></style>
