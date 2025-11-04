<script setup lang="ts">
import MapController from '@/actions/App/Http/Controllers/MapController';
import Logo from '@/components/icons/Logo.vue';
import SatelliteDish from '@/components/icons/SatelliteDish.vue';
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
import { TMapSummary } from '@/pages/maps';
import MapUserSettings from '@/routes/map-user-settings';
import { Link } from '@inertiajs/vue3';
import { ref } from 'vue';

const { map } = defineProps<{
    map: TMapSummary;
}>();

const open = ref(false);
</script>

<template>
    <Card>
        <CardHeader>
            <Link class="group flex items-center gap-3" :href="MapController.show(map.slug)">
                <div class="rounded-lg bg-primary/10 p-2">
                    <Logo class="h-5 w-5 text-primary" />
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
                    <Button variant="secondary" v-if="map.map_user_setting?.tracking_allowed" class="flex items-center gap-2">
                        <div class="h-2 w-2 rounded-full bg-green-500"></div>
                        <span class="text-sm text-muted-foreground">Active</span>
                    </Button>
                    <Button variant="secondary" v-else class="flex items-center gap-2">
                        <div class="h-2 w-2 rounded-full bg-red-500"></div>
                        <span class="text-sm text-muted-foreground">Inactive</span>
                    </Button>
                </DialogTrigger>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle> Manage Map Tracking Consent</DialogTitle>
                        <DialogDescription>
                            <p v-if="map.map_user_setting?.tracking_allowed">
                                You have consented to allow tracking of your location on this map. Other users may see your location and movements. If
                                you want to revoke this consent, click the button below.
                            </p>
                            <p v-else>
                                You have not consented to allow tracking of your location on this map. Other users will not see your location and
                                movements, but you won't be able to use certain features. If you want to provide consent, click the button below.
                            </p>
                        </DialogDescription>
                    </DialogHeader>
                    <DialogFooter>
                        <DialogClose as-child>
                            <Button variant="secondary">Close</Button>
                        </DialogClose>
                        <Button as-child>
                            <Link
                                :href="MapUserSettings.update(map.map_user_setting!.id)"
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
                <Link :href="MapController.show(map.slug)">View Map →</Link>
            </Button>
        </CardFooter>
    </Card>
</template>

<style scoped></style>
