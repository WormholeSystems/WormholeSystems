<script setup lang="ts">
import MapController from '@/actions/App/Http/Controllers/MapController';
import MapUserSettingController from '@/actions/App/Http/Controllers/MapUserSettingController';
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
        class="group relative flex flex-col overflow-hidden rounded bg-card ring-1 ring-border transition-shadow hover:ring-foreground/25"
        :class="{ 'opacity-70': isArchived }"
    >
        <!-- Panel header -->
        <div class="flex h-9 shrink-0 items-center justify-between gap-2 border-b border-border/50 bg-muted/30 pr-1.5 pl-3">
            <span class="flex min-w-0 items-center gap-1.5 font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
                <component :is="map.is_public ? Globe : Lock" class="size-3 shrink-0" />
                <span class="truncate">map · {{ map.is_public ? 'public' : 'private' }}</span>
            </span>

            <DropdownMenu>
                <DropdownMenuTrigger as-child>
                    <button class="shrink-0 rounded-sm p-1 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground">
                        <MoreVertical class="size-3.5" />
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

        <!-- Body: name and stats, one large click target -->
        <Link :href="MapController.show(map.slug)" prefetch class="group/body flex flex-1 flex-col gap-3 p-4">
            <h3 class="truncate font-display text-lg font-bold tracking-tight transition-colors group-hover/body:text-primary">
                {{ map.name }}
            </h3>
            <div class="flex flex-wrap items-center gap-x-4 gap-y-1 font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
                <span class="inline-flex items-center gap-1.5">
                    <SatelliteDish class="size-3" />
                    {{ map.map_solarsystems_count }} systems
                </span>
                <span class="inline-flex items-center gap-1.5">
                    <Spline class="size-3" />
                    {{ map.map_connections_count }} connections
                </span>
            </div>
        </Link>

        <!-- Footer -->
        <div class="flex items-center justify-between gap-2 border-t border-border/50 px-3 py-2">
            <span
                v-if="role"
                class="inline-flex items-center gap-1 rounded-sm border px-1.5 py-0.5 font-mono text-[10px] tracking-wider uppercase"
                :class="role.class"
            >
                <component :is="role.icon" class="size-3" />
                {{ role.label }}
            </span>
            <span v-else />

            <Dialog v-model:open="open">
                <DialogTrigger as-child>
                    <button
                        class="inline-flex shrink-0 items-center gap-1.5 rounded-sm border border-border/60 px-1.5 py-0.5 font-mono text-[10px] tracking-wider text-muted-foreground uppercase transition-colors hover:text-foreground"
                        :title="trackingAllowed ? 'Location sharing is on' : 'Location sharing is off'"
                    >
                        <span class="size-1.5 rounded-full" :class="trackingAllowed ? 'bg-green-500' : 'bg-red-500'" />
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
