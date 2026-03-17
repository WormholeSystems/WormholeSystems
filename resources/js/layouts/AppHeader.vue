<script setup lang="ts">
import AboutController from '@/actions/App/Http/Controllers/AboutController';
import LoginController from '@/actions/App/Http/Controllers/LoginController';
import MapController from '@/actions/App/Http/Controllers/MapController';
import ScopeController from '@/actions/App/Http/Controllers/ScopeController';
import { CharacterImage } from '@/components/images';
import ServerStatus from '@/components/server-status/ServerStatus.vue';
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from '@/components/ui/sheet';
import UserMenuContent from '@/components/user/UserMenuContent.vue';
import useUser from '@/composables/useUser';
import Appearance from '@/layouts/Appearance.vue';
import AppLogo from '@/layouts/AppLogo.vue';
import { home } from '@/routes';
import type { AppPageProps, NavItem } from '@/types';
import { TCharacter } from '@/types/models';
import { Link, usePage } from '@inertiajs/vue3';
import { AlertTriangle, Info, LayoutGrid, LogIn, Menu, Shield } from 'lucide-vue-next';
import { computed } from 'vue';

const page = usePage<
    AppPageProps<{
        missing_scopes: TCharacter[];
    }>
>();
const user = useUser();

const isCurrentRoute = computed(() => (url: string) => page.url === url);

const mainNavItems: NavItem[] = [
    {
        title: 'Maps',
        href: MapController.index().url,
        icon: LayoutGrid,
    },
    {
        title: 'About',
        href: AboutController.index().url,
        icon: Info,
    },
];
</script>

<template>
    <div>
        <div class="border-b border-border/50 bg-card">
            <div class="grid h-16 grid-cols-[1fr_auto] items-center px-4 md:grid-cols-3">
                <!-- Left: Mobile menu + Logo + Divider + Desktop nav -->
                <div class="flex items-center gap-4">
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
                                        <template v-for="item in mainNavItems" :key="item.title">
                                            <Link
                                                v-if="!item.isExternal"
                                                :href="item.href"
                                                class="flex items-center gap-3 rounded-md px-2 py-2 text-sm font-medium text-muted-foreground hover:bg-muted/50 hover:text-foreground"
                                                :class="{ 'bg-muted/50 text-foreground': isCurrentRoute(item.href) }"
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
                                    </nav>
                                    <div class="flex flex-col space-y-4">
                                        <div v-if="user" class="border-t border-border/50 pt-4">
                                            <Link
                                                :href="ScopeController.index()"
                                                class="flex items-center gap-3 text-sm font-medium text-muted-foreground hover:text-foreground"
                                                :class="{ 'text-red-400': page.props.missing_scopes.length }"
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

                    <Link :href="home()" class="flex items-center gap-x-2">
                        <AppLogo />
                    </Link>

                    <!-- Divider -->
                    <div class="hidden h-6 border-l border-border/50 lg:block"></div>

                    <!-- Desktop Menu -->
                    <nav class="hidden items-center gap-1 lg:flex">
                        <template v-for="(item, index) in mainNavItems" :key="index">
                            <Link
                                v-if="!item.isExternal"
                                :href="item.href"
                                class="flex items-center gap-2 rounded-md px-3 py-1.5 text-xs font-medium text-muted-foreground transition-colors hover:bg-muted/50 hover:text-foreground"
                                :class="{ 'bg-muted/50 text-foreground': isCurrentRoute(item.href) }"
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
                    </nav>
                </div>

                <!-- Center: Server Status (desktop only) -->
                <div class="hidden justify-center md:flex">
                    <ServerStatus />
                </div>

                <!-- Right: Actions -->
                <div class="flex items-center justify-end gap-4 md:col-start-3">
                    <!-- Scopes status -->
                    <template v-if="user">
                        <Link
                            :href="ScopeController.index()"
                            class="hidden items-center gap-2 text-xs font-medium transition-colors sm:flex"
                            :class="
                                page.props.missing_scopes.length ? 'text-red-400 hover:text-red-300' : 'text-muted-foreground hover:text-foreground'
                            "
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
                                        :character_id="user.active_character.id"
                                        :character_name="user.active_character.name"
                                        class="size-6 rounded"
                                    />
                                    <span class="hidden max-w-28 truncate text-xs font-medium text-muted-foreground sm:block">
                                        {{ user.active_character.name }}
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
                            :href="LoginController.show().url"
                            class="flex items-center gap-2 rounded-md px-3 py-1.5 text-xs font-medium text-muted-foreground transition-colors hover:bg-muted/50 hover:text-foreground"
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
