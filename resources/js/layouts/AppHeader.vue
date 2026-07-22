<script setup lang="ts">
import DocumentationController from '@/actions/App/Http/Controllers/DocumentationController';
import LoginController from '@/actions/App/Http/Controllers/LoginController';
import MapController from '@/actions/App/Http/Controllers/MapController';
import SettingsController from '@/actions/App/Http/Controllers/SettingsController';
import { CharacterImage } from '@/components/images';
import ServerStatus from '@/components/server-status/ServerStatus.vue';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from '@/components/ui/sheet';
import UserMenuContent from '@/components/user/UserMenuContent.vue';
import useUser from '@/composables/useUser';
import Appearance from '@/layouts/Appearance.vue';
import AppLogo from '@/layouts/AppLogo.vue';
import { home } from '@/routes';
import type { AppPageProps, NavItem } from '@/types';
import { TCharacter } from '@/types/models';
import { Link, usePage } from '@inertiajs/vue3';
import { AlertTriangle, CircleHelp, LayoutGrid, LogIn, Map as MapIcon, Menu, Shield } from 'lucide-vue-next';
import { computed } from 'vue';

const page = usePage<
    AppPageProps<{
        missing_scopes: TCharacter[];
    }>
>();
const user = useUser();

const isCurrentRoute = computed(() => (url: string) => page.url === url);

const pinnedMaps = computed(() => page.props.pinned_maps ?? []);

/**
 * Only a handful of pinned maps render as inline navbar links; the rest move
 * into a "+N" dropdown so they can never crowd the server status column.
 */
const inline_pinned_limit = 4;
const inlinePinnedMaps = computed(() => pinnedMaps.value.slice(0, inline_pinned_limit));
const overflowPinnedMaps = computed(() => pinnedMaps.value.slice(inline_pinned_limit));

const mainNavItems: NavItem[] = [
    {
        title: 'Maps',
        href: MapController.index().url,
        icon: LayoutGrid,
    },
];

const documentationItem: NavItem = {
    title: 'Documentation',
    href: DocumentationController.index().url,
    icon: CircleHelp,
};
</script>

<template>
    <div>
        <div class="border-b border-border/50 bg-card">
            <div class="grid h-16 grid-cols-[1fr_auto] items-center px-4 md:grid-cols-[1fr_auto_1fr]">
                <!-- Left: Mobile menu + Logo + Divider + Desktop nav -->
                <div class="flex min-w-0 items-center gap-4">
                    <!-- Mobile Menu -->
                    <div class="lg:hidden">
                        <Sheet>
                            <SheetTrigger :as-child="true">
                                <button class="text-muted-foreground hover:text-foreground">
                                    <Menu class="size-5" />
                                </button>
                            </SheetTrigger>
                            <SheetContent side="left" class="w-[300px] p-6">
                                <SheetTitle class="sr-only">Navigation Menu</SheetTitle>
                                <SheetHeader class="flex justify-start text-left">
                                    <AppLogo />
                                </SheetHeader>
                                <div class="flex h-full flex-1 flex-col justify-between space-y-4 py-6">
                                    <nav class="-mx-2 space-y-1">
                                        <template v-for="item in [...mainNavItems, documentationItem]" :key="item.title">
                                            <Link
                                                v-if="!item.isExternal"
                                                :href="item.href"
                                                class="flex items-center gap-3 rounded-md px-2 py-2 text-sm font-medium text-muted-foreground hover:bg-muted/50 hover:text-foreground"
                                                :class="{ 'bg-muted/50 text-foreground': isCurrentRoute(item.href) }"
                                                prefetch
                                            >
                                                <component v-if="item.icon" :is="item.icon" class="size-4" />
                                                {{ item.title }}
                                            </Link>
                                            <a
                                                v-else
                                                :href="item.href"
                                                target="_blank"
                                                rel="noopener noreferrer"
                                                class="flex items-center gap-3 rounded-md px-2 py-2 text-sm font-medium text-muted-foreground hover:bg-muted/50 hover:text-foreground"
                                            >
                                                <component v-if="item.icon" :is="item.icon" class="size-4" />
                                                {{ item.title }}
                                            </a>
                                        </template>
                                        <Link
                                            v-for="pinned in pinnedMaps"
                                            :key="`pinned-${pinned.id}`"
                                            :href="MapController.show(pinned.slug)"
                                            class="flex items-center gap-3 rounded-md px-2 py-2 text-sm font-medium text-muted-foreground hover:bg-muted/50 hover:text-foreground"
                                            :class="{ 'bg-muted/50 text-foreground': isCurrentRoute(MapController.show(pinned.slug).url) }"
                                            prefetch
                                        >
                                            <MapIcon class="size-4" />
                                            <span class="truncate">{{ pinned.name }}</span>
                                        </Link>
                                    </nav>
                                    <div class="flex flex-col space-y-4">
                                        <div v-if="user" class="border-t border-border/50 pt-4">
                                            <Link
                                                :href="SettingsController.show({ query: { section: 'esi' } })"
                                                class="flex items-center gap-3 text-sm font-medium text-muted-foreground hover:text-foreground"
                                                :class="{ 'text-red-400': page.props.missing_scopes.length }"
                                                prefetch
                                            >
                                                <AlertTriangle v-if="page.props.missing_scopes.length" class="size-4" />
                                                <Shield v-else class="size-4" />
                                                <span>
                                                    {{
                                                        page.props.missing_scopes.length
                                                            ? `${page.props.missing_scopes.length} Missing Scopes`
                                                            : 'Manage Scopes'
                                                    }}
                                                </span>
                                            </Link>
                                        </div>

                                        <div class="border-t border-border/50 pt-4">
                                            <div class="mb-2 text-xs text-muted-foreground">Server Status</div>
                                            <ServerStatus />
                                        </div>
                                    </div>
                                </div>
                            </SheetContent>
                        </Sheet>
                    </div>

                    <Link :href="home()" class="flex items-center gap-x-2" prefetch>
                        <AppLogo />
                    </Link>

                    <!-- Divider -->
                    <div class="hidden h-6 border-l border-border/50 lg:block"></div>

                    <!-- Desktop Menu -->
                    <nav class="hidden min-w-0 flex-1 items-center gap-1 overflow-hidden lg:flex">
                        <template v-for="(item, index) in mainNavItems" :key="index">
                            <Link
                                v-if="!item.isExternal"
                                :href="item.href"
                                class="flex items-center gap-2 rounded-md px-3 py-1.5 text-xs font-medium text-muted-foreground transition-colors hover:bg-muted/50 hover:text-foreground"
                                :class="{ 'bg-muted/50 text-foreground': isCurrentRoute(item.href) }"
                                prefetch
                            >
                                <component v-if="item.icon" :is="item.icon" class="size-4" />
                                {{ item.title }}
                            </Link>
                            <a
                                v-else
                                :href="item.href"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="flex items-center gap-2 rounded-md px-3 py-1.5 text-xs font-medium text-muted-foreground transition-colors hover:bg-muted/50 hover:text-foreground"
                            >
                                <component v-if="item.icon" :is="item.icon" class="size-4" />
                                {{ item.title }}
                            </a>
                        </template>
                        <Link
                            v-for="pinned in inlinePinnedMaps"
                            :key="`pinned-${pinned.id}`"
                            :href="MapController.show(pinned.slug)"
                            class="flex max-w-44 min-w-0 shrink items-center gap-2 rounded-md px-3 py-1.5 text-xs font-medium text-muted-foreground transition-colors hover:bg-muted/50 hover:text-foreground"
                            :class="{ 'bg-muted/50 text-foreground': isCurrentRoute(MapController.show(pinned.slug).url) }"
                            prefetch
                        >
                            <MapIcon class="size-3.5 shrink-0" />
                            <span class="truncate">{{ pinned.name }}</span>
                        </Link>
                        <DropdownMenu v-if="overflowPinnedMaps.length">
                            <DropdownMenuTrigger as-child>
                                <button
                                    class="flex shrink-0 items-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs font-medium text-muted-foreground transition-colors hover:bg-muted/50 hover:text-foreground"
                                >
                                    <MapIcon class="size-3.5" />
                                    +{{ overflowPinnedMaps.length }}
                                </button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="start" class="w-56">
                                <DropdownMenuItem v-for="pinned in overflowPinnedMaps" :key="`pinned-overflow-${pinned.id}`" as-child>
                                    <Link :href="MapController.show(pinned.slug)" prefetch class="flex w-full items-center gap-2">
                                        <MapIcon class="size-4" />
                                        <span class="truncate">{{ pinned.name }}</span>
                                    </Link>
                                </DropdownMenuItem>
                            </DropdownMenuContent>
                        </DropdownMenu>
                    </nav>
                </div>

                <!-- Center: Server Status (desktop only) -->
                <div class="hidden justify-center md:flex">
                    <ServerStatus />
                </div>

                <!-- Right: Actions -->
                <div class="flex items-center justify-end gap-4 md:col-start-3">
                    <!-- Documentation -->
                    <Link
                        :href="documentationItem.href"
                        class="hidden items-center gap-2 text-xs font-medium text-muted-foreground transition-colors hover:text-foreground lg:flex"
                        :class="{ 'text-foreground': isCurrentRoute(documentationItem.href) }"
                        prefetch
                    >
                        <CircleHelp class="size-4" />
                        <span>Documentation</span>
                    </Link>

                    <!-- Divider -->
                    <div class="hidden h-6 border-l border-border/50 lg:block"></div>

                    <!-- Scopes status -->
                    <template v-if="user">
                        <Link
                            :href="SettingsController.show({ query: { section: 'esi' } })"
                            class="hidden items-center gap-2 text-xs font-medium transition-colors sm:flex"
                            :class="
                                page.props.missing_scopes.length ? 'text-red-400 hover:text-red-300' : 'text-muted-foreground hover:text-foreground'
                            "
                            prefetch
                        >
                            <AlertTriangle v-if="page.props.missing_scopes.length" class="size-4" />
                            <Shield v-else class="size-4" />
                            <span>
                                {{ page.props.missing_scopes.length ? `${page.props.missing_scopes.length} Missing` : 'Scopes' }}
                            </span>
                        </Link>
                    </template>

                    <!-- Divider -->
                    <div class="hidden h-6 border-l border-border/50 sm:block"></div>

                    <Appearance />

                    <!-- Divider -->
                    <div class="hidden h-6 border-l border-border/50 sm:block"></div>

                    <!-- User menu -->
                    <template v-if="user">
                        <DropdownMenu>
                            <DropdownMenuTrigger :as-child="true">
                                <button class="flex cursor-pointer items-center gap-2.5 rounded-md px-2 py-1.5 transition-colors hover:bg-muted/50">
                                    <CharacterImage
                                        v-if="user.active_character"
                                        :character_id="user.active_character.id"
                                        :character_name="user.active_character.name"
                                        class="size-6 rounded"
                                    />
                                    <span class="hidden max-w-28 truncate text-xs font-medium text-muted-foreground sm:block">
                                        {{ user.active_character?.name }}
                                    </span>
                                </button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="end" class="w-56">
                                <UserMenuContent :user="user" />
                            </DropdownMenuContent>
                        </DropdownMenu>
                    </template>
                    <template v-else>
                        <Link
                            :href="LoginController.show()"
                            class="flex items-center gap-2 rounded-md px-3 py-1.5 text-xs font-medium text-muted-foreground transition-colors hover:bg-muted/50 hover:text-foreground"
                            prefetch
                        >
                            <LogIn class="size-4" />
                            <span class="hidden sm:inline">Login</span>
                        </Link>
                    </template>
                </div>
            </div>
        </div>
    </div>
</template>
