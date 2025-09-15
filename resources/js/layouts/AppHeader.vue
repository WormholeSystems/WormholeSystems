<script setup lang="ts">
import MapController from '@/actions/App/Http/Controllers/MapController';
import ScopeController from '@/actions/App/Http/Controllers/ScopeController';
import DiscordIcon from '@/components/icons/DiscordIcon.vue';
import { CharacterImage } from '@/components/images';
import ServerStatus from '@/components/server-status/ServerStatus.vue';
import { Button } from '@/components/ui/button';
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { NavigationMenu, NavigationMenuItem, NavigationMenuList, navigationMenuTriggerStyle } from '@/components/ui/navigation-menu';
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from '@/components/ui/sheet';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import UserMenuContent from '@/components/user/UserMenuContent.vue';
import useUser from '@/composables/useUser';
import Appearance from '@/layouts/Appearance.vue';
import AppLogo from '@/layouts/AppLogo.vue';
import { home } from '@/routes';
import type { AppPageProps, NavItem } from '@/types';
import { TCharacter } from '@/types/models';
import { Link, usePage } from '@inertiajs/vue3';
import { AlertTriangle, LayoutGrid, Menu, Shield } from 'lucide-vue-next';
import { computed } from 'vue';

const page = usePage<
    AppPageProps<{
        missing_scopes: TCharacter[];
    }>
>();
const user = useUser();

const isCurrentRoute = computed(() => (url: string) => page.url === url);

const activeItemStyles = computed(
    () => (url: string) => (isCurrentRoute.value(url) ? 'text-neutral-900 dark:bg-neutral-800 dark:text-neutral-100' : ''),
);

const mainNavItems: NavItem[] = [
    {
        title: 'Maps',
        href: MapController.index().url,
        icon: LayoutGrid,
    },
    {
        title: 'Discord',
        href: page.props.discord.invite,
        icon: DiscordIcon,
        isExternal: true,
    },
];
</script>

<template>
    <div>
        <div class="border-b border-sidebar-border/80">
            <div class="grid h-16 grid-cols-[1fr_auto] items-center px-4 md:grid-cols-3">
                <!-- Left: Mobile menu + Logo + Desktop nav -->
                <div class="flex items-center gap-4">
                    <!-- Mobile Menu -->
                    <div class="lg:hidden">
                        <Sheet>
                            <SheetTrigger :as-child="true">
                                <Button variant="ghost" size="icon" class="h-9 w-9">
                                    <Menu class="h-5 w-5" />
                                </Button>
                            </SheetTrigger>
                            <SheetContent side="left" class="w-[300px] p-6">
                                <SheetTitle class="sr-only">Navigation Menu</SheetTitle>
                                <SheetHeader class="flex justify-start text-left">
                                    <AppLogo />
                                </SheetHeader>
                                <div class="flex h-full flex-1 flex-col justify-between space-y-4 py-6">
                                    <nav class="-mx-3 space-y-1">
                                        <template v-for="item in mainNavItems" :key="item.title">
                                            <Link
                                                v-if="!item.isExternal"
                                                :href="item.href"
                                                class="flex items-center gap-x-3 rounded-lg px-3 py-2 text-sm font-medium hover:bg-accent"
                                                :class="activeItemStyles(item.href)"
                                            >
                                                <component v-if="item.icon" :is="item.icon" class="h-5 w-5" />
                                                {{ item.title }}
                                            </Link>
                                            <a
                                                v-else
                                                :href="item.href"
                                                target="_blank"
                                                rel="noopener noreferrer"
                                                class="flex items-center gap-x-3 rounded-lg px-3 py-2 text-sm font-medium hover:bg-accent"
                                            >
                                                <component v-if="item.icon" :is="item.icon" class="h-5 w-5" />
                                                {{ item.title }}
                                            </a>
                                        </template>
                                    </nav>
                                    <div class="flex flex-col space-y-4">
                                        <!-- Scopes status for mobile -->
                                        <div v-if="user" class="border-t border-sidebar-border/50 pt-4">
                                            <Button
                                                :variant="page.props.missing_scopes.length ? 'destructive' : 'ghost'"
                                                size="sm"
                                                as-child
                                                class="w-full justify-start"
                                            >
                                                <Link :href="ScopeController.index()" class="flex items-center space-x-3">
                                                    <AlertTriangle v-if="page.props.missing_scopes.length" class="h-4 w-4" />
                                                    <Shield v-else class="h-4 w-4" />
                                                    <span class="text-sm font-medium">
                                                        {{
                                                            page.props.missing_scopes.length
                                                                ? `Missing Scopes (${page.props.missing_scopes.length})`
                                                                : 'Manage Scopes'
                                                        }}
                                                    </span>
                                                </Link>
                                            </Button>
                                        </div>

                                        <div class="border-t border-sidebar-border/50 pt-4">
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

                    <!-- Desktop Menu -->
                    <div class="hidden h-full lg:flex">
                        <NavigationMenu class="flex h-full items-stretch">
                            <NavigationMenuList class="flex h-full items-stretch space-x-2">
                                <NavigationMenuItem v-for="(item, index) in mainNavItems" :key="index" class="relative flex h-full items-center">
                                    <Link
                                        v-if="!item.isExternal"
                                        :class="[navigationMenuTriggerStyle(), activeItemStyles(item.href), 'h-9 cursor-pointer px-3']"
                                        :href="item.href"
                                    >
                                        <component v-if="item.icon" :is="item.icon" class="mr-2 h-4 w-4" />
                                        {{ item.title }}
                                    </Link>
                                    <a
                                        v-else
                                        :class="[navigationMenuTriggerStyle(), activeItemStyles(item.href), 'h-9 cursor-pointer px-3']"
                                        :href="item.href"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                    >
                                        <component v-if="item.icon" :is="item.icon" class="mr-2 h-4 w-4" />
                                        {{ item.title }}
                                    </a>
                                    <div
                                        v-if="isCurrentRoute(item.href)"
                                        class="absolute bottom-0 left-0 h-0.5 w-full translate-y-px bg-black dark:bg-white"
                                    ></div>
                                </NavigationMenuItem>
                            </NavigationMenuList>
                        </NavigationMenu>
                    </div>
                </div>

                <!-- Center: Server Status (desktop only) -->
                <div class="hidden justify-center md:flex">
                    <ServerStatus />
                </div>

                <!-- Right: Actions -->
                <div class="flex items-center justify-end space-x-1 sm:space-x-2 md:col-start-3">
                    <!-- Scopes status button -->
                    <template v-if="user">
                        <TooltipProvider :delay-duration="0">
                            <Tooltip>
                                <TooltipTrigger as-child>
                                    <Button :variant="page.props.missing_scopes.length ? 'destructive' : 'outline'" size="icon" as-child>
                                        <Link :href="ScopeController.index()">
                                            <AlertTriangle v-if="page.props.missing_scopes.length" class="h-4 w-4" />
                                            <Shield v-else class="h-4 w-4" />
                                            <span class="sr-only">
                                                {{ page.props.missing_scopes.length ? 'Missing ESI scopes' : 'Manage ESI scopes' }}
                                            </span>
                                        </Link>
                                    </Button>
                                </TooltipTrigger>
                                <TooltipContent>
                                    <template v-if="page.props.missing_scopes.length">
                                        <p class="font-medium">
                                            {{ page.props.missing_scopes.length }} character{{ page.props.missing_scopes.length === 1 ? '' : 's' }}
                                            missing scopes for some features
                                        </p>
                                        <p class="mt-1 text-xs text-muted-foreground">Click to manage ESI scopes</p>
                                    </template>
                                    <template v-else>
                                        <p>Manage ESI scopes</p>
                                    </template>
                                </TooltipContent>
                            </Tooltip>
                        </TooltipProvider>
                    </template>
                    <Appearance />

                    <!-- User menu -->
                    <template v-if="user">
                        <DropdownMenu>
                            <DropdownMenuTrigger :as-child="true">
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    class="relative h-9 w-9 rounded-lg p-1 focus-within:ring-2 focus-within:ring-primary"
                                >
                                    <CharacterImage
                                        :character_id="user.active_character.id"
                                        :character_name="user.active_character.name"
                                        class="h-7 w-7 rounded-lg"
                                    />
                                </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="end" class="w-56">
                                <UserMenuContent :user="user" />
                            </DropdownMenuContent>
                        </DropdownMenu>
                    </template>
                </div>
            </div>
        </div>
    </div>
</template>
