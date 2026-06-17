<script setup lang="ts">
import MapController from '@/actions/App/Http/Controllers/MapController';
import MapUserSettingController from '@/actions/App/Http/Controllers/MapUserSettingController';
import Logo from '@/components/icons/Logo.vue';
import SatelliteDish from '@/components/icons/SatelliteDish.vue';
import { Button } from '@/components/ui/button';
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
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { TMapSummary } from '@/pages/maps';
import { Link, router } from '@inertiajs/vue3';
import { Archive, ArchiveRestore, ArrowRight, Crown, Eye, Globe, Lock, MoreVertical, Pencil, Settings, Spline } from 'lucide-vue-next';
import { computed, ref, type Component } from 'vue';

const { map } = defineProps<{
    map: TMapSummary;
}>();

const open = ref(false);

const isArchived = computed(() => Boolean(map.map_user_setting?.is_archived));
const trackingAllowed = computed(() => Boolean(map.map_user_setting?.tracking_allowed));

const roleMeta: Record<NonNullable<TMapSummary['role']>, { label: string; icon: Component; class: string }> = {
    owner: { label: 'Owner', icon: Crown, class: 'border-amber-500/30 bg-amber-500/10 text-amber-500' },
    manager: { label: 'Manager', icon: Settings, class: 'border-purple-400/30 bg-purple-400/10 text-purple-400' },
    member: { label: 'Member', icon: Pencil, class: 'border-green-400/30 bg-green-400/10 text-green-400' },
    viewer: { label: 'Viewer', icon: Eye, class: 'border-blue-400/30 bg-blue-400/10 text-blue-400' },
};

const role = computed(() => (map.role ? roleMeta[map.role] : null));

function setArchived(value: boolean): void {
    router.put(MapUserSettingController.update(map.slug).url, { is_archived: value }, { preserveScroll: true, only: ['maps'] });
}
</script>

<template>
    <div
        class="group relative flex flex-col rounded-xl border border-border/60 bg-card p-5 transition-all hover:border-border hover:shadow-lg hover:shadow-black/5"
        :class="{ 'opacity-70': isArchived }"
    >
        <!-- Header -->
        <div class="flex items-start justify-between gap-3">
            <Link :href="MapController.show(map.slug)" prefetch class="group/title flex min-w-0 items-center gap-3">
                <div class="rounded-lg bg-primary/10 p-2">
                    <Logo class="h-5 w-5 text-primary" />
                </div>
                <h3 class="truncate font-semibold tracking-tight transition-colors group-hover/title:text-primary">{{ map.name }}</h3>
            </Link>

            <DropdownMenu>
                <DropdownMenuTrigger as-child>
                    <button class="-mr-1 shrink-0 rounded-md p-1.5 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground">
                        <MoreVertical class="size-4" />
                    </button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" class="w-48">
                    <DropdownMenuItem as-child>
                        <Link :href="MapController.show(map.slug)" prefetch class="flex w-full items-center gap-2">
                            <ArrowRight class="size-4" />
                            Open map
                        </Link>
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem v-if="!isArchived" @select="setArchived(true)">
                        <Archive class="size-4" />
                        Archive map
                    </DropdownMenuItem>
                    <DropdownMenuItem v-else @select="setArchived(false)">
                        <ArchiveRestore class="size-4" />
                        Unarchive map
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
        </div>

        <!-- Stats -->
        <div class="mt-5 flex flex-wrap items-center gap-2">
            <span class="inline-flex items-center gap-1.5 rounded-md border border-border/60 bg-muted/30 px-2 py-1 text-xs font-medium">
                <SatelliteDish class="size-3.5 text-muted-foreground" />
                {{ map.map_solarsystems_count }}
                <span class="text-muted-foreground">systems</span>
            </span>
            <span class="inline-flex items-center gap-1.5 rounded-md border border-border/60 bg-muted/30 px-2 py-1 text-xs font-medium">
                <Spline class="size-3.5 text-muted-foreground" />
                {{ map.map_connections_count }}
                <span class="text-muted-foreground">connections</span>
            </span>
        </div>

        <!-- Footer -->
        <div class="mt-5 flex items-center justify-between gap-2 border-t border-border/50 pt-4">
            <div class="flex min-w-0 items-center gap-2">
                <span v-if="role" class="inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-[11px] font-medium" :class="role.class">
                    <component :is="role.icon" class="size-3" />
                    {{ role.label }}
                </span>
                <span class="inline-flex items-center gap-1 text-xs text-muted-foreground">
                    <component :is="map.is_public ? Globe : Lock" class="size-3.5" />
                    {{ map.is_public ? 'Public' : 'Private' }}
                </span>
            </div>

            <Dialog v-model:open="open">
                <DialogTrigger as-child>
                    <button
                        class="inline-flex shrink-0 items-center gap-1.5 rounded-full border border-border/60 px-2 py-0.5 text-xs text-muted-foreground transition-colors hover:text-foreground"
                        :title="trackingAllowed ? 'Location sharing is on' : 'Location sharing is off'"
                    >
                        <span class="size-2 rounded-full" :class="trackingAllowed ? 'bg-green-500' : 'bg-red-500'" />
                        {{ trackingAllowed ? 'Tracking' : 'Hidden' }}
                    </button>
                </DialogTrigger>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>Manage map tracking consent</DialogTitle>
                        <DialogDescription>
                            <span v-if="trackingAllowed">
                                You have consented to share your location on this map. Other users may see your location and movements. Revoke consent
                                with the button below.
                            </span>
                            <span v-else>
                                You have not consented to share your location on this map. Other users won't see your location, but some features will
                                be unavailable. Provide consent with the button below.
                            </span>
                        </DialogDescription>
                    </DialogHeader>
                    <DialogFooter>
                        <DialogClose as-child>
                            <Button variant="secondary">Close</Button>
                        </DialogClose>
                        <Button as-child>
                            <Link
                                :href="MapUserSettingController.update(map.slug).url"
                                :data="{ tracking_allowed: !trackingAllowed }"
                                method="put"
                                preserve-scroll
                                :only="['maps']"
                                @click="open = false"
                            >
                                {{ trackingAllowed ? 'Revoke consent' : 'Provide consent' }}
                            </Link>
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </div>
    </div>
</template>
